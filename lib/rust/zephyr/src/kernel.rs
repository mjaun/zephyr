// Copyright (c) 2024 Zühlke Engineering AG
// SPDX-License-Identifier: Apache-2.0

use alloc::boxed::Box;
use alloc::ffi::CString;
use core::ffi::{c_char, c_void};
use core::mem::{align_of, size_of};
use core::ptr;
use core::time::Duration;
use crate::errno::Errno;

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

pub struct Thread {
    tid: zephyr_sys::k_tid_t,
    thread: *mut zephyr_sys::k_thread,
    thread_stack: *mut zephyr_sys::k_thread_stack_t,
}

impl Thread {
    pub fn new<F>(entry_point: F, stack_size: usize, priority: i32, delay: Duration) -> Result<Thread, Errno>
    where
        F: FnOnce() + 'static,
        F: Send,
    {
        let boxed_closure = Box::new(Box::new(entry_point) as Box<dyn FnOnce()>);
        let closure_ptr = Box::into_raw(boxed_closure) as *mut c_void;

        unsafe {
            let thread = zephyr_sys::k_aligned_alloc(
                align_of::<zephyr_sys::k_thread>(),
                size_of::<zephyr_sys::k_thread>()
            ) as *mut zephyr_sys::k_thread;

            if thread == ptr::null_mut() {
                return Err(Errno::ENOMEM);
            }

            let thread_stack = zephyr_sys::k_thread_stack_alloc(stack_size, 0);

            if thread_stack == ptr::null_mut() {
                zephyr_sys::k_free(thread as *mut c_void);
                return Err(Errno::ENOMEM);
            }

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

    pub fn join(&self, timeout: Duration) -> Result<(), Errno> {
        unsafe {
            Errno::from(zephyr_sys::k_thread_join(self.tid, into_timeout(timeout)))
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
            zephyr_sys::k_free(self.thread as *mut c_void);
            zephyr_sys::k_thread_stack_free(self.thread_stack);
        }
    }
}

extern fn rust_thread_entry(closure_ptr: *mut c_void, _p2: *mut c_void, _p3: *mut c_void) {
    let closure_box: Box<Box<dyn FnOnce()>> = unsafe {
        Box::from_raw(closure_ptr as *mut Box<dyn FnOnce()>)
    };

    closure_box();
}
