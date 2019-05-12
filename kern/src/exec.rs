use super::*;

pub unsafe extern "C" fn exec(path: *mut u8, argv: *mut *mut u8) -> i32 {
    cprintf("exec start.\n", &[]);

    check_it("exec (1)");

    let mut ustack = [0usize; 3 + MAXARG + 1];
    let mut elf = core::mem::zeroed::<Elfhdr>();
    let mut ph = core::mem::zeroed::<Proghdr>();

    let curproc = myproc();

    begin_op();

    check_it("exec (2)");

    cprintf("exec:  namei start\n", &[]);
    let mut ip = namei(path);
    if (ip.is_null()) {
        end_op();
        cprintf("exec: fail\n", &[]);
        return -1;
    }
    cprintf("exec:  namei end\n", &[]);

    cprintf("exec:  ilock start\n", &[]);
    ilock(ip);
    cprintf("exec:  ilock end\n", &[]);
    let mut pgdir = null_mut();

    'bad: loop {
        cprintf("exec:  readi start\n", &[]);
        // Check ELF header
        if (readi(ip, &mut elf as *mut Elfhdr as *mut u8, 0, size_of_val(&elf))
            != size_of_val(&elf) as i32)
        {
            break 'bad;
        }
        cprintf("exec:  readi end\n", &[]);
        if (elf.magic != ELF_MAGIC) {
            break 'bad;
        }

        cprintf("exec:  setupkvm start\n", &[]);
        let pgdir2 = setupkvm();
        cprintf("exec:  setupkvm end\n", &[]);
        if pgdir2.is_none() {
            break 'bad;
        }
        pgdir = pgdir2.unwrap().pd.0 as *mut pte_t;

        // Load program into memory.
        let mut sz = 0;
        let mut i = 0;
        let mut off = elf.phoff;
        while i < elf.phnum {
            cprintf("exec: readi(2) start\n", &[]);
            if (readi(
                ip,
                &mut ph as *mut Proghdr as *mut u8,
                off,
                size_of_val(&ph),
            ) != size_of_val(&ph) as i32)
            {
                break 'bad;
            }
            cprintf("exec: readi(2) end\n", &[]);
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
            cprintf("exec: allocuvm start\n", &[]);
            sz = allocuvm(pgdir, sz, ph.vaddr + ph.memsz);
            if sz == 0 {
                break 'bad;
            }
            cprintf("exec: allocuvm end\n", &[]);
            if (ph.vaddr % PGSIZE != 0) {
                break 'bad;
            }
            cprintf("exec: loaduvm start\n", &[]);
            if (loaduvm(pgdir, ph.vaddr as *mut u8, ip, ph.off, ph.filesz) < 0) {
                break 'bad;
            }
            cprintf("exec: loaduvm end\n", &[]);
            i += 1;
            off += size_of_val(&ph);
        }
        cprintf("exec: iunlockput start\n", &[]);
        iunlockput(ip);
        cprintf("exec: iunlockput end\n", &[]);
        end_op();
        ip = null_mut();

        // Allocate two pages at the next page boundary.
        // Make the first inaccessible.  Use the second as the user stack.
        sz = PGROUNDUP(sz);
        cprintf("exec: allocuvm start\n", &[]);
        sz = allocuvm(pgdir, sz, sz + 2 * PGSIZE);
        cprintf("exec: allocuvm end\n", &[]);
        if sz == 0 {
            break 'bad;
        }
        cprintf("exec: clearpteu start\n", &[]);
        clearpteu(pgdir, (sz - 2 * PGSIZE) as *mut u8);
        cprintf("exec: clearpteu end\n", &[]);
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
        cprintf("exec: copyout start\n", &[]);
        if (copyout(
            pgdir,
            sp,
            ustack.as_mut_ptr() as *mut (),
            (3 + argc + 1) * 4,
        ) < 0)
        {
            break 'bad;
        }
        cprintf("exec: copyout end\n", &[]);

        let mut last = path;
        let mut s = path;
        // Save program name for debugging.
        while *s != 0 {
            if (*s == b'/') {
                last = s.offset(1);
            }
            s = s.add(1);
        }
        cprintf("exec: safestrcpy start\n", &[]);
        safestrcpy(
            (*curproc).name.as_mut_ptr(),
            last,
            size_of_val(&(*curproc).name) as i32,
        );
        cprintf("exec: safestrcpy end\n", &[]);

        // Commit to the user image.
        let oldpgdir = (*curproc).pgdir;
        (*curproc).pgdir = pgdir;
        (*curproc).sz = sz;
        (*(*curproc).tf).eip = elf.entry; // main
        (*(*curproc).tf).esp = sp;
        cprintf("exec: switchuvm start\n", &[]);
        enable_check = false;
        switchuvm(curproc);
        cprintf("exec: switchuvm end\n", &[]);
        cprintf("exec: freevm start\n", &[]);
        freevm(oldpgdir);
        cprintf("exec: freevm end\n", &[]);
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
