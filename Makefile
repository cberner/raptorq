.PHONY: build
build: pre
	cargo build

.PHONY: pre
pre:
	cargo deny check licenses
	cargo fmt --all -- --check
	cargo clippy --all

.PHONY: release
release: pre
	cargo build --release

.PHONY: test
test: pre
	cargo build --features benchmarking,python,serde_support
	cargo test --features benchmarking

.PHONY: test_extended
test_extended: pre
	RUSTFLAGS="-C opt-level=3" nice cargo test --features benchmarking -- --ignored --nocapture

.PHONY: bench
bench: pre
	cargo bench --features benchmarking

.PHONY: profile
profile:
	RUSTFLAGS='-Cforce-frame-pointers' cargo bench --no-run --features benchmarking

.PHONY: fuzz
fuzz:
	 cargo fuzz run --sanitizer=none fuzz_raptorq

.PHONY: build_py
build_py: pre
	maturin build

.PHONY: release_py
release_py: pre
	maturin build --release

.PHONY: publish_py
publish_py: test_py
	docker pull quay.io/pypa/manylinux2014_x86_64
	docker run -it --rm -v $(shell pwd):/raptorq-ro:ro quay.io/pypa/manylinux2014_x86_64 /raptorq-ro/py_publish.sh

.PHONY: install_py
install_py: pre
	maturin develop

test_py: install_py
	python3 -m unittest discover

build_wasm: pre
	wasm-pack build --target web --features wasm
