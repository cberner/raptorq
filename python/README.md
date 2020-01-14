The Python bindings are generated using [pyo3](https://github.com/PyO3/pyo3). 
Rust 1.37.0-nightly or higher is required for building [pyo3](https://github.com/PyO3/pyo3) projects.
```
$ rustup install nightly
$ rustup override set nightly
```

Some operating systems require additional packages to be installed.
```
$ sudo apt install python3-dev
```

[maturin](https://github.com/PyO3/maturin) is recommended for building this crate.
```
$ pip install maturin
$ maturin build
```

Alternatively, refer to the [Building and Distribution section](https://pyo3.rs/v0.8.5/building_and_distribution.html) in the [pyo3 user guide](https://pyo3.rs/v0.8.5/).