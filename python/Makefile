build:
	maturin build

release:
	maturin build --release

lint:
	cargo lichking check
	cargo fmt --all -- --check
	cargo clippy --all
