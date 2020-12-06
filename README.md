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
symbol count = 10, encoded 127 MB in 0.514secs, throughput: 1992.1Mbit/s
symbol count = 100, encoded 127 MB in 0.560secs, throughput: 1827.6Mbit/s
symbol count = 250, encoded 127 MB in 0.505secs, throughput: 2025.6Mbit/s
symbol count = 500, encoded 127 MB in 0.461secs, throughput: 2213.7Mbit/s
symbol count = 1000, encoded 126 MB in 0.512secs, throughput: 1983.6Mbit/s
symbol count = 2000, encoded 126 MB in 0.565secs, throughput: 1797.6Mbit/s
symbol count = 5000, encoded 122 MB in 0.604secs, throughput: 1616.8Mbit/s
symbol count = 10000, encoded 122 MB in 0.732secs, throughput: 1334.1Mbit/s
symbol count = 20000, encoded 122 MB in 1.047secs, throughput: 932.7Mbit/s
symbol count = 50000, encoded 122 MB in 1.470secs, throughput: 664.3Mbit/s

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
symbol count = 10, decoded 127 MB in 0.740secs using 0.0% overhead, throughput: 1383.7Mbit/s
symbol count = 100, decoded 127 MB in 0.677secs using 0.0% overhead, throughput: 1511.7Mbit/s
symbol count = 250, decoded 127 MB in 0.566secs using 0.0% overhead, throughput: 1807.3Mbit/s
symbol count = 500, decoded 127 MB in 0.563secs using 0.0% overhead, throughput: 1812.6Mbit/s
symbol count = 1000, decoded 126 MB in 0.585secs using 0.0% overhead, throughput: 1736.1Mbit/s
symbol count = 2000, decoded 126 MB in 0.642secs using 0.0% overhead, throughput: 1582.0Mbit/s
symbol count = 5000, decoded 122 MB in 0.734secs using 0.0% overhead, throughput: 1330.5Mbit/s
symbol count = 10000, decoded 122 MB in 0.919secs using 0.0% overhead, throughput: 1062.6Mbit/s
symbol count = 20000, decoded 122 MB in 1.182secs using 0.0% overhead, throughput: 826.2Mbit/s
symbol count = 50000, decoded 122 MB in 1.991secs using 0.0% overhead, throughput: 490.5Mbit/s

symbol count = 10, decoded 127 MB in 0.742secs using 5.0% overhead, throughput: 1380.0Mbit/s
symbol count = 100, decoded 127 MB in 0.672secs using 5.0% overhead, throughput: 1523.0Mbit/s
symbol count = 250, decoded 127 MB in 0.567secs using 5.0% overhead, throughput: 1804.1Mbit/s
symbol count = 500, decoded 127 MB in 0.547secs using 5.0% overhead, throughput: 1865.6Mbit/s
symbol count = 1000, decoded 126 MB in 0.563secs using 5.0% overhead, throughput: 1804.0Mbit/s
symbol count = 2000, decoded 126 MB in 0.595secs using 5.0% overhead, throughput: 1706.9Mbit/s
symbol count = 5000, decoded 122 MB in 0.675secs using 5.0% overhead, throughput: 1446.8Mbit/s
symbol count = 10000, decoded 122 MB in 0.852secs using 5.0% overhead, throughput: 1146.2Mbit/s
symbol count = 20000, decoded 122 MB in 1.098secs using 5.0% overhead, throughput: 889.4Mbit/s
symbol count = 50000, decoded 122 MB in 1.839secs using 5.0% overhead, throughput: 531.0Mbit/s
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
