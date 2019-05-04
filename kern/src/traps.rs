// x86 trap and interrupt pub constants.

// Processor-defined:
pub const T_DIVIDE: usize = 0; // divide error
pub const T_DEBUG: usize = 1; // debug exception
pub const T_NMI: usize = 2; // non-maskable interrupt
pub const T_BRKPT: usize = 3; // breakpoint
pub const T_OFLOW: usize = 4; // overflow
pub const T_BOUND: usize = 5; // bounds check
pub const T_ILLOP: usize = 6; // illegal opcode
pub const T_DEVICE: usize = 7; // device not available
pub const T_DBLFLT: usize = 8; // double fault
                               // #define T_COPROC      9      // reserved (not used since 486)
pub const T_TSS: usize = 10; // invalid task switch segment
pub const T_SEGNP: usize = 11; // segment not present
pub const T_STACK: usize = 12; // stack exception
pub const T_GPFLT: usize = 13; // general protection fault
pub const T_PGFLT: usize = 14; // page fault
                               // #define T_RES        15      // reserved
pub const T_FPERR: usize = 16; // floating point error
pub const T_ALIGN: usize = 17; // aligment check
pub const T_MCHK: usize = 18; // machine check
pub const T_SIMDERR: usize = 19; // SIMD floating point error

// These are arbitrarily chosen, but with care not to overlap
// processor defined exceptions or interrupt vectors.
pub const T_SYSCALL: usize = 64; // system call
pub const T_DEFAULT: usize = 500; // catchall

pub const T_IRQ0: usize = 32; // IRQ 0 corresponds to int T_IRQ

pub const IRQ_TIMER: usize = 0;
pub const IRQ_KBD: usize = 1;
pub const IRQ_COM1: usize = 4;
pub const IRQ_IDE: usize = 14;
pub const IRQ_ERROR: usize = 19;
pub const IRQ_SPURIOUS: usize = 31;
