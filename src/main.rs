#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::{boxed::Box, rc::Rc, vec, vec::Vec};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use rust_os::{allocator, println};

// instead of defining our own start function, using pub extern C, we use entry_point function caller
// provided by bootiamge crate, so we know what type of function with what arguments should the
// boot function have.
entry_point!(kernel_boot);

fn kernel_boot(boot_info: &'static BootInfo) -> ! {
    use rust_os::memory::{self, BootInfoFrameAllocator};
    use x86_64::{structures::paging::Page, structures::paging::Translate, VirtAddr};

    println!("Hello World{}", "!");
    rust_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    // map an unused page
    //let page = Page::containing_address(VirtAddr::new(0xdeadbeaf000));
    //memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    // write the string `New!` to the screen through the new mapping.
    //let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    //unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e) };

    //let addresses = [
    // the identity-mapped vga buffer page
    //0xb8000,
    // some code page
    //0x201008,
    // some stack page
    //  0x0100_0020_1a10,
    // virtual address mapped to physical address 0
    //  boot_info.physical_memory_offset,
    //];

    //for &address in &addresses {
    //let virt = VirtAddr::new(address);
    // let phys = mapper.translate_addr(virt);
    //  println!("{:?} -> {:?}", virt, phys);

    // the last 12 bits of the addresses remain same
    // because they are page offset and are not
    // part of translation.
    //}

    // allocate a number on the heap.
    let heap_value = Box::new(41);
    println!("heap value at {:p}", heap_value);

    // create a dynamically sized vector.
    let mut vec = Vec::new();
    for i in 0..500 {
        vec.push(i);
    }

    println!("vec at {:p}", vec.as_slice());

    // create a reference counted vector -> will be freed when count reaches 0.
    // rc -> Reference Counted -> reference counting pointer.
    let reference_counted = Rc::new(vec![1, 2, 3]);
    let cloned_reference = reference_counted.clone();
    println!(
        "current reference count is {}",
        Rc::strong_count(&cloned_reference)
    );
    core::mem::drop(reference_counted);
    println!(
        "reference count is {} now",
        Rc::strong_count(&cloned_reference)
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

// heap allocation.
// local variables are only kept for the time until the parent function is alive and are stored
// on call stack.
// static variables are stored on a fixed memory location and always live for the complete
// lifetime of program.
// these variables are assigned a memory location during compile time by the linker
// and are encoded in the execution.
// for static variables, we know their location at compile time, so we dont need a reference to them.
// Data race -> two threads modify a static variable at the same time.
// The only way to modify a static variable is to encapsulate it in a Mutex type,
// which ensures that only a single &mut reference exists at any point in time

// dynamic types work by allocating a larger amount of memory when they become full,
// copying all elements over, and then deallocating the old allocation.
