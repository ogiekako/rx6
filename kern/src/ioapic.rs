// The I/O APIC manages hardware interrupts for an SMP system.
// http://www.intel.com/design/chipsets/datashts/29056601.pdf
// See also picirq.c.

use console::*;
use core;
use mp::*;
use traps::*;
use x86::*;

pub const IOAPIC: usize = 0xFEC00000; // Default physical address of IO APIC

pub const REG_ID: u32 = 0x00; // Register index: ID
pub const REG_VER: u32 = 0x01; // Register index: version
pub const REG_TABLE: u32 = 0x10; // Redirection table base

// The redirection table starts at REG_TABLE and uses
// two registers to configure each interrupt.
// The first (low) register in a pair contains configuration bits.
// The second (high) register contains a bitmask telling which
// CPUs can serve that interrupt.
pub const INT_DISABLED: u32 = 0x00010000; // Interrupt disabled
pub const INT_LEVEL: u32 = 0x00008000; // Level-triggered (vs edge-)
pub const INT_ACTIVELOW: u32 = 0x00002000; // Active low (vs high)
pub const INT_LOGICAL: u32 = 0x00000800; // Destination is CPU id (vs APIC ID)

// TODO: volatile
static mut ioapic: *mut Ioapic = core::ptr::null_mut();

// IO APIC MMIO structure: write reg, then read or write data.
#[repr(C)]
struct Ioapic {
    reg: u32,
    pad: [u32; 3],
    data: u32,
}

pub unsafe fn ioapicread(reg: u32) -> u32 {
    (*ioapic).reg = reg;
    (*ioapic).data
}

pub unsafe fn ioapicwrite(reg: u32, data: u32) {
    (*ioapic).reg = reg;
    (*ioapic).data = data;
}

pub unsafe fn ioapicinit() {
    if !ismp {
        return;
    }

    ioapic = IOAPIC as *mut Ioapic;
    let maxintr = (ioapicread(REG_VER) >> 16) & 0xFF;
    let id = (ioapicread(REG_ID) >> 24) as u8;
    if id != ioapicid {
        cprintf("ioapicinit: id isn't equal to ioapicid; not a MP\n", &[]);
    }

    // Mark all interrupts edge-triggered, active high, disabled,
    // and not routed to any CPUs.
    for i in 0..=maxintr {
        ioapicwrite(REG_TABLE + 2 * i, INT_DISABLED | (T_IRQ0 + i));
        ioapicwrite(REG_TABLE + 2 * i + 1, 0);
    }
}

pub unsafe fn ioapicenable(irq: u32, cpunum: u32) {
    if (!ismp) {
        return;
    }

    // Mark interrupt edge-triggered, active high,
    // enabled, and routed to the given cpunum,
    // which happens to be that cpu's APIC ID.
    ioapicwrite(REG_TABLE + 2 * irq, T_IRQ0 + irq);
    ioapicwrite(REG_TABLE + 2 * irq + 1, cpunum << 24);
}
