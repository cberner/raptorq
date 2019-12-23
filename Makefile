build:
	cargo build

release:
	cargo build --release

test:
	cargo test --features benchmarking

test_extended:
	RUSTFLAGS="-C opt-level=3" cargo test --features benchmarking -- --ignored --nocapture

bench:
	cargo bench --features benchmarking

profile:
	RUSTFLAGS='-Cforce-frame-pointers' cargo bench --no-run --features benchmarking
