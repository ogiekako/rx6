use x86::*;

pub const IO_PIC1: i32 = 0x20i32;
pub const IO_PIC2: i32 = 0xa0i32;
pub const IRQ_SLAVE: i32 = 2i32;

static mut irqmask: u16 = (0xffffi32 & !(1i32 << IRQ_SLAVE)) as (u16);

unsafe fn picsetmask(mut mask: u16) {
    irqmask = mask;
    outb((IO_PIC1 + 1i32) as (u16), mask as (u8));
    outb((IO_PIC2 + 1i32) as (u16), (mask as (i32) >> 8i32) as (u8));
}

pub unsafe fn picenable(mut irq: i32) {
    picsetmask((irqmask as (i32) & !(1i32 << irq)) as (u16));
}

pub unsafe fn picinit() {
    outb((IO_PIC1 + 1i32) as (u16), 0xffu8);
    outb((IO_PIC2 + 1i32) as (u16), 0xffu8);
    outb(IO_PIC1 as (u16), 0x11u8);
    outb((IO_PIC1 + 1i32) as (u16), 32u8);
    outb((IO_PIC1 + 1i32) as (u16), (1i32 << IRQ_SLAVE) as (u8));
    outb((IO_PIC1 + 1i32) as (u16), 0x3u8);
    outb(IO_PIC2 as (u16), 0x11u8);
    outb((IO_PIC2 + 1i32) as (u16), (32i32 + 8i32) as (u8));
    outb((IO_PIC2 + 1i32) as (u16), IRQ_SLAVE as (u8));
    outb((IO_PIC2 + 1i32) as (u16), 0x3u8);
    outb(IO_PIC1 as (u16), 0x68u8);
    outb(IO_PIC1 as (u16), 0xau8);
    outb(IO_PIC2 as (u16), 0x68u8);
    outb(IO_PIC2 as (u16), 0xau8);
    if irqmask as (i32) != 0xffffi32 {
        picsetmask(irqmask);
    }
}
