// Mutual exclusion spin locks.

use core::cell::UnsafeCell;
use core::ops::Deref;
use core::ops::DerefMut;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering;

use super::*;

#[repr(C)]
pub struct Mutex<T> {
    lock: AtomicBool,
    val: UnsafeCell<T>,
}
#[repr(C)]
pub struct Mutex2<T> {
    // val: UnsafeCell<T>,
    val: T
}
impl<T> Mutex2<T> {
    pub unsafe fn hoge(&self) -> &T {
        &self.val
    }
}

// For test
unsafe impl<T> Sync for Mutex<T> {}

unsafe impl<T> Sync for Mutex2<T> {}

pub struct Obj<'a, T: 'a> {
    lock: &'a AtomicBool,
    data: &'a mut T,
}

impl<T> Mutex<T> {
    pub const fn new2(val: T) -> Mutex2<T> {
        Mutex2 {
            val : val,
            // val: UnsafeCell::new(val),
        }
    }

    pub const fn new(val: T) -> Mutex<T> {
        Mutex {
            lock: AtomicBool::new(false),
            val: UnsafeCell::new(val),
        }
    }
    unsafe fn acquire(&self) {
        // TODO: disable interrupt without breaking test (and the first call scenario)
        //// pushcli(); // disable interrupts to avoid deadlock.

        // The xchg is atomic.
        while self.lock.swap(true, Ordering::Acquire) {}
        // TODO: uncomment the following for debugging.
        //         // Record info about lock acquisition for debugging.
        ////         lk->cpu = mycpu();
        ////         getcallerpcs(&lk, lk->pcs);
    }
    pub fn lock(&self) -> Obj<T> {
        unsafe {
            self.acquire();
            Obj {
                lock: &self.lock,
                data: &mut *self.val.get(),
            }
        }
    }
}

impl<'a, T: 'a> Deref for Obj<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &*self.data
    }
}

impl<'a, T: 'a> DerefMut for Obj<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut *self.data
    }
}

impl<'a, T: 'a> Drop for Obj<'a, T> {
    // Release the lock.
    fn drop(&mut self) {
        // TODO: uncomment the following for debug
        //// if(!holding(lk))
        ////     panic("release");
        ////
        //// lk->pcs[0] = 0;
        //// lk->cpu = 0;

        self.lock.store(false, Ordering::Release);
        // TODO: enable interrupt.
        //// popcli();
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::Arc;
    use std::thread;

    #[test]
    fn spin() {
        let spin_mutex = Mutex::new(0);

        // Modify the data
        {
            let mut data = spin_mutex.lock();
            *data = 2;
        }

        // Read the data
        let answer = {
            let data = spin_mutex.lock();
            *data
        };

        assert_eq!(answer, 2);
    }

    #[test]
    fn thread_safe() {
        let m = Arc::new(Mutex::new(0));
        let mut handles = vec![];
        let n = 1000;
        for _ in 0..n {
            let m = Arc::clone(&m);
            let handle = thread::spawn(move || {
                let mut v = m.lock();
                *v += 1;
            });
            handles.push(handle);
        }
        for handle in handles {
            handle.join().unwrap();
        }
        let v = m.lock();
        assert_eq!(*v, n);
    }
}
