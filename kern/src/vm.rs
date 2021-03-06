use core;

use super::*;

// Set up CPU's kernel segment descriptors.
// Run once on entry on each CPU.
pub unsafe extern "C" fn seginit() {
    // Map "logical" addresses to virtual addresses using identity map.
    // Cannot share a CODE descriptor for both kernel and user
    // because it would have to have DPL_USR, but the CPU forbids
    // an interrupt from CPL=0 to DPL=3.
    let mut c = &mut cpus[cpuid()];
    c.gdt[SEG_KCODE] = SEG(STA_X | STA_R, 0, 0xffffffff, 0);
    c.gdt[SEG_KDATA] = SEG(STA_W, 0, 0xffffffff, 0);
    c.gdt[SEG_UCODE] = SEG(STA_X | STA_R, 0, 0xffffffff, DPL_USER);
    c.gdt[SEG_UDATA] = SEG(STA_W, 0, 0xffffffff, DPL_USER);

    lgdt(c.gdt.as_ptr(), core::mem::size_of_val(&c.gdt) as u16);
}

// for use in scheduler()
pub static mut kpgdir: PageDir = PageDir { pd: V(0) };

pub struct PageDir {
    pub pd: V, // [pd, pd+PGSIZE)
}

pub static mut enable_check:bool = false;
pub unsafe fn check_it(msg: &'static str) {
    if !enable_check {
        return
    }
    let addr = &msg as *const &'static str as usize as i32;
    let st = &stack as *const u8 as usize as i32;
    // process stack
    static mut esp0: i32 = -1;
    if (readeflags() & FL_IF == 0) {
        enable_check = false;
        esp0 = (*mycpu()).ts.esp0 as i32;
        enable_check = true;
    }
    if 0 <= esp0 - addr && esp0 - addr < 4096 {
        // OK.
    } else if 0 <= st + 4096 - addr && st + 4096 - addr < 4096 {
        // OK.
    } else {
        piyo();
        cpanic("foo");
    }

    let pd = PageDir::from(first_user_pgdir);
    let va = 0xff3ffc00;
    if first_user_debug_pa2 != pd.get_pa(va) {
        piyo();
        let pde = pd.get_pde(va);
        let pte_addr = (pde & !0xFFF) + KERNBASE.0;
        piyo2(pte_addr);
        // cprintf("&pte: 0x%p", &[Arg::Int(pte_addr as i32)]);
        
        cpanic(msg);
    }
    if first_user_debug_pa != pd.get_pa(0xfe000000) {
        piyo();
        cpanic(msg);
    }
}
pub unsafe fn setup_debug() {
    let pd = PageDir::from(first_user_pgdir);
    first_user_debug_pa = pd.get_pa(0xfe000000);
    first_user_debug_pa2 = pd.get_pa(0xff3ffc00);
}

impl PageDir {
    pub fn from(ptr: *mut pde_t) -> PageDir {
        PageDir {
            pd: V(ptr as usize),
        }
    }
    pub unsafe fn get_pa_for_fe000000(&self) -> Option<(usize, usize)> {
        self.get_pa(0xfe000000)
    }
    #[inline(always)]
    pub unsafe fn get_pde(&self, va: usize) -> usize {
        let page_directory: &'static [usize; 1024] = transmute(self.pd.0);
        let dir = va >> 22;
        page_directory[dir] // PPN(20) | Flags(12)
    }
    // (pa, uwp)
    pub unsafe fn get_pa(&self, va: usize) -> Option<(usize, usize)> {
        let pde = self.get_pde(va); // PPN(20) | Flags(12)
        if pde & PTE_P == 0 {
            // No mapping
            return None;
        }
        let page_table: &'static [usize; 1024] = transmute((pde & !0xFFF) + KERNBASE.0);
        let table = (va >> 12) & 1023;
        let pte = page_table[table];
        if pte & PTE_P == 0 {
            return None;
        }
        let pa = pte >> 12 << 12;
        let uwp = pde & pte & 7;
        Some((pa, uwp))
    }

    unsafe fn dump(va: usize, pa: usize, sz: usize, uwp: usize) {
        let mut s = [b'-'; 3];
        if uwp & 4 > 0 {
            s[0] = b'u';
        }
        if uwp & 2 > 0 {
            s[2] = b'w';
        }
        if uwp & 1 > 0 {
            s[1] = b'r';
        }

        cprintf(
            "[%p,%p) ->  [%p,%p) %s\n",
            &[
                Arg::Int(va as i32),
                Arg::Int((va.wrapping_add(sz)) as i32),
                Arg::Int(pa as i32),
                Arg::Int((pa.wrapping_add(sz)) as i32),
                Arg::Strp(s.as_ptr()),
            ],
        );
    }

    pub unsafe fn dumppgdir(&self) {
        let page_directory: &'static [usize; 1024] = transmute(self.pd.0);
        let mut from_va = 0;
        let mut state = None; // (va, pa, uwp)

        cprintf("===  page_dir in 0x%x ===\n", &[Arg::Int(self.pd.0 as i32)]);

        for dir in 0..1024 {
            let pde = page_directory[dir]; // PPN(20) | Flags(12)
            if pde & PTE_P == 0 {
                // No mapping
                if let Some((va, pa, uwp)) = state {
                    let sz = va - from_va;
                    PageDir::dump(va - sz, pa - sz, sz, uwp);
                }
                state = None;
                continue;
            }
            let page_table: &'static [usize; 1024] = transmute((pde & !0xFFF) + KERNBASE.0);
            for table in 0..1024 {
                let pte = page_table[table];
                if pte & PTE_P == 0 {
                    if let Some((va, pa, uwp)) = state {
                        let sz = va - from_va;
                        PageDir::dump(va - sz, pa - sz, sz, uwp);
                    }
                    state = None;
                    continue;
                }
                let va = dir << 22 | table << 12;
                let pa = pte >> 12 << 12;
                let uwp = pde & pte & 7;
                if state == Some((va, pa, uwp)) {
                    // Do nothing.
                } else {
                    if let Some((va, pa, uwp)) = state {
                        let sz = va - from_va;
                        PageDir::dump(va - sz, pa - sz, sz, uwp);
                    }
                    from_va = va;
                }
                state = Some((va.wrapping_add(4096), pa.wrapping_add(4096), uwp));
            }
        }
        if let Some((va, pa, uwp)) = state {
            let sz = va.wrapping_sub(from_va);
            PageDir::dump(va.wrapping_sub(sz), pa.wrapping_sub(sz), sz, uwp);
        }
    }
    // Return the address of the PTE in page table pgdir
    // that corresponds to virtual address va.  If alloc!=0,
    // create any required page table pages.
    pub unsafe extern "C" fn walkpgdir(&mut self, va: V, alloc: bool) -> Option<V> {
        let pde = ((self.pd.0 as *mut pde_t).add(va.pdx())) as *mut PTE;
        let mut pgtab: V;
        if ((*pde).0 & PTE_P > 0) {
            pgtab = p2v((*pde).addr());
        } else {
            if !alloc {
                return None;
            }
            match kalloc() {
                None => {
                    return None;
                }
                Some(p) => {
                    pgtab = p;
                }
            }
            // Make sure all those PTE_P bits are zero.
            // TODO: use memset(pgtab, 0, PGSIZE).
            for i in 0..PGSIZE {
                *((pgtab + i).0 as *mut u8) = 0u8;
            }
            // The permissions here are overly generous, but they can
            // be further restricted by the permissions in the page table
            // entries, if necessary.
            *pde = PTE(v2p(pgtab).0 | PTE_P | PTE_W | PTE_U);
        }
        pgtab += va.ptx() * 4;
        return Some(pgtab);
    }
    // Unmap a present page map.
    pub unsafe extern "C" fn unmap(&mut self, va: V) {
        let mut a = va.pgrounddown();
        let pte = self.walkpgdir(a, false).unwrap().0 as *mut usize;
        assert_eq!(*pte & PTE_P, PTE_P, "no map");

        *pte = 0;
    }

    // Create PTEs for virtual addresses starting at va that refer to
    // physical addresses starting at pa. va and size might not
    // be page-aligned.
    // returns success or not.
    pub unsafe extern "C" fn mappages(
        &mut self,
        va: V,
        size: usize,
        mut pa: P,
        perm: usize,
    ) -> bool {
        assert!(size > 0);

        let mut a = va.pgrounddown();
        let last = (va + size.wrapping_sub(1)).pgrounddown();
        loop {
            let pte = self.walkpgdir(a, true);
            if pte.is_none() {
                return false;
            }
            let pte = pte.unwrap().0 as *mut usize;
            assert_eq!(*pte & PTE_P, 0, "remap");

            *pte = pa.0 | perm as usize | PTE_P;
            if a == last {
                break;
            }
            a += PGSIZE;
            pa += PGSIZE;
        }
        return true;
    }
}

// There is one page table per process, plus one that's used when
// a CPU is not running any process (kpgdir). The kernel uses the
// current process's page table during system calls and interrupts;
// page protection bits prevent user code from using the kernel's
// mappings.
//
// setupkvm() and exec() set up every page table like this:
//
//   0..KERNBASE: user memory (text+data+stack+heap), mapped to
//                phys memory allocated by the kernel
//   KERNBASE..KERNBASE+EXTMEM: mapped to 0..EXTMEM (for I/O space)
//   KERNBASE+EXTMEM..data: mapped to EXTMEM..V2P(data)
//                for the kernel's instructions and r/o data
//   data..KERNBASE+PHYSTOP: mapped to V2P(data)..PHYSTOP,
//                                  rw data + free physical memory
//   0xfe000000..0: mapped direct (devices such as ioapic)
//
// The kernel allocates physical memory for its heap and for user memory
// between V2P(end) and the end of physical memory (PHYSTOP)
// (directly addressable from end..P2V(PHYSTOP)).

// This table defines the kernel's mappings, which are present in
// every process's page table.
pub struct Kmap {
    virt: V,
    phys_start: P,
    phys_end: P,
    perm: usize,
}

impl Kmap {
    pub const fn new(virt: V, phys_start: P, phys_end: P, perm: usize) -> Kmap {
        Kmap {
            virt,
            phys_start,
            phys_end,
            perm,
        }
    }
}

fn kmap() -> [Kmap; 4] {
    [
        Kmap::new(KERNBASE, P(0), EXTMEM, PTE_W), // I/O space
        Kmap::new(KERNLINK, v2p(KERNLINK), v2p(linker::data()), 0), // kern text+rodata
        Kmap::new(linker::data(), v2p(linker::data()), PHYSTOP, PTE_W), // kern data+memory
        Kmap::new(V(DEVSPACE), P(DEVSPACE), P(0), PTE_W), // more devices
    ]
}

// Set up kernel part of a page table.
pub unsafe extern "C" fn setupkvm() -> Option<PageDir> {
    let p = kalloc();
    if p.is_none() {
        return None;
    }
    let mut pgdir = PageDir { pd: p.unwrap() };
    // memset(pgdir, 0, PGSIZE);
    for i in 0..PGSIZE {
        *((pgdir.pd.0 + i) as *mut u8) = 0u8;
    }

    if p2v(PHYSTOP).0 > DEVSPACE {
        cpanic("PHYSTOP too high");
    }

    for k in kmap().into_iter() {
        if !pgdir.mappages(
            k.virt,
            (k.phys_end.0 as i32 - k.phys_start.0 as i32) as usize,
            k.phys_start,
            k.perm,
        ) {
            return None;
        }
    }
    return Some(pgdir);
}

// Allocate one page table for the machine for the kernel address
// space for scheduler processes.
pub unsafe extern "C" fn kvmalloc() {
    kpgdir = setupkvm().expect("kvmalloc");
    switchkvm();
}

// Switch h/w page table register to the kernel-only page table,
// for when no process is running.
pub unsafe extern "C" fn switchkvm() {
    // cprintf("switchkvm\n", &[]);
    lcr3(v2p(kpgdir.pd).0 as usize); // switch to the kernel page table
}

// Switch TSS and h/w page table to correspond to process p.
pub unsafe extern "C" fn switchuvm(p: *const Proc) {
    if (p == null_mut()) {
        cpanic("switchuvm: no process");
    }
    if ((*p).kstack == null_mut()) {
        cpanic("switchuvm: no kstack");
    }
    if ((*p).kstackguard == null_mut()) {
        cpanic("switchuvm: no kstackguard");
    }
    if ((*p).pgdir == null_mut()) {
        cpanic("switchuvm: no pgdir");
    }

    pushcli();

    // cprintf("uvm ...\n", &[]);
    ((*mycpu()).gdt)[SEG_TSS] = SEG16(
        STS_T32A,
        (&(*mycpu()).ts as *const Taskstate) as usize,
        size_of_val(&(*mycpu()).ts) - 1,
        0,
    );
    (*mycpu()).gdt[SEG_TSS].set_s(0);
    (*mycpu()).ts.ss0 = (SEG_KDATA << 3) as u16;
    (*mycpu()).ts.esp0 = (*p).kstack as usize + KSTACKSIZE;
    // setting IOPL=0 in eflags *and* iomb beyond the tss segment limit
    // forbids I/O instructions (e.g., inb and outb) from user space
    (*mycpu()).ts.iomb = 0xFFFFu16;
    ltr((SEG_TSS << 3) as u16);
    lcr3(V2P((*p).pgdir as usize)); // switch to process's address space
    // cprintf("uvm start\n", &[]);
    popcli();
}

// Load the initcode into address 0 of pgdir.
// sz must be less than a page.
pub unsafe extern "C" fn inituvm(pgdir: *mut pde_t, init: *mut u8, sz: usize) {
    if sz >= PGSIZE {
        cpanic("inituvm: more than a page");
    }
    let mem: *mut u8 = kalloc().map(|v| v.0).unwrap_or(0) as *mut u8;
    memset(mem, 0, PGSIZE);

    (PageDir {
        pd: V(pgdir as usize),
    })
    .mappages(V(0), PGSIZE, v2p(V(mem as usize)), PTE_W | PTE_U);
    memmove(mem, init, sz as usize);
}

// Load a program segment into pgdir.  addr must be page-aligned
// and the pages from addr to addr+sz must already be mapped.
pub unsafe extern "C" fn loaduvm(
    pgdir: *mut pde_t,
    addr: *mut u8,
    ip: *mut Inode,
    offset: usize,
    sz: usize,
) -> i32 {
    if ((addr as usize) % PGSIZE != 0) {
        cpanic("loaduvm: addr must be page aligned");
    }
    for i in (0..sz).step_by(PGSIZE) {
        let pte = (&mut PageDir {
            pd: V(pgdir as usize),
        })
            .walkpgdir(V(addr.add(i) as usize), false);
        if pte.is_none() {
            cpanic("loaduvm: address should exist");
        }
        let pte = pte.unwrap().0 as *mut pte_t;
        let pa = PTE(*pte).addr().0;
        let n: usize;
        if (sz - i < PGSIZE) {
            n = sz - i;
        } else {
            n = PGSIZE;
        }
        if (readi(ip, P2V(pa as *mut u8), offset + i, n) != n as i32) {
            return -1;
        }
    }
    return 0;
}

// Allocate page tables and physical memory to grow process from oldsz to
// newsz, which need not be page aligned.  Returns new size or 0 on error.
pub unsafe extern "C" fn allocuvm(pgdir: *mut pde_t, oldsz: usize, newsz: usize) -> usize {
    if (newsz >= KERNBASE.0) {
        return 0;
    }
    if (newsz < oldsz) {
        return oldsz;
    }

    let mut a = PGROUNDUP(oldsz);
    while a < newsz {
        let mem = kalloc();
        if (mem.is_none()) {
            cprintf("allocuvm out of memory\n", &[]);
            deallocuvm(pgdir, newsz, oldsz);
            return 0;
        }
        let mem = mem.unwrap();
        memset(mem.0 as *mut u8, 0, PGSIZE);
        if !((&mut PageDir {
            pd: V(pgdir as usize),
        })
            .mappages(V(a), PGSIZE, v2p(mem), PTE_W | PTE_U))
        {
            cprintf("allocuvm out of memory (2)\n", &[]);
            deallocuvm(pgdir, newsz, oldsz);
            kfree(mem);
            return 0;
        }
        a += PGSIZE;
    }
    return newsz;
}

// Deallocate user pages to bring the process size from oldsz to
// newsz.  oldsz and newsz need not be page-aligned, nor does newsz
// need to be less than oldsz.  oldsz can be larger than the actual
// process size.  Returns the new process size.
pub unsafe extern "C" fn deallocuvm(pgdir: *mut pde_t, oldsz: usize, newsz: usize) -> usize {
    if (newsz >= oldsz) {
        return oldsz;
    }

    let mut a = PGROUNDUP(newsz);
    while a < oldsz {
        let pte = (&mut PageDir {
            pd: V(pgdir as usize),
        })
            .walkpgdir(V(a), false);
        if (pte.is_none()) {
            a = V::pgaddr(V(a).pdx() + 1, 0, 0).0 - PGSIZE;
        } else if (*(pte.unwrap().0 as *const pte_t) & PTE_P) != 0 {
            let pa = PTE(*(pte.unwrap().0 as *const pte_t)).addr();
            if (pa.0 == 0) {
                cpanic("kfree");
            }
            let v = p2v(pa);
            kfree(v);
            *(pte.unwrap().0 as *mut pte_t) = 0;
        }
        a += PGSIZE;
    }
    return newsz;
}

// Free a page table and all the physical memory pages
// in the user part.
pub unsafe extern "C" fn freevm(pgdir: *mut pde_t) {
    if (pgdir == null_mut()) {
        cpanic("freevm: no pgdir");
    }
    deallocuvm(pgdir, KERNBASE.0, 0);
    for i in 0..NPDENTRIES {
        if (*(pgdir.add(i)) & PTE_P) != 0 {
            let v = p2v(PTE(*(pgdir.add(i))).addr());
            kfree(v);
        }
    }
    kfree(V(pgdir as usize));
}

// Clear PTE_U on a page. Used to create an inaccessible
// page beneath the user stack.
pub unsafe extern "C" fn clearpteu(pgdir: *mut pde_t, uva: *mut u8) {
    let pte = (&mut PageDir {
        pd: V(pgdir as usize),
    })
        .walkpgdir(V(uva as usize), false);
    if (pte.is_none()) {
        cpanic("clearpteu");
    }
    *(pte.unwrap().0 as *mut pte_t) &= !PTE_U;
}

// Given a parent process's page table, create a copy
// of it for a child.
pub unsafe extern "C" fn copyuvm(pgdir: *mut pde_t, sz: usize) -> *mut pde_t {
    let mut pgdir = PageDir::from(pgdir);
    let mut d = setupkvm();
    if (d.is_none()) {
        return null_mut();
    }
    let mut d = d.unwrap();
    let mut bad = false;
    for i in (0..sz).step_by(PGSIZE) {
        let pte = pgdir.walkpgdir(V(i), false);
        if pte.is_none() {
            cpanic("copyuvm: pte should exist");
        }
        let pte = pte.unwrap().0 as *mut pde_t;
        if ((*pte & PTE_P) == 0) {
            if i != (*myproc()).kstackguard as usize {
                cpanic("copyuvm: page not present");
            }
            continue;
        }
        let pa = PTE(*pte).addr();
        let flags = PTE(*pte).flags();
        let mem = kalloc();
        if (mem.is_none()) {
            cprintf("error: copyuvm: mem is none\n", &[]);
            bad = true;
            break;
        }
        let mem = mem.unwrap();
        memmove(mem.0 as *mut u8, p2v(pa).0 as *const u8, PGSIZE);
        if !d.mappages(V(i), PGSIZE, v2p(mem), flags) {
            cprintf("error: copyuvm: mappages fail\n", &[]);
            bad = true;
            break;
        }
    }
    if !bad {
        return d.pd.0 as *mut pde_t;
    }

    freevm(d.pd.0 as *mut pde_t);
    return null_mut();
}

// Map user virtual address to kernel address.
pub unsafe extern "C" fn uva2ka(pgdir: *mut pde_t, uva: *mut u8) -> *mut u8 {
    let pte = (&mut PageDir {
        pd: V(pgdir as usize),
    })
        .walkpgdir(V(uva as usize), false)
        .unwrap()
        .0 as *const pte_t;
    if ((*pte & PTE_P) == 0) {
        return null_mut();
    }
    if ((*pte & PTE_U) == 0) {
        return null_mut();
    }
    return p2v(PTE(*pte).addr()).0 as *mut u8;
}

// Copy len bytes from p to user address va in page table pgdir.
// Most useful when pgdir is not the current page table.
// uva2ka ensures this only works for PTE_U pages.
pub unsafe extern "C" fn copyout(
    pgdir: *mut pde_t,
    mut va: usize,
    p: *mut (),
    mut len: usize,
) -> i32 {
    let mut buf = p as *mut u8;
    while (len > 0) {
        let mut va0 = PGROUNDDOWN(va) as usize;
        let pa0 = uva2ka(pgdir, va0 as *mut u8);
        if (pa0 == null_mut()) {
            return -1;
        }
        let mut n = PGSIZE - (va - va0);
        if (n > len) {
            n = len;
        }
        memmove(pa0.add(va - va0), buf, n);
        len -= n;
        buf = buf.add(n);
        va = va0 + PGSIZE;
    }
    return 0;
}
