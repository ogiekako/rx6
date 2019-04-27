2019-04-22

Mac で動かそうとしている。http://sairoutine.hatenablog.com/entry/2016/09/02/232318 を参照。
こっちのほうがいい。https://attonblog.blogspot.com/2015/04/32bit-xv6-yosemite.html

"正しく進むことが早く進む唯一の道である。" 自分が理解していないことをやらない。
 なにをやっているのかを理解した上で、着実に進んでいく。というわけで、まずは bootmain について適切になにをやっているのかを書いていくのが最初のステップになるだろう。 unsafe コード使いまくりでただのコピペでは、知性がみられない。→ C を Rust に自動変換するツールなどに時間を使うべきではない。
 目標は、Rust らしい適切な抽象化がされ、unsafe は抽象の下に隠蔽されたコードで、xv6 と同じ機能の OS をつくること。これができれば強いアピールポイントになる。OS の理解という意味と、Rust の理解という意味で。

コードを書きながら、自分の理解を確認するために同時にドキュメントも書いていく。これができないと、途中で中断してから再開するときに非常にコストがかかる。コードを書き始めるまえに独白を書き、試行錯誤をへらす努力をする。
あたまのなかで考えず、なるべく文字におこす。テディベアメソッド。

というわけで、boot process について記述していく。
これは、kernel code を読み込み、kernel のエントリポイントにジャンプする。カーネルが適切に走り出すための準備をする。

そのまえに、実行ファイルがどのようにコンパイルされるかを見ていく。
Makefile にかかれている。

qemu が走らせる fs.img, xv6.img というのがある。この .img というのはなにか？
これは bootimage で、ディスクにこのまま書かれる。10000 のところから、

bootblock の作られ方をみる。

```Makefile
bootblock: bootasm.S $(wildcard bootmain/src/*.rs)
	gcc $(CFLAGS) -fno-pic -nostdinc -I. -c bootasm.S           # 1
	(cd bootmain && cross build --target $(TARGET) --release)   # 2
	$(LD) $(LDFLAGS) -N \
		-e start \
		-Ttext 0x7C00 \
		-o bootblock.o \
		bootasm.o bootmain/target/$(TARGET)/release/libbootmain.a # 3
	$(OBJDUMP) -S bootblock.o > bootblock.asm                   # 4
	$(OBJCOPY) -S -O binary -j .text bootblock.o bootblock      # 5
	./sign.pl bootblock
```

CFLAGS は、

```
CFLAGS = -fno-pic -static -fno-builtin -fno-strict-aliasing -O2 -Wall -MD -ggdb -m32 -Werror -fno-omit-frame-pointer
CFLAGS += $(shell gcc -fno-stack-protector -E -x c /dev/null >/dev/null 2>&1 && echo -fno-stack-protector)
```

CFLAGS もいろいろあるけど、オプションの意味はどうしらべるといいのかな。
[ここ](https://gcc.gnu.org/onlinedocs/gcc/Option-Summary.html) にすべての option がまとまっている。

```
-fno-pic:  position independent code を作成しない。position independent code は共有ライブラリ(どこにロードされるかわからない) をコンパイルする際に使う。
-static: 共有ライブラリとのリンクを防ぐ
-fno-builtin: GCC の組み込み関数をつかわない。
-fno-strict-aliasing: union を使ったコードにおける、コンパイラの最適化バグの可能性を防ぐ。
-m32: 32 bit 用のバイナリを吐く
-fno-omit-frame-pointer: Frame pointer をつかってなくても削除しない(デバッグを難しくしないため).
-fno-stack-protector: バッファオーバーフローを detect するためのコードを吐かない (その前のコマンドは、単にこのオプションが有効か確かめているだけ)

-nostdinc : standard system directory にヘッダファイルを探さない. -I で指定されたところのみでヘッダファイルを探す。
-c : コンパイルだけをして、リンクはしない。
```

PIC とは：共有ライブラリは、特定の予めきまったアドレスにロードできるとは限らない。なので、どの位置にあっても動く（位置独立で）なければならない。そういうコードを PIC とよぶ。PIC について、別の[ページ](docs/pic.md)に書いていく。

まあようするに余計なことをしないで、書いたコードをそのまま executable として出力させるためのオプションのようだ。
Makefile の一行目は、bootasm.S を余計な最適化とか、ライブラリのリンクなしにコンパイルする。

bootblock `# 2` は、Rust で書かれた bootmain を x86 用にビルドして、スタティックライブラリを作っている。
static library とは、単に object files の集まりであって、静的にリンクされる。つまり、結果としてできる executable は、静的ライブラリの内容を含んでいて、サイズとしては dynamic library を使っているものよりも大きくなる。しかし、単一の executable で完結するので、今回の場合はこれが適している。

`# 3` はファイルをリンクしている。再掲する。 `#1` で作られたA bootasm.o と、先のライブラリをリンクして、bootblock.o を作っている。

```
	$(LD) $(LDFLAGS) -N \
		-e start \
		-Ttext 0x7C00 \
		-o bootblock.o \
		bootasm.o bootmain/target/$(TARGET)/release/libbootmain.a
```

オプションの意味 ([ここ](https://sourceware.org/binutils/docs/ld/Options.html))を参照した)

```
-e start : 実行を開始する関数名
-Ttext 0x7C00 : text section を 0x7C00 からにする。.text というのは実行プログラムを格納している領域。
-o : 出力ファイル名
```

x86 は、0x7C00 から実行を開始するため、その位置に text section をセットしている。

`# 4` はデバッグ用にアセンブリを出力、`# 5` は、objcopyの[オプション](https://sourceware.org/binutils/docs/binutils/objcopy.html)

```
	$(OBJCOPY) -S -O binary -j .text bootblock.o bootblock      # 5
```

```
-S : Do not copy relocation and symbol information from the source file. つまり、純粋にプログラム部分だけを取り出す
-O binary : binary format で出力する。
-j .text : .text section のみをコピーする
```

すべてを理解しようとしない。一番上に書いた大目標を忘れない。説明できるのは大事だけど、すべてを完璧にするのは無理。OS の根幹として大事なところをおさえる。深入りしすぎるのもよくない。難しいところからやる。ざっくり途中からみるのも大事。

```
	./sign.pl bootblock  # 6
```
512 バイトの最後のバイトが aa55 だと起動ディスクとして認識される。そのためのアップデートを sign.pl がやっている。


TODO: gdb でよくつかうコマンドについてまとめる。
- memory を見る

rx6.img は、以下で作られる。

```
rx6.img: bootblock kernel
	dd if=/dev/zero of=rx6.img count=10000
	dd if=bootblock of=rx6.img conv=notrunc
	dd if=kernel of=rx6.img seek=1 conv=notrunc
```

notrunc - Do not truncate the output file.
seek=1  - Seek 1 block from the beginning of the output before copying.

block とは、

bootblock をアドレス 10000 においている。

できれば、適切な抽象化をして、unsafe をなるべくすくなくしていきたい。
Kernel thread が以下のことができるようなコンテナを作りたい。

まず、多くのものは linked list で管理されている。
以下のためのコンテナを作りたい。
Box - allocation

データ構造を理解すれば、自ずとそれらの関係の理解につながるから、まずは主要なデータ構造についてまとめていく。


## 16:09

さっそくバグっている。

0x0010000c には到達するけど、kernmain にはたどり着かない。
0x10000c に行ったあと、0x7c00 にもどっている。(再起動)
あと step 実行ができない。s の代わりに si でよかった。

```
=> 0x100028:    Error while running hook_stop:
Cannot access memory at address 0x100028
0x00100028 in ?? ()
(gdb) si
The target architecture is assumed to be i8086
[f000:e05b]    0xfe05b: cmpw   $0xffc8,%cs:(%esi)
0x0000e05b in ?? ()
(gdb)
```

## 仮想メモリ

## ページ

- 

## プロセス



