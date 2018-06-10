use fs::*;

pub struct Buf {
    flags: i32,
    dev: u32,
    blockno: u32,
    refcnt: u32,
    prev: Option<&'static Buf>, // LRU cache list
    next: Option<&'static Buf>,
    qnext: Option<&'static Buf>, // disk queue
    data: [u8; BSIZE],
}

pub const B_VALID: i32 = 0x2; // buffer has been read from disk
pub const B_DIRTY: i32 = 0x4; // buffer needs to be written to disk
