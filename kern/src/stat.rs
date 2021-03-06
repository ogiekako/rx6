pub const T_DIR: i16 = 1; // Directory
pub const T_FILE: i16 = 2; // File
pub const T_DEV: i16 = 3; // Device

pub struct Stat {
    pub type_: i16,  // Type of file
    pub dev: i32,    // File system's disk device
    pub ino: usize,  // Inode number
    pub nlink: i16,  // Number of links to file
    pub size: usize, // Size of file in bytes
}
