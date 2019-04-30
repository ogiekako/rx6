// Console input and output.
// Input is from the keyboard or serial port.
// Output is written to the screen and serial port.

use super::*;
use spinlock::spinlock;
use core;

static mut panicked: bool = false;

struct Cons {
    lock: spinlock,
    locking: i32,
}

impl Cons {
    const unsafe fn uninit() -> Cons {
        Cons {
            lock: spinlock::uninit(),
            locking: 0,
        }
    }
}

static mut cons: Cons = Cons::uninit();

//// static struct {
////   struct spinlock lock;
////   int locking;
//// } cons;

unsafe fn printint(xx: i32, base: u32, sign: bool) {
    let mut negative = false;
    let mut x = if (sign && xx < 0) {
        negative = true;
        -xx as u32
    } else {
        xx as u32
    };

    let digits = "0123456789abcdef";
    let mut buf = [0u8; 16];
    let mut i = 0;
    loop {
        buf[i] = digits.as_bytes()[(x % base) as usize];
        i += 1;
        x /= base;
        if x == 0 {
            break;
        }
    }

    if negative {
        buf[i] = '-' as u8;
        i += 1;
    }

    for j in (0..i).rev() {
        consputc(buf[j] as u16);
    }
}

pub enum Arg<'a> {
    Int(i32),
    Str(&'a str),
}

// Print to the console. only understands %d, %x, %p, %s.
pub unsafe fn cprintf(fmt: &str, args: &[Arg]) {
    // TOOD: use lock.
    //// locking = cons.locking;
    //// if(locking)
    //// acquire(&cons.lock);

    let mut fmtit = fmt.chars();
    let mut argit = args.iter();
    loop {
        let c = fmtit.next();
        if c.is_none() {
            break;
        }
        let c = c.unwrap();
        if (c != '%') {
            consputc(c as u16);
            continue;
        }
        let c = fmtit.next();
        if (c.is_none()) {
            break;
        }
        match c.unwrap() {
            'd' => {
                if let Some(Arg::Int(i)) = argit.next() {
                    printint(*i, 10, true);
                } else {
                    panic!();
                }
            }
            'x' | 'p' => {
                if let Some(Arg::Int(i)) = argit.next() {
                    printint(*i, 16, false);
                } else {
                    panic!();
                }
            }
            's' => {
                if let Some(Arg::Str(s)) = argit.next() {
                    for c in s.chars() {
                        consputc(c as u16);
                    }
                } else {
                    panic!();
                }
            }
            '%' => {
                consputc('%' as u16);
            }
            c => {
                // Print unknown % sequence to draw attention.
                consputc('%' as u16);
                consputc(c as u16);
            }
        }

        //// if(locking)
        //// release(&cons.lock);
    }
}

pub unsafe fn panic(s: *mut str) {
    let mut pcs = [0u32; 10];
    let mut i = 0;

    cli();
    cons.locking = 0;
    //// cprintf("cpu %d: panic: ", cpuid());
    //// cprintf(s);
    //// cprintf("\n");
    getcallerpcs(&s, &mut pcs);
    //// for(i=0; i<10; i++)
    ////   cprintf(" %p", pcs[i]);
    panicked = true; // freeze other CPU
    loop {}
}

pub const BACKSPACE: u16 = 0x100;
pub const CRTPORT: u16 = 0x3d4;
static mut crt: *mut u16 = p2v(P(0xb8000)).as_ptr() as *mut u16; // CGA memory

unsafe fn cgaputc(c: i32) {
    // Cursor position: col + 80*row.
    outb(CRTPORT, 14);
    let mut pos = (inb(CRTPORT + 1) as isize) << 8;
    outb(CRTPORT, 15);
    pos |= inb(CRTPORT + 1) as isize;

    if (c == '\n' as i32) {
        pos += 80 - pos % 80;
    } else if (c == BACKSPACE as i32) {
        if (pos > 0) {
            pos = pos - 1;
        }
    } else {
        *(crt.offset(pos)) = ((c & 0xff) | 0x0700) as u16; // black on white
        pos += 1;
    }

    if (pos < 0 || pos > 25 * 80) {
        panic!("pos under/overflow");
    }

    if ((pos / 80) >= 24) {
        // Scroll up.
        memmove(
            crt as usize as *mut u8,
            crt.offset(80) as usize as *mut u8,
            core::mem::size_of_val(&*crt) * 23 * 80,
        );
        pos -= 80;
        memset(
            crt.offset(pos) as usize as *mut u8,
            0,
            core::mem::size_of_val(&*crt) * ((24 * 80 - pos) as usize),
        );
    }

    outb(CRTPORT, 14);
    outb(CRTPORT + 1, (pos >> 8) as u8);
    outb(CRTPORT, 15);
    outb(CRTPORT + 1, pos as u8);
    *(crt.offset(pos)) = ' ' as u16 | 0x0700;
}

unsafe fn consputc(c: u16) {
    if (panicked) {
        cli();
        loop {}
    }

    if (c == BACKSPACE) {
        uartputc('\x08' as u8);
        uartputc(' ' as u8);
        uartputc('\x08' as u8);
    } else if c < 0xff {
        uartputc(c as u8);
    } else {
        panic!();
    }
    cgaputc(c.into());
}

// #define INPUT_BUF 128
//// struct {
////   char buf[INPUT_BUF];
////   uint r;  // Read index
////   uint w;  // Write index
////   uint e;  // Edit index
//// } input;
////
//// #define C(x)  ((x)-'@')  // Control-x
////
//// void
//// consoleintr(int (*getc)(void))
//// {
////   int c, doprocdump = 0;
////
////   acquire(&cons.lock);
////   while((c = getc()) >= 0){
////     switch(c){
////     case C('P'):  // Process listing.
////       // procdump() locks cons.lock indirectly; invoke later
////       doprocdump = 1;
////       break;
////     case C('U'):  // Kill line.
////       while(input.e != input.w &&
////             input.buf[(input.e-1) % INPUT_BUF] != '\n'){
////         input.e--;
////         consputc(BACKSPACE);
////       }
////       break;
////     case C('H'): case '\x7f':  // Backspace
////       if(input.e != input.w){
////         input.e--;
////         consputc(BACKSPACE);
////       }
////       break;
////     default:
////       if(c != 0 && input.e-input.r < INPUT_BUF){
////         c = (c == '\r') ? '\n' : c;
////         input.buf[input.e++ % INPUT_BUF] = c;
////         consputc(c);
////         if(c == '\n' || c == C('D') || input.e == input.r+INPUT_BUF){
////           input.w = input.e;
////           wakeup(&input.r);
////         }
////       }
////       break;
////     }
////   }
////   release(&cons.lock);
////   if(doprocdump) {
////     procdump();  // now call procdump() wo. cons.lock held
////   }
//// }
////
//// int
//// consoleread(struct inode *ip, char *dst, int n)
//// {
////   uint target;
////   int c;
////
////   iunlock(ip);
////   target = n;
////   acquire(&cons.lock);
////   while(n > 0){
////     while(input.r == input.w){
////       if(myproc()->killed){
////         release(&cons.lock);
////         ilock(ip);
////         return -1;
////       }
////       sleep(&input.r, &cons.lock);
////     }
////     c = input.buf[input.r++ % INPUT_BUF];
////     if(c == C('D')){  // EOF
////       if(n < target){
////         // Save ^D for next time, to make sure
////         // caller gets a 0-byte result.
////         input.r--;
////       }
////       break;
////     }
////     *dst++ = c;
////     --n;
////     if(c == '\n')
////       break;
////   }
////   release(&cons.lock);
////   ilock(ip);
////
////   return target - n;
//// }
////
//// int
//// consolewrite(struct inode *ip, char *buf, int n)
//// {
////   int i;
////
////   iunlock(ip);
////   acquire(&cons.lock);
////   for(i = 0; i < n; i++)
////     consputc(buf[i] & 0xff);
////   release(&cons.lock);
////   ilock(ip);
////
////   return n;
//// }

pub unsafe fn consoleinit() {
    // TODO: lock
    //// initlock(&cons.lock, "console");

    //// devsw[CONSOLE].write = consolewrite;
    //// devsw[CONSOLE].read = consoleread;
    //// cons.locking = 1;

    picenable(IRQ_KBD as i32);
    ioapicenable(IRQ_KBD, 0);
}
