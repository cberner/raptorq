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
symbol count = 10, encoded 127 MB in 11.898secs, throughput: 86.1Mbit/s
symbol count = 100, encoded 127 MB in 8.862secs, throughput: 115.5Mbit/s
symbol count = 250, encoded 127 MB in 9.103secs, throughput: 112.4Mbit/s
symbol count = 500, encoded 127 MB in 8.806secs, throughput: 115.9Mbit/s
symbol count = 1000, encoded 126 MB in 9.412secs, throughput: 107.9Mbit/s
symbol count = 2000, encoded 126 MB in 7.041secs, throughput: 144.2Mbit/s
symbol count = 5000, encoded 122 MB in 12.119secs, throughput: 80.6Mbit/s
symbol count = 10000, encoded 122 MB in 9.694secs, throughput: 100.7Mbit/s
symbol count = 20000, encoded 122 MB in 12.087secs, throughput: 80.8Mbit/s
symbol count = 50000, encoded 122 MB in 23.912secs, throughput: 40.8Mbit/s

Symbol size: 1280 bytes (with pre-built plan)
symbol count = 10, encoded 127 MB in 8.399secs, throughput: 121.9Mbit/s
symbol count = 100, encoded 127 MB in 4.660secs, throughput: 219.6Mbit/s
symbol count = 250, encoded 127 MB in 6.373secs, throughput: 160.5Mbit/s
symbol count = 500, encoded 127 MB in 4.699secs, throughput: 217.2Mbit/s
symbol count = 1000, encoded 126 MB in 5.978secs, throughput: 169.9Mbit/s
symbol count = 2000, encoded 126 MB in 6.182secs, throughput: 164.3Mbit/s
symbol count = 5000, encoded 122 MB in 5.958secs, throughput: 163.9Mbit/s
symbol count = 10000, encoded 122 MB in 7.228secs, throughput: 135.1Mbit/s
symbol count = 20000, encoded 122 MB in 6.764secs, throughput: 144.4Mbit/s
symbol count = 50000, encoded 122 MB in 6.649secs, throughput: 146.9Mbit/s

Symbol size: 1280 bytes
symbol count = 10, decoded 127 MB in 13.727secs using 0.0% overhead, throughput: 74.6Mbit/s
symbol count = 100, decoded 127 MB in 9.727secs using 0.0% overhead, throughput: 105.2Mbit/s
symbol count = 250, decoded 127 MB in 12.135secs using 0.0% overhead, throughput: 84.3Mbit/s
symbol count = 500, decoded 127 MB in 10.658secs using 0.0% overhead, throughput: 95.8Mbit/s
symbol count = 1000, decoded 126 MB in 10.414secs using 0.0% overhead, throughput: 97.5Mbit/s
symbol count = 2000, decoded 126 MB in 10.828secs using 0.0% overhead, throughput: 93.8Mbit/s
symbol count = 5000, decoded 122 MB in 12.545secs using 0.0% overhead, throughput: 77.8Mbit/s
symbol count = 10000, decoded 122 MB in 10.667secs using 0.0% overhead, throughput: 91.5Mbit/s
symbol count = 20000, decoded 122 MB in 19.769secs using 0.0% overhead, throughput: 49.4Mbit/s
symbol count = 50000, decoded 122 MB in 25.817secs using 0.0% overhead, throughput: 37.8Mbit/s

symbol count = 10, decoded 127 MB in 11.557secs using 5.0% overhead, throughput: 88.6Mbit/s
symbol count = 100, decoded 127 MB in 9.586secs using 5.0% overhead, throughput: 106.8Mbit/s
symbol count = 250, decoded 127 MB in 11.725secs using 5.0% overhead, throughput: 87.2Mbit/s
symbol count = 500, decoded 127 MB in 10.859secs using 5.0% overhead, throughput: 94.0Mbit/s
symbol count = 1000, decoded 126 MB in 7.036secs using 5.0% overhead, throughput: 144.3Mbit/s
symbol count = 2000, decoded 126 MB in 11.247secs using 5.0% overhead, throughput: 90.3Mbit/s
symbol count = 5000, decoded 122 MB in 12.590secs using 5.0% overhead, throughput: 77.6Mbit/s
symbol count = 10000, decoded 122 MB in 15.379secs using 5.0% overhead, throughput: 63.5Mbit/s
symbol count = 20000, decoded 122 MB in 18.543secs using 5.0% overhead, throughput: 52.7Mbit/s
symbol count = 50000, decoded 122 MB in 32.090secs using 5.0% overhead, throughput: 30.4Mbit/s
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
