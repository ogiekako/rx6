// // Physical memory allocator, intended to allocate
// // memory for user processes, kernel stacks, page table pages,
// // and pipe buffers. Allocates 4096-byte pages.

use core;

// #include "types.h"
// #include "defs.h"
// #include "param.h"
// #include "memlayout.h"
use mmu::*;
// #include "spinlock.h"
//
// extern char end[]; // first address after kernel loaded from ELF file

struct Run {
    next: Option<&'static mut Run>,
}

struct Kmem {
//   struct spinlock lock;
//   int use_lock;
//   struct run *freelist;
}

static mut kmem: Option<&'static mut Run> = None;

// Initialization happens in two phases.
// 1. main() calls kinit1() while still using entrypgdir to place just
// the pages mapped by entrypgdir on free list.
// 2. main() calls kinit2() with the rest of the physical pages
// after installing a full page table that maps them on all cores.
pub unsafe fn kinit1(vstart: V, vend: V) {
    assert!(vstart < vend);
    // initlock(&kmem.lock, "kmem");
    // kmem.use_lock = 0;
    freerange(vstart, vend);
}

// void
// kinit2(void *vstart, void *vend)
// {
//   freerange(vstart, vend);
//   kmem.use_lock = 1;
// }

unsafe fn freerange(vstart: V, vend: V) {
    let mut p = vstart.pgroundup();
    while p + PGSIZE <= vend {
        kfree(p);
        p += PGSIZE;
    }
}

// Free the page of physical memory pointed at by v,
// which normally should have been returned by a
// call to kalloc().  (The exception is when
// initializing the allocator; see kinit above.)
unsafe fn kfree(v: V) {
    //  if((uint)v % PGSIZE || v < end || V2P(v) >= PHYSTOP)
    //    panic("kfree");
    //
    //  // Fill with junk to catch dangling refs.
    //  memset(v, 1, PGSIZE);
    //
    //  if(kmem.use_lock)
    //    acquire(&kmem.lock);
    let r: *mut Run = v.as_ptr() as usize as *mut Run;
    *r = Run { next: kmem.take() };
    kmem = Some(&mut *r);
    // kmem.freelist = r;

    //  if(kmem.use_lock)
    //    release(&kmem.lock);
}

// Allocate one 4096-byte page of physical memory.
// Returns a pointer that the kernel can use.
// Returns None if the memory cannot be allocated.
pub fn kalloc() -> Option<V> {
    unsafe {
        //  if(kmem.use_lock)
        //    acquire(&kmem.lock);
        let res = if (&kmem).is_none() {
            None
        } else {
            let a = &mut kmem.take().unwrap().next;
            let p = V(a as *const Option<&'static mut Run> as usize);
            kmem = a.take();
            Some(p)
        };
        //   if(kmem.use_lock)
        //     release(&kmem.lock);
        res
    }
}

#[cfg(test)]
mod tests {
    use core;
    use mmu::PGSIZE;

    #[test]
    fn kfree_kalloc() {
        unsafe {
            assert_eq!(super::kalloc(), None);

            let a = [100u8; PGSIZE as usize * 10];
            let mut v = V(core::mem::transmute(&a)).pgroundup();

            let one = v;
            let two = v + PGSIZE;

            super::kfree(two); // head = two
            super::kfree(one); // head = one -> two

            let mut x = super::kalloc().unwrap(); // head = two
            assert_eq!(one, x);
            for i in 0..(PGSIZE as usize) {
                *(x.0 as *mut u8) = 42;
                x = x + 1;
            }
            assert_eq!(a[PGSIZE.wrapping_sub(1) as usize], 42);

            let x = super::kalloc().unwrap();
            assert_eq!(two, x);
            let r: *const super::Run = core::mem::transmute(x.0);
            assert!((*r).next.is_none());

            assert_eq!(super::kalloc(), None);
        }
    }
}
