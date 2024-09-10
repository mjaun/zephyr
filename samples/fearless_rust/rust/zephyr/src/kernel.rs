use core::time::Duration;
use crate::sys::{k_sleep};

pub mod printk;
pub mod errno;
pub mod mutex;
pub mod semaphore;
pub mod message_queue;
pub mod condition_variable;

mod panic;
mod allocator;
mod timeout;

pub fn sleep(duration: Duration) -> Duration {
    unsafe {
        let ret = k_sleep(duration.into());
        Duration::from_millis(ret as u64)
    }
}
