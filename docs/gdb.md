gdb の便利コマンド集

よく使いそう

- backtrace
- break ... if cond
- tbreak - one stop break
- watch expr [thread num]
    - e.g. `watch foo` : foo の変化で break

- list  : 対応する source code 表示
- set listsize count

- disas

- p/x  (print values in hex)


メモ

- rbreak regex
- handle _signal_
  - handle SIGTRAP nostop  など。

References

- [Debugging with GDB] https://www.eecs.umich.edu/courses/eecs373/readings/Debugger.pdf 



QEMU

- info pg    -- show the page table
- info mtree -- show memory tree
- info mem   -- show the active virtual memory mappings
- info cpu   -- show infosfor each CPU
