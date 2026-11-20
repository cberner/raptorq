build: pre
    cargo build

pre:
    cargo deny --all-features check licenses
    cargo fmt --all -- --check
    cargo clippy --all --all-targets -- -Dwarnings

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

install_py: pre
    maturin develop

test_py: install_py
    python3 -m unittest discover
