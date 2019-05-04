// Console input and output.
// Input is from the keyboard or serial port.
// Output is written to the screen and serial port.

use super::*;
use core;

static mut panicked: bool = false;

struct Cons {
    lock: Spinlock,
    locking: i32,
}

impl Cons {
    const unsafe fn uninit() -> Cons {
        Cons {
            lock: Spinlock::uninit(),
            locking: 0,
        }
    }
}

static mut cons: Cons = unsafe { Cons::uninit() };

unsafe fn printint(xx: i32, base: usize, sign: bool) {
    let mut negative = false;
    let mut x = if (sign && xx < 0) {
        negative = true;
        -xx as usize
    } else {
        xx as usize
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
    let locking = cons.locking;
    if (locking != 0) {
        acquire(&mut cons.lock as *mut Spinlock);
    }

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
                    cpanic("cprintf [d]");
                }
            }
            'x' | 'p' => {
                if let Some(Arg::Int(i)) = argit.next() {
                    printint(*i, 16, false);
                } else {
                    cpanic("cprintf [xp]");
                }
            }
            's' => {
                if let Some(Arg::Str(s)) = argit.next() {
                    for c in s.chars() {
                        consputc(c as u16);
                    }
                } else {
                    cpanic("cprintf [s]");
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

        if (locking != 0) {
            release(&mut cons.lock as *mut Spinlock);
        }
    }
}

pub unsafe fn cpanic(s: &str) -> ! {
    let mut pcs = [0usize; 10];
    let mut i = 0;

    cli();
    cons.locking = 0;
    cprintf("cpu %d: panic: ", &[Arg::Int(cpuid() as i32)]);
    cprintf(s, &[]);
    cprintf("\n", &[]);
    getcallerpcs(s.as_ptr() as *const (), &mut pcs);
    for i in 0..10 {
        cprintf(" %p", &[Arg::Int(pcs[i] as i32)]);
    }
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
        cpanic("pos under/overflow");
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
        cpanic("consputc");
    }
    cgaputc(c.into());
}

const INPUT_BUF: usize = 128;
struct Input {
    buf: [u8; INPUT_BUF],
    r: usize, // Read index
    w: usize, // Write index
    e: usize, // Edit index
}
static mut input: Input = unsafe { transmute([0u8; size_of::<Input>()]) };

const fn C(x: u8) -> i32 {
    // Control-x
    (x - b'@') as i32
}

pub unsafe fn consoleintr(getc: unsafe fn()->i32)
{
    acquire(&mut cons.lock as *mut Spinlock);
    let mut doprocdump = 0;
    loop {
        let mut c = getc();
        if c < 0 {
            break;
        }
        if c == C(b'P') {
            // Process listing.
            // procdump() locks cons.lock indirectly; invoke later
            doprocdump = 1;
        } else if c == C(b'U') {
            // Kill line.
            while (input.e != input.w && input.buf[(input.e - 1) % INPUT_BUF] != b'\n') {
                input.e -= 1;
                consputc(BACKSPACE);
            }
        } else if c == C(b'H') || c == b'\x7f' 
        /*Backspace */ as i32
        {
            if (input.e != input.w) {
                input.e -= 1;
                consputc(BACKSPACE);
            }
        } else {
            if (c != 0 && input.e - input.r < INPUT_BUF) {
                c = if (c == b'\r' as i32) { b'\n' as i32 } else { c };
                input.buf[input.e % INPUT_BUF] = c as u8;
                input.e += 1;
                consputc(c as u16);
                if (c == b'\n'.into() || c == C(b'D') || input.e == input.r + INPUT_BUF) {
                    input.w = input.e;
                    wakeup(&mut input.r as *mut usize as *mut ());
                }
            }
        }
    }
    release(&mut cons.lock as *mut Spinlock);
    if (doprocdump != 0) {
        procdump(); // now call procdump() wo. cons.lock held
    }
}

pub unsafe fn consoleread(ip: *mut Inode, mut dst: *mut u8, mut n: i32) -> i32 {
  iunlock(ip);
  let mut target = n;
  acquire(&mut cons.lock as *mut Spinlock);
  while(n > 0){
    while(input.r == input.w){
      if( (*myproc()).killed){
        release(&mut cons.lock as *mut Spinlock);
        ilock(ip);
        return -1;
      }
      sleep(&mut input.r as *mut usize as *mut (), &mut cons.lock as *mut Spinlock);
    }
    let c = input.buf[input.r % INPUT_BUF];
    input.r += 1;
    if(c == C(b'D') as u8){  // EOF
      if(n < target){
        // Save ^D for next time, to make sure
        // caller gets a 0-byte result.
        input.r-=1;
      }
      break;
    }
    *dst = c;
    dst = dst.offset(1);
    n -= 1;
    if(c == b'\n') {
      break;
    }
  }
  release(&mut cons.lock as *mut Spinlock);
  ilock(ip);

  return target - n;
}

pub unsafe fn consolewrite(ip: *mut Inode, buf: *mut u8, n: i32) -> i32 {
  iunlock(ip);
  acquire(&mut cons.lock as *mut Spinlock);
  for i in 0..n {
    consputc((*(buf.offset(i as isize)) & 0xff) as u16);
  }
  release(&mut cons.lock as *mut Spinlock);
  ilock(ip);

  return n;
}

pub unsafe fn consoleinit() {
    initlock(&mut cons.lock as *mut Spinlock, "console");

    devsw[CONSOLE].write = Some(consolewrite);
    devsw[CONSOLE].read = Some(consoleread);
    cons.locking = 1;

    picenable(IRQ_KBD as i32);
    ioapicenable(IRQ_KBD, 0);
}
