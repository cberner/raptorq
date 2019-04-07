build:
	cargo build

release:
	cargo build --release

test:
	cargo test --features benchmarking

bench:
	cargo bench --features benchmarking

profile:
	RUSTFLAGS='-Cforce-frame-pointers' cargo bench --no-run --features benchmarking
