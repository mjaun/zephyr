// Copyright (c) 2024 Zühlke Engineering AG
// SPDX-License-Identifier: Apache-2.0

use alloc::boxed::Box;
use alloc::ffi::CString;
use alloc::string::String;
use core::ffi::{c_char, c_int, c_uint, c_void, CStr};
use core::ptr;
use core::time::Duration;
use crate::errno::{check_ptr, check_ptr_mut, check_result, Errno, ZephyrResult};

#[macro_export]
macro_rules! printk {
    ($msg:expr) => {
        crate::kernel::printk($msg);
    };
    ($fmt:expr, $($arg:tt)*) => {
        crate::kernel::printk(::alloc::format!($fmt, $($arg)*).as_str());
    };
}

pub fn printk(msg: &str) {
    let cstring = CString::new(msg).unwrap();

    unsafe {
        zephyr_sys::printk("%s\0".as_ptr() as *const c_char, cstring.as_ptr() as *const c_char);
    }
}

pub fn msleep(ms: i32) -> i32 {
    unsafe { zephyr_sys::k_msleep(ms) }
}

pub fn rand(range: core::ops::Range<u32>) -> u32 {
    let range_size = range.end - range.start;
    let random_value = unsafe { zephyr_sys::sys_rand32_get() };
    range.start + (random_value % range_size)
}

const FOREVER: zephyr_sys::k_timeout_t = zephyr_sys::k_timeout_t { ticks: -1 as zephyr_sys::k_ticks_t };
const NO_WAIT: zephyr_sys::k_timeout_t = zephyr_sys::k_timeout_t { ticks: 0 };

fn into_timeout(value: Duration) -> zephyr_sys::k_timeout_t {
    if value == Duration::MAX {
        return FOREVER;
    }
    if value == Duration::ZERO {
        return NO_WAIT;
    }

    const NANOS_PER_SEC: u128 = 1_000_000_000;
    const TICKS_PER_SEC: u128 = crate::kconfig::CONFIG_SYS_CLOCK_TICKS_PER_SEC as u128;

    let ticks = (value.as_nanos() * TICKS_PER_SEC).div_ceil(NANOS_PER_SEC);
    zephyr_sys::k_timeout_t { ticks: ticks as zephyr_sys::k_ticks_t }
}

#[repr(u32)]
enum Kobject {
    Thread = zephyr_sys::k_objects_K_OBJ_THREAD,
    Mutex = zephyr_sys::k_objects_K_OBJ_MUTEX,
    Semaphore = zephyr_sys::k_objects_K_OBJ_SEM,
}

trait KobjectType {
    fn get_enum() -> Kobject;
}

impl KobjectType for zephyr_sys::k_thread {
    fn get_enum() -> Kobject {
        Kobject::Thread
    }
}

impl KobjectType for zephyr_sys::k_mutex {
    fn get_enum() -> Kobject {
        Kobject::Mutex
    }
}

impl KobjectType for zephyr_sys::k_sem {
    fn get_enum() -> Kobject {
        Kobject::Semaphore
    }
}

unsafe fn object_alloc<T: KobjectType>() -> ZephyrResult<*mut T> {
    check_ptr_mut(zephyr_sys::k_object_alloc(T::get_enum() as c_uint) as *mut T, Errno::ENOMEM)
}

unsafe fn object_free<T: KobjectType>(obj: *mut T) {
    zephyr_sys::k_object_free(obj as *mut c_void);
}

/*
unsafe fn object_release<T: KobjectType>(obj: *mut T) {
    zephyr_sys::k_object_release(obj as *const c_void);
}
*/

unsafe fn thread_stack_alloc(size: usize, flags: u32) -> ZephyrResult<*mut zephyr_sys::k_thread_stack_t> {
    check_ptr_mut(zephyr_sys::k_thread_stack_alloc(size, flags as c_int), Errno::ENOMEM)
}

unsafe fn thread_stack_free(thread_stack: *mut zephyr_sys::k_thread_stack_t) {
    zephyr_sys::k_thread_stack_free(thread_stack);
}

pub enum ThreadMode {
    Supervisor,
    User,
}

pub struct Thread {
    tid: zephyr_sys::k_tid_t,
    thread: *mut zephyr_sys::k_thread,
    thread_stack: *mut zephyr_sys::k_thread_stack_t,
}

impl Thread {
    const K_USER: u32 = 1 << 2;
    const K_INHERIT_PERMS: u32 = 1 << 3;

    pub fn new<F>(entry_point: F, stack_size: usize, priority: i32, mode: ThreadMode, delay: Duration) -> ZephyrResult<Self>
    where
        F: FnOnce() + 'static,
        F: Send,
    {
        let boxed_closure = Box::new(Box::new(entry_point) as Box<dyn FnOnce()>);
        let closure_ptr = Box::into_raw(boxed_closure) as *mut c_void;

        unsafe {
            let thread = object_alloc()?;

            let stack_flags = match mode {
                ThreadMode::Supervisor => 0,
                ThreadMode::User => Self::K_USER,
            };

            let thread_flags = match mode {
                ThreadMode::Supervisor => Self::K_INHERIT_PERMS,
                ThreadMode::User => Self::K_USER | Self::K_INHERIT_PERMS,
            };

            let thread_stack = match thread_stack_alloc(stack_size, stack_flags) {
                Ok(thread_stack) => Ok(thread_stack),
                Err(errno) => {
                    object_free(thread);
                    Err(errno)
                }
            }?;

            let tid = zephyr_sys::k_thread_create(
                thread,
                thread_stack,
                stack_size,
                Some(rust_thread_entry),
                closure_ptr,
                ptr::null_mut(),
                ptr::null_mut(),
                priority,
                thread_flags,
                into_timeout(delay),
            );

            Ok(Thread { tid, thread, thread_stack })
        }
    }

    pub fn start(&self) {
        unsafe {
            zephyr_sys::k_thread_start(self.tid);
        }
    }

    pub fn join(&self, timeout: Duration) -> ZephyrResult<()> {
        unsafe {
            check_result(zephyr_sys::k_thread_join(self.tid, into_timeout(timeout)))
        }
    }

    pub fn abort(&self) {
        unsafe {
            zephyr_sys::k_thread_abort(self.tid);
        }
    }

    pub fn name_set(&self, name: &str) -> ZephyrResult<()> {
        unsafe {
            let cname = CString::new(name).unwrap();
            check_result(zephyr_sys::k_thread_name_set(self.tid, cname.as_ptr()))
        }
    }

    pub fn name_get(&self) -> ZephyrResult<String> {
        unsafe {
            let name_ptr = check_ptr(zephyr_sys::k_thread_name_get(self.tid), Errno::ENOTSUP)?;
            Ok(String::from(CStr::from_ptr(name_ptr).to_str().unwrap()))
        }
    }

    pub fn user_mode_enter<F>(entry_point: F) -> !
    where
        F: FnOnce() + 'static,
        F: Send,
    {
        let boxed_closure = Box::new(Box::new(entry_point) as Box<dyn FnOnce()>);
        let closure_ptr = Box::into_raw(boxed_closure) as *mut c_void;

        unsafe {
            zephyr_sys::k_thread_user_mode_enter(
                Some(rust_thread_entry),
                closure_ptr,
                ptr::null_mut(),
                ptr::null_mut()
            )
        }
    }
}

impl Drop for Thread {
    fn drop(&mut self) {
        self.abort();

        unsafe {
            object_free(self.thread);
            thread_stack_free(self.thread_stack);
        }
    }
}

extern fn rust_thread_entry(closure_ptr: *mut c_void, _p2: *mut c_void, _p3: *mut c_void) {
    let closure_box: Box<Box<dyn FnOnce()>> = unsafe {
        Box::from_raw(closure_ptr as *mut Box<dyn FnOnce()>)
    };

    closure_box();
}

pub struct Mutex {
    mutex: *mut zephyr_sys::k_mutex,
}

impl Mutex {
    pub fn new() -> ZephyrResult<Self> {
        unsafe {
            let mutex = object_alloc()?;
            zephyr_sys::k_mutex_init(mutex);
            Ok(Mutex { mutex })
        }
    }

    pub fn lock(&self, timeout: Duration) -> ZephyrResult<()> {
        unsafe {
            check_result(zephyr_sys::k_mutex_lock(self.mutex, into_timeout(timeout)))
        }
    }

    pub fn unlock(&self) -> ZephyrResult<()> {
        unsafe {
            check_result(zephyr_sys::k_mutex_unlock(self.mutex))
        }
    }
}

unsafe impl Send for Mutex {}
unsafe impl Sync for Mutex {}

impl Drop for Mutex {
    fn drop(&mut self) {
        unsafe {
            object_free(self.mutex);
        }
    }
}

pub struct Semaphore {
    sem: *mut zephyr_sys::k_sem,
}

impl Semaphore {
    pub fn new(limit: u32, initial_count: u32) -> ZephyrResult<Self> {
        unsafe {
            let sem = object_alloc()?;
            zephyr_sys::k_sem_init(sem, limit, initial_count);
            Ok(Semaphore { sem })
        }
    }

    pub fn take(&self, timeout: Duration) -> ZephyrResult<()> {
        unsafe {
            check_result(zephyr_sys::k_sem_take(self.sem, into_timeout(timeout)))
        }
    }

    pub fn give(&self) {
        unsafe {
            zephyr_sys::k_sem_give(self.sem);
        }
    }

    pub fn reset(&self) {
        unsafe {
            zephyr_sys::k_sem_reset(self.sem);
        }
    }

    pub fn count_get(&self) -> u32 {
        unsafe {
            zephyr_sys::k_sem_count_get(self.sem)
        }
    }
}

unsafe impl Send for Semaphore {}
unsafe impl Sync for Semaphore {}
