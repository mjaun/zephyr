use core::time::Duration;
use crate::sys::{k_ticks_t, k_timeout_t};

const FOREVER: k_timeout_t = k_timeout_t { ticks: -1 as k_ticks_t };
const NO_WAIT: k_timeout_t = k_timeout_t { ticks: 0 };

impl Into<k_timeout_t> for Duration {
    fn into(self) -> k_timeout_t {
        if self == Duration::MAX {
            return FOREVER;
        }
        if self == Duration::ZERO {
            return NO_WAIT;
        }

        const NANOS_PER_SEC: u128 = 1_000_000_000;
        const TICKS_PER_SEC: u128 = crate::sys::CONFIG_SYS_CLOCK_TICKS_PER_SEC as u128;

        let ticks = (self.as_nanos() * TICKS_PER_SEC).div_ceil(NANOS_PER_SEC);
        k_timeout_t { ticks: ticks as k_ticks_t }
    }
}
