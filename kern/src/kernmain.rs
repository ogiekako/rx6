use bio;
/// main.c in xv6
use console;
use console::*;
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
    kalloc::kinit2(p2v(P(4 * 1024 * 1024)), p2v(PHYSTOP)); // must come after startothers()
    process::userinit(); // first user process (TODO)
    mpmain(); // finish this processor's setup (TODO)
    console::cprintf("looping\n", &[]);
    loop {}
}

// // Other CPUs jump here from entryother.S.
// static void
// mpenter(void)
// {
//   switchkvm();
//   seginit();
//   lapicinit();
//   mpmain();
// }

// Common CPU setup code.
fn mpmain() {
    // cprintf("cpu%d: starting %d\n", cpuid(), cpuid());
    //  idtinit();       // load idt register
    //  xchg(&(mycpu()->started), 1); // tell startothers() we're up
    //  scheduler();     // start running processes
}

// Start the non-boot (AP) processors.
fn startothers() {
    //  extern uchar _binary_entryother_start[], _binary_entryother_size[];
    //  uchar *code;
    //  struct cpu *c;
    //  char *stack;
    //
    //  // Write entry code to unused memory at 0x7000.
    //  // The linker has placed the image of entryother.S in
    //  // _binary_entryother_start.
    //  code = P2V(0x7000);
    //  memmove(code, _binary_entryother_start, (uint)_binary_entryother_size);
    //
    //  for(c = cpus; c < cpus+ncpu; c++){
    //    if(c == mycpu())  // We've started already.
    //      continue;
    //
    //    // Tell entryother.S what stack to use, where to enter, and what
    //    // pgdir to use. We cannot use kpgdir yet, because the AP processor
    //    // is running in low  memory, so we use entrypgdir for the APs too.
    //    stack = kalloc();
    //    *(void**)(code-4) = stack + KSTACKSIZE;
    //    *(void**)(code-8) = mpenter;
    //    *(int**)(code-12) = (void *) V2P(entrypgdir);
    //
    //    lapicstartap(c->apicid, V2P(code));
    //
    //    // wait for cpu to finish mpmain()
    //    while(c->started == 0)
    //      ;
    //  }
}

// // The boot page table used in entry.S and entryother.S.
// // Page directories (and page tables) must start on page boundaries,
// // hence the __aligned__ attribute.
// // PTE_PS in a page directory entry enables 4Mbyte pages.
//
// __attribute__((__aligned__(PGSIZE)))
// pde_t entrypgdir[NPDENTRIES] = {
//   // Map VA's [0, 4MB) to PA's [0, 4MB)
//   [0] = (0) | PTE_P | PTE_W | PTE_PS,
//   // Map VA's [KERNBASE, KERNBASE+4MB) to PA's [0, 4MB)
//   [KERNBASE>>PDXSHIFT] = (0) | PTE_P | PTE_W | PTE_PS,
// };
//
// //PAGEBREAK!
// // Blank page.
// //PAGEBREAK!
