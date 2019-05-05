use super::*;
// sleeplock.h

// Long-term locks for processes
pub struct Sleeplock {
    pub locked: usize, // Is the lock held?
    pub lk: Spinlock,  // spinlock protecting this sleep lock

    // For debugging:
    pub name: *const u8, // Name of lock.
    pub pid: i32,        // Process holding lock
}

// sleeplock.c

// Sleeping locks

pub unsafe extern "C" fn initsleeplock(lk: *mut Sleeplock, name: *const u8) {
    initlock(&mut ((*lk).lk) as *mut Spinlock, "sleep lock");
    (*lk).name = name;
    (*lk).locked = 0;
    (*lk).pid = 0;
}

pub unsafe extern "C" fn acquiresleep(lk: *mut Sleeplock) {
    acquire(&mut ((*lk).lk) as *mut Spinlock);
    while (*lk).locked != 0 {
        sleep(lk as *mut (), &mut (*lk).lk as *mut Spinlock);
    }
    (*lk).locked = 1;
    (*lk).pid = (*myproc()).pid;
    release(&mut (*lk).lk as *mut Spinlock);
}

pub unsafe extern "C" fn releasesleep(lk: *mut Sleeplock) {
    acquire(&mut (*lk).lk as *mut Spinlock);
    (*lk).locked = 0;
    (*lk).pid = 0;
    wakeup(lk as *mut ());
    release(&mut (*lk).lk as *mut Spinlock);
}

pub unsafe extern "C" fn holdingsleep(lk: *mut Sleeplock) -> i32 {
    acquire(&mut (*lk).lk as *mut Spinlock);
    let r = (*lk).locked;
    release(&mut (*lk).lk as *mut Spinlock);
    r as i32
}
