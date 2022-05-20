#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

use core::panic::PanicInfo;

static HELLO: &[u8] = b"Hello World!";

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    // this function is the entry point, since the linker looks for a function
    // named `_start` by default
    let vga_buffer = 0xb800 as *mut u8; // raw pointer converted to an u8 integer.

    for (i, &byte) in HELLO.iter().enumerate() {
        //  i -> index of characters from HELLO, byte -> bytes from HELLO.
        unsafe {
            // writing content from HELLO to VGA Buffer with cyan (0xb) color.
            *vga_buffer.offset(i as isize * 2) = byte;
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
        }
    }
    loop {}
}

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
