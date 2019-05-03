# Xv6

xv6 specific なメモ

uint -> u32
int  -> i32

# How to translate C to Rust


- すべてが unsafe
- すべてが public
  - `fn` ではなく、`pub unsafe fn` と書きましょう
  - lib.rs, foo.rs, bar.rs ... がある場合、`lib.rs` に `pub use foo::*; pub use bar::*;` などと書き、foo.rs では、`use super::*;` とすると良いでしょう。
  - (static int とかは、pub でなくてよい。)
- 初期値は、`core::mem::zeroed()`
  - Rust nomicon. Drop trait の場合は危険。
- 初期値つき static 変数は `lazy_static!`
  - stack が消費されることに注意。サイズが大きいものは要注意。
  - core::mem::zeroed() が const fn でないのが annoying.
- lifetime は `&'static`
- アセンブラは、`asm!`
  - だいたい GCC 拡張と一緒。最後に volatile をつける。%0 -> $0
  - unstable book の asm の項を参照。
- グローバル変数は static mut
  - const fn バージョンの core::mem::zeroed()
  - (C言語では、静的変数は 0 初期化される)
- 基本的に、静的なデータ構造におけるポインタは、raw pointer に翻訳し、reference は使わないのがよいでしょう。
  - 初期化されていないばあいなど、予期せぬ UB, Drop が発生する危険。具体例？
  - cast が発生しまくるが割り切る。

- `__sync_synchronize` は、`core::sync::atomic::fence(SeqCst)` .

- function pointer は、`&(mpenter as unsafe fn()) as *const unsafe fn() as usize` のようにすれば作れる。

- static mut の初期化は feature const_transmute を有効化して、`core::mem::transmute([0u8; size_of<T>()])`.

- extern ?




- pointer は ptr
- pointer 型 `*u8` の使用
  - p.offset
  - ptr::copy() の使用
  - See [`core::ptr`].
  - p = `*u32` の場合、p[1] は、`*(p.offset(1))`
  - 0 との比較は、`ptr::null_mut()` に置き換え
  - 関数ポインタ. fn foo() のポインタをとるには、foo as *const fn();  でよい。
  - 配列 a = [0; 2] へのポインタは、C では、`a` だが、Rust では、`&a` .

- #[repr(C)] を使う.
  - 構造体に対して offset でアクセスしているコードを正しく動かすには、#[repr(C)] をつけましょう。

[`core::ptr`]: https://doc.rust-lang.org/core/ptr/index.html

- ownership エラーは `core::mem::transmute()`, `core::mem::transmute_copy で回避` - これはなるべく避けたい。(型自体を変換する必要があることはそんなにないはず……)

- 暗黙の型変換用に into() を定義する

- 文字列の扱い。Rust の str は null terminate になっているとは限らない。
    - s="hoge"; t="piyo" みたいにやって、strings すると hogepiyo と出る
    - "hoge\0".as_ptr() とするとよい。
- 予約語 (type, ref, yield など) type_, ref_ として対処した
