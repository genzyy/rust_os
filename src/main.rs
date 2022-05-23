#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(custom_test_frameworks)] // rust custom test framework that is independent of std library
#![test_runner(crate::test_runner)] // test runner for our test framework
#![reexport_test_harness_main = "test_main"]

mod vga_buffer;
use core::panic::PanicInfo;

// static HELLO: &[u8] = b"Hello World!";

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    // use core::fmt::Write;
    // vga_buffer::WRITER.lock().write_str("Hello again!").unwrap();
    // write!(vga_buffer::WRITER.lock(), "some numbers: {} {}", 42, 1.337).unwrap();
    println!("This is {}'s kernel printing some BS", "genzyy");
    // panic!("panic message here");

    #[cfg(test)]
    test_main();

    loop {}
}

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    // tests is a list of references to types that can be executed as functions.
    // so test is a reference to a function which can be executed with parenthesis.
    println!("Running {} tests...", tests.len());
    for test in tests {
        test();
    }
}
