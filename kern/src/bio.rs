// Buffer cache.
//
// The buffer cache is a linked list of buf structures holding
// cached copies of disk block contents.  Caching disk blocks
// in memory reduces the number of disk reads and also provides
// a synchronization point for disk blocks used by multiple processes.
//
// Interface:
// * To get a buffer for a particular disk block, call bread.
// * After changing buffer data, call bwrite to write it to disk.
// * When done with the buffer, call brelse.
// * Do not use the buffer after calling brelse.
// * Only one process at a time can use a buffer,
//     so do not keep them longer than necessary.
//
// The implementation uses two state flags internally:
// * B_VALID: the buffer data has been read from the disk.
// * B_DIRTY: the buffer data has been modified
//     and needs to be written to disk.

use super::*;

#[repr(C)]
pub struct Bcache {
    pub lock: Spinlock,
    pub buf: [Buf; NBUF],
    pub head: Buf,
}

impl Bcache {
    pub const unsafe fn uninit() -> Bcache {
        Bcache {
            lock: Spinlock::uninit(),
            buf: [
                Buf::uninit(),
                Buf::uninit(),
                Buf::uninit(),
                Buf::uninit(),
                Buf::uninit(),
                Buf::uninit(),
                Buf::uninit(),
                Buf::uninit(),
                Buf::uninit(),
                Buf::uninit(),
                Buf::uninit(),
                Buf::uninit(),
                Buf::uninit(),
                Buf::uninit(),
                Buf::uninit(),
                Buf::uninit(),
                Buf::uninit(),
                Buf::uninit(),
                Buf::uninit(),
                Buf::uninit(),
                Buf::uninit(),
                Buf::uninit(),
                Buf::uninit(),
                Buf::uninit(),
                Buf::uninit(),
                Buf::uninit(),
                Buf::uninit(),
                Buf::uninit(),
                Buf::uninit(),
                Buf::uninit(),
            ],
            head: Buf::uninit(),
        }
    }
}

static mut bcache: Bcache = unsafe { Bcache::uninit() };

pub unsafe extern "C" fn binit() {
    initlock(&mut bcache.lock as *mut Spinlock, "bcache" as *const str);

    // Create linked list of buffers
    bcache.head.prev = &mut bcache.head as *mut Buf;
    bcache.head.next = &mut bcache.head as *mut Buf;
    for i in 0..NBUF {
        let mut b = &mut bcache.buf[i] as *mut Buf;
        (*b).next = bcache.head.next;
        (*b).prev = &mut bcache.head as *mut Buf;
        initsleeplock(&mut (*b).lock as *mut Sleeplock, "buffer".as_ptr());
        (*bcache.head.next).prev = b;
        bcache.head.next = b;
    }
}

// Look through buffer cache for block on device dev.
// If not found, allocate a buffer.
// In either case, return locked buffer.
pub unsafe extern "C" fn bget(dev: usize, blockno: usize) -> *mut Buf {
    acquire(&mut bcache.lock as *mut Spinlock);

    let mut b = bcache.head.next;
    // Is the block already cached?
    while b != &mut bcache.head as *mut Buf {
        if ((*b).dev == dev && (*b).blockno == blockno) {
            (*b).refcnt += 1;
            release(&mut bcache.lock as *mut Spinlock);
            acquiresleep(&mut (*b).lock as *mut Sleeplock);
            return b;
        }
        b = (*b).next;
    }

    // Not cached; recycle some unused buffer and clean buffer
    // "clean" because B_DIRTY and not locked means log.c
    // hasn't yet committed the changes to the buffer.
    b = bcache.head.prev;
    while b != &mut bcache.head as *mut Buf {
        if ((*b).refcnt == 0 && ((*b).flags & B_DIRTY) == 0) {
            (*b).dev = dev;
            (*b).blockno = blockno;
            (*b).flags = 0;
            (*b).refcnt = 1;
            release(&mut bcache.lock as *mut Spinlock);
            acquiresleep(&mut (*b).lock as *mut Sleeplock);
            return b;
        }
        b = (*b).prev;
    }
    cpanic("bget: no buffers");
}

// Return a locked buf with the contents of the indicated block.
pub unsafe extern "C" fn bread(dev: usize, blockno: usize) -> *mut Buf {
    let b = bget(dev, blockno);
    if (!((*b).flags & B_VALID)) != 0 {
        iderw(b);
    }
    b
}

// Write b's contents to disk.  Must be locked.
pub unsafe extern "C" fn bwrite(b: *mut Buf) {
    if holdingsleep(&mut (*b).lock as *mut Sleeplock) == 0 {
        cpanic("bwrite");
    }
    (*b).flags |= B_DIRTY;
    iderw(b);
}

// Release a locked buffer.
// Move to the head of the MRU list.
pub unsafe extern "C" fn brelse(b: *mut Buf) {
    if holdingsleep(&mut (*b).lock as *mut Sleeplock) == 0 {
        cpanic("brelse");
    }

    releasesleep(&mut (*b).lock as *mut Sleeplock);

    acquire(&mut bcache.lock as *mut Spinlock);
    (*b).refcnt -= 1;
    if ((*b).refcnt == 0) {
        // no one is waiting for it.
        (*(*b).next).prev = (*b).prev;
        (*(*b).prev).next = (*b).next;
        (*b).next = bcache.head.next;
        (*b).prev = &mut bcache.head as *mut Buf;
        (*bcache.head.next).prev = b;
        bcache.head.next = b;
    }

    release(&mut bcache.lock as *mut Spinlock);
}
