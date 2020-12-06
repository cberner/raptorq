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
symbol count = 10, encoded 127 MB in 0.484secs, throughput: 2115.5Mbit/s
symbol count = 100, encoded 127 MB in 0.509secs, throughput: 2010.7Mbit/s
symbol count = 250, encoded 127 MB in 0.482secs, throughput: 2122.3Mbit/s
symbol count = 500, encoded 127 MB in 0.463secs, throughput: 2204.1Mbit/s
symbol count = 1000, encoded 126 MB in 0.492secs, throughput: 2064.3Mbit/s
symbol count = 2000, encoded 126 MB in 0.565secs, throughput: 1797.6Mbit/s
symbol count = 5000, encoded 122 MB in 0.594secs, throughput: 1644.0Mbit/s
symbol count = 10000, encoded 122 MB in 0.716secs, throughput: 1363.9Mbit/s
symbol count = 20000, encoded 122 MB in 1.059secs, throughput: 922.2Mbit/s
symbol count = 50000, encoded 122 MB in 1.508secs, throughput: 647.6Mbit/s

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
symbol count = 10, decoded 127 MB in 0.706secs using 0.0% overhead, throughput: 1450.3Mbit/s
symbol count = 100, decoded 127 MB in 0.619secs using 0.0% overhead, throughput: 1653.4Mbit/s
symbol count = 250, decoded 127 MB in 0.568secs using 0.0% overhead, throughput: 1801.0Mbit/s
symbol count = 500, decoded 127 MB in 0.560secs using 0.0% overhead, throughput: 1822.3Mbit/s
symbol count = 1000, decoded 126 MB in 0.601secs using 0.0% overhead, throughput: 1689.9Mbit/s
symbol count = 2000, decoded 126 MB in 0.670secs using 0.0% overhead, throughput: 1515.9Mbit/s
symbol count = 5000, decoded 122 MB in 0.767secs using 0.0% overhead, throughput: 1273.2Mbit/s
symbol count = 10000, decoded 122 MB in 0.970secs using 0.0% overhead, throughput: 1006.8Mbit/s
symbol count = 20000, decoded 122 MB in 1.222secs using 0.0% overhead, throughput: 799.2Mbit/s
symbol count = 50000, decoded 122 MB in 2.046secs using 0.0% overhead, throughput: 477.3Mbit/s

symbol count = 10, decoded 127 MB in 0.698secs using 5.0% overhead, throughput: 1466.9Mbit/s
symbol count = 100, decoded 127 MB in 0.617secs using 5.0% overhead, throughput: 1658.7Mbit/s
symbol count = 250, decoded 127 MB in 0.565secs using 5.0% overhead, throughput: 1810.5Mbit/s
symbol count = 500, decoded 127 MB in 0.545secs using 5.0% overhead, throughput: 1872.5Mbit/s
symbol count = 1000, decoded 126 MB in 0.563secs using 5.0% overhead, throughput: 1804.0Mbit/s
symbol count = 2000, decoded 126 MB in 0.599secs using 5.0% overhead, throughput: 1695.5Mbit/s
symbol count = 5000, decoded 122 MB in 0.689secs using 5.0% overhead, throughput: 1417.4Mbit/s
symbol count = 10000, decoded 122 MB in 0.881secs using 5.0% overhead, throughput: 1108.5Mbit/s
symbol count = 20000, decoded 122 MB in 1.117secs using 5.0% overhead, throughput: 874.3Mbit/s
symbol count = 50000, decoded 122 MB in 1.848secs using 5.0% overhead, throughput: 528.4Mbit/s
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
