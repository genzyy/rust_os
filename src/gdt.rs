use lazy_static::lazy_static;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

// zombie process -> a process which is complete but is still in process table.
// Usually happens for parent processes as they are waiting for child process' exit status.
// Thread Stack -> available for every ongoing thread -> contains useful data as long as a thread is alive.

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    // TSS -> task state segment -> contains information about a task.
    // In protected mode, it is used for hardware task switching.
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        return tss;
    };
}

// double fault happens when there is not entry in IDT (Interrupt Descriptor Table)
// to catch first fault (first fault can be anything related to hardware or software fault or memory fault).
// There is triple fault as well which occurs when there is no entry or a handler for double fault.
// When there triple fault occurs, the device shuts down removing everything from the memory.

// Newer chips don't have a triple fault handler because they get the double fault handler right.

// identity mapping means how the page tables are set up.

// stacks on x86 grow downwards, i.e. from high addresses to low addresses.
// Bootloader maps immutable statics to read-only page.

// GDT -> Global Descriptor Table
// used for memory segmentation.
// gives characteristics of various memory area/segments which are being used
// during the program execution including read/write access priviledge, everything.

// was used to contain structures of a program to isolate different programs on older machines.
// on x86_64, it used for switching between kernel space and user space and loading a TSS structure.

// though it is said that in x64, segmentation is not used but it is being used when fs and gs registers are used.
// https://stackoverflow.com/questions/57222727/is-segmentation-completely-not-used-in-x64

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (
            gdt,
            Selectors {
                code_selector,
                tss_selector,
            },
        )
    };
}

struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

pub fn init() {
    use x86_64::instructions::segmentation::{Segment, CS};
    use x86_64::instructions::tables::load_tss;

    GDT.0.load();
    unsafe {
        // CS -> code segment register.
        // code segment in the memory is the place where executable program is stored.
        // cs register (code segment register) is used to access code segment memory.
        CS::set_reg(GDT.1.code_selector);
        load_tss(GDT.1.tss_selector);
    }
}
