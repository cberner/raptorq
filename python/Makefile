build:
	maturin build

release:
	maturin build --release

install:
	maturin develop

lint:
	cargo lichking check
	cargo fmt --all -- --check
	cargo clippy --all

test: install
	python3 -m unittest discover