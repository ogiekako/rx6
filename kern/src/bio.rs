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

//// struct {
////   struct spinlock lock;
////   struct buf buf[NBUF];
////
////   // Linked list of all buffers, through prev/next.
////   // head.next is most recently used.
////   struct buf head;
//// } bcache;

#[repr(C)]
#[derive(Default)]
struct Bcache {
    buf: [Buf; NBUF],
    head: Buf,
}

lazy_static! {
    static ref bcache: Mutex<Bcache> = Mutex::new(Bcache::default());
}

pub unsafe fn binit() {
    let mut bcache2 = bcache.lock();

    //    let a = hoge.hoge();
    // Create linked list of buffers

    // bcache2.head.prev = core::mem::transmute(&mut bcache2.head);
    // bcache2.head.next = core::mem::transmute(&mut bcache2.head);
    // for i in 0..NBUF {
    //     let mut b: &'static mut Buf = core::mem::transmute(&mut bcache2.buf[i]);
    //     b.next = core::mem::transmute_copy(&mut bcache2.head.next);
    //     b.prev = core::mem::transmute(&mut bcache2.head);
    //     ////    initsleeplock(&b->lock, "buffer");
    //     bcache2.head.next.prev = core::mem::transmute_copy(&b);
    //     bcache2.head.next = b;
    // }
}

// Look through buffer cache for block on device dev.
// If not found, allocate a buffer.
// In either case, return locked buffer.
//// static struct buf*
//// bget(uint dev, uint blockno)
//// {
////   struct buf *b;
////
////   acquire(&bcache.lock);
////
////   // Is the block already cached?
////   for(b = bcache.head.next; b != &bcache.head; b = b->next){
////     if(b->dev == dev && b->blockno == blockno){
////       b->refcnt++;
////       release(&bcache.lock);
////       acquiresleep(&b->lock);
////       return b;
////     }
////   }
////
////   // Not cached; recycle some unused buffer and clean buffer
////   // "clean" because B_DIRTY and not locked means log.c
////   // hasn't yet committed the changes to the buffer.
////   for(b = bcache.head.prev; b != &bcache.head; b = b->prev){
////     if(b->refcnt == 0 && (b->flags & B_DIRTY) == 0) {
////       b->dev = dev;
////       b->blockno = blockno;
////       b->flags = 0;
////       b->refcnt = 1;
////       release(&bcache.lock);
////       acquiresleep(&b->lock);
////       return b;
////     }
////   }
////   panic("bget: no buffers");
//// }
////
//// // Return a locked buf with the contents of the indicated block.
//// struct buf*
//// bread(uint dev, uint blockno)
//// {
////   struct buf *b;
////
////   b = bget(dev, blockno);
////   if(!(b->flags & B_VALID)) {
////     iderw(b);
////   }
////   return b;
//// }
////
//// // Write b's contents to disk.  Must be locked.
//// void
//// bwrite(struct buf *b)
//// {
////   if(!holdingsleep(&b->lock))
////     panic("bwrite");
////   b->flags |= B_DIRTY;
////   iderw(b);
//// }
////
//// // Release a locked buffer.
//// // Move to the head of the MRU list.
//// void
//// brelse(struct buf *b)
//// {
////   if(!holdingsleep(&b->lock))
////     panic("brelse");
////
////   releasesleep(&b->lock);
////
////   acquire(&bcache.lock);
////   b->refcnt--;
////   if (b->refcnt == 0) {
////     // no one is waiting for it.
////     b->next->prev = b->prev;
////     b->prev->next = b->next;
////     b->next = bcache.head.next;
////     b->prev = &bcache.head;
////     bcache.head.next->prev = b;
////     bcache.head.next = b;
////   }
////
////   release(&bcache.lock);
//// }
