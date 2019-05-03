 2019-05-02
// 
// 14:30 - startothers のデバッグを開始。
// 
// x86 環境で test を走らせたくなってきた。

starting thread 1 の表示までできた。
thread が正常に走っている？

line count でいうとまだまだ。

まず、ptable が Mutex で守られているのをもとに戻す。

userinit をコンパイルしようとしている。
`lazy_static` は危険なのでなくしたい。

まず、struct をすべて compile できるようにして、存在しない変数になやまないようにするか。

- kalloc もOption つかわない version に戻したい。... とおもったけど意外とつかわれていないのか。じゃあほっとくか。

trapret はどこ？

関数の依存グラフ

* : 途中 , + : 依存に対するコメントアウト以外おわり

- userinit (process.rs)
      - initlog *
  - p.cwd
    - Inode (file.rs) +
      - Sleeplock +
        - extern struct proc = asm(%gs)in (proc.h) *
        - sleep *,
        - wakeup *
  - namei * (fs.rs)
    - namex *
      - iget, idup, ilock ...

initcode.S の SYS_exec が存在しない。これどうしようかな。必要なとこだけ header でいいや。どうせ対して使わないし。


u32 は使わず、すべて usize に置き換えたほうが良かったかもしれない。あとでいいけど。

# 2019-05-01

13:18 - 開始が遅れてしまった。

Default で move semantics であることが、C の default は copy であることで問題を起こしたりしないだろうか。これを防ぐには、すべてに Derive(Copy) をつけておけば良い？

14:33 - 休憩。
15:45 - 再開。macbook になった。

const_transmute feature により、core::mem::transmute が const になったので、static 変数の初期化が楽になった。


startothers を翻訳するにあたり、extern をどうするか考える。
まず、xv6 でこれがどういう意味でなぜ使われているのかを理解する。

`main.c` にある、`extern _binary_entryother_start` ってなにかと思ったが、これは、linker が自動生成する変数っぽい。??

```
>-$(LD) $(LDFLAGS) -T kernel.ld -o kernel entry.o $(OBJS) -b binary initcode entryother
```

上記の、option において、途中に -b binary がはさまっている。
-b の手前までは、i386 elf binary として解釈され、それ以降は、binary として解釈される。
linker が binary をどう解釈するんだ？

16:41 - 場所を移動。
16:56 - 再開。

-b で指定できるのは、[BFD library] で規定されているもの。
[BFD library]: https://en.wikipedia.org/wiki/Binary_File_Descriptor_library
objdump -i で使用可能なリストを取得できる。

おそらく、ld は binary を解釈せず、たんに後ろにつなげる？
[Embedding Blobs in Binaries](https://gareus.org/wiki/embedding_resources_in_executables) というブログによるとそう。

ld -r -b binary -o x.o x とすると、x のデータが、.data section に収められた、elf relocatable object file ができあがる。さらに、
`_binary_x_start, _binary_x_end, _binary_x_size` 変数が定義される。
この自動生成される定数について、リファレンスのどこに書かれているかが見つからない。なにをやっているかはわかったので一旦それを探すのは保留にしよう。
対応する gold のソースコードをみつけた。(_FILENAME_ で検索)
http://ftp.netbsd.org/pub/NetBSD/NetBSD-current/src/external/gpl3/binutils/dist/gold/binary.cc

さて、binary が埋め込まれるのはわかった。それを xv6 はどのように利用しているのだろうか？

17:24 - 休憩。(場所を移動)
17:36 - 再開

entryother は、bootblockyother.o の .text 部分のみを取り出したもの。
0x7000 にそれを貼り付けて、0x7000 に飛べば動くように Makefile の entryother section で、entryother.S から作られている。

0x7000 から始まるメモリ領域(とそのまえの entrypgdir 用の領域) が unused であることをみこしている。

17:59 - トイレ行く
18:26 - 再開。休憩し過ぎ感。

kalloc が Option を返すようになってる... これももとに戻そう...

```
    xchg(&mut ((*mycpu()).started) as *mut u32, 1); // tell startothers() we're up
```

Rust compiler の ICE を踏んでしまったぽい。

Minimal reproduce code https://play.rust-lang.org/?version=nightly&mode=debug&edition=2018&gist=63dcfc89365288e1713fa47e038dba23 . xchg の asm! が問題であった。
inline assembly について、ICE 報告はいくつか上がっている。
https://github.com/rust-lang/rust/issues?q=is%3Aissue+is%3Aopen+asm+ICE+label%3AA-inline-assembly
けっこうバグバグなのかな……。

https://github.com/rust-lang/rust/issues/51130 - `We should validate lots and lots of things about the values passed to inline asm, and we currently don't do any of them. That's no reason to not fix it, but it will be a drop in the bucket.`
なるほど。

19:53 - 休憩
20:51 - 帰宅して再開.

Rust の [intrinsics] の volatiles と、atomics の項の、Acuiqre, Release の説明がとても簡潔でわかりやすかった。

[intrinsics](https://doc.rust-lang.org/core/intrinsics/index.html)

TODO: 読む？ [Synchronization in Xv6 – Brian Pan – Medium](https://medium.com/@ppan.brian/synchronization-in-xv6-be05ae0b34ec)

22:21 - 再開

GCC と LLVM の inline assembler の違いを抑えておきたい。まずは、LLVM のほうのドキュメントを読む。

Documents:

- [GCC asm]: https://gcc.gnu.org/onlinedocs/gcc/Extended-Asm.html#Extended-Asm
- [LLVM asm]: http://llvm.org/docs/LangRef.html#inline-assembler-expressions

違い (GCC -> LLVM)
- %0, %1 ... -> $0, $1 ...
- $0, ... -> $$0, ...
- "=a" -> "={eax}"

以下のコードを読み解いていくか。

```
static inline uint
xchg(volatile uint *addr, uint newval)
{
  uint result;
  // The + in "+m" denotes a read-modify-write operand.
  asm volatile("lock; xchgl %0, %1" :
               "+m" (*addr), "=a" (result) :
               "1" (newval) :
               "cc");
  return result;
}
```

`=` は、そこに書き込みが発生するということ。
`+` は、Read も Write の発生するということ。`=`, `+` 以外は、readonly とみなされる。([Modifier])
m という constraint は、memory を意味する。([Simple Constraints])
a という constraint は、a register を意味する。(See [Machine Constraints])
"1" は、%1 と同じものを指すことを表す。

clober list の "cc" は、flag registers が変化することをしめす。
"memory" は、input, output に示されていないメモリの読み書きが発生することを示す。(結果として read/write memory barrier がコンパイラにより生成される)

[Simple Constraints]: https://gcc.gnu.org/onlinedocs/gcc/Simple-Constraints.html#Simple-Constraints
[Modifiers]: https://gcc.gnu.org/onlinedocs/gcc/Modifiers.html#Modifiers
[Machine Constraints]: https://gcc.gnu.org/onlinedocs/gcc/Machine-Constraints.html#Machine-Constraints

addr は C では volatile 変数だけど、これは Rust でどう表せば？

clang で、llvm にコンパイルすれば、対応する llvm inline asm がわかるのでは？ それを Rust に翻訳しなおせばよさそう。

Rust における、asm! の エラーテスト:
https://github.com/rust-lang/rust/tree/9ebf47851a357faa4cd97f4b1dc7835f6376e639/src/test/ui/asm

x86.h を llvm にコンパイル。

```
clang --target=i686-unknown-linux-gnu -S -emit-llvm a.c
```

asm 部分のコードは以下のようになった。

```
  %8 = call i32 asm sideeffect "lock; xchgl $0, $1", "=*m,={ax},1,*m,~{cc},~{dirflag},~{fpsr},~{flags}"(i32* %6, i32 %7, i32* %6) #1, !srcloc !3
```

asm! において、`*addr` を `addr` にしたらコンパイルとおったけど、本当にこれでいいのかな。
LLVM として出力されるものを見るか……。

```
%4 = call i32 asm sideeffect "lock; xchgl $0, $1", "=*m,={ax},1,~{cc},~{dirflag},~{fpsr},~{flags}"(i32* %3, i32 %2), !dbg !33, !srcloc !34
```

微妙にちがうね。
しかし、addr に関しては完全に同じアクセスのされかたであった。つまり、`*` を外したのは正しかったぽい。


# 2019-04-30

Bcache の initial value を定義するのに、const fn 内で使える array! macro ないのかなと思ったけど、わからなかった。これは TODO にしておく。arr! は良さそうに見えたが、no_std でそのままでは動かなかった。build 時のみの依存とかはできないのかな。

ついにエラーなくロックを突破できた。

Mutex を一旦捨てたい。もし必要になったらログから取れるから、一旦この段階では捨てて、完全に機械的に書き換えるようにしていこう。spinlock を復活させる。

cprintf もなんか変な抽象化が入っている…… こういうのやめてほしいな。

[sync.md](sync.md) に、memory synchronization 関連の情報をまとめた(WIP)。

# 2019-04-29

8:52 - Qemu が、terminal の折り返し設定をバグらせるらしい。qemu 起動したあとで、長いコマンドを表示すると、折り返しが次の行にいかなくなってしまう。

コードをおっていくと、`x86_64` のコードを呼んでいるところがあるようだ。これがおかしいのでは。

```
=> 0x801226b7 <core::ops::function::FnOnce::call_once+71>:      pop    %ebx
0x801226b7 in core::ops::function::FnOnce::call_once () at /Users/okakeigo/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/src/rust/src/libcore/ops/function.rs:231
231         extern "rust-call" fn call_once(self, args: Args) -> Self::Output;
```
それは関係ないだろう。

# 2019-04-28

## 午後

Incrementally に書いていけないかを考えている。C のコードから、Rust の Library を呼べれば、それはできそう。
キメラみたいな感じになる。Interface をきれいに切れていれば可能なはず？しかしメモリレイアウトがどうなるか？
あーそっか、メモリレイアウトについては、linker がよしなにやってくれるはずだから、別に大丈夫なのかな。
いま考えているのは、xv6-public を update して、途中から Rust のコードを呼ぶようにするということ。
これができれば、incremental な改善にかなり近づいていくとおもう。
まだがっととにかくやるアプローチを実験していない。まずはそれかな。それで一気にいける目処がたてば、それがいちばんいいわけだし。
関数がないところはスタブで残しておいて、ともかく end to end でコンパイルして実行できるところを一段落とすれば、ある意味 incremental にできるかもしれない。
最高速でやったときの、LOC の減り方を見てみたい。一時間でどれくらい行けるか。どかどか脳死で変換するのが一つのベースライン的なやり方であって、それができればその後がだいぶやりやすくなる。というわけで実験、推定。


単純な命名規則に従う。創造性を可能な限りおさえ、スピードに集中する。
unsafe をいとわない。

Mutex とか変にいれたせいで、単純なかきかえがやりにくくなっているな。
mutex で lock しただけで落ちるな。
Mutex を自前実装しているのがわるいきもするから、C のほうの lock に切り替える？でも実際なにがおこっているのかわからないのは気持ち悪いから調査するか。しょうがない。今回はスピードの実験としては失敗だな。単純な書き換えメソッドを確立する必要がある。

17:42 - 開始

Linux でやったほうが、qemu が fault したときに、単に再起動しないで、`info registers` して落ちてくれるから便利だな。

Mutex は、別のファイルにうつしておいて、C の方の 実装をもってこようかな。それで、できるだけ機械的に翻訳をやるというのをまずは試したい。
しかし、Mutex の実装がまずいのか、ほかがなにか悪いのかの切り分けはしたほうがいい気がする。しばらくデバッグして、わからなければ、C実装をもってくるのをためそう。

わかった。lazy_static を使っているのが原因である。lazy_static は内部的に割り込みを使っているっぽい。いや、 zeroed で memset つかってるのか。
なんかそもそも lazy_static で問題おこってるぽいな。

# 2019-04-26

18:16: fs.img, xv6.img という 2 つのドライブと、それらを指し示す物理アドレス、論理アドレスの関係がわかっていない。kernel image は、hda の block 1 から、fs.img は hdb の block 1 から開始するように見える。これらは OS からはどう見えているのだろう？
だめだ、眠い。いまいちスピード感が持てていないなあ。

## 午後

内容を理解するのと、数値てきな進捗を出すのを同時にやったほうがいい。最悪でもむりやり書き換えすすめれば進捗はでる。いっぽう何も考えずやるだけも考えもの。何も考えずやる時間を記録して、その進み方でどれくらいでできるのかを押さえる。

kernmain をみたところの、現在の状況をまとめていく。

freelist (ページを管理するやつ) の `use_lock` はないが、page 管理はすでに実装されている。
- kfree, kalloc は実装済み. lock も存在する。

kernmain のなかに TODO と書いてあるのが未実装ということかな。
であれば、概ね以下が未実装である。

- Buffer cache  `bio::binit`
- File table    `file::fileinit`
- Disk initialization `ide::ideinit`
- Uniprocessor timer initialization.
  - これは非 multiprocessor ではとおらないので優先度は低い
- Starting other processors `startothers`
- Running first user process `process::usertinit`
- Mpmain (finish setup)  `mpmain`

その他、システムコールの実装は別でやらないといけないと思う。

すでに実装してあるものも興味深いけど、学ぶ優先度としては未実装のもののほうが高い。

まず、buffer cache は sleep lock を使用しているので、sleep lock ができるために、割り込みを実装する必要がある。

## Twitter

PとVについて、into<u32>() を実装するといいのではないだろうか？
Physical address と Virtual address をくべつできるとよい。

しかし、優先度的には、まだ実装されていないもののほうが優先度は高い。リファクタリングもいいけど、機能実装をスピードアップしないのならば後回しでよい。なので、現状をまとめておくのがひとまずは最優先だろう。

## 午前

binutils-2.24 OK
binutils-2.30 NG
binutils-2.32 NG

この[メール](http://lists.llvm.org/pipermail/llvm-dev/2012-December/057390.html), linux における linker script の使用機能がまとまっていてよい。

PHDRS で、explicit に program header を指定できるっぽいな。なにか問題がおきたら試してみよう。

とりあえず、まともに、make qemu-nox が動くようになった。ようやく本題にとりかかれる。
ひとまず、現状を復習するのと、それを sheet にまとめるところからやるか。何行 translate しなければいけないかを単純に見ていくのがよさそう。

C言語のコメントアウトは、 //// にするというのをまずして、translate されていないものの数を見ていくみたいにするのがいいのかなあ。そうするとそれなりのセンスはわかりそう。
allow

```
> $ grep -E '^////' *.rs | wc  # C comment out
2782   11125   96698

> $ grep -E -v '//' *.rs | wc  # Rust code
1792    6196   51146

> $ grep -E '^//' *.rs | wc    # All comments
3119   13561  113581
```

こんな感じになった。まだ、C のコメントアウトのほうが Rust のコードより多いんだな……。
他には、.rs にうつしていないコードはないのだろうか。

kernel 以外に、fs.img 用のコードもあるはずだ。こちらはなにも手を付けていない。


fs.img はユーザプログラム。mkfs によって作られている。mkfs は、mkfs.c, fs.h を使っている。

mkfs.c は 297 行, fs.h は 57 行あった。(dinode とかの定義をしている)

user programs は、以下の 15 種がある。

```
_cat\
_echo\
_forktest\
_grep\
_init\
_kill\
_ln\
_ls\
_mkdir\
_rm\
_sh\
_stressfs\
_usertests\
_wc\
_zombie\
```

User progam だけで 3000 行くらいあるんだな。これらは、kernel とは無関係で、そのままつかえるはずではある。

43    87   589 ../xv6-public/cat.c
13    34   198 ../xv6-public/echo.c
56   112   758 ../xv6-public/forktest.c
107   292  1954 ../xv6-public/grep.c
37    83   649 ../xv6-public/init.c
17    29   232 ../xv6-public/kill.c
15    35   264 ../xv6-public/ln.c
85   197  1525 ../xv6-public/ls.c
23    45   327 ../xv6-public/mkdir.c
23    45   322 ../xv6-public/rm.c
493  1071  8240 ../xv6-public/sh.c
49   160  1028 ../xv6-public/stressfs.c
1803  4493 34697 ../xv6-public/usertests.c
54   124   820 ../xv6-public/wc.c
14    33   214 ../xv6-public/zombie.c
2832  6840 51817 total

さらに、これらの user programs は、ulib.o, usys.o, printf.o, umalloc.o を使っている。(kernel code とは dinsinct).

```
105  221 1243 ../../../xv6-public/ulib.c
31   43  461 ../../../xv6-public/usys.S
85  232 1466 ../../../xv6-public/printf.c
90  247 1652 ../../../xv6-public/umalloc.c
311  743 4822 total
```

まあたいしたコード量じゃないな。

kernelmemfs はなんだろう？ これは、disk image を in memory におく version のカーネル。ide.o の代わりに memide.o (60行) を使うだけが違い。ひつまず考えなくていい。

xv6-public 以下の `*.h` file 全体が、1182 行ある。ほぼ struct と空の関数宣言, 定数の define だけなので、これはそんなに気にしなくても良さそうだ。

vectors.S は、vectors.pl によって自動生成されるものなので、まあその perl program 自体をコピーしてくるか。.S に関してはわざわざ rust に変換する必要もなさそう。
kern/asm directory でもつくってそこに入れておくか。いれる場合は、Makefile も適切にいじらないとわすれてしまうから注意。

TODO: swtch.S, vectors.pl, trapasm.S と対応する Makefile をそこにコピーすればOK.
trapasm.S, vectors.pl についてはすでにやっていた。

[ここ](https://docs.google.com/spreadsheets/d/1QNyT3kbKktdmjbfZg4R_JPp7AtYYrBqik7Fu3apItEw/edit#gid=0) で進捗を track していくことにした。

# 2019-04-25

20:05 - 作業開始。
bootmain に unittest をつけたいところだ。いまやっちゃっていいかな……。直してからのほうがいい気がする。
しかしきりわけにやくだつから微妙なところだ。

```
> $ readelf -l kernel.bad

Elf file type is EXEC (Executable file)
Entry point 0x10000c
There are 3 program headers, starting at offset 52

Program Headers:
  Type           Offset   VirtAddr   PhysAddr   FileSiz MemSiz  Flg Align
  LOAD           0x001000 0x80100000 0x00100000 0x46c3a 0x46c3a R E 0x1000
  LOAD           0x047c3a 0x80146c3a 0x80146c3a 0x07b06 0x09336 RW  0x1000
  GNU_STACK      0x000000 0x00000000 0x00000000 0x00000 0x00000 RWE 0x10

 Section to Segment mapping:
  Segment Sections...
   00     .text .rodata .gcc_except_table .debug_gdb_scripts
   01     .stab .data .got .got.plt .bss
   02
```

OK. bootloader は、program header の paddr にかかれている場所を起点としてデータを読み込む。しかし、2 番目の program header の paddr がなぜか、vaddr と同じになっているため、bootloader はその場所からデータを読む。しかし、そこには物理的な実体は存在しないため、すべて 0 を読み込んでしまい、バグる。結局 root cause は、paddr が期待と異なることであった。

なぜこうなるのか. [SECTIONS Command](https://access.redhat.com/documentation/en-US/Red_Hat_Enterprise_Linux/4/html/Using_ld_the_GNU_Linker/sections.html) を読んでみる。

Glossary:
-  LMA - load address. (physical addr のことか).

The linker will normally set the LMA equal to the VMA. You can change that by using the AT keyword.

Output Seciotn LMA を読んで、AT() を explicit に指定したら、具体的に何がわるいのかわかった。

```
> $ ld -melf_i386 -T kernel.ld -o kernel entry.o entrypgdir.o kern/target/i686-unknown-linux-gnu/debug/libkern.a trapasm.o vectors.o -b binary                                 [±master ●●]
ld: section .stab LMA [0000000080146acd,0000000080146acd] overlaps section .gcc_except_table LMA [0000000080146ac8,0000000080146c0f]
```
暗黙に追加された .gcc_except_table が、.stab の LMA を侵食しているのだ。
input section にリストされていない、マッチしなかった section は、自動的に自身と同じ名前で出力に追加されるということか。/DISCARD/ に .gcc_except_table を追加すればなおるのかな。
-> .gcc_except_table, .debug_gdb_scripts を追加したところなおった……。
とおもったけど、結局、LMA がおかしいのは治らなかった。SECTIONS Command を参考にして、explicit に >region AT>lma_region を追加したところうまく行った。もう全部 explicit に書いたほうがいいな。そうしないと ld が勝手に解釈してすきあらば VMA, LMA を同じにしてしまう気がする。

libkern.a を readelf -S すると、gcc_except_table が含まれているのが気になるな…　サイズも 0 ではないし、なんなのだろう。おなじ疑問をもった人が[いた](https://users.rust-lang.org/t/exception-tables/19671)。panic を handle するのに使うのかな。
ちなみに、gcc_except_table は、rust の方の object file にしかない。(C code では生成されない). .debug_gdb_script も同様である。

LD, warning for unlisted input みたいなのないのかな……
ぐぐぐ、VirtAddr と PhysAddr の offset がずれている…… ぜんぶ一つの領域においてほしいだけなんだが……

これは、Rust でコンパイルしたバイナリが、PIC であることが関係しているのかもしれない [この](https://forum.osdev.org/viewtopic.php?f=1&t=32639) thread によるとそんな気配がする。

cross compiler (i386-elf-ld) を、https://qiita.com/maueki/items/b38d06c7d332d94e2981 を参考にして使うようにしたらなおった……。

2019-04-24

asm file に rust の対応するコードをコメントで埋め込む方法はないのだろうか。これはできたら便利なので早めに調べたい。

2019-04-23

Ubuntu で、make qemu-nox したら、0x100028 で Triple fault を起こすので、勉強しつつ、なぜかを考えていく。

Fault した時点のレジスタは以下。

```
EAX=80010011 EBX=00007eac ECX=00000000 EDX=00000000
ESI=00010094 EDI=00000000 EBP=00010094 ESP=00007bdc
EIP=00100028 EFL=00000086 [--S--P-] CPL=0 II=0 A20=1 SMM=0 HLT=0
ES =0010 00000000 ffffffff 00cf9300 DPL=0 DS   [-WA]
CS =0008 00000000 ffffffff 00cf9a00 DPL=0 CS32 [-R-]
SS =0010 00000000 ffffffff 00cf9300 DPL=0 DS   [-WA]
DS =0010 00000000 ffffffff 00cf9300 DPL=0 DS   [-WA]
FS =0000 00000000 00000000 00000000
GS =0000 00000000 00000000 00000000
LDT=0000 00000000 0000ffff 00008200 DPL=0 LDT
TR =0000 00000000 0000ffff 00008b00 DPL=0 TSS32-busy
GDT=     00007c60 00000017
IDT=     00000000 000003ff
CR0=80010011 CR2=00100028 CR3=00147000 CR4=00000010
DR0=00000000 DR1=00000000 DR2=00000000 DR3=00000000
DR6=ffff0ff0 DR7=00000400
EFER=0000000000000000
```

ちなみに、xv6 のその時点でのレジスタが以下。

```
EAX=80010011 EBX=00010074 ECX=00000000 EDX=000001f0
ESI=00010074 EDI=00000000 EBP=00007bf8 ESP=00007bdc
EIP=00100028 EFL=00000086 [--S--P-] CPL=0 II=0 A20=1 SMM=0 HLT=0
ES =0010 00000000 ffffffff 00cf9300 DPL=0 DS   [-WA]
CS =0008 00000000 ffffffff 00cf9a00 DPL=0 CS32 [-R-]
SS =0010 00000000 ffffffff 00cf9300 DPL=0 DS   [-WA]
DS =0010 00000000 ffffffff 00cf9300 DPL=0 DS   [-WA]
FS =0000 00000000 00000000 00000000
GS =0000 00000000 00000000 00000000
LDT=0000 00000000 0000ffff 00008200 DPL=0 LDT
TR =0000 00000000 0000ffff 00008b00 DPL=0 TSS32-busy
GDT=     00007c60 00000017
IDT=     00000000 000003ff
CR0=80010011 CR2=00000000 CR3=00109000 CR4=00000010
DR0=00000000 DR1=00000000 DR2=00000000 DR3=00000000
DR6=ffff0ff0 DR7=00000400
EFER=0000000000000000
```

# 仕様

- CR0 の `CR0_PG` - enable paging.

- CR3 -> page table.

# 観察

cr3 = entrypgdir がの中身が空になってしまっているようだ。
もともとの kernel のデータには、entrypgdir のアドレス には、値 0x83 が入っている。

```
% objdump -s -j .data ./kernel 

./kernel:     file format elf32-i386

Contents of section .data:
 80147000 83000000 00000000 00000000 00000000  ................
```

しかし、EIP=0x100025 の時点で gdb で見てみると、メモリの値が 0 になっている。

```
gdb-peda$ x/10x entrypgdir
0x80147000 <entrypgdir>:        0x00000000      0x00000000      0x00000000      0x00000000
0x80147010 <entrypgdir+16>:     0x00000000      0x00000000      0x00000000      0x00000000
0x80147020 <entrypgdir+32>:     0x00000000      0x00000000
gdb-peda$ x/10x 0x147000
0x147000:       0x00000000      0x00000000      0x00000000      0x00000000
```

bootblock でカーネルがすべてよみこまれていない？

xv6 では、entrypgdir は main.c に定義されていて、値付きで初期化されている。

rx6 では、entrypgdir.c があって、それが kernel にリンクされている。

おそらく、ページングの設定の問題。

mov %eax, %cr0 の直後で、

cr0 - 
cr3 - 


kernel を、xv6 のものに置き換えたら、ちゃんと起動した。つまり、bootloader は間違っていないということか。

よく見ると、make kernel 時に、

```
ld: warning: dot moved backwards before `.stab'
```

という warning がでていた。当然これは、xv6 では出ていない。
RELEASE = release にして実行してみたら、link の warning は出なくなって、カーネルパニックがもとの場所では起きなくなった。しかし、原因をちゃんと突き止める必要がある。
- ld のオプションで warning を error する。 (Done)
- なぜエラーになったかを調べる。


このエラーの意味をかんがえる。まず、dot とはなにか？ これは linker script における概念。[ld manual] によると、dot is a special linker variable.
. に対して値を代入するのは、output cursor を移動させるという副作用がある。
  The location counter may never be moved backwards.
とある。さきの warning はこれに違反していることを示すものだった。

man ld によると、linker script とは、AT&T's Link Editor Command Language syntax で書かれている。

.stab とはなにか？

[ld manual]: https://ftp.gnu.org/old-gnu/Manuals/ld-2.9.1/html_chapter/ld_3.html

.stab と、.stabstr の中身をコメントアウトしたら、急にちゃんと動くようになった。謎すぎる。これをほおっておくとまたへんなバグの原因になりかねないからちゃんと調べよう。
BYTE(0) が悪さをしているっぽいんだよなあ。


- そもそもなぜこれをいれるのが良いと思われていたのか
- BYTE(0) があることによってなぜおかしくなるのか
- まえはよかったのになんで急におかしくなりはじめたのか
- Debug のときだけそれが起きる原因はなにか？
- Warning の真意はなにか？


BYTE(0) を LONG(0) に置き換えても動いた。

BYTE のほうを、kernel.bad, LONG(0) のほうを kernel.good としてコンパイルした。 (kernel.bad のほうは、EIP=0x100028 で落ちる

```
% readelf -S kernel.bad  

There are 24 section headers, starting at offset 0xa2f30:

 [Nr] Name              Type            Addr     Off    Size   ES Flg Lk Inf Al
 [ 0]                   NULL            00000000 000000 000000 00      0   0  0
 [ 1] .text             PROGBITS        80100000 001000 031dd5 00  AX  0   0 16
 [ 2] .rodata           PROGBITS        80131de0 032de0 014ced 00   A  0   0 16
 [ 3] .gcc_except_table PROGBITS        80146ad0 047ad0 000148 00   A  0   0  4
 [ 4] .debug_gdb_script PROGBITS        80146c18 047c18 000022 01 AMS  0   0  1
 [ 5] .stab             PROGBITS        80146c3a 047c3a 000001 00  WA  0   0  1
 [ 6] .data             PROGBITS        80147000 048000 0076fc 00  WA  0   0 4096
 [ 7] .got              PROGBITS        8014e6fc 04f6fc 000038 00  WA  0   0  4
 [ 8] .got.plt          PROGBITS        8014e734 04f734 00000c 04  WA  0   0  4
 [ 9] .bss              NOBITS          8014e740 04f740 001830 00  WA  0   0 16
 [10] .debug_line       PROGBITS        00000000 04f740 007031 00      0   0  1
 [11] .debug_info       PROGBITS        00000000 056771 00c727 00      0   0  1
 ...
 
% readelf -S kernel.good

There are 24 section headers, starting at offset 0xa2f30:

Section Headers:
 [Nr] Name              Type            Addr     Off    Size   ES Flg Lk Inf Al
 [ 0]                   NULL            00000000 000000 000000 00      0   0  0
 [ 1] .text             PROGBITS        80100000 001000 031dd5 00  AX  0   0 16
 [ 2] .rodata           PROGBITS        80131de0 032de0 014ced 00   A  0   0 16
 [ 3] .gcc_except_table PROGBITS        80146ad0 047ad0 000148 00   A  0   0  4
 [ 4] .debug_gdb_script PROGBITS        80146c18 047c18 000022 01 AMS  0   0  1
 [ 5] .stab             PROGBITS        80146c3a 047c3a 000004 00  WA  0   0  1
 [ 6] .data             PROGBITS        80147000 048000 0076fc 00  WA  0   0 4096
 [ 7] .got              PROGBITS        8014e6fc 04f6fc 000038 00  WA  0   0  4
 [ 8] .got.plt          PROGBITS        8014e734 04f734 00000c 04  WA  0   0  4
 [ 9] .bss              NOBITS          8014e740 04f740 001830 00  WA  0   0 16
 [10] .debug_line       PROGBITS        00000000 04f740 007031 00      0   0  1
 ...
Key to Flags:
  W (write), A (alloc), X (execute), M (merge), S (strings), I (info),
  L (link order), O (extra OS processing required), G (group), T (TLS),
  C (compressed), x (unknown), o (OS specific), E (exclude),
  p (processor specific)
```

`diff <(readelf -S kernel.good) <(readelf -S kernel.bad) ` としても、stab のサイズ以外の違いはなし。
`diff <(objdump -s -j .data kernel.good) <(objdump -s -j .data kernel.bad) ` としても、違いはなし。

```
diff <(readelf -h kernel.good) <(readelf -h kernel.bad)                                                                                   [±master ●●]
17c17
<   Number of program headers:         2
---
>   Number of program headers:         3
```

というのはあやしい。プログラムヘッダの数が違う……？
ELF のプログラムヘッダとはなんだろうか？

それぞれのプログラムヘッダを見てみるとこうだった。

```
% readelf -l kernel.good

Elf file type is EXEC (Executable file)
Entry point 0x10000c
There are 2 program headers, starting at offset 52

Program Headers:
  Type           Offset   VirtAddr   PhysAddr   FileSiz MemSiz  Flg Align
  LOAD           0x001000 0x80100000 0x00100000 0x4e740 0x4ff70 RWE 0x1000
  GNU_STACK      0x000000 0x00000000 0x00000000 0x00000 0x00000 RWE 0x10

 Section to Segment mapping:
  Segment Sections...
   00     .text .rodata .gcc_except_table .debug_gdb_scripts .stab .data .got .got.plt .bss
   01

% readelf -l kernel.bad

Elf file type is EXEC (Executable file)
Entry point 0x10000c
There are 3 program headers, starting at offset 52

Program Headers:
  Type           Offset   VirtAddr   PhysAddr   FileSiz MemSiz  Flg Align
  LOAD           0x001000 0x80100000 0x00100000 0x46c3a 0x46c3a R E 0x1000
  LOAD           0x047c3a 0x80146c3a 0x80146c3a 0x07b06 0x09336 RW  0x1000
  GNU_STACK      0x000000 0x00000000 0x00000000 0x00000 0x00000 RWE 0x10

 Section to Segment mapping:
  Segment Sections...
   00     .text .rodata .gcc_except_table .debug_gdb_scripts
   01     .stab .data .got .got.plt .bss
   02
```

ふむ、どうやら、bootmain で kernel を読み込む部分で、program header を一つしか読んでいないようだ。それで kernel.bad の場合は 2つめのものが読まれず、バグっている。

xv6-public の bootblock をコピペしてきてもバグっているので、もともとそこがバグっていたのかな。

## TIL

- Qemu で、`C-a c` してから、info registers で、その時点のレジスタ情報を見れる。
  - `C-a h` でヘルプ。

