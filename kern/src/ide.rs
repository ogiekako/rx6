use super::*;

// Simple PIO-based (non-DMA) IDE driver code.

const SECTOR_SIZE: usize = 512;
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

pub static mut havedisk1: i32 = 0;

// Wait for IDE disk to become ready.
pub unsafe extern "C" fn idewait(checkerr: i32) -> i32 {
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

pub unsafe extern "C" fn ideinit() {
    initlock(&mut idelock as *mut Spinlock, "ide");

    picenable(IRQ_IDE as i32);
    ioapicenable(IRQ_IDE, ncpu as usize - 1);
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

// Start the request for b.  Caller must hold idelock.
pub unsafe extern "C" fn idestart(b: *mut Buf) {
    if (b == core::ptr::null_mut()) {
        cpanic("idestart");
    }
    if ((*b).blockno >= FSSIZE) {
        cpanic("incorrect blockno");
    }
    let mut sector_per_block = BSIZE / SECTOR_SIZE;
    let mut sector = (*b).blockno * sector_per_block;
    let mut read_cmd = if (sector_per_block == 1) {
        IDE_CMD_READ
    } else {
        IDE_CMD_RDMUL
    };
    let mut write_cmd = if (sector_per_block == 1) {
        IDE_CMD_WRITE
    } else {
        IDE_CMD_WRMUL
    };

    if (sector_per_block > 7) {
        cpanic("idestart");
    }

    idewait(0);
    outb(0x3f6, 0); // generate interrupt
    outb(0x1f2, sector_per_block as u8); // number of sectors
    outb(0x1f3, (sector & 0xff) as u8);
    outb(0x1f4, ((sector >> 8) & 0xff) as u8);
    outb(0x1f5, ((sector >> 16) & 0xff) as u8);
    outb(
        0x1f6,
        (0xe0 | (((*b).dev & 1) << 4) | ((sector >> 24) & 0x0f)) as u8,
    );
    if ((*b).flags & B_DIRTY) != 0 {
        outb(0x1f7, write_cmd);
        outsl(0x1f0, (*b).data.as_mut_ptr() as *mut (), (BSIZE / 4) as i32);
    } else {
        outb(0x1f7, read_cmd);
    }
}

// Interrupt handler.
pub unsafe extern "C" fn ideintr() {
    // First queued buffer is the active request.
    acquire(&mut idelock as *mut Spinlock);
    let mut b = idequeue;
    if b == core::ptr::null_mut() {
        release(&mut idelock as *mut Spinlock);
        return;
    }
    idequeue = (*b).qnext;

    // Read data if needed.
    if (((*b).flags & B_DIRTY) == 0 && idewait(1) >= 0) {
        insl(0x1f0, (*b).data.as_mut_ptr() as *mut (), (BSIZE / 4) as i32);
    }

    // Wake process waiting for this buf.
    (*b).flags |= B_VALID;
    (*b).flags &= !B_DIRTY;
    wakeup(b as *mut ());

    // Start disk on next buf in queue.
    if (idequeue != core::ptr::null_mut()) {
        idestart(idequeue);
    }

    release(&mut idelock as *mut Spinlock);
}

// Sync buf with disk.
// If B_DIRTY is set, write buf to disk, clear B_DIRTY, set B_VALID.
// Else if B_VALID is not set, read buf from disk, set B_VALID.
pub unsafe extern "C" fn iderw(b: *mut Buf) {
    if holdingsleep(&mut (*b).lock as *mut Sleeplock) == 0 {
        cpanic("iderw: buf not locked");
    }
    if ((*b).flags & (B_VALID | B_DIRTY)) == B_VALID {
        cpanic("iderw: nothing to do");
    }
    if (*b).dev != 0 && havedisk1 == 0 {
        cpanic("iderw: ide disk 1 not present");
    }

    acquire(&mut idelock as *mut Spinlock);

    // Append b to idequeue.
    (*b).qnext = core::ptr::null_mut();
    // for(pp=&mut idequeue as *mut *mut Buf; *pp != core::ptr::null_mut(); pp=&mut (*(*pp)).qnext as *mut *mut Buf)
    let mut pp = &mut idequeue as *mut *mut Buf;
    while *pp != core::ptr::null_mut() {
        pp = &mut (*(*pp)).qnext as *mut *mut Buf;
    }
    *pp = b;

    // Start disk if necessary.
    if idequeue == b {
        idestart(b);
    }

    cprintf("iderw: start sleep\n", &[]);
    check_it("iderw (1)");
    // Wait for request to finish.
    while ((*b).flags & (B_VALID | B_DIRTY)) != B_VALID {
        sleep(b as *mut (), &mut idelock as *mut Spinlock);
    }
    cprintf("iderw: end sleep\n", &[]);

    check_it("iderw (2)");

    release(&mut idelock as *mut Spinlock);

    check_it("iderw (3)");
    cprintf("z", &[]);
}
