// use types::*;
// use defs::*;
// use param::*;
use memlayout::*;
use mmu::*;
use process::*;
use syscall::*;
use traps::*;
use x86::*;
// use spinlock::*;

// Interrupt descriptor table (shared by all CPUs).
static mut idt: [Gatedesc; 256] = [Gatedesc::zero(); 256];

extern "C" {
    // TODO: use macro-generated function pointers
    static mut vectors: [usize; 256]; // in vectors.S: array of 256 entry pointers
}

//// struct spinlock tickslock;
//// uint ticks;

pub unsafe fn tvinit() {
    for i in 0..256 {
        idt[i].setgate(false, (SEG_KCODE as u16) << 3, vectors[i], 0);
    }
    idt[T_SYSCALL].setgate(true, (SEG_KCODE as u16) << 3, vectors[T_SYSCALL], DPL_USER);

    // TODO: lock
    //// initlock(&tickslock, "time");
}

pub unsafe fn idtinit() {
    lidt(&idt as *const Gatedesc, core::mem::size_of_val(&idt) as i32);
}

#[no_mangle]
pub unsafe extern "C" fn trap(tf: *mut Trapframe) {
    if (*tf).trapno == T_SYSCALL as u32 {
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

    //// switch(tf->trapno){
    //// case T_IRQ0 + IRQ_TIMER:
    ////   if(cpuid() == 0){
    ////     acquire(&tickslock);
    ////     ticks++;
    ////     wakeup(&ticks);
    ////     release(&tickslock);
    ////   }
    ////   lapiceoi();
    ////   break;
    //// case T_IRQ0 + IRQ_IDE:
    ////   ideintr();
    ////   lapiceoi();
    ////   break;
    //// case T_IRQ0 + IRQ_IDE+1:
    ////   // Bochs generates spurious IDE1 interrupts.
    ////   break;
    //// case T_IRQ0 + IRQ_KBD:
    ////   kbdintr();
    ////   lapiceoi();
    ////   break;
    //// case T_IRQ0 + IRQ_COM1:
    ////   uartintr();
    ////   lapiceoi();
    ////   break;
    //// case T_IRQ0 + 7:
    //// case T_IRQ0 + IRQ_SPURIOUS:
    ////   cprintf("cpu%d: spurious interrupt at %x:%x\n",
    ////           cpuid(), tf->cs, tf->eip);
    ////   lapiceoi();
    ////   break;
    ////
    //// //PAGEBREAK: 13
    //// default:
    ////   if(myproc() == 0 || (tf->cs&3) == 0){
    ////     // In kernel, it must be our mistake.
    ////     cprintf("unexpected trap %d from cpu %d eip %x (cr2=0x%x)\n",
    ////             tf->trapno, cpuid(), tf->eip, rcr2());
    ////     panic("trap");
    ////   }
    ////   // In user space, assume process misbehaved.
    ////   cprintf("pid %d %s: trap %d err %d on cpu %d "
    ////           "eip 0x%x addr 0x%x--kill proc\n",
    ////           myproc()->pid, myproc()->name, tf->trapno, tf->err, cpuid(), tf->eip,
    ////           rcr2());
    ////   myproc()->killed = 1;
    //// }
    ////
    //// // Force process exit if it has been killed and is in user space.
    //// // (If it is still executing in the kernel, let it keep running
    //// // until it gets to the regular system call return.)
    //// if(myproc() && myproc()->killed && (tf->cs&3) == DPL_USER)
    ////   exit();
    ////
    //// // Force process to give up CPU on clock tick.
    //// // If interrupts were on while locks held, would need to check nlock.
    //// if(myproc() && myproc()->state == RUNNING && tf->trapno == T_IRQ0+IRQ_TIMER)
    ////   yield();
    ////
    //// // Check if the process has been killed since we yielded
    //// if(myproc() && myproc()->killed && (tf->cs&3) == DPL_USER)
    ////   exit();
}
