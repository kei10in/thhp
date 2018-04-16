# Very Fast HTTP Paser - Totemo-Hayai (とても速い) HTTP Parser

[![Build Status](https://travis-ci.org/kei10in/thhp.svg?branch=master)](https://travis-ci.org/kei10in/thhp)


## Benchmark

  ```
  $ RUSTFLAGS="-C target-feature=+sse4.2" rustup run nightly cargo bench --features=nightly
  ...
  test bench_httparse       ... bench:         373 ns/iter (+/- 26) = 1884 MB/s
  test bench_picohttpparser ... bench:         199 ns/iter (+/- 9) = 3532 MB/s
  test bench_thhp           ... bench:         195 ns/iter (+/- 22) = 3605 MB/s
  ...
  ```
