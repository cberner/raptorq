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
symbol count = 10, encoded 127 MB in 9.478secs, throughput: 108.0Mbit/s
symbol count = 100, encoded 127 MB in 6.281secs, throughput: 162.9Mbit/s
symbol count = 250, encoded 127 MB in 7.216secs, throughput: 141.8Mbit/s
symbol count = 500, encoded 127 MB in 7.623secs, throughput: 133.9Mbit/s
symbol count = 1000, encoded 126 MB in 8.424secs, throughput: 120.6Mbit/s
symbol count = 2000, encoded 126 MB in 8.775secs, throughput: 115.7Mbit/s
symbol count = 5000, encoded 122 MB in 8.439secs, throughput: 115.7Mbit/s
symbol count = 10000, encoded 122 MB in 8.297secs, throughput: 117.7Mbit/s
symbol count = 20000, encoded 122 MB in 9.329secs, throughput: 104.7Mbit/s
symbol count = 50000, encoded 122 MB in 11.724secs, throughput: 83.3Mbit/s

Symbol size: 1280 bytes (with pre-built plan)
symbol count = 10, encoded 127 MB in 6.298secs, throughput: 162.6Mbit/s
symbol count = 100, encoded 127 MB in 5.402secs, throughput: 189.5Mbit/s
symbol count = 250, encoded 127 MB in 5.312secs, throughput: 192.6Mbit/s
symbol count = 500, encoded 127 MB in 5.296secs, throughput: 192.7Mbit/s
symbol count = 1000, encoded 126 MB in 4.081secs, throughput: 248.9Mbit/s
symbol count = 2000, encoded 126 MB in 4.110secs, throughput: 247.1Mbit/s
symbol count = 5000, encoded 122 MB in 5.947secs, throughput: 164.2Mbit/s
symbol count = 10000, encoded 122 MB in 6.271secs, throughput: 155.7Mbit/s
symbol count = 20000, encoded 122 MB in 6.745secs, throughput: 144.8Mbit/s
symbol count = 50000, encoded 122 MB in 6.646secs, throughput: 146.9Mbit/s

Symbol size: 1280 bytes
symbol count = 10, decoded 127 MB in 11.529secs using 0.0% overhead, throughput: 88.8Mbit/s
symbol count = 100, decoded 127 MB in 8.011secs using 0.0% overhead, throughput: 127.8Mbit/s
symbol count = 250, decoded 127 MB in 9.322secs using 0.0% overhead, throughput: 109.7Mbit/s
symbol count = 500, decoded 127 MB in 9.388secs using 0.0% overhead, throughput: 108.7Mbit/s
symbol count = 1000, decoded 126 MB in 7.614secs using 0.0% overhead, throughput: 133.4Mbit/s
symbol count = 2000, decoded 126 MB in 6.706secs using 0.0% overhead, throughput: 151.5Mbit/s
symbol count = 5000, decoded 122 MB in 8.677secs using 0.0% overhead, throughput: 112.5Mbit/s
symbol count = 10000, decoded 122 MB in 9.529secs using 0.0% overhead, throughput: 102.5Mbit/s
symbol count = 20000, decoded 122 MB in 10.766secs using 0.0% overhead, throughput: 90.7Mbit/s
symbol count = 50000, decoded 122 MB in 13.497secs using 0.0% overhead, throughput: 72.4Mbit/s

symbol count = 10, decoded 127 MB in 14.057secs using 5.0% overhead, throughput: 72.8Mbit/s
symbol count = 100, decoded 127 MB in 10.187secs using 5.0% overhead, throughput: 100.5Mbit/s
symbol count = 250, decoded 127 MB in 9.220secs using 5.0% overhead, throughput: 110.9Mbit/s
symbol count = 500, decoded 127 MB in 9.276secs using 5.0% overhead, throughput: 110.0Mbit/s
symbol count = 1000, decoded 126 MB in 8.117secs using 5.0% overhead, throughput: 125.1Mbit/s
symbol count = 2000, decoded 126 MB in 8.459secs using 5.0% overhead, throughput: 120.1Mbit/s
symbol count = 5000, decoded 122 MB in 8.410secs using 5.0% overhead, throughput: 116.1Mbit/s
symbol count = 10000, decoded 122 MB in 11.370secs using 5.0% overhead, throughput: 85.9Mbit/s
symbol count = 20000, decoded 122 MB in 11.923secs using 5.0% overhead, throughput: 81.9Mbit/s
symbol count = 50000, decoded 122 MB in 17.768secs using 5.0% overhead, throughput: 55.0Mbit/s
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
