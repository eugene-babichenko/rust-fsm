#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[allow(unused_imports)]
use rust_fsm;

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    loop {}
}
