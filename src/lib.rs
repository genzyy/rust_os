#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

#[cfg(test)]
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;

// By adding this extern crate statement, we specify that the compiler should try to include it.
extern crate alloc;

pub mod allocator;
pub mod gdt;
pub mod interrupts;
pub mod memory;
pub mod serial;
pub mod vga_buffer;

pub fn init() {
    gdt::init();
    interrupts::init_dt();
    unsafe { interrupts::PICS.lock().initialize() };
    // this function is also unsafe because it can cause undefined
    // behavior if the PIC is misconfigured.
    x86_64::instructions::interrupts::enable();
}

// the hlt instruction halts the CPU until next interrupt arrives so it uses less energy.

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

#[cfg(test)]
entry_point!(test_kernel_main);

/// Entry point for `cargo xtest`
#[cfg(test)]
fn test_kernel_main(boot_info: &'static BootInfo) -> ! {
    test_main();
    init();
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

// IST -> Interrupt Stack Table takes all interrupts into the stack instead of adding them to kernel stack
// This switch from kernel stack to IST happens on hardware level and is done before CPU pushes exception to the stack.
// IST is an array of 7 stack pointers.

// struct InterruptStackTable {
//     stack_pointers: [Option<StackPointer>; 7],
// }

// TSS -> Task State Segment
// TSS is used to hold information about the tasks (resources, addresses, etc.) for 32-bit machines.
// IST is a legacy part of TSS

// On x86_64, the TSS no longer holds any task specific information at all. Instead, it holds two stack tables (the IST is one of them) -> IST and PST
// The only common field between the 32-bit and 64-bit TSS is the pointer to the I/O port permissions bitmap.
// The Privilege Stack Table is used by the CPU when the privilege level changes

// TLB -> translation lookaside buffer -> special cache to store all recently used transactions.
