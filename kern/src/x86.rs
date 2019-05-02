// Routines to let C code use special x86 instructions.

use mmu::*;

pub unsafe fn inb(port: u16) -> u8 {
    let data: u8;
    asm!("inb %dx, %al" : "={ax}" (data) : "{dx}"(port) :: "volatile");
    data
}

#[allow(unused_assignments)]
pub unsafe fn insl(port: i32, mut addr: *mut (), mut cnt: i32) {
    asm!("cld; rep insl" :
         "={di}" (addr), "={ecx}" (cnt) :
         "{edx}" (port), "0" (addr), "1" (cnt) :
         "memory", "cc" : "volatile");
}

pub unsafe fn outb(port: u16, data: u8) {
    asm!("outb %al, %dx" :: "{dx}"(port), "{al}"(data) :: "volatile");
}

//// static inline void
//// outw(ushort port, ushort data)
//// {
////   asm volatile("out %0,%1" : : "a" (data), "d" (port));
//// }

//// static inline void
//// outsl(int port, const void *addr, int cnt)
//// {
////   asm volatile("cld; rep outsl" :
////                "=S" (addr), "=c" (cnt) :
////                "d" (port), "0" (addr), "1" (cnt) :
////                "cc");
//// }

#[allow(unused_assignments)]
pub unsafe fn stosb(mut addr: *mut (), data: i32, mut cnt: i32) {
    asm!("cld; rep stosb" :
         "={di}" (addr), "={ecx}" (cnt) :
         "0" (addr), "1" (cnt), "{eax}" (data) :
         "memory", "cc": "volatile");
}

#[allow(unused_assignments)]
pub unsafe fn stosl(mut addr: *mut (), data: i32, mut cnt: i32) {
    asm!("cld; rep stosl" :
         "={di}" (addr), "={ecx}" (cnt) :
         "0" (addr), "1" (cnt), "{eax}" (data) :
         "memory", "cc": "volatile");
}

pub unsafe fn lgdt(p: *const Segdesc, size: u16) {
    let mut pd = [0u16; 3];
    pd[0] = size - 1;
    pd[1] = p as usize as u16;
    pd[2] = (p as usize >> 16) as u16;

    asm!("lgdt ($0)" :: "r" (&pd) : "memory":"volatile");
}

pub unsafe fn lidt(p: *const Gatedesc, size: i32) {
    let mut pd: [u16; 3] = [
        (size - 1) as u16,
        p as usize as u16,
        ((p as usize) >> 16) as u16,
    ];

    asm!("lidt ($0)" : : "r" (&pd) : : "volatile");
}

//// static inline void
//// ltr(ushort sel)
//// {
////   asm volatile("ltr %0" : : "r" (sel));
//// }

pub unsafe fn readeflags() -> u32 {
    let mut eflags = 0u32;
    asm!("pushfl; popl $0" : "=r" (eflags)::::"volatile");
    eflags
}

//// static inline void
//// loadgs(ushort v)
//// {
////   asm volatile("movw %0, %%gs" : : "r" (v));
//// }

pub unsafe fn cli() {
    asm!("cli":::::"volatile");
}

pub unsafe fn sti() {
    asm!("sti":::::"volatile");
}

#[inline]
pub unsafe fn xchg(addr: *mut u32, newval: u32) -> u32 {
    let result: u32;
    // The + in "+m" denotes a read-modify-write operand.
    asm!("lock; xchgl $0, $1":
       "+*m"(addr), "={eax}"(result):
       "1"(newval):
       "cc":
       "volatile");
    result
}

//// static inline uint
//// rcr2(void)
//// {
////   uint val;
////   asm volatile("movl %%cr2,%0" : "=r" (val));
////   return val;
//// }

pub unsafe fn lcr3(val: u32) {
    asm!("mov $0, %cr3"::"r"(val):"memory":"volatile");
}

// Layout of the trap frame built on the stack by the
// hardware and by trapasm.S, and passed to trap().
pub struct Trapframe {
    // registers as pushed by pusha
    pub edi: u32,
    pub esi: u32,
    pub ebp: u32,
    pub oesp: u32, // useless & ignored
    pub ebx: u32,
    pub edx: u32,
    pub ecx: u32,
    pub eax: u32,

    // rest of trap frame
    pub gs: u16,
    pub padding1: u16,
    pub fs: u16,
    pub padding2: u16,
    pub es: u16,
    pub padding3: u16,
    pub ds: u16,
    pub padding4: u16,
    pub trapno: u32,

    // below here defined by x86 hardware
    pub err: u32,
    pub eip: u32,
    pub cs: u16,
    pub padding5: u16,
    pub eflags: u32,

    // below here only when crossing rings, such as from user to kernel
    pub esp: u32,
    pub ss: u16,
    pub padding6: u16,
}
