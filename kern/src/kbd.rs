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

//// static uchar shiftcode[256] =
//// {
////   [0x1D] CTL,
////   [0x2A] SHIFT,
////   [0x36] SHIFT,
////   [0x38] ALT,
////   [0x9D] CTL,
////   [0xB8] ALT
//// };
////
//// static uchar togglecode[256] =
//// {
////   [0x3A] CAPSLOCK,
////   [0x45] NUMLOCK,
////   [0x46] SCROLLLOCK
//// };
////
//// static uchar normalmap[256] =
//// {
////   NO,   0x1B, '1',  '2',  '3',  '4',  '5',  '6',  // 0x00
////   '7',  '8',  '9',  '0',  '-',  '=',  '\b', '\t',
////   'q',  'w',  'e',  'r',  't',  'y',  'u',  'i',  // 0x10
////   'o',  'p',  '[',  ']',  '\n', NO,   'a',  's',
////   'd',  'f',  'g',  'h',  'j',  'k',  'l',  ';',  // 0x20
////   '\'', '`',  NO,   '\\', 'z',  'x',  'c',  'v',
////   'b',  'n',  'm',  ',',  '.',  '/',  NO,   '*',  // 0x30
////   NO,   ' ',  NO,   NO,   NO,   NO,   NO,   NO,
////   NO,   NO,   NO,   NO,   NO,   NO,   NO,   '7',  // 0x40
////   '8',  '9',  '-',  '4',  '5',  '6',  '+',  '1',
////   '2',  '3',  '0',  '.',  NO,   NO,   NO,   NO,   // 0x50
////   [0x9C] '\n',      // KP_Enter
////   [0xB5] '/',       // KP_Div
////   [0xC8] KEY_UP,    [0xD0] KEY_DN,
////   [0xC9] KEY_PGUP,  [0xD1] KEY_PGDN,
////   [0xCB] KEY_LF,    [0xCD] KEY_RT,
////   [0x97] KEY_HOME,  [0xCF] KEY_END,
////   [0xD2] KEY_INS,   [0xD3] KEY_DEL
//// };
////
//// static uchar shiftmap[256] =
//// {
////   NO,   033,  '!',  '@',  '#',  '$',  '%',  '^',  // 0x00
////   '&',  '*',  '(',  ')',  '_',  '+',  '\b', '\t',
////   'Q',  'W',  'E',  'R',  'T',  'Y',  'U',  'I',  // 0x10
////   'O',  'P',  '{',  '}',  '\n', NO,   'A',  'S',
////   'D',  'F',  'G',  'H',  'J',  'K',  'L',  ':',  // 0x20
////   '"',  '~',  NO,   '|',  'Z',  'X',  'C',  'V',
////   'B',  'N',  'M',  '<',  '>',  '?',  NO,   '*',  // 0x30
////   NO,   ' ',  NO,   NO,   NO,   NO,   NO,   NO,
////   NO,   NO,   NO,   NO,   NO,   NO,   NO,   '7',  // 0x40
////   '8',  '9',  '-',  '4',  '5',  '6',  '+',  '1',
////   '2',  '3',  '0',  '.',  NO,   NO,   NO,   NO,   // 0x50
////   [0x9C] '\n',      // KP_Enter
////   [0xB5] '/',       // KP_Div
////   [0xC8] KEY_UP,    [0xD0] KEY_DN,
////   [0xC9] KEY_PGUP,  [0xD1] KEY_PGDN,
////   [0xCB] KEY_LF,    [0xCD] KEY_RT,
////   [0x97] KEY_HOME,  [0xCF] KEY_END,
////   [0xD2] KEY_INS,   [0xD3] KEY_DEL
//// };

#[rustfmt::skip]
static ctlmap: [u8; 256] = [
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
//
//// #include "types.h"
//// #include "x86.h"
//// #include "defs.h"
//// #include "kbd.h"
////
//// int
//// kbdgetc(void)
//// {
////   static uint shift;
////   static uchar *charcode[4] = {
////     normalmap, shiftmap, ctlmap, ctlmap
////   };
////   uint st, data, c;
////
////   st = inb(KBSTATP);
////   if((st & KBS_DIB) == 0)
////     return -1;
////   data = inb(KBDATAP);
////
////   if(data == 0xE0){
////     shift |= E0ESC;
////     return 0;
////   } else if(data & 0x80){
////     // Key released
////     data = (shift & E0ESC ? data : data & 0x7F);
////     shift &= ~(shiftcode[data] | E0ESC);
////     return 0;
////   } else if(shift & E0ESC){
////     // Last character was an E0 escape; or with 0x80
////     data |= 0x80;
////     shift &= ~E0ESC;
////   }
////
////   shift |= shiftcode[data];
////   shift ^= togglecode[data];
////   c = charcode[shift & (CTL | SHIFT)][data];
////   if(shift & CAPSLOCK){
////     if('a' <= c && c <= 'z')
////       c += 'A' - 'a';
////     else if('A' <= c && c <= 'Z')
////       c += 'a' - 'A';
////   }
////   return c;
//// }
////
//// void
//// kbdintr(void)
//// {
////   consoleintr(kbdgetc);
//// }
