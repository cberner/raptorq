build:
	cargo build

release:
	cargo build --release

test:
	cargo test

bench:
	cargo bench

profile:
	RUSTFLAGS='-Cforce-frame-pointers' cargo bench --no-run
