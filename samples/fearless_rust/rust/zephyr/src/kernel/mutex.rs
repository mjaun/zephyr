use alloc::alloc::{alloc, dealloc};
use core::alloc::Layout;
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::time::Duration;
use crate::kernel::errno::{check_result, ErrnoResult};

pub struct Mutex<T> {
    mutex: *mut crate::sys::k_mutex,
    data: UnsafeCell<T>,
}

impl<T> Mutex<T> {
    pub fn new(data: T) -> Self {
        unsafe {
            let mutex = alloc(Layout::new::<crate::sys::k_mutex>()) as *mut crate::sys::k_mutex;
            crate::sys::k_mutex_init(mutex);
            Self { mutex, data: UnsafeCell::new(data) }
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
            dealloc(self.mutex as *mut u8, Layout::new::<crate::sys::k_mutex>());
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
