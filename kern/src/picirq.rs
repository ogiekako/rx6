// Intel 8259A programmable interrupt controllers.

use traps::*;
use x86::*;

// I/O Addresses of the two programmable interrupt controllers
pub const IO_PIC1: u16 = 0x20; // Master (IRQs 0-7)
pub const IO_PIC2: u16 = 0xa0; // Slave (IRQs 8-15)
pub const IRQ_SLAVE: u16 = 2; // IRQ at which slave connects to master

// Current IRQ mask.
// Initial IRQ mask has interrupt 2 enabled (for slave 8259A).
static mut irqmask: u16 = 0xffff & !(1 << IRQ_SLAVE);

pub unsafe extern "C" fn picsetmask(mask: u16) {
    irqmask = mask;
    outb(IO_PIC1 + 1, mask as u8);
    outb(IO_PIC2 + 1, (mask >> 8) as u8);
}

pub unsafe extern "C" fn picenable(irq: i32) {
    picsetmask(irqmask & !(1 << irq));
}

// Initialize the 8259A interrupt controllers.
pub unsafe extern "C" fn picinit() {
    // mask all interrupts
    outb(IO_PIC1 + 1, 0xFF);
    outb(IO_PIC2 + 1, 0xFF);

    // Set up master (8259A-1)

    // ICW1:  0001g0hi
    //    g:  0 = edge triggering, 1 = level triggering
    //    h:  0 = cascaded PICs, 1 = master only
    //    i:  0 = no ICW4, 1 = ICW4 required
    outb(IO_PIC1, 0x11);

    // ICW2:  Vector offset
    outb(IO_PIC1 + 1, T_IRQ0 as u8);

    // ICW3:  (master PIC) bit mask of IR lines connected to slaves
    //        (slave PIC) 3-bit # of slave's connection to master
    outb(IO_PIC1 + 1, 1 << IRQ_SLAVE);

    // ICW4:  000nbmap
    //    n:  1 = special fully nested mode
    //    b:  1 = buffered mode
    //    m:  0 = slave PIC, 1 = master PIC
    //      (ignored when b is 0, as the master/slave role
    //      can be hardwired).
    //    a:  1 = Automatic EOI mode
    //    p:  0 = MCS-80/85 mode, 1 = intel x86 mode
    outb(IO_PIC1 + 1, 0x3);

    // Set up slave (8259A-2)
    outb(IO_PIC2, 0x11); // ICW1
    outb(IO_PIC2 + 1, T_IRQ0 as u8 + 8); // ICW2
    outb(IO_PIC2 + 1, IRQ_SLAVE as u8); // ICW3
                                        // NB Automatic EOI mode doesn't tend to work on the slave.
                                        // Linux source code says it's "to be investigated".
    outb(IO_PIC2 + 1, 0x3); // ICW4

    // OCW3:  0ef01prs
    //   ef:  0x = NOP, 10 = clear specific mask, 11 = set specific mask
    //    p:  0 = no polling, 1 = polling mode
    //   rs:  0x = NOP, 10 = read IRR, 11 = read ISR
    outb(IO_PIC1, 0x68); // clear specific mask
    outb(IO_PIC1, 0x0a); // read IRR by default

    outb(IO_PIC2, 0x68); // OCW3
    outb(IO_PIC2, 0x0a); // OCW3

    if irqmask != 0xFFFF {
        picsetmask(irqmask);
    }
}
