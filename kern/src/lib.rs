#![feature(lang_items, asm)]
#![no_std]

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
    unsafe {
        kernmain::kernmain();
    }
}
