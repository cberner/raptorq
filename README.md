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
symbol count = 10, encoded 127 MB in 0.548secs, throughput: 1868.5Mbit/s
symbol count = 100, encoded 127 MB in 0.637secs, throughput: 1606.7Mbit/s
symbol count = 250, encoded 127 MB in 0.507secs, throughput: 2017.7Mbit/s
symbol count = 500, encoded 127 MB in 0.488secs, throughput: 2091.2Mbit/s
symbol count = 1000, encoded 126 MB in 0.523secs, throughput: 1941.9Mbit/s
symbol count = 2000, encoded 126 MB in 0.599secs, throughput: 1695.5Mbit/s
symbol count = 5000, encoded 122 MB in 0.636secs, throughput: 1535.5Mbit/s
symbol count = 10000, encoded 122 MB in 0.769secs, throughput: 1269.9Mbit/s
symbol count = 20000, encoded 122 MB in 1.122secs, throughput: 870.4Mbit/s
symbol count = 50000, encoded 122 MB in 1.597secs, throughput: 611.5Mbit/s

Symbol size: 1280 bytes (with pre-built plan)
symbol count = 10, encoded 127 MB in 0.221secs, throughput: 4633.1Mbit/s
symbol count = 100, encoded 127 MB in 0.154secs, throughput: 6645.7Mbit/s
symbol count = 250, encoded 127 MB in 0.160secs, throughput: 6393.4Mbit/s
symbol count = 500, encoded 127 MB in 0.163secs, throughput: 6260.8Mbit/s
symbol count = 1000, encoded 126 MB in 0.173secs, throughput: 5870.7Mbit/s
symbol count = 2000, encoded 126 MB in 0.199secs, throughput: 5103.6Mbit/s
symbol count = 5000, encoded 122 MB in 0.255secs, throughput: 3829.7Mbit/s
symbol count = 10000, encoded 122 MB in 0.339secs, throughput: 2880.7Mbit/s
symbol count = 20000, encoded 122 MB in 0.425secs, throughput: 2297.8Mbit/s
symbol count = 50000, encoded 122 MB in 0.536secs, throughput: 1821.9Mbit/s

Symbol size: 1280 bytes
symbol count = 10, decoded 127 MB in 0.758secs using 0.0% overhead, throughput: 1350.8Mbit/s
symbol count = 100, decoded 127 MB in 0.740secs using 0.0% overhead, throughput: 1383.0Mbit/s
symbol count = 250, decoded 127 MB in 0.577secs using 0.0% overhead, throughput: 1772.9Mbit/s
symbol count = 500, decoded 127 MB in 0.582secs using 0.0% overhead, throughput: 1753.4Mbit/s
symbol count = 1000, decoded 126 MB in 0.628secs using 0.0% overhead, throughput: 1617.2Mbit/s
symbol count = 2000, decoded 126 MB in 0.684secs using 0.0% overhead, throughput: 1484.8Mbit/s
symbol count = 5000, decoded 122 MB in 0.785secs using 0.0% overhead, throughput: 1244.0Mbit/s
symbol count = 10000, decoded 122 MB in 0.965secs using 0.0% overhead, throughput: 1012.0Mbit/s
symbol count = 20000, decoded 122 MB in 1.345secs using 0.0% overhead, throughput: 726.1Mbit/s
symbol count = 50000, decoded 122 MB in 2.101secs using 0.0% overhead, throughput: 464.8Mbit/s

symbol count = 10, decoded 127 MB in 0.753secs using 5.0% overhead, throughput: 1359.8Mbit/s
symbol count = 100, decoded 127 MB in 0.731secs using 5.0% overhead, throughput: 1400.1Mbit/s
symbol count = 250, decoded 127 MB in 0.575secs using 5.0% overhead, throughput: 1779.0Mbit/s
symbol count = 500, decoded 127 MB in 0.552secs using 5.0% overhead, throughput: 1848.7Mbit/s
symbol count = 1000, decoded 126 MB in 0.568secs using 5.0% overhead, throughput: 1788.1Mbit/s
symbol count = 2000, decoded 126 MB in 0.626secs using 5.0% overhead, throughput: 1622.4Mbit/s
symbol count = 5000, decoded 122 MB in 0.713secs using 5.0% overhead, throughput: 1369.7Mbit/s
symbol count = 10000, decoded 122 MB in 0.893secs using 5.0% overhead, throughput: 1093.6Mbit/s
symbol count = 20000, decoded 122 MB in 1.147secs using 5.0% overhead, throughput: 851.4Mbit/s
symbol count = 50000, decoded 122 MB in 1.943secs using 5.0% overhead, throughput: 502.6Mbit/s
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
