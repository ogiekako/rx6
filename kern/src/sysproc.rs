use super::*;

pub unsafe extern "C" fn sys_fork() -> i32 {
    return fork();
}

pub unsafe extern "C" fn sys_exit() -> i32 {
    exit();
    return 0; // not reached
}

pub unsafe extern "C" fn sys_wait() -> i32 {
    return wait();
    return 0;
}

pub unsafe extern "C" fn sys_kill() -> i32 {
    let mut pid = 0i32;

    if (argint(0, &mut pid as *mut i32) < 0) {
        return -1;
    }
    return kill(pid);
}

pub unsafe extern "C" fn sys_getpid() -> i32 {
    return (*myproc()).pid;
}

pub unsafe extern "C" fn sys_sbrk() -> i32 {
    let mut n = 0i32;

    if (argint(0, &mut n as *mut i32) < 0) {
        return -1;
    }
    let addr = (*myproc()).sz;
    if (growproc(n) < 0) {
        return -1;
    }
    return addr as i32;
}

pub unsafe extern "C" fn sys_sleep() -> i32 {
    let mut n = 0i32;
    if (argint(0, &mut n as *mut i32) < 0) {
        return -1;
    }
    acquire(&mut tickslock as *mut Spinlock);
    let mut ticks0 = ticks;
    while (ticks - ticks0 < n as usize) {
        if ((*myproc()).killed) {
            release(&mut tickslock as *mut Spinlock);
            return -1;
        }
        sleep(
            &mut ticks as *mut usize as *mut (),
            &mut tickslock as *mut Spinlock,
        );
    }
    release(&mut tickslock as *mut Spinlock);
    return 0;
}

// return how many clock tick interrupts have occurred
// since start.
pub unsafe extern "C" fn sys_uptime() -> i32 {
    acquire(&mut tickslock as *mut Spinlock);
    let xticks = ticks;
    release(&mut tickslock as *mut Spinlock);
    return xticks as i32;
}
