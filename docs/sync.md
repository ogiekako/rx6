C言語のvolatile を翻訳しようとしている。まずは、C言語の volatile がどういう意味なのかを把握しよう。volatile - 複数スレッドからアクセスされる可能性のある変数につける。その変数はいつでも変化しうるとコンパイラに伝える。

Rust の asm! の仕様、一回調べて、x86.rs を一気に書き換えたいな。

Rust nomicon の、Uninitialized memory の章を読んでいる。Drop すべきかどうかの情報は、runtime に track され(る場合もあり)、その情報は stack に置かれる。

asm! の使い方は、GCC の asm と似ている。
%0 を $0 に変える、volatile の場合は、最後に "volatile" とつける、以外は変更なしかな。

`__sync_synchronize` は、メモリバリア (full memory barrier) を挟む。任意のメモリオペランドは、この境界をまたいで入れ替わらない。
これは、GCC の機能で、[Built-in functions for atomic memory access](http://gcc.gnu.org/onlinedocs/gcc-4.6.2/gcc/Atomic-Builtins.html)に書いてある。

const transmute は実は feature としてはあるのか…… file:///home/oka/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/share/doc/rust/html/unstable-book/language-features/const-transmute.html

  spinlock の `__sync_synchronize();` を翻訳しようとしている。おそらく、core::sync::atomic::fence を使えばよい。
まず、そもそもここでどうして atomic fence が必要なのかを理解する。その次に、fench に与える Ordering それぞれの意味を理解し、
`__sync_synchronize` が対応するものを判断する。

`__sync_synchronize` が保証しているのは、lock を取る前に、読み取りが発生しないこと。これがないと、コンパイラは、lock を取る前に、それが保護しているものを

Linux kernel memory barrier についての文章が見つかった。https://www.kernel.org/doc/Documentation/memory-barriers.txt

https://en.wikipedia.org/wiki/Memory_barrier をまずは読もう。 -> 読んだ。

基本的に、コンパイラは、single thread を仮定した optimize を自由にできる。つまり、無関係に見える２つの statement を入れ替えてよい。しかし、これは、lock の文脈では問題である。lock を取ることと、それによって守られる領域の使用は、逆順になってしまうと、ciritcal section の invariant が守られなくなってしまう。よって、lock の取得の直後には、memory fence が必要である。(fence と barrier はおそらく同じ意味)

Rust において、full memory barrier と対応するのは、fence ではなく、`compiler_fence` かとも思ったが、fence は、compiler_fence よりも強く、CPU reordering も防ぐので、やはり fence が適切かもしれない。 `compiler_fence is generally only useful for preventing a thread from racing with itself.` とあるので、`compiler_fence` が使用されるべき状況は限られている。今回は fence が適している。

fence に与える ordering として以下がある。Release, Acquire, AcqRel, SeqCst. これは、LLVM の [Atomics.html](https://llvm.org/docs/Atomics.html#release) に対応しており、これによると、GCC の `__sync_*` は、SeqCst に対応する。

Release, Acquire については、この解説がわかりやすそうだ。https://yamasa.hatenablog.jp/entry/20090816/1250446250
TODO: 理解する。しかし、結果的には常に SeqCst を使うのが一番安全ぽい。

