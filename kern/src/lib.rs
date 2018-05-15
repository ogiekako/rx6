#![feature(lang_items, asm)]
#![feature(const_fn)]
#![feature(ptr_offset_from)]
#![no_std]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(unused)]

pub mod console;
pub mod ioapic;
pub mod kalloc;
pub mod kernmain;
pub mod lapic;
pub mod linker;
pub mod memlayout;
pub mod mmu;
pub mod mp;
pub mod param;
pub mod picirq;
pub mod process;
pub mod string;
pub mod traps;
pub mod uart;
pub mod vm;
pub mod x86;

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

// For debug binary
#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn _Unwind_Resume() {}

#[no_mangle]
pub extern "C" fn kernmain() {
    assert_eq!(core::mem::size_of::<usize>(), 4);
    unsafe {
        kernmain::kernmain();
    }
}

#[cfg(test)]
mod tests {
    use core;
    #[test]
    fn is_32bit() {
        assert_eq!(core::mem::size_of::<u8>(), 1);
        assert_eq!(core::mem::size_of::<usize>(), 4);
    }
}
