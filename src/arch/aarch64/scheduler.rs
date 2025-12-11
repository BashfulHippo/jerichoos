/*
 * ARM64 Task Scheduler
 *
 * Simple round-robin scheduler for testing context switching.
 */

use super::task::TaskContext;
use core::ptr;
use core::sync::atomic::{AtomicU64, Ordering};

/// Maximum number of tasks
const MAX_TASKS: usize = 8;

/// Global context switch counter for benchmarking
static CONTEXT_SWITCH_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Task stack size (16 KB per task)
const TASK_STACK_SIZE: usize = 16 * 1024;

/// Task states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Ready,
    Running,
    Blocked,
}

/// Task Control Block
#[repr(C)]
pub struct Task {
    pub context: TaskContext,
    pub stack: [u8; TASK_STACK_SIZE],
    pub state: TaskState,
    pub id: usize,
}

impl Task {
    pub const fn new() -> Self {
        Task {
            context: TaskContext::new(),
            stack: [0; TASK_STACK_SIZE],
            state: TaskState::Blocked,
            id: 0,
        }
    }
}

/// Global scheduler
pub struct Scheduler {
    pub tasks: [Task; MAX_TASKS],
    pub num_tasks: usize,
    pub current_task: usize,
}

impl Scheduler {
    pub const fn new() -> Self {
        const INIT_TASK: Task = Task::new();
        Scheduler {
            tasks: [INIT_TASK; MAX_TASKS],
            num_tasks: 0,
            current_task: 0,
        }
    }

    /// Add a new task to the scheduler
    ///
    /// # Arguments
    /// * `entry_point` - Function pointer to task entry
    ///
    /// # Returns
    /// Task ID, or None if scheduler is full
    pub fn spawn(&mut self, entry_point: extern "C" fn() -> !) -> Option<usize> {
        if self.num_tasks >= MAX_TASKS {
            return None;
        }

        let task_id = self.num_tasks;
        let task = &mut self.tasks[task_id];

        // Initialize task
        task.id = task_id;
        task.state = TaskState::Ready;

        // Calculate stack top (stacks grow downward on ARM)
        let stack_top = task.stack.as_ptr() as usize + TASK_STACK_SIZE;

        // Initialize task context
        task.context = TaskContext::init(entry_point as usize, stack_top);

        self.num_tasks += 1;

        uart_puts("[SCHED] Spawned task #");
        uart_puts_hex(task_id as u64);
        uart_puts(" at entry 0x");
        uart_puts_hex(entry_point as usize as u64);
        uart_puts("\n");

        Some(task_id)
    }

    /// Get current running task
    pub fn current(&self) -> &Task {
        &self.tasks[self.current_task]
    }

    /// Get mutable reference to current task
    pub fn current_mut(&mut self) -> &mut Task {
        &mut self.tasks[self.current_task]
    }

    /// Switch to the next ready task (round-robin)
    pub fn schedule(&mut self) {
        if self.num_tasks == 0 {
            return;
        }

        // Find next ready task
        let start = self.current_task;
        loop {
            self.current_task = (self.current_task + 1) % self.num_tasks;

            if self.tasks[self.current_task].state == TaskState::Ready {
                break;
            }

            // If we've checked all tasks and none are ready, go back to start
            if self.current_task == start {
                break;
            }
        }
    }

    /// Perform context switch to next task
    ///
    /// # Safety
    /// Must be called with interrupts disabled
    pub unsafe fn switch_to_next(&mut self) {
        if self.num_tasks <= 1 {
            return; // No other task to switch to
        }

        let prev_task = self.current_task;

        // Mark current task as ready (unless it's blocked)
        if self.tasks[prev_task].state == TaskState::Running {
            self.tasks[prev_task].state = TaskState::Ready;
        }

        // Schedule next task
        self.schedule();

        let next_task = self.current_task;

        // Mark next task as running
        self.tasks[next_task].state = TaskState::Running;

        if prev_task == next_task {
            return; // Same task, no switch needed
        }

        // Get context pointers
        let prev_ctx = &mut self.tasks[prev_task].context as *mut TaskContext;
        let next_ctx = &self.tasks[next_task].context as *const TaskContext;

        // Perform the context switch
        super::task::switch_context(prev_ctx, next_ctx);
    }

    /// Get number of tasks
    pub fn num_tasks(&self) -> usize {
        self.num_tasks
    }
}

// Global scheduler instance
pub static mut SCHEDULER: Scheduler = Scheduler::new();

/// Initialize the scheduler
pub fn init() {
    uart_puts("[SCHED] Scheduler initialized\n");
}

/// Spawn a new task
pub fn spawn(entry_point: extern "C" fn() -> !) -> Option<usize> {
    unsafe { SCHEDULER.spawn(entry_point) }
}

/// Switch to the next task
pub unsafe fn switch_to_next() {
    SCHEDULER.switch_to_next();
}

/// Get current task ID
pub fn current_task_id() -> usize {
    unsafe { SCHEDULER.current_task }
}

/// Get number of tasks
pub fn num_tasks() -> usize {
    unsafe { SCHEDULER.num_tasks() }
}

/// Reset the context switch counter (for benchmarking)
pub fn reset_switch_counter() {
    CONTEXT_SWITCH_COUNTER.store(0, Ordering::SeqCst);
}

/// Get the current context switch count
pub fn get_switch_count() -> u64 {
    use core::arch::asm;
    // Ensure all previous memory operations complete before reading
    unsafe { asm!("dsb sy", "isb", options(nostack, preserves_flags)); }
    let count = CONTEXT_SWITCH_COUNTER.load(Ordering::SeqCst);
    unsafe { asm!("dsb sy", options(nostack, preserves_flags)); }
    count
}

// C-callable wrapper for IRQ handler
// This is called from the IRQ exception handler with the exception frame
// Returns pointer to the exception frame to restore from (on next task's stack)

#[no_mangle]
pub extern "C" fn scheduler_switch_task(frame_ptr: *mut super::exceptions::ExceptionFrame) -> *mut super::exceptions::ExceptionFrame {
    let frame = unsafe { &mut *frame_ptr };

    unsafe {
        let prev_task = SCHEDULER.current_task;

        // Save current task's context from exception frame
        let current_idx = prev_task;
        if current_idx < SCHEDULER.num_tasks {
            let ctx = &mut SCHEDULER.tasks[current_idx].context;

            // Save ALL registers from exception frame to preserve complete task state
            // Caller-saved registers (x0-x18)
            ctx.x0 = frame.x0;
            ctx.x1 = frame.x1;
            ctx.x2 = frame.x2;
            ctx.x3 = frame.x3;
            ctx.x4 = frame.x4;
            ctx.x5 = frame.x5;
            ctx.x6 = frame.x6;
            ctx.x7 = frame.x7;
            ctx.x8 = frame.x8;
            ctx.x9 = frame.x9;
            ctx.x10 = frame.x10;
            ctx.x11 = frame.x11;
            ctx.x12 = frame.x12;
            ctx.x13 = frame.x13;
            ctx.x14 = frame.x14;
            ctx.x15 = frame.x15;
            ctx.x16 = frame.x16;
            ctx.x17 = frame.x17;
            ctx.x18 = frame.x18;

            // Callee-saved registers (x19-x30)
            ctx.x19 = frame.x19;
            ctx.x20 = frame.x20;
            ctx.x21 = frame.x21;
            ctx.x22 = frame.x22;
            ctx.x23 = frame.x23;
            ctx.x24 = frame.x24;
            ctx.x25 = frame.x25;
            ctx.x26 = frame.x26;
            ctx.x27 = frame.x27;
            ctx.x28 = frame.x28;
            ctx.x29_fp = frame.x29;
            ctx.x30_lr = frame.x30_lr;

            // Save PC and PSTATE from exception state
            // NOTE: We do NOT update SP here! The task's SP is preserved in its context
            // and should not be modified. The exception frame is on the IRQ stack, not the task stack.
            ctx.pc = frame.elr_el1; // Return address (where task was interrupted)
            ctx.pstate = frame.spsr_el1;

            // Mark current task as ready for re-scheduling
            SCHEDULER.tasks[current_idx].state = TaskState::Ready;
        }

        // Schedule next task
        SCHEDULER.schedule();

        // Get next task's context
        let next_idx = SCHEDULER.current_task;
        SCHEDULER.tasks[next_idx].state = TaskState::Running;

        // Increment context switch counter for benchmarking
        CONTEXT_SWITCH_COUNTER.fetch_add(1, Ordering::SeqCst);
        // Ensure counter update is visible to all cores/contexts
        core::arch::asm!("dsb sy", options(nostack, preserves_flags));

        // Compact logging: [S] C=0 N=1
        uart_putc(b'[');
        uart_putc(b'S');
        uart_putc(b']');
        uart_putc(b' ');
        uart_putc(b'C');
        uart_putc(b'=');
        uart_putc(b'0' + (prev_task as u8));
        uart_putc(b' ');
        uart_putc(b'N');
        uart_putc(b'=');
        uart_putc(b'0' + (next_idx as u8));
        uart_putc(b' ');

        let ctx = &SCHEDULER.tasks[next_idx].context;

        // Build exception frame on next task's stack
        let next_frame_ptr = (ctx.sp - 272) as *mut super::exceptions::ExceptionFrame;

        let next_frame = &mut *next_frame_ptr;

        // Restore ALL registers from next task's context (NOT from current frame!)
        // This ensures each task maintains its own complete register state

        // Caller-saved registers (x0-x18)
        next_frame.x0 = ctx.x0;
        next_frame.x1 = ctx.x1;
        next_frame.x2 = ctx.x2;
        next_frame.x3 = ctx.x3;
        next_frame.x4 = ctx.x4;
        next_frame.x5 = ctx.x5;
        next_frame.x6 = ctx.x6;
        next_frame.x7 = ctx.x7;
        next_frame.x8 = ctx.x8;
        next_frame.x9 = ctx.x9;
        next_frame.x10 = ctx.x10;
        next_frame.x11 = ctx.x11;
        next_frame.x12 = ctx.x12;
        next_frame.x13 = ctx.x13;
        next_frame.x14 = ctx.x14;
        next_frame.x15 = ctx.x15;
        next_frame.x16 = ctx.x16;
        next_frame.x17 = ctx.x17;
        next_frame.x18 = ctx.x18;

        // Callee-saved registers (x19-x30)
        next_frame.x19 = ctx.x19;
        next_frame.x20 = ctx.x20;
        next_frame.x21 = ctx.x21;
        next_frame.x22 = ctx.x22;
        next_frame.x23 = ctx.x23;
        next_frame.x24 = ctx.x24;
        next_frame.x25 = ctx.x25;
        next_frame.x26 = ctx.x26;
        next_frame.x27 = ctx.x27;
        next_frame.x28 = ctx.x28;
        next_frame.x29 = ctx.x29_fp;
        next_frame.x30_lr = ctx.x30_lr;
        next_frame.sp_el0 = ctx.sp;

        // Restore exception return state from task context
        next_frame.elr_el1 = ctx.pc; // Where to return to
        next_frame.spsr_el1 = ctx.pstate;

        // Return pointer to next task's frame
        // Assembly will switch SP to this before RESTORE_REGS
        next_frame_ptr
    }
}

// Helper functions for UART output

const UART_BASE: usize = 0x09000000;
const UART_DR: usize = UART_BASE + 0x00;
const UART_FR: usize = UART_BASE + 0x18;
const UART_FR_TXFF: u32 = 1 << 5;

fn uart_putc(c: u8) {
    unsafe {
        while (core::ptr::read_volatile(UART_FR as *const u32) & UART_FR_TXFF) != 0 {
            core::hint::spin_loop();
        }
        core::ptr::write_volatile(UART_DR as *mut u32, c as u32);
    }
}

fn uart_puts(s: &str) {
    for byte in s.bytes() {
        if byte == b'\n' {
            uart_putc(b'\r');
        }
        uart_putc(byte);
    }
}

fn uart_puts_hex(mut val: u64) {
    const HEX_CHARS: &[u8; 16] = b"0123456789ABCDEF";
    let mut buf = [0u8; 16];

    for i in 0..16 {
        buf[15 - i] = HEX_CHARS[(val & 0xF) as usize];
        val >>= 4;
    }

    for &b in &buf {
        uart_putc(b);
    }
}
