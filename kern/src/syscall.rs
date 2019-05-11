use super::*;
use core;

// System call numbers
pub const SYS_fork: usize = 1;
pub const SYS_exit: usize = 2;
pub const SYS_wait: usize = 3;
pub const SYS_pipe: usize = 4;
pub const SYS_read: usize = 5;
pub const SYS_kill: usize = 6;
pub const SYS_exec: usize = 7;
pub const SYS_fstat: usize = 8;
pub const SYS_chdir: usize = 9;
pub const SYS_dup: usize = 10;
pub const SYS_getpid: usize = 11;
pub const SYS_sbrk: usize = 12;
pub const SYS_sleep: usize = 13;
pub const SYS_uptime: usize = 14;
pub const SYS_open: usize = 15;
pub const SYS_write: usize = 16;
pub const SYS_mknod: usize = 17;
pub const SYS_unlink: usize = 18;
pub const SYS_link: usize = 19;
pub const SYS_mkdir: usize = 20;
pub const SYS_close: usize = 21;

pub const SYS_num: usize = 22;

// User code makes a system call with INT T_SYSCALL.
// System call number in %eax.
// Arguments on the stack, from the user call to the C
// library system call function. The saved user %esp points
// to a saved program counter, and then the first argument.

// Fetch the int at addr from the current process.
pub unsafe extern "C" fn fetchint(addr: usize, ip: *mut i32) -> i32 {
    let curproc = myproc();

    if (addr >= (*curproc).sz || addr + 4 > (*curproc).sz) {
        return -1;
    }
    *ip = *(addr as *const i32);
    return 0;
}

// Fetch the nul-terminated string at addr from the current process.
// Doesn't actually copy the string - just sets *pp to point at it.
// Returns length of string, not including nul.
pub unsafe extern "C" fn fetchstr(addr: usize, pp: *mut *mut u8) -> i32 {
    let mut curproc = myproc();

    if (addr >= (*curproc).sz) {
        return -1;
    }
    *pp = addr as *mut u8;
    let ep = (*curproc).sz as *mut u8;
    let mut s = *pp;
    while s < ep {
        if (*s == 0) {
            return s.offset_from(*pp) as i32;
        }
        s = s.offset(1);
    }
    return -1;
}

// Fetch the nth 32-bit system call argument.
pub unsafe extern "C" fn argint(n: i32, ip: *mut i32) -> i32 {
    return fetchint(((*(*myproc()).tf).esp) + 4 + 4 * n as usize, ip);
}

// Fetch the nth word-sized system call argument as a pointer
// to a block of memory of size bytes.  Check that the pointer
// lies within the process address space.
pub unsafe extern "C" fn argptr(n: i32, pp: *mut *mut u8, size: i32) -> i32 {
    let mut i = 0;
    let curproc = myproc();

    if (argint(n, &mut i as *mut i32) < 0) {
        return -1;
    }
    if (size < 0 || i as usize >= (*curproc).sz || i as usize + size as usize > (*curproc).sz) {
        return -1;
    }
    *pp = i as *mut u8;
    return 0;
}

// Fetch the nth word-sized system call argument as a string pointer.
// Check that the pointer is valid and the string is nul-terminated.
// (There is no shared writable memory, so the string can't change
// between this check and being used by the kernel.)
pub unsafe extern "C" fn argstr(n: i32, pp: *mut *mut u8) -> i32 {
    let mut addr = 0i32;
    if (argint(n, &mut addr as *mut i32) < 0) {
        return -1;
    }
    return fetchstr(addr as usize, pp);
}

// TODO: generate this table with macro.
const syscalls: [Option<unsafe extern "C" fn() -> i32>; SYS_num] = [
    None,
    Some(sys_fork),
    Some(sys_exit),
    Some(sys_wait),
    Some(sys_pipe),
    Some(sys_read),
    Some(sys_kill),
    Some(sys_exec),
    Some(sys_fstat),
    Some(sys_chdir),
    Some(sys_dup),
    Some(sys_getpid),
    Some(sys_sbrk),
    Some(sys_sleep),
    Some(sys_uptime),
    Some(sys_open),
    Some(sys_write),
    Some(sys_mknod),
    Some(sys_unlink),
    Some(sys_link),
    Some(sys_mkdir),
    Some(sys_close),
];

pub unsafe extern "C" fn syscall() {
    let curproc = myproc();

    let num = (*(*curproc).tf).eax as usize;
    if (num > 0 && num < syscalls.len() && syscalls[num].is_some()) {
        cprintf("syscall %d start\n", &[Arg::Int(num as i32)]);
        (*(*curproc).tf).eax = (syscalls[num].unwrap())() as usize;
    } else {
        cprintf(
            "%d %s: unknown sys call %d\n",
            &[
                Arg::Int((*curproc).pid),
                Arg::Str(core::str::from_utf8_unchecked(&(*curproc).name)),
                Arg::Int(num as i32),
            ],
        );
        (*(*curproc).tf).eax = -1i32 as usize;
    }
}
