# Very Fast HTTP Paser - Totemo-Hayai (とても速い) HTTP Parser

[![crates.io](https://img.shields.io/crates/v/finchers.svg)](https://crates.io/crates/thhp)
[![Build Status](https://travis-ci.org/kei10in/thhp.svg?branch=master)](https://travis-ci.org/kei10in/thhp)
[![codecov](https://codecov.io/gh/kei10in/thhp/branch/master/graph/badge.svg)](https://codecov.io/gh/kei10in/thhp)
[![Released API docs](https://docs.rs/thhp/badge.svg)](https://docs.rs/thhp)

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

### Use SSE4.2 explicitly

```
$ RUSTFLAGS="-C target-feature=+sse4.2" cargo +nightly bench
...
test bench_httparse             ... bench:         279 ns/iter (+/- 18) = 2519 MB/s
test bench_httparse_short       ... bench:          41 ns/iter (+/- 9) = 1365 MB/s
test bench_picohttpparser       ... bench:         160 ns/iter (+/- 37) = 4393 MB/s
test bench_picohttpparser_short ... bench:          45 ns/iter (+/- 5) = 1244 MB/s
test bench_thhp                 ... bench:         159 ns/iter (+/- 23) = 4421 MB/s
test bench_thhp_short           ... bench:          35 ns/iter (+/- 5) = 1600 MB/s
...
```

### Use SSE4.2 implicitly

```
$ rustup run nightly cargo bench
...
test bench_httparse             ... bench:         208 ns/iter (+/- 26) = 3379 MB/s
test bench_httparse_short       ... bench:          41 ns/iter (+/- 4) = 1365 MB/s
test bench_picohttpparser       ... bench:         153 ns/iter (+/- 24) = 4594 MB/s
test bench_picohttpparser_short ... bench:          42 ns/iter (+/- 7) = 1333 MB/s
test bench_thhp                 ... bench:         198 ns/iter (+/- 33) = 3550 MB/s
test bench_thhp_short           ... bench:          37 ns/iter (+/- 8) = 1513 MB/s
...
```
