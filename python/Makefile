build:
	maturin build

release:
	maturin build --release

install:
	maturin develop

lint:
	cargo deny check licenses
	cargo fmt --all -- --check

test: install
	python3 -m unittest discover