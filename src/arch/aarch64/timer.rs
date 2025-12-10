/*
 * ARM Generic Timer Driver
 *
 * The ARM Generic Timer provides a consistent timer across all ARM64 systems.
 * It generates periodic interrupts for scheduling and timekeeping.
 */

use core::arch::asm;

/// Initialize the ARM Generic Timer
pub fn init() {
    unsafe {
        uart_puts("[TIMER] Initializing ARM Generic Timer...\n");

        // Read counter frequency (Hz)
        let freq: u64;
        asm!("mrs {0}, cntfrq_el0", out(reg) freq);

        uart_puts("[TIMER] Counter frequency: ");
        uart_puts_hex(freq);
        uart_puts(" Hz\n");

        // Calculate timer value for 10ms ticks (100 Hz)
        let ticks_per_10ms = freq / 100;

        uart_puts("[TIMER] Setting timer for 10ms ticks (");
        uart_puts_hex(ticks_per_10ms);
        uart_puts(" ticks)\n");

        // Set timer compare value
        asm!(
            "msr cntp_tval_el0, {0}",
            in(reg) ticks_per_10ms as u32,
        );

        // Enable timer
        // CNTP_CTL_EL0:
        // - bit 0: Enable
        // - bit 1: IMASK (0 = not masked)
        // - bit 2: ISTATUS (read-only)
        let ctl: u32 = 0b001; // Enable, not masked
        asm!(
            "msr cntp_ctl_el0, {0}",
            in(reg) ctl,
        );

        uart_puts("[TIMER] Timer enabled\n");
    }
}

/// Get the current timer count
pub fn get_counter() -> u64 {
    let count: u64;
    unsafe {
        asm!("mrs {0}, cntpct_el0", out(reg) count);
    }
    count
}

/// Re-arm the timer for the next interrupt
pub fn rearm() {
    unsafe {
        // Read counter frequency
        let freq: u64;
        asm!("mrs {0}, cntfrq_el0", out(reg) freq);

        // Set timer for next 10ms
        let ticks_per_10ms = freq / 100;
        asm!(
            "msr cntp_tval_el0, {0}",
            in(reg) ticks_per_10ms as u32,
        );
    }
}

// C-callable wrapper for exception handlers

#[no_mangle]
pub extern "C" fn timer_rearm() {
    rearm();
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
