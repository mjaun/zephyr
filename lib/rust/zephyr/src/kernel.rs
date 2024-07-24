// Copyright (c) 2024 Zühlke Engineering AG
// SPDX-License-Identifier: Apache-2.0

use alloc::boxed::Box;
use alloc::ffi::CString;
use core::ffi::{c_char, c_void};
use core::mem::{align_of, size_of};
use core::ptr;
use core::time::Duration;
use crate::errno::{check_ptr, check_result, Errno, ZephyrResult};

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

unsafe fn kobj_alloc<T>() -> ZephyrResult<*mut T> {
    check_ptr(zephyr_sys::k_aligned_alloc(align_of::<T>(), size_of::<T>()) as *mut T, Errno::ENOMEM)
}

unsafe fn kobj_free<T>(obj: *mut T) {
    zephyr_sys::k_free(obj as *mut c_void);
}

unsafe fn thread_stack_alloc(size: usize) -> ZephyrResult<*mut zephyr_sys::k_thread_stack_t> {
    check_ptr(zephyr_sys::k_thread_stack_alloc(size, 0), Errno::ENOMEM)
}

unsafe fn thread_stack_free(thread_stack: *mut zephyr_sys::k_thread_stack_t) {
    zephyr_sys::k_thread_stack_free(thread_stack);
}

pub struct Thread {
    tid: zephyr_sys::k_tid_t,
    thread: *mut zephyr_sys::k_thread,
    thread_stack: *mut zephyr_sys::k_thread_stack_t,
}

impl Thread {
    pub fn new<F>(entry_point: F, stack_size: usize, priority: i32, delay: Duration) -> ZephyrResult<Self>
    where
        F: FnOnce() + 'static,
        F: Send,
    {
        let boxed_closure = Box::new(Box::new(entry_point) as Box<dyn FnOnce()>);
        let closure_ptr = Box::into_raw(boxed_closure) as *mut c_void;

        unsafe {
            let thread = kobj_alloc()?;

            let thread_stack = match thread_stack_alloc(stack_size) {
                Ok(thread_stack) => Ok(thread_stack),
                Err(errno) => {
                    kobj_free(thread);
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
                0,
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
}

impl Drop for Thread {
    fn drop(&mut self) {
        self.abort();

        unsafe {
            kobj_free(self.thread);
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
            let mutex = kobj_alloc()?;
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
            kobj_free(self.mutex);
        }
    }
}

pub struct Semaphore {
    sem: *mut zephyr_sys::k_sem,
}

impl Semaphore {
    pub fn new(limit: u32, initial_count: u32) -> ZephyrResult<Self> {
        unsafe {
            let sem = kobj_alloc()?;
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
