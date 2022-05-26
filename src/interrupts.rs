// during cpu function call, first six integer arguments passed in registers are,
use lazy_static::lazy_static;
/**
 * rdi -> register destination index
 * rsi -> register source index
 * rdx -> register d extended
 * rcx -> register c extended
 * rax -> register a extended
 * r8 -> register 8
 * r9 -> register 9
 * rip -> points to next cpu instruction
 */
// additional arguments are passed on stack
// results are returned in rax and rdx.
// the preserved registers should maintain their value and are only allowed to change
// if they retain their previous value.
// most calls start with push rbp.

// SSE instructions -> Streaming SIMD Extensions.
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

// to not have unsafe blocks and use static mut, lazy_statics are being used
// which are initialized when the first time they are referenced.
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt: InterruptDescriptorTable = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        return idt;
    };
}

// static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new(); // creating new idt for our kernel.

pub fn init_dt() {
    // idt -> interrupt descriptor table that contains all functions which will be used to handle exceptions.
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    crate::println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

// The breakpoint exception is the perfect exception to test exception handling. Its only purpose is to temporarily pause a program when the breakpoint instruction int3 is executed.

#[test_case]
pub fn test_breakpoint_exception() {
    //invoke a breakpoint exception.
    x86_64::instructions::interrupts::int3(); // invoking breakpoint exception.
}
