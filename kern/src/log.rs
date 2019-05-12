use super::*;

// Simple logging that allows concurrent FS system calls.
//
// A log transaction contains the updates of multiple FS system
// calls. The logging system only commits when there are
// no FS system calls active. Thus there is never
// any reasoning required about whether a commit might
// write an uncommitted system call's updates to disk.
//
// A system call should call begin_op()/end_op() to mark
// its start and end. Usually begin_op() just increments
// the count of in-progress FS system calls and returns.
// But if it thinks the log is close to running out, it
// sleeps until the last outstanding end_op() commits.
//
// The log is a physical re-do log containing disk blocks.
// The on-disk log format:
//   header block, containing block #s for block A, B, C, ...
//   block A
//   block B
//   block C
//   ...
// Log appends are synchronous.

// Contents of the header block, used for both the on-disk header block
// and to keep track in memory of logged block# before commit.
#[repr(C)]
pub struct Logheader {
    pub n: i32,
    pub block: [i32; LOGSIZE],
}

#[repr(C)]
pub struct Log {
    pub lock: Spinlock,
    pub start: i32,
    pub size: i32,
    pub outstanding: i32, // how many FS sys calls are executing.
    pub committing: i32,  // in commit(), please wait.
    pub dev: i32,
    pub lh: Logheader,
}

pub static mut log: Log = unsafe { core::mem::transmute([0u8; core::mem::size_of::<Log>()]) };

pub unsafe extern "C" fn initlog(dev: i32) {
    if (core::mem::size_of::<Logheader>() >= BSIZE) {
        cpanic("initlog: too big logheader");
    }

    let sp: Superblock = core::mem::transmute([0u8; core::mem::size_of::<Superblock>()]);
    initlock(&mut log.lock as *mut Spinlock, "log");
    readsb(dev, &mut sb as *mut Superblock);
    log.start = sb.logstart as i32;
    log.size = sb.nlog as i32;
    log.dev = dev;
    recover_from_log();
}

// Copy committed blocks from log to their home location
pub unsafe extern "C" fn install_trans() {
    for tail in 0..log.lh.n {
        let lbuf = bread(log.dev as usize, (log.start + tail + 1) as usize); // read log block
        let dbuf = bread(log.dev as usize, log.lh.block[tail as usize] as usize); // read dst
        memmove((*dbuf).data.as_mut_ptr(), (*lbuf).data.as_ptr(), BSIZE); // copy block to dst
        bwrite(dbuf); // write dst to disk
        brelse(lbuf);
        brelse(dbuf);
    }
}

// Read the log header from disk into the in-memory log header
pub unsafe extern "C" fn read_head() {
    let mut buf = bread(log.dev as usize, log.start as usize);
    let mut lh = (*buf).data.as_mut_ptr() as *mut Logheader;
    log.lh.n = (*lh).n;
    for i in 0..log.lh.n {
        log.lh.block[i as usize] = (*lh).block[i as usize];
    }
    brelse(buf);
}

// Write in-memory log header to disk.
// This is the true point at which the
// current transaction commits.
pub unsafe extern "C" fn write_head() {
    let buf = bread(log.dev as usize, log.start as usize);
    let hb = (*buf).data.as_mut_ptr() as *mut Logheader;
    (*hb).n = log.lh.n;
    for i in 0..log.lh.n {
        (*hb).block[i as usize] = log.lh.block[i as usize];
    }
    bwrite(buf);
    brelse(buf);
}

pub unsafe extern "C" fn recover_from_log() {
    read_head();
    install_trans(); // if committed, copy from log to disk
    log.lh.n = 0;
    write_head(); // clear the log
}

// called at the start of each FS system call.
pub unsafe extern "C" fn begin_op() {
    acquire(&mut log.lock as *mut Spinlock);
    loop {
        if (log.committing != 0) {
            sleep(
                &mut log as *mut Log as *mut (),
                &mut log.lock as *mut Spinlock,
            );
        } else if (log.lh.n + (log.outstanding + 1) * MAXOPBLOCKS as i32 > LOGSIZE as i32) {
            // this op might exhaust log space; wait for commit.
            sleep(
                &mut log as *mut Log as *mut (),
                &mut log.lock as *mut Spinlock,
            );
        } else {
            log.outstanding += 1;
            release(&mut log.lock as *mut Spinlock);
            break;
        }
    }
}

// called at the end of each FS system call.
// commits if this was the last outstanding operation.
pub unsafe extern "C" fn end_op() {
    let mut do_commit = 0i32;

    acquire(&mut log.lock as *mut Spinlock);
    log.outstanding -= 1;
    if (log.committing != 0) {
        cpanic("log.committing");
    }
    if (log.outstanding == 0) {
        do_commit = 1;
        log.committing = 1;
    } else {
        // begin_op() may be waiting for log space.
        wakeup(&mut log as *mut Log as *mut ());
    }
    release(&mut log.lock as *mut Spinlock);

    if (do_commit != 0) {
        // call commit w/o holding locks, since not allowed
        // to sleep with locks.
        commit();
        acquire(&mut log.lock as *mut Spinlock);
        log.committing = 0;
        wakeup(&mut log as *mut Log as *mut ());
        release(&mut log.lock as *mut Spinlock);
    }
}

// Copy modified blocks from cache to log.
pub unsafe extern "C" fn write_log() {
    for tail in 0..log.lh.n {
        let to = bread(log.dev as usize, (log.start + tail + 1) as usize); // log block
        let from = bread(log.dev as usize, log.lh.block[tail as usize] as usize); // cache block
        memmove((*to).data.as_mut_ptr(), (*from).data.as_ptr(), BSIZE);
        bwrite(to); // write the log
        brelse(from);
        brelse(to);
    }
}

pub unsafe extern "C" fn commit() {
    if (log.lh.n > 0) {
        write_log(); // Write modified blocks from cache to log
        write_head(); // Write header to disk -- the real commit
        install_trans(); // Now install writes to home locations
        log.lh.n = 0;
        write_head(); // Erase the transaction from the log
    }
}

// Caller has modified b->data and is done with the buffer.
// Record the block number and pin in the cache with B_DIRTY.
// commit()/write_log() will do the disk write.
//
// log_write() replaces bwrite(); a typical use is:
//   bp = bread(...)
//   modify bp->data[]
//   log_write(bp)
//   brelse(bp)
pub unsafe extern "C" fn log_write(b: *mut Buf) {
    if (log.lh.n >= LOGSIZE as i32 || log.lh.n >= log.size - 1) {
        cpanic("too big a transaction");
    }
    if (log.outstanding < 1) {
        cpanic("log_write outside of trans");
    }

    acquire(&mut log.lock as *mut Spinlock);
    let mut i = 0usize;
    while i < log.lh.n as usize {
        if (log.lh.block[i] == (*b).blockno as i32) {
            // log absorbtion
            break;
        }
        i += 1;
    }
    log.lh.block[i] = (*b).blockno as i32;
    if (i as i32 == log.lh.n) {
        log.lh.n += 1;
    }
    (*b).flags |= B_DIRTY; // prevent eviction
    release(&mut log.lock as *mut Spinlock);
}
