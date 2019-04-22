# PIC

ここでは、PIC とはなにか、なぜ必要か、その実装について書く。
PIC について：

基本的に、executable は決まったアドレスにロードされる。その実行は、readelf で確認できる、Entry point address から開始される。
Executable が、外部ライブラリを使わないならばそれでなんの問題もない。

ライブラリのコンパイルにおいては、あらかじめそれがどこに置かれるかはわからない。それを決めてしまっていたら、複数ライブラリを使うプログラムにおいて、アドレスのかぶりが生じてしまう。

そこで、ライブラリをコンパイルするときには、すべてのアドレスを決めることはせずに、スタブとしてのこしておき、ロード時にその空白を埋めてリンクするということが行われる。以下のコードにおける `g` のアドレスなどがどうである。g が実際にどこにおかれるかはロードされるまでわからない。こういう loader を dynamic loader と呼ぶ。Linux の ld は dynamic linker/loader である。
ld は実行時に必要なライブラリを適切な書き換えをおこないつつロードするのである。

```
int g = 42;
int f() {
  return g++;
}
```

共有ライブラリを、アドレスを決めずにコンパイルする手法には２つある。
1. Load-time relocation
2. Position independent code

1 のアプローチについてまず説明する。
-fPIC なしでコンパイルする。

TODO: PIC についてつづきを書く

## References

[Load-time relocation of shared libraries - Eli Bendersky's website](https://eli.thegreenplace.net/2011/08/25/load-time-relocation-of-shared-libraries/)
[Position Independent Code (PIC) in shared libraries - Eli Bendersky's website](https://eli.thegreenplace.net/2011/11/03/position-independent-code-pic-in-shared-libraries/)

