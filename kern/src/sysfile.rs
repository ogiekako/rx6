use super::*;

//
// File-system system calls.
// Mostly argument checking, since we don't trust
// user code, and calls into file.c and fs.c.
//

// Fetch the nth word-sized system call argument as a file descriptor
// and return both the descriptor and the corresponding struct file.
pub unsafe extern "C" fn argfd(n: i32, pfd: *mut i32, pf: *mut *mut File) -> i32 {
    let mut fd = 0i32;

    if (argint(n, &mut fd as *mut i32) < 0) {
        return -1;
    }
    if (fd < 0 || fd >= NOFILE as i32) {
        return -1;
    }
    let f = (*myproc()).ofile[fd as usize];
    if f.is_null() {
        return -1;
    }
    if (!pfd.is_null()) {
        *pfd = fd;
    }
    if (!pf.is_null()) {
        *pf = f;
    }
    return 0;
}

// Allocate a file descriptor for the given file.
// Takes over file reference from caller on success.
unsafe extern "C" fn fdalloc(f: *mut File) -> i32 {
    let curproc = myproc();

    for fd in 0..NOFILE {
        if ((*curproc).ofile[fd].is_null()) {
            (*curproc).ofile[fd] = f;
            return fd as i32;
        }
    }
    return -1;
}

pub unsafe extern "C" fn sys_dup() -> i32 {
    let mut f: *mut File = null_mut();
    if (argfd(0, null_mut(), &mut f as *mut *mut File) < 0) {
        cprintf("sys_dup: fail (1)\n", &[]);
        return -1;
    }
    // cprintf("sys_dup: fdalloc start\n", &[]);
    let fd = fdalloc(f);
    // cprintf("sys_dup: fdalloc end\n", &[]);
    if (fd < 0) {
        cprintf("sys_dup: fail (2)\n", &[]);
        return -1;
    }
    // cprintf("sys_dup: filedup start\n", &[]);
    filedup(f);
    // cprintf("sys_dup: filedup end\n", &[]);
    return fd;
}

pub unsafe extern "C" fn sys_read() -> i32 {
    let mut f: *mut File = null_mut();
    let mut n: i32 = 0;
    let mut p: *mut u8 = null_mut();

    if (argfd(0, null_mut(), &mut f as *mut *mut File) < 0
        || argint(2, &mut n as *mut i32) < 0
        || argptr(1, &mut p as *mut *mut u8, n) < 0)
    {
        return -1;
    }
    // cprintf("sys_read: fileread start\n", &[]);
    let res = fileread(f, p, n);
    // cprintf("sys_read: fileread end  p = %s  res = %d\n", &[Arg::Strp(p), Arg::Int(res)]);
    res
}

pub unsafe extern "C" fn sys_write() -> i32 {
    let mut f: *mut File = null_mut();
    let mut n: i32 = 0;
    let mut p: *mut u8 = null_mut();

    if (argfd(0, null_mut(), &mut f as *mut *mut File) < 0
        || argint(2, &mut n as *mut i32) < 0
        || argptr(1, &mut p as *mut *mut u8, n) < 0)
    {
        return -1;
    }
    return filewrite(f, p, n);
}

pub unsafe extern "C" fn sys_close() -> i32 {
    let mut f: *mut File = null_mut();
    let mut fd: i32 = 0;

    if (argfd(0, &mut fd as *mut i32, &mut f as *mut *mut File) < 0) {
        return -1;
    }
    (*myproc()).ofile[fd as usize] = null_mut();
    fileclose(f);
    0
}

pub unsafe extern "C" fn sys_fstat() -> i32 {
    let mut f: *mut File = null_mut();
    let mut st: *mut Stat = null_mut();

    if (argfd(0, null_mut(), &mut f as *mut *mut File) < 0
        || argptr(
            1,
            &mut st as *mut *mut Stat as *mut *mut u8,
            size_of_val(&(*st)) as i32,
        ) < 0)
    {
        return -1;
    }
    return filestat(f, st);
}

// Create the path new as a link to the same inode as old.
pub unsafe extern "C" fn sys_link() -> i32 {
    let mut name = [0u8; DIRSIZ];
    let mut new: *mut u8 = null_mut();
    let mut old: *mut u8 = null_mut();

    if (argstr(0, &mut old as *mut *mut u8) < 0 || argstr(1, &mut new as *mut *mut u8) < 0) {
        return -1;
    }

    begin_op();
    let mut ip = namei(old);
    if (ip.is_null()) {
        end_op();
        return -1;
    }

    ilock(ip);
    if ((*ip).type_ == T_DIR as i16) {
        iunlockput(ip);
        end_op();
        return -1;
    }

    (*ip).nlink += 1;
    iupdate(ip);
    iunlock(ip);

    'bad: loop {
        let dp = nameiparent(new, name.as_mut_ptr());
        if (dp.is_null()) {
            break 'bad;
        }
        ilock(dp);
        if ((*dp).dev != (*ip).dev || dirlink(dp, name.as_mut_ptr(), (*ip).inum) < 0) {
            iunlockput(dp);
            break 'bad;
        }
        iunlockput(dp);
        iput(ip);

        end_op();

        return 0;
    }

    ilock(ip);
    (*ip).nlink -= 1;
    iupdate(ip);
    iunlockput(ip);
    end_op();
    return -1;
}

// Is the directory dp empty except for "." and ".." ?
pub unsafe extern "C" fn isdirempty(dp: *mut Inode) -> i32 {
    let mut de: Dirent = core::mem::zeroed();

    for off in ((2 * size_of_val(&de))..((*dp).size)).step_by(size_of_val(&de)) {
        if (readi(dp, &mut de as *mut Dirent as *mut u8, off, size_of_val(&de))
            != size_of_val(&de) as i32)
        {
            cpanic("isdirempty: readi");
        }
        if (de.inum != 0) {
            return 0;
        }
    }
    return 1;
}

pub unsafe extern "C" fn sys_unlink() -> i32 {
    let mut de: Dirent = core::mem::zeroed();
    let mut name = [0u8; DIRSIZ];
    let mut path: *mut u8 = null_mut();

    if (argstr(0, &mut path as *mut *mut u8) < 0) {
        return -1;
    }

    begin_op();
    let mut dp = nameiparent(path, name.as_mut_ptr());
    if (dp.is_null()) {
        end_op();
        return -1;
    }

    ilock(dp);

    'bad: loop {
        // Cannot unlink "." or "..".
        if (namecmp(name.as_ptr(), ".\0".as_ptr()) == 0 || namecmp(name.as_ptr(), "..\0".as_ptr()) == 0)
        {
            break 'bad;
        }

        let mut off = 0usize;
        let mut ip = dirlookup(dp, name.as_mut_ptr(), &mut off as *mut usize);
        if ip.is_null() {
            break 'bad;
        }
        ilock(ip);

        if ((*ip).nlink < 1) {
            cpanic("unlink: nlink < 1");
        }
        if ((*ip).type_ == T_DIR as i16 && isdirempty(ip) == 0) {
            iunlockput(ip);
            break 'bad;
        }

        memset(&mut de as *mut Dirent as *mut u8, 0, size_of_val(&de));
        if (writei(dp, &mut de as *mut Dirent as *mut u8, off, size_of_val(&de))
            != size_of_val(&de) as i32)
        {
            cpanic("unlink: writei");
        }
        if ((*ip).type_ == T_DIR as i16) {
            (*dp).nlink -= 1;
            iupdate(dp);
        }
        iunlockput(dp);

        (*ip).nlink -= 1;
        iupdate(ip);
        iunlockput(ip);

        end_op();

        return 0;
    }

    iunlockput(dp);
    end_op();
    return -1;
}

unsafe extern "C" fn create(path: *mut u8, type_: i16, major: i16, minor: i16) -> *mut Inode {
    let mut name = [0u8; DIRSIZ];

    let mut dp = nameiparent(path, name.as_mut_ptr());
    if dp.is_null() {
        return null_mut();
    }
    ilock(dp);

    let mut off = 0usize;
    let mut ip = dirlookup(dp, name.as_mut_ptr(), &mut off as *mut usize);
    if !ip.is_null() {
        iunlockput(dp);
        ilock(ip);
        if (type_ == T_FILE as i16 && (*ip).type_ == T_FILE as i16) {
            return ip;
        }
        iunlockput(ip);
        return null_mut();
    }

    let ip = ialloc((*dp).dev, type_);
    if ip.is_null() {
        cpanic("create: ialloc");
    }

    ilock(ip);
    (*ip).major = major;
    (*ip).minor = minor;
    (*ip).nlink = 1;
    iupdate(ip);

    if (type_ == T_DIR) {
        // Create . and .. entries.
        (*dp).nlink += 1; // for ".."
        iupdate(dp);
        // No ip->nlink++ for ".": avoid cyclic ref count.
        if (dirlink(ip, ".\0".as_ptr(), (*ip).inum) < 0 || dirlink(ip, "..\0".as_ptr(), (*dp).inum) < 0)
        {
            cpanic("create dots");
        }
    }

    if (dirlink(dp, name.as_mut_ptr(), (*ip).inum) < 0) {
        cpanic("create: dirlink");
    }

    iunlockput(dp);

    return ip;
}

pub unsafe extern "C" fn sys_open() -> i32 {
    // cprintf("sys_open start\n", &[]);
    let mut path: *mut u8 = null_mut();
    let mut omode = 0i32;

    if (argstr(0, &mut path as *mut *mut u8) < 0 || argint(1, &mut omode as *mut i32) < 0) {
        return -1;
    }

    begin_op();

    let ip: *mut Inode;
    if (omode & O_CREATE) != 0 {
        ip = create(path, T_FILE as i16, 0, 0);
        if (ip.is_null()) {
            end_op();
            return -1;
        }
    } else {
        // cprintf("sys_open: namei start\n", &[]);
        ip = namei(path);
        // cprintf("sys_open: namei end\n", &[]);
        if ip.is_null() {
            end_op();
            return -1;
        }
        ilock(ip);
        if ((*ip).type_ == T_DIR as i16 && omode != O_RDONLY) {
            iunlockput(ip);
            end_op();
            return -1;
        }
    }

    let f = filealloc();
    if f.is_null() {
        iunlockput(ip);
        end_op();
        return -1;
    }
    let fd = fdalloc(f);
    if fd < 0 {
        fileclose(f);
        iunlockput(ip);
        end_op();
        return -1;
    }
    iunlock(ip);
    end_op();

    (*f).type_ = FD_INODE;
    (*f).ip = ip;
    (*f).off = 0;
    (*f).readable = if (omode & O_WRONLY) == 0 { 1 } else { 0 };
    (*f).writable = if (omode & O_WRONLY) != 0 || (omode & O_RDWR) != 0 {
        1
    } else {
        0
    };
    return fd;
}

pub unsafe extern "C" fn sys_mkdir() -> i32 {
    let mut path: *mut u8 = null_mut();
    let mut ip: *mut Inode = null_mut();

    begin_op();
    if argstr(0, &mut path as *mut *mut u8) < 0 {
        end_op();
        return -1;
    }
    let ip = create(path, T_DIR, 0, 0);
    if (ip.is_null()) {
        end_op();
        return -1;
    }
    iunlockput(ip);
    end_op();
    return 0;
}

pub unsafe extern "C" fn sys_mknod() -> i32 {
    let mut ip: *mut Inode = null_mut();
    let mut path: *mut u8 = null_mut();
    let mut major = 0i32;
    let mut minor = 0i32;

    begin_op();
    if ((argstr(0, &mut path as *mut *mut u8)) < 0
        || argint(1, &mut major as *mut i32) < 0
        || argint(2, &mut minor as *mut i32) < 0)
    {
        end_op();
        cprintf("sys_mknod: fail (1)\n", &[]);
        return -1;
    }
    // cprintf("sys_mknod: create start\n", &[]);
    let ip = create(path, T_DEV, major as i16, minor as i16);
    // cprintf("sys_mknod: create end\n", &[]);
    if ip.is_null() {
        end_op();
        cprintf("sys_mknod: fail (2)\n", &[]);
        return -1;
    }
    iunlockput(ip);
    end_op();
    return 0;
}

pub unsafe extern "C" fn sys_chdir() -> i32 {
    let mut path: *mut u8 = null_mut();
    let curproc = myproc();

    begin_op();
    if argstr(0, &mut path as *mut *mut u8) < 0 {
        end_op();
        return -1;
    }
    let ip = namei(path);
    if ip.is_null() {
        end_op();
        return -1;
    }

    ilock(ip);
    if ((*ip).type_ != T_DIR) {
        iunlockput(ip);
        end_op();
        return -1;
    }
    iunlock(ip);
    iput((*curproc).cwd);
    end_op();
    (*curproc).cwd = ip;
    return 0;
}

pub unsafe extern "C" fn sys_exec() -> i32 {
    let mut path: *mut u8 = null_mut();
    let mut uargv = 0usize;
    let mut uarg = 0usize;

    if (argstr(0, &mut path as *mut *mut u8) < 0
        || argint(1, &mut uargv as *mut usize as *mut i32) < 0)
    {
        return -1;
    }
    let mut argv: [*mut u8; MAXARG] = core::mem::zeroed();
    let mut i = 0;
    loop {
        if (i >= argv.len()) {
            return -1;
        }
        if (fetchint(uargv + 4 * i, &mut uarg as *mut usize as *mut i32) < 0) {
            return -1;
        }
        if (uarg == 0) {
            argv[i] = null_mut();
            break;
        }
        if (fetchstr(uarg, &mut argv[i] as *mut *mut u8) < 0) {
            return -1;
        }
        i += 1;
    }
    return exec(path, argv.as_mut_ptr());
}

pub unsafe extern "C" fn sys_pipe() -> i32 {
    let mut fd: *mut i32 = null_mut();
    let mut rf: *mut File = null_mut();
    let mut wf: *mut File = null_mut();

    if (argptr(
        0,
        &mut fd as *mut *mut i32 as *mut *mut u8,
        2 * size_of_val(&(*fd)) as i32,
    ) < 0)
    {
        return -1;
    }
    if (pipealloc(&mut rf as *mut *mut File, &mut wf as *mut *mut File) < 0) {
        return -1;
    }
    let fd0 = fdalloc(rf);
    if fd0 < 0 {
        fileclose(rf);
        fileclose(wf);
        return -1;
    }
    let fd1 = fdalloc(wf);
    if fd1 < 0 {
        (*myproc()).ofile[fd0 as usize] = null_mut();
        fileclose(rf);
        fileclose(wf);
        return -1;
    }
    *fd = fd0;
    *(fd.add(1)) = fd1;
    return 0;
}
