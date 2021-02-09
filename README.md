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
symbol count = 10, encoded 127 MB in 0.423secs, throughput: 2420.6Mbit/s
symbol count = 100, encoded 127 MB in 0.393secs, throughput: 2604.2Mbit/s
symbol count = 250, encoded 127 MB in 0.373secs, throughput: 2742.5Mbit/s
symbol count = 500, encoded 127 MB in 0.362secs, throughput: 2819.1Mbit/s
symbol count = 1000, encoded 126 MB in 0.371secs, throughput: 2737.5Mbit/s
symbol count = 2000, encoded 126 MB in 0.401secs, throughput: 2532.7Mbit/s
symbol count = 5000, encoded 122 MB in 0.432secs, throughput: 2260.6Mbit/s
symbol count = 10000, encoded 122 MB in 0.492secs, throughput: 1984.9Mbit/s
symbol count = 20000, encoded 122 MB in 0.642secs, throughput: 1521.1Mbit/s
symbol count = 50000, encoded 122 MB in 0.862secs, throughput: 1132.9Mbit/s

Symbol size: 1280 bytes (with pre-built plan)
symbol count = 10, encoded 127 MB in 0.213secs, throughput: 4807.2Mbit/s
symbol count = 100, encoded 127 MB in 0.141secs, throughput: 7258.4Mbit/s
symbol count = 250, encoded 127 MB in 0.153secs, throughput: 6685.9Mbit/s
symbol count = 500, encoded 127 MB in 0.162secs, throughput: 6299.4Mbit/s
symbol count = 1000, encoded 126 MB in 0.165secs, throughput: 6155.3Mbit/s
symbol count = 2000, encoded 126 MB in 0.184secs, throughput: 5519.7Mbit/s
symbol count = 5000, encoded 122 MB in 0.214secs, throughput: 4563.4Mbit/s
symbol count = 10000, encoded 122 MB in 0.281secs, throughput: 3475.3Mbit/s
symbol count = 20000, encoded 122 MB in 0.373secs, throughput: 2618.1Mbit/s
symbol count = 50000, encoded 122 MB in 0.518secs, throughput: 1885.3Mbit/s

Symbol size: 1280 bytes
symbol count = 10, decoded 127 MB in 0.610secs using 0.0% overhead, throughput: 1678.6Mbit/s
symbol count = 100, decoded 127 MB in 0.484secs using 0.0% overhead, throughput: 2114.5Mbit/s
symbol count = 250, decoded 127 MB in 0.458secs using 0.0% overhead, throughput: 2233.5Mbit/s
symbol count = 500, decoded 127 MB in 0.438secs using 0.0% overhead, throughput: 2329.9Mbit/s
symbol count = 1000, decoded 126 MB in 0.450secs using 0.0% overhead, throughput: 2256.9Mbit/s
symbol count = 2000, decoded 126 MB in 0.485secs using 0.0% overhead, throughput: 2094.1Mbit/s
symbol count = 5000, decoded 122 MB in 0.534secs using 0.0% overhead, throughput: 1828.8Mbit/s
symbol count = 10000, decoded 122 MB in 0.621secs using 0.0% overhead, throughput: 1572.6Mbit/s
symbol count = 20000, decoded 122 MB in 0.819secs using 0.0% overhead, throughput: 1192.4Mbit/s
symbol count = 50000, decoded 122 MB in 1.116secs using 0.0% overhead, throughput: 875.1Mbit/s

symbol count = 10, decoded 127 MB in 0.609secs using 5.0% overhead, throughput: 1681.3Mbit/s
symbol count = 100, decoded 127 MB in 0.490secs using 5.0% overhead, throughput: 2088.6Mbit/s
symbol count = 250, decoded 127 MB in 0.463secs using 5.0% overhead, throughput: 2209.4Mbit/s
symbol count = 500, decoded 127 MB in 0.443secs using 5.0% overhead, throughput: 2303.6Mbit/s
symbol count = 1000, decoded 126 MB in 0.464secs using 5.0% overhead, throughput: 2188.8Mbit/s
symbol count = 2000, decoded 126 MB in 0.490secs using 5.0% overhead, throughput: 2072.7Mbit/s
symbol count = 5000, decoded 122 MB in 0.555secs using 5.0% overhead, throughput: 1759.6Mbit/s
symbol count = 10000, decoded 122 MB in 0.667secs using 5.0% overhead, throughput: 1464.1Mbit/s
symbol count = 20000, decoded 122 MB in 0.830secs using 5.0% overhead, throughput: 1176.6Mbit/s
symbol count = 50000, decoded 122 MB in 1.328secs using 5.0% overhead, throughput: 735.4Mbit/s
```

The following were run on a Raspberry Pi 3 B+ (Cortex-A53 @ 1.4GHz)

```
Symbol size: 1280 bytes (without pre-built plan)
symbol count = 10, encoded 127 MB in 29.579secs, throughput: 34.6Mbit/s
symbol count = 100, encoded 127 MB in 19.524secs, throughput: 52.4Mbit/s
symbol count = 250, encoded 127 MB in 16.283secs, throughput: 62.8Mbit/s
symbol count = 500, encoded 127 MB in 14.094secs, throughput: 72.4Mbit/s
symbol count = 1000, encoded 126 MB in 17.296secs, throughput: 58.7Mbit/s
symbol count = 2000, encoded 126 MB in 15.915secs, throughput: 63.8Mbit/s
symbol count = 5000, encoded 122 MB in 16.614secs, throughput: 58.8Mbit/s
symbol count = 10000, encoded 122 MB in 21.570secs, throughput: 45.3Mbit/s
symbol count = 20000, encoded 122 MB in 22.630secs, throughput: 43.2Mbit/s
symbol count = 50000, encoded 122 MB in 31.688secs, throughput: 30.8Mbit/s

Symbol size: 1280 bytes (with pre-built plan)
symbol count = 10, encoded 127 MB in 21.806secs, throughput: 47.0Mbit/s
symbol count = 100, encoded 127 MB in 12.074secs, throughput: 84.8Mbit/s
symbol count = 250, encoded 127 MB in 10.360secs, throughput: 98.7Mbit/s
symbol count = 500, encoded 127 MB in 14.323secs, throughput: 71.2Mbit/s
symbol count = 1000, encoded 126 MB in 13.773secs, throughput: 73.7Mbit/s
symbol count = 2000, encoded 126 MB in 12.506secs, throughput: 81.2Mbit/s
symbol count = 5000, encoded 122 MB in 10.762secs, throughput: 90.7Mbit/s
symbol count = 10000, encoded 122 MB in 14.438secs, throughput: 67.6Mbit/s
symbol count = 20000, encoded 122 MB in 17.267secs, throughput: 56.6Mbit/s
symbol count = 50000, encoded 122 MB in 19.008secs, throughput: 51.4Mbit/s

Symbol size: 1280 bytes
symbol count = 10, decoded 127 MB in 36.226secs using 0.0% overhead, throughput: 28.3Mbit/s
symbol count = 100, decoded 127 MB in 20.325secs using 0.0% overhead, throughput: 50.4Mbit/s
symbol count = 250, decoded 127 MB in 17.845secs using 0.0% overhead, throughput: 57.3Mbit/s
symbol count = 500, decoded 127 MB in 17.711secs using 0.0% overhead, throughput: 57.6Mbit/s
symbol count = 1000, decoded 126 MB in 14.858secs using 0.0% overhead, throughput: 68.4Mbit/s
symbol count = 2000, decoded 126 MB in 20.109secs using 0.0% overhead, throughput: 50.5Mbit/s
symbol count = 5000, decoded 122 MB in 19.526secs using 0.0% overhead, throughput: 50.0Mbit/s
symbol count = 10000, decoded 122 MB in 18.602secs using 0.0% overhead, throughput: 52.5Mbit/s
symbol count = 20000, decoded 122 MB in 30.212secs using 0.0% overhead, throughput: 32.3Mbit/s
symbol count = 50000, decoded 122 MB in 44.993secs using 0.0% overhead, throughput: 21.7Mbit/s

symbol count = 10, decoded 127 MB in 30.031secs using 5.0% overhead, throughput: 34.1Mbit/s
symbol count = 100, decoded 127 MB in 14.511secs using 5.0% overhead, throughput: 70.5Mbit/s
symbol count = 250, decoded 127 MB in 14.992secs using 5.0% overhead, throughput: 68.2Mbit/s
symbol count = 500, decoded 127 MB in 20.358secs using 5.0% overhead, throughput: 50.1Mbit/s
symbol count = 1000, decoded 126 MB in 15.109secs using 5.0% overhead, throughput: 67.2Mbit/s
symbol count = 2000, decoded 126 MB in 18.539secs using 5.0% overhead, throughput: 54.8Mbit/s
symbol count = 5000, decoded 122 MB in 16.017secs using 5.0% overhead, throughput: 61.0Mbit/s
symbol count = 10000, decoded 122 MB in 17.969secs using 5.0% overhead, throughput: 54.3Mbit/s
symbol count = 20000, decoded 122 MB in 25.134secs using 5.0% overhead, throughput: 38.9Mbit/s
symbol count = 50000, decoded 122 MB in 41.441secs using 5.0% overhead, throughput: 23.6Mbit/s
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
