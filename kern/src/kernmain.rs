use bio;
/// main.c in xv6
use console;
use file;
use ide;
use ioapic;
use kalloc;
use lapic;
use linker;
use memlayout;
use memlayout::*;
use mmu::P;
use mp;
use mp::*;
use picirq;
use process;
use trap;
use uart;
use vm;

// main in main.c
pub unsafe fn kernmain() {
    kalloc::kinit1(linker::end(), p2v(P(4 * 1024 * 1024))); // phys page allocator
    vm::kvmalloc(); // kernel page table
    mp::mpinit(); // detect other processors
    lapic::lapicinit(); // interrupt controller
    vm::seginit(); // segment descriptors
    picirq::picinit(); // another interrupt controller
    ioapic::ioapicinit(); // another interrupt controller
    console::consoleinit(); // console hardware
    uart::uartinit(); // serial port
    process::pinit(); // process table
    trap::tvinit(); // trap vectors
    bio::binit(); // buffer cache (TODO)
    file::fileinit(); // file table (TODO)
    ide::ideinit(); // disk (TODO)
    assert!(ismp);
    // if(!ismp)
    //   timerinit();   // uniprocessor timer (TODO)
    startothers(); // start other processors
    kalloc::kinit2(p2v(P(4 * 1024 * 1024)), p2v(PHYSTOP)); // must come after startothers() (TODO)
    userinit(); // first user process (TODO)
    mpmain(); // finish this processor's setup (TODO)
    console::cprintf("looping\n", &[]);
    loop {}
}

fn startothers() {}
fn userinit() {}
fn mpmain() {}
