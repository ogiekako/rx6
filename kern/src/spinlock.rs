use super::*;

use core::sync::atomic;

// Mutual exclusion lock.
pub struct Spinlock {
    locked: usize, // Is the lock held?
    // For debugging:
    name: *const str, // Name of lock.
    cpu: *mut Cpu,    // The cpu holding the lock.
    pcs: [usize; 10], // The call stack (an array of program counters)
}

impl Spinlock {
    pub const fn uninit() -> Spinlock {
        Spinlock {
            locked: 0,
            name: "" as *const str,
            cpu: core::ptr::null_mut(),
            pcs: [0; 10],
        }
    }
}

// Mutual exclusion spin locks.

pub unsafe fn initlock(lk: *mut Spinlock, name: *const str) {
    (*lk).name = name;
    (*lk).locked = 0;
    (*lk).cpu = null_mut();
}

// Acquire the lock.
// Loops (spins) until the lock is acquired.
// Holding a lock for a long time may cause
// other CPUs to waste time spinning to acquire it.
pub unsafe fn acquire(lk: *mut Spinlock) {
    pushcli(); // disable interrupts to avoid deadlock.

    if holding(lk) {
        panic!("acquire");
    }

    // The xchg is atomic.
    while (xchg(&mut (*lk).locked as *mut usize, 1) != 0) {}

    // Tell the C compiler and the processor to not move loads or stores
    // past this point, to ensure that the critical section's memory
    // references happen after the lock is acquired.
    atomic::fence(atomic::Ordering::SeqCst);

    // Record info about lock acquisition for debugging.
    (*lk).cpu = cpu();
    getcallerpcs(lk as *const (), &mut (*lk).pcs);
}

// Release the lock.
pub unsafe fn release(lk: *mut Spinlock) {
    if !holding(lk) {
        panic!("release");
    }

    (*lk).pcs[0] = 0;
    (*lk).cpu = null_mut();

    // Tell the C compiler and the processor to not move loads or stores
    // past this point, to ensure that all the stores in the critical
    // section are visible to other cores before the lock is released.
    // Both the C compiler and the hardware may re-order loads and
    // stores; __sync_synchronize() tells them both not to.
    atomic::fence(atomic::Ordering::SeqCst);

    // Release the lock, equivalent to lk->locked = 0.
    // This code can't use a C assignment, since it might
    // not be atomic. A real OS would use C atomics here.
    asm!("movl $$0, $0" : "=*m" (&(*lk).locked) ::::"volatile");

    popcli();
}

// Record the current call stack in pcs[] by following the %ebp chain.
pub unsafe fn getcallerpcs(v: *const (), pcs: &mut [usize]) {
    let mut ebp = (v as *const usize).offset(-2);
    let mut i = 0;
    while i < 10 {
        if ebp == core::ptr::null_mut()
            || ebp < KERNBASE.0 as *const usize
            || ebp == (0xffffffff as *const usize)
        {
            break;
        }
        pcs[i] = *(ebp.offset(1)) as usize; // saved %eip
        ebp = (*ebp) as *const usize; // saved %ebp
        i += 1;
    }

    while i < 10 {
        pcs[i] = 0;
        i += 1;
    }
}

// Check whether this cpu is holding the lock.
pub unsafe fn holding(lock: *mut Spinlock) -> bool {
    (*lock).locked != 0 && (*lock).cpu == cpu()
}

// Pushcli/popcli are like cli/sti except that they are matched:
// it takes two popcli to undo two pushcli.  Also, if interrupts
// are off, then pushcli, popcli leaves them off.

pub unsafe fn pushcli() {
    let eflags = readeflags();
    cli();
    if ((*mycpu()).ncli == 0) {
        (*mycpu()).intena = (eflags & FL_IF) as i32;
    }
    (*mycpu()).ncli += 1;
}

pub fn popcli() {
    unsafe {
        if (readeflags() & FL_IF > 0) {
            panic!("popcli - interruptible");
        }
        (*mycpu()).ncli -= 1;
        if ((*mycpu()).ncli < 0) {
            panic!("popcli");
        }
        if ((*mycpu()).ncli == 0 && (*mycpu()).intena > 0) {
            sti();
        }
    }
}
