use fs::*;

#[repr(C)]
pub struct Buf {
    pub flags: i32,
    pub dev: u32,
    pub blockno: u32,
    pub refcnt: u32,
    // pub prev: *mut Buf, // LRU cache list
    //    pub next: &'static mut Buf,
    //    pub qnext: &'static mut Buf, // disk queue
    pub data: [u8; SZ],
}

const SZ: usize = 217;

impl core::default::Default for Buf {
    fn default() -> Buf {
        unsafe {
            Buf {
                flags: 0,
                dev: 0,
                blockno: 0,
                refcnt: 0,
                //          prev: core::mem::zeroed(),
                //          prev: core::mem::uninitialized(),
                data: [0; SZ],
            }
        }
    }
}

pub const B_VALID: i32 = 0x2; // buffer has been read from disk
pub const B_DIRTY: i32 = 0x4; // buffer needs to be written to disk
