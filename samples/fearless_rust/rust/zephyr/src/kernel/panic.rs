use core::panic::PanicInfo;
use crate::printkln;

extern "C" {
    fn rust_panic() -> !;
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    printkln!("{}", info);

    unsafe {
        rust_panic();
    }
}
