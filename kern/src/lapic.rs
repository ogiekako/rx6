// The local APIC manages internal (non-I/O) interrupts.
// See Chapter 8 & Appendix C of Intel processor manual volume 3.

use super::*;
use core;

// Local APIC registers, divided by 4 for use as uint[] indices.
const ID: usize = (0x0020 / 4); // ID
const VER: usize = (0x0030 / 4); // Version
const TPR: usize = (0x0080 / 4); // Task Priority
const EOI: usize = (0x00B0 / 4); // EOI
const SVR: usize = (0x00F0 / 4); // Spurious Interrupt Vector
const ENABLE: usize = 0x00000100; // Unit Enable
const ESR: usize = (0x0280 / 4); // Error Status
const ICRLO: usize = (0x0300 / 4); // Interrupt Command
const INIT: usize = 0x00000500; // INIT/RESET
const STARTUP: usize = 0x00000600; // Startup IPI
const DELIVS: usize = 0x00001000; // Delivery status
const ASSERT: usize = 0x00004000; // Assert interrupt (vs deassert)
const DEASSERT: usize = 0x00000000;
const LEVEL: usize = 0x00008000; // Level triggered
const BCAST: usize = 0x00080000; // Send to all APICs, including self.
const BUSY: usize = 0x00001000;
const FIXED: usize = 0x00000000;
const ICRHI: usize = (0x0310 / 4); // Interrupt Command [63:32]
const TIMER: usize = (0x0320 / 4); // Local Vector Table 0 (TIMER)
const X1: usize = 0x0000000B; // divide counts by 1
const PERIODIC: usize = 0x00020000; // Periodic
const PCINT: usize = (0x0340 / 4); // Performance Counter LVT
const LINT0: usize = (0x0350 / 4); // Local Vector Table 1 (LINT0)
const LINT1: usize = (0x0360 / 4); // Local Vector Table 2 (LINT1)
const ERROR: usize = (0x0370 / 4); // Local Vector Table 3 (ERROR)
const MASKED: usize = 0x00010000; // Interrupt masked
const TICR: usize = (0x0380 / 4); // Timer Initial Count
const TCCR: usize = (0x0390 / 4); // Timer Current Count
const TDCR: usize = (0x03E0 / 4); // Timer Divide Configuration

// volatile read/write
pub static mut lapic: *mut usize = null_mut(); // Initialized in mp.c

unsafe extern "C" fn lapicw(index: usize, value: usize) {
    cprintf("lapicw   lapic: %x, index: %d  value: %d\n", &[Arg::Int(lapic as usize as i32), Arg::Int(index as i32), Arg::Int(value as i32)]);
    core::ptr::write_volatile(lapic.offset(index as isize), value);
    lapicr(ID); // wait for write to finish, by reading
}

unsafe extern "C" fn lapicr(index: usize) -> usize {
    if lapic.is_null() {
        cpanic("lapicr");
    }
    core::ptr::read_volatile(lapic.offset(index as isize))
}

pub unsafe extern "C" fn lapicinit() {
    if (lapic.is_null()) {
        cpanic("lapicinit");
        return;
    }

    // Enable local APIC; set spurious interrupt vector.
    lapicw(SVR, ENABLE | (T_IRQ0 + IRQ_SPURIOUS));

    // The timer repeatedly counts down at bus frequency
    // from lapic[TICR] and then issues an interrupt.
    // If xv6 cared more about precise timekeeping,
    // TICR would be calibrated using an external time source.
    lapicw(TDCR, X1);
    lapicw(TIMER, PERIODIC | (T_IRQ0 + IRQ_TIMER));
    lapicw(TICR, 10000000);

    // Disable logical interrupt lines.
    lapicw(LINT0, MASKED);
    lapicw(LINT1, MASKED);

    // Disable performance counter overflow interrupts
    // on machines that provide that interrupt entry.
    if (((lapicr(VER) >> 16) & 0xFF) >= 4) {
        lapicw(PCINT, MASKED);
    }

    // Map error interrupt to IRQ_ERROR.
    lapicw(ERROR, T_IRQ0 + IRQ_ERROR);

    // Clear error status register (requires back-to-back writes).
    lapicw(ESR, 0);
    lapicw(ESR, 0);

    // Ack any outstanding interrupts.
    lapicw(EOI, 0);

    // Send an Init Level De-Assert to synchronise arbitration ID's.
    lapicw(ICRHI, 0);
    lapicw(ICRLO, BCAST | INIT | LEVEL);
    while lapicr(ICRLO) & DELIVS > 0 {}

    // Enable interrupts on the APIC (but not on the processor).
    lapicw(TPR, 0);
}

// Should be called with interrupts disabled: the calling thread shouldn't be
// rescheduled between reading lapic[ID] and checking against cpu array.
pub unsafe extern "C" fn lapiccpunum() -> usize {
    if (lapic as usize == 0) {
        cpanic("cpunum");
        return 0;
    }

    let apicid = (lapicr(ID) >> 24) as u8;
    for i in 0..ncpu {
        if (cpus[i].apicid == apicid) {
            return i;
        }
    }
    cpanic("unknown apicid");
}

// Acknowledge interrupt.
pub unsafe extern "C" fn lapiceoi() {
    if (!lapic.is_null()) {
        lapicw(EOI, 0);
    }
}

// Spin for a given number of microseconds.
// On real hardware would want to tune this dynamically.
pub unsafe extern "C" fn microdelay(us: i32) {}

const CMOS_PORT: u16 = 0x70;
const CMOS_RETURN: u16 = 0x71;

// Start additional processor running entry code at addr.
// See Appendix B of MultiProcessor Specification.
pub unsafe extern "C" fn lapicstartap(apicid: u8, addr: usize) {
    // "The BSP must initialize CMOS shutdown code to 0AH
    // and the warm reset vector (DWORD based at 40:67) to point at
    // the AP startup code prior to the [universal startup algorithm]."
    outb(CMOS_PORT, 0xF); // offset 0xF is shutdown code
    outb(CMOS_PORT + 1, 0x0A);
    let mut wrv = p2v(P(0x40 << 4 | 0x67)).0 as *mut u16; // Warm reset vector
    core::ptr::write(wrv, 0);
    core::ptr::write(wrv.offset(1), (addr >> 4) as u16);

    // "Universal startup algorithm."
    // Send INIT (level-triggered) interrupt to reset other CPU.
    lapicw(ICRHI, ((apicid as usize) << 24) as usize);
    lapicw(ICRLO, INIT | LEVEL | ASSERT);
    microdelay(200);
    lapicw(ICRLO, INIT | LEVEL);
    microdelay(100); // should be 10ms, but too slow in Bochs!

    // Send startup IPI (twice!) to enter code.
    // Regular hardware is supposed to only accept a STARTUP
    // when it is in the halted state due to an INIT.  So the second
    // should be ignored, but it is part of the official Intel algorithm.
    // Bochs complains about the second one.  Too bad for Bochs.
    for i in 0..2 {
        lapicw(ICRHI, ((apicid as usize) << 24) as usize);
        lapicw(ICRLO, STARTUP | (addr >> 12));
        microdelay(200);
    }
}

const CMOS_STATA: usize = 0x0a;
const CMOS_STATB: usize = 0x0b;
const CMOS_UIP: usize = (1 << 7); // RTC update in progress

const SECS: usize = 0x00;
const MINS: usize = 0x02;
const HOURS: usize = 0x04;
const DAY: usize = 0x07;
const MONTH: usize = 0x08;
const YEAR: usize = 0x09;

unsafe extern "C" fn cmos_read(reg: usize) -> usize {
    outb(CMOS_PORT, reg as u8);
    microdelay(200);

    return inb(CMOS_RETURN) as usize;
}

unsafe extern "C" fn fill_rtcdate(r: *mut Rtcdate) {
    (*r).second = cmos_read(SECS);
    (*r).minute = cmos_read(MINS);
    (*r).hour = cmos_read(HOURS);
    (*r).day = cmos_read(DAY);
    (*r).month = cmos_read(MONTH);
    (*r).year = cmos_read(YEAR);
}

// qemu seems to use 24-hour GWT and the values are BCD encoded
pub unsafe extern "C" fn cmostime(r: *mut Rtcdate) {
    let mut t1: Rtcdate = core::mem::zeroed();
    let mut t2: Rtcdate = core::mem::zeroed();
    let sb_ = cmos_read(CMOS_STATB);

    let bcd = (sb_ & (1 << 2)) == 0;

    // make sure CMOS doesn't modify time while we read it
    loop {
        fill_rtcdate(&mut t1 as *mut Rtcdate);
        if (cmos_read(CMOS_STATA) & CMOS_UIP) != 0 {
            continue;
        }
        fill_rtcdate(&mut t2 as *mut Rtcdate);
        if (memcmp(
            &mut t1 as *mut Rtcdate as *mut u8,
            &mut t2 as *mut Rtcdate as *mut u8,
            size_of_val(&t1),
        ) == 0)
        {
            break;
        }
    }

    // convert
    if (bcd) {
        macro_rules! CONV {
            ($x: ident) => {
                (t1.$x = ((t1.$x >> 4) * 10) + (t1.$x & 0xf));
            };
        }
        CONV!(second);
        CONV!(minute);
        CONV!(hour);
        CONV!(day);
        CONV!(month);
        CONV!(year);
    }

    *r = t1;
    (*r).year += 2000;
}
