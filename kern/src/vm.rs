use core;

use kalloc::*;
use linker;
use memlayout::*;
use mmu::*;
use x86;

// #include "param.h"
// #include "types.h"
// #include "defs.h"
// #include "x86.h"
// #include "memlayout.h"
// #include "mmu.h"
// #include "proc.h"
// #include "elf.h"
//
// // Set up CPU's kernel segment descriptors.
// // Run once on entry on each CPU.
// void
// seginit(void)
// {
//   struct cpu *c;
//
//   // Map "logical" addresses to virtual addresses using identity map.
//   // Cannot share a CODE descriptor for both kernel and user
//   // because it would have to have DPL_USR, but the CPU forbids
//   // an interrupt from CPL=0 to DPL=3.
//   c = &cpus[cpunum()];
//   c->gdt[SEG_KCODE] = SEG(STA_X|STA_R, 0, 0xffffffff, 0);
//   c->gdt[SEG_KDATA] = SEG(STA_W, 0, 0xffffffff, 0);
//   c->gdt[SEG_UCODE] = SEG(STA_X|STA_R, 0, 0xffffffff, DPL_USER);
//   c->gdt[SEG_UDATA] = SEG(STA_W, 0, 0xffffffff, DPL_USER);
//
//   // Map cpu and proc -- these are private per cpu.
//   c->gdt[SEG_KCPU] = SEG(STA_W, &c->cpu, 8, 0);
//
//   lgdt(c->gdt, sizeof(c->gdt));
//   loadgs(SEG_KCPU << 3);
//
//   // Initialize cpu-local storage.
//   cpu = c;
//   proc = 0;
// }

// for use in scheduler()
static mut kpgdir: PageDir = PageDir { pd: V(0) };

struct PageDir {
    pd: V, // [pd, pd+PGSIZE)
}

impl PageDir {
    // Return the address of the PTE in page table pgdir
    // that corresponds to virtual address va.  If alloc!=0,
    // create any required page table pages.
    unsafe fn walkpgdir(&mut self, va: V, alloc: bool) -> Option<V> {
        let pde = (self.pd.0 + va.pdx() * 4) as *mut PTE;
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
            // TODO: memset(pgtab, 0, PGSIZE);
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

    // Create PTEs for virtual addresses starting at va that refer to
    // physical addresses starting at pa. va and size might not
    // be page-aligned.
    // returns success or not.
    unsafe fn mappages(&mut self, va: V, size: usize, mut pa: P, perm: usize) -> bool {
        {
            assert!(size > 0);

            let mut a = va.pgrounddown();
            let last = (va + size.wrapping_sub(1)).pgrounddown();
            loop {
                let pte = self.walkpgdir(a, true);
                if pte.is_none() {
                    return false;
                }
                let mut pte = pte.unwrap().0 as *mut usize;
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
struct Kmap {
    virt: V,
    phys_start: P,
    phys_end: P,
    perm: usize,
}

impl Kmap {
    fn new(virt: V, phys_start: P, phys_end: P, perm: usize) -> Kmap {
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
unsafe fn setupkvm() -> Option<PageDir> {
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
        panic!("PHYSTOP too high");
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
pub unsafe fn kvmalloc() {
    kpgdir = setupkvm().expect("kvmalloc");
    switchkvm();
}

// Switch h/w page table register to the kernel-only page table,
// for when no process is running.
unsafe fn switchkvm() {
    x86::lcr3(v2p(kpgdir.pd).0 as u32); // switch to the kernel page table
}

// // Switch TSS and h/w page table to correspond to process p.
// void
// switchuvm(struct proc *p)
// {
//   if(p == 0)
//     panic("switchuvm: no process");
//   if(p->kstack == 0)
//     panic("switchuvm: no kstack");
//   if(p->pgdir == 0)
//     panic("switchuvm: no pgdir");
//
//   pushcli();
//   cpu->gdt[SEG_TSS] = SEG16(STS_T32A, &cpu->ts, sizeof(cpu->ts)-1, 0);
//   cpu->gdt[SEG_TSS].s = 0;
//   cpu->ts.ss0 = SEG_KDATA << 3;
//   cpu->ts.esp0 = (uint)p->kstack + KSTACKSIZE;
//   // setting IOPL=0 in eflags *and* iomb beyond the tss segment limit
//   // forbids I/O instructions (e.g., inb and outb) from user space
//   cpu->ts.iomb = (ushort) 0xFFFF;
//   ltr(SEG_TSS << 3);
//   lcr3(V2P(p->pgdir));  // switch to process's address space
//   popcli();
// }
//
// // Load the initcode into address 0 of pgdir.
// // sz must be less than a page.
// void
// inituvm(pde_t *pgdir, char *init, uint sz)
// {
//   char *mem;
//
//   if(sz >= PGSIZE)
//     panic("inituvm: more than a page");
//   mem = kalloc();
//   memset(mem, 0, PGSIZE);
//   mappages(pgdir, 0, PGSIZE, V2P(mem), PTE_W|PTE_U);
//   memmove(mem, init, sz);
// }
//
// // Load a program segment into pgdir.  addr must be page-aligned
// // and the pages from addr to addr+sz must already be mapped.
// int
// loaduvm(pde_t *pgdir, char *addr, struct inode *ip, uint offset, uint sz)
// {
//   uint i, pa, n;
//   pte_t *pte;
//
//   if((uint) addr % PGSIZE != 0)
//     panic("loaduvm: addr must be page aligned");
//   for(i = 0; i < sz; i += PGSIZE){
//     if((pte = walkpgdir(pgdir, addr+i, 0)) == 0)
//       panic("loaduvm: address should exist");
//     pa = PTE_ADDR(*pte);
//     if(sz - i < PGSIZE)
//       n = sz - i;
//     else
//       n = PGSIZE;
//     if(readi(ip, P2V(pa), offset+i, n) != n)
//       return -1;
//   }
//   return 0;
// }
//
// // Allocate page tables and physical memory to grow process from oldsz to
// // newsz, which need not be page aligned.  Returns new size or 0 on error.
// int
// allocuvm(pde_t *pgdir, uint oldsz, uint newsz)
// {
//   char *mem;
//   uint a;
//
//   if(newsz >= KERNBASE)
//     return 0;
//   if(newsz < oldsz)
//     return oldsz;
//
//   a = PGROUNDUP(oldsz);
//   for(; a < newsz; a += PGSIZE){
//     mem = kalloc();
//     if(mem == 0){
//       cprintf("allocuvm out of memory\n");
//       deallocuvm(pgdir, newsz, oldsz);
//       return 0;
//     }
//     memset(mem, 0, PGSIZE);
//     if(mappages(pgdir, (char*)a, PGSIZE, V2P(mem), PTE_W|PTE_U) < 0){
//       cprintf("allocuvm out of memory (2)\n");
//       deallocuvm(pgdir, newsz, oldsz);
//       kfree(mem);
//       return 0;
//     }
//   }
//   return newsz;
// }
//
// // Deallocate user pages to bring the process size from oldsz to
// // newsz.  oldsz and newsz need not be page-aligned, nor does newsz
// // need to be less than oldsz.  oldsz can be larger than the actual
// // process size.  Returns the new process size.
// int
// deallocuvm(pde_t *pgdir, uint oldsz, uint newsz)
// {
//   pte_t *pte;
//   uint a, pa;
//
//   if(newsz >= oldsz)
//     return oldsz;
//
//   a = PGROUNDUP(newsz);
//   for(; a  < oldsz; a += PGSIZE){
//     pte = walkpgdir(pgdir, (char*)a, 0);
//     if(!pte)
//       a = PGADDR(PDX(a) + 1, 0, 0) - PGSIZE;
//     else if((*pte & PTE_P) != 0){
//       pa = PTE_ADDR(*pte);
//       if(pa == 0)
//         panic("kfree");
//       char *v = P2V(pa);
//       kfree(v);
//       *pte = 0;
//     }
//   }
//   return newsz;
// }
//
// // Free a page table and all the physical memory pages
// // in the user part.
// void
// freevm(pde_t *pgdir)
// {
//   uint i;
//
//   if(pgdir == 0)
//     panic("freevm: no pgdir");
//   deallocuvm(pgdir, KERNBASE, 0);
//   for(i = 0; i < NPDENTRIES; i++){
//     if(pgdir[i] & PTE_P){
//       char * v = P2V(PTE_ADDR(pgdir[i]));
//       kfree(v);
//     }
//   }
//   kfree((char*)pgdir);
// }
//
// // Clear PTE_U on a page. Used to create an inaccessible
// // page beneath the user stack.
// void
// clearpteu(pde_t *pgdir, char *uva)
// {
//   pte_t *pte;
//
//   pte = walkpgdir(pgdir, uva, 0);
//   if(pte == 0)
//     panic("clearpteu");
//   *pte &= ~PTE_U;
// }
//
// // Given a parent process's page table, create a copy
// // of it for a child.
// pde_t*
// copyuvm(pde_t *pgdir, uint sz)
// {
//   pde_t *d;
//   pte_t *pte;
//   uint pa, i, flags;
//   char *mem;
//
//   if((d = setupkvm()) == 0)
//     return 0;
//   for(i = 0; i < sz; i += PGSIZE){
//     if((pte = walkpgdir(pgdir, (void *) i, 0)) == 0)
//       panic("copyuvm: pte should exist");
//     if(!(*pte & PTE_P))
//       panic("copyuvm: page not present");
//     pa = PTE_ADDR(*pte);
//     flags = PTE_FLAGS(*pte);
//     if((mem = kalloc()) == 0)
//       goto bad;
//     memmove(mem, (char*)P2V(pa), PGSIZE);
//     if(mappages(d, (void*)i, PGSIZE, V2P(mem), flags) < 0)
//       goto bad;
//   }
//   return d;
//
// bad:
//   freevm(d);
//   return 0;
// }
//
// //PAGEBREAK!
// // Map user virtual address to kernel address.
// char*
// uva2ka(pde_t *pgdir, char *uva)
// {
//   pte_t *pte;
//
//   pte = walkpgdir(pgdir, uva, 0);
//   if((*pte & PTE_P) == 0)
//     return 0;
//   if((*pte & PTE_U) == 0)
//     return 0;
//   return (char*)P2V(PTE_ADDR(*pte));
// }
//
// // Copy len bytes from p to user address va in page table pgdir.
// // Most useful when pgdir is not the current page table.
// // uva2ka ensures this only works for PTE_U pages.
// int
// copyout(pde_t *pgdir, uint va, void *p, uint len)
// {
//   char *buf, *pa0;
//   uint n, va0;
//
//   buf = (char*)p;
//   while(len > 0){
//     va0 = (uint)PGROUNDDOWN(va);
//     pa0 = uva2ka(pgdir, (char*)va0);
//     if(pa0 == 0)
//       return -1;
//     n = PGSIZE - (va - va0);
//     if(n > len)
//       n = len;
//     memmove(pa0 + (va - va0), buf, n);
//     len -= n;
//     buf += n;
//     va = va0 + PGSIZE;
//   }
//   return 0;
// }
//
// //PAGEBREAK!
// // Blank page.
// //PAGEBREAK!
// // Blank page.
// //PAGEBREAK!
// // Blank page.
//