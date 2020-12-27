# raptorq
[![Build Status](https://travis-ci.com/cberner/raptorq.svg?branch=master)](https://travis-ci.com/cberner/raptorq)
[![Crates](https://img.shields.io/crates/v/raptorq.svg)](https://crates.io/crates/raptorq)
[![Documentation](https://docs.rs/raptorq/badge.svg)](https://docs.rs/raptorq)
[![PyPI](https://img.shields.io/pypi/v/raptorq.svg)](https://pypi.org/project/raptorq/)
[![dependency status](https://deps.rs/repo/github/cberner/raptorq/status.svg)](https://deps.rs/repo/github/cberner/raptorq)

Rust implementation of RaptorQ (RFC6330)

Recovery properties:
Reconstruction probability after receiving K + h packets = 1 - 1/256^(h + 1). Where K is the number of packets in the
original message, and h is the number of additional packets received.
See "RaptorQ Technical Overview" by Qualcomm

This crate requires Rust 1.46 or newer.

### Examples
See the `examples/` directory for usage.

### Benchmarks

The following were run on an Intel Core i5-6600K @ 3.50GHz

```
Symbol size: 1280 bytes (without pre-built plan)
symbol count = 10, encoded 127 MB in 0.478secs, throughput: 2142.1Mbit/s
symbol count = 100, encoded 127 MB in 0.443secs, throughput: 2310.2Mbit/s
symbol count = 250, encoded 127 MB in 0.444secs, throughput: 2303.9Mbit/s
symbol count = 500, encoded 127 MB in 0.428secs, throughput: 2384.4Mbit/s
symbol count = 1000, encoded 126 MB in 0.459secs, throughput: 2212.7Mbit/s
symbol count = 2000, encoded 126 MB in 0.487secs, throughput: 2085.5Mbit/s
symbol count = 5000, encoded 122 MB in 0.562secs, throughput: 1737.7Mbit/s
symbol count = 10000, encoded 122 MB in 0.637secs, throughput: 1533.1Mbit/s
symbol count = 20000, encoded 122 MB in 0.868secs, throughput: 1125.1Mbit/s
symbol count = 50000, encoded 122 MB in 1.236secs, throughput: 790.1Mbit/s

Symbol size: 1280 bytes (with pre-built plan)
symbol count = 10, encoded 127 MB in 0.226secs, throughput: 4530.6Mbit/s
symbol count = 100, encoded 127 MB in 0.150secs, throughput: 6822.9Mbit/s
symbol count = 250, encoded 127 MB in 0.165secs, throughput: 6199.7Mbit/s
symbol count = 500, encoded 127 MB in 0.168secs, throughput: 6074.5Mbit/s
symbol count = 1000, encoded 126 MB in 0.184secs, throughput: 5519.7Mbit/s
symbol count = 2000, encoded 126 MB in 0.200secs, throughput: 5078.1Mbit/s
symbol count = 5000, encoded 122 MB in 0.249secs, throughput: 3921.9Mbit/s
symbol count = 10000, encoded 122 MB in 0.339secs, throughput: 2880.7Mbit/s
symbol count = 20000, encoded 122 MB in 0.426secs, throughput: 2292.4Mbit/s
symbol count = 50000, encoded 122 MB in 0.585secs, throughput: 1669.3Mbit/s

Symbol size: 1280 bytes
symbol count = 10, decoded 127 MB in 0.671secs using 0.0% overhead, throughput: 1526.0Mbit/s
symbol count = 100, decoded 127 MB in 0.526secs using 0.0% overhead, throughput: 1945.7Mbit/s
symbol count = 250, decoded 127 MB in 0.526secs using 0.0% overhead, throughput: 1944.8Mbit/s
symbol count = 500, decoded 127 MB in 0.504secs using 0.0% overhead, throughput: 2024.8Mbit/s
symbol count = 1000, decoded 126 MB in 0.517secs using 0.0% overhead, throughput: 1964.5Mbit/s
symbol count = 2000, decoded 126 MB in 0.575secs using 0.0% overhead, throughput: 1766.3Mbit/s
symbol count = 5000, decoded 122 MB in 0.638secs using 0.0% overhead, throughput: 1530.7Mbit/s
symbol count = 10000, decoded 122 MB in 0.784secs using 0.0% overhead, throughput: 1245.6Mbit/s
symbol count = 20000, decoded 122 MB in 0.987secs using 0.0% overhead, throughput: 989.4Mbit/s
symbol count = 50000, decoded 122 MB in 1.472secs using 0.0% overhead, throughput: 663.4Mbit/s

symbol count = 10, decoded 127 MB in 0.654secs using 5.0% overhead, throughput: 1565.6Mbit/s
symbol count = 100, decoded 127 MB in 0.537secs using 5.0% overhead, throughput: 1905.8Mbit/s
symbol count = 250, decoded 127 MB in 0.540secs using 5.0% overhead, throughput: 1894.4Mbit/s
symbol count = 500, decoded 127 MB in 0.509secs using 5.0% overhead, throughput: 2004.9Mbit/s
symbol count = 1000, decoded 126 MB in 0.543secs using 5.0% overhead, throughput: 1870.4Mbit/s
symbol count = 2000, decoded 126 MB in 0.575secs using 5.0% overhead, throughput: 1766.3Mbit/s
symbol count = 5000, decoded 122 MB in 0.665secs using 5.0% overhead, throughput: 1468.5Mbit/s
symbol count = 10000, decoded 122 MB in 0.830secs using 5.0% overhead, throughput: 1176.6Mbit/s
symbol count = 20000, decoded 122 MB in 1.048secs using 5.0% overhead, throughput: 931.8Mbit/s
symbol count = 50000, decoded 122 MB in 1.740secs using 5.0% overhead, throughput: 561.2Mbit/s
```

### Public API
Note that the additional classes exported by the `benchmarking` feature flag are not considered part of this
crate's public API. Breaking changes to those classes may occur without warning. The flag is only provided
so that internal classes can be used in this crate's benchmarks.

## Python bindings

The Python bindings are generated using [pyo3](https://github.com/PyO3/pyo3). 

Some operating systems require additional packages to be installed.
```
$ sudo apt install python3-dev
```

[maturin](https://github.com/PyO3/maturin) is recommended for building the Python bindings in this crate.
```
$ pip install maturin
$ maturin build --cargo-extra-args="--features python"
```

Alternatively, refer to the [Building and Distribution section](https://pyo3.rs/v0.8.5/building_and_distribution.html) in the [pyo3 user guide](https://pyo3.rs/v0.8.5/).
Note, you must pass the `--cargo-extra-args="--features python"` argument to Maturin when building this crate
to enable the Python binding features.

## License

Licensed under

 * Apache License, Version 2.0 ([LICENSE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be licensed as above, without any
additional terms or conditions.
