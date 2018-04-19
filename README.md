# Very Fast HTTP Paser - Totemo-Hayai (とても速い) HTTP Parser

[![Build Status](https://travis-ci.org/kei10in/thhp.svg?branch=master)](https://travis-ci.org/kei10in/thhp)


## Benchmark

  ```
  $ RUSTFLAGS="-C target-feature=+sse4.2" rustup run nightly cargo bench --features=nightly
  ...
  test bench_httparse             ... bench:         403 ns/iter (+/- 9) = 1744 MB/s
  test bench_httparse_short       ... bench:          44 ns/iter (+/- 0) = 1272 MB/s
  test bench_picohttpparser       ... bench:         203 ns/iter (+/- 14) = 3463 MB/s
  test bench_picohttpparser_short ... bench:          51 ns/iter (+/- 2) = 1098 MB/s
  test bench_thhp                 ... bench:         242 ns/iter (+/- 6) = 2904 MB/s
  test bench_thhp_short           ... bench:          62 ns/iter (+/- 4) = 903 MB/s
  ...
  ```
