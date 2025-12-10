/*
 * ARM Generic Interrupt Controller (GIC) v2 Driver
 *
 * For QEMU virt machine:
 * - GIC Distributor: 0x08000000
 * - GIC CPU Interface: 0x08010000
 */

use core::ptr::{read_volatile, write_volatile};

// GIC Distributor registers
const GICD_BASE: usize = 0x08000000;
const GICD_CTLR: usize = GICD_BASE + 0x000;      // Distributor Control Register
const GICD_TYPER: usize = GICD_BASE + 0x004;     // Interrupt Controller Type Register
const GICD_ISENABLER0: usize = GICD_BASE + 0x100; // Interrupt Set-Enable Registers
const GICD_IPRIORITYR: usize = GICD_BASE + 0x400; // Interrupt Priority Registers

// GIC CPU Interface registers
const GICC_BASE: usize = 0x08010000;
const GICC_CTLR: usize = GICC_BASE + 0x000;      // CPU Interface Control Register
const GICC_PMR: usize = GICC_BASE + 0x004;       // Interrupt Priority Mask Register
const GICC_IAR: usize = GICC_BASE + 0x00C;       // Interrupt Acknowledge Register
const GICC_EOIR: usize = GICC_BASE + 0x010;      // End of Interrupt Register

// ARM Generic Timer interrupt ID (for QEMU virt machine)
const ARM_TIMER_IRQ: u32 = 30; // PPI 14 (16 + 14 = 30)

/// Initialize the GIC
pub fn init() {
    unsafe {
        uart_puts("[GIC] Initializing Generic Interrupt Controller v2...\n");

        // Read GIC type register to get info
        let typer = read_volatile(GICD_TYPER as *const u32);
        let it_lines_number = typer & 0x1F;
        let num_interrupts = (it_lines_number + 1) * 32;

        uart_puts("[GIC] Number of interrupt lines: ");
        uart_puts_hex(num_interrupts as u64);
        uart_puts("\n");

        // Disable distributor
        write_volatile(GICD_CTLR as *mut u32, 0);

        // Disable all interrupts
        for i in 0..((num_interrupts / 32) as usize) {
            write_volatile((GICD_ISENABLER0 + i * 4) as *mut u32, 0xFFFFFFFF);
        }

        // Set all priorities to a default value (higher number = lower priority)
        for i in 0..((num_interrupts / 4) as usize) {
            write_volatile((GICD_IPRIORITYR + i * 4) as *mut u32, 0xA0A0A0A0);
        }

        // Enable distributor
        write_volatile(GICD_CTLR as *mut u32, 1);
        uart_puts("[GIC] Distributor enabled\n");

        // Configure CPU interface
        // Set priority mask to lowest priority (all interrupts allowed)
        write_volatile(GICC_PMR as *mut u32, 0xFF);

        // Enable CPU interface
        write_volatile(GICC_CTLR as *mut u32, 1);
        uart_puts("[GIC] CPU interface enabled\n");

        uart_puts("[GIC] Initialization complete\n");
    }
}

/// Enable a specific interrupt
pub fn enable_interrupt(irq_num: u32) {
    unsafe {
        let reg = GICD_ISENABLER0 + ((irq_num / 32) * 4) as usize;
        let bit = 1 << (irq_num % 32);

        let current = read_volatile(reg as *const u32);
        write_volatile(reg as *mut u32, current | bit);

        uart_puts("[GIC] Enabled interrupt #");
        uart_puts_hex(irq_num as u64);
        uart_puts("\n");
    }
}

/// Enable the ARM Generic Timer interrupt
pub fn enable_timer_interrupt() {
    enable_interrupt(ARM_TIMER_IRQ);
}

/// Acknowledge an interrupt (returns interrupt ID)
pub fn acknowledge_interrupt() -> u32 {
    unsafe { read_volatile(GICC_IAR as *const u32) }
}

/// Signal end of interrupt
pub fn end_of_interrupt(irq_num: u32) {
    unsafe {
        write_volatile(GICC_EOIR as *mut u32, irq_num);
    }
}

// C-callable wrappers for exception handlers

#[no_mangle]
pub extern "C" fn gic_acknowledge_interrupt() -> u32 {
    acknowledge_interrupt()
}

#[no_mangle]
pub extern "C" fn gic_end_of_interrupt(irq_num: u32) {
    end_of_interrupt(irq_num);
}

// Helper functions for UART output

const UART_BASE: usize = 0x09000000;
const UART_DR: usize = UART_BASE + 0x00;
const UART_FR: usize = UART_BASE + 0x18;
const UART_FR_TXFF: u32 = 1 << 5;

fn uart_putc(c: u8) {
    unsafe {
        while (read_volatile(UART_FR as *const u32) & UART_FR_TXFF) != 0 {
            core::hint::spin_loop();
        }
        write_volatile(UART_DR as *mut u32, c as u32);
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
