use super::*;

#[derive(Clone, PartialEq)]
#[repr(C)]
pub enum FileType {
    FD_NONE,
    FD_PIPE,
    FD_INODE,
}

pub use FileType::*;

#[derive(Clone)]
#[repr(C)]
pub struct File {
    pub type_: FileType,
    pub ref_: i32, // reference count

    pub readable: u8,
    pub writable: u8,
    pub pipe: *mut Pipe,
    pub ip: *mut Inode,
    pub off: usize,
}

// in-memory copy of an inode
#[repr(C)]
pub struct Inode {
    pub dev: usize,  // Device number
    pub inum: usize, // Inode number
    pub ref_: i32,   // Reference count
    pub lock: Sleeplock,
    pub flags: i32, // I_VALID

    pub type_: i16, // copy of disk inode
    pub major: i16,
    pub minor: i16,
    pub nlink: i16,
    pub size: usize,
    pub addrs: [usize; NDIRECT + 1],
}
pub const I_VALID: i32 = 0x2;

// table mapping major device number to
// device functions
pub struct Devsw {
    pub read: Option<unsafe extern "C" fn(*mut Inode, *mut u8, i32) -> i32>,
    pub write: Option<unsafe extern "C" fn(*mut Inode, *mut u8, i32) -> i32>,
}

pub const CONSOLE: usize = 1;

// file.c
//
// File descriptors
//
pub static mut devsw: [Devsw; NDEV] = unsafe { transmute([0u8; size_of::<[Devsw; NDEV]>()]) };
pub struct Ftable {
    pub lock: Spinlock,
    pub file: [File; NFILE],
}

impl Ftable {
    pub const unsafe fn uninit() -> Ftable {
        Ftable {
            lock: Spinlock::uninit(),
            file: core::mem::transmute([0u8; core::mem::size_of::<[File; NFILE]>()]),
        }
    }
}

pub static mut ftable: Ftable = unsafe { Ftable::uninit() };

pub unsafe extern "C" fn fileinit() {
    initlock(&mut ftable.lock as *mut Spinlock, "ftable");
}

// Allocate a file structure.
pub unsafe extern "C" fn filealloc() -> *mut File {
    acquire(&mut ftable.lock as *mut Spinlock);
    for i in 0..NFILE {
        let mut f = &mut ftable.file[i];
        if (f.ref_ == 0) {
            f.ref_ = 1;
            release(&mut ftable.lock as *mut Spinlock);
            return f as *mut File;
        }
    }
    release(&mut ftable.lock as *mut Spinlock);
    return null_mut();
}

// Increment ref count for file f.
pub unsafe extern "C" fn filedup(f: *mut File) -> *mut File {
    acquire(&mut ftable.lock as *mut Spinlock);
    if ((*f).ref_ < 1) {
        cpanic("filedup");
    }
    (*f).ref_ += 1;
    release(&mut ftable.lock as *mut Spinlock);
    return f;
}

// Close file f.  (Decrement ref count, close when reaches 0.)
pub unsafe extern "C" fn fileclose(f: *mut File) {
    acquire(&mut ftable.lock as *mut Spinlock);
    if ((*f).ref_ < 1) {
        cpanic("fileclose");
    }
    (*f).ref_ -= 1;
    if ((*f).ref_ > 0) {
        release(&mut ftable.lock as *mut Spinlock);
        return;
    }
    let ff = (*f).clone();
    (*f).ref_ = 0;
    (*f).type_ = FD_NONE;
    release(&mut ftable.lock as *mut Spinlock);

    if (ff.type_ == FD_PIPE) {
        pipeclose(ff.pipe, ff.writable as i32);
    } else if (ff.type_ == FD_INODE) {
        begin_op();
        iput(ff.ip);
        end_op();
    }
}

// Get metadata about file f.
pub unsafe extern "C" fn filestat(f: *mut File, st: *mut Stat) -> i32 {
    if ((*f).type_ == FD_INODE) {
        ilock((*f).ip);
        stati((*f).ip, st);
        iunlock((*f).ip);
        return 0;
    }
    return -1;
}

// Read from file f.
pub unsafe extern "C" fn fileread(f: *mut File, addr: *mut u8, n: i32) -> i32 {
    if ((*f).readable == 0) {
        cprintf("fileread: not readable\n", &[]);
        return -1;
    }
    if ((*f).type_ == FD_PIPE) {
        cprintf("fileread: piperead start\n", &[]);
        let res = piperead((*f).pipe, addr, n);
        cprintf("fileread: piperead end\n", &[]);
        return res;
    }
    if ((*f).type_ == FD_INODE) {
        ilock((*f).ip);
        let r = readi((*f).ip, addr, (*f).off, n as usize);
        if (r > 0) {
            (*f).off += r as usize;
        }
        iunlock((*f).ip);
        return r;
    }
    cpanic("fileread");
}

// Write to file f.
pub unsafe extern "C" fn filewrite(f: *mut File, addr: *mut u8, n: i32) -> i32 {
    if ((*f).writable == 0) {
        return -1;
    }
    if ((*f).type_ == FD_PIPE) {
        return pipewrite((*f).pipe, addr, n);
    }
    if ((*f).type_ == FD_INODE) {
        // write a few blocks at a time to avoid exceeding
        // the maximum log transaction size, including
        // i-node, indirect block, allocation blocks,
        // and 2 blocks of slop for non-aligned writes.
        // this really belongs lower down, since writei()
        // might be writing a device like the console.
        let max = ((LOGSIZE - 1 - 1 - 2) / 2) * 512;
        let mut i = 0;
        while (i < n) {
            let mut n1 = n - i;
            if (n1 > max as i32) {
                n1 = max as i32;
            }

            begin_op();
            ilock((*f).ip);
            let r = writei((*f).ip, addr.offset(i as isize), (*f).off, n1 as usize);
            if (r > 0) {
                (*f).off += r as usize;
            }
            iunlock((*f).ip);
            end_op();

            if (r < 0) {
                break;
            }
            if (r != n1) {
                cpanic("short filewrite");
            }
            i += r;
        }
        return if i == n { n } else { -1 };
    }
    cpanic("filewrite");
}
