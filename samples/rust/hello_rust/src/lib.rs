// Copyright (c) 2024 Linaro LTD
// SPDX-License-Identifier: Apache-2.0

#![no_std]

extern crate alloc;

use core::time::Duration;
use zephyr::*;  // Wildcard import to bring in nested macros, maybe there is a better way
use zephyr::drivers::gpio::{GpioPin, GpioFlags};

#[no_mangle]
extern "C" fn rust_main() {
    printk!("Hello, world! {}\n", kconfig::CONFIG_BOARD);

    let variable = 10;

    let thread = kernel::Thread::new(move || {
        printk!("Variable within thread: {}\n", variable);
    }, 1024, 5, Duration::MAX).unwrap();

    thread.start();
    thread.join(Duration::MAX).unwrap();

    let gpio_pin = GpioPin::new(gpio_dt_spec_get!(dt_alias!(led0), gpios));

    gpio_pin.configure(GpioFlags::OutputActive)
        .expect("Failed to configure pin.");

    loop {
        kernel::msleep(1000);

        gpio_pin.toggle()
            .expect("Failed to toggle pin.");
    }
}
