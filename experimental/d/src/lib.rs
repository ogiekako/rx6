#![feature(lang_items, asm)]
#![feature(const_fn)]
#![feature(start)]
#![feature(ptr_offset_from)]
#![feature(const_transmute)]
#![no_std]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(unused)]

#[lang = "eh_personality"]
#[no_mangle]
pub extern "C" fn eh_personality() {}

#[panic_handler]
#[no_mangle]
pub extern "C" fn panic(info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn _Unwind_Resume() {}

static mut a: [usize;1] = [0;1];

#[no_mangle]
pub extern "C" fn hoge() {
    unsafe {
        seginit();
    }
}

pub unsafe fn seginit() {
    a[id()] = 1;
}

pub unsafe fn id() -> usize {
    let i = 42;
    i
}

