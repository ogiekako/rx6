// #include "types.h"
// #include "x86.h"
//
// void*
// memset(void *dst, int c, uint n)
// {
//   if ((int)dst%4 == 0 && n%4 == 0){
//     c &= 0xFF;
//     stosl(dst, (c<<24)|(c<<16)|(c<<8)|c, n/4);
//   } else
//     stosb(dst, c, n);
//   return dst;
// }

pub unsafe fn memcmp(mut v1: *const u8, mut v2: *const u8, n: usize) -> u8 {
    for i in 0..n {
        if *v1 != *v2 {
            return (*v1).wrapping_sub(*v2);
        }
        v1 = v1.offset(1);
        v2 = v2.offset(1);
    }

    return 0;
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

// void*
// memmove(void *dst, const void *src, uint n)
// {
//   const char *s;
//   char *d;
//
//   s = src;
//   d = dst;
//   if(s < d && s + n > d){
//     s += n;
//     d += n;
//     while(n-- > 0)
//       *--d = *--s;
//   } else
//     while(n-- > 0)
//       *d++ = *s++;
//
//   return dst;
// }
//
// // memcpy exists to placate GCC.  Use memmove.
// void*
// memcpy(void *dst, const void *src, uint n)
// {
//   return memmove(dst, src, n);
// }
//
// int
// strncmp(const char *p, const char *q, uint n)
// {
//   while(n > 0 && *p && *p == *q)
//     n--, p++, q++;
//   if(n == 0)
//     return 0;
//   return (uchar)*p - (uchar)*q;
// }
//
// char*
// strncpy(char *s, const char *t, int n)
// {
//   char *os;
//
//   os = s;
//   while(n-- > 0 && (*s++ = *t++) != 0)
//     ;
//   while(n-- > 0)
//     *s++ = 0;
//   return os;
// }
//
// // Like strncpy but guaranteed to NUL-terminate.
// char*
// safestrcpy(char *s, const char *t, int n)
// {
//   char *os;
//
//   os = s;
//   if(n <= 0)
//     return os;
//   while(--n > 0 && (*s++ = *t++) != 0)
//     ;
//   *s = 0;
//   return os;
// }
//
// int
// strlen(const char *s)
// {
//   int n;
//
//   for(n = 0; s[n]; n++)
//     ;
//   return n;
// }
//
