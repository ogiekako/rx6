// Intel 8253/8254/82C54 Programmable Interval Timer (PIT).
// Only used on uniprocessors;
// SMP machines use the local APIC timer.

use super::*;
use core::convert::TryInto;

const IO_TIMER1: u16 = 0x040; // 8253 Timer #1

// Frequency of all three count-down timers;
// (TIMER_FREQ/freq) is the appropriate count
// to generate a frequency of freq Hz.

const TIMER_FREQ: usize = 1193182;
fn TIMER_DIV(x: usize) -> u16 {
    ((TIMER_FREQ + (x) / 2) / (x)).try_into().unwrap()
}

const TIMER_MODE: u16 = (IO_TIMER1 + 3); // timer mode port
const TIMER_SEL0: u8 = 0x00; // select counter 0
const TIMER_RATEGEN: u8 = 0x04; // mode 2, rate generator
const TIMER_16BIT: u8 = 0x30; // r/w counter 16 bits, LSB first

pub unsafe fn timerinit() {
    // Interrupt 100 times/sec.
    outb(TIMER_MODE, TIMER_SEL0 | TIMER_RATEGEN | TIMER_16BIT);
    outb(IO_TIMER1, (TIMER_DIV(100) % 256) as u8);
    outb(IO_TIMER1, (TIMER_DIV(100) / 256) as u8);
    picenable(IRQ_TIMER as i32);
}
