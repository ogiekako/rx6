// proc.{h,c}.
// renamed to process.rs because proc is a reserved keyword in Rust.

use super::*;
use core;

// Per-CPU state
pub struct Cpu {
    pub apicid: u8,              // Local APIC ID
    pub scheduler: *mut Context, // swtch() here to enter scheduler
    pub ts: taskstate,           // Used by x86 to find stack for interrupt
    pub gdt: [Segdesc; NSEGS],   // x86 global descriptor table
    // TODO volatile
    pub started: u32,       // Has the CPU started?
    pub ncli: i32,          // Depth of pushcli nesting.
    pub intena: i32,        // Were interrupts enabled before pushcli?
    pub process: *mut Proc, // The process running on this cpu or null
}

impl Cpu {
    pub const unsafe fn zero() -> Cpu {
        Cpu {
            apicid: 0,
            scheduler: 0usize as *mut Context,
            ts: taskstate {},
            gdt: [
                seg(0, 0, 0, 0),
                seg(0, 0, 0, 0),
                seg(0, 0, 0, 0),
                seg(0, 0, 0, 0),
                seg(0, 0, 0, 0),
                seg(0, 0, 0, 0),
            ],
            started: 0,
            ncli: 0,
            intena: 0,
            process: 0usize as *mut Proc,
        }
    }
}

// Saved registers for kernel context switches.
// Don't need to save all the segment registers (%cs, etc),
// because they are constant across kernel contexts.
// Don't need to save %eax, %ecx, %edx, because the
// x86 convention is that the caller has saved them.
// Contexts are stored at the bottom of the stack they
// describe; the stack pointer is the address of the context.
// The layout of the context matches the layout of the stack in swtch.S
// at the "Switch stacks" comment. Switch doesn't save eip explicitly,
// but it is on the stack and allocproc() manipulates it.
pub struct Context {
    pub edi: u32,
    pub esi: u32,
    pub ebx: u32,
    pub ebp: u32,
    pub eip: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Procstate {
    UNUSED,
    EMBRYO,
    SLEEPING,
    RUNNABLE,
    RUNNING,
    ZOMBIE,
}

use self::Procstate::*;

// Per-process state
pub struct Proc {
    pub sz: u32,               // Size of process memory (bytes)
    pub pgdir: *mut pde_t,     // Page table
    pub kstack: *mut u8,       // Bottom of kernel stack for this process
    pub state: Procstate,      // Process state
    pub pid: i32,              // Process ID
    pub parent: *mut Proc,     // Parent process
    pub tf: *mut Trapframe,    // Trap frame for current syscall
    pub context: *mut Context, // swtch() here to run process
    pub chan: *mut (),         // If non-zero, sleeping on chan
    pub killed: bool,          // If non-zero, have been killed
    // TODO:fix
    //// pub ofile: [File; NOFILE],  // Open files
    pub cwd: *mut Inode, // Current directory
    pub name: [u8; 16],  // Process name (debugging)
}

// Process memory is laid out contiguously, low addresses first:
//   text
//   original data and bss
//   fixed-size stack
//   expandable heap

// proc.c
pub struct Ptable {
    pub lock: Spinlock,
    pub proc: [Proc; NPROC],
}

pub static mut ptable: Ptable =
    unsafe { core::mem::transmute([0u8; core::mem::size_of::<Ptable>()]) };

pub static mut initproc: *mut Proc = unsafe { core::ptr::null_mut() };

pub static mut nextpid: i32 = 1;

extern "C" {
    fn trapret();
}

pub unsafe fn pinit() {
    initlock(&mut ptable.lock as *mut Spinlock, "ptable");
}

// Must be called with interrupts disabled
pub unsafe fn cpuid() -> usize {
    let i = (mycpu() as *const Cpu).offset_from(cpus.as_ptr());
    assert!(i >= 0);
    i as usize
}

static mut n: i32 = 0;
// Must be called with interrupts disabled
pub unsafe fn mycpu() -> *mut Cpu {
    // Would prefer to panic but even printing is chancy here: almost everything,
    // including cprintf and panic, calls mycpu(), often indirectly through
    // acquire and release.
    if (readeflags() & FL_IF > 0) {
        let nn = n;
        n += 1;
        if (nn == 0) {
            // TODO: fix
            // cprintf("mycpu called from %x with interrupts enabled\n", __builtin_return_address(0));
        }
    }

    return &mut cpus[lapiccpunum()] as *mut Cpu;
}

// Disable interrupts so that we are not rescheduled
// while reading proc from the cpu structure
pub unsafe fn myproc() -> *mut Proc {
    pushcli();
    let c = mycpu();
    let p = (*c).process;
    popcli();
    return p;
}

// Look in the process table for an UNUSED proc.
// If found, change state to EMBRYO and initialize
// state required to run in the kernel.
// Otherwise return 0.
pub unsafe fn allocproc() -> *mut Proc {
    acquire(&mut ptable.lock as *mut Spinlock);

    let mut p = core::ptr::null_mut();
    let mut found = false;
    for i in 0..NPROC {
        p = &mut ptable.proc[i] as *mut Proc;

        if (*p).state == UNUSED {
            found = true;
            break;
        }
    }
    if !found {
        release(&mut ptable.lock as *mut Spinlock);
        return core::ptr::null_mut();
    }
    (*p).state = EMBRYO;
    (*p).pid = nextpid;
    nextpid += 1;

    release(&mut ptable.lock as *mut Spinlock);

    // Allocate kernel stack.
    (*p).kstack = kalloc().unwrap_or(V(0)).0 as *mut u8;
    if (*p).kstack == core::ptr::null_mut() {
        (*p).state = UNUSED;
        return core::ptr::null_mut();
    }
    let mut sp = (*p).kstack.offset(KSTACKSIZE as isize);

    // Leave room for trap frame.
    sp = sp.offset(-(core::mem::size_of_val(&((*p).tf)) as isize));
    (*p).tf = sp as *mut Trapframe;

    // Set up new context to start executing at forkret,
    // which returns to trapret.
    sp = sp.offset(-4);
    core::ptr::write(sp as *mut u32, trapret as u32);

    sp = sp.offset(-(core::mem::size_of_val(&((*p).context)) as isize));
    (*p).context = sp as *mut Context;
    memset(
        (*p).context as *mut u8,
        0,
        core::mem::size_of_val(&((*p).context)),
    );
    (*(*p).context).eip = forkret as u32;

    p
}

extern "C" {
    static mut _binary_initcode_start: u8;
    static mut _binary_initcode_size: u8;
}

// Set up first user process.
pub unsafe fn userinit() {
    let p: *mut Proc;

    p = allocproc();

    initproc = p;
    (*p).pgdir = setupkvm().map(|p| p.pd.0).unwrap_or(0) as *mut pde_t;
    if ((*p).pgdir == core::ptr::null_mut()) {
        panic!("userinit: out of memory?");
    }
    inituvm(
        (*p).pgdir,
        &mut _binary_initcode_start,
        &_binary_initcode_size as *const u8 as u32,
    );
    (*p).sz = PGSIZE as u32;
    memset((*p).tf as *mut u8, 0, core::mem::size_of_val(&(*(*p).tf)));
    (*(*p).tf).cs = (SEG_UCODE << 3) as u16 | DPL_USER as u16;
    (*(*p).tf).ds = (SEG_UDATA << 3) as u16 | DPL_USER as u16;
    (*(*p).tf).es = (*(*p).tf).ds;
    (*(*p).tf).ss = (*(*p).tf).ds;
    (*(*p).tf).eflags = FL_IF;
    (*(*p).tf).esp = PGSIZE as u32;
    (*(*p).tf).eip = 0; // beginning of initcode.S

    safestrcpy(
        ((*p).name).as_mut_ptr(),
        "initcode\0".as_ptr(),
        core::mem::size_of_val(&(*p).name) as i32,
    );
    (*p).cwd = namei("/\0".as_ptr());

    // this assignment to p->state lets other cores
    // run this process. the acquire forces the above
    // writes to be visible, and the lock is also needed
    // because the assignment might not be atomic.
    acquire(&mut ptable.lock as *mut Spinlock);

    (*p).state = RUNNABLE;

    release(&mut ptable.lock as *mut Spinlock);
}

// // Grow current process's memory by n bytes.
// // Return 0 on success, -1 on failure.
//// int
//// growproc(int n)
//// {
////   uint sz;
////   struct proc *curproc = myproc();
////
////   sz = curproc->sz;
////   if(n > 0){
////     if((sz = allocuvm(curproc->pgdir, sz, sz + n)) == 0)
////       return -1;
////   } else if(n < 0){
////     if((sz = deallocuvm(curproc->pgdir, sz, sz + n)) == 0)
////       return -1;
////   }
////   curproc->sz = sz;
////   switchuvm(curproc);
////   return 0;
//// }

// // Create a new process copying p as the parent.
// // Sets up stack to return as if from system call.
// // Caller must set state of returned proc to RUNNABLE.
//// int
//// fork(void)
//// {
////   int i, pid;
////   struct proc *np;
////   struct proc *curproc = myproc();
////
////   // Allocate process.
////   if((np = allocproc()) == 0){
////     return -1;
////   }
////
////   // Copy process state from proc.
////   if((np->pgdir = copyuvm(curproc->pgdir, curproc->sz)) == 0){
////     kfree(np->kstack);
////     np->kstack = 0;
////     np->state = UNUSED;
////     return -1;
////   }
////   np->sz = curproc->sz;
////   np->parent = curproc;
////   *np->tf = *curproc->tf;
////
////   // Clear %eax so that fork returns 0 in the child.
////   np->tf->eax = 0;
////
////   for(i = 0; i < NOFILE; i++)
////     if(curproc->ofile[i])
////       np->ofile[i] = filedup(curproc->ofile[i]);
////   np->cwd = idup(curproc->cwd);
////
////   safestrcpy(np->name, curproc->name, sizeof(curproc->name));
////
////   pid = np->pid;
////
////   acquire(&ptable.lock);
////
////   np->state = RUNNABLE;
////
////   release(&ptable.lock);
////
////   return pid;
//// }

// Exit the current process.  Does not return.
// An exited process remains in the zombie state
// until its parent calls wait() to find out it exited.
pub unsafe fn exit() {
    // TODO: fix
    let curproc = myproc();

    ////  if(curproc == initproc) {
    ////    panic!("init exiting");
    ////  }
    ////
    //// // Close all open files.
    //// for fd in 0..NOFILE {
    ////   if (*curproc).ofile[fd] {
    ////     fileclose(curproc->ofile[fd]);
    ////     (*curproc).ofile[fd] = 0;
    ////   }
    //// }
    ////
    //// begin_op();
    //// iput(curproc->cwd);
    //// end_op();
    //// curproc->cwd = 0;
    ////
    //// acquire(&ptable.lock);
    ////
    //// // Parent might be sleeping in wait().
    ////
    //// wakeup1(curproc->parent);
    ////
    //// // Pass abandoned children to init.
    //// for(p = ptable.proc; p < &ptable.proc[NPROC]; p++){
    ////   if(p->parent == curproc){
    ////     p->parent = initproc;
    ////     if(p->state == ZOMBIE)
    ////       wakeup1(initproc);
    ////   }
    //// }

    //// Jump into the scheduler, never to return.
    //// curproc->state = ZOMBIE;
    //// sched();
    //// panic("zombie exit");
}

// // Wait for a child process to exit and return its pid.
// // Return -1 if this process has no children.
//// int
//// wait(void)
//// {
////   struct proc *p;
////   int havekids, pid;
////   struct proc *curproc = myproc();
////
////   acquire(&ptable.lock);
////   for(;;){
////     // Scan through table looking for exited children.
////     havekids = 0;
////     for(p = ptable.proc; p < &ptable.proc[NPROC]; p++){
////       if(p->parent != curproc)
////         continue;
////       havekids = 1;
////       if(p->state == ZOMBIE){
////         // Found one.
////         pid = p->pid;
////         kfree(p->kstack);
////         p->kstack = 0;
////         freevm(p->pgdir);
////         p->pid = 0;
////         p->parent = 0;
////         p->name[0] = 0;
////         p->killed = 0;
////         p->state = UNUSED;
////         release(&ptable.lock);
////         return pid;
////       }
////     }
////
////     // No point waiting if we don't have any children.
////     if(!havekids || curproc->killed){
////       release(&ptable.lock);
////       return -1;
////     }
////
////     // Wait for children to exit.  (See wakeup1 call in proc_exit.)
////     sleep(curproc, &ptable.lock);  //DOC: wait-sleep
////   }
//// }

// //PAGEBREAK: 42
// // Per-CPU process scheduler.
// // Each CPU calls scheduler() after setting itself up.
// // Scheduler never returns.  It loops, doing:
// //  - choose a process to run
// //  - swtch to start running that process
// //  - eventually that process transfers control
// //      via swtch back to the scheduler.
//// void
//// scheduler(void)
//// {
////   struct proc *p;
////   struct cpu *c = mycpu();
////   c->proc = 0;
////
////   for(;;){
////     // Enable interrupts on this processor.
////     sti();
////
////     // Loop over process table looking for process to run.
////     acquire(&ptable.lock);
////     for(p = ptable.proc; p < &ptable.proc[NPROC]; p++){
////       if(p->state != RUNNABLE)
////         continue;
////
////       // Switch to chosen process.  It is the process's job
////       // to release ptable.lock and then reacquire it
////       // before jumping back to us.
////       c->proc = p;
////       switchuvm(p);
////       p->state = RUNNING;
////
////       swtch(&(c->scheduler), p->context);
////       switchkvm();
////
////       // Process is done running for now.
////       // It should have changed its p->state before coming back.
////       c->proc = 0;
////     }
////     release(&ptable.lock);
////
////   }
//// }

// Enter scheduler.  Must hold only ptable.lock
// and have changed proc->state. Saves and restores
// intena because intena is a property of this
// kernel thread, not this CPU. It should
// be proc->intena and proc->ncli, but that would
// break in the few places where a lock is held but
// there's no process.
pub unsafe fn sched() {
    let p = myproc();

    if (!holding(&mut ptable.lock as *mut Spinlock)) {
        panic!("sched ptable.lock");
    }
    if ((*mycpu()).ncli != 1) {
        panic!("sched locks");
    }
    if ((*p).state == RUNNING) {
        panic!("sched running");
    }
    if (readeflags() & FL_IF) != 0 {
        panic!("sched interruptible");
    }
    let intena = (*mycpu()).intena;
    //// swtch(&(*p).context, (*mycpu()).scheduler);
    (*mycpu()).intena = intena;
}

// Give up the CPU for one scheduling round.
pub unsafe fn yield_() {
    acquire(&mut ptable.lock as *mut Spinlock);
    (*myproc()).state = RUNNABLE;
    sched();
    release(&mut ptable.lock as *mut Spinlock);
}

// A fork child's very first scheduling by scheduler()
// will swtch here.  "Return" to user space.

#[no_mangle]
pub unsafe extern "C" fn forkret() {
    static mut first: i32 = 1;
    // Still holding ptable.lock from scheduler.
    release(&mut ptable.lock as *mut Spinlock);

    if (first > 0) {
        // Some initialization functions must be run in the context
        // of a regular process (e.g., they call sleep), and thus cannot
        // be run from main().
        first = 0;
        //// iinit(ROOTDEV);
        //// initlog(ROOTDEV);
    }

    // Return to "caller", actually trapret (see allocproc).
}

// Atomically release lock and sleep on chan.
// Reacquires lock when awakened.
pub unsafe fn sleep(chan: *mut (), lk: *mut Spinlock) {
    let p = myproc();

    if (p == core::ptr::null_mut()) {
        panic!("sleep");
    }

    if (lk == core::ptr::null_mut()) {
        panic!("sleep without lk");
    }

    // Must acquire ptable.lock in order to
    // change p->state and then call sched.
    // Once we hold ptable.lock, we can be
    // guaranteed that we won't miss any wakeup
    // (wakeup runs with ptable.lock locked),
    // so it's okay to release lk.
    if (lk != &mut ptable.lock as *mut Spinlock) {
        //DOC: sleeplock0
        acquire(&mut ptable.lock as *mut Spinlock); //DOC: sleeplock1
        release(lk);
    }

    // Go to sleep.
    (*p).chan = chan;
    (*p).state = SLEEPING;

    sched();

    // Tidy up.
    (*p).chan = core::ptr::null_mut();

    // Reacquire original lock.
    if (lk != &mut ptable.lock as *mut Spinlock) {
        release(&mut ptable.lock as *mut Spinlock);
        acquire(lk);
    }
}

//PAGEBREAK!
// Wake up all processes sleeping on chan.
// The ptable lock must be held.
pub unsafe fn wakeup1(chan: *mut ()) {
    for i in 0..NPROC {
        let p = &mut ptable.proc[i];
        if (p.state == SLEEPING && p.chan == chan) {
            p.state = RUNNABLE;
        }
    }
}

// Wake up all processes sleeping on chan.
pub unsafe fn wakeup(chan: *mut ()) {
    acquire(&mut ptable.lock as *mut Spinlock);
    wakeup1(chan);
    release(&mut ptable.lock as *mut Spinlock);
}

// Kill the process with the given pid.
// Process won't exit until it returns
// to user space (see trap in trap.c).
pub unsafe fn kill(pid: i32) -> i32 {
    acquire(&mut ptable.lock as *mut Spinlock);
    for i in 0..NPROC {
        let p = &mut ptable.proc[i];
        if (p.pid == pid) {
            p.killed = true;
            // Wake process from sleep if necessary.
            if p.state == SLEEPING {
                p.state = RUNNABLE;
            }
            release(&mut ptable.lock as *mut Spinlock);
            return 0;
        }
    }
    release(&mut ptable.lock as *mut Spinlock);
    return -1;
}
//
// //PAGEBREAK: 36
// // Print a process listing to console.  For debugging.
// // Runs when user types ^P on console.
// // No lock to avoid wedging a stuck machine further.
//// void
//// procdump(void)
//// {
////   static char *states[] = {
////   [UNUSED]    "unused",
////   [EMBRYO]    "embryo",
////   [SLEEPING]  "sleep ",
////   [RUNNABLE]  "runble",
////   [RUNNING]   "run   ",
////   [ZOMBIE]    "zombie"
////   };
////   int i;
////   struct proc *p;
////   char *state;
////   uint pc[10];
////
////   for(p = ptable.proc; p < &ptable.proc[NPROC]; p++){
////     if(p->state == UNUSED)
////       continue;
////     if(p->state >= 0 && p->state < NELEM(states) && states[p->state])
////       state = states[p->state];
////     else
////       state = "???";
////     cprintf("%d %s %s", p->pid, state, p->name);
////     if(p->state == SLEEPING){
////       getcallerpcs((uint*)p->context->ebp+2, pc);
////       for(i=0; i<10 && pc[i] != 0; i++)
////         cprintf(" %p", pc[i]);
////     }
////     cprintf("\n");
////   }
//// }
