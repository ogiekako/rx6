use fs::*;

#[repr(C)]
pub struct Buf {
    pub flags: i32,
    pub dev: u32,
    pub blockno: u32,
    pub refcnt: u32,
    pub prev: &'static mut Buf, // LRU cache list
    pub next: &'static mut Buf,
    pub qnext: &'static mut Buf, // disk queue
    pub data: [u8; BSIZE],
}

pub const B_VALID: i32 = 0x2; // buffer has been read from disk
pub const B_DIRTY: i32 = 0x4; // buffer needs to be written to disk
