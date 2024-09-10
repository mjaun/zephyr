use alloc::alloc::{alloc, dealloc};
use core::alloc::Layout;
use core::time::Duration;
use crate::kernel::errno::{check_result, ErrnoResult};

pub struct Semaphore {
    sem: *mut crate::sys::k_sem,
}

impl Semaphore {
    pub fn new(initial_count: u32, limit: u32) -> Self {
        unsafe {
            let sem = alloc(Layout::new::<crate::sys::k_sem>()) as *mut crate::sys::k_sem;
            crate::sys::k_sem_init(sem, initial_count, limit);
            Self { sem }
        }
    }

    pub fn take(&self, timeout: Duration) -> ErrnoResult<()> {
        unsafe {
            check_result(crate::sys::k_sem_take(self.sem, timeout.into()))
        }
    }

    pub fn give(&self) {
        unsafe {
            crate::sys::k_sem_give(self.sem);
        }
    }

    pub fn reset(&self) {
        unsafe {
            crate::sys::k_sem_reset(self.sem);
        }
    }
}

impl Drop for Semaphore {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.sem as *mut u8, Layout::new::<crate::sys::k_sem>());
        }
    }
}