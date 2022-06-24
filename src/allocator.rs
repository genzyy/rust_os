use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use linked_list_allocator::LockedHeap;

pub struct Dummy;

unsafe impl GlobalAlloc for Dummy {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        panic!("dealloc should be never called")
    }

    // alloc_zeroed and realloc have their default implementation.
}

// assigning a global allocator which provides
// allocate and deallocate functions.
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocaion error: {:?}", layout)
}

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KB.

use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTable, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};

pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        // containing_address -> returns the page that contains given virtual address.
        let heap_start_page = Page::containing_address(heap_start);
        let heap_page_end = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_page_end)
    };

    for page in page_range {
        // allocates a frame of memory.
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?; // if fails gives an early error.
                                                        // ? -> returns error instead of panic.
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        // PRESENT -> Specifies whether the mapped frame or page table is loaded in memory.
        // WRITABLE -> Controls whether writes to the mapped frames are allowed.
        // if the bit for this in level 1 page table is unset that means it is read only. If it is unset in
        // higher level page tables, then the complete range of mapped pages is read only.

        unsafe { mapper.map_to(page, frame, flags, frame_allocator)?.flush() };
    }
    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }
    Ok(())
}

// alloc crate documentation.

// crate providing smart pointers.
// references are only used to get the address and they cannot do much.
// smart pointers are structures which are used as references which can do a lot more.

// references borrow the data which they point to.
// smart pointers own the data which they are made to refer to.
// source: https://www.youtube.com/watch?v=m76sRj2VgGo
// Cons is a variant of List enum. Cons(i32, Box<List>) says there are two possible cases for linked list,
// either an i32 value/variable or an empty list or Nil.

//enum List {
//    Cons(i32, Box<List>),
//    Nil,
//}
// https://stackoverflow.com/questions/23311773/what-is-cons

//Box stores a pointer of fixed size on stack but the memory size of its reference is expandable.

// Box: a simple way to allocate memory for some value on heap.
// Box cannot allocate more than isize::MAX memory.
