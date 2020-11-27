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
symbol count = 10, encoded 127 MB in 0.545secs, throughput: 1878.8Mbit/s
symbol count = 100, encoded 127 MB in 0.645secs, throughput: 1586.7Mbit/s
symbol count = 250, encoded 127 MB in 0.509secs, throughput: 2009.7Mbit/s
symbol count = 500, encoded 127 MB in 0.503secs, throughput: 2028.8Mbit/s
symbol count = 1000, encoded 126 MB in 0.544secs, throughput: 1867.0Mbit/s
symbol count = 2000, encoded 126 MB in 0.628secs, throughput: 1617.2Mbit/s
symbol count = 5000, encoded 122 MB in 0.686secs, throughput: 1423.6Mbit/s
symbol count = 10000, encoded 122 MB in 0.833secs, throughput: 1172.3Mbit/s
symbol count = 20000, encoded 122 MB in 1.234secs, throughput: 791.4Mbit/s
symbol count = 50000, encoded 122 MB in 1.786secs, throughput: 546.8Mbit/s

Symbol size: 1280 bytes (with pre-built plan)
symbol count = 10, encoded 127 MB in 0.221secs, throughput: 4633.1Mbit/s
symbol count = 100, encoded 127 MB in 0.149secs, throughput: 6868.7Mbit/s
symbol count = 250, encoded 127 MB in 0.164secs, throughput: 6237.5Mbit/s
symbol count = 500, encoded 127 MB in 0.169secs, throughput: 6038.5Mbit/s
symbol count = 1000, encoded 126 MB in 0.178secs, throughput: 5705.8Mbit/s
symbol count = 2000, encoded 126 MB in 0.214secs, throughput: 4745.9Mbit/s
symbol count = 5000, encoded 122 MB in 0.262secs, throughput: 3727.3Mbit/s
symbol count = 10000, encoded 122 MB in 0.344secs, throughput: 2838.8Mbit/s
symbol count = 20000, encoded 122 MB in 0.427secs, throughput: 2287.0Mbit/s
symbol count = 50000, encoded 122 MB in 0.541secs, throughput: 1805.1Mbit/s

Symbol size: 1280 bytes
symbol count = 10, decoded 127 MB in 0.749secs using 0.0% overhead, throughput: 1367.1Mbit/s
symbol count = 100, decoded 127 MB in 0.742secs using 0.0% overhead, throughput: 1379.3Mbit/s
symbol count = 250, decoded 127 MB in 0.589secs using 0.0% overhead, throughput: 1736.8Mbit/s
symbol count = 500, decoded 127 MB in 0.594secs using 0.0% overhead, throughput: 1718.0Mbit/s
symbol count = 1000, decoded 126 MB in 0.638secs using 0.0% overhead, throughput: 1591.9Mbit/s
symbol count = 2000, decoded 126 MB in 0.718secs using 0.0% overhead, throughput: 1414.5Mbit/s
symbol count = 5000, decoded 122 MB in 0.829secs using 0.0% overhead, throughput: 1178.0Mbit/s
symbol count = 10000, decoded 122 MB in 1.049secs using 0.0% overhead, throughput: 930.9Mbit/s
symbol count = 20000, decoded 122 MB in 1.382secs using 0.0% overhead, throughput: 706.6Mbit/s
symbol count = 50000, decoded 122 MB in 2.355secs using 0.0% overhead, throughput: 414.7Mbit/s

symbol count = 10, decoded 127 MB in 0.740secs using 5.0% overhead, throughput: 1383.7Mbit/s
symbol count = 100, decoded 127 MB in 0.747secs using 5.0% overhead, throughput: 1370.1Mbit/s
symbol count = 250, decoded 127 MB in 0.581secs using 5.0% overhead, throughput: 1760.7Mbit/s
symbol count = 500, decoded 127 MB in 0.565secs using 5.0% overhead, throughput: 1806.2Mbit/s
symbol count = 1000, decoded 126 MB in 0.605secs using 5.0% overhead, throughput: 1678.7Mbit/s
symbol count = 2000, decoded 126 MB in 0.656secs using 5.0% overhead, throughput: 1548.2Mbit/s
symbol count = 5000, decoded 122 MB in 0.763secs using 5.0% overhead, throughput: 1279.9Mbit/s
symbol count = 10000, decoded 122 MB in 0.959secs using 5.0% overhead, throughput: 1018.3Mbit/s
symbol count = 20000, decoded 122 MB in 1.242secs using 5.0% overhead, throughput: 786.3Mbit/s
symbol count = 50000, decoded 122 MB in 2.146secs using 5.0% overhead, throughput: 455.1Mbit/s
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
