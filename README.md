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
symbol count = 10, encoded 127 MB in 0.483secs, throughput: 2119.9Mbit/s
symbol count = 100, encoded 127 MB in 0.480secs, throughput: 2132.2Mbit/s
symbol count = 250, encoded 127 MB in 0.458secs, throughput: 2233.5Mbit/s
symbol count = 500, encoded 127 MB in 0.466secs, throughput: 2189.9Mbit/s
symbol count = 1000, encoded 126 MB in 0.474secs, throughput: 2142.7Mbit/s
symbol count = 2000, encoded 126 MB in 0.542secs, throughput: 1873.8Mbit/s
symbol count = 5000, encoded 122 MB in 0.571secs, throughput: 1710.3Mbit/s
symbol count = 10000, encoded 122 MB in 0.682secs, throughput: 1431.9Mbit/s
symbol count = 20000, encoded 122 MB in 0.901secs, throughput: 1083.9Mbit/s
symbol count = 50000, encoded 122 MB in 1.348secs, throughput: 724.5Mbit/s

Symbol size: 1280 bytes (with pre-built plan)
symbol count = 10, encoded 127 MB in 0.238secs, throughput: 4302.2Mbit/s
symbol count = 100, encoded 127 MB in 0.151secs, throughput: 6777.7Mbit/s
symbol count = 250, encoded 127 MB in 0.167secs, throughput: 6125.4Mbit/s
symbol count = 500, encoded 127 MB in 0.174secs, throughput: 5865.0Mbit/s
symbol count = 1000, encoded 126 MB in 0.191secs, throughput: 5317.4Mbit/s
symbol count = 2000, encoded 126 MB in 0.226secs, throughput: 4493.9Mbit/s
symbol count = 5000, encoded 122 MB in 0.258secs, throughput: 3785.1Mbit/s
symbol count = 10000, encoded 122 MB in 0.336secs, throughput: 2906.4Mbit/s
symbol count = 20000, encoded 122 MB in 0.428secs, throughput: 2281.7Mbit/s
symbol count = 50000, encoded 122 MB in 0.603secs, throughput: 1619.5Mbit/s

Symbol size: 1280 bytes
symbol count = 10, decoded 127 MB in 0.727secs using 0.0% overhead, throughput: 1408.4Mbit/s
symbol count = 100, decoded 127 MB in 0.598secs using 0.0% overhead, throughput: 1711.4Mbit/s
symbol count = 250, decoded 127 MB in 0.570secs using 0.0% overhead, throughput: 1794.6Mbit/s
symbol count = 500, decoded 127 MB in 0.572secs using 0.0% overhead, throughput: 1784.1Mbit/s
symbol count = 1000, decoded 126 MB in 0.600secs using 0.0% overhead, throughput: 1692.7Mbit/s
symbol count = 2000, decoded 126 MB in 0.652secs using 0.0% overhead, throughput: 1557.7Mbit/s
symbol count = 5000, decoded 122 MB in 0.719secs using 0.0% overhead, throughput: 1358.2Mbit/s
symbol count = 10000, decoded 122 MB in 0.866secs using 0.0% overhead, throughput: 1127.7Mbit/s
symbol count = 20000, decoded 122 MB in 1.085secs using 0.0% overhead, throughput: 900.1Mbit/s
symbol count = 50000, decoded 122 MB in 1.566secs using 0.0% overhead, throughput: 623.6Mbit/s

symbol count = 10, decoded 127 MB in 0.711secs using 5.0% overhead, throughput: 1440.1Mbit/s
symbol count = 100, decoded 127 MB in 0.610secs using 5.0% overhead, throughput: 1677.8Mbit/s
symbol count = 250, decoded 127 MB in 0.596secs using 5.0% overhead, throughput: 1716.4Mbit/s
symbol count = 500, decoded 127 MB in 0.574secs using 5.0% overhead, throughput: 1777.9Mbit/s
symbol count = 1000, decoded 126 MB in 0.630secs using 5.0% overhead, throughput: 1612.1Mbit/s
symbol count = 2000, decoded 126 MB in 0.653secs using 5.0% overhead, throughput: 1555.3Mbit/s
symbol count = 5000, decoded 122 MB in 0.781secs using 5.0% overhead, throughput: 1250.4Mbit/s
symbol count = 10000, decoded 122 MB in 0.994secs using 5.0% overhead, throughput: 982.5Mbit/s
symbol count = 20000, decoded 122 MB in 1.318secs using 5.0% overhead, throughput: 740.9Mbit/s
symbol count = 50000, decoded 122 MB in 2.182secs using 5.0% overhead, throughput: 447.6Mbit/s
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
