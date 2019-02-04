build:
	cargo build

release:
	cargo build --release

test:
	cargo test

profile:
	RUSTFLAGS=-Cforce-frame-pointers cargo bench --no-run
