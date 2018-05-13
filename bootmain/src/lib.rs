#![feature(lang_items, asm)]
#![no_std]

mod elf;
mod x86;

use elf::*;
use x86::*;

const SECTSIZE: u32 = 512;

#[no_mangle]
pub unsafe extern "C" fn bootmain() {
    let elf: *const Elfhdr = 0x10000 as *const Elfhdr; // scratch space

    // Read 1st page off disk
    readseg(elf as *mut u8, 4096, 0);

    // Is this an ELF executable?
    if (*elf).magic != ELF_MAGIC {
        return; // let bootasm.S handle error
    }

    // Load each program segment (ignores ph flags).
    let mut ph = ((elf as *const u8).offset((*elf).phoff as isize)) as *const Proghdr;
    let eph = ph.offset((*elf).phnum as isize);
    while ph < eph {
        let pa = (*ph).paddr as *mut u8;
        readseg(pa, (*ph).filesz, (*ph).off);
        if (*ph).memsz > (*ph).filesz {
            stosb(
                pa.offset((*ph).filesz as isize),
                0,
                (*ph).memsz as i32 - (*ph).filesz as i32,
            );
        }

        ph = ph.offset(1);
    }
    // Call the entry point from the ELF header.
    // Does not return!
    let entry: extern "C" fn() = core::mem::transmute((*elf).entry);
    entry();
}

#[inline(never)] // Avoid boot loader to blow up over 510 bytes.
                 // Read ’count’ bytes at ’offset’ from kernel into physical address ’pa’.
                 // Might copy more than asked.
unsafe fn readseg(pa: *mut u8, count: u32, offset: u32) {
    let mut pa = pa;
    let epa = pa.offset(count as isize);
    // Round down to sector boundary.
    pa = pa.offset(-((offset % SECTSIZE) as isize));

    let mut offset = offset;
    // Translate from bytes to sectors; kernel starts at sector 1.
    offset = (offset / SECTSIZE) + 1;

    // If this is too slow, we could read lots of sectors at a time.
    // We’d write more to memory than asked, but it doesn’t matter −−
    // we load in increasing order.
    loop {
        if pa >= epa {
            break;
        }
        readsect(pa, offset);
        pa = pa.offset(SECTSIZE as isize);
        offset += 1;
    }
}

fn waitdisk() {
    unsafe {
        // Wait for disk ready.
        while (inb(0x1F7) & 0xC0) != 0x40 {}
    }
}

// Read a single sector at offset into dst.
unsafe fn readsect(dst: *mut u8, offset: u32) {
    // Issue command.
    waitdisk();
    outb(0x1F2, 1); // count = 1
    outb(0x1F3, offset as u8);
    outb(0x1F4, (offset >> 8) as u8);
    outb(0x1F5, (offset >> 16) as u8);
    outb(0x1F6, ((offset >> 24) | 0xE0) as u8);
    outb(0x1F7, 0x20); // cmd 0x20 − read sectors

    // Read data.
    waitdisk();
    insl(0x1F0, dst, (SECTSIZE / 4) as i32);
}

#[cfg(not(test))]
#[lang = "eh_personality"]
#[no_mangle]
pub extern "C" fn eh_personality() {}
#[cfg(not(test))]
#[lang = "panic_fmt"]
#[no_mangle]
pub extern "C" fn panic_fmt() -> ! {
    loop {}
}

#[cfg(test)]
mod tests {
    extern crate std;
    #[test]
    fn elf_size() {
        assert_eq!(52, std::mem::size_of::<super::Elfhdr>());
    }
}
