use alloc::boxed::Box;
use core::mem::MaybeUninit;
use core::time::Duration;
use crate::kernel::errno::{check_result, ErrnoResult};

pub struct Semaphore {
    sem: *mut crate::sys::k_sem,
}

impl Semaphore {
    pub fn new(initial_count: u32, limit: u32) -> Self {
        unsafe {
            let sem_box: Box<MaybeUninit<crate::sys::k_sem>> = Box::new(MaybeUninit::uninit());
            let sem_ptr = (*Box::into_raw(sem_box)).as_mut_ptr();

            crate::sys::k_sem_init(sem_ptr, initial_count, limit);
            Self { sem: sem_ptr }
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
            // turn into box to delete
            let _ = Box::from_raw(self.sem);
        }
    }
}