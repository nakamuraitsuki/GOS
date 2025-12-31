#![no_std]
#![no_main]

#[no_mangle]
fn efi_main() {
    // NOTE: std マクロ使えないので後回し
    // println!("Hello, world!");
    loop {}
}

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}