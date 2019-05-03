use super::*;

#[repr(C)]
pub struct Buf {
    pub flags: i32,
    pub dev: usize,
    pub blockno: usize,
    pub lock: Sleeplock,
    pub refcnt: usize,
    pub prev: *mut Buf, // LRU cache list
    pub next: *mut Buf,
    pub qnext: *mut Buf, // disk queue
    pub data: [u8; BSIZE],
}

impl Buf {
    pub const unsafe fn uninit() -> Buf {
        core::mem::transmute([0u8; core::mem::size_of::<Buf>()])
    }
}

pub const B_VALID: i32 = 0x2; // buffer has been read from disk
pub const B_DIRTY: i32 = 0x4; // buffer needs to be written to disk
