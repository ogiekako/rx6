# How to translate C to Rust

- すべてが unsafe
- すべてが public
  - `fn` ではなく、`pub unsafe fn` と書きましょう
  - lib.rs, foo.rs, bar.rs ... がある場合、`lib.rs` に `pub use foo::*; pub use bar::*;` などと書き、foo.rs では、`use super::*;` とすると良いでしょう。
- 初期値は、`core::mem::zeroed()`
- 初期値つき static 変数は `lazy_static!`
- lifetime は `&'static`
- アセンブラは、`asm!`

- pointer は ptr
- pointer 型 `*u8` の使用
  - p.offset
  - ptr::copy() の使用
  - See [`core::ptr`].

[`core::ptr`]: https://doc.rust-lang.org/core/ptr/index.html

- ownership エラーは `core::mem::transmute()`, `core::mem::transmute_copy で回避` - これはなるべく避けたい。(型自体を変換する必要があることはそんなにないはず……)
