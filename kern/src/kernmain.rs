/// main.c in xv6

// memlayout.h
const KERNBASE:u32 = 0x80000000;         // First kernel virtual address

// mmu.h
const PGSIZE:i32 = 4096;
const NPDENTRIES:i32 = 1024;    // # directory entries per page directory;

const PTE_P:i32      =     0x001;   // Present
const PTE_W:i32      =     0x002;   // Writeable
const PTE_PS:i32    =     0x080 ;  // Page Size

const PDXSHIFT:i32 = 22;

pub fn kernmain() {
}

