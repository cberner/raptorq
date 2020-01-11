build: pre
	cargo build

pre:
	cargo lichking check
	cargo fmt --all -- --check
	cargo clippy --all

release: pre
	cargo build --release

test: pre
	cargo test --features benchmarking

test_extended: pre
	RUSTFLAGS="-C opt-level=3" cargo test --features benchmarking -- --ignored --nocapture

bench: pre
	cargo bench --features benchmarking

profile:
	RUSTFLAGS='-Cforce-frame-pointers' cargo bench --no-run --features benchmarking

build_py:
	$(MAKE) -C python build
