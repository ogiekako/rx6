use super::*;

// Interrupt descriptor table (shared by all CPUs).
static mut idt: [Gatedesc; 256] = [Gatedesc::zero(); 256];

extern "C" {
    // TODO: use macro-generated function pointers
    static mut vectors: [usize; 256]; // in vectors.S: array of 256 entry pointers
}

static mut tickslock: Spinlock = unsafe { transmute([0u8; size_of::<Spinlock>()]) };
static mut ticks: usize = 0;

pub unsafe fn tvinit() {
    for i in 0..256 {
        idt[i].setgate(false, (SEG_KCODE as u16) << 3, vectors[i], 0);
    }
    idt[T_SYSCALL].setgate(true, (SEG_KCODE as u16) << 3, vectors[T_SYSCALL], DPL_USER);

    initlock(&mut tickslock as *mut Spinlock, "time");
}

pub unsafe fn idtinit() {
    lidt(&idt as *const Gatedesc, core::mem::size_of_val(&idt) as i32);
}

#[no_mangle]
pub unsafe extern "C" fn trap(tf: *mut Trapframe) {
    if (*tf).trapno == T_SYSCALL {
        if ((*myproc()).killed) {
            exit();
        }
        (*myproc()).tf = tf;
        syscall();
        if (*myproc()).killed {
            exit();
        }
        return;
    }
    let t = (*tf).trapno;
    if t == T_IRQ0 + IRQ_TIMER {
        if (cpuid() == 0) {
            acquire(&mut tickslock as *mut Spinlock);
            ticks += 1;
            wakeup(&mut ticks as *mut usize as *mut ());
            release(&mut tickslock as *mut Spinlock);
        }
        lapiceoi();
    } else if t == T_IRQ0 + IRQ_IDE {
        ideintr();
        lapiceoi();
    } else if t == T_IRQ0 + IRQ_IDE + 1 {
        // Bochs generates spurious IDE1 interrupts.
    } else if t == T_IRQ0 + IRQ_KBD {
        //// kbdintr();
        lapiceoi();
    } else if t == T_IRQ0 + IRQ_COM1 {
        uartintr();
        lapiceoi();
    } else if t == T_IRQ0 + 7 || t == T_IRQ0 + IRQ_SPURIOUS {
        cprintf(
            "cpu%d: spurious interrupt at %x:%x\n",
            &[
                Arg::Int(cpuid() as i32),
                Arg::Int((*tf).cs as i32),
                Arg::Int((*tf).eip as i32),
            ],
        );
        lapiceoi();
    } else {
        if (myproc().is_null() || ((*tf).cs & 3) == 0) {
            // In kernel, it must be our mistake.
            cprintf(
                "unexpected trap %d from cpu %d eip %x (cr2=0x%x)\n",
                &[
                    Arg::Int((*tf).trapno as i32),
                    Arg::Int(cpuid() as i32),
                    Arg::Int((*tf).eip as i32),
                    Arg::Int(rcr2() as i32),
                ],
            );
            panic!("trap");
        }
        // In user space, assume process misbehaved.
        cprintf(
            "pid %d %s: trap %d err %d on cpu %d eip 0x%x addr 0x%x--kill proc\n",
            &[
                Arg::Int((*myproc()).pid),
                Arg::Str(core::str::from_utf8(&(*myproc()).name).unwrap()),
                Arg::Int((*tf).trapno as i32),
                Arg::Int((*tf).err as i32),
                Arg::Int(cpuid() as i32),
                Arg::Int((*tf).eip as i32),
                Arg::Int(rcr2() as i32),
            ],
        );
        (*myproc()).killed = true;
    }

    // Force process exit if it has been killed and is in user space.
    // (If it is still executing in the kernel, let it keep running
    // until it gets to the regular system call return.)
    if (!myproc().is_null() && (*myproc()).killed && ((*tf).cs & 3) == DPL_USER as u16) {
        exit();
    }

    // Force process to give up CPU on clock tick.
    // If interrupts were on while locks held, would need to check nlock.
    if (!myproc().is_null() && (*myproc()).state == RUNNING && (*tf).trapno == T_IRQ0 + IRQ_TIMER) {
        yield_();
    }

    // Check if the process has been killed since we yielded
    if (!myproc().is_null() && (*myproc()).killed && ((*tf).cs & 3) == DPL_USER as u16) {
        exit();
    }
}
