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

# Download the `python-dist` artifact from the Wheels workflow into
# target/wheels before publishing. The PyPI token never leaves this machine.
publish_py: test_py
    @test -n "$(find target/wheels -maxdepth 1 -type f \( -name '*.whl' -o -name '*.tar.gz' \) -print -quit 2>/dev/null)" || (echo "No release artifacts in target/wheels. Download and extract the python-dist artifact from the Wheels workflow first." >&2; exit 1)
    @MATURIN_PYPI_TOKEN=$(cat ~/.pypi/raptorq_token) maturin upload --skip-existing target/wheels/*

install_py: pre
    maturin develop

test_py: install_py
    python3 -m unittest discover
