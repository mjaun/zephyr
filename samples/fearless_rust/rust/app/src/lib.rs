#![no_std]

extern crate zephyr;
extern crate alloc;

use alloc::vec;
use zephyr::printkln;

#[no_mangle]
extern "C" fn rust_main() {
    printkln!("Hello World!");

    let test_vec = vec!(1, 2, 3, 4, 5);

    let mut sum = 0;
    for item in test_vec {
        sum += item;
    }

    printkln!("sum={}", sum);
}
