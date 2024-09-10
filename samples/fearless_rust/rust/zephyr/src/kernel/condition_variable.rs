use alloc::alloc::{alloc, dealloc};
use core::alloc::Layout;
use core::time::Duration;

use crate::kernel::errno::{check_result, check_value};
use crate::kernel::mutex::MutexGuard;

pub struct ConditionVariable {
    condvar: *mut crate::sys::k_condvar,
}

impl ConditionVariable {
    pub fn new() -> Self {
        unsafe {
            let condvar = alloc(Layout::new::<crate::sys::k_condvar>()) as *mut crate::sys::k_condvar;
            check_result(crate::sys::k_condvar_init(condvar)).unwrap();
            Self { condvar }
        }
    }

    pub fn wait<'a, T>(&self, guard: MutexGuard<'a, T>, timeout: Duration) -> WaitResult<'a, T> {
        unsafe {
            let ret = crate::sys::k_condvar_wait(self.condvar, guard.mutex_ptr(), timeout.into());

            match check_result(ret) {
                Ok(_) => Ok(guard),
                Err(errno) => Err(WaitError { errno, guard }),
            }
        }
    }

    pub fn signal(&self) {
        unsafe {
            check_result(crate::sys::k_condvar_signal(self.condvar)).unwrap()
        }
    }

    pub fn broadcast(&self) -> u32 {
        unsafe {
            check_value(crate::sys::k_condvar_broadcast(self.condvar)).unwrap()
        }
    }
}

impl Drop for ConditionVariable {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.condvar as *mut u8, Layout::new::<crate::sys::k_condvar>());
        }
    }
}

pub struct WaitError<'a, T> {
    errno: u32,
    guard: MutexGuard<'a, T>
}

impl<'a, T> WaitError<'a, T> {
    pub fn errno(&self) -> u32 { self.errno }
    pub fn into_inner(self) -> MutexGuard<'a, T> { self.guard }
}

pub type WaitResult<'a, T> = Result<MutexGuard<'a, T>, WaitError<'a, T>>;
