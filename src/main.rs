#![no_std] // don't link the std rust standard library.
#![no_main] // disable all rust-level entry points.
#![feature(custom_test_frameworks)] // enable rust custom test framework that is independent of std.
#![test_runner(rust_os::test_runner)]   // test runner for our test framework.
#![reexport_test_harness_main = "test_main"]

use rust_os::println;
use core::panic::PanicInfo;

#[no_mangle] // don't mangle the name of this function to some random name.
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");

    #[cfg(test)]
    test_main();

    loop {}
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]

// test panic handler.
fn panic(info: &PanicInfo) -> ! {
    rust_os::test_panic_handler(info)
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
