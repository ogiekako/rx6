// memlayout.h
#define KERNBASE 0x80000000         // First kernel virtual address

// mmu.h
#define PGSIZE 4096
#define NPDENTRIES 1024    // # directory entries per page directory

#define PTE_P           0x001   // Present
#define PTE_W           0x002   // Writeable
#define PTE_PS          0x080   // Page Size

#define PDXSHIFT 22

typedef unsigned int pde_t;

pde_t entrypgdir[];

int main(void) {
}

__attribute__((__aligned__(PGSIZE)))
pde_t entrypgdir[NPDENTRIES] = {
  // Map VA's [0, 4MB) to PA's [0, 4MB)
  [0] = (0) | PTE_P | PTE_W | PTE_PS,
  // Map VA's [KERNBASE, KERNBASE+4MB) to PA's [0, 4MB)
  [KERNBASE>>PDXSHIFT] = (0) | PTE_P | PTE_W | PTE_PS,
};
