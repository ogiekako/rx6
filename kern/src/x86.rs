use mmu::*;

// Routines to let C code use special x86 instructions.
pub unsafe fn inb(port: u16) -> u8 {
    let data: u8;
    asm!("inb %dx, %al" : "={ax}" (data) : "{dx}"(port) :: "volatile");
    return data;
}

pub unsafe fn insl(port: i32, mut addr: *mut (), mut cnt: i32) {
    asm!("cld; rep insl" :
         "={di}" (addr), "={ecx}" (cnt) :
         "{edx}" (port), "0" (addr), "1" (cnt) :
         "memory", "cc" : "volatile");
}

pub unsafe fn outb(port: u16, data: u8) {
    asm!("outb %al, %dx" :: "{dx}"(port), "{al}"(data) :: "volatile");
}

// static inline void
// outw(ushort port, ushort data)
// {
//   asm volatile("out %0,%1" : : "a" (data), "d" (port));
// }

// static inline void
// outsl(int port, const void *addr, int cnt)
// {
//   asm volatile("cld; rep outsl" :
//                "=S" (addr), "=c" (cnt) :
//                "d" (port), "0" (addr), "1" (cnt) :
//                "cc");
// }

#[allow(unused_assignments)]
pub unsafe fn stosb(mut addr: *mut (), data: i32, mut cnt: i32) {
    asm!("cld; rep stosb" :
         "={di}" (addr), "={ecx}" (cnt) :
         "0" (addr), "1" (cnt), "{eax}" (data) :
         "memory", "cc": "volatile");
}

// static inline void
// stosl(void *addr, int data, int cnt)
// {
//   asm volatile("cld; rep stosl" :
//                "=D" (addr), "=c" (cnt) :
//                "0" (addr), "1" (cnt), "a" (data) :
//                "memory", "cc");
// }
//
// struct segdesc;

pub unsafe fn lgdt(p: *const Segdesc, size: u16) {
    let mut pd = [0u16; 3];
    pd[0] = size - 1;
    pd[1] = p as usize as u16;
    pd[2] = (p as usize >> 16) as u16;

    asm!("lgdt ($0)" :: "r" (&pd) : "memory":"volatile");
}

// struct gatedesc;
//
// static inline void
// lidt(struct gatedesc *p, int size)
// {
//   volatile ushort pd[3];
//
//   pd[0] = size-1;
//   pd[1] = (uint)p;
//   pd[2] = (uint)p >> 16;
//
//   asm volatile("lidt (%0)" : : "r" (pd));
// }
//
// static inline void
// ltr(ushort sel)
// {
//   asm volatile("ltr %0" : : "r" (sel));
// }

pub unsafe fn readeflags() -> u32 {
    let mut eflags = 0u32;
    asm!("pushfl; popl $0" : "=r" (eflags)::::"volatile");
    eflags
}

// static inline void
// loadgs(ushort v)
// {
//   asm volatile("movw %0, %%gs" : : "r" (v));
// }
//
// static inline void
// cli(void)
// {
//   asm volatile("cli");
// }
//
// static inline void
// sti(void)
// {
//   asm volatile("sti");
// }
//
// static inline uint
// xchg(volatile uint *addr, uint newval)
// {
//   uint result;
//
//   // The + in "+m" denotes a read-modify-write operand.
//   asm volatile("lock; xchgl %0, %1" :
//                "+m" (*addr), "=a" (result) :
//                "1" (newval) :
//                "cc");
//   return result;
// }

// static inline uint
// rcr2(void)
// {
//   uint val;
//   asm volatile("movl %%cr2,%0" : "=r" (val));
//   return val;
// }

pub unsafe fn lcr3(val: u32) {
    asm!("mov $0, %cr3"::"r"(val):"memory":"volatile");
}

// //PAGEBREAK: 36
// // Layout of the trap frame built on the stack by the
// // hardware and by trapasm.S, and passed to trap().
// struct trapframe {
//   // registers as pushed by pusha
//   uint edi;
//   uint esi;
//   uint ebp;
//   uint oesp;      // useless & ignored
//   uint ebx;
//   uint edx;
//   uint ecx;
//   uint eax;
//
//   // rest of trap frame
//   ushort gs;
//   ushort padding1;
//   ushort fs;
//   ushort padding2;
//   ushort es;
//   ushort padding3;
//   ushort ds;
//   ushort padding4;
//   uint trapno;
//
//   // below here defined by x86 hardware
//   uint err;
//   uint eip;
//   ushort cs;
//   ushort padding5;
//   uint eflags;
//
//   // below here only when crossing rings, such as from user to kernel
//   uint esp;
//   ushort ss;
//   ushort padding6;
// };
