use super::*;

pub enum FileType {
    FD_NONE,
    FD_PIPE,
    FD_INODE,
}

pub struct File {
    pub _type: FileType,
    pub _ref: i32, // reference count

    pub readable: u8,
    pub writable: u8,
    //// struct pipe *pipe;
    //// struct inode *ip;
    pub off: usize,
}

// in-memory copy of an inode
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
////
//// // table mapping major device number to
//// // device functions
//// struct devsw {
////   int (*read)(struct inode*, char*, int);
////   int (*write)(struct inode*, char*, int);
//// };
////
//// extern struct devsw devsw[];
////
//// #define CONSOLE 1
////
//// //PAGEBREAK!
//// // Blank page.
//// //
//// // File descriptors
//// //
////
//// #include "types.h"
//// #include "defs.h"
//// #include "param.h"
//// #include "fs.h"
//// #include "spinlock.h"
//// #include "sleeplock.h"
//// #include "file.h"
////
//// struct devsw devsw[NDEV];
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

pub unsafe fn fileinit() {
    initlock(&mut ftable.lock as *mut Spinlock, "ftable");
}

//// // Allocate a file structure.
//// struct file*
//// filealloc(void)
//// {
////   struct file *f;
////
////   acquire(&ftable.lock);
////   for(f = ftable.file; f < ftable.file + NFILE; f++){
////     if(f->ref == 0){
////       f->ref = 1;
////       release(&ftable.lock);
////       return f;
////     }
////   }
////   release(&ftable.lock);
////   return 0;
//// }
////
//// // Increment ref count for file f.
//// struct file*
//// filedup(struct file *f)
//// {
////   acquire(&ftable.lock);
////   if(f->ref < 1)
////     panic("filedup");
////   f->ref++;
////   release(&ftable.lock);
////   return f;
//// }
////
//// // Close file f.  (Decrement ref count, close when reaches 0.)
//// void
//// fileclose(struct file *f)
//// {
////   struct file ff;
////
////   acquire(&ftable.lock);
////   if(f->ref < 1)
////     panic("fileclose");
////   if(--f->ref > 0){
////     release(&ftable.lock);
////     return;
////   }
////   ff = *f;
////   f->ref = 0;
////   f->type = FD_NONE;
////   release(&ftable.lock);
////
////   if(ff.type == FD_PIPE)
////     pipeclose(ff.pipe, ff.writable);
////   else if(ff.type == FD_INODE){
////     begin_op();
////     iput(ff.ip);
////     end_op();
////   }
//// }
////
//// // Get metadata about file f.
//// int
//// filestat(struct file *f, struct stat *st)
//// {
////   if(f->type == FD_INODE){
////     ilock(f->ip);
////     stati(f->ip, st);
////     iunlock(f->ip);
////     return 0;
////   }
////   return -1;
//// }
////
//// // Read from file f.
//// int
//// fileread(struct file *f, char *addr, int n)
//// {
////   int r;
////
////   if(f->readable == 0)
////     return -1;
////   if(f->type == FD_PIPE)
////     return piperead(f->pipe, addr, n);
////   if(f->type == FD_INODE){
////     ilock(f->ip);
////     if((r = readi(f->ip, addr, f->off, n)) > 0)
////       f->off += r;
////     iunlock(f->ip);
////     return r;
////   }
////   panic("fileread");
//// }
////
//// //PAGEBREAK!
//// // Write to file f.
//// int
//// filewrite(struct file *f, char *addr, int n)
//// {
////   int r;
////
////   if(f->writable == 0)
////     return -1;
////   if(f->type == FD_PIPE)
////     return pipewrite(f->pipe, addr, n);
////   if(f->type == FD_INODE){
////     // write a few blocks at a time to avoid exceeding
////     // the maximum log transaction size, including
////     // i-node, indirect block, allocation blocks,
////     // and 2 blocks of slop for non-aligned writes.
////     // this really belongs lower down, since writei()
////     // might be writing a device like the console.
////     int max = ((LOGSIZE-1-1-2) / 2) * 512;
////     int i = 0;
////     while(i < n){
////       int n1 = n - i;
////       if(n1 > max)
////         n1 = max;
////
////       begin_op();
////       ilock(f->ip);
////       if ((r = writei(f->ip, addr + i, f->off, n1)) > 0)
////         f->off += r;
////       iunlock(f->ip);
////       end_op();
////
////       if(r < 0)
////         break;
////       if(r != n1)
////         panic("short filewrite");
////       i += r;
////     }
////     return i == n ? n : -1;
////   }
////   panic("filewrite");
//// }
////
