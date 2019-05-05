// Multiprocessor support
// Search memory for MP description structures.
// http://developer.intel.com/design/pentium/datashts/24201606.pdf
// https://pdos.csail.mit.edu/6.828/2016/readings/ia32/MPspec.pdf

use console::cpanic;
use lapic::*;
use memlayout::*;
use mmu::*;
use param::*;
use process::*;
use string;
use string::*;
use x86::*;

use core;

// See MultiProcessor Specification Version 1.[14]
#[repr(C)]
struct Mp {
    // floating pointer
    signature: [u8; 4], // "_MP_"
    physaddr: P,        // phys addr of MP config table
    length: u8,         // 1
    specrev: u8,        // [14]
    checksum: u8,       // all bytes must add up to 0
    typ: u8,            // MP system config type
    imcrp: u8,
    reserved: [u8; 3],
}

#[repr(C)]
struct Mpconf {
    // configuration table header
    signature: [u8; 4],      // "PCMP"
    length: u16,             // total table length
    version: u8,             // [14]
    checksum: u8,            // all bytes must add up to 0
    product: [u8; 20],       // product id
    oemtable: *const usize,  // OEM table pointer
    oemlength: u16,          // OEM table length
    entry: u16,              // entry count
    lapicaddr: *const usize, // address of local APIC
    xlength: u16,            // extended table length
    xchecksum: u8,           // extended table checksum
    reserved: u8,
}

const MPBOOT: u8 = 0x02; // This proc is the bootstrap processor.
#[repr(C)]
struct Mpproc {
    // processor table entry
    typ: u8,            // entry type (0)
    apicid: u8,         // local APIC id
    version: u8,        // local APIC verison
    flags: u8,          // CPU flags
    signature: [u8; 4], // CPU signature
    feature: usize,     // feature flags from CPUID instruction
    reserved: [u8; 8],
}

#[repr(C)]
struct Mpioapic {
    // I/O APIC table entry
    typ: u8,            // entry type (2)
    apicno: u8,         // I/O APIC id
    version: u8,        // I/O APIC version
    flags: u8,          // I/O APIC flags
    addr: *const usize, // I/O APIC address
}

// Table entry types
const MPPROC: u8 = 0x00; // One per processor
const MPBUS: u8 = 0x01; // One per bus
const MPIOAPIC: u8 = 0x02; // One per I/O APIC
const MPIOINTR: u8 = 0x03; // One per bus interrupt source
const MPLINTR: u8 = 0x04; // One per system interrupt source

// TODO: fix
pub static mut cpus: [Cpu; NCPU as usize] = unsafe {
    [
        Cpu::zero(),
        Cpu::zero(),
        Cpu::zero(),
        Cpu::zero(),
        Cpu::zero(),
        Cpu::zero(),
        Cpu::zero(),
        Cpu::zero(),
    ]
};
pub static mut ismp: bool = true;
pub static mut ncpu: usize = 0;
pub static mut ioapicid: u8 = 0;

unsafe extern "C" fn sum(addr: *const u8, len: usize) -> u8 {
    let mut sum = 0u8;
    for i in 0..len {
        sum = sum.wrapping_add(*addr.offset(i as isize));
    }
    sum
}

// Look for an MP structure in the len bytes at addr.
unsafe extern "C" fn mpsearch1(a: P, len: usize) -> Option<*const Mp> {
    let mut addr = p2v(a);
    let e = addr + len;
    while addr < e {
        if string::memcmp(addr.0 as *const u8, "_MP_".as_ptr(), 4) == 0
            && sum(addr.as_ptr(), core::mem::size_of::<Mp>()) == 0
        {
            return Some(addr.as_ptr() as usize as *const Mp);
        }
        addr += core::mem::size_of::<Mp>();
    }
    return None;
}

// Search for the MP Floating Pointer Structure, which according to the
// spec is in one of the following three locations:
// 1) in the first KB of the EBDA;
// 2) in the last KB of system base memory;
// 3) in the BIOS ROM between 0xE0000 and 0xFFFFF.
unsafe extern "C" fn mpsearch() -> Option<*const Mp> {
    let bda: *const u8 = p2v(P(0x400)).as_ptr();

    let mut p = (((*bda.offset(0x0F) as usize) << 8usize) | *bda.offset(0x0E) as usize) << 4usize;
    if p != 0 {
        match mpsearch1(P(p), 1024) {
            None => {}
            Some(mp) => return Some(mp),
        };
    } else {
        p = (((*bda.offset(0x14) as usize) << 8usize) | *bda.offset(0x13) as usize) * 1024usize;
        match mpsearch1(P(p.wrapping_sub(1024)), 1024) {
            None => {}
            Some(mp) => return Some(mp),
        };
    }
    mpsearch1(P(0xF0000), 0x10000)
}

// Search for an MP configuration table.  For now,
// don't accept the default configurations (physaddr == 0).
// Check for correct signature, calculate the checksum and,
// if correct, check the version.
// To do: check extended table checksum.
unsafe extern "C" fn mpconfig(pmp: *mut *const Mp) -> Option<*const Mpconf> {
    let mp = mpsearch()?;
    if (*mp).physaddr == P(0) {
        return None;
    }
    let conf: *const Mpconf = p2v((*mp).physaddr).as_ptr() as usize as *const Mpconf;

    if string::memcmp(conf as usize as *const u8, "PCMP".as_ptr(), 4) != 0 {
        return None;
    }

    if (*conf).version != 1 && (*conf).version != 4 {
        return None;
    }

    if sum(conf as usize as *const u8, (*conf).length as usize) != 0 {
        return None;
    }
    *pmp = mp;
    Some(conf)
}

pub unsafe extern "C" fn mpinit() {
    // TODO: make mpconfig return mp
    let mut mp: *const Mp = 0 as *const Mp;

    let conf = match mpconfig(&mut mp) {
        None => {
            return;
        }
        Some(conf) => conf,
    };

    ismp = true;
    lapic = (*conf).lapicaddr as *mut usize;

    let mut p = conf.offset(1) as usize as *const u8;
    let e = (conf as usize as *const u8).offset((*conf).length as isize);
    while p < e {
        match *p {
            MPPROC => {
                let process = p as usize as *const Mpproc;
                if ncpu < NCPU {
                    cpus[ncpu].apicid = (*process).apicid; // apicid may differ from ncpu
                    ncpu += 1;
                }
                p = p.offset(core::mem::size_of::<Mpproc>() as isize);
                continue;
            }
            MPIOAPIC => {
                let ioapic = p as usize as *const Mpioapic;
                ioapicid = (*ioapic).apicno;
                p = p.offset(core::mem::size_of::<Mpioapic>() as isize);
                continue;
            }
            MPBUS | MPIOINTR | MPLINTR => {
                p = p.offset(8);
                continue;
            }
            _ => {
                ismp = false;
                break;
            }
        };
    }
    if (!ismp) {
        // Didn't like what we found; fall back to no MP.
        ncpu = 1;
        lapic = 0 as *mut usize;
        ioapicid = 0;
        cpanic("unexpected");
        return;
    }

    if (*mp).imcrp != 0 {
        // Bochs doesn't support IMCR, so this doesn't run on Bochs.
        // But it would on real hardware.
        outb(0x22, 0x70); // Select IMCR
        outb(0x23, inb(0x23) | 1); // Mask external interrupts.
        cpanic("unexpected");
    }
    assert_eq!(ncpu, 2);
}
