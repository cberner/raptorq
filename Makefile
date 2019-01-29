build:
	cargo build

test:
	cargo test

profile:
	RUSTFLAGS=-Cforce-frame-pointers cargo bench --no-run
