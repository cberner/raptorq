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
symbol count = 10, encoded 127 MB in 0.455secs, throughput: 2250.4Mbit/s
symbol count = 100, encoded 127 MB in 0.419secs, throughput: 2442.6Mbit/s
symbol count = 250, encoded 127 MB in 0.410secs, throughput: 2495.0Mbit/s
symbol count = 500, encoded 127 MB in 0.407secs, throughput: 2507.4Mbit/s
symbol count = 1000, encoded 126 MB in 0.428secs, throughput: 2373.0Mbit/s
symbol count = 2000, encoded 126 MB in 0.466secs, throughput: 2179.5Mbit/s
symbol count = 5000, encoded 122 MB in 0.507secs, throughput: 1926.2Mbit/s
symbol count = 10000, encoded 122 MB in 0.613secs, throughput: 1593.1Mbit/s
symbol count = 20000, encoded 122 MB in 0.786secs, throughput: 1242.4Mbit/s
symbol count = 50000, encoded 122 MB in 1.067secs, throughput: 915.2Mbit/s

Symbol size: 1280 bytes (with pre-built plan)
symbol count = 10, encoded 127 MB in 0.227secs, throughput: 4510.7Mbit/s
symbol count = 100, encoded 127 MB in 0.152secs, throughput: 6733.1Mbit/s
symbol count = 250, encoded 127 MB in 0.167secs, throughput: 6125.4Mbit/s
symbol count = 500, encoded 127 MB in 0.171secs, throughput: 5967.9Mbit/s
symbol count = 1000, encoded 126 MB in 0.188secs, throughput: 5402.3Mbit/s
symbol count = 2000, encoded 126 MB in 0.216secs, throughput: 4702.0Mbit/s
symbol count = 5000, encoded 122 MB in 0.273secs, throughput: 3577.2Mbit/s
symbol count = 10000, encoded 122 MB in 0.357secs, throughput: 2735.5Mbit/s
symbol count = 20000, encoded 122 MB in 0.435secs, throughput: 2245.0Mbit/s
symbol count = 50000, encoded 122 MB in 0.595secs, throughput: 1641.3Mbit/s

Symbol size: 1280 bytes
symbol count = 10, decoded 127 MB in 0.660secs using 0.0% overhead, throughput: 1551.4Mbit/s
symbol count = 100, decoded 127 MB in 0.522secs using 0.0% overhead, throughput: 1960.6Mbit/s
symbol count = 250, decoded 127 MB in 0.501secs using 0.0% overhead, throughput: 2041.8Mbit/s
symbol count = 500, decoded 127 MB in 0.481secs using 0.0% overhead, throughput: 2121.6Mbit/s
symbol count = 1000, decoded 126 MB in 0.507secs using 0.0% overhead, throughput: 2003.2Mbit/s
symbol count = 2000, decoded 126 MB in 0.557secs using 0.0% overhead, throughput: 1823.4Mbit/s
symbol count = 5000, decoded 122 MB in 0.624secs using 0.0% overhead, throughput: 1565.0Mbit/s
symbol count = 10000, decoded 122 MB in 0.769secs using 0.0% overhead, throughput: 1269.9Mbit/s
symbol count = 20000, decoded 122 MB in 0.988secs using 0.0% overhead, throughput: 988.4Mbit/s
symbol count = 50000, decoded 122 MB in 1.343secs using 0.0% overhead, throughput: 727.2Mbit/s

symbol count = 10, decoded 127 MB in 0.658secs using 5.0% overhead, throughput: 1556.1Mbit/s
symbol count = 100, decoded 127 MB in 0.533secs using 5.0% overhead, throughput: 1920.1Mbit/s
symbol count = 250, decoded 127 MB in 0.505secs using 5.0% overhead, throughput: 2025.6Mbit/s
symbol count = 500, decoded 127 MB in 0.491secs using 5.0% overhead, throughput: 2078.4Mbit/s
symbol count = 1000, decoded 126 MB in 0.519secs using 5.0% overhead, throughput: 1956.9Mbit/s
symbol count = 2000, decoded 126 MB in 0.558secs using 5.0% overhead, throughput: 1820.1Mbit/s
symbol count = 5000, decoded 122 MB in 0.640secs using 5.0% overhead, throughput: 1525.9Mbit/s
symbol count = 10000, decoded 122 MB in 0.821secs using 5.0% overhead, throughput: 1189.5Mbit/s
symbol count = 20000, decoded 122 MB in 1.010secs using 5.0% overhead, throughput: 966.9Mbit/s
symbol count = 50000, decoded 122 MB in 1.588secs using 5.0% overhead, throughput: 615.0Mbit/s
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
