#![feature(lang_items)]
#![no_std]

const SECTSIZE: u32 = 512;

struct Elfhdr {
    magic: u32,
    elf: [u8;12],
    typ: u16,
    machine: u16,
    version: u32,
    entry: u32,
    phoff: u32,
    shoff: u32,
    flags: u32,
    ehsize: u16,
    phentsize: u16,
    phnum:u16,
    shntsize: u16,
    shnum: u16,
    shstrndx: u16,
}

#[no_mangle]
pub extern fn bootmain() {
    let com1 = 0x3f8;
    
}

#[cfg(not(test))]
#[lang = "eh_personality"] #[no_mangle] pub extern fn eh_personality() {}
#[cfg(not(test))]
#[lang = "panic_fmt"] #[no_mangle] pub extern fn panic_fmt() -> ! {loop{}}

#[cfg(test)]
mod tests {
    extern crate std;
#[test]
fn elf_size() {
        assert_eq!(52, std::mem::size_of::<super::Elfhdr>());
}
}
