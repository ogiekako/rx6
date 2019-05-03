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
//// pub unsafe fn fetchint(uint addr, int *ip) -> i32
//// {
////   struct proc *curproc = myproc();
////
////   if(addr >= curproc->sz || addr+4 > curproc->sz)
////     return -1;
////   *ip = *(int*)(addr);
////   return 0;
//// }
//
// // Fetch the nul-terminated string at addr from the current process.
// // Doesn't actually copy the string - just sets *pp to point at it.
// // Returns length of string, not including nul.
//// int
//// fetchstr(uint addr, char **pp)
//// {
////   char *s, *ep;
////   struct proc *curproc = myproc();
////
////   if(addr >= curproc->sz)
////     return -1;
////   *pp = (char*)addr;
////   ep = (char*)curproc->sz;
////   for(s = *pp; s < ep; s++){
////     if(*s == 0)
////       return s - *pp;
////   }
////   return -1;
//// }
//
// // Fetch the nth 32-bit system call argument.
//// int
//// argint(int n, int *ip)
//// {
////   return fetchint((myproc()->tf->esp) + 4 + 4*n, ip);
//// }
//
// // Fetch the nth word-sized system call argument as a pointer
// // to a block of memory of size bytes.  Check that the pointer
// // lies within the process address space.
//// int
//// argptr(int n, char **pp, int size)
//// {
////   int i;
////   struct proc *curproc = myproc();
////
////   if(argint(n, &i) < 0)
////     return -1;
////   if(size < 0 || (uint)i >= curproc->sz || (uint)i+size > curproc->sz)
////     return -1;
////   *pp = (char*)i;
////   return 0;
//// }
//
// // Fetch the nth word-sized system call argument as a string pointer.
// // Check that the pointer is valid and the string is nul-terminated.
// // (There is no shared writable memory, so the string can't change
// // between this check and being used by the kernel.)
//// int
//// argstr(int n, char **pp)
//// {
////   int addr;
////   if(argint(n, &addr) < 0)
////     return -1;
////   return fetchstr(addr, pp);
//// }

pub unsafe fn syscall() {
    // TODO: 1. index with SYS_* enums.
    // 2. make it lazy_static.
    let syscalls: [*const fn() -> i32; SYS_num] = [
        core::ptr::null(),
        sys_fork as (*const fn() -> i32),
        sys_exit as (*const fn() -> i32),
        sys_wait as (*const fn() -> i32),
        sys_pipe as (*const fn() -> i32),
        sys_read as (*const fn() -> i32),
        sys_kill as (*const fn() -> i32),
        sys_exec as (*const fn() -> i32),
        sys_fstat as (*const fn() -> i32),
        sys_chdir as (*const fn() -> i32),
        sys_dup as (*const fn() -> i32),
        sys_getpid as (*const fn() -> i32),
        sys_sbrk as (*const fn() -> i32),
        sys_sleep as (*const fn() -> i32),
        sys_uptime as (*const fn() -> i32),
        sys_open as (*const fn() -> i32),
        sys_write as (*const fn() -> i32),
        sys_mknod as (*const fn() -> i32),
        sys_unlink as (*const fn() -> i32),
        sys_link as (*const fn() -> i32),
        sys_mkdir as (*const fn() -> i32),
        sys_close as (*const fn() -> i32),
    ];

    let curproc = myproc();

    let num = (*(*curproc).tf).eax as usize;
    if (num > 0 && num < syscalls.len() && !syscalls[num].is_null()) {
        (*(*curproc).tf).eax = (*syscalls[num])() as usize;
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
