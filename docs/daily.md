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


## TIL

- Qemu で、`C-a c` してから、info registers で、その時点のレジスタ情報を見れる。
  - `C-a h` でヘルプ。
