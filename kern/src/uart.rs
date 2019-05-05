use super::*;
// Intel 8250 serial port (UART).

// Memory mapped I/O interface is binded here.
pub const COM1: u16 = 0x3f8;

static mut uart: bool = false; // is there a uart?

pub unsafe extern "C" fn uartinit() {
    // Turn off the FIFO
    outb(COM1 + 2, 0);

    // 9600 baud, 8 data bits, 1 stop bit, parity off.
    outb(COM1 + 3, 0x80); // Unlock divisor
    outb(COM1 + 0, (115200usize / 9600usize) as u8);
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

// Put the letter to display.
pub unsafe extern "C" fn uartputc(c: u8) {
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

pub unsafe extern "C" fn uartgetc() -> i32 {
    if !uart {
        return -1;
    }
    if (inb(COM1 + 5) & 0x01) == 0 {
        return -1;
    }
    inb(COM1 + 0) as i32
}

pub unsafe extern "C" fn uartintr() {
    consoleintr(uartgetc);
}
