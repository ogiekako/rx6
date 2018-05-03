#![feature(lang_items, asm)]
#![no_std]

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(unused)]

pub mod kalloc;
pub mod kernmain;
pub mod memlayout;
pub mod mmu;
pub mod x86;

#[cfg(not(test))]
#[lang = "eh_personality"] #[no_mangle] pub extern fn eh_personality() {}
#[cfg(not(test))]
#[lang = "panic_fmt"] #[no_mangle] pub extern fn panic_fmt() -> ! {loop{}}

// For debug binary
#[cfg(not(test))]
#[no_mangle]
pub extern fn _Unwind_Resume() {}

#[no_mangle]
pub extern fn kernmain() {
    assert_eq!(core::mem::size_of::<usize>(), 4);
    unsafe {
        kernmain::kernmain();
    }
}

#[cfg(test)]
mod tests {
    use core;
    #[test]
    fn it_works() {
        assert_eq!(core::mem::size_of::<u8>(), 1);
        // assert_eq!(core::mem::size_of::<usize>(), 4);
    }
}
