// during cpu function call, first six integer arguments passed in registers are,
use crate::{gdt, hlt_loop, print, println};
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
 * IDT -> interrupt descriptor table -> tasks and interrupts.
 * GDT -> global descriptor table -> contains memory segments.
 * FFI -> foreign function interfaces.
 * ABI -> application binary interface.
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
    instructions::{
        hlt,
        port::{PortGeneric, ReadWriteAccess},
    },
    structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode},
};

// to not have unsafe blocks and use static mut, lazy_statics are being used
// which are initialized when the first time they are referenced.

// lazily evaluated statics.
// these are initalized when they are accessed for the first time.
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt: InterruptDescriptorTable = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);

        // the unsafe block in rust means "Trust me, I know what I am doing.".
        // Basically code outside unsafe block, is rejected by compiler if it thinks it might break something
        // or is accessing wrong address or anything fishy. But adding it in unsafe block, it says I am doing this
        // but you the developer is responsible.
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        return idt;
    };
}

// Interrupts provide a way to notify the CPU from attached hardware devices.

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

// const is immutable and does not represent a memory address.
// static variable point to a precise memory location. These can be modified but it is unsafe (should be done in unsafe block).

// Interior Mutability -> ability to mutate a variable when there are immutable references to it.

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });
// unsafe because wrong offsets could cause undefined behavior.

#[derive(Debug, Clone, Copy)]
// Debug -> able to print anything for debug purpose ({:?}).
// Clone -> able to clone resource instead of assigning or copying.
// Copy -> let x = 3; let y = x; now x cannot be used as it was moved to y. But using derive(Copy) this can be prevented.
#[repr(u8)]
// repr -> represent the memory location as a multiple of u8.
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

// providing a function to C ABI or x86-interupt ABI.
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

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    // when a page fault occurs, the cpu sets cr2 register for the page fault which contains
    // the virtual address accessed during the page fault or we can say the address which caused
    // page fault.
    // error code gives us information about the type of memory access occured -> read/write.
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    hlt_loop();

    // we can read from the current instruction pointer but we cannot write to it.
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

// If there is no interrupt for any task or from a hardware, there will be timer interrupt.
// A timer interrupt is an interrupt caused by a timer. Any Interrupt grabs control of a processor core away from whatever process it was running before. It is the responsibility of every interrupt service routine to save any registers it will change and restore them before passing control back to the interrupted program at its next sequential instruction.
// Timer interrupt is also used for tracking how much time a process is taking to complete.

// Paging and Memory Management

// no process should be able to access another process's memory address.
// paging is used to create seperate memory for different processes.
// MPU -> Memory Protection Unit.
// MPU is used to define small number of memory regions with different access permissions.
// On each memory access the MPU ensures that the address is in a region with correct access permissions and throws an exception otherwise.
// On x86, the hardware supports two different approaches to memory protection: segmentation and paging.
// CS -> code segment register used to fetch instructions.
// SS -> stack segment register is used for stack operations.
// DS -> data segment
// ES -> extra segment

// Virtual Memory -> VPN to access physical memory.
// Addresses before translation are virtual addresses.
// Addresses after translation are physical.
// phyisical addresses will be different from each other as they point to different memory locations
// but each location will have same address.
// virtual addresses depends on translation function and it is possible that two different virtual addresses
// point to same physical address.
// fragmentation problem is one of the reasons that segmentation is no longer used by most systems.
// segmentation is not even supported in 64-bit mode on x86 anymore

//  to divide both the virtual and the physical memory space into small, fixed-size blocks.
// The blocks of the virtual memory space are called pages and the blocks of the physical address space are called frames.

// Each page can be individually mapped to a frame, which makes it possible to split larger memory regions across non-continuous physical frames.
// to maintain a record of pages and frames, cpu uses a page table with virtual and physical addresses, with permission flags.
// CR3 register is used to map to this table.

// if we have a very large table, then we have two tables where table level 2  maps memory regions to table level 1.

// The first four characters from right are offset of the address, and then following four characters from right to left are table levels -> virtual address.
// first 16 characters are sign extensions.

// adding page offset to frame address gives us the phyiscal address.

// Bootimage crate.

// BootInfo -> contains information that is passed to the kernel on boot.
// It has two parts, memory_map and phyiscal_memory_offset.

// memory_map -> overview of physical memory and gives us how much physical memory is available
// in the system.

// pyhsical_memory_map -> gives virtual start address of physical memory.
// adding this offset to physical address gives us virtual address.

// translation of phyisical_memory_offset should point to physical address 0.
