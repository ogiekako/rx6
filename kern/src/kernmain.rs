use super::*;
// main.c in xv6

// main in main.c
pub unsafe extern "C" fn kernmain() {
    kinit1(end(), p2v(P(4 * 1024 * 1024))); // phys page allocator
    kvmalloc(); // kernel page table
    mpinit(); // detect other processors
    lapicinit(); // interrupt controller
    seginit(); // segment descriptors
    picinit(); // another interrupt controller
    ioapicinit(); // another interrupt controller
    consoleinit(); // console hardware
    uartinit(); // serial port (Outputs "xv6...")

    cprintf("  done: uartinit   \n", &[]);
    kpgdir.dumppgdir();
    pinit(); // process table
    tvinit(); // trap vectors
    binit(); // buffer cache
    fileinit(); // file table
    ideinit(); // disk

    startothers(); // start other processors
    kinit2(p2v(P(4 * 1024 * 1024)), p2v(PHYSTOP)); // must come after startothers()
    userinit(); // first user process
    cprintf("  done: userinit \n", &[]);
    PageDir {
        pd: V(first_user_pgdir as usize),
    }.dumppgdir();
    first_user_debug_pa = PageDir::from(first_user_pgdir).get_pa_for_fe000000();
    enable_check = true;
    cprintf("start mpmain\n", &[]);

    mpmain(); // finish this processor's setup
    cprintf("looping\n", &[]);
    loop {}
}

// Other CPUs jump here from entryother.S.
pub unsafe extern "C" fn mpenter() {
    switchkvm();
    seginit();
    lapicinit();
    mpmain();
}

// Common CPU setup code.
pub unsafe extern "C" fn mpmain() {
    cprintf(
        "cpu%d: starting %d\n",
        &[Arg::Int(cpuid() as i32), Arg::Int(cpuid() as i32)],
    );
    idtinit(); // load idt register
    if first_user_debug_pa != None {
        if PageDir::from(first_user_pgdir).get_pa_for_fe000000() != first_user_debug_pa {
            piyo();
            cpanic("mpmain(1): broken pgdir");
        }
    }
    xchg(&mut ((*mycpu()).started) as *mut usize, 1); // tell startothers() we're up
    if first_user_debug_pa != None {
        if PageDir::from(first_user_pgdir).get_pa_for_fe000000() != first_user_debug_pa {
            piyo();
            cpanic("mpmain(2): broken pgdir");
        }
    }
    scheduler(); // start running processes
}

extern "C" {
    static mut _binary_entryother_start: u8;
    static mut _binary_entryother_size: u8;
    static mut entrypgdir: u8;
}

// Start the non-boot (AP) processors.
unsafe extern "C" fn startothers() {
    let mut stack_: *mut i8;

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
        let mut stack_ = kalloc().unwrap();
        core::ptr::write(code.sub(4) as *mut usize, stack_.0 + KSTACKSIZE);
        core::ptr::write(
            code.sub(8) as *mut usize,
            mpenter as *const unsafe fn() as usize,
        );
        core::ptr::write(
            code.sub(12) as *mut usize,
            v2p(V(&entrypgdir as *const u8 as usize)).0 as usize,
        );
        cprintf(
            "Starting cpu %d  stack: 0x%x.  ",
            &[Arg::Int(i as i32), Arg::Int(stack_.0 as i32)],
        );

        lapicstartap((*c).apicid, v2p(V(code as usize)).0 as usize);

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
// TODO: remove?
// __attribute__((__aligned__(PGSIZE)))
// pde_t entrypgdir[NPDENTRIES] = {
//   // Map VA's [0, 4MB) to PA's [0, 4MB)
//   [0] = (0) | PTE_P | PTE_W | PTE_PS,
//   // Map VA's [KERNBASE, KERNBASE+4MB) to PA's [0, 4MB)
//   [KERNBASE>>PDXSHIFT] = (0) | PTE_P | PTE_W | PTE_PS,
// };
