// proc.{h,c}.
// renamed to process.rs because proc is a reserved keyword in Rust.

use super::*;
use core;

// Per-CPU state
#[repr(C)]
pub struct Cpu {
    pub apicid: u8,              // Local APIC ID
    pub scheduler: *mut Context, // swtch() here to enter scheduler
    pub ts: Taskstate,           // Used by x86 to find stack for interrupt
    pub gdt: [Segdesc; NSEGS],   // x86 global descriptor table
    // TODO volatile
    pub started: usize, // Has the CPU started?
    pub ncli: i32,      // Depth of pushcli nesting.
    pub intena: i32,    // Were interrupts enabled before pushcli?

    pub process: *mut Proc, // The process running on this cpu or null
}

impl Cpu {
    pub const unsafe fn zero() -> Cpu {
        transmute([0u8; size_of::<Cpu>()])
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
#[repr(C)]
pub struct Context {
    pub edi: usize,
    pub esi: usize,
    pub ebx: usize,
    pub ebp: usize,
    pub eip: usize,
}

#[derive(Debug, PartialEq, Eq)]
#[repr(C)]
pub enum Procstate {
    UNUSED,
    EMBRYO,
    SLEEPING,
    RUNNABLE,
    RUNNING,
    ZOMBIE,
}

pub use Procstate::*;

impl Procstate {
    fn to_str(&self) -> &'static str {
        match self {
            UNUSED => "unused",
            EMBRYO => "embryo",
            SLEEPING => "sleep ",
            RUNNABLE => "runble",
            RUNNING => "run   ",
            ZOMBIE => "zombie",
            _ => "???",
        }
    }
}

use self::Procstate::*;

// Per-process state
#[repr(C)]
pub struct Proc {
    pub sz: usize,                  // Size of process memory (bytes)
    pub pgdir: *mut pde_t,          // Page table
    pub kstack: *mut u8,            // Bottom of kernel stack for this process
    pub kstackguard: *mut u8,       // kernel stack guard which is unmapped.
    pub state: Procstate,           // Process state
    pub pid: i32,                   // Process ID
    pub parent: *mut Proc,          // Parent process
    pub tf: *mut Trapframe,         // Trap frame for current syscall
    pub context: *mut Context,      // swtch() here to run process
    pub chan: *mut (),              // If non-zero, sleeping on chan
    pub killed: bool,               // If non-zero, have been killed
    pub ofile: [*mut File; NOFILE], // Open files
    pub cwd: *mut Inode,            // Current directory
    pub name: [u8; 16],             // Process name (debugging)
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

pub unsafe extern "C" fn pinit() {
    initlock(&mut ptable.lock as *mut Spinlock, "ptable");
}

// Must be called with interrupts disabled
pub unsafe extern "C" fn cpuid() -> usize {
    let i = (mycpu() as *const Cpu).offset_from(cpus.as_ptr());
    assert!(i >= 0);
    i as usize
}

static mut n_mycpu: i32 = 0;
// Must be called with interrupts disabled
pub unsafe fn mycpu() -> *mut Cpu {
    // Would prefer to panic but even printing is chancy here: almost everything,
    // including cprintf and panic, calls mycpu(), often indirectly through
    // acquire and release.
    if (readeflags() & FL_IF > 0) {
        let nn = n_mycpu;
        n_mycpu += 1;
        if (nn == 0) {
            piyo();
            cpanic("mycpu called with interrupts enabled\n");
            // , __builtin_return_address(0));

            // TODO: fix
            // cprintf("mycpu called from %x with interrupts enabled\n", __builtin_return_address(0));
        }
    }

    check_it("mycpu (0.5)");
    // hoge();
    return &mut cpus[lapiccpunum()] as *mut Cpu;
}
unsafe fn hoge() -> u8 {
    let a = [0u8; 100];
    check_it("hoge (0.5)");
    a[0]
}

// Disable interrupts so that we are not rescheduled
// while reading proc from the cpu structure
pub unsafe extern "C" fn myproc() -> *mut Proc {
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
unsafe extern "C" fn allocproc() -> *mut Proc {
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
    (*p).kstackguard = kalloc().unwrap_or(V(0)).0 as *mut u8;
    if (*p).kstackguard.is_null() {
        (*p).state = UNUSED;
        return null_mut();
    }
    (*p).kstack = kalloc().unwrap_or(V(0)).0 as *mut u8;
    if (*p).kstack.is_null() {
        (*p).state = UNUSED;
        kfree(V((*p).kstackguard as usize));
        return null_mut();
    }
    let mut sp = (*p).kstack.add(KSTACKSIZE);

    // Leave room for trap frame.
    sp = sp.sub(core::mem::size_of_val(&(*(*p).tf)));
    (*p).tf = sp as *mut Trapframe;

    // Set up new context to start executing at forkret,
    // which returns to trapret.
    sp = sp.sub(4);
    core::ptr::write(sp as *mut usize, trapret as usize);

    sp = sp.sub(core::mem::size_of_val(&(*(*p).context)));
    if size_of::<Context>() != core::mem::size_of_val(&(*(*p).context)) {
        cpanic("allocproc: hogehoge\n");
    }
    (*p).context = sp as *mut Context;
    memset(
        (*p).context as *mut u8,
        0,
        core::mem::size_of_val(&(*(*p).context)),
    );
    (*(*p).context).eip = forkret as usize;
    cprintf(
        "allocproc:  p: 0x%p, pid: %d, eip: 0x%x, kstack: 0x%p\n",
        &[
            Arg::Int(p as usize as i32),
            Arg::Int((*p).pid as i32),
            Arg::Int((*(*p).context).eip as i32),
            Arg::Int((*p).kstack as i32),
        ],
    );
    p
}

extern "C" {
    static mut _binary_initcode_start: u8;
    static mut _binary_initcode_size: u8;
}

// For debug. FIXME
pub static mut first_user_pgdir: *mut usize = null_mut();
// pa for 0xfe000000 FIXME
pub static mut first_user_debug_pa: Option<(usize, usize)> = None;
// pa for 0xffc00000 FIXME
pub static mut first_user_debug_pa2: Option<(usize, usize)> = None;

// Set up first user process.
pub unsafe extern "C" fn userinit() {
    let p: *mut Proc;

    p = allocproc();

    initproc = p;
    (*p).pgdir = setupkvm().map(|p| p.pd.0).unwrap_or(0) as *mut pde_t;
    if ((*p).pgdir == core::ptr::null_mut()) {
        cpanic("userinit: out of memory?");
    }
    // unmap kstackguard.
    PageDir::from((*p).pgdir).unmap(V((*p).kstackguard as usize));
    
    if !first_user_pgdir.is_null() {
        cpanic("userinit: is_null");
    }
    inituvm(
        (*p).pgdir,
        &mut _binary_initcode_start,
        &_binary_initcode_size as *const u8 as usize,
    );
    first_user_pgdir = (*p).pgdir;
    setup_debug();

    (*p).sz = PGSIZE as usize;
    memset((*p).tf as *mut u8, 0, core::mem::size_of_val(&(*(*p).tf)));
    (*(*p).tf).cs = (SEG_UCODE << 3) as u16 | DPL_USER as u16;
    (*(*p).tf).ds = (SEG_UDATA << 3) as u16 | DPL_USER as u16;
    (*(*p).tf).es = (*(*p).tf).ds;
    (*(*p).tf).ss = (*(*p).tf).ds;
    (*(*p).tf).eflags = FL_IF;
    (*(*p).tf).esp = PGSIZE as usize;
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

// Grow current process's memory by n bytes.
// Return 0 on success, -1 on failure.
pub unsafe extern "C" fn growproc(n: i32) -> i32 {
    let curproc = myproc();

    let mut sz = (*curproc).sz;
    if (n > 0) {
        sz = allocuvm((*curproc).pgdir, sz, sz + n as usize);
        if sz == 0 {
            return -1;
        }
    } else if (n < 0) {
        sz = deallocuvm((*curproc).pgdir, sz, sz + n as usize);
        if sz == 0 {
            return -1;
        }
    }
    (*curproc).sz = sz;
    switchuvm(curproc);
    return 0;
}

// Create a new process copying p as the parent.
// Sets up stack to return as if from system call.
// Caller must set state of returned proc to RUNNABLE.
pub unsafe extern "C" fn fork() -> i32 {
    let curproc = myproc();

    // Allocate process.
    let np = allocproc();
    if np == null_mut() {
        return -1;
    }

    // Copy process state from proc.
    (*np).pgdir = copyuvm((*curproc).pgdir, (*curproc).sz);
    if ((*np).pgdir == null_mut()) {
        kfree(V((*np).kstack as usize));
        kfree(V((*np).kstackguard as usize));
        (*np).kstack = null_mut();
        (*np).kstackguard = null_mut();
        (*np).state = UNUSED;
        return -1;
    }
    (*np).sz = (*curproc).sz;
    (*np).parent = curproc;
    *(*np).tf = (*(*curproc).tf).clone();

    // Clear %eax so that fork returns 0 in the child.
    (*(*np).tf).eax = 0;

    for i in 0..NOFILE {
        if ((*curproc).ofile[i]) != null_mut() {
            (*np).ofile[i] = filedup((*curproc).ofile[i]);
        }
    }
    (*np).cwd = idup((*curproc).cwd);

    safestrcpy(
        (*np).name.as_mut_ptr(),
        (*curproc).name.as_ptr(),
        size_of_val(&(*curproc).name) as i32,
    );

    let pid = (*np).pid;

    acquire(&mut ptable.lock as *mut Spinlock);

    (*np).state = RUNNABLE;

    release(&mut ptable.lock as *mut Spinlock);

    return pid;
}

// Exit the current process.  Does not return.
// An exited process remains in the zombie state
// until its parent calls wait() to find out it exited.
pub unsafe extern "C" fn exit() {
    let curproc = myproc();

    if (curproc == initproc) {
        cpanic("init exiting");
    }

    // Close all open files.
    for fd in 0..NOFILE {
        if (*curproc).ofile[fd] != null_mut() {
            fileclose((*curproc).ofile[fd]);
            (*curproc).ofile[fd] = null_mut();
        }
    }

    begin_op();
    iput((*curproc).cwd);
    end_op();
    (*curproc).cwd = null_mut();

    acquire(&mut ptable.lock as *mut Spinlock);

    // Parent might be sleeping in wait().

    wakeup1((*curproc).parent as *mut ());

    // Pass abandoned children to init.
    for i in 0..NPROC {
        let p = &mut ptable.proc[i];
        if (p.parent == curproc) {
            p.parent = initproc;
            if (p.state == ZOMBIE) {
                wakeup1(initproc as *mut ());
            }
        }
    }

    // Jump into the scheduler, never to return.
    (*curproc).state = ZOMBIE;
    sched();
    cpanic("zombie exit");
}

// Wait for a child process to exit and return its pid.
// Return -1 if this process has no children.
pub unsafe extern "C" fn wait() -> i32 {
    let curproc = myproc();

    acquire(&mut ptable.lock as *mut Spinlock);
    loop {
        // Scan through table looking for exited children.
        let mut havekids = 0;
        for i in 0..NPROC {
            let p = &mut ptable.proc[i];
            if (p.parent != curproc) {
                continue;
            }
            havekids = 1;
            if (p.state == ZOMBIE) {
                // Found one.
                let pid = p.pid;
                kfree(V(p.kstack as usize));
                kfree(V(p.kstackguard as usize));
                p.kstack = null_mut();
                p.kstackguard = null_mut();
                freevm(p.pgdir);
                p.pid = 0;
                p.parent = null_mut();
                p.name[0] = 0;
                p.killed = true;
                p.state = UNUSED;
                release(&mut ptable.lock as *mut Spinlock);
                return pid;
            }
        }

        // No point waiting if we don't have any children.
        if (havekids == 0 || (*curproc).killed) {
            release(&mut ptable.lock as *mut Spinlock);
            return -1;
        }

        // Wait for children to exit.  (See wakeup1 call in proc_exit.)
        sleep(curproc as *mut (), &mut ptable.lock as *mut Spinlock);
    }
}

extern "C" {
    #[no_mangle]
    fn swtch(old: *mut *mut Context, new: *mut Context);
}

// Per-CPU process scheduler.
// Each CPU calls scheduler() after setting itself up.
// Scheduler never returns.  It loops, doing:
//  - choose a process to run
//  - swtch to start running that process
//  - eventually that process transfers control
//      via swtch back to the scheduler.
pub unsafe extern "C" fn scheduler() {
    let c = mycpu();
    (*c).process = null_mut();

    loop {
        check_it("scheduler (0)");

        // Enable interrupts on this processor.
        sti();

        // Loop over process table looking for process to run.
        acquire(&mut ptable.lock as *mut Spinlock);
        check_it("scheduler (1)");
        cprintf("1", &[]);
        for i in 0..NPROC {
            let mut p = &mut ptable.proc[i];
            if (p.state != RUNNABLE) {
                continue;
            }
            check_it("scheduler (1.5)");

            // Switch to chosen process.  It is the process's job
            // to release ptable.lock and then reacquire it
            // before jumping back to us.
            (*c).process = p;
            check_it("scheduler (2)");
            cprintf("2", &[]);

            // switch ltr(
            switchuvm(p as *const Proc);

            cprintf("3", &[]);

            // cprintf("3\n", &[]);

            p.state = RUNNING;

            cprintf("4 `%d", &[Arg::Int((*c).ncli)]);
            swtch(&mut ((*c).scheduler) as *mut *mut Context, (*p).context);
            cprintf("4.5", &[]);
            check_it("scheduler (2)");

            cprintf("5", &[]);
            switchkvm();
            cprintf("6", &[]);

            check_it("scheduler (3)");

            // Process is done running for now.
            // It should have changed its p->state before coming back.
            (*c).process = null_mut();
        }
        check_it("scheduler (4)");
        release(&mut ptable.lock as *mut Spinlock);
        check_it("scheduler (5)");
    }
}

// Enter scheduler.  Must hold only ptable.lock
// and have changed proc->state. Saves and restores
// intena because intena is a property of this
// kernel thread, not this CPU. It should
// be proc->intena and proc->ncli, but that would
// break in the few places where a lock is held but
// there's no process.
pub unsafe extern "C" fn sched() {
    let p = myproc();

    if (!holding(&mut ptable.lock as *mut Spinlock)) {
        cpanic("sched ptable.lock");
    }
    if ((*mycpu()).ncli != 1) {
        cpanic("sched locks");
    }
    if ((*p).state == RUNNING) {
        cpanic("sched running");
    }
    if (readeflags() & FL_IF) != 0 {
        cpanic("sched interruptible");
    }
    let intena = (*mycpu()).intena;
    swtch(&mut (*p).context as *mut *mut Context, (*mycpu()).scheduler);
    (*mycpu()).intena = intena;
}

// Give up the CPU for one scheduling round.
pub unsafe extern "C" fn yield_() {
    acquire(&mut ptable.lock as *mut Spinlock);
    (*myproc()).state = RUNNABLE;
    check_it("yeild (1)");
    sched();
    check_it("yeild (2)");
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
        iinit(ROOTDEV as i32);
        initlog(ROOTDEV as i32);
    }
    cprintf("fr ", &[]);

    // Return to "caller", actually trapret (see allocproc).
}

// Atomically release lock and sleep on chan.
// Reacquires lock when awakened.
pub unsafe extern "C" fn sleep(chan: *mut (), lk: *mut Spinlock) {
    let p = myproc();

    if (p == core::ptr::null_mut()) {
        cpanic("sleep");
    }

    if (lk == core::ptr::null_mut()) {
        cpanic("sleep without lk");
    }

    // Must acquire ptable.lock in order to
    // change p->state and then call sched.
    // Once we hold ptable.lock, we can be
    // guaranteed that we won't miss any wakeup
    // (wakeup runs with ptable.lock locked),
    // so it's okay to release lk.
    if (lk != &mut ptable.lock as *mut Spinlock) {
        acquire(&mut ptable.lock as *mut Spinlock);
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

// Wake up all processes sleeping on chan.
// The ptable lock must be held.
pub unsafe extern "C" fn wakeup1(chan: *mut ()) {
    for i in 0..NPROC {
        let p = &mut ptable.proc[i];
        if (p.state == SLEEPING && p.chan == chan) {
            p.state = RUNNABLE;
        }
    }
}

// Wake up all processes sleeping on chan.
pub unsafe extern "C" fn wakeup(chan: *mut ()) {
    acquire(&mut ptable.lock as *mut Spinlock);
    wakeup1(chan);
    release(&mut ptable.lock as *mut Spinlock);
}

// Kill the process with the given pid.
// Process won't exit until it returns
// to user space (see trap in trap.c).
pub unsafe extern "C" fn kill(pid: i32) -> i32 {
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

// Print a process listing to console.  For debugging.
// Runs when user types ^P on console.
// No lock to avoid wedging a stuck machine further.
pub unsafe extern "C" fn procdump() {
    let mut pc = [0usize; 10];

    for i in 0..NPROC {
        let p = &ptable.proc[i];
        if (p.state == UNUSED) {
            continue;
        }
        let state = p.state.to_str();
        cprintf(
            "%d %s %s",
            &[
                Arg::Int(p.pid),
                Arg::Str(state),
                Arg::Str(core::str::from_utf8(&p.name).unwrap()),
            ],
        );
        if p.state == SLEEPING {
            getcallerpcs(
                ((*p.context).ebp as *const usize).add(2) as *const (),
                &mut pc,
            );
            for i in 0..10 {
                if pc[i] == 0 {
                    break;
                }
                cprintf(" %p", &[Arg::Int(pc[i] as i32)]);
            }
        }
        cprintf("\n", &[]);
    }
}
