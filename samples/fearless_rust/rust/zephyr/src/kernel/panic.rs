use core::panic::PanicInfo;
use crate::printkln;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    printkln!("{}", info);

    unsafe {
        crate::sys::rust_panic();
    }
}
