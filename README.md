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
symbol count = 10, encoded 127 MB in 0.537secs, throughput: 1906.8Mbit/s
symbol count = 100, encoded 127 MB in 0.590secs, throughput: 1734.6Mbit/s
symbol count = 250, encoded 127 MB in 0.572secs, throughput: 1788.4Mbit/s
symbol count = 500, encoded 127 MB in 0.554secs, throughput: 1842.1Mbit/s
symbol count = 1000, encoded 126 MB in 0.583secs, throughput: 1742.1Mbit/s
symbol count = 2000, encoded 126 MB in 0.657secs, throughput: 1545.9Mbit/s
symbol count = 5000, encoded 122 MB in 0.725secs, throughput: 1347.0Mbit/s
symbol count = 10000, encoded 122 MB in 0.915secs, throughput: 1067.3Mbit/s
symbol count = 20000, encoded 122 MB in 1.335secs, throughput: 731.5Mbit/s
symbol count = 50000, encoded 122 MB in 1.990secs, throughput: 490.7Mbit/s

Symbol size: 1280 bytes
symbol count = 10, decoded 127 MB in 0.728secs using 0.0% overhead, throughput: 1406.5Mbit/s
symbol count = 100, decoded 127 MB in 0.699secs using 0.0% overhead, throughput: 1464.1Mbit/s
symbol count = 250, decoded 127 MB in 0.639secs using 0.0% overhead, throughput: 1600.9Mbit/s
symbol count = 500, decoded 127 MB in 0.643secs using 0.0% overhead, throughput: 1587.1Mbit/s
symbol count = 1000, decoded 126 MB in 0.674secs using 0.0% overhead, throughput: 1506.9Mbit/s
symbol count = 2000, decoded 126 MB in 0.749secs using 0.0% overhead, throughput: 1356.0Mbit/s
symbol count = 5000, decoded 122 MB in 0.877secs using 0.0% overhead, throughput: 1113.5Mbit/s
symbol count = 10000, decoded 122 MB in 1.182secs using 0.0% overhead, throughput: 826.2Mbit/s
symbol count = 20000, decoded 122 MB in 1.528secs using 0.0% overhead, throughput: 639.1Mbit/s
symbol count = 50000, decoded 122 MB in 2.596secs using 0.0% overhead, throughput: 376.2Mbit/s

symbol count = 10, decoded 127 MB in 0.725secs using 5.0% overhead, throughput: 1412.3Mbit/s
symbol count = 100, decoded 127 MB in 0.702secs using 5.0% overhead, throughput: 1457.9Mbit/s
symbol count = 250, decoded 127 MB in 0.626secs using 5.0% overhead, throughput: 1634.1Mbit/s
symbol count = 500, decoded 127 MB in 0.620secs using 5.0% overhead, throughput: 1646.0Mbit/s
symbol count = 1000, decoded 126 MB in 0.634secs using 5.0% overhead, throughput: 1601.9Mbit/s
symbol count = 2000, decoded 126 MB in 0.685secs using 5.0% overhead, throughput: 1482.7Mbit/s
symbol count = 5000, decoded 122 MB in 0.830secs using 5.0% overhead, throughput: 1176.6Mbit/s
symbol count = 10000, decoded 122 MB in 1.059secs using 5.0% overhead, throughput: 922.2Mbit/s
symbol count = 20000, decoded 122 MB in 1.370secs using 5.0% overhead, throughput: 712.8Mbit/s
symbol count = 50000, decoded 122 MB in 2.348secs using 5.0% overhead, throughput: 415.9Mbit/s
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
