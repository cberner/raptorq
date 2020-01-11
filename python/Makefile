build: pre
	maturin build

release: pre
	maturin build --release

pre:
	cargo lichking check
	cargo fmt --all -- --check
	cargo clippy --all