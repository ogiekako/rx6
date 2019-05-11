use super::*;

// On-disk file system format.
// Both the kernel and user programs use this header file.

pub const ROOTINO: usize = 1; // root i-number
pub const BSIZE: usize = 512; // block size

// Disk layout:
// [ boot block | super block | log | inode blocks |
//                                          free bit map | data blocks]
//
// mkfs computes the super block and builds an initial file system. The
// super block describes the disk layout:
#[repr(C)]
pub struct Superblock {
    pub size: usize,       // Size of file system image (blocks)
    pub nblocks: usize,    // Number of data blocks
    pub ninodes: usize,    // Number of inodes.
    pub nlog: usize,       // Number of log blocks
    pub logstart: usize,   // Block number of first log block
    pub inodestart: usize, // Block number of first inode block
    pub bmapstart: usize,  // Block number of first free map block
}

pub const NDIRECT: usize = 12;
pub const NINDIRECT: usize = (BSIZE / core::mem::size_of::<usize>());
pub const MAXFILE: usize = (NDIRECT + NINDIRECT);

// On-disk inode structure
#[repr(C)]
pub struct Dinode {
    pub type_: i16,                  // File type
    pub major: i16,                  // Major device number (T_DEV only)
    pub minor: i16,                  // Minor device number (T_DEV only)
    pub nlink: i16,                  // Number of links to inode in file system
    pub size: usize,                 // Size of file (bytes)
    pub addrs: [usize; NDIRECT + 1], // Data block addresses
}

// Inodes per block.
pub const IPB: usize = (BSIZE / core::mem::size_of::<Dinode>());

// Block containing inode i
macro_rules! IBLOCK {
    ($i: expr, $sb: ident) => {
        $i / IPB + $sb.inodestart
    };
}

// Bitmap bits per block
pub const BPB: usize = (BSIZE * 8);

// Block of free map containing bit for block b
macro_rules! BBLOCK {
    ($b: expr, $sb: ident) => {
        $b / BPB + $sb.bmapstart
    };
}

// Directory is a file containing a sequence of dirent structures.
pub const DIRSIZ: usize = 14;

#[repr(C)]
pub struct Dirent {
    pub inum: u16,
    pub name: [u8; DIRSIZ],
}

// File system implementation.  Five layers:
//   + Blocks: allocator for raw disk blocks.
//   + Log: crash recovery for multi-step updates.
//   + Files: inode allocator, reading, writing, metadata.
//   + Directories: inode with special contents (list of other inodes!)
//   + Names: paths like /usr/rtm/xv6/fs.c for convenient naming.
//
// This file contains the low-level file system manipulation
// routines.  The (higher-level) system call implementations
// are in sysfile.c.

// there should be one superblock per disk device, but we run with
// only one device
pub static mut sb: Superblock =
    unsafe { core::mem::transmute([0u8; core::mem::size_of::<Superblock>()]) };

// Read the super block.
pub unsafe extern "C" fn readsb(dev: i32, sb_: *mut Superblock) {
    let bp = bread(dev as usize, 1);
    memmove(
        sb_ as *mut u8,
        (*bp).data.as_ptr(),
        core::mem::size_of_val(&(*sb_)),
    );
    brelse(bp);
}

// Zero a block.
pub unsafe extern "C" fn bzero(dev: i32, bno: i32) {
    let bp = bread(dev as usize, bno as usize);
    memset((*bp).data.as_mut_ptr(), 0, BSIZE);
    log_write(bp);
    brelse(bp);
}

// Blocks.

// Allocate a zeroed disk block.
pub unsafe extern "C" fn balloc(dev: usize) -> usize {
    for b in (0..(sb.size)).step_by(BPB) {
        let bp = bread(dev, BBLOCK!(b, sb));
        let mut bi = 0;
        while bi < BPB && b + bi < sb.size {
            let m = 1 << (bi % 8);
            if (((*bp).data[bi / 8] & m) == 0) {
                // Is block free?
                (*bp).data[bi / 8] |= m; // Mark block in use.
                log_write(bp);
                brelse(bp);
                bzero(dev as i32, (b + bi) as i32);
                return b + bi;
            }
            bi += 1;
        }
        brelse(bp);
    }
    cpanic("balloc: out of blocks");
}

// Free a disk block.
pub unsafe extern "C" fn bfree(dev: usize, b: usize) {
    readsb(dev as i32, &mut sb as *mut Superblock);
    let bp = bread(dev, BBLOCK!(b, sb));
    let bi = b % BPB;
    let m = 1 << (bi % 8);
    if (((*bp).data[bi / 8] & m) == 0) {
        cpanic("freeing free block");
    }
    (*bp).data[bi / 8] &= !m;
    log_write(bp);
    brelse(bp);
}

// Inodes.
//
// An inode describes a single unnamed file.
// The inode disk structure holds metadata: the file's type,
// its size, the number of links referring to it, and the
// list of blocks holding the file's content.
//
// The inodes are laid out sequentially on disk at
// sb.startinode. Each inode has a number, indicating its
// position on the disk.
//
// The kernel keeps a cache of in-use inodes in memory
// to provide a place for synchronizing access
// to inodes used by multiple processes. The cached
// inodes include book-keeping information that is
// not stored on disk: ip->ref and ip->flags.
//
// An inode and its in-memory represtative go through a
// sequence of states before they can be used by the
// rest of the file system code.
//
// * Allocation: an inode is allocated if its type (on disk)
//   is non-zero. ialloc() allocates, iput() frees if
//   the link count has fallen to zero.
//
// * Referencing in cache: an entry in the inode cache
//   is free if ip->ref is zero. Otherwise ip->ref tracks
//   the number of in-memory pointers to the entry (open
//   files and current directories). iget() to find or
//   create a cache entry and increment its ref, iput()
//   to decrement ref.
//
// * Valid: the information (type, size, &c) in an inode
//   cache entry is only correct when the I_VALID bit
//   is set in ip->flags. ilock() reads the inode from
//   the disk and sets I_VALID, while iput() clears
//   I_VALID if ip->ref has fallen to zero.
//
// * Locked: file system code may only examine and modify
//   the information in an inode and its content if it
//   has first locked the inode.
//
// Thus a typical sequence is:
//   ip = iget(dev, inum)
//   ilock(ip)
//   ... examine and modify ip->xxx ...
//   iunlock(ip)
//   iput(ip)
//
// ilock() is separate from iget() so that system calls can
// get a long-term reference to an inode (as for an open file)
// and only lock it for short periods (e.g., in read()).
// The separation also helps avoid deadlock and races during
// pathname lookup. iget() increments ip->ref so that the inode
// stays cached and pointers to it remain valid.
//
// Many internal file system functions expect the caller to
// have locked the inodes involved; this lets callers create
// multi-step atomic operations.

struct Icache {
    lock: Spinlock,
    inode: [Inode; NINODE],
}
static mut icache: Icache = unsafe { core::mem::transmute([0u8; core::mem::size_of::<Icache>()]) };

pub unsafe extern "C" fn iinit(dev: i32) {
    initlock(&mut icache.lock as *mut Spinlock, "icache");
    for i in 0..NINODE {
        initsleeplock(
            &mut icache.inode[i].lock as *mut Sleeplock,
            "inode\0".as_ptr(),
        );
    }

    readsb(dev, &mut sb as *mut Superblock);
    cprintf(
        "sb: size %d nblocks %d ninodes %d nlog %d logstart %d inodestart %d bmap start %d\n",
        &[
            Arg::Int(sb.size as i32),
            Arg::Int(sb.nblocks as i32),
            Arg::Int(sb.ninodes as i32),
            Arg::Int(sb.nlog as i32),
            Arg::Int(sb.logstart as i32),
            Arg::Int(sb.inodestart as i32),
            Arg::Int(sb.bmapstart as i32),
        ],
    );
}
//PAGEBREAK!
// Allocate a new inode with the given type on device dev.
// A free inode has a type of zero.
pub unsafe extern "C" fn ialloc(dev: usize, type_: i16) -> *mut Inode {
    for inum in 1..(sb.ninodes) {
        let bp = bread(dev, IBLOCK!(inum, sb));
        let dip = ((*bp).data.as_mut_ptr() as *mut Dinode).add(inum % IPB);
        if ((*dip).type_ == 0) {
            // a free inode
            memset(dip as *mut u8, 0, size_of_val(&(*dip)));
            (*dip).type_ = type_;
            log_write(bp); // mark it allocated on the disk
            brelse(bp);
            return iget(dev, inum);
        }
        brelse(bp);
    }
    cpanic("ialloc: no inodes");
}

// Copy a modified in-memory inode to disk.
pub unsafe extern "C" fn iupdate(ip: *mut Inode) {
    let bp = bread((*ip).dev, IBLOCK!((*ip).inum, sb));
    let dip = ((*bp).data.as_mut_ptr() as *mut Dinode).offset(((*ip).inum % IPB) as isize);
    (*dip).type_ = (*ip).type_;
    (*dip).major = (*ip).major;
    (*dip).minor = (*ip).minor;
    (*dip).nlink = (*ip).nlink;
    (*dip).size = (*ip).size;
    memmove(
        (*dip).addrs.as_mut_ptr() as *mut u8,
        (*ip).addrs.as_mut_ptr() as *mut u8,
        size_of_val(&(*ip).addrs),
    );
    log_write(bp);
    brelse(bp);
}

// Find the inode with number inum on device dev
// and return the in-memory copy. Does not lock
// the inode and does not read it from disk.
pub unsafe extern "C" fn iget(dev: usize, inum: usize) -> *mut Inode {
    check_it("iget (1)");
    acquire(&mut icache.lock as *mut Spinlock);
    check_it("iget (2)");

    // Is the inode already cached?
    let mut empty: *mut Inode = core::ptr::null_mut();
    let mut ip: *mut Inode;
    for i in 0..NINODE {
        ip = &mut icache.inode[i] as *mut Inode;
        if (*ip).ref_ > 0 && (*ip).dev == dev && (*ip).inum == inum {
            (*ip).ref_ += 1;
            release(&mut icache.lock as *mut Spinlock);
            cprintf("iget: a  type: %d\n", &[Arg::Int((*ip).type_ as i32)]);
            check_it("iget (3)");
            return ip;
        }
        if empty.is_null() && (*ip).ref_ == 0 {
            // Remember empty slot.
            empty = ip;
        }
        check_it("iget (4)");
    }

    // Recycle an inode cache entry.
    if (empty == core::ptr::null_mut()) {
        cpanic("iget: no inodes");
    }

    ip = empty;
    (*ip).dev = dev;
    (*ip).inum = inum;
    (*ip).ref_ = 1;
    (*ip).flags = 0;
    release(&mut icache.lock as *mut Spinlock);
    check_it("iget (5)");

    ip
}

// Increment reference count for ip.
// Returns ip to enable ip = idup(ip1) idiom.
pub unsafe extern "C" fn idup(ip: *mut Inode) -> *mut Inode {
    acquire(&mut icache.lock as *mut Spinlock);
    (*ip).ref_ += 1;
    release(&mut icache.lock as *mut Spinlock);
    return ip;
}

// Lock the given inode.
// Reads the inode from disk if necessary.
pub unsafe extern "C" fn ilock(ip: *mut Inode) {
    if (ip == core::ptr::null_mut() || (*ip).ref_ < 1) {
        cpanic("ilock");
    }

    acquiresleep(&mut (*ip).lock as *mut Sleeplock);

    if (((*ip).flags & I_VALID) == 0) {
        let bp = bread((*ip).dev, IBLOCK!((*ip).inum, sb));
        let dip = ((*bp).data.as_mut_ptr() as *mut Dinode).offset(((*ip).inum % IPB) as isize);
        (*ip).type_ = (*dip).type_;
        (*ip).major = (*dip).major;
        (*ip).minor = (*dip).minor;
        (*ip).nlink = (*dip).nlink;
        (*ip).size = (*dip).size;
        memmove(
            (*ip).addrs.as_mut_ptr() as *mut u8,
            (*dip).addrs.as_mut_ptr() as *mut u8,
            core::mem::size_of_val(&(*ip).addrs),
        );
        brelse(bp);
        (*ip).flags |= I_VALID;
        if ((*ip).type_ == 0) {
            cpanic("ilock: no type");
        }
    }
}

// Unlock the given inode.
pub unsafe extern "C" fn iunlock(ip: *mut Inode) {
    if (ip == core::ptr::null_mut()
        || holdingsleep(&mut (*ip).lock as *mut Sleeplock) == 0
        || (*ip).ref_ < 1)
    {
        cpanic("iunlock");
    }

    releasesleep(&mut (*ip).lock as *mut Sleeplock);
}

// Drop a reference to an in-memory inode.
// If that was the last reference, the inode cache entry can
// be recycled.
// If that was the last reference and the inode has no links
// to it, free the inode (and its content) on disk.
// All calls to iput() must be inside a transaction in
// case it has to free the inode.
pub unsafe extern "C" fn iput(ip: *mut Inode) {
    acquire(&mut icache.lock as *mut Spinlock);
    if ((*ip).ref_ == 1 && ((*ip).flags & I_VALID) != 0 && (*ip).nlink == 0) {
        // inode has no links and no other references: truncate and free.
        release(&mut icache.lock as *mut Spinlock);
        itrunc(ip);
        (*ip).type_ = 0;
        iupdate(ip);
        acquire(&mut icache.lock as *mut Spinlock);
        (*ip).flags = 0;
    }
    (*ip).ref_ -= 1;
    release(&mut icache.lock as *mut Spinlock);
}

// Common idiom: unlock, then put.
pub unsafe extern "C" fn iunlockput(ip: *mut Inode) {
    iunlock(ip);
    iput(ip);
}

// Inode content
//
// The content (data) associated with each inode is stored
// in blocks on the disk. The first NDIRECT block numbers
// are listed in ip->addrs[].  The next NINDIRECT blocks are
// listed in block ip->addrs[NDIRECT].

// Return the disk block address of the nth block in inode ip.
// If there is no such block, bmap allocates one.
pub unsafe extern "C" fn bmap(ip: *mut Inode, mut bn: usize) -> usize {
    if (bn < NDIRECT) {
        let mut addr = (*ip).addrs[bn];
        if (addr == 0) {
            addr = balloc((*ip).dev);
            (*ip).addrs[bn] = addr;
        }
        return addr;
    }
    bn -= NDIRECT;

    if (bn < NINDIRECT) {
        // Load indirect block, allocating if necessary.
        let mut addr = (*ip).addrs[NDIRECT];
        if (addr == 0) {
            addr = balloc((*ip).dev);
            (*ip).addrs[NDIRECT] = addr;
        }
        let bp = bread((*ip).dev, addr);
        let a = (*bp).data.as_ptr() as *mut usize;
        addr = *(a.add(bn));
        if (addr == 0) {
            addr = balloc((*ip).dev);
            *(a.add(bn)) = addr;
            log_write(bp);
        }
        brelse(bp);
        return addr;
    }

    cpanic("bmap: out of range");
}

// Truncate inode (discard contents).
// Only called when the inode has no links
// to it (no directory entries referring to it)
// and has no in-memory reference to it (is
// not an open file or current directory).
pub unsafe extern "C" fn itrunc(ip: *mut Inode) {
    for i in 0..NDIRECT {
        if ((*ip).addrs[i]) != 0 {
            bfree((*ip).dev, (*ip).addrs[i]);
            (*ip).addrs[i] = 0;
        }
    }

    if ((*ip).addrs)[NDIRECT] != 0 {
        let bp = bread((*ip).dev, (*ip).addrs[NDIRECT]);
        let a = (*bp).data.as_mut_ptr() as *mut usize;
        for j in 0..NINDIRECT {
            if (*(a.add(j))) != 0 {
                bfree((*ip).dev, *(a.add(j)));
            }
        }
        brelse(bp);
        bfree((*ip).dev, (*ip).addrs[NDIRECT]);
        (*ip).addrs[NDIRECT] = 0;
    }

    (*ip).size = 0;
    iupdate(ip);
}

// Copy stat information from inode.
pub unsafe extern "C" fn stati(ip: *mut Inode, st: *mut Stat) {
    (*st).dev = (*ip).dev as i32;
    (*st).ino = (*ip).inum;
    (*st).type_ = (*ip).type_;
    (*st).nlink = (*ip).nlink;
    (*st).size = (*ip).size;
}

// Read data from inode.
pub unsafe extern "C" fn readi(
    ip: *mut Inode,
    mut dst: *mut u8,
    mut off: usize,
    mut n: usize,
) -> i32 {
    if ((*ip).type_ == T_DEV as i16) {
        if ((*ip).major < 0
            || (*ip).major >= NDEV as i16
            || devsw[(*ip).major as usize].read.is_none())
        {
            return -1;
        }
        return devsw[(*ip).major as usize].read.unwrap()(ip, dst, n as i32);
    }

    if (off > (*ip).size || off + n < off) {
        return -1;
    }
    if (off + n > (*ip).size) {
        n = (*ip).size - off;
    }

    let mut tot = 0;
    while tot < n {
        let bp = bread((*ip).dev, bmap(ip, off / BSIZE));
        let m = core::cmp::min(n - tot, BSIZE - off % BSIZE);
        /*
        cprintf("data off %d:\n", off);
        for (int j = 0; j < min(m, 10); j++) {
          cprintf("%x ", bp->data[off%BSIZE+j]);
        }
        cprintf("\n");
        */
        memmove(dst, (*bp).data.as_ptr().add(off % BSIZE), m);
        brelse(bp);
        tot += m;
        off += m;
        dst = dst.offset(m as isize);
    }
    return n as i32;
}

// Write data to inode.
pub unsafe extern "C" fn writei(ip: *mut Inode, mut src: *mut u8, mut off: usize, n: usize) -> i32 {
    if ((*ip).type_ == T_DEV as i16) {
        if ((*ip).major < 0
            || (*ip).major >= NDEV as i16
            || devsw[(*ip).major as usize].write.is_none())
        {
            return -1;
        }
        return devsw[(*ip).major as usize].write.unwrap()(ip, src, n as i32);
    }

    if (off > (*ip).size || off + n < off) {
        return -1;
    }
    if (off + n > MAXFILE * BSIZE) {
        return -1;
    }

    let mut tot = 0;
    while tot < n {
        let bp = bread((*ip).dev, bmap(ip, off / BSIZE));
        let m = core::cmp::min(n - tot, BSIZE - off % BSIZE);
        memmove((*bp).data.as_mut_ptr().add(off % BSIZE), src, m);
        log_write(bp);
        brelse(bp);
        tot += m;
        off += m;
        src = src.offset(m as isize);
    }

    if (n > 0 && off > (*ip).size) {
        (*ip).size = off as usize;
        iupdate(ip);
    }
    return n as i32;
}

// Directories
pub unsafe extern "C" fn namecmp(s: *const u8, t: *const u8) -> i32 {
    return strncmp(s, t, DIRSIZ);
}

// Look for a directory entry in a directory.
// If found, set *poff to byte offset of entry.
pub unsafe extern "C" fn dirlookup(
    dp: *mut Inode,
    name: *const u8,
    poff: *mut usize,
) -> *mut Inode {
    if ((*dp).type_ != T_DIR as i16) {
        cpanic("dirlookup not DIR");
    }

    let mut de: Dirent = core::mem::transmute([0u8; core::mem::size_of::<Dirent>()]);

    for off in (0..(*dp).size).step_by(core::mem::size_of_val(&de)) {
        if (readi(dp, &mut de as *mut Dirent as *mut u8, off, size_of_val(&de))
            != size_of_val(&de) as i32)
        {
            cpanic("dirlink read");
        }
        if (de.inum == 0) {
            continue;
        }
        if (namecmp(name, de.name.as_ptr()) == 0) {
            // entry matches path element
            if (poff != null_mut()) {
                *poff = off;
            }
            let inum = de.inum;
            return iget((*dp).dev, inum as usize);
        }
    }

    return null_mut();
}

// Write a new directory entry (name, inum) into the directory dp.
pub unsafe extern "C" fn dirlink(dp: *mut Inode, name: *const u8, inum: usize) -> i32 {
    let mut de: Dirent = transmute([0u8; size_of::<Dirent>()]);

    // Check that name is not present.
    let ip = dirlookup(dp, name, null_mut());
    if (ip != null_mut()) {
        iput(ip);
        return -1;
    }

    let mut off = 0;
    // Look for an empty dirent.
    while off < (*dp).size {
        if (readi(dp, &mut de as *mut Dirent as *mut u8, off, size_of_val(&de))
            != size_of_val(&de) as i32)
        {
            cpanic("dirlink read");
        }
        if (de.inum == 0) {
            break;
        }
        off += size_of_val(&de);
    }

    strncpy(de.name.as_mut_ptr(), name, DIRSIZ as i32);
    de.inum = inum as u16;
    if (writei(dp, &mut de as *mut Dirent as *mut u8, off, size_of_val(&de))
        != size_of_val(&de) as i32)
    {
        cpanic("dirlink");
    }

    return 0;
}

// Paths

// Copy the next path element from path into name.
// Return a pointer to the element following the copied one.
// The returned path has no leading slashes,
// so the caller can check *path=='\0' to see if the name is the last one.
// If no name to remove, return 0.
//
// Examples:
//   skipelem("a/bb/c", name) = "bb/c", setting name = "a"
//   skipelem("///a//bb", name) = "bb", setting name = "a"
//   skipelem("a", name) = "", setting name = "a"
//   skipelem("", name) = skipelem("////", name) = 0
//
unsafe extern "C" fn skipelem(mut path: *const u8, name: *mut u8) -> *const u8 {
    while (*path == b'/') {
        path = path.offset(1);
    }
    if (*path == 0) {
        return null_mut();
    }
    let s = path;
    while (*path != b'/' && *path != 0) {
        path = path.offset(1);
    }
    let len = path.offset_from(s);
    if (len >= DIRSIZ as isize) {
        memmove(name, s, DIRSIZ);
    } else {
        memmove(name, s, len as usize);
        core::ptr::write(name.offset(len), 0);
    }
    while (*path == b'/') {
        path = path.offset(1);
    }
    return path;
}

// Look up and return the inode for a path name.
// If parent != 0, return the inode for the parent and copy the final
// path element into name, which must have room for DIRSIZ bytes.
// Must be called inside a transaction since it calls iput().
pub unsafe extern "C" fn namex(mut path: *const u8, nameiparent: i32, name: *mut u8) -> *mut Inode {
    let mut ip: *mut Inode;
    if (*path == b'/') {
        check_it("namex (0)");
        ip = iget(ROOTDEV, ROOTINO);
        check_it("namex (0.1)");
    } else {
        ip = idup((*myproc()).cwd);
    }

    loop {
        path = skipelem(path, name);
        if path == core::ptr::null_mut() {
            break;
        }
        cprintf(
            "fs loop  path: \"%s\"  name: \"%s\"\n",
            &[Arg::Strp(path), Arg::Strp(name)],
        );
        check_it("namex (1)");
        ilock(ip);
        if ((*ip).type_ != T_DIR as i16) {
            iunlockput(ip);
            cprintf("fs: c   type: %d\n", &[Arg::Int((*ip).type_ as i32)]);
            return core::ptr::null_mut();
        }
        if (nameiparent != 0 && *path == b'\0') {
            // Stop one level early.
            iunlock(ip);
            return ip;
        }

        let next = dirlookup(ip, name, null_mut());
        if next == core::ptr::null_mut() {
            iunlockput(ip);
            return core::ptr::null_mut();
        }
        iunlockput(ip);
        ip = next;
    }
    if (nameiparent != 0) {
        iput(ip);
        return core::ptr::null_mut();
    }
    return ip;
}

pub unsafe extern "C" fn namei(path: *const u8) -> *mut Inode {
    check_it("namei (1)");
    let mut name = [0u8; DIRSIZ];
    namex(path, 0, name.as_mut_ptr())
}

pub unsafe extern "C" fn nameiparent(path: *const u8, name: *mut u8) -> *mut Inode {
    return namex(path, 1, name);
}
