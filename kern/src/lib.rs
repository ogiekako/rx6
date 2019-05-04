#![feature(lang_items, asm)]
#![feature(const_fn)]
#![feature(ptr_offset_from)]
#![feature(const_transmute)]
#![no_std]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(unused)]

pub use core::mem::size_of;
pub use core::mem::size_of_val;
pub use core::mem::transmute;
pub use core::ptr::null;
pub use core::ptr::null_mut;

pub use bio::*;
pub use buf::*;
pub use console::*;
pub use date::*;
pub use elf::*;
pub use exec::*;
pub use fcntl::*;
pub use file::*;
pub use fs::*;
pub use ide::*;
pub use ioapic::*;
pub use kalloc::*;
pub use kbd::*;
pub use kernmain::*;
pub use lapic::*;
pub use linker::*;
pub use log::*;
pub use memlayout::*;
pub use mmu::*;
pub use mp::*;
pub use param::*;
pub use picirq::*;
pub use pipe::*;
pub use process::*;
pub use sleeplock::*;
pub use spinlock::*;
pub use stat::*;
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

pub mod spinlock_mutex;

pub mod bio;
pub mod buf;
pub mod console;
pub mod date;
pub mod elf;
pub mod exec;
pub mod fcntl;
pub mod file;
pub mod fs;
pub mod ide;
pub mod ioapic;
pub mod kalloc;
pub mod kbd;
pub mod kernmain;
pub mod lapic;
pub mod linker;
pub mod log;
pub mod memlayout;
pub mod mmu;
pub mod mp;
pub mod param;
pub mod picirq;
pub mod pipe;
pub mod process;
pub mod sleeplock;
pub mod spinlock;
pub mod stat;
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
    unsafe {
        if let Some(s) = info.payload().downcast_ref::<&str>() {
            console::cpanic(s);
        } else {
            console::cpanic("panic");
        }
        loop {}
    }
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
