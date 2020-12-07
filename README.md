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
symbol count = 10, encoded 127 MB in 0.465secs, throughput: 2202.0Mbit/s
symbol count = 100, encoded 127 MB in 0.483secs, throughput: 2118.9Mbit/s
symbol count = 250, encoded 127 MB in 0.474secs, throughput: 2158.1Mbit/s
symbol count = 500, encoded 127 MB in 0.460secs, throughput: 2218.5Mbit/s
symbol count = 1000, encoded 126 MB in 0.490secs, throughput: 2072.7Mbit/s
symbol count = 2000, encoded 126 MB in 0.562secs, throughput: 1807.2Mbit/s
symbol count = 5000, encoded 122 MB in 0.578secs, throughput: 1689.6Mbit/s
symbol count = 10000, encoded 122 MB in 0.687secs, throughput: 1421.5Mbit/s
symbol count = 20000, encoded 122 MB in 1.019secs, throughput: 958.4Mbit/s
symbol count = 50000, encoded 122 MB in 1.432secs, throughput: 682.0Mbit/s

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
symbol count = 10, decoded 127 MB in 0.679secs using 0.0% overhead, throughput: 1508.0Mbit/s
symbol count = 100, decoded 127 MB in 0.583secs using 0.0% overhead, throughput: 1755.5Mbit/s
symbol count = 250, decoded 127 MB in 0.564secs using 0.0% overhead, throughput: 1813.7Mbit/s
symbol count = 500, decoded 127 MB in 0.539secs using 0.0% overhead, throughput: 1893.3Mbit/s
symbol count = 1000, decoded 126 MB in 0.571secs using 0.0% overhead, throughput: 1778.7Mbit/s
symbol count = 2000, decoded 126 MB in 0.708secs using 0.0% overhead, throughput: 1434.5Mbit/s
symbol count = 5000, decoded 122 MB in 0.769secs using 0.0% overhead, throughput: 1269.9Mbit/s
symbol count = 10000, decoded 122 MB in 0.902secs using 0.0% overhead, throughput: 1082.7Mbit/s
symbol count = 20000, decoded 122 MB in 1.135secs using 0.0% overhead, throughput: 860.4Mbit/s
symbol count = 50000, decoded 122 MB in 1.929secs using 0.0% overhead, throughput: 506.3Mbit/s

symbol count = 10, decoded 127 MB in 0.669secs using 5.0% overhead, throughput: 1530.5Mbit/s
symbol count = 100, decoded 127 MB in 0.582secs using 5.0% overhead, throughput: 1758.5Mbit/s
symbol count = 250, decoded 127 MB in 0.550secs using 5.0% overhead, throughput: 1859.9Mbit/s
symbol count = 500, decoded 127 MB in 0.520secs using 5.0% overhead, throughput: 1962.5Mbit/s
symbol count = 1000, decoded 126 MB in 0.548secs using 5.0% overhead, throughput: 1853.3Mbit/s
symbol count = 2000, decoded 126 MB in 0.582secs using 5.0% overhead, throughput: 1745.1Mbit/s
symbol count = 5000, decoded 122 MB in 0.658secs using 5.0% overhead, throughput: 1484.1Mbit/s
symbol count = 10000, decoded 122 MB in 0.835secs using 5.0% overhead, throughput: 1169.5Mbit/s
symbol count = 20000, decoded 122 MB in 1.105secs using 5.0% overhead, throughput: 883.8Mbit/s
symbol count = 50000, decoded 122 MB in 1.784secs using 5.0% overhead, throughput: 547.4Mbit/s
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
