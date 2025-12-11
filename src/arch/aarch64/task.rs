/*
 * ARM64 Task Management
 *
 * This module provides task context structures and context switching for ARM64.
 */

use core::arch::asm;

/// ARM64 Task Context
///
/// Saves ALL registers for complete task state preservation during interrupts.
/// This is necessary because interrupts can occur at any point, and we need
/// to preserve caller-saved registers (x0-x18) as well as callee-saved (x19-x30).
/// Total size: 272 bytes (matches ExceptionFrame)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TaskContext {
    // Caller-saved registers x0-x18 (must be preserved across context switches)
    pub x0: u64,
    pub x1: u64,
    pub x2: u64,
    pub x3: u64,
    pub x4: u64,
    pub x5: u64,
    pub x6: u64,
    pub x7: u64,
    pub x8: u64,
    pub x9: u64,
    pub x10: u64,
    pub x11: u64,
    pub x12: u64,
    pub x13: u64,
    pub x14: u64,
    pub x15: u64,
    pub x16: u64,
    pub x17: u64,
    pub x18: u64,

    // Callee-saved registers x19-x29
    pub x19: u64,
    pub x20: u64,
    pub x21: u64,
    pub x22: u64,
    pub x23: u64,
    pub x24: u64,
    pub x25: u64,
    pub x26: u64,
    pub x27: u64,
    pub x28: u64,
    pub x29_fp: u64,  // Frame pointer

    // Special registers
    pub x30_lr: u64,  // Link register (return address)
    pub sp: u64,      // Stack pointer
    pub pc: u64,      // Program counter (ELR_EL1)
    pub pstate: u64,  // Processor state (SPSR_EL1)
}

impl TaskContext {
    /// Create a new empty task context
    pub const fn new() -> Self {
        TaskContext {
            x0: 0,
            x1: 0,
            x2: 0,
            x3: 0,
            x4: 0,
            x5: 0,
            x6: 0,
            x7: 0,
            x8: 0,
            x9: 0,
            x10: 0,
            x11: 0,
            x12: 0,
            x13: 0,
            x14: 0,
            x15: 0,
            x16: 0,
            x17: 0,
            x18: 0,
            x19: 0,
            x20: 0,
            x21: 0,
            x22: 0,
            x23: 0,
            x24: 0,
            x25: 0,
            x26: 0,
            x27: 0,
            x28: 0,
            x29_fp: 0,
            x30_lr: 0,
            sp: 0,
            pc: 0,
            pstate: 0,
        }
    }

    /// Initialize a task context for a new task
    ///
    /// # Arguments
    /// * `entry_point` - Function pointer to task entry
    /// * `stack_top` - Top of the task's stack
    pub fn init(entry_point: usize, stack_top: usize) -> Self {
        let mut ctx = Self::new();

        // Set program counter to entry point
        ctx.pc = entry_point as u64;

        // Set stack pointer
        ctx.sp = stack_top as u64;

        // Set processor state for EL1 (kernel mode)
        // SPSR_EL1: M[4:0] = 0b00101 (EL1h - EL1 with SP_EL1)
        //           D = 0 (Debug exceptions unmasked)
        //           A = 0 (SError unmasked)
        //           I = 0 (IRQ unmasked)
        //           F = 0 (FIQ unmasked)
        ctx.pstate = 0b00101; // EL1h mode

        ctx
    }
}

/// Switch from current task context to next task context
///
/// # Safety
/// This function manipulates CPU registers and stack pointers.
/// Caller must ensure both contexts are valid.
#[unsafe(naked)]
#[no_mangle]
pub unsafe extern "C" fn switch_context(current: *mut TaskContext, next: *const TaskContext) {
    core::arch::naked_asm!(
        // Save current task context
        // x0 = current context pointer

        // Save callee-saved registers
        "stp x19, x20, [x0, #0]",
        "stp x21, x22, [x0, #16]",
        "stp x23, x24, [x0, #32]",
        "stp x25, x26, [x0, #48]",
        "stp x27, x28, [x0, #64]",
        "stp x29, x30, [x0, #80]",

        // Save stack pointer
        "mov x9, sp",
        "str x9, [x0, #96]",

        // Save return address as PC (where we'll return to)
        "adr x9, 1f",  // Address of label 1 (return point)
        "str x9, [x0, #104]",

        // Save processor state (SPSR_EL1)
        "mrs x9, spsr_el1",
        "str x9, [x0, #112]",

        // Load next task context
        // x1 = next context pointer

        // Restore callee-saved registers
        "ldp x19, x20, [x1, #0]",
        "ldp x21, x22, [x1, #16]",
        "ldp x23, x24, [x1, #32]",
        "ldp x25, x26, [x1, #48]",
        "ldp x27, x28, [x1, #64]",
        "ldp x29, x30, [x1, #80]",

        // Restore stack pointer
        "ldr x9, [x1, #96]",
        "mov sp, x9",

        // Restore program counter and jump to it
        // Note: For context switching between tasks, we use br (not eret)
        // since we're not returning from an exception
        "ldr x9, [x1, #104]",
        "br x9",

        // Return point for current task when it resumes
        "1:",
        "ret",
    );
}

/// Task entry wrapper
///
/// This is called when a new task starts. It sets up the task environment
/// and calls the actual task function.
#[no_mangle]
pub extern "C" fn task_entry_wrapper(task_fn: extern "C" fn() -> !) -> ! {
    // Enable interrupts for the new task
    unsafe {
        asm!("msr daifclr, #2");  // Clear IRQ mask
    }

    // Call the task function
    task_fn()
}

/// Get current stack pointer
pub fn get_sp() -> u64 {
    let sp: u64;
    unsafe {
        asm!("mov {}, sp", out(reg) sp);
    }
    sp
}

/// Get current program counter (approximate)
pub fn get_pc() -> u64 {
    let pc: u64;
    unsafe {
        asm!("adr {}, .", out(reg) pc);
    }
    pc
}
