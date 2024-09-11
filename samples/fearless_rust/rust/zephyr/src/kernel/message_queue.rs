use alloc::alloc::{alloc, dealloc};
use core::alloc::Layout;
use core::ffi::{c_char, c_void};
use core::marker::PhantomData;
use core::mem::{MaybeUninit, size_of};
use core::time::Duration;
use crate::kernel::errno::{check_result, ErrnoResult};

pub struct MessageQueue<T: Copy> {
    msgq: *mut crate::sys::k_msgq,
    buffer: *mut u8,
    phantom: PhantomData<T>
}

impl<T: Copy> MessageQueue<T> {
    const MSG_SIZE: usize = size_of::<T>();

    pub fn new(max_msgs: usize) -> Self {
        unsafe {
            let buffer = alloc(Layout::from_size_align(Self::MSG_SIZE * max_msgs, 1).unwrap());
            let msgq = alloc(Layout::new::<crate::sys::k_msgq>()) as *mut crate::sys::k_msgq;

            crate::sys::k_msgq_init(msgq, buffer as *mut c_char, Self::MSG_SIZE, max_msgs as u32);

            MessageQueue { msgq, buffer, phantom: PhantomData }
        }
    }

    pub fn put(&self, msg: T) {
        self.try_put_for(msg, Duration::MAX).unwrap()
    }

    pub fn try_put(&self, msg: T) -> ErrnoResult<()> {
        self.try_put_for(msg, Duration::ZERO)
    }

    pub fn try_put_for(&self, msg: T, timeout: Duration) -> ErrnoResult<()> {
        unsafe {
            let msg_ptr = &msg as *const T as *const c_void;
            let ret = crate::sys::k_msgq_put(self.msgq, msg_ptr, timeout.into());
            check_result(ret)
        }
    }

    pub fn get(&self) -> T {
        self.try_get_for(Duration::MAX).unwrap()
    }

    pub fn try_get(&self) -> ErrnoResult<T> {
        self.try_get_for(Duration::ZERO)
    }

    pub fn try_get_for(&self, timeout: Duration) -> ErrnoResult<T> {
        unsafe {
            let mut msg: MaybeUninit<T> = MaybeUninit::uninit();
            let ret = crate::sys::k_msgq_get(self.msgq, msg.as_mut_ptr() as *mut c_void, timeout.into());

            check_result(ret)?;
            Ok(msg.assume_init())
        }
    }
}

impl<T: Copy> Drop for MessageQueue<T> {
    fn drop(&mut self) {
        unsafe {
            let max_msgs = (*self.msgq).max_msgs as usize;

            dealloc(self.msgq as *mut u8, Layout::new::<crate::sys::k_msgq>());
            dealloc(self.buffer, Layout::from_size_align(Self::MSG_SIZE * max_msgs, 1).unwrap());
        }
    }
}
