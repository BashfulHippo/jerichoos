//! Global Descriptor Table (GDT) for JerichoOS
//!
//! In x86-64 long mode, segmentation is mostly legacy, but we still need:
//! - Code and data segments
//! - TSS (Task State Segment) for interrupt handling

use x86_64::VirtAddr;
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use lazy_static::lazy_static;

/// Double fault stack index in IST
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    /// Task State Segment
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();

        // Set up the double fault stack
        // This gives us a separate stack for double fault handling
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5; // 20 KiB
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE as u64;
            stack_end // Stack grows downward
        };

        tss
    };
}

lazy_static! {
    /// Global Descriptor Table with segments
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();

        // Add code segment
        let code_selector = gdt.append(Descriptor::kernel_code_segment());

        // Add data segment
        let data_selector = gdt.append(Descriptor::kernel_data_segment());

        // Add TSS segment
        let tss_selector = gdt.append(Descriptor::tss_segment(&TSS));

        (gdt, Selectors {
            code_selector,
            data_selector,
            tss_selector
        })
    };
}

/// Segment selectors for our GDT entries
struct Selectors {
    code_selector: SegmentSelector,
    data_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

/// Initialize the GDT
pub fn init() {
    use x86_64::instructions::segmentation::{CS, DS, Segment};
    use x86_64::instructions::tables::load_tss;

    // Load the GDT
    GDT.0.load();

    unsafe {
        // Reload code segment register
        CS::set_reg(GDT.1.code_selector);

        // Reload data segment register
        DS::set_reg(GDT.1.data_selector);

        // Load TSS
        load_tss(GDT.1.tss_selector);
    }
}
