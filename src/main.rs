#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use rust_os::println;

// instead of defining our own start function, using pub extern C, we use entry_point function caller
// provided by bootiamge crate, so we know what type of function with what arguments should the
// boot function have.
entry_point!(kernel_boot);

fn kernel_boot(boot_info: &'static BootInfo) -> ! {
    println!("Hello World{}", "!");

    rust_os::init();

    x86_64::instructions::interrupts::int3(); // invoking breakpoint exceptions.

    use x86_64::registers::control::Cr3;

    let (level_4_page_table, _) = Cr3::read();
    println!(
        "Level 4 page table at: {:?}",
        level_4_page_table.start_address()
    );

    #[cfg(test)]
    test_main();

    println!("It did not crash!");

    // instead of using an endless loop that uses CPU to its 100%
    // we should halt the CPU so it waits for a new interrupt and uses less energy
    // when there is no interrupt to be handled.
    rust_os::hlt_loop();
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    rust_os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rust_os::test_panic_handler(info)
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
