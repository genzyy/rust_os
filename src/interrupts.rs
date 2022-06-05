use core::convert::TryInto;

// during cpu function call, first six integer arguments passed in registers are,
use crate::{gdt, print};
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin;
/**
 * rdi -> register destination index
 * rsi -> register source index
 * rdx -> register d extended
 * rcx -> register c extended
 * rax -> register a extended
 * r8 -> register 8
 * r9 -> register 9
 * rip -> points to next cpu instruction
 * PIC 8259 -> programmable interrupt controller
 */
// additional arguments are passed on stack
// results are returned in rax and rdx.
// the preserved registers should maintain their value and are only allowed to change
// if they retain their previous value.
// most calls start with push rbp.

// SSE instructions -> Streaming SIMD Extensions.

// double fault has the vector number 8.
// if double faults are unhandled, a triple fault will occur.
// Triple faults can’t be caught and most hardware reacts with a system reset.
use x86_64::{
    instructions::port::{PortGeneric, ReadWriteAccess},
    structures::idt::{InterruptDescriptorTable, InterruptStackFrame},
};

// to not have unsafe blocks and use static mut, lazy_statics are being used
// which are initialized when the first time they are referenced.
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt: InterruptDescriptorTable = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        return idt;
    };
}

// Interrupts provide a way to notify the CPU from attached hardware devices.

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });
// unsafe because wrong offsets could cause undefined behavior.

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

// static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new(); // creating new idt for our kernel.

pub fn init_dt() {
    // idt -> interrupt descriptor table that contains all functions which will be used to handle exceptions.
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    crate::println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
    use spin::Mutex;
    use x86_64::instructions::port::Port;

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(
            Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore)
        );
    }

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

// The breakpoint exception is the perfect exception to test exception handling. Its only purpose is to temporarily pause a program when the breakpoint instruction int3 is executed.

#[test_case]
pub fn test_breakpoint_exception() {
    //invoke a breakpoint exception.
    // x86_64::instructions::interrupts::int3(); // invoking breakpoint exception.
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    /*
     * PIC expects an explicit “end of interrupt” (EOI) signal from our interrupt handler.
     * This signal tells the controller that the interrupt was processed and that the system is ready to receive the next interrupt
     */

    print!(".");

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }

    // We need to be careful to use the correct interrupt vector number,
    // otherwise we could accidentally delete an important unsent interrupt or cause our system to hang.
    // This is the reason that the function is unsafe.

    // The hardware timer that we use is called the Programmable Interval Timer or PIT.
}

//  Deadlocks occur if a thread tries to acquire a lock that will never become free. Thus the thread hangs indefinitely.

// Almost everything in computers, is interrupt driven.
// The CPU of these computers, work until they get the instructions and interrupt handlers.
