# Very Fast HTTP Paser - Totemo-Hayai (とても速い) HTTP Parser

[![Build Status](https://travis-ci.org/kei10in/thhp.svg?branch=master)](https://travis-ci.org/kei10in/thhp)
[![codecov](https://codecov.io/gh/kei10in/thhp/branch/master/graph/badge.svg)](https://codecov.io/gh/kei10in/thhp)

## Usage

  ```rust
  let buf = b"GET / HTTP/1.1\r\nHost: example.com";
  let mut headers = Vec::<thhp::HeaderField>::with_capacity(16);
  match thhp::Request::parse(buf, &mut headers) {
     Ok(thhp::Complete((ref req, len))) => {
         // Use request.
     },
     Ok(thhp::Incomplete) => {
         // Read more and parse again.
     },
     Err(err) => {
         // Handle error.
     }
  }
  ```


## Benchmark

### With SSE4.2

  ```
  $ RUSTFLAGS="-C target-feature=+sse4.2" rustup run nightly cargo bench --features=nightly
  ...
  test bench_httparse             ... bench:         404 ns/iter (+/- 28) = 1740 MB/s
  test bench_httparse_short       ... bench:          44 ns/iter (+/- 1) = 1272 MB/s
  test bench_picohttpparser       ... bench:         199 ns/iter (+/- 7) = 3532 MB/s
  test bench_picohttpparser_short ... bench:          57 ns/iter (+/- 2) = 982 MB/s
  test bench_thhp                 ... bench:         219 ns/iter (+/- 18) = 3210 MB/s
  test bench_thhp_short           ... bench:          41 ns/iter (+/- 3) = 1365 MB/s
  ...
  ```

### Without SIMD

  ```
  $ rustup run nightly cargo bench
  ...
  test bench_httparse             ... bench:         407 ns/iter (+/- 50) = 1727 MB/s
  test bench_httparse_short       ... bench:          43 ns/iter (+/- 7) = 1302 MB/s
  test bench_picohttpparser       ... bench:         321 ns/iter (+/- 31) = 2190 MB/s
  test bench_picohttpparser_short ... bench:          46 ns/iter (+/- 4) = 1217 MB/s
  test bench_thhp                 ... bench:         308 ns/iter (+/- 21) = 2282 MB/s
  test bench_thhp_short           ... bench:          43 ns/iter (+/- 2) = 1302 MB/s
  ...
  ```
