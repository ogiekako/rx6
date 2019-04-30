use fs::*;

#[repr(C)]
pub struct Buf {
    pub flags: i32,
    pub dev: u32,
    pub blockno: u32,
    pub refcnt: u32,
    pub prev: *mut Buf, // LRU cache list
    pub next: *mut Buf,
    pub qnext: *mut Buf, // disk queue
    pub data: [u8; BSIZE],
}

impl Buf {
    pub const unsafe fn uninit() -> Buf {
        unsafe {
            Buf {
                flags: 0,
                dev: 0,
                blockno: 0,
                refcnt: 0,
                prev: core::ptr::null_mut(),
                next: core::ptr::null_mut(),
                qnext: core::ptr::null_mut(),
                data: [0; BSIZE],
            }
        }
    }
}

pub const B_VALID: i32 = 0x2; // buffer has been read from disk
pub const B_DIRTY: i32 = 0x4; // buffer needs to be written to disk
