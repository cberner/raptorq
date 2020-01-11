build: pre
	maturin build

pre:
	cargo lichking check
	cargo fmt --all -- --check
	cargo clippy --all