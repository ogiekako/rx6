/// main.c in xv6
use kalloc;
use lapic;
use linker;
use memlayout;
use memlayout::p2v;
use mmu::P;
use mp;
use vm;

// main in main.c
pub unsafe fn kernmain() {
    kalloc::kinit1(linker::end(), p2v(P(4 * 1024 * 1024))); // phys page allocator
    vm::kvmalloc(); // kernel page table
    mp::mpinit(); // detect other processors
    lapic::lapicinit(); // interrupt controller
                        // seginit();       // segment descriptors
                        // cprintf("\ncpu%d: starting xv6\n\n", cpunum());
                        // picinit();       // another interrupt controller
                        // ioapicinit();    // another interrupt controller
                        // consoleinit();   // console hardware
                        // uartinit();      // serial port
                        // pinit();         // process table
                        // tvinit();        // trap vectors
                        // binit();         // buffer cache
                        // fileinit();      // file table
                        // ideinit();       // disk
                        // if(!ismp)
                        //   timerinit();   // uniprocessor timer
                        // startothers();   // start other processors
                        // kinit2(P2V(4*1024*1024), P2V(PHYSTOP)); // must come after startothers()
                        // userinit();      // first user process
                        // mpmain();        // finish this processor's setup
}
