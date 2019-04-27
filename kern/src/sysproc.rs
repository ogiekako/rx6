use memlayout::*;
use mmu::*;
use process::*;
use x86::*;

pub unsafe fn sys_fork() -> i32 {
    //// return fork();
    return 0;
}

pub unsafe fn sys_exit() -> i32 {
    exit();
    return 0; // not reached
}

pub unsafe fn sys_wait() -> i32 {
    //// return wait();
    return 0;
}

pub unsafe fn sys_kill() -> i32 {
    ////  int pid;
    ////
    ////  if(argint(0, &pid) < 0)
    ////    return -1;
    ////  return kill(pid);
    return 0;
}

pub unsafe fn sys_getpid() -> i32 {
    //// return myproc()->pid;
    return 0;
}

pub unsafe fn sys_sbrk() -> i32 {
    ////  int addr;
    ////  int n;
    ////
    ////  if(argint(0, &n) < 0)
    ////    return -1;
    ////  addr = myproc()->sz;
    ////  if(growproc(n) < 0)
    ////    return -1;
    ////  return addr;
    return 0;
}

pub unsafe fn sys_sleep() -> i32 {
    ////   int n;
    ////   uint ticks0;
    ////
    ////   if(argint(0, &n) < 0)
    ////     return -1;
    ////   acquire(&tickslock);
    ////   ticks0 = ticks;
    ////   while(ticks - ticks0 < n){
    ////     if(myproc()->killed){
    ////       release(&tickslock);
    ////       return -1;
    ////     }
    ////     sleep(&ticks, &tickslock);
    ////   }
    ////   release(&tickslock);
    ////   return 0;
    return 0;
}

// return how many clock tick interrupts have occurred
// since start.
pub unsafe fn sys_uptime() -> i32 {
    ////  uint xticks;
    ////
    ////  acquire(&tickslock);
    ////  xticks = ticks;
    ////  release(&tickslock);
    ////  return xticks;
    return 0;
}
