# raptorq
[![Build Status](https://travis-ci.com/cberner/raptorq.svg?branch=master)](https://travis-ci.com/cberner/raptorq)
[![Crates](https://img.shields.io/crates/v/raptorq.svg)](https://crates.io/crates/raptorq)
[![Documentation](https://docs.rs/raptorq/badge.svg)](https://docs.rs/raptorq)
[![dependency status](https://deps.rs/repo/github/cberner/raptorq/status.svg)](https://deps.rs/repo/github/cberner/raptorq)

Rust implementation of RaptorQ (RFC6330)

Recovery properties:
Reconstruction probability after receiving K + h packets = 1 - 1/256^(h + 1). Where K is the number of packets in the
original message, and h is the number of additional packets received.
See "RaptorQ Technical Overview" by Qualcomm

### Examples
See the `examples/` directory for usage.

### Benchmarks

The following were run on an Intel Core i5-6600K @ 3.50GHz

```
Symbol size: 1280 bytes
symbol count = 10, encoded 127 MB in 0.494secs, throughput: 2072.7Mbit/s
symbol count = 100, encoded 127 MB in 0.554secs, throughput: 1847.4Mbit/s
symbol count = 250, encoded 127 MB in 0.859secs, throughput: 1190.9Mbit/s
symbol count = 500, encoded 127 MB in 0.841secs, throughput: 1213.4Mbit/s
symbol count = 1000, encoded 126 MB in 0.949secs, throughput: 1070.2Mbit/s
symbol count = 2000, encoded 126 MB in 1.119secs, throughput: 907.6Mbit/s
symbol count = 5000, encoded 122 MB in 1.327secs, throughput: 735.9Mbit/s
symbol count = 10000, encoded 122 MB in 1.749secs, throughput: 558.4Mbit/s
symbol count = 20000, encoded 122 MB in 2.784secs, throughput: 350.8Mbit/s
symbol count = 50000, encoded 122 MB in 4.381secs, throughput: 222.9Mbit/s

Symbol size: 1280 bytes
symbol count = 10, decoded 127 MB in 0.687secs using 0.0% overhead, throughput: 1490.4Mbit/s
symbol count = 100, decoded 127 MB in 0.705secs using 0.0% overhead, throughput: 1451.7Mbit/s
symbol count = 250, decoded 127 MB in 0.928secs using 0.0% overhead, throughput: 1102.3Mbit/s
symbol count = 500, decoded 127 MB in 0.969secs using 0.0% overhead, throughput: 1053.2Mbit/s
symbol count = 1000, decoded 126 MB in 1.108secs using 0.0% overhead, throughput: 916.6Mbit/s
symbol count = 2000, decoded 126 MB in 1.286secs using 0.0% overhead, throughput: 789.8Mbit/s
symbol count = 5000, decoded 122 MB in 1.601secs using 0.0% overhead, throughput: 610.0Mbit/s
symbol count = 10000, decoded 122 MB in 2.169secs using 0.0% overhead, throughput: 450.2Mbit/s
symbol count = 20000, decoded 122 MB in 2.945secs using 0.0% overhead, throughput: 331.6Mbit/s
symbol count = 50000, decoded 122 MB in 5.602secs using 0.0% overhead, throughput: 174.3Mbit/s

symbol count = 10, decoded 127 MB in 0.684secs using 5.0% overhead, throughput: 1497.0Mbit/s
symbol count = 100, decoded 127 MB in 0.704secs using 5.0% overhead, throughput: 1453.7Mbit/s
symbol count = 250, decoded 127 MB in 0.906secs using 5.0% overhead, throughput: 1129.1Mbit/s
symbol count = 500, decoded 127 MB in 0.920secs using 5.0% overhead, throughput: 1109.2Mbit/s
symbol count = 1000, decoded 126 MB in 1.108secs using 5.0% overhead, throughput: 916.6Mbit/s
symbol count = 2000, decoded 126 MB in 1.275secs using 5.0% overhead, throughput: 796.6Mbit/s
symbol count = 5000, decoded 122 MB in 1.508secs using 5.0% overhead, throughput: 647.6Mbit/s
symbol count = 10000, decoded 122 MB in 2.006secs using 5.0% overhead, throughput: 486.8Mbit/s
symbol count = 20000, decoded 122 MB in 2.729secs using 5.0% overhead, throughput: 357.8Mbit/s
symbol count = 50000, decoded 122 MB in 4.498secs using 5.0% overhead, throughput: 217.1Mbit/s
```

### Public API
Note that the additional classes exported by the `benchmarking` feature flag are not considered part of this
crate's public API. Breaking changes to those classes may occur without warning. The flag is only provided
so that internal classes can be used in this crate's benchmarks.

## License

Licensed under

 * Apache License, Version 2.0 ([LICENSE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be licensed as above, without any
additional terms or conditions.
