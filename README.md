# raptorq
![CI](https://github.com/cberner/raptorq/actions/workflows/ci.yml/badge.svg)
[![Crates](https://img.shields.io/crates/v/raptorq.svg)](https://crates.io/crates/raptorq)
[![Documentation](https://docs.rs/raptorq/badge.svg)](https://docs.rs/raptorq)
[![PyPI](https://img.shields.io/pypi/v/raptorq.svg)](https://pypi.org/project/raptorq/)
[![License](https://img.shields.io/crates/l/raptorq)](https://crates.io/crates/raptorq)
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
symbol count = 10, encoded 127 MB in 6.138secs, throughput: 166.8Mbit/s
symbol count = 100, encoded 127 MB in 4.439secs, throughput: 230.6Mbit/s
symbol count = 250, encoded 127 MB in 4.867secs, throughput: 210.2Mbit/s
symbol count = 500, encoded 127 MB in 4.892secs, throughput: 208.6Mbit/s
symbol count = 1000, encoded 126 MB in 5.029secs, throughput: 202.0Mbit/s
symbol count = 2000, encoded 126 MB in 5.600secs, throughput: 181.4Mbit/s
symbol count = 5000, encoded 122 MB in 6.074secs, throughput: 160.8Mbit/s
symbol count = 10000, encoded 122 MB in 6.820secs, throughput: 143.2Mbit/s
symbol count = 20000, encoded 122 MB in 7.971secs, throughput: 122.5Mbit/s
symbol count = 50000, encoded 122 MB in 10.061secs, throughput: 97.1Mbit/s

Symbol size: 1280 bytes (with pre-built plan)
symbol count = 10, encoded 127 MB in 4.416secs, throughput: 231.9Mbit/s
symbol count = 100, encoded 127 MB in 2.964secs, throughput: 345.3Mbit/s
symbol count = 250, encoded 127 MB in 3.374secs, throughput: 303.2Mbit/s
symbol count = 500, encoded 127 MB in 3.476secs, throughput: 293.6Mbit/s
symbol count = 1000, encoded 126 MB in 3.661secs, throughput: 277.4Mbit/s
symbol count = 2000, encoded 126 MB in 4.107secs, throughput: 247.3Mbit/s
symbol count = 5000, encoded 122 MB in 4.447secs, throughput: 219.6Mbit/s
symbol count = 10000, encoded 122 MB in 4.891secs, throughput: 199.7Mbit/s
symbol count = 20000, encoded 122 MB in 5.413secs, throughput: 180.4Mbit/s
symbol count = 50000, encoded 122 MB in 6.645secs, throughput: 147.0Mbit/s

Symbol size: 1280 bytes
symbol count = 10, decoded 127 MB in 7.302secs using 0.0% overhead, throughput: 140.2Mbit/s
symbol count = 100, decoded 127 MB in 5.435secs using 0.0% overhead, throughput: 188.3Mbit/s
symbol count = 250, decoded 127 MB in 5.612secs using 0.0% overhead, throughput: 182.3Mbit/s
symbol count = 500, decoded 127 MB in 5.678secs using 0.0% overhead, throughput: 179.7Mbit/s
symbol count = 1000, decoded 126 MB in 5.923secs using 0.0% overhead, throughput: 171.5Mbit/s
symbol count = 2000, decoded 126 MB in 6.720secs using 0.0% overhead, throughput: 151.1Mbit/s
symbol count = 5000, decoded 122 MB in 7.236secs using 0.0% overhead, throughput: 135.0Mbit/s
symbol count = 10000, decoded 122 MB in 7.990secs using 0.0% overhead, throughput: 122.2Mbit/s
symbol count = 20000, decoded 122 MB in 9.228secs using 0.0% overhead, throughput: 105.8Mbit/s
symbol count = 50000, decoded 122 MB in 11.829secs using 0.0% overhead, throughput: 82.6Mbit/s

symbol count = 10, decoded 127 MB in 7.258secs using 5.0% overhead, throughput: 141.1Mbit/s
symbol count = 100, decoded 127 MB in 5.433secs using 5.0% overhead, throughput: 188.4Mbit/s
symbol count = 250, decoded 127 MB in 5.639secs using 5.0% overhead, throughput: 181.4Mbit/s
symbol count = 500, decoded 127 MB in 5.789secs using 5.0% overhead, throughput: 176.3Mbit/s
symbol count = 1000, decoded 126 MB in 6.068secs using 5.0% overhead, throughput: 167.4Mbit/s
symbol count = 2000, decoded 126 MB in 6.808secs using 5.0% overhead, throughput: 149.2Mbit/s
symbol count = 5000, decoded 122 MB in 7.513secs using 5.0% overhead, throughput: 130.0Mbit/s
symbol count = 10000, decoded 122 MB in 8.604secs using 5.0% overhead, throughput: 113.5Mbit/s
symbol count = 20000, decoded 122 MB in 10.417secs using 5.0% overhead, throughput: 93.7Mbit/s
symbol count = 50000, decoded 122 MB in 14.916secs using 5.0% overhead, throughput: 65.5Mbit/s
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
