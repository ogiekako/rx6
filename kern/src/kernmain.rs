/// main.c in xv6
use super::*;

// main in main.c
pub unsafe fn kernmain() {
    kinit1(end(), p2v(P(4 * 1024 * 1024))); // phys page allocator
    kvmalloc(); // kernel page table
    mpinit(); // detect other processors
    lapicinit(); // interrupt controller
    seginit(); // segment descriptors
    picinit(); // another interrupt controller
               //// cprintf("\ncpu%d: starting xv6\n\n", &[Arg::Int(cpunum())]);
    ioapicinit(); // another interrupt controller
    consoleinit(); // console hardware
    uartinit(); // serial port (Outputs "xv6...")
    pinit(); // process table
    tvinit(); // trap vectors
    binit(); // buffer cache
    fileinit(); // file table
    ideinit(); // disk
    assert!(ismp);
    // if(!ismp)
    //   timerinit();   // uniprocessor timer (TODO)
    startothers(); // start other processors
    kinit2(p2v(P(4 * 1024 * 1024)), p2v(PHYSTOP)); // must come after startothers()
    userinit(); // first user process (TODO)
    mpmain(); // finish this processor's setup (TODO)
    cprintf("looping\n", &[]);
    loop {}
}

// Other CPUs jump here from entryother.S.
pub unsafe fn mpenter() {
    switchkvm();
    seginit();
    lapicinit();
    mpmain();
}

// Common CPU setup code.
pub unsafe fn mpmain() {
    cprintf(
        "cpu%d: starting %d\n",
        &[Arg::Int(cpuid() as i32), Arg::Int(cpuid() as i32)],
    );
    idtinit(); // load idt register
    xchg(&mut ((*mycpu()).started) as *mut usize, 1); // tell startothers() we're up
                                                      //// scheduler();     // start running processes
}

extern "C" {
    static mut _binary_entryother_start: u8;
    static mut _binary_entryother_size: u8;
    static mut entrypgdir: u8;
}

// unsafe fn hoge() -> usize {
//     &_binary_entryother_size as *const () as usize
// }

// Start the non-boot (AP) processors.
unsafe fn startothers() {
    let mut stack: *mut i8;

    // Write entry code to unused memory at 0x7000.
    // The linker has placed the image of entryother.S in
    // _binary_entryother_start.
    let code = p2v(P(0x7000)).0 as *mut u8;
    memmove(
        code,
        &_binary_entryother_start as *const u8,
        &_binary_entryother_size as *const u8 as usize,
    );

    for i in 0..ncpu {
        let mut c = &mut cpus[i] as *mut Cpu;
        if c == mycpu() {
            // We've started already.
            continue;
        }
        // Tell entryother.S what stack to use, where to enter, and what
        // pgdir to use. We cannot use kpgdir yet, because the AP processor
        // is running in low  memory, so we use entrypgdir for the APs too.
        let mut stack = kalloc().unwrap();
        core::ptr::write(code.offset(-4) as *mut usize, stack.0 + KSTACKSIZE);
        core::ptr::write(
            code.offset(-8) as *mut usize,
            mpenter as *const unsafe fn() as usize,
        );
        core::ptr::write(
            code.offset(-12) as *mut u32,
            v2p(V(&entrypgdir as *const u8 as usize)).0 as u32,
        );

        lapicstartap((*c).apicid, v2p(V(code as usize)).0 as u32);

        // wait for cpu to finish mpmain()
        while ((*c).started == 0) {}
        c = c.offset(1);
    }
}

// // The boot page table used in entry.S and entryother.S.
// // Page directories (and page tables) must start on page boundaries,
// // hence the __aligned__ attribute.
// // PTE_PS in a page directory entry enables 4Mbyte pages.
//
//// __attribute__((__aligned__(PGSIZE)))
//// pde_t entrypgdir[NPDENTRIES] = {
////   // Map VA's [0, 4MB) to PA's [0, 4MB)
////   [0] = (0) | PTE_P | PTE_W | PTE_PS,
////   // Map VA's [KERNBASE, KERNBASE+4MB) to PA's [0, 4MB)
////   [KERNBASE>>PDXSHIFT] = (0) | PTE_P | PTE_W | PTE_PS,
//// };
