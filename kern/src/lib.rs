#![feature(lang_items, asm)]
#![feature(const_fn)]
#![feature(ptr_offset_from)]
#![no_std]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(unused)]

#[macro_use]
extern crate lazy_static;

pub use bio::*;
pub use buf::*;
pub use console::*;
pub use file::*;
pub use fs::*;
pub use ide::*;
pub use ioapic::*;
pub use kalloc::*;
pub use kernmain::*;
pub use lapic::*;
pub use linker::*;
pub use memlayout::*;
pub use mmu::*;
pub use mp::*;
pub use param::*;
pub use picirq::*;
pub use process::*;
pub use spinlock::*;
pub use string::*;
pub use syscall::*;
pub use sysfile::*;
pub use sysproc::*;
pub use trap::*;
pub use traps::*;
pub use types::*;
pub use uart::*;
pub use vm::*;
pub use x86::*;

pub mod bio;
pub mod buf;
pub mod console;
pub mod file;
pub mod fs;
pub mod ide;
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
pub mod spinlock;
pub mod string;
pub mod syscall;
pub mod sysfile;
pub mod sysproc;
pub mod trap;
pub mod traps;
pub mod types;
pub mod uart;
pub mod vm;
pub mod x86;

#[cfg(not(test))]
#[lang = "eh_personality"]
#[no_mangle]
pub extern "C" fn eh_personality() {}
#[cfg(not(test))]
#[panic_handler]
#[no_mangle]
pub extern "C" fn panic(info: &core::panic::PanicInfo) -> ! {
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
