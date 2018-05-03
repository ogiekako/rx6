/// main.c in xv6
use kalloc;
use memlayout;

// memlayout.h
const KERNBASE: u32 = 0x80000000; // First kernel virtual address

// mmu.h
const PGSIZE: i32 = 4096;
const NPDENTRIES: i32 = 1024; // # directory entries per page directory;

const PTE_P: i32 = 0x001; // Present
const PTE_W: i32 = 0x002; // Writeable
const PTE_PS: i32 = 0x080; // Page Size

const PDXSHIFT: i32 = 22;

extern "C" {
    // first address after kernel loaded from ELF file
    static mut end: u8;
}

// main in main.c
pub unsafe fn kernmain() {
    kalloc::kinit1(
        (&mut end) as *mut u8,
        memlayout::p2v(4 * 1024 * 1024) as *mut u8,
    ); // phys page allocator
}
