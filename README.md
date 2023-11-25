# raptorq
![CI](https://github.com/cberner/raptorq/actions/workflows/ci.yml/badge.svg)
[![Crates](https://img.shields.io/crates/v/raptorq.svg)](https://crates.io/crates/raptorq)
[![Documentation](https://docs.rs/raptorq/badge.svg)](https://docs.rs/raptorq)
[![PyPI](https://img.shields.io/pypi/v/raptorq.svg)](https://pypi.org/project/raptorq/)
[![License](https://img.shields.io/crates/l/raptorq)](https://crates.io/crates/raptorq)
[![dependency status](https://deps.rs/repo/github/cberner/raptorq/status.svg)](https://deps.rs/repo/github/cberner/raptorq)

### Status: NO LONGER MAINTAINED; HAS KNOWN BUGS.

### Overview

Rust implementation of RaptorQ (RFC6330)

Recovery properties:
Reconstruction probability after receiving K + h packets = 1 - 1/256^(h + 1). Where K is the number of packets in the
original message, and h is the number of additional packets received.
See "RaptorQ Technical Overview" by Qualcomm

### Examples
See the `examples/` directory for usage.

### Benchmarks

The following were run on a Ryzen 9 5900X @ 3.70GHz
```
Symbol size: 1280 bytes (without pre-built plan)
symbol count = 10, encoded 127 MB in 0.259secs, throughput: 3953.4Mbit/s
symbol count = 100, encoded 127 MB in 0.217secs, throughput: 4716.3Mbit/s
symbol count = 250, encoded 127 MB in 0.215secs, throughput: 4757.9Mbit/s
symbol count = 500, encoded 127 MB in 0.216secs, throughput: 4724.6Mbit/s
symbol count = 1000, encoded 126 MB in 0.221secs, throughput: 4595.6Mbit/s
symbol count = 2000, encoded 126 MB in 0.230secs, throughput: 4415.8Mbit/s
symbol count = 5000, encoded 122 MB in 0.248secs, throughput: 3937.8Mbit/s
symbol count = 10000, encoded 122 MB in 0.289secs, throughput: 3379.1Mbit/s
symbol count = 20000, encoded 122 MB in 0.362secs, throughput: 2697.7Mbit/s
symbol count = 50000, encoded 122 MB in 0.482secs, throughput: 2026.1Mbit/s

Symbol size: 1280 bytes (with pre-built plan)
symbol count = 10, encoded 127 MB in 0.119secs, throughput: 8604.4Mbit/s
symbol count = 100, encoded 127 MB in 0.084secs, throughput: 12183.8Mbit/s
symbol count = 250, encoded 127 MB in 0.092secs, throughput: 11119.0Mbit/s
symbol count = 500, encoded 127 MB in 0.093secs, throughput: 10973.2Mbit/s
symbol count = 1000, encoded 126 MB in 0.093secs, throughput: 10920.7Mbit/s
symbol count = 2000, encoded 126 MB in 0.102secs, throughput: 9957.1Mbit/s
symbol count = 5000, encoded 122 MB in 0.111secs, throughput: 8797.9Mbit/s
symbol count = 10000, encoded 122 MB in 0.138secs, throughput: 7076.5Mbit/s
symbol count = 20000, encoded 122 MB in 0.178secs, throughput: 5486.3Mbit/s
symbol count = 50000, encoded 122 MB in 0.265secs, throughput: 3685.1Mbit/s

Symbol size: 1280 bytes
symbol count = 10, decoded 127 MB in 0.398secs using 0.0% overhead, throughput: 2572.7Mbit/s
symbol count = 100, decoded 127 MB in 0.323secs using 0.0% overhead, throughput: 3168.5Mbit/s
symbol count = 250, decoded 127 MB in 0.302secs using 0.0% overhead, throughput: 3387.2Mbit/s
symbol count = 500, decoded 127 MB in 0.290secs using 0.0% overhead, throughput: 3519.0Mbit/s
symbol count = 1000, decoded 126 MB in 0.309secs using 0.0% overhead, throughput: 3286.8Mbit/s
symbol count = 2000, decoded 126 MB in 0.326secs using 0.0% overhead, throughput: 3115.4Mbit/s
symbol count = 5000, decoded 122 MB in 0.340secs using 0.0% overhead, throughput: 2872.2Mbit/s
symbol count = 10000, decoded 122 MB in 0.374secs using 0.0% overhead, throughput: 2611.1Mbit/s
symbol count = 20000, decoded 122 MB in 0.452secs using 0.0% overhead, throughput: 2160.5Mbit/s
symbol count = 50000, decoded 122 MB in 0.625secs using 0.0% overhead, throughput: 1562.5Mbit/s

symbol count = 10, decoded 127 MB in 0.398secs using 5.0% overhead, throughput: 2572.7Mbit/s
symbol count = 100, decoded 127 MB in 0.324secs using 5.0% overhead, throughput: 3158.8Mbit/s
symbol count = 250, decoded 127 MB in 0.303secs using 5.0% overhead, throughput: 3376.1Mbit/s
symbol count = 500, decoded 127 MB in 0.291secs using 5.0% overhead, throughput: 3506.9Mbit/s
symbol count = 1000, decoded 126 MB in 0.315secs using 5.0% overhead, throughput: 3224.2Mbit/s
symbol count = 2000, decoded 126 MB in 0.328secs using 5.0% overhead, throughput: 3096.4Mbit/s
symbol count = 5000, decoded 122 MB in 0.349secs using 5.0% overhead, throughput: 2798.2Mbit/s
symbol count = 10000, decoded 122 MB in 0.402secs using 5.0% overhead, throughput: 2429.3Mbit/s
symbol count = 20000, decoded 122 MB in 0.500secs using 5.0% overhead, throughput: 1953.1Mbit/s
symbol count = 50000, decoded 122 MB in 0.746secs using 5.0% overhead, throughput: 1309.1Mbit/s
```

The following were run on an Intel Core i5-6600K @ 3.50GHz, as of raptorq version 1.6.4

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
symbol count = 10, encoded 127 MB in 5.078secs, throughput: 201.6Mbit/s
symbol count = 100, encoded 127 MB in 3.966secs, throughput: 258.1Mbit/s
symbol count = 250, encoded 127 MB in 4.293secs, throughput: 238.3Mbit/s
symbol count = 500, encoded 127 MB in 4.451secs, throughput: 229.3Mbit/s
symbol count = 1000, encoded 126 MB in 4.606secs, throughput: 220.5Mbit/s
symbol count = 2000, encoded 126 MB in 5.127secs, throughput: 198.1Mbit/s
symbol count = 5000, encoded 122 MB in 5.615secs, throughput: 173.9Mbit/s
symbol count = 10000, encoded 122 MB in 6.321secs, throughput: 154.5Mbit/s
symbol count = 20000, encoded 122 MB in 7.450secs, throughput: 131.1Mbit/s
symbol count = 50000, encoded 122 MB in 9.407secs, throughput: 103.8Mbit/s

Symbol size: 1280 bytes (with pre-built plan)
symbol count = 10, encoded 127 MB in 3.438secs, throughput: 297.8Mbit/s
symbol count = 100, encoded 127 MB in 2.476secs, throughput: 413.3Mbit/s
symbol count = 250, encoded 127 MB in 2.908secs, throughput: 351.8Mbit/s
symbol count = 500, encoded 127 MB in 3.085secs, throughput: 330.8Mbit/s
symbol count = 1000, encoded 126 MB in 3.284secs, throughput: 309.3Mbit/s
symbol count = 2000, encoded 126 MB in 3.700secs, throughput: 274.5Mbit/s
symbol count = 5000, encoded 122 MB in 4.045secs, throughput: 241.4Mbit/s
symbol count = 10000, encoded 122 MB in 4.451secs, throughput: 219.4Mbit/s
symbol count = 20000, encoded 122 MB in 4.948secs, throughput: 197.4Mbit/s
symbol count = 50000, encoded 122 MB in 6.078secs, throughput: 160.7Mbit/s

Symbol size: 1280 bytes
symbol count = 10, decoded 127 MB in 6.561secs using 0.0% overhead, throughput: 156.1Mbit/s
symbol count = 100, decoded 127 MB in 4.936secs using 0.0% overhead, throughput: 207.3Mbit/s
symbol count = 250, decoded 127 MB in 5.206secs using 0.0% overhead, throughput: 196.5Mbit/s
symbol count = 500, decoded 127 MB in 5.298secs using 0.0% overhead, throughput: 192.6Mbit/s
symbol count = 1000, decoded 126 MB in 5.565secs using 0.0% overhead, throughput: 182.5Mbit/s
symbol count = 2000, decoded 126 MB in 6.309secs using 0.0% overhead, throughput: 161.0Mbit/s
symbol count = 5000, decoded 122 MB in 6.805secs using 0.0% overhead, throughput: 143.5Mbit/s
symbol count = 10000, decoded 122 MB in 7.517secs using 0.0% overhead, throughput: 129.9Mbit/s
symbol count = 20000, decoded 122 MB in 8.875secs using 0.0% overhead, throughput: 110.0Mbit/s
symbol count = 50000, decoded 122 MB in 11.253secs using 0.0% overhead, throughput: 86.8Mbit/s

symbol count = 10, decoded 127 MB in 6.157secs using 5.0% overhead, throughput: 166.3Mbit/s
symbol count = 100, decoded 127 MB in 4.842secs using 5.0% overhead, throughput: 211.4Mbit/s
symbol count = 250, decoded 127 MB in 5.213secs using 5.0% overhead, throughput: 196.2Mbit/s
symbol count = 500, decoded 127 MB in 5.328secs using 5.0% overhead, throughput: 191.5Mbit/s
symbol count = 1000, decoded 126 MB in 5.630secs using 5.0% overhead, throughput: 180.4Mbit/s
symbol count = 2000, decoded 126 MB in 6.364secs using 5.0% overhead, throughput: 159.6Mbit/s
symbol count = 5000, decoded 122 MB in 7.035secs using 5.0% overhead, throughput: 138.8Mbit/s
symbol count = 10000, decoded 122 MB in 8.165secs using 5.0% overhead, throughput: 119.6Mbit/s
symbol count = 20000, decoded 122 MB in 9.929secs using 5.0% overhead, throughput: 98.4Mbit/s
symbol count = 50000, decoded 122 MB in 14.399secs using 5.0% overhead, throughput: 67.8Mbit/s
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
