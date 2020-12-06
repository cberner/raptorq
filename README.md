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
symbol count = 10, encoded 127 MB in 0.566secs, throughput: 1809.1Mbit/s
symbol count = 100, encoded 127 MB in 0.684secs, throughput: 1496.3Mbit/s
symbol count = 250, encoded 127 MB in 0.506secs, throughput: 2021.6Mbit/s
symbol count = 500, encoded 127 MB in 0.471secs, throughput: 2166.7Mbit/s
symbol count = 1000, encoded 126 MB in 0.525secs, throughput: 1934.5Mbit/s
symbol count = 2000, encoded 126 MB in 0.583secs, throughput: 1742.1Mbit/s
symbol count = 5000, encoded 122 MB in 0.625secs, throughput: 1562.5Mbit/s
symbol count = 10000, encoded 122 MB in 0.726secs, throughput: 1345.1Mbit/s
symbol count = 20000, encoded 122 MB in 1.057secs, throughput: 923.9Mbit/s
symbol count = 50000, encoded 122 MB in 1.494secs, throughput: 653.7Mbit/s

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
symbol count = 10, decoded 127 MB in 0.794secs using 0.0% overhead, throughput: 1289.6Mbit/s
symbol count = 100, decoded 127 MB in 0.785secs using 0.0% overhead, throughput: 1303.7Mbit/s
symbol count = 250, decoded 127 MB in 0.585secs using 0.0% overhead, throughput: 1748.6Mbit/s
symbol count = 500, decoded 127 MB in 0.557secs using 0.0% overhead, throughput: 1832.2Mbit/s
symbol count = 1000, decoded 126 MB in 0.600secs using 0.0% overhead, throughput: 1692.7Mbit/s
symbol count = 2000, decoded 126 MB in 0.656secs using 0.0% overhead, throughput: 1548.2Mbit/s
symbol count = 5000, decoded 122 MB in 0.763secs using 0.0% overhead, throughput: 1279.9Mbit/s
symbol count = 10000, decoded 122 MB in 0.930secs using 0.0% overhead, throughput: 1050.1Mbit/s
symbol count = 20000, decoded 122 MB in 1.171secs using 0.0% overhead, throughput: 834.0Mbit/s
symbol count = 50000, decoded 122 MB in 1.962secs using 0.0% overhead, throughput: 497.7Mbit/s

symbol count = 10, decoded 127 MB in 0.786secs using 5.0% overhead, throughput: 1302.7Mbit/s
symbol count = 100, decoded 127 MB in 0.783secs using 5.0% overhead, throughput: 1307.1Mbit/s
symbol count = 250, decoded 127 MB in 0.585secs using 5.0% overhead, throughput: 1748.6Mbit/s
symbol count = 500, decoded 127 MB in 0.536secs using 5.0% overhead, throughput: 1903.9Mbit/s
symbol count = 1000, decoded 126 MB in 0.568secs using 5.0% overhead, throughput: 1788.1Mbit/s
symbol count = 2000, decoded 126 MB in 0.598secs using 5.0% overhead, throughput: 1698.4Mbit/s
symbol count = 5000, decoded 122 MB in 0.692secs using 5.0% overhead, throughput: 1411.2Mbit/s
symbol count = 10000, decoded 122 MB in 0.875secs using 5.0% overhead, throughput: 1116.1Mbit/s
symbol count = 20000, decoded 122 MB in 1.082secs using 5.0% overhead, throughput: 902.6Mbit/s
symbol count = 50000, decoded 122 MB in 1.790secs using 5.0% overhead, throughput: 545.6Mbit/s
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
