use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;

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
static ALLOCATOR: Dummy = Dummy;

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocaion error: {:?}", layout)
}
