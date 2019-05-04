pub const ELF_MAGIC: usize = 0x464C457F; // "\x7FELF" in little endian

#[repr(C)]
pub struct Elfhdr {
    pub magic: usize,
    pub elf: [u8; 12],
    pub type_: u16,
    pub machine: u16,
    pub version: usize,
    pub entry: usize,
    pub phoff: usize, // program header offset
    pub shoff: usize,
    pub flags: usize,
    pub ehsize: u16,
    pub phentsize: u16,
    pub phnum: u16,
    pub shntsize: u16,
    pub shnum: u16,
    pub shstrndx: u16,
}

#[repr(C)]
pub struct Proghdr {
    pub type_: usize,
    pub off: usize,
    pub vaddr: usize,
    pub paddr: usize,
    pub filesz: usize,
    pub memsz: usize,
    pub flags: usize,
    pub align: usize,
}

// Values for Proghdr type
pub const ELF_PROG_LOAD: usize = 1;
