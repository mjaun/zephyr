// Copyright (c) 2024 Linaro LTD
// SPDX-License-Identifier: Apache-2.0

#![no_std]

extern crate alloc;

use alloc::sync::Arc;
use alloc::vec::Vec;
use core::time::Duration;

use zephyr::*;  // Wildcard import to bring in nested macros, maybe there is a better way
use zephyr::drivers::gpio::{GpioPin, GpioFlags};

const PHILOSOPHER_COUNT: usize = 5;
const FORK_COUNT: usize = 4;

fn random_sleep() {
    let duration = kernel::rand(400..800);
    kernel::msleep(duration as i32);
}

fn philosopher(id: usize, fork1: &kernel::Mutex, fork2: &kernel::Mutex) {
    loop {
        printk!("Phil {} is THINKING\n", id);
        random_sleep();

        printk!("Phil {} is HUNGRY\n", id);
        fork1.lock(Duration::MAX).unwrap();
        fork2.lock(Duration::MAX).unwrap();

        printk!("Phil {} is EATING\n", id);
        random_sleep();

        fork2.unlock().unwrap();
        fork1.unlock().unwrap();
    }
}

#[no_mangle]
extern "C" fn rust_main() {
    printk!("Hello, world! {}\n", kconfig::CONFIG_BOARD);

    let mut forks = Vec::new();

    for _ in 0..FORK_COUNT {
        forks.push(Arc::new(kernel::Mutex::new().unwrap()));
    }

    let mut philosophers = Vec::new();

    for phil_id in 0..PHILOSOPHER_COUNT {
        let mut fork_id1 = phil_id % FORK_COUNT;
        let mut fork_id2 = (phil_id + 1) % FORK_COUNT;

        if fork_id1 > fork_id2 {
            core::mem::swap(&mut fork_id1, &mut fork_id2);
        }

        let fork1 = forks[fork_id1].clone();
        let fork2 = forks[fork_id2].clone();

        philosophers.push(kernel::Thread::new(move || {
            philosopher(phil_id, fork1.as_ref(), fork2.as_ref());
        }, 1024, 5, Duration::ZERO).unwrap());
    }

    let gpio_pin = GpioPin::new(gpio_dt_spec_get!(dt_alias!(led0), gpios));

    gpio_pin.configure(GpioFlags::OutputActive)
        .expect("Failed to configure pin.");

    loop {
        kernel::msleep(1000);

        gpio_pin.toggle()
            .expect("Failed to toggle pin.");
    }
}
