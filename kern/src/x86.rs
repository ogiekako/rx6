pub unsafe fn inb(port: u16) -> u8 {
    let data: u8;
    asm!("inb %dx, %al" : "={ax}" (data) : "{dx}"(port) :: "volatile");
    return data;
}

pub unsafe fn insl(port: i32, mut addr: *mut (), mut cnt: i32) {
    asm!("cld; rep insl" :
         "={di}" (addr), "={ecx}" (cnt) :
         "{edx}" (port), "0" (addr), "1" (cnt) :
         "memory", "cc" : "volatile");
}

pub unsafe fn outb(port: u16, data: u8) {
    asm!("outb %al, %dx" :: "{dx}"(port), "{al}"(data) :: "volatile");
}

pub unsafe fn stosb(mut addr: *mut (), data: i32, mut cnt: i32) {
    asm!("cld; rep stosb" :
         "={di}" (addr), "={ecx}" (cnt) :
         "0" (addr), "1" (cnt), "{eax}" (data) :
         "memory", "cc": "volatile");
}

pub unsafe fn lcr3(val: u32) {
    asm!("mov $0, %cr3"::"r"(val):"memory":"volatile");
}
