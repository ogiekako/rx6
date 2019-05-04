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

pub unsafe fn outsl(port: i32, mut addr: *mut (), mut cnt: i32) {
    asm!("cld; rep outsl" :
               "={si}" (addr), "={ecx}" (cnt) :
               "{edx}" (port), "0" (addr), "1" (cnt) :
               "cc": "volatile");
}

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

#[inline]
pub unsafe fn ltr(sel: u16) {
    asm!("ltr $0" : : "r" (sel) ::"volatile");
}

pub unsafe fn readeflags() -> usize {
    let mut eflags = 0usize;
    asm!("pushfl; popl $0" : "=r" (eflags)::::"volatile");
    eflags
}

pub unsafe fn loadgs(v: u16) {
    asm!("movw $0, %gs" : : "r" (v) : : : "volatile");
}

pub unsafe fn cli() {
    asm!("cli":::::"volatile");
}

pub unsafe fn sti() {
    asm!("sti":::::"volatile");
}

#[inline]
pub unsafe fn xchg(addr: *mut usize, newval: usize) -> usize {
    let result: usize;
    // The + in "+m" denotes a read-modify-write operand.
    asm!("lock; xchgl $0, $1":
       "+*m"(addr), "={eax}"(result):
       "1"(newval):
       "cc":
       "volatile");
    result
}

pub unsafe fn rcr2() -> usize {
    let val: usize;
    asm!("movl %cr2,$0" : "=r" (val) ::: "volatile");
    val
}

pub unsafe fn lcr3(val: usize) {
    asm!("mov $0, %cr3"::"r"(val):"memory":"volatile");
}

// Layout of the trap frame built on the stack by the
// hardware and by trapasm.S, and passed to trap().
#[derive(Clone)]
pub struct Trapframe {
    // registers as pushed by pusha
    pub edi: usize,
    pub esi: usize,
    pub ebp: usize,
    pub oesp: usize, // useless & ignored
    pub ebx: usize,
    pub edx: usize,
    pub ecx: usize,
    pub eax: usize,

    // rest of trap frame
    pub gs: u16,
    pub padding1: u16,
    pub fs: u16,
    pub padding2: u16,
    pub es: u16,
    pub padding3: u16,
    pub ds: u16,
    pub padding4: u16,
    pub trapno: usize,

    // below here defined by x86 hardware
    pub err: usize,
    pub eip: usize,
    pub cs: u16,
    pub padding5: u16,
    pub eflags: usize,

    // below here only when crossing rings, such as from user to kernel
    pub esp: usize,
    pub ss: u16,
    pub padding6: u16,
}
