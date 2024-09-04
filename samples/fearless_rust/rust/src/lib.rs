#![no_std]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_ :&PanicInfo) -> ! {
    loop {
    }
}

extern "C" {
    fn say_hello();
}

#[no_mangle]
extern "C" fn rust_main() {
    unsafe {
        say_hello();
    }
}
