// Copyright (c) 2024 Linaro LTD
// SPDX-License-Identifier: Apache-2.0

//! Printk implementation for Rust.
//!
//! This uses the `k_str_out` syscall, which is part of printk to output to the console.

use core::fmt::{
    Arguments,
    Result,
    Write,
    write,
};

#[macro_export]
macro_rules! printk {
    ($($arg:tt)*) => {{
        $crate::kernel::printk::printk(format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! printkln {
    ($($arg:tt)*) => {{
        $crate::kernel::printk::printkln(format_args!($($arg)*));
    }};
}

// This could readily be optimized for the configuration where we don't have userspace, as well as
// when we do, and are not running in userspace.  This initial implementation will always use a
// string buffer, as it doesn't depend on static symbols in print.c.
//
// A consequence is that the CONFIG_PRINTK_SYNC is enabled, the synchonization will only occur at
// the granularity of these buffers rather than at the print message level.

/// The buffer size for syscall.  This is a tradeoff between efficiency (large buffers need fewer
/// syscalls) and needing more stack space.
const BUF_SIZE: usize = 64;

struct Context {
    // How many characters are used in the buffer.
    count: usize,
    // Bytes written.
    buf: [u8; BUF_SIZE],
}

impl Context {
    fn add_byte(&mut self, b: u8) {
        // Ensure we have room for an entire UTF-8 sequence.
        if self.count + 4 > self.buf.len() {
            self.flush();
        }

        self.buf[self.count] = b;
        self.count += 1;
    }

    fn flush(&mut self) {
        if self.count > 0 {
            unsafe {
                crate::sys::k_str_out(self.buf.as_mut_ptr() as *mut i8, self.count);
            }
            self.count = 0;
        }
    }
}

impl Write for Context {
    fn write_str(&mut self, s: &str) -> Result {
        for b in s.bytes() {
            self.add_byte(b);
        }
        Ok(())
    }
}

pub fn printk(args: Arguments<'_>) {
    let mut context = Context {
        count: 0,
        buf: [0; BUF_SIZE],
    };
    write(&mut context, args).unwrap();
    context.flush();
}

pub fn printkln(args: Arguments<'_>) {
    let mut context = Context {
        count: 0,
        buf: [0; BUF_SIZE],
    };
    write(&mut context, args).unwrap();
    context.add_byte(b'\n');
    context.flush();
}
