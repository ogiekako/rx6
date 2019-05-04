use super::*;
// kbd.h

// PC keyboard interface constants

const KBSTATP: u8 = 0x64; // kbd controller status port(I)
const KBS_DIB: u8 = 0x01; // kbd data in buffer
const KBDATAP: u8 = 0x60; // kbd data port(I)

const NO: u8 = 0;

const SHIFT: u8 = (1 << 0);
const CTL: u8 = (1 << 1);
const ALT: u8 = (1 << 2);

const CAPSLOCK: u8 = (1 << 3);
const NUMLOCK: u8 = (1 << 4);
const SCROLLLOCK: u8 = (1 << 5);

const E0ESC: u8 = (1 << 6);

// Special keycodes
const KEY_HOME: u8 = 0xE0;
const KEY_END: u8 = 0xE1;
const KEY_UP: u8 = 0xE2;
const KEY_DN: u8 = 0xE3;
const KEY_LF: u8 = 0xE4;
const KEY_RT: u8 = 0xE5;
const KEY_PGUP: u8 = 0xE6;
const KEY_PGDN: u8 = 0xE7;
const KEY_INS: u8 = 0xE8;
const KEY_DEL: u8 = 0xE9;

// C('A') == Control-A
const fn C(x: u8) -> u8 {
    x.wrapping_sub(b'@')
}

fn shiftcode(x: u8) -> u8 {
    match x {
        0x1D => CTL,
        0x2A => SHIFT,
        0x36 => SHIFT,
        0x38 => ALT,
        0x9D => CTL,
        0xB8 => ALT,
        _ => 0,
    }
}

fn togglecode(x: u8) -> u8 {
    match x {
        0x3A => CAPSLOCK,
        0x45 => NUMLOCK,
        0x46 => SCROLLLOCK,
        _ => 0,
    }
}

#[rustfmt::skip]
const normalmap: [u8; 256] = [
  NO,   0x1B, b'1',  b'2',  b'3',  b'4',  b'5',  b'6',  // 0x00
  b'7',  b'8',  b'9',  b'0',  b'-',  b'=',  8 /* bs */, b'\t',
  b'q',  b'w',  b'e',  b'r',  b't',  b'y',  b'u',  b'i',  // 0x10
  b'o',  b'p',  b'[',  b']',  b'\n', NO,   b'a',  b's',
  b'd',  b'f',  b'g',  b'h',  b'j',  b'k',  b'l',  b';',  // 0x20
  b'\'', b'`',  NO,   b'\\', b'z',  b'x',  b'c',  b'v',
  b'b',  b'n',  b'm',  b',',  b'.',  b'/',  NO,   b'*',  // 0x30
  NO,   b' ',  NO,   NO,   NO,   NO,   NO,   NO,
  NO,   NO,   NO,   NO,   NO,   NO,   NO,   b'7',  // 0x40
  b'8',  b'9',  b'-',  b'4',  b'5',  b'6',  b'+',  b'1',
  b'2',  b'3',  b'0',  b'.',  NO,   NO,   NO,   NO,   // 0x50
  NO, NO, NO, NO, NO, NO, NO, NO,
  NO, NO, NO, NO, NO, NO, NO, NO, // 0x60
  NO, NO, NO, NO, NO, NO, NO, NO,
  NO, NO, NO, NO, NO, NO, NO, NO, // 0x70
  NO, NO, NO, NO, NO, NO, NO, NO,
  NO, NO, NO, NO, NO, NO, NO, NO, // 0x80
  NO, NO, NO, NO, NO, NO, NO, NO,
  NO, NO, NO, NO, NO, NO, NO, KEY_HOME, // 0x90
  NO, NO, NO, NO, b'\n' /* KP_Enter */, NO, NO, NO,
  NO, NO, NO, NO, NO, NO, NO, NO, // 0xa0
  NO, NO, NO, NO, NO, NO, NO, NO,
  NO, NO, NO, NO, NO, b'/' /* KP_Div */ , NO, NO, // 0xb0
  NO, NO, NO, NO, NO, NO, NO, NO,
  NO, NO, NO, NO, NO, NO, NO, NO, // 0xc0
  KEY_UP, KEY_PGUP, NO, KEY_LF, NO, KEY_RT, NO, KEY_END,
  KEY_DN, KEY_PGDN, KEY_INS, KEY_DEL, NO, NO, NO, NO, // 0xd0
  NO, NO, NO, NO, NO, NO, NO, NO,
  NO, NO, NO, NO, NO, NO, NO, NO, // 0xe0
  NO, NO, NO, NO, NO, NO, NO, NO,
  NO, NO, NO, NO, NO, NO, NO, NO, // 0xf0
  NO, NO, NO, NO, NO, NO, NO, NO,
];

#[rustfmt::skip]
const shiftmap: [u8; 256] = [
  NO,   0o33 /* esc */,  b'!',  b'@',  b'#',  b'$',  b'%',  b'^',  // 0x00
  b'&',  b'*',  b'(',  b')',  b'_',  b'+',  8 /* Backspace */, b'\t',
  b'Q',  b'W',  b'E',  b'R',  b'T',  b'Y',  b'U',  b'I',  // 0x10
  b'O',  b'P',  b'{',  b'}',  b'\n', NO,   b'A',  b'S',
  b'D',  b'F',  b'G',  b'H',  b'J',  b'K',  b'L',  b':',  // 0x20
  b'"',  b'~',  NO,   b'|',  b'Z',  b'X',  b'C',  b'V',
  b'B',  b'N',  b'M',  b'<',  b'>',  b'?',  NO,   b'*',  // 0x30
  NO,   b' ',  NO,   NO,   NO,   NO,   NO,   NO,
  NO,   NO,   NO,   NO,   NO,   NO,   NO,   b'7',  // 0x40
  b'8',  b'9',  b'-',  b'4',  b'5',  b'6',  b'+',  b'1',
  b'2',  b'3',  b'0',  b'.',  NO,   NO,   NO,   NO,   // 0x50
  NO, NO, NO, NO, NO, NO, NO, NO,
  NO, NO, NO, NO, NO, NO, NO, NO, // 0x60
  NO, NO, NO, NO, NO, NO, NO, NO,
  NO, NO, NO, NO, NO, NO, NO, NO, // 0x70
  NO, NO, NO, NO, NO, NO, NO, NO,
  NO, NO, NO, NO, NO, NO, NO, NO, // 0x80
  NO, NO, NO, NO, NO, NO, NO, NO,
  NO, NO, NO, NO, NO, NO, NO, KEY_HOME, // 0x90
  NO, NO, NO, NO, b'\n' /* KP_Enter */, NO, NO, NO,
  NO, NO, NO, NO, NO, NO, NO, NO, // 0xa0
  NO, NO, NO, NO, NO, NO, NO, NO,
  NO, NO, NO, NO, NO, b'/' /* KP_Div */, NO, NO, // 0xb0
  NO, NO, NO, NO, NO, NO, NO, NO,
  NO, NO, NO, NO, NO, NO, NO, NO, // 0xc0
  KEY_UP, KEY_PGUP, NO, KEY_LF, NO, KEY_RT, NO, KEY_END,
  KEY_DN, KEY_PGDN, KEY_INS, KEY_DEL, NO, NO, NO, NO, // 0xd0
  NO, NO, NO, NO, NO, NO, NO, NO,
  NO, NO, NO, NO, NO, NO, NO, NO, // 0xe0
  NO, NO, NO, NO, NO, NO, NO, NO,
  NO, NO, NO, NO, NO, NO, NO, NO, // 0xf0
  NO, NO, NO, NO, NO, NO, NO, NO,
];

#[rustfmt::skip]
const ctlmap: [u8; 256] = [
    NO, NO, NO, NO, NO, NO, NO, NO, // 0x00
    NO, NO, NO, NO, NO, NO, NO, NO,
    C(b'Q'), C(b'W'), C(b'E'), C(b'R'), C(b'T'), C(b'Y'), C(b'U'), C(b'I'), // 0x10
    C(b'O'), C(b'P'), NO, NO, b'\r', NO, C(b'A'), C(b'S'),
    C(b'D'), C(b'F'), C(b'G'), C(b'H'), C(b'J'), C(b'K'), C(b'L'), NO, // 0x20
    NO, NO, NO, C(b'\\'), C(b'Z'), C(b'X'), C(b'C'), C(b'V'),
    C(b'B'), C(b'N'), C(b'M'), NO, NO, C(b'/'), NO, NO, // 0x30
    NO, NO, NO, NO, NO, NO, NO, NO,
    NO, NO, NO, NO, NO, NO, NO, NO, // 0x40
    NO, NO, NO, NO, NO, NO, NO, NO,
    NO, NO, NO, NO, NO, NO, NO, NO, // 0x50
    NO, NO, NO, NO, NO, NO, NO, NO,
    NO, NO, NO, NO, NO, NO, NO, NO, // 0x60
    NO, NO, NO, NO, NO, NO, NO, NO,
    NO, NO, NO, NO, NO, NO, NO, NO, // 0x70
    NO, NO, NO, NO, NO, NO, NO, NO,
    NO, NO, NO, NO, NO, NO, NO, NO, // 0x80
    NO, NO, NO, NO, NO, NO, NO, NO,
    NO, NO, NO, NO, NO, NO, NO, KEY_HOME, // 0x90
    NO, NO, NO, NO, b'\r' /* KP_Enter */, NO, NO, NO,
    NO, NO, NO, NO, NO, NO, NO, NO, // 0xa0
    NO, NO, NO, NO, NO, NO, NO, NO,
    NO, NO, NO, NO, NO, C(b'/') /* KP_Div */, NO, NO, // 0xb0
    NO, NO, NO, NO, NO, NO, NO, NO,
    NO, NO, NO, NO, NO, NO, NO, NO, // 0xc0
    KEY_UP, KEY_PGUP, NO, KEY_LF, NO, KEY_RT, NO, KEY_END,
    KEY_DN, KEY_PGDN, KEY_INS, KEY_DEL, NO, NO, NO, NO, // 0xd0
    NO, NO, NO, NO, NO, NO, NO, NO,
    NO, NO, NO, NO, NO, NO, NO, NO, // 0xe0
    NO, NO, NO, NO, NO, NO, NO, NO,
    NO, NO, NO, NO, NO, NO, NO, NO, // 0xf0
    NO, NO, NO, NO, NO, NO, NO, NO,
];

// kbd.c

static mut shift: u8 = 0;
const charcode: [&[u8; 256]; 4] = [&normalmap, &shiftmap, &ctlmap, &ctlmap];
pub unsafe fn kbdgetc() -> i32 {
    let st = inb(KBSTATP as u16);
    if ((st & KBS_DIB) == 0) {
        return -1;
    }
    let mut data = inb(KBDATAP as u16);

    if (data == 0xE0) {
        shift |= E0ESC;
        return 0;
    } else if ((data & 0x80) != 0) {
        // Key released
        data = if (shift & E0ESC) != 0 {
            data
        } else {
            data & 0x7F
        };
        shift &= !(shiftcode(data) | E0ESC);
        return 0;
    } else if (shift & E0ESC) != 0 {
        // Last character was an E0 escape; or with 0x80
        data |= 0x80;
        shift &= !E0ESC;
    }

    shift |= shiftcode(data);
    shift ^= togglecode(data);
    let mut c = charcode[(shift & (CTL | SHIFT)) as usize][data as usize];
    if (shift & CAPSLOCK) != 0 {
        if (b'a' <= c && c <= b'z') {
            c -= b'a' - b'A';
        } else if (b'A' <= c && c <= b'Z') {
            c += b'a' - b'A';
        }
    }
    return c as i32;
}

pub unsafe fn kbdintr() {
    consoleintr(kbdgetc);
}
