use mmu::V;

extern "C" {
    static mut __data: u8;
    // first address after kernel loaded from ELF file
    static mut __end: u8;
}

pub fn end() -> V {
    unsafe {
        V(&__end as *const u8 as usize)
    }
}

pub fn data() -> V {
    unsafe {
        V(&__data as *const u8 as usize)
    }
}
