// This file contains definitions for the
// x86 memory management unit (MMU).

use core::ops::{Add, AddAssign};

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Debug)]
pub struct V(pub usize);

impl V {
    pub unsafe fn pgroundup(self) -> V {
        V(PGROUNDUP(self.0))
    }
    pub unsafe fn pgrounddown(self) -> V {
        V(PGROUNDDOWN(self.0))
    }

    pub const fn as_ptr(self) -> *const u8 {
        self.0 as *const u8
    }

    pub const fn as_mut_ptr(self) -> *mut u8 {
        self.0 as *mut u8
    }
}

impl Add<usize> for V {
    type Output = V;

    fn add(self, other: usize) -> V {
        V(self.0 + other)
    }
}

impl AddAssign<usize> for V {
    fn add_assign(&mut self, other: usize) {
        self.0 += other;
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Debug)]
pub struct P(pub usize);

impl Add<usize> for P {
    type Output = P;

    fn add(self, other: usize) -> P {
        P(self.0 + other)
    }
}

impl AddAssign<usize> for P {
    fn add_assign(&mut self, other: usize) {
        self.0 += other;
    }
}

// Eflags register
pub const FL_CF: usize = 0x00000001; // Carry Flag
pub const FL_PF: usize = 0x00000004; // Parity Flag
pub const FL_AF: usize = 0x00000010; // Auxiliary carry Flag
pub const FL_ZF: usize = 0x00000040; // Zero Flag
pub const FL_SF: usize = 0x00000080; // Sign Flag
pub const FL_TF: usize = 0x00000100; // Trap Flag
pub const FL_IF: usize = 0x00000200; // Interrupt Enable
pub const FL_DF: usize = 0x00000400; // Direction Flag
pub const FL_OF: usize = 0x00000800; // Overflow Flag
pub const FL_IOPL_MASK: usize = 0x00003000; // I/O Privilege Level bitmask
pub const FL_IOPL_0: usize = 0x00000000; //   IOPL == 0
pub const FL_IOPL_1: usize = 0x00001000; //   IOPL == 1
pub const FL_IOPL_2: usize = 0x00002000; //   IOPL == 2
pub const FL_IOPL_3: usize = 0x00003000; //   IOPL == 3
pub const FL_NT: usize = 0x00004000; // Nested Task
pub const FL_RF: usize = 0x00010000; // Resume Flag
pub const FL_VM: usize = 0x00020000; // Virtual 8086 mode
pub const FL_AC: usize = 0x00040000; // Alignment Check
pub const FL_VIF: usize = 0x00080000; // Virtual Interrupt Flag
pub const FL_VIP: usize = 0x00100000; // Virtual Interrupt Pending
pub const FL_ID: usize = 0x00200000; // ID flag

// Control Register flags
pub const CR0_PE: usize = 0x00000001; // Protection Enable
pub const CR0_MP: usize = 0x00000002; // Monitor coProcessor
pub const CR0_EM: usize = 0x00000004; // Emulation
pub const CR0_TS: usize = 0x00000008; // Task Switched
pub const CR0_ET: usize = 0x00000010; // Extension Type
pub const CR0_NE: usize = 0x00000020; // Numeric Errror
pub const CR0_WP: usize = 0x00010000; // Write Protect
pub const CR0_AM: usize = 0x00040000; // Alignment Mask
pub const CR0_NW: usize = 0x20000000; // Not Writethrough
pub const CR0_CD: usize = 0x40000000; // Cache Disable
pub const CR0_PG: usize = 0x80000000; // Paging

pub const CR4_PSE: usize = 0x00000010; // Page size extension

// various segment selectors.
pub const SEG_KCODE: usize = 1; // kernel code
pub const SEG_KDATA: usize = 2; // kernel data+stack
pub const SEG_UCODE: usize = 3; // user code
pub const SEG_UDATA: usize = 4; // user data+stack
pub const SEG_TSS: usize = 5; // this process's task state

// cpu->gdt[NSEGS] holds the above segments.
pub const NSEGS: usize = 6;

// Segment Descriptor
#[repr(C)]
pub struct Segdesc {
    pub lim_15_0: u16,  // Low bits of segment limit
    pub base_15_0: u16, // Low bits of segment base address
    pub base_23_16: u8, // Middle bits of segment base address
    pub typ_s_dpl_p: u8,
    //// uint typ : 4;       // Segment type (see STS_ constants)
    //// uint s : 1;          // 0 = system, 1 = application
    //// uint dpl : 2;        // Descriptor Privilege Level
    //// uint p : 1;          // Present
    pub lim_19_16_avl_rsv1_db_g: u8,
    //// uint lim_19_16 : 4;  // High bits of segment limit
    //// uint avl : 1;        // Unused (available for software use)
    //// uint rsv1 : 1;       // Reserved
    //// uint db : 1;         // 0 = 16-bit segment, 1 = 32-bit segment
    //// uint g : 1;          // Granularity: limit scaled by 4K when set
    pub base_31_24: u8, // High bits of segment base address
}

impl Segdesc {
    const fn new(
        lim_15_0: u16,
        base_15_0: u16,
        base_23_16: u8,
        typ: u8,
        s: u8,
        dpl: u8,
        p: u8,
        lim_19_16: u8,
        avl: u8,
        rsv1: u8,
        db: u8,
        g: u8,
        base_31_24: u8,
    ) -> Segdesc {
        // TODO: fix
        //// assert!(typ < 1<<4);
        //// assert!(s   < 1<<1);
        //// assert!(dpl < 1<<2);
        //// assert!(p < 1<<1);
        //// assert!(lim_19_16 < 1<<4);
        //// assert!(avl < 1<<1);
        //// assert!(rsv1 < 1<<1);
        //// assert!(db < 1<<1);
        //// assert!(g < 1<<1);

        Segdesc {
            lim_15_0,
            base_15_0,
            base_23_16,
            typ_s_dpl_p: typ | s << 4 | dpl << 5 | p << 7,
            lim_19_16_avl_rsv1_db_g: lim_19_16 | avl << 4 | rsv1 << 5 | db << 6 | g << 7,
            base_31_24,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core;
    #[test]
    fn Segdesc() {
        assert_eq!(core::mem::size_of::<Segdesc>(), 8);
    }
}

// Normal segment
pub const fn SEG(typ: u8, base: usize, lim: usize, dpl: u8) -> Segdesc {
    Segdesc::new(
        ((lim >> 12) & 0xffff) as u16,
        (base & 0xffff) as u16,
        ((base >> 16) & 0xff) as u8,
        typ,
        1,
        dpl,
        1,
        (lim >> 28) as u8,
        0,
        0,
        1,
        1,
        (base >> 24) as u8,
    )
}

pub unsafe fn SEG16(typ: u8, base: usize, lim: usize, dpl: u8) -> Segdesc {
    Segdesc::new(
        (lim & 0xffff) as u16,
        (base & 0xffff) as u16,
        ((base >> 16) & 0xff) as u8,
        typ,
        1,
        dpl,
        1,
        (lim >> 16) as u8,
        0,
        0,
        1,
        0,
        (base >> 24) as u8,
    )
}

pub const DPL_USER: u8 = 0x3; // User DPL

// Application segment type bits
pub const STA_X: u8 = 0x8; // Executable segment
pub const STA_E: u8 = 0x4; // Expand down (non-executable segments)
pub const STA_C: u8 = 0x4; // Conforming code segment (executable only)
pub const STA_W: u8 = 0x2; // Writeable (non-executable segments)
pub const STA_R: u8 = 0x2; // Readable (executable segments)
pub const STA_A: u8 = 0x1; // Accessed

// System segment type bits (u4)
pub const STS_T16A: u8 = 0x1; // Available 16-bit TSS
pub const STS_LDT: u8 = 0x2; // Local Descriptor Table
pub const STS_T16B: u8 = 0x3; // Busy 16-bit TSS
pub const STS_CG16: u8 = 0x4; // 16-bit Call Gate
pub const STS_TG: u8 = 0x5; // Task Gate / Coum Transmitions

pub const STS_IG16: u8 = 0x6; // 16-bit Interrupt Gate
pub const STS_TG16: u8 = 0x7; // 16-bit Trap Gate
pub const STS_T32A: u8 = 0x9; // Available 32-bit TSS
pub const STS_T32B: u8 = 0xB; // Busy 32-bit TSS
pub const STS_CG32: u8 = 0xC; // 32-bit Call Gate
pub const STS_IG32: u8 = 0xE; // 32-bit Interrupt Gate
pub const STS_TG32: u8 = 0xF; // 32-bit Trap Gate

// A virtual address 'la' has a three-part structure as follows:
//
// +--------10------+-------10-------+---------12----------+
// | Page Directory |   Page Table   | Offset within Page  |
// |      Index     |      Index     |                     |
// +----------------+----------------+---------------------+
//  \--- PDX(va) --/ \--- PTX(va) --/

impl V {
    // page directory index
    pub unsafe fn pdx(self) -> usize {
        (self.0 >> PDXSHIFT) & 0x3FF
    }
    // page table index
    pub unsafe fn ptx(self) -> usize {
        (self.0 >> PTXSHIFT) & 0x3FF
    }
    // construct virtual address from indexes and offset
    pub unsafe fn pgaddr(d: usize, t: usize, o: usize) -> V {
        V((d << PDXSHIFT) | (t << PTXSHIFT) | o)
    }
}

// Page directory and page table constants.
pub const NPDENTRIES: usize = 1024; // # directory entries per page directory
pub const NPTENTRIES: usize = 1024; // # PTEs per page table
pub const PGSIZE: usize = 4096; // bytes mapped by a page

pub const PGSHIFT: usize = 12; // log2(PGSIZE)
pub const PTXSHIFT: usize = 12; // offset of PTX in a linear address
pub const PDXSHIFT: usize = 22; // offset of PDX in a linear address

pub fn PGROUNDUP(sz: usize) -> usize {
    ((sz) + (PGSIZE - 1)) & (!(PGSIZE - 1))
}

pub fn PGROUNDDOWN(a: usize) -> usize {
    a & (!(PGSIZE - 1))
}

// Page table/directory entry flags.
pub const PTE_P: usize = 0x001; // Present
pub const PTE_W: usize = 0x002; // Writeable
pub const PTE_U: usize = 0x004; // User
pub const PTE_PWT: usize = 0x008; // Write-Through
pub const PTE_PCD: usize = 0x010; // Cache-Disable
pub const PTE_A: usize = 0x020; // Accessed
pub const PTE_D: usize = 0x040; // Dirty
pub const PTE_PS: usize = 0x080; // Page Size
pub const PTE_MBZ: usize = 0x180; // Bits must be zero

pub struct PTE(pub usize);

// Address in page table or page directory entry
impl PTE {
    pub unsafe fn addr(&self) -> P {
        P(self.0 & (!0xFFF))
    }
    pub fn flags(&self) -> usize {
        self.0 & 0xFFF
    }
}

pub type pte_t = usize;

// Task state segment format
#[repr(C)]
pub struct Taskstate {
    pub link: usize, // Old ts selector
    pub esp0: usize, // Stack pointers and segment selectors
    pub ss0: u16,    //   after an increase in privilege level
    pub padding1: u16,
    pub esp1: *mut usize,
    pub ss1: u16,
    pub padding2: u16,
    pub esp2: *mut usize,
    pub ss2: u16,
    pub padding3: u16,
    pub cr3: *mut (),    // Page directory base
    pub eip: *mut usize, // Saved state from last task switch
    pub eflags: usize,
    pub eax: usize, // More saved state (registers)
    pub ecx: usize,
    pub edx: usize,
    pub ebx: usize,
    pub esp: *mut usize,
    pub ebp: *mut usize,
    pub esi: usize,
    pub edi: usize,
    pub es: u16, // Even more saved state (segment selectors)
    pub padding4: u16,
    pub cs: u16,
    pub padding5: u16,
    pub ss: u16,
    pub padding6: u16,
    pub ds: u16,
    pub padding7: u16,
    pub fs: u16,
    pub padding8: u16,
    pub gs: u16,
    pub padding9: u16,
    pub ldt: u16,
    pub padding10: u16,
    pub t: u16,    // Trap on task switch
    pub iomb: u16, // I/O map base address
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Gatedesc {
    off_15_0: u16, // low 16 bits of offset in segment
    cs: u16,       // code segment selector
    args_rsv1: u8,
    //// args : u5;        // # args, 0 for interrupt/trap gates
    //// rsv1 : u3;        // reserved(should be zero I guess)
    type_s_dpl_p: u8,
    //// type : u4;        // type(STS_{TG,IG32,TG32})
    //// s : u1;           // must be 0 (system)
    //// dpl : u2;         // descriptor(meaning new) privilege level
    //// p : u1;           // Present
    off_31_16: u16, // high bits of offset in segment
}

impl Gatedesc {
    pub const fn zero() -> Gatedesc {
        Gatedesc {
            off_15_0: 0,
            cs: 0,
            args_rsv1: 0,
            type_s_dpl_p: 0,
            off_31_16: 0,
        }
    }

    // Set up a normal interrupt/trap gate descriptor.
    // - istrap: 1 for a trap (= exception) gate, 0 for an interrupt gate.
    //   interrupt gate clears FL_IF, trap gate leaves FL_IF alone
    // - sel: Code segment selector for interrupt/trap handler
    // - off: Offset in code segment for interrupt/trap handler
    // - dpl: Descriptor Privilege Level -
    //        the privilege level required for software to invoke
    //        this interrupt/trap gate explicitly using an int instruction.
    pub unsafe fn setgate(&mut self, istrap: bool, sel: u16, off: usize, dpl: u8) {
        assert!(dpl < 1 << 2);
        self.off_15_0 = (off & 0xffff) as u16;
        self.cs = sel;
        let args = 0;
        let rsv1 = 0;
        self.args_rsv1 = 0 | 0 << 5;
        let typ = if istrap { STS_TG32 } else { STS_IG32 };
        let s = 0;
        let dpl = dpl;
        let p = 1;
        self.type_s_dpl_p = typ | s << 4 | dpl << 5 | p << 7;
        self.off_31_16 = (off >> 16) as u16;
    }
}
