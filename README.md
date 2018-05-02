# Very Fast HTTP Paser - Totemo-Hayai (とても速い) HTTP Parser

[![Build Status](https://travis-ci.org/kei10in/thhp.svg?branch=master)](https://travis-ci.org/kei10in/thhp)


## Benchmark

### With SSE4.2

  ```
  $ RUSTFLAGS="-C target-feature=+sse4.2" rustup run nightly cargo bench --features=nightly
  ...
  test bench_httparse             ... bench:         403 ns/iter (+/- 21) = 1744 MB/s
  test bench_httparse_short       ... bench:          43 ns/iter (+/- 4) = 1302 MB/s
  test bench_picohttpparser       ... bench:         205 ns/iter (+/- 11) = 3429 MB/s
  test bench_picohttpparser_short ... bench:          54 ns/iter (+/- 2) = 1037 MB/s
  test bench_thhp                 ... bench:         229 ns/iter (+/- 10) = 3069 MB/s
  test bench_thhp_short           ... bench:          49 ns/iter (+/- 2) = 1142 MB/s
  ...
  ```

### Without SIMD

  ```
  $ rustup run nightly cargo bench
  ...
  test bench_httparse             ... bench:         405 ns/iter (+/- 56) = 1735 MB/s
  test bench_httparse_short       ... bench:          42 ns/iter (+/- 4) = 1333 MB/s
  test bench_picohttpparser       ... bench:         333 ns/iter (+/- 31) = 2111 MB/s
  test bench_picohttpparser_short ... bench:          46 ns/iter (+/- 4) = 1217 MB/s
  test bench_thhp                 ... bench:         339 ns/iter (+/- 20) = 2073 MB/s
  test bench_thhp_short           ... bench:          55 ns/iter (+/- 2) = 1018 MB/s
  ...
  ```
