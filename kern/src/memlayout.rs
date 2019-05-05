// Memory layout

use mmu::{P, V};

pub const EXTMEM: P = P(0x100000); // Start of extended memory
pub const PHYSTOP: P = P(0xE000000); // Top physical memory
pub const DEVSPACE: usize = 0xFE000000; // Other devices are at high addresses

// Key addresses for address space layout (see kmap in vm.c for layout)
pub const KERNBASE: V = V(0x80000000); // First kernel virtual address
pub const KERNLINK: V = V(KERNBASE.0 + EXTMEM.0); // Address where kernel is linked

pub fn v2p(v: V) -> P {
    assert!(v >= KERNBASE, "v2p");
    P(v.0.wrapping_sub(KERNBASE.0))
}

pub fn V2P(a: usize) -> usize {
    a - KERNBASE.0
}

pub unsafe extern "C" fn P2V(a: *mut u8) -> *mut u8 {
    a.add(KERNBASE.0)
}

pub const fn p2v(p: P) -> V {
    V(p.0 + KERNBASE.0)
}
