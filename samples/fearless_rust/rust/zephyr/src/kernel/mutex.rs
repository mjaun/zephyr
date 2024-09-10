use alloc::boxed::Box;
use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut};
use core::time::Duration;
use crate::kernel::errno::{check_result, ErrnoResult};

pub struct Mutex<T> {
    data: UnsafeCell<T>,
    mutex: *mut crate::sys::k_mutex,
}

impl<T> Mutex<T> {
    pub fn new(value: T) -> Self {
        unsafe {
            let mutex_box: Box<MaybeUninit<crate::sys::k_mutex>> = Box::new(MaybeUninit::uninit());
            let mutex_ptr = (*Box::into_raw(mutex_box)).as_mut_ptr();

            crate::sys::k_mutex_init(mutex_ptr);
            Self { data: UnsafeCell::new(value), mutex: mutex_ptr }
        }
    }

    pub fn lock(&self, timeout: Duration) -> LockResult<T> {
        unsafe {
            check_result(crate::sys::k_mutex_lock(self.mutex, timeout.into()))?;
            Ok(MutexGuard { mutex: &self })
        }
    }

    fn unlock(&self) -> ErrnoResult<()> {
        unsafe {
            check_result(crate::sys::k_mutex_unlock(self.mutex))
        }
    }
}

impl<T> Drop for Mutex<T> {
    fn drop(&mut self) {
        unsafe {
            // turn into box to delete
            let _ = Box::from_raw(self.mutex);
        }
    }
}

pub struct MutexGuard<'a, T> {
    mutex: &'a Mutex<T>,
}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &(*self.mutex.data.get()) }
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut (*self.mutex.data.get()) }
    }
}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        // unwrap() should be fine, because this call should only fail if the mutex was not locked
        self.mutex.unlock().unwrap()
    }
}

pub type LockResult<'a, T> = ErrnoResult<MutexGuard<'a, T>>;
