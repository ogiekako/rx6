# How to translate C to Rust

- すべてが unsafe
- すべてが public
  - `fn` ではなく、`pub unsafe fn` と書きましょう
  - lib.rs, foo.rs, bar.rs ... がある場合、`lib.rs` に `pub use foo::*; pub use bar::*;` などと書き、foo.rs では、`use super::*;` とすると良いでしょう。
- 初期値は、`core::mem::zeroed()`
  - memset が必要。compiler-builtins crate を使う。
  -る compiler_builtin::mem::memset が `#[no_std]` の場合は、提供されている。cargo xbuild をつかっていれば、対象ターゲット用に、コンパイルされるはず。
  - static int とかは、pub でなくてよい。
- 初期値つき static 変数は `lazy_static!`
  - stack が消費されることに注意。サイズが大きいものは要注意。
  - core::mem::zeroed() が const fn でないのが annoying.
- lifetime は `&'static`
- アセンブラは、`asm!`
- グローバル変数は static mut
  - const fn バージョンの core::mem::zeroed()
  - (C言語では、静的変数は 0 初期化される)
- 基本的に、静的なデータ構造におけるポインタは、raw pointer に翻訳し、reference は使わないのがよいでしょう。
  - 初期化されていないばあいなど、予期せぬ UB が発生する危険。具体例？

- 初期化されていないリファレンスには、MaybeUninit を使う

- pointer は ptr
- pointer 型 `*u8` の使用
  - p.offset
  - ptr::copy() の使用
  - See [`core::ptr`].

- #[repr(C)] を使う.
  - 構造体に対して offset でアクセスしているコードを正しく動かすには、#[repr(C)] をつけましょう。

[`core::ptr`]: https://doc.rust-lang.org/core/ptr/index.html

- ownership エラーは `core::mem::transmute()`, `core::mem::transmute_copy で回避` - これはなるべく避けたい。(型自体を変換する必要があることはそんなにないはず……)

- 暗黙の型変換用に into() を定義する
