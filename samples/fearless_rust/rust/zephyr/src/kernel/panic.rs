use core::panic::PanicInfo;

extern "C" {
    fn rust_panic() -> !;
}

#[panic_handler]
fn panic(_ :&PanicInfo) -> ! {
    unsafe {
        rust_panic();
    }
}
