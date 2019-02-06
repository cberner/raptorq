build:
	cargo build

release:
	cargo build --release

test:
	RUSTFLAGS='-C target-feature=+avx2' cargo test

bench:
	RUSTFLAGS='-C target-feature=+avx2' cargo bench

profile:
	RUSTFLAGS=-Cforce-frame-pointers cargo bench --no-run
