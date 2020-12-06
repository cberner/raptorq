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
symbol count = 10, encoded 127 MB in 0.529secs, throughput: 1935.6Mbit/s
symbol count = 100, encoded 127 MB in 0.584secs, throughput: 1752.5Mbit/s
symbol count = 250, encoded 127 MB in 0.499secs, throughput: 2050.0Mbit/s
symbol count = 500, encoded 127 MB in 0.472secs, throughput: 2162.1Mbit/s
symbol count = 1000, encoded 126 MB in 0.525secs, throughput: 1934.5Mbit/s
symbol count = 2000, encoded 126 MB in 0.594secs, throughput: 1709.8Mbit/s
symbol count = 5000, encoded 122 MB in 0.617secs, throughput: 1582.8Mbit/s
symbol count = 10000, encoded 122 MB in 0.735secs, throughput: 1328.7Mbit/s
symbol count = 20000, encoded 122 MB in 1.061secs, throughput: 920.4Mbit/s
symbol count = 50000, encoded 122 MB in 1.515secs, throughput: 644.6Mbit/s

Symbol size: 1280 bytes (with pre-built plan)
symbol count = 10, encoded 127 MB in 0.220secs, throughput: 4654.2Mbit/s
symbol count = 100, encoded 127 MB in 0.149secs, throughput: 6868.7Mbit/s
symbol count = 250, encoded 127 MB in 0.167secs, throughput: 6125.4Mbit/s
symbol count = 500, encoded 127 MB in 0.163secs, throughput: 6260.8Mbit/s
symbol count = 1000, encoded 126 MB in 0.173secs, throughput: 5870.7Mbit/s
symbol count = 2000, encoded 126 MB in 0.199secs, throughput: 5103.6Mbit/s
symbol count = 5000, encoded 122 MB in 0.257secs, throughput: 3799.9Mbit/s
symbol count = 10000, encoded 122 MB in 0.341secs, throughput: 2863.8Mbit/s
symbol count = 20000, encoded 122 MB in 0.427secs, throughput: 2287.0Mbit/s
symbol count = 50000, encoded 122 MB in 0.540secs, throughput: 1808.4Mbit/s

Symbol size: 1280 bytes
symbol count = 10, decoded 127 MB in 0.762secs using 0.0% overhead, throughput: 1343.7Mbit/s
symbol count = 100, decoded 127 MB in 0.692secs using 0.0% overhead, throughput: 1479.0Mbit/s
symbol count = 250, decoded 127 MB in 0.591secs using 0.0% overhead, throughput: 1730.9Mbit/s
symbol count = 500, decoded 127 MB in 0.564secs using 0.0% overhead, throughput: 1809.4Mbit/s
symbol count = 1000, decoded 126 MB in 0.607secs using 0.0% overhead, throughput: 1673.2Mbit/s
symbol count = 2000, decoded 126 MB in 0.664secs using 0.0% overhead, throughput: 1529.6Mbit/s
symbol count = 5000, decoded 122 MB in 0.751secs using 0.0% overhead, throughput: 1300.3Mbit/s
symbol count = 10000, decoded 122 MB in 0.934secs using 0.0% overhead, throughput: 1045.6Mbit/s
symbol count = 20000, decoded 122 MB in 1.198secs using 0.0% overhead, throughput: 815.2Mbit/s
symbol count = 50000, decoded 122 MB in 1.997secs using 0.0% overhead, throughput: 489.0Mbit/s

symbol count = 10, decoded 127 MB in 0.757secs using 5.0% overhead, throughput: 1352.6Mbit/s
symbol count = 100, decoded 127 MB in 0.680secs using 5.0% overhead, throughput: 1505.1Mbit/s
symbol count = 250, decoded 127 MB in 0.592secs using 5.0% overhead, throughput: 1728.0Mbit/s
symbol count = 500, decoded 127 MB in 0.550secs using 5.0% overhead, throughput: 1855.5Mbit/s
symbol count = 1000, decoded 126 MB in 0.576secs using 5.0% overhead, throughput: 1763.2Mbit/s
symbol count = 2000, decoded 126 MB in 0.607secs using 5.0% overhead, throughput: 1673.2Mbit/s
symbol count = 5000, decoded 122 MB in 0.698secs using 5.0% overhead, throughput: 1399.1Mbit/s
symbol count = 10000, decoded 122 MB in 0.882secs using 5.0% overhead, throughput: 1107.2Mbit/s
symbol count = 20000, decoded 122 MB in 1.104secs using 5.0% overhead, throughput: 884.6Mbit/s
symbol count = 50000, decoded 122 MB in 1.825secs using 5.0% overhead, throughput: 535.1Mbit/s
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
