build: pre
    cargo build

pre:
    cargo deny --all-features check licenses
    cargo fmt --all -- --check
    cargo clippy --all

release: pre
    cargo build --release

test: pre
    cargo build --features benchmarking,python,serde_support
    cargo test --features benchmarking

test_extended: pre
    RUSTFLAGS="-C opt-level=3" nice cargo test --features benchmarking -- --ignored --nocapture

bench: pre
    cargo bench --features benchmarking

profile:
    RUSTFLAGS='-Cforce-frame-pointers' cargo bench --no-run --features benchmarking

fuzz:
    cargo fuzz run --sanitizer=none --release fuzz_raptorq

build_py: pre
    maturin build

release_py: pre
    maturin build --release

publish_py: test_py
    docker pull quay.io/pypa/manylinux2014_x86_64
    @MATURIN_PYPI_TOKEN=$(cat ~/.pypi/raptorq_token) docker run -it --rm -e "MATURIN_PYPI_TOKEN" -v $(pwd):/raptorq-ro:ro quay.io/pypa/manylinux2014_x86_64 /raptorq-ro/py_publish.sh

install_py: pre
    maturin develop

test_py: install_py
    python3 -m unittest discover
