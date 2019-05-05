# 2019-05-05 15:58

```
qemu-system-i386 -nographic -drive file=fs.img,index=1,media=disk,format=raw -drive file=rx6.img,index=0,media=disk,format=raw -smp 2 -m 512 -S -gdb tcp::26001
 xv6...
  done: uartinit   Starting cpu 1  stack: 0x803be000.  cpu1: starting 1
  cpu0: starting 0
  sb: size 1000 nblocks 941 ninodes 200 nlog 30 logstart 2inodestart 32 bmap start 58
  unexpected trap 14 from cpu 1 eip 53e58955 (cr2=0x53e58955)
```

page fault が発生している (14 = page fault 例外). cpu 1 のほうか。

[cr2] レジスタとは？ page fault linear address. When a page fault occurs, the address the program attempted to access is stored in the CR2 register. つまりこのアドレスにアクセスしようとして、page fault が起きたということ。

page fault が起きた時点で止まるようにできないかな。

[cr2]: https://wiki.osdev.org/CPU_Registers_x86#CR2

IDT が、trap handler (%cs, %eip) を指定している。%cs は特権レベル、%eip は飛ぶ場所。
単純には、割り込みが発生すると、CPU は、いくつかの register を push してから、CPL を変更して eip に飛ぶ。

```
            (*c).process = p;
80124d9b:	8b 44 24 6c          	mov    0x6c(%esp),%eax
```

```
Thread 1 hit Breakpoint 1, kern::process::scheduler () at src/process.rs:452
452                 (*c).process = p;
gdb-peda$ p ptable.proc[0]
$1 = kern::process::Proc {
  sz: 0x1000,
  pgdir: 0x8dffe000,
  kstack: 0x8dfff000 "\000",
  state: kern::process::Procstate::RUNNABLE,
  pid: 0x1,
  parent: 0x0,
  tf: 0x8dffffb4,
  context: 0x8dffff9c,
  chan: 0x0,
  killed: 0x0,
  ofile: [0x0 <repeats 16 times>],
  cwd: 0x8016da10 <kern::fs::icache+56>,
  name: [0x69, 0x6e, 0x69, 0x74, 0x63, 0x6f, 0x64, 0x65, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0]
}
```

switchuvm で user process に移ったあとの、user process を見たい。

QEMU で OS をデバッグする best practice がどこかにまとまってそうだから、まずはそれを読もう。今日は十分デバッグに時間を費やしたので、あとは input の時間とする。


なんか、thread 2 が alltraps に飛んでいる。

```
gdb-peda$ thr ap 2 bt

Thread 2 (Thread 2):
#0  <T as core::convert::TryFrom<U>>::try_from (value=0x1) at /home/oka/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/src/libcore/convert.rs:568
#1  0x80139593 in <usize as core::iter::range::Step>::add_usize (self=0x803be960, n=0x1) at /home/oka/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/src/libcore/iter/range.rs:82
#2  0x803be918 in ?? ()
#3  0x80140654 in kern::lapic::lapiccpunum () at src/lapic.rs:106
#4  0x8012351d in kern::process::mycpu () at src/process.rs:143
#5  0x8011f99b in kern::spinlock::popcli () at src/spinlock.rs:127
#6  0x8011f53c in kern::spinlock::release (lk=0x8016b26c <kern::process::ptable>) at src/spinlock.rs:78
#7  0x80125510 in kern::process::wakeup (chan=0x8016053c <kern::bio::bcache+56>) at src/process.rs:580
#8  0x8012aa5d in kern::ide::ideintr () at src/ide.rs:123
#9  0x8012ba98 in trap (tf=0x803beddc) at src/trap.rs:50
#10 0x80146a93 in alltraps () at trapasm.S:23
#11 0x803beddc in ?? ()
#12 0x8011f53c in kern::spinlock::release (lk=0x8016b26c <kern::process::ptable>) at src/spinlock.rs:78
#13 0x80124d55 in kern::process::scheduler () at src/process.rs:463
#14 0x8011ebef in kern::kernmain::mpmain () at src/kernmain.rs:49
#15 0x8011eab6 in kern::kernmain::mpenter () at src/kernmain.rs:38
#16 0x0000705a in ?? ()
```

```
gdb-peda$ print ('kern::x86::Trapframe')*0x803beddc
$8 = {
  edi = 0x803beea8,
  esi = 0x803bee68,
  ebp = 0x803bee60,
  oesp = 0x803bedfc,
  ebx = 0x8016ac3c,
  edx = 0x1,
  ecx = 0x8016ad01,
  eax = 0x8016ad90,
  gs = 0x0,
  padding1 = 0x0,
  fs = 0x0,
  padding2 = 0x0,
  es = 0x10,
  padding3 = 0x0,
  ds = 0x10,
  padding4 = 0x0,
  trapno = 0x2e,
  err = 0x0,
  eip = 0x8011fa2e,
  cs = 0x8,
  padding5 = 0x0,
  eflags = 0x202,
  esp = 0x803bee68,
  ss = 0xac3c,
  padding6 = 0x8016
}
```

```
8011fa2e:	8d 65 f8             	lea    -0x8(%ebp),%esp
```

ここで page fault が発生した。

```
gdb-peda$ list *0x8011fa2e
0x8011fa2e is in kern::spinlock::popcli (src/spinlock.rs:134).
129             }
130             if ((*mycpu()).ncli == 0 && (*mycpu()).intena > 0) {
131                 sti();
132             }
133         }
134     }
```

swtch 直後に、spinlock の部分に飛んでいる。
やはり、swtch して、ちゃんと適切な context になっていないのかな。
initcode に飛べているのかどうかが気になる。

```
gdb-peda$ n
=> 0x80124dc6 <kern::process::scheduler+438>:   mov    eax,DWORD PTR [esp+0x3c]
456                 swtch(&mut ((*c).scheduler) as *mut *mut Context, (*p).context);
gdb-peda$ n
=> 0x8011fa2e <kern::spinlock::popcli+302>:     lea    esp,[ebp-0x8]

Thread 2 hit Breakpoint 3, kern::spinlock::popcli () at src/spinlock.rs:134
134     }
```

```
8016ab58 D _binary_initcode_end
0000002c A _binary_initcode_size
8016ab2c D _binary_initcode_start
```

これを貼り付けているはず。


```
Thread 1 hit Breakpoint 1, kern::vm::inituvm (pgdir=0x8dffe000, init=0x8016ab2c "h$\000", sz=0x2c) at src/vm.rs:218
```

```
=> 0x80128c55 <kern::vm::inituvm+213>:  mov    ecx,DWORD PTR [esp+0x3c]
220         memset(mem, 0, PGSIZE);
gdb-peda$ p mem
$1 = (u8 *) 0x8dfbd000 "\000"
```

0x8dffe000 にある page table に、V(0) を phisical page 0x8dfbd000 - 0x80000000 に map すると書いてある。

thread 1 が initcode に飛び、V(0x5) にあるコードを動かしている。
しかしその時、thread 2 は、page fault で死につつある。

inituvm そのものには問題がなくて、scheduler の排他制御の問題？中途半端な状態のプロセス？


```
gdb-peda$
=> 0x8013635d <kern::syscall::syscall+637>:     mov    ebx,DWORD PTR [esp+0x38]
0x8013635d      132             (*(*curproc).tf).eax = (*(syscalls[num]))() as usize;
gdb-peda$
=> 0x80136361 <kern::syscall::syscall+641>:     call   ecx
0x80136361      132             (*(*curproc).tf).eax = (*(syscalls[num]))() as usize;
gdb-peda$
```

# 2019-05-05 15:44

```
xv6...
  done: uartinit   Starting cpu 1  stack: 0x803be000.  cpu1: starting 1
  cpu0: starting 0
  sb: size 1000 nblocks 941 ninodes 200 nlog 30 logstart 2inodestart 32 bmap start 58
  cpu 1: panic: iderw: nothing to do
   64656b63 0 0 0 0 0 0 0 0 0
```

# 2019-05-05 8:24

 EAX=00000000 EBX=00000000 ECX=00000012 EDX=8dfffffc
 ESI=00000013 EDI=8e000000 EBP=8016d6a8 ESP=8016d5c4
 EIP=8012a5d5 EFL=00000082 [--S----] CPL=0 II=0 A20=1 SMM=0 HLT=0
 ES =0010 00000000 ffffffff 00cf9300 DPL=0 DS   [-WA]
 CS =0008 00000000 ffffffff 00cf9a00 DPL=0 CS32 [-R-]
 SS =0010 00000000 ffffffff 00cf9300 DPL=0 DS   [-WA]
 DS =0010 00000000 ffffffff 00cf9300 DPL=0 DS   [-WA]
 FS =0000 00000000 00000000 00000000
 GS =0000 00000000 00000000 00000000
 LDT=0000 00000000 0000ffff 00008200 DPL=0 LDT
 TR =0000 00000000 0000ffff 00008b00 DPL=0 TSS32-busy
 GDT=     8016ba00 00000037
 IDT=     00000000 000003ff
 CR0=80010011 CR2=00000040 CR3=003ff000 CR4=00000010
 DR0=00000000 DR1=00000000 DR2=00000000 DR3=00000000
 DR6=ffff0ff0 DR7=00000400
 EFER=0000000000000000
 Triple fault.  Halting for inspection via QEMU monitor.

stosl で死んだ。
process.rs で、size\_of\_val の対象に * を付け忘れていた。解決。

GDB, QEMU のマニュアルをちゃんと読む。
kinit2 の kfree でスタックが壊れている？

startothers をしなくても同じように死ぬ。

picinit に入った段階で、stack が壊れている？

その前で、gdt をロードしている。

SEG の呼び出しで、stack を壊していた? -> 違ったかも。
esp < stack になるのを watch しておいたほうがいいな。

[rr] よさそう。... とおもったけど、Linux on Intel のみか。
[rr]: https://bitshifter.github.io/rr+rust/index.html#20

x86\_64 のときと違って、libtherad\_db library がないからうまく行かないのかもしれない。

# 2019-05-05 0:19

release で stack がこわれて死んでいる。

```
(gdb) bt
#0  kern::console::cpanic (s=...) at src/console.rs:127
#1  0x80125176 in kern::spinlock::release (lk=0x8015ce74 <kern::console::cons>) at src/spinlock.rs:60
#2  0x00000007 in ?? ()
#3  0x80167c6c in ?? ()
#4  0x80121a1f in kern::kernmain::mpmain () at src/kernmain.rs:41
#5  0x80121960 in kern::kernmain::mpenter () at src/kernmain.rs:36
```

done: uartinit   Starting cpu 1  stack: 0x803be000.

cprintf の release の位置がおかしいだけだった。

# 2019-05-04 22:13

 startothers 内部で止まっている。
 mpmain の
 thread2 による acquire に失敗している。
 cpu() 命令で死んでいる。

```
=> 0x801250e8 <kern::spinlock::acquire+120>:    movb   $0x4,0x23(%esp)
50          atomic::fence(atomic::Ordering::SeqCst);
(gdb)
=> 0x8012510b <kern::spinlock::acquire+155>:    mov    %gs:0x0,%eax
53          (*lk).cpu = cpu();
(gdb)
The target architecture is assumed to be i8086
[f000:d09b]    0xfd09b: cli
0x0000d09b in ?? ()
```

cpu けすパッチを持ってきて当てた。

# 2019-05-04 22:08

seginit で落ちている。
loadgs か。写し忘れだった。
解決。

# 2019-05-02 14:39

```
=> 0x8011c2c2 <kern::kernmain::startothers+114>:        mov    -0x20(%eax),%ecx
0x8011c2c2      66              core::mem::transmute(_binary_entryother_start),
(gdb)
=> 0x8011c2c8 <kern::kernmain::startothers+120>:        mov    (%ecx),%ecx
67              core::mem::transmute(_binary_entryother_size),
(gdb)

The target architecture is assumed to be i8086
```

`_binary_entryother_size` を読み出す時点で落ちている？
`_binary_entryother_*` は、address を読まないといけなかった。なおした。

(gdb)
=> 0x8011c4c0 <kern::kernmain::startothers+624>:        mov    (%ecx),%ecx
93                  v2p(V(entrypgdir as usize)).0 as u32,
(gdb)

で無限ループになっている。

^C
Thread 1 received signal SIGINT, Interrupt.
=> 0x80100058 <rust_begin_unwind+8>:    jmp    0x80100058 <rust_begin_unwind+8>
rust_begin_unwind (info=0x80153520 <stack+3488>) at src/lib.rs:86
86          loop {}

p2v でおかしくなる？

assert! で落ちているのか……。

```
=> 0x8011c4c0 <kern::kernmain::startothers+624>:        mov    (%ecx),%ecx
92                  v2p(V(entrypgdir as usize)).0 as u32,
```

なるほど、entrypgdir は extern されているからか。

(gdb) print entrypgdir
$1 = 0x80146000 <entrypgdir>

複数 thread の

thread 2 も起動するようになったぽい？
gdb で multithread のデバッグをする方法を学んだほうがよさそう。

- [1] http://www.sourceware.org/gdb/current/onlinedocs/gdb/Threads.html
- [2] https://access.redhat.com/documentation/en-US/Red_Hat_Enterprise_Linux/4/html/Debugging_with_gdb/threads.html
- [3] http://www.drdobbs.com/cpp/multithreaded-debugging-techniques/199200938?pgno=6
 
とりあえず、上記を読む。

gdb

- info threads
- thread `id`  - switch thread
- thread apply `id` args - apply command to the thread.
- set print thread-events

`break buffer.c:33 thread 7 if level > watermark` のように書ける。[3]


lapicstartap は機能して、thread 2 が走りはじめている。
0x7000 にでも breakpoint しかければいいのかな。

うまく走っているように見える。

# 2019-04-29 13:02 (done)


```
=> 0x8011e935 <kern::spinlock::Mutex<T>::new+5>:        push   %esi
0x8011e935      26          pub const fn new(val: T) -> Mutex<T> {
(gdb)
=> 0x8011e936 <kern::spinlock::Mutex<T>::new+6>:        and    $0xfffffff8,%esp
0x8011e936      26          pub const fn new(val: T) -> Mutex<T> {
(gdb)
=> 0x8011e939 <kern::spinlock::Mutex<T>::new+9>:        sub    $0x8310,%esp
0x8011e939      26          pub const fn new(val: T) -> Mutex<T> {
(gdb)
=> 0x8011e93f <kern::spinlock::Mutex<T>::new+15>:       call   0x8011e944 <kern::spinlock::Mutex<T>::new+20>
0x8011e93f      26          pub const fn new(val: T) -> Mutex<T> {
(gdb)
[f000:e05b]    0xfe05b: cmpw   $0xffc8,%cs:(%esi)
0x0000e05b in ?? ()
(gdb)

[f000:e062]    0xfe062: jne    0xd241d0cd
0x0000e062 in ?? ()

```

`lazy_static` ではなく、Mutex::new で落ちている。

```
call instruction で、直後の命令に飛んでいるだけに見えるんだけど。stack pointer がおかしいのだろうか。call instruction が暗黙に、rip を stack に push するはずなので、そこで例外が飛んでいるのだろうか。

Thread 1 hit Breakpoint 1, 0x8011e93f in kern::spinlock::Mutex<T>::new (val=...) at src/spinlock.rs:26
26          pub const fn new(val: T) -> Mutex<T> {
(gdb) info registers
eax            0x8014fef8          -2146107656
ecx            0x80147bc0          -2146141248
edx            0x8014fef8          -2146107656
ebx            0x8014e824          -2146113500
esp            0x8013f870          0x8013f870
ebp            0x80147b90          0x80147b90 <entrypgdir+2960>
esi            0x80147bc0          -2146141248
edi            0x8014bd38          -2146124488
eip            0x8011e93f          0x8011e93f <kern::spinlock::Mutex<T>::new+15>
eflags         0x82                [ SF ]
cs             0x8                 8
ss             0x10                16
ds             0x10                16
es             0x10                16
fs             0x0                 0
gs             0x0                 0
```

ESP の値がおかしい気がする。ちなみに、binit での値は以下。

```
Thread 1 hit Breakpoint 2, kern::bio::binit () at src/bio.rs:43
43          let mut bcache2 = bcache.lock();
(gdb) info registers
eax            0x8014e824          -2146113500
ecx            0x80141924          -2146166492
edx            0x80138013          -2146205677
ebx            0x8014e824          -2146113500
esp            0x801540a8          0x801540a8 <stack+3816>
ebp            0x801540cc          0x801540cc <stack+3852>
```

kernel のコードが、　
`801319b1:	e9 6b f5 ff ff       	jmp    80130f21 <alltraps>` まで存在する.
なので、esp がかなりそのページに迫ってしまっている感じがする。それが原因なのかな……

```
(gdb) bt
#0  0x8011e93f in kern::spinlock::Mutex<T>::new (val=...) at src/spinlock.rs:26
#1  0x801227f4 in <kern::bio::bcache as core::ops::deref::Deref>::deref::__static_ref_initialize () at src/bio.rs:39
#2  core::ops::function::FnOnce::call_once () at /Users/okakeigo/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/src/rust/src/libcore/ops/function.rs:231
#3  0x8012fcc0 in spin::once::Once<T>::call_once (self=0x8014f04c <<kern::bio::bcache as core::ops::deref::Deref>::deref::__stability::LAZY>, builder=0x0)
    at /Users/okakeigo/.cargo/registry/src/github.com-1ecc6299db9ec823/spin-0.5.0/src/once.rs:110
#4  0x80122330 in lazy_static::lazy::Lazy<T>::get (self=0x8014f04c <<kern::bio::bcache as core::ops::deref::Deref>::deref::__stability::LAZY>)
    at /Users/okakeigo/.cargo/registry/src/github.com-1ecc6299db9ec823/lazy_static-1.3.0/src/core_lazy.rs:21
#5  <kern::bio::bcache as core::ops::deref::Deref>::deref::__stability () at <::lazy_static::__lazy_static_internal macros>:12
#6  <kern::bio::bcache as core::ops::deref::Deref>::deref (self=0x80141924) at <::lazy_static::__lazy_static_internal macros>:13
#7  0x00000100 in ?? ()
#8  0x80141924 in ?? ()
#9  0x8011bfb5 in kern::kernmain::kernmain () at src/kernmain.rs:16
--Type <RET> for more, q to quit, c to continue without paging--
#10 0x00000000 in ?? ()
```

かなり階層が深いしな。

entry.S で、kernel の esp はセットされている。
KSTACKSIZE が小さすぎるのだろう。

GDB で、ページの権限がどうなっているかはどうみるのか？

entry.S の stack という値はどこで定義されている？
KSTACKSIZE = 8192 にしたが変わらなかった。

KSTACKSIZE = (4096 * 8) としてみたが、全く同じ場所で落ちる。

zeroed や、uninitialized が悪い可能性もある。切り分け。

[MaybeUninit] にあったこれが原因か。
The compiler, in general, assumes that variables are properly initialized at their respective type. For example, a variable of reference type must be aligned and non-NULL. This is an invariant that must always be upheld, even in unsafe code. As a consequence, zero-initializing a variable of reference type causes instantaneous undefined behavior, no matter whether that reference ever gets used to access memory:


[MaybeUninit](https://doc.rust-lang.org/core/mem/union.MaybeUninit.html)

nomicon を読むべきかもしれない。そうすることで、safe な unsafe code を書くことができるようになる。逆に読まないと、はまりどころがまたありそうだ。
Undefined behavior を書いてはいけない。

よく考えると、`lazy_static` って、メモリレイアウトどうなるんだ。static と同じく、.bss 領域に書き込まれるとおもっていたけど違うのかな。

変数宣言と、Buf の定義を以下のようにしたところ、loop までたどりつくようになった。

```
    static ref bcache: Mutex<Bcache> = Mutex::new(Bcache::default());

#[repr(C)]
pub struct Buf {
    pub flags: i32,
    pub dev: u32,
    pub blockno: u32,
    pub refcnt: u32,
    // pub prev: *mut Buf, // LRU cache list
    //    pub next: &'static mut Buf,
    //    pub qnext: &'static mut Buf, // disk queue
    // pub data: [u8; BSIZE],
}
```

registers, backtrace は以下。esp が stack + ... を指している。
```
(gdb) si
=> 0x8011e944 <kern::spinlock::Mutex<T>::new+20>:       pop    %eax
0x8011e944      26          pub const fn new(val: T) -> Mutex<T> {
(gdb) info registers
eax            0x80150ef8          -2146103560
ecx            0x80150aa8          -2146104664
edx            0x80150880          -2146105216
ebx            0x8014f824          -2146109404
esp            0x8015063c          0x8015063c <stack+1004>
ebp            0x80150a78          0x80150a78 <stack+2088>
esi            0x80150aa8          -2146104664
edi            0x80150ca8          -2146104152
eip            0x8011e944          0x8011e944 <kern::spinlock::Mutex<T>::new+20>
eflags         0x82                [ SF ]
...
(gdb) bt
#0  0x8011e944 in kern::spinlock::Mutex<T>::new (val=...) at src/spinlock.rs:26
#1  0x801228a4 in <kern::bio::bcache as core::ops::deref::Deref>::deref::__static_ref_initialize () at src/bio.rs:40
```

Buf の定義において、

```
    pub data: [u8; SZ],
```

として、SZ = 4 のときは、loop までたどりつくが、SZ = 5 のときはたどりつかない。

216 では同じく無限ループ

217 でクラッシュ:

```
=> 0x80122f09 <core::cell::UnsafeCell<T>::new+9>:       sub    $0x1cb8,%esp
0x80122f09      1492        pub const fn new(value: T) -> UnsafeCell<T> {
(gdb)
=> 0x80122f0f <core::cell::UnsafeCell<T>::new+15>:      call   0x80122f14 <core::cell::UnsafeCell<T>::new+20>
0x80122f0f      1492        pub const fn new(value: T) -> UnsafeCell<T> {
(gdb)
The target architecture is assumed to be i8086
[f000:e05b]    0xfe05b: cmpw   $0xffc8,%cs:(%esi)
```

qemu で、info mem すると、page table mapping がわかる。

```
(qemu) info mem
0000000080000000-0000000080100000 0000000000100000 -rw
0000000080100000-0000000080148000 0000000000048000 -r-
0000000080148000-000000008e000000 000000000deb8000 -rw
00000000fe000000-0000000100000000 0000000002000000 -rw
```

8014800 より下は、stack ガードになっていて、これ以上それが広がると死ぬということがわかる。これを踏んでいた。
info mem については、OS Dev の Qemu のページで知った。

カーネルででかいスタックをつかうとやばいということですね。Kernel のメモリレイアウトがちゃんと把握できていないので、今一度まとめておこう。
- Stack はどこからどこまでを使っていいのか、
- .data, .bss 領域というのもあるけど、実のところどれくらいの大きさになっているのかとか、それらがどうマッピングされているのかということを理解したい。
- stack 変数とはなんなのか。

こういうのを理解するのは前提条件だから焦るとかじゃなくてやらないといけない。やってからデバッグしたほうが絶対効率いいから、これはいまからやる。

なんか余裕で踏み越えている気がするなあ。

```
$2 = (<data variable, no debug info> *) 0x80152d00
(gdb) print &__data
$3 = (<data variable, no debug info> *) 0x80148000 <entrypgdir>
(gdb) print &stack
$4 = (<data variable, no debug info> *) 0x80151d00 <stack>
```

stack ってどこから値を得ているんだ。

なんか、kernmain の stack をセットアップしているところがコメントアウトされているように見える。これがバグの原因か。

[GNU as] reference を読む。

[GNU as]: https://sourceware.org/binutils/docs/as/Comm.html#Comm

.comm は、変数の宣言をしているのか。.comm symbol length で、 そのシンボル用の領域を、その長さだけ確保している。

```
> $ i386-elf-nm entry.o                                                                                                                                                 [±master ●●]
8000000c T _start
0000000c T entry
         U entrypgdir
         U main
00000000 T multiboot_header
00001000 C stack
```

```
nm - list symbols from object files.

           "C" The symbol is common.  Common symbols are uninitialized data.
               When linking, multiple common symbols may appear with the same
               name.  If the symbol is defined anywhere, the common symbols
               are treated as undefined references.

           "T"
           "t" The symbol is in the text (code) section.

           "U" The symbol is undefined.
```

どの symbol がどの section にいるのかをひと目で知る方法は

stack 領域は、.bss section に、他の変数と混じって存在することがわかった。4K byte を踏み越えると容赦なくバグるということか。

xv6 に対して実験。

```
> $ i386-elf-readelf -S kernel                                                                                                                                          [±master ●●]
There are 18 section headers, starting at offset 0x32fd8:

Section Headers:
  [Nr] Name              Type            Addr     Off    Size   ES Flg Lk Inf Al
  [ 0]                   NULL            00000000 000000 000000 00      0   0  0
  [ 1] .text             PROGBITS        80100000 001000 0065fa 00  AX  0   0  4
  [ 2] .rodata           PROGBITS        80106600 007600 0009ac 00   A  0   0 32
  [ 3] .stab             PROGBITS        80106fac 007fac 000001 0c  WA  4   0  1
  [ 4] .stabstr          STRTAB          80106fad 007fad 000001 00  WA  0   0  1
  [ 5] .data             PROGBITS        80107000 008000 002516 00  WA  0   0 4096
  [ 6] .bss              NOBITS          80109520 00a516 00b008 00  WA  0   0 32
  [ 7] .debug_line       PROGBITS        00000000 00a516 005ee2 00      0   0  1
 ...


$ i386-elf-nm kernel -lnsS | less 

...
80109516 D _binary_entryother_end
80109520 00000038 b cons        /Users/okakeigo/src/github.com/ogiekako/xv6-public/console.c:25
80109558 00000004 b panicked    /Users/okakeigo/src/github.com/ogiekako/xv6-public/console.c:20
80109560 00000004 b havedisk1   /Users/okakeigo/src/github.com/ogiekako/xv6-public/ide.c:34
80109564 00000004 b idequeue    /Users/okakeigo/src/github.com/ogiekako/xv6-public/ide.c:32
80109580 00000034 b idelock     /Users/okakeigo/src/github.com/ogiekako/xv6-public/ide.c:31
801095b4 00000004 b shift.1423
801095b8 00000004 b n.1539
801095bc 00000004 b initproc    /Users/okakeigo/src/github.com/ogiekako/xv6-public/proc.c:15
801095c0 00000004 b uart        /Users/okakeigo/src/github.com/ogiekako/xv6-public/uart.c:17
801095d0 00001000 B stack
8010a5e0 00004958 B bcache      /Users/okakeigo/src/github.com/ogiekako/xv6-public/bio.c:36
8010ef40 0000008c B input       /Users/okakeigo/src/github.com/ogiekako/xv6-public/console.c:186
```

bss と common の違いってこういうことなのか。https://stackoverflow.com/questions/16835716/bss-vs-common-what-goes-where

stack がどう定義されているかは完全にこれでわかったな。それは、.bss section の中に他の変数と入り混じって存在する。

rx6 では、stack は以下。

```
80151d00 00001000 B stack
80152d00 B __end
```

`stack + 4096` から、esp は開始している。
kernmain の時点で、すでに、stack + 3936 になっている。あまりにも下がり過ぎでは？
そんなことなかった。

Mutex::new においてはすでに踏み越えている。
call_once

lazy_static::lazy::Lazy<T>::get
defer::__stability
deref

```
(gdb) bt
#0  lazy_static::lazy::Lazy<T>::get (self=0x8015004c <<kern::bio::bcache as core::ops::deref::Deref>::deref::__stability::LAZY>)
    at /Users/okakeigo/.cargo/registry/src/github.com-1ecc6299db9ec823/lazy_static-1.3.0/src/core_lazy.rs:21
    #1  <kern::bio::bcache as core::ops::deref::Deref>::deref::__stability () at <::lazy_static::__lazy_static_internal macros>:12
    #2  <kern::bio::bcache as core::ops::deref::Deref>::deref (self=0x801424a4) at <::lazy_static::__lazy_static_internal macros>:13
    #3  0x00000100 in ?? ()
    #4  0x00000100 in ?? ()
    #5  0x8015004c in kern::lapic::lapic ()
    #6  0x8014f824 in ?? ()
    #7  0x8011bfb5 in kern::kernmain::kernmain () at src/kernmain.rs:16
    #8  0x00000000 in ?? ()
(gdb) print $esp
$7 = (void *) 0x80152bd0 <stack+3792>
```


とりあえず、binit でとめるか。

stack = 80151d00

call_once - 8014f210
deref    - stack+3792  (bt が壊れている。)
binit    - stack+3816
kernmain - stack+3936


BCache というのは、BUF の 31 倍の大きさの構造体なんだな。そりゃおかしくなるな。そして、Buf は、BSIZE = 512 bytes の配列を持っているわけで。

結局、でかい構造体を初期化するのに、 lazy_static は使うなという教訓が得られた。

2019-04-30 11:35 - デバッグ終了。













# 2019-04-29 10:32 

```
(gdb) n
=> 0x8011d1ff <spin::once::Once<T>::call_once+239>:     mov    %eax,0x7c(%esp)
spin::once::Once<T>::call_once (self=0x8014f7f0 <<kern::kernmain::piyo as core::ops::deref::Deref>::deref::__stability::LAZY>, builder=0x3ff000)
    at /Users/okakeigo/.cargo/registry/src/github.com-1ecc6299db9ec823/spin-0.5.0/src/once.rs:110
    110                     unsafe { *self.data.get() = Some(builder()) };
    (gdb) n
    The target architecture is assumed to be i8086
    [f000:e05b]    0xfe05b: cmpw   $0xffc8,%cs:(%esi)
    0x0000e05b in ?? ()
    (gdb)
```

once.rs:110 内部でおかしくなっている。
さらに詳しく見ていく。

```
(gdb)
=> 0x8011d25f <spin::once::Once<T>::call_once+335>:     movb   $0x0,0x6f(%esp)
0x8011d25f      110                     unsafe { *self.data.get() = Some(builder()) };
(gdb)
=> 0x8011d264 <spin::once::Once<T>::call_once+340>:     jmp    0x8011d3aa <spin::once::Once<T>::call_once+666>
0x8011d264      110                     unsafe { *self.data.get() = Some(builder()) };
(gdb)
=> 0x8011d3aa <spin::once::Once<T>::call_once+666>:     movsd  0x58(%esp),%xmm0
0x8011d3aa      110                     unsafe { *self.data.get() = Some(builder()) };
(gdb)
The target architecture is assumed to be i8086
[f000:e05b]    0xfe05b: cmpw   $0xffc8,%cs:(%esi)
0x0000e05b in ?? ()
```

xmm0 レジスタに movsd している。これのアラインメントがおかしかったりするのかなあ。

```

   0x8011d399 <+649>:   jne    0x8011d38a <spin::once::Once<T>::call_once+634>
   0x8011d39b <+651>:   jmp    0x8011d168 <spin::once::Once<T>::call_once+88>
   0x8011d3a0 <+656>:   movb   $0x0,0x6f(%esp)
   0x8011d3a5 <+661>:   jmp    0x8011d23d <spin::once::Once<T>::call_once+301>
=> 0x8011d3aa <+666>:   movsd  0x58(%esp),%xmm0
   0x8011d3b0 <+672>:   mov    0x28(%esp),%eax
```


xmm0 を使うと無条件で落ちるのか否かをしらべてみるか。
たんに SSE2 がサポートされていない可能性がある。
やっぱりそうだ。ようするに、単に xmm0 を使う命令がサポートされていない。これだけのことになんでこんなに時間がかかっているんだ。複雑なバグだと思い込みすぎなんだろうなあ。

QEMU で、SSE2 に対応する CPU ID を enable すればよさそう。

`-cpu qemu32,+sse2` を追加。

[ここ](https://www.linuxquestions.org/questions/slackware-14/sigill-due-to-movsd-on-pentium-iii-4175632396/) に書いてあることには、SSE2 

[How to disable SIMD](https://os.phil-opp.com/disable-simd/) をみて disable しようとしたけど、されない。以下を i686-unknown-linux-gnu.json として保存して、--target=i686-unknown-linux-gnu.json にしたのだけど、コンパイル結果の kernel.asm に変化がなかった。

```
{
  "llvm-target": "i686-unknown-none",
  "data-layout": "e-m:e-p:32:32-f64:32:64-f80:32-n8:16:32-S128",
  "linker-flavor": "gcc",
  "target-c-int-width": "32",
  "target-endian": "little",
  "target-pointer-width": "32",
  "arch": "x86",
  "os": "none",
  "features": "-mmx,-sse"
}
```

... とおもったら、勘違いで、これでよかった。`couldn't find crate lazy_static ...` みたいなエラーが出たけど、kern/ で cargo clean して core を再コンパイルしたらそのエラーは消えた。

12:57 - 解決。
