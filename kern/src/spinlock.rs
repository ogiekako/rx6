use super::*;

// Mutual exclusion lock.
struct spinlock {
    locked: u32, // Is the lock held?
    // For debugging:
    name: *mut str, // Name of lock.
    //// cpu: *mut cpu,  // The cpu holding the lock.
    pcs: [u32; 10], // The call stack (an array of program counters)
}

impl spinlock {
    fn const uninit() -> spinlock {
        spinlock {
            locked: 0,
            name: "",
            pcs: [0; 10]
        }
    }
}

// Mutual exclusion spin locks.

pub unsafe fn initlock(lk: *mut spinlock, name: *mut str) {
    (*lk).name = name;
    (*lk).locked = 0;
    //// lk.cpu = 0;
}

// Acquire the lock.
// Loops (spins) until the lock is acquired.
// Holding a lock for a long time may cause
// other CPUs to waste time spinning to acquire it.
pub unsafe fn acquire(lk: *mut spinlock) {
    pushcli(); // disable interrupts to avoid deadlock.

    //// if holding(lk) {
    ////     panic("acquire");
    //// }

    // The xchg is atomic.
    while (xchg((*lk).locked, 1) != 0) {}

    // Tell the C compiler and the processor to not move loads or stores
    // past this point, to ensure that the critical section's memory
    // references happen after the lock is acquired.
    __sync_synchronize();

    // Record info about lock acquisition for debugging.
    //// lk->cpu = cpu;
    //// getcallerpcs(&lk, lk->pcs);
}
//// void
//// acquire(struct spinlock *lk)
//// {
////   pushcli(); // disable interrupts to avoid deadlock.
////   if(holding(lk))
////     panic("acquire");
////
////   // The xchg is atomic.
////   while(xchg(&lk->locked, 1) != 0)
////     ;
////
////   // Tell the C compiler and the processor to not move loads or stores
////   // past this point, to ensure that the critical section's memory
////   // references happen after the lock is acquired.
////   __sync_synchronize();
////
////   // Record info about lock acquisition for debugging.
////   lk->cpu = cpu;
////   getcallerpcs(&lk, lk->pcs);
//// }
////
//// // Release the lock.
//// void
//// release(struct spinlock *lk)
//// {
////   if(!holding(lk))
////     panic("release");
////
////   lk->pcs[0] = 0;
////   lk->cpu = 0;
////
////   // Tell the C compiler and the processor to not move loads or stores
////   // past this point, to ensure that all the stores in the critical
////   // section are visible to other cores before the lock is released.
////   // Both the C compiler and the hardware may re-order loads and
////   // stores; __sync_synchronize() tells them both not to.
////   __sync_synchronize();
////
////   // Release the lock, equivalent to lk->locked = 0.
////   // This code can't use a C assignment, since it might
////   // not be atomic. A real OS would use C atomics here.
////   asm volatile("movl $0, %0" : "+m" (lk->locked) : );
////
////   popcli();
//// }

// Record the current call stack in pcs[] by following the %ebp chain.
pub unsafe fn getcallerpcs(v: *mut (), pcs: &mut [u32]) {
    let mut ebp = (v as *mut u32).offset(-2);
    let mut i = 0;
    while i < 10 {
        if ebp == 0 || ebp < KERNBASE as *mut u32 || ebp == (0xffffffff as *mut u32) {
            break;
        }
        pcs[i] = ebp[1]; // saved %eip
        ebp = ebp[0] as *mut u32; // saved %ebp
        i += 1;
    }

    while i < 10 {
        pcs[i] = 0;
        i += 1;
    }
}

// Check whether this cpu is holding the lock.
//// pub unsafe fn holding(lock: *mut spinlock) -> bool {
////     return (*lock).locked && (*lock).cpu == cpu;
//// }

// Pushcli/popcli are like cli/sti except that they are matched:
// it takes two popcli to undo two pushcli.  Also, if interrupts
// are off, then pushcli, popcli leaves them off.

pub unsafe fn pushcli() {
    let eflags = readeflags();
    cli();
    if (mycpu().ncli == 0) {
        mycpu().intena = (eflags & FL_IF) as i32;
    }
    mycpu().ncli += 1;
}

pub fn popcli() {
    unsafe {
        if (readeflags() & FL_IF > 0) {
            panic!("popcli - interruptible");
        }
        (mycpu()).ncli -= 1;
        if ((mycpu()).ncli < 0) {
            panic!("popcli");
        }
        if ((mycpu()).ncli == 0 && (*mycpu()).intena > 0) {
            sti();
        }
    }
}
