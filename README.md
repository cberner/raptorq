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
symbol count = 10, encoded 127 MB in 0.574secs, throughput: 1783.8Mbit/s
symbol count = 100, encoded 127 MB in 0.653secs, throughput: 1567.3Mbit/s
symbol count = 250, encoded 127 MB in 0.497secs, throughput: 2058.2Mbit/s
symbol count = 500, encoded 127 MB in 0.477secs, throughput: 2139.4Mbit/s
symbol count = 1000, encoded 126 MB in 0.513secs, throughput: 1979.8Mbit/s
symbol count = 2000, encoded 126 MB in 0.590secs, throughput: 1721.4Mbit/s
symbol count = 5000, encoded 122 MB in 0.629secs, throughput: 1552.6Mbit/s
symbol count = 10000, encoded 122 MB in 0.736secs, throughput: 1326.9Mbit/s
symbol count = 20000, encoded 122 MB in 1.094secs, throughput: 892.7Mbit/s
symbol count = 50000, encoded 122 MB in 1.548secs, throughput: 630.9Mbit/s

Symbol size: 1280 bytes (with pre-built plan)
symbol count = 10, encoded 127 MB in 0.226secs, throughput: 4530.6Mbit/s
symbol count = 100, encoded 127 MB in 0.149secs, throughput: 6868.7Mbit/s
symbol count = 250, encoded 127 MB in 0.162secs, throughput: 6314.5Mbit/s
symbol count = 500, encoded 127 MB in 0.164secs, throughput: 6222.6Mbit/s
symbol count = 1000, encoded 126 MB in 0.178secs, throughput: 5705.8Mbit/s
symbol count = 2000, encoded 126 MB in 0.204secs, throughput: 4978.6Mbit/s
symbol count = 5000, encoded 122 MB in 0.269secs, throughput: 3630.3Mbit/s
symbol count = 10000, encoded 122 MB in 0.348secs, throughput: 2806.2Mbit/s
symbol count = 20000, encoded 122 MB in 0.437secs, throughput: 2234.7Mbit/s
symbol count = 50000, encoded 122 MB in 0.549secs, throughput: 1778.8Mbit/s

Symbol size: 1280 bytes
symbol count = 10, decoded 127 MB in 0.759secs using 0.0% overhead, throughput: 1349.0Mbit/s
symbol count = 100, decoded 127 MB in 0.746secs using 0.0% overhead, throughput: 1371.9Mbit/s
symbol count = 250, decoded 127 MB in 0.569secs using 0.0% overhead, throughput: 1797.8Mbit/s
symbol count = 500, decoded 127 MB in 0.556secs using 0.0% overhead, throughput: 1835.4Mbit/s
symbol count = 1000, decoded 126 MB in 0.591secs using 0.0% overhead, throughput: 1718.5Mbit/s
symbol count = 2000, decoded 126 MB in 0.660secs using 0.0% overhead, throughput: 1538.8Mbit/s
symbol count = 5000, decoded 122 MB in 0.738secs using 0.0% overhead, throughput: 1323.3Mbit/s
symbol count = 10000, decoded 122 MB in 0.931secs using 0.0% overhead, throughput: 1048.9Mbit/s
symbol count = 20000, decoded 122 MB in 1.192secs using 0.0% overhead, throughput: 819.3Mbit/s
symbol count = 50000, decoded 122 MB in 2.050secs using 0.0% overhead, throughput: 476.4Mbit/s

symbol count = 10, decoded 127 MB in 0.747secs using 5.0% overhead, throughput: 1370.7Mbit/s
symbol count = 100, decoded 127 MB in 0.745secs using 5.0% overhead, throughput: 1373.7Mbit/s
symbol count = 250, decoded 127 MB in 0.562secs using 5.0% overhead, throughput: 1820.2Mbit/s
symbol count = 500, decoded 127 MB in 0.540secs using 5.0% overhead, throughput: 1889.8Mbit/s
symbol count = 1000, decoded 126 MB in 0.564secs using 5.0% overhead, throughput: 1800.8Mbit/s
symbol count = 2000, decoded 126 MB in 0.612secs using 5.0% overhead, throughput: 1659.5Mbit/s
symbol count = 5000, decoded 122 MB in 0.702secs using 5.0% overhead, throughput: 1391.1Mbit/s
symbol count = 10000, decoded 122 MB in 0.879secs using 5.0% overhead, throughput: 1111.0Mbit/s
symbol count = 20000, decoded 122 MB in 1.112secs using 5.0% overhead, throughput: 878.2Mbit/s
symbol count = 50000, decoded 122 MB in 1.909secs using 5.0% overhead, throughput: 511.6Mbit/s
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
