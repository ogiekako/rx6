use x86::*;

pub unsafe fn memset(dst: *mut u8, mut c: i32, n: usize) {
    if dst as usize % 4 == 0 && n % 4 == 0 {
        c &= 0xFF;
        stosl(
            dst as usize as *mut (),
            (c << 24) | (c << 16) | (c << 8) | c,
            n as i32 / 4,
        );
    } else {
        stosb(dst as usize as *mut (), c, n as i32);
    }
}

pub unsafe fn memcmp(mut v1: *const u8, mut v2: *const u8, n: usize) -> u8 {
    for i in 0..n {
        if *v1 != *v2 {
            return (*v1).wrapping_sub(*v2);
        }
        v1 = v1.offset(1);
        v2 = v2.offset(1);
    }
    0
}

#[cfg(test)]
mod tests {
    #[test]
    fn memcmp() {
        unsafe {
            assert!(super::memcmp("hoge".as_ptr(), "piyo".as_ptr(), 4) != 0);
            assert_eq!(super::memcmp("hoge".as_ptr(), "hoge".as_ptr(), 4), 0);
        }
    }
}

pub unsafe fn memmove(mut dst: *mut u8, mut src: *const u8, n: usize) {
    if src < dst && src.offset(n as isize) > dst {
        src = src.offset(n as isize);
        dst = dst.offset(n as isize);
        for i in 0..n {
            dst = dst.offset(-1);
            src = src.offset(-1);
            *dst = *src;
        }
    } else {
        for i in 0..n {
            *dst = *src;
            dst = dst.offset(1);
            src = src.offset(1);
        }
    }
}

// // memcpy exists to placate GCC.  Use memmove.
//// void*
//// memcpy(void *dst, const void *src, uint n)
//// {
////   return memmove(dst, src, n);
//// }
//
//// int
//// strncmp(const char *p, const char *q, uint n)
//// {
////   while(n > 0 && *p && *p == *q)
////     n--, p++, q++;
////   if(n == 0)
////     return 0;
////   return (uchar)*p - (uchar)*q;
//// }
//
//// char*
//// strncpy(char *s, const char *t, int n)
//// {
////   char *os;
////
////   os = s;
////   while(n-- > 0 && (*s++ = *t++) != 0)
////     ;
////   while(n-- > 0)
////     *s++ = 0;
////   return os;
//// }

// Like strncpy but guaranteed to NUL-terminate.
pub unsafe fn safestrcpy(mut s: *mut u8, mut t: *const u8, mut n: i32) -> *mut u8 {
    let mut os = s;
    if n <= 0 {
        return os;
    }
    loop {
        n -= 1;
        if n <= 0 {
            break;
        }
        *s = *t;
        if *s == 0 {
            break;
        }
        s = s.offset(1);
        t = t.offset(1);
    }
    *s = 0;
    os
}
//
//// int
//// strlen(const char *s)
//// {
////   int n;
////
////   for(n = 0; s[n]; n++)
////     ;
////   return n;
//// }
