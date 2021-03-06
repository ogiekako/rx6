use super::*;

use core::sync::atomic;

// Mutual exclusion lock.
pub struct Spinlock {
    locked: usize, // Is the lock held?
    // For debugging:
    name: *const str, // Name of lock.
    cpu: *mut Cpu,    // The cpu holding the lock.
    // pcs: [usize; 10], // The call stack (an array of program counters)
}

impl Spinlock {
    pub const fn uninit() -> Spinlock {
        Spinlock {
            locked: 0,
            name: "" as *const str,
            cpu: core::ptr::null_mut(),
            // pcs: [0; 10],
        }
    }
}

// Mutual exclusion spin locks.

pub unsafe extern "C" fn initlock(lk: *mut Spinlock, name: *const str) {
    (*lk).name = name;
    (*lk).locked = 0;
    (*lk).cpu = null_mut();
}

// Acquire the lock.
// Loops (spins) until the lock is acquired.
// Holding a lock for a long time may cause
// other CPUs to waste time spinning to acquire it.
pub unsafe extern "C" fn acquire(lk: *mut Spinlock) {
    pushcli(); // disable interrupts to avoid deadlock.

    if holding(lk) {
        cpanic("acquire");
    }

    // The xchg is atomic.
    while (xchg(&mut (*lk).locked as *mut usize, 1) != 0) {}

    // Tell the C compiler and the processor to not move loads or stores
    // past this point, to ensure that the critical section's memory
    // references happen after the lock is acquired.
    atomic::fence(atomic::Ordering::SeqCst);

    // Record info about lock acquisition for debugging.
    (*lk).cpu = mycpu();
    // getcallerpcs(lk as *const (), &mut (*lk).pcs);
}

// Release the lock.
pub unsafe extern "C" fn release(lk: *mut Spinlock) {
    if !holding(lk) {
        cpanic("release");
    }

    // (*lk).pcs[0] = 0;
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
pub unsafe extern "C" fn getcallerpcs(v: *const (), pcs: &mut [usize]) {
    // TODO: get caller pcs for Rust programs.
}

// Check whether this cpu is holding the lock.
pub unsafe extern "C" fn holding(lock: *mut Spinlock) -> bool {
    (*lock).locked != 0 && (*lock).cpu == mycpu()
}

// Pushcli/popcli are like cli/sti except that they are matched:
// it takes two popcli to undo two pushcli.  Also, if interrupts
// are off, then pushcli, popcli leaves them off.

pub unsafe extern "C" fn pushcli() {
    let eflags = readeflags();
    cli();
    if ((*mycpu()).ncli == 0) {
        (*mycpu()).intena = (eflags & FL_IF) as i32;
    }
    (*mycpu()).ncli += 1;
}

pub fn popcli() {
    unsafe {
        if (readeflags() & FL_IF != 0) {
            cpanic("popcli - interruptible");
        }
        (*mycpu()).ncli -= 1;
        if ((*mycpu()).ncli < 0) {
            cpanic("popcli");
        }
        if (*mycpu()).ncli == 0 && (*mycpu()).intena != 0 {
            sti();
        }
    }
}
