#![feature(lang_items, asm)]
#![no_std]

mod kernmain;

#[cfg(not(test))]
#[lang = "eh_personality"] #[no_mangle] pub extern fn eh_personality() {}
#[cfg(not(test))]
#[lang = "panic_fmt"] #[no_mangle] pub extern fn panic_fmt() -> ! {loop{}}

#[no_mangle]
pub extern "C" fn kernmain() {
    kernmain::kernmain();
}

