use super::*;

pub unsafe extern "C" fn exec(path: *mut u8, argv: *mut *mut u8) -> i32 {
    cprintf("exec start.\n", &[]);
    loop {}
    check_it("exec (1)");

    let mut ustack = [0usize; 3 + MAXARG + 1];
    let mut elf = core::mem::zeroed::<Elfhdr>();
    let mut ph = core::mem::zeroed::<Proghdr>();

    let curproc = myproc();

    begin_op();

    check_it("exec (2)");

    let mut ip = namei(path);
    if (ip.is_null()) {
        end_op();
        cprintf("exec: fail\n", &[]);
        return -1;
    }
    ilock(ip);
    let mut pgdir = null_mut();

    'bad: loop {
        // Check ELF header
        if (readi(ip, &mut elf as *mut Elfhdr as *mut u8, 0, size_of_val(&elf))
            != size_of_val(&elf) as i32)
        {
            break 'bad;
        }
        if (elf.magic != ELF_MAGIC) {
            break 'bad;
        }

        let pgdir2 = setupkvm();
        if pgdir2.is_none() {
            break 'bad;
        }
        pgdir = pgdir2.unwrap().pd.0 as *mut pte_t;

        // Load program into memory.
        let mut sz = 0;
        let mut i = 0;
        let mut off = elf.phoff;
        while i < elf.phnum {
            if (readi(
                ip,
                &mut ph as *mut Proghdr as *mut u8,
                off,
                size_of_val(&ph),
            ) != size_of_val(&ph) as i32)
            {
                break 'bad;
            }
            if (ph.type_ != ELF_PROG_LOAD) {
                i += 1;
                off += size_of_val(&ph);
                continue;
            }
            if (ph.memsz < ph.filesz) {
                break 'bad;
            }
            if (ph.vaddr + ph.memsz < ph.vaddr) {
                break 'bad;
            }
            sz = allocuvm(pgdir, sz, ph.vaddr + ph.memsz);
            if sz == 0 {
                break 'bad;
            }
            if (ph.vaddr % PGSIZE != 0) {
                break 'bad;
            }
            if (loaduvm(pgdir, ph.vaddr as *mut u8, ip, ph.off, ph.filesz) < 0) {
                break 'bad;
            }
            i += 1;
            off += size_of_val(&ph);
        }
        iunlockput(ip);
        end_op();
        ip = null_mut();

        // Allocate two pages at the next page boundary.
        // Make the first inaccessible.  Use the second as the user stack.
        sz = PGROUNDUP(sz);
        sz = allocuvm(pgdir, sz, sz + 2 * PGSIZE);
        if sz == 0 {
            break 'bad;
        }
        clearpteu(pgdir, (sz - 2 * PGSIZE) as *mut u8);
        let mut sp = sz;

        // Push argument strings, prepare rest of stack in ustack.
        let mut argc = 0usize;
        while !(*argv.add(argc)).is_null() {
            if (argc >= MAXARG) {
                break 'bad;
            }
            sp = (sp - (strlen(*(argv.add(argc))) + 1) as usize) & !3;
            if (copyout(
                pgdir,
                sp,
                *(argv.add(argc)) as *mut (),
                strlen(*(argv.add(argc))) as usize + 1,
            ) < 0)
            {
                break 'bad;
            }
            ustack[3 + argc] = sp;

            argc += 1;
        }
        ustack[3 + argc] = 0;

        ustack[0] = 0xffffffff; // fake return PC
        ustack[1] = argc;
        ustack[2] = sp - (argc + 1) * 4; // argv pointer

        sp -= ((3 + argc + 1) * 4);
        if (copyout(
            pgdir,
            sp,
            ustack.as_mut_ptr() as *mut (),
            (3 + argc + 1) * 4,
        ) < 0)
        {
            break 'bad;
        }

        let mut last = path;
        let mut s = path;
        // Save program name for debugging.
        while *s != 0 {
            if (*s == b'/') {
                last = s.offset(1);
            }
            s = s.add(1);
        }
        safestrcpy(
            (*curproc).name.as_mut_ptr(),
            last,
            size_of_val(&(*curproc).name) as i32,
        );

        // Commit to the user image.
        let oldpgdir = (*curproc).pgdir;
        (*curproc).pgdir = pgdir;
        (*curproc).sz = sz;
        (*(*curproc).tf).eip = elf.entry; // main
        (*(*curproc).tf).esp = sp;
        switchuvm(curproc);
        freevm(oldpgdir);
        return 0;
    }

    if (!pgdir.is_null()) {
        freevm(pgdir);
    }
    if (!ip.is_null()) {
        iunlockput(ip);
        end_op();
    }
    return -1;
}
