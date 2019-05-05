use super::*;

pub const PIPESIZE: usize = 512;

pub struct Pipe {
    lock: Spinlock,
    data: [u8; PIPESIZE],
    nread: usize,   // number of bytes read
    nwrite: usize,  // number of bytes written
    readopen: i32,  // read fd is still open
    writeopen: i32, // write fd is still open
}

pub unsafe extern "C" fn pipealloc(f0: *mut *mut File, f1: *mut *mut File) -> i32 {
    let mut p = null_mut();
    *f0 = null_mut();
    *f1 = null_mut();
    loop {
        *f0 = filealloc();
        if *f0 == null_mut() {
            break;
        }
        *f1 = filealloc();
        if *f1 == null_mut() {
            break;
        }
        let pp = kalloc();
        if (pp.is_none()) {
            break;
        }
        p = pp.unwrap().0 as *mut Pipe;
        (*p).readopen = 1;
        (*p).writeopen = 1;
        (*p).nwrite = 0;
        (*p).nread = 0;
        initlock(&mut (*p).lock as *mut Spinlock, "pipe");
        (*(*f0)).type_ = FD_PIPE;
        (*(*f0)).readable = 1;
        (*(*f0)).writable = 0;
        (*(*f0)).pipe = p;
        (*(*f1)).type_ = FD_PIPE;
        (*(*f1)).readable = 0;
        (*(*f1)).writable = 1;
        (*(*f1)).pipe = p;
        return 0;
    }

    if (p != null_mut()) {
        kfree(V(p as usize));
    }
    if (*f0 != null_mut()) {
        fileclose(*f0);
    }
    if (*f1 != null_mut()) {
        fileclose(*f1);
    }
    return -1;
}

pub unsafe extern "C" fn pipeclose(p: *mut Pipe, writable: i32) {
    acquire(&mut (*p).lock as *mut Spinlock);
    if (writable != 0) {
        (*p).writeopen = 0;
        wakeup(&mut (*p).nread as *mut usize as *mut ());
    } else {
        (*p).readopen = 0;
        wakeup(&mut (*p).nwrite as *mut usize as *mut ());
    }
    if ((*p).readopen == 0 && (*p).writeopen == 0) {
        release(&mut (*p).lock as *mut Spinlock);
        kfree(V(p as usize));
    } else {
        release(&mut (*p).lock as *mut Spinlock);
    }
}

pub unsafe extern "C" fn pipewrite(p: *mut Pipe, addr: *mut u8, n: i32) -> i32 {
    acquire(&mut (*p).lock as *mut Spinlock);
    for i in 0..n {
        while ((*p).nwrite == (*p).nread + PIPESIZE) {
            //DOC: pipewrite-full
            if ((*p).readopen == 0 || (*myproc()).killed) {
                release(&mut (*p).lock as *mut Spinlock);
                return -1;
            }
            wakeup(&mut (*p).nread as *mut usize as *mut ());
            sleep(
                &mut (*p).nwrite as *mut usize as *mut (),
                &mut (*p).lock as *mut Spinlock,
            );
        }
        (*p).data[(*p).nwrite % PIPESIZE] = *(addr.offset(i as isize));
        (*p).nwrite += 1;
    }
    wakeup(&mut (*p).nread as *mut usize as *mut ());
    release(&mut (*p).lock as *mut Spinlock);
    return n;
}

pub unsafe extern "C" fn piperead(p: *mut Pipe, addr: *mut u8, n: i32) -> i32 {
    acquire(&mut (*p).lock as *mut Spinlock);
    while ((*p).nread == (*p).nwrite && (*p).writeopen != 0) {
        if ((*myproc()).killed) {
            release(&mut (*p).lock as *mut Spinlock);
            return -1;
        }
        sleep(
            &mut (*p).nread as *mut usize as *mut (),
            &mut (*p).lock as *mut Spinlock,
        );
    }
    let mut i = 0;
    while i < n {
        if ((*p).nread == (*p).nwrite) {
            break;
        }
        *addr.offset(i as isize) = (*p).data[(*p).nread % PIPESIZE];
        (*p).nread += 1;
        i += 1;
    }
    wakeup(&mut (*p).nwrite as *mut usize as *mut ());
    release(&mut (*p).lock as *mut Spinlock);
    return i;
}
