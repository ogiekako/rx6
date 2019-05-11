use super::*;

// Interrupt descriptor table (shared by all CPUs).
static mut idt: [Gatedesc; 256] = unsafe { transmute([0u8; size_of::<[Gatedesc; 256]>()]) };

extern "C" {
    // TODO: use macro-generated function pointers
    static vectors: [usize; 256]; // in vectors.S: array of 256 entry pointers
}

pub static mut tickslock: Spinlock = unsafe { transmute([0u8; size_of::<Spinlock>()]) };
pub static mut ticks: usize = 0;

pub unsafe extern "C" fn tvinit() {
    for i in 0..256 {
        idt[i].setgate(false, (SEG_KCODE as u16) << 3, vectors[i], 0);
    }
    idt[T_SYSCALL].setgate(true, (SEG_KCODE as u16) << 3, vectors[T_SYSCALL], DPL_USER);

    initlock(&mut tickslock as *mut Spinlock, "time");
}

pub unsafe extern "C" fn idtinit() {
    if first_user_debug_pa != None {
        if PageDir::from(first_user_pgdir).get_pa_for_fe000000() != first_user_debug_pa {
            piyo();
            cpanic("idtinit(1): broken pgdir");
        }
    }

    lidt(&idt as *const Gatedesc, core::mem::size_of_val(&idt) as i32);
    if first_user_debug_pa != None {
        if PageDir::from(first_user_pgdir).get_pa_for_fe000000() != first_user_debug_pa {
            piyo();
            cpanic("idtinit(2): broken pgdir");
        }
    }
}

extern "C" {
    pub static stack: u8;
}

pub unsafe fn kern_stack_addr() -> usize {
    &stack as *const u8 as usize
}

#[no_mangle]
pub unsafe extern "C" fn trap(tf: *mut Trapframe) {
    check_it("trap (0)");
    check_it("trap (0.4)");
    // kernel stack (scheduler uses)
    let st = &stack as *const u8 as usize as i32;
    // process stack
    static mut esp0: i32 = -1;
    if (*tf).trapno != T_SYSCALL {
      check_it("trap (0.5)");
      esp0 = (*mycpu()).ts.esp0 as i32;
      check_it("trap (0.6)");
    }
    let tf_addr = &tf as *const *mut Trapframe as usize as i32;
    check_it("trap (0.7)");
    cprintf("trap:  &tf = %p  esp0 = %p  stack = %p   tf.eip = %p  tf.trapno = %d\n", &[Arg::Int(tf_addr), Arg::Int(esp0), Arg::Int(st), Arg::Int((*tf).eip as i32), Arg::Int((*tf).trapno as i32)]);
    check_it("trap (0.8)");
    if 0 <= esp0 - tf_addr && esp0 - tf_addr < 4096 {
        // OK.
    } else if 0 <= st + 4096 - tf_addr && st + 4096 - tf_addr < 4096 {
        // OK.
    } else {
        // cprintf("hoge:  &tf = %p  esp0 = %p  stack = %p\n", &[Arg::Int(&tf as *const *mut Trapframe as usize as i32), Arg::Int(esp0 as i32), Arg::Int(st as usize as i32)]);
        cpanic("foo");
    }

    // let ts_addr = &(*mycpu()).ts;
    // let base = (*mycpu()).gdt[SEG_TSS].base();
    // cprintf("trap:  &tf = %p  esp0 = %p  ts_addr = %p   base = %p  stack = %p\n", &[Arg::Int(&tf as *const *mut Trapframe as usize as i32), Arg::Int(esp0 as i32), Arg::Int(ts_addr as *const Taskstate as usize as i32), Arg::Int(base as i32), Arg::Int(st as usize as i32)]);
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
    check_it("trap (1)");
    let t = (*tf).trapno;
    if t == T_IRQ0 + IRQ_TIMER {
        check_it("trap (2)");
        if (cpuid() == 0) {
            check_it("trap (3)");
            acquire(&mut tickslock as *mut Spinlock);
            check_it("trap (4)");

            ticks += 1;
            wakeup(&mut ticks as *mut usize as *mut ());
            check_it("trap (5)");
            release(&mut tickslock as *mut Spinlock);
        }
        check_it("trap (6)");
        lapiceoi();
        check_it("trap (7)");
    } else if t == T_IRQ0 + IRQ_IDE {
        check_it("trap (6)");
        ideintr();
        check_it("trap (6)");
        lapiceoi();
        check_it("trap (6)");
    } else if t == T_IRQ0 + IRQ_IDE + 1 {
        // Bochs generates spurious IDE1 interrupts.
    } else if t == T_IRQ0 + IRQ_KBD {
        check_it("trap (6)");
        kbdintr();
        check_it("trap (6)");
        lapiceoi();
        check_it("trap (6)");
    } else if t == T_IRQ0 + IRQ_COM1 {
        check_it("trap (6)");
        uartintr();
        check_it("trap (6)");
        lapiceoi();
        check_it("trap (6)");
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
        check_it("trap (6)");
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
            cpanic("trap");
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
    check_it("trap (6)");

    // Force process exit if it has been killed and is in user space.
    // (If it is still executing in the kernel, let it keep running
    // until it gets to the regular system call return.)
    if (!myproc().is_null() && (*myproc()).killed && ((*tf).cs & 3) == DPL_USER as u16) {
        exit();
    }
    check_it("trap (6)");
    check_it("trap (6)");

    // Force process to give up CPU on clock tick.
    // If interrupts were on while locks held, would need to check nlock.
    if (!myproc().is_null() && (*myproc()).state == RUNNING && (*tf).trapno == T_IRQ0 + IRQ_TIMER) {
        check_it("trap (6)");
        cprintf("yield_\n", &[]);
        yield_();
        check_it("trap (6)");
    }
    check_it("trap (6)");

    // Check if the process has been killed since we yielded
    if (!myproc().is_null() && (*myproc()).killed && ((*tf).cs & 3) == DPL_USER as u16) {
        exit();
    }
}
