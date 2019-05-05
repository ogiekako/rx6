use x86::*;

pub unsafe extern "C" fn memset(dst: *mut u8, mut c: i32, n: usize) {
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

pub unsafe extern "C" fn memcmp(mut v1: *const u8, mut v2: *const u8, n: usize) -> u8 {
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

pub unsafe extern "C" fn memmove(mut dst: *mut u8, mut src: *const u8, n: usize) -> *mut () {
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
    dst as *mut ()
}

// memcpy exists to placate GCC.  Use memmove.
pub unsafe extern "C" fn memcpy(dst: *mut (), src: *const (), n: usize) -> *mut () {
    memmove(dst as *mut u8, src as *const u8, n)
}

pub unsafe extern "C" fn strncmp(mut p: *const u8, mut q: *const u8, mut n: usize) -> i32 {
    loop {
        if n <= 0 || *p == 0 || *p != *q {
            break;
        }
        n -= 1;
        p = p.offset(1);
        q = q.offset(1);
    }
    if (n == 0) {
        return 0;
    }
    (*p as i32) - (*q as i32)
}

pub unsafe extern "C" fn strncpy(mut s: *mut u8, mut t: *const u8, mut n: i32) -> *mut u8 {
    let os = s;
    loop {
        n -= 1;
        if n < 0 {
            break;
        }
        *s = *t;
        let x = *s;
        s = s.offset(1);
        t = t.offset(1);
        if x == 0 {
            break;
        }
    }
    while n > 0 {
        n -= 1;
        *s = 0;
        s = s.offset(1);
    }
    return os;
}

// Like strncpy but guaranteed to NUL-terminate.
pub unsafe extern "C" fn safestrcpy(mut s: *mut u8, mut t: *const u8, mut n: i32) -> *mut u8 {
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

pub unsafe extern "C" fn strlen(s: *const u8) -> i32 {
    let mut n = 0;
    while *(s.offset(n)) != 0 {
        n += 1;
    }
    return n as i32;
}
