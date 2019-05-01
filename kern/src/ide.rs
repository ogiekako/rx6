use super::*;

// Simple PIO-based (non-DMA) IDE driver code.

const SECTOR_SIZE: u32 = 512;
const IDE_BSY: u8 = 0x80;
const IDE_DRDY: u8 = 0x40;
const IDE_DF: u8 = 0x20;
const IDE_ERR: u8 = 0x01;

const IDE_CMD_READ: u8 = 0x20;
const IDE_CMD_WRITE: u8 = 0x30;
const IDE_CMD_RDMUL: u8 = 0xc4;
const IDE_CMD_WRMUL: u8 = 0xc5;

// idequeue points to the buf now being read/written to the disk.
// idequeue->qnext points to the next buf to be processed.
// You must hold idelock while manipulating queue.

pub static mut idelock: Spinlock = unsafe { Spinlock::uninit() };
pub static mut idequeue: *mut Buf = unsafe { core::ptr::null_mut() };

static mut havedisk1: i32 = 0;
//// static void idestart(struct buf*);

// Wait for IDE disk to become ready.
pub unsafe fn idewait(checkerr: i32) -> i32 {
    let mut r;
    loop {
        r = inb(0x1f7);
        if r & (IDE_BSY | IDE_DRDY) == IDE_DRDY {
            break;
        }
    }

    if checkerr != 0 && (r & (IDE_DF | IDE_ERR)) != 0 {
        return -1;
    }
    return 0;
}

pub unsafe fn ideinit() {
    initlock(&mut idelock as *mut Spinlock, "ide");

    picenable(IRQ_IDE as i32);
    ioapicenable(IRQ_IDE, ncpu as u32 - 1);
    idewait(0);

    // Check if disk 1 is present
    outb(0x1f6, 0xe0 | (1 << 4));
    for i in 0..1000 {
        if inb(0x1f7) != 0 {
            havedisk1 = 1;
            break;
        }
    }

    // Switch back to disk 0.
    outb(0x1f6, 0xe0 | (0 << 4));
}

//// // Start the request for b.  Caller must hold idelock.
//// static void
//// idestart(struct buf *b)
//// {
////   if(b == 0)
////     panic("idestart");
////   if(b->blockno >= FSSIZE)
////     panic("incorrect blockno");
////   int sector_per_block =  BSIZE/SECTOR_SIZE;
////   int sector = b->blockno * sector_per_block;
////   int read_cmd = (sector_per_block == 1) ? IDE_CMD_READ :  IDE_CMD_RDMUL;
////   int write_cmd = (sector_per_block == 1) ? IDE_CMD_WRITE : IDE_CMD_WRMUL;
////
////   if (sector_per_block > 7) panic("idestart");
////
////   idewait(0);
////   outb(0x3f6, 0);  // generate interrupt
////   outb(0x1f2, sector_per_block);  // number of sectors
////   outb(0x1f3, sector & 0xff);
////   outb(0x1f4, (sector >> 8) & 0xff);
////   outb(0x1f5, (sector >> 16) & 0xff);
////   outb(0x1f6, 0xe0 | ((b->dev&1)<<4) | ((sector>>24)&0x0f));
////   if(b->flags & B_DIRTY){
////     outb(0x1f7, write_cmd);
////     outsl(0x1f0, b->data, BSIZE/4);
////   } else {
////     outb(0x1f7, read_cmd);
////   }
//// }
////
//// // Interrupt handler.
//// void
//// ideintr(void)
//// {
////   struct buf *b;
////
////   // First queued buffer is the active request.
////   acquire(&idelock);
////   if((b = idequeue) == 0){
////     release(&idelock);
////     // cprintf("spurious IDE interrupt\n");
////     return;
////   }
////   idequeue = b->qnext;
////
////   // Read data if needed.
////   if(!(b->flags & B_DIRTY) && idewait(1) >= 0)
////     insl(0x1f0, b->data, BSIZE/4);
////
////   // Wake process waiting for this buf.
////   b->flags |= B_VALID;
////   b->flags &= ~B_DIRTY;
////   wakeup(b);
////
////   // Start disk on next buf in queue.
////   if(idequeue != 0)
////     idestart(idequeue);
////
////   release(&idelock);
//// }
////
//// //PAGEBREAK!
//// // Sync buf with disk.
//// // If B_DIRTY is set, write buf to disk, clear B_DIRTY, set B_VALID.
//// // Else if B_VALID is not set, read buf from disk, set B_VALID.
//// void
//// iderw(struct buf *b)
//// {
////   struct buf **pp;
////
////   if(!holdingsleep(&b->lock))
////     panic("iderw: buf not locked");
////   if((b->flags & (B_VALID|B_DIRTY)) == B_VALID)
////     panic("iderw: nothing to do");
////   if(b->dev != 0 && !havedisk1)
////     panic("iderw: ide disk 1 not present");
////
////   acquire(&idelock);  //DOC:acquire-lock
////
////   // Append b to idequeue.
////   b->qnext = 0;
////   for(pp=&idequeue; *pp; pp=&(*pp)->qnext)  //DOC:insert-queue
////     ;
////   *pp = b;
////
////   // Start disk if necessary.
////   if(idequeue == b)
////     idestart(b);
////
////   // Wait for request to finish.
////   while((b->flags & (B_VALID|B_DIRTY)) != B_VALID){
////     sleep(b, &idelock);
////   }
////
////   release(&idelock);
//// }
