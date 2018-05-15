// Intel 8250 serial port (UART).

use ioapic::*;
use lapic::*;
use mmu::*;
use param::*;
use picirq::*;
use process::*;
use traps::*;
use x86::*;

pub const COM1: u16 = 0x3f8;

static mut uart: bool = false; // is there a uart?

pub unsafe fn uartinit() {
    // Turn off the FIFO
    outb(COM1 + 2, 0);

    // 9600 baud, 8 data bits, 1 stop bit, parity off.
    outb(COM1 + 3, 0x80); // Unlock divisor
    outb(COM1 + 0, (115200u32 / 9600u32) as u8);
    outb(COM1 + 1, 0);
    outb(COM1 + 3, 0x03); // Lock divisor, 8 data bits.
    outb(COM1 + 4, 0);
    outb(COM1 + 1, 0x01); // Enable receive interrupts.

    // If status is 0xFF, no serial port.
    if (inb(COM1 + 5) == 0xFF) {
        return;
    }
    uart = true;

    // Acknowledge pre-existing interrupt conditions;
    // enable interrupts.
    inb(COM1 + 2);
    inb(COM1 + 0);
    picenable(IRQ_COM1 as i32);
    ioapicenable(IRQ_COM1, 0);

    for p in "xv6...\n".bytes() {
        uartputc(p);
    }
}

pub unsafe fn uartputc(c: u8) {
    if !uart {
        return;
    }
    for i in 0..128 {
        if inb(COM1 + 5) & 0x20 != 0 {
            break;
        }
        microdelay(10);
    }
    outb(COM1 + 0, c);
}

pub unsafe fn uartgetc() -> Option<u8> {
    if !uart {
        return None;
    }
    if ((inb(COM1 + 5) & 0x01) == 0) {
        return None;
    }
    Some(inb(COM1 + 0))
}

// fn uartintr()
// {
//   consoleintr(uartgetc);
// }
