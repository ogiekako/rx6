// Memory layout

pub type V = u32;
pub type P = u32;

pub const EXTMEM: u32 = 0x100000; // Start of extended memory
pub const PHYSTOP: u32 = 0xE000000; // Top physical memory
pub const DEVSPACE: u32 = 0xFE000000; // Other devices are at high addresses

// Key addresses for address space layout (see kmap in vm.c for layout)
pub const KERNBASE: u32 = 0x80000000; // First kernel virtual address
pub const KERNLINK: u32 = (KERNBASE + EXTMEM); // Address where kernel is linked

pub fn v2p(a: V) -> P {
    a - KERNBASE
}
pub fn p2v(a: P) -> V {
    a + KERNBASE
}
