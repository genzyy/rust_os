pub unsafe trait GlobalAlloc {
    // function that allocates memory.
    unsafe fn alloc(&self, layout: Layout) -> *mut u8;
    // function to free memory.
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout);

    // function to allocate memory and assign all memory locations as zero.
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 { ... }
    // function to resize the allocated memory.
    unsafe fn realloc(
        &self,
        ptr: *mut u8,
        layout: Layout,
        new_size: usize
    ) -> *mut u8 { ... }
}

// alloc function takes a Layout arguments which describes the required size and alignment that the allocated memory
// should have and returns a raw pointer to the first byte of the allocated memory.

// dealloc function frees the memory allocated by alloc function. It requires two arguments, pointer to the allocated memory
// and layout used while allocating memory.

// alloc_zeroed is same as alloc but setting the allocated memory block to zero.

// realloc function allows to grow or shrink the memory allocation.

// all functions are unsafe because the developer should make sure these functions do what they are made to do and do not return
// wrong address. For example, alloc should never return a memory location that is already in use.
