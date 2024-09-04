#![no_std]

mod ffi;
mod panic;

extern "C" {
    fn say_hello();
}

#[no_mangle]
extern "C" fn rust_main() {
    unsafe {
        say_hello();
    }
}
