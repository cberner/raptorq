build: pre
    cargo build

pre:
    cargo deny --all-features check licenses
    cargo fmt --all -- --check
    cargo clippy --all --all-targets -- -Dwarnings

release: pre
    cargo build --release

test: pre
    cargo build --features benchmarking,python,serde_support
    cargo test --features benchmarking

test_extended: pre
    RUSTFLAGS="-C opt-level=3" nice cargo test --features benchmarking -- --ignored --nocapture

bench: pre
    cargo bench --features benchmarking

profile:
    RUSTFLAGS='-Cforce-frame-pointers' cargo bench --no-run --features benchmarking

fuzz:
    cargo fuzz run --sanitizer=none --release fuzz_raptorq

build_py: pre
    maturin build

release_py: pre
    maturin build --release

# Upload the wheels and sdist built by the Wheels workflow for HEAD (push the tag and wait for it to pass first)
publish_py: test_py
    #!/usr/bin/env bash
    set -euo pipefail
    commit=$(git rev-parse HEAD)
    # Skip pull_request runs: they build a merge commit, not the commit that is checked out.
    run_id=$(gh run list --repo cberner/raptorq --workflow wheels.yml --commit "$commit" --status success --limit 20 --json databaseId,event --jq '[.[] | select(.event != "pull_request")][0].databaseId // empty')
    if [ -z "$run_id" ]; then
        echo "No successful Wheels workflow run found for $commit" >&2
        exit 1
    fi
    rm -rf target/python-dist
    gh run download "$run_id" --repo cberner/raptorq --name python-dist --dir target/python-dist
    just check_py_dist target/python-dist
    # `maturin upload` is deprecated (PyO3/maturin#2334); upload with uv instead
    UV_PUBLISH_TOKEN=$(cat ~/.pypi/raptorq_token) uv publish target/python-dist/*

# Check that `dir` holds exactly the 5 wheels and the sdist for the version in Cargo.toml, and that none are corrupt
check_py_dist dir:
    #!/usr/bin/env bash
    set -euo pipefail
    shopt -s nullglob
    dir="{{dir}}"
    version=$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -n 1)
    expected=(
        "raptorq-$version-*manylinux_2_17_x86_64*.whl"
        "raptorq-$version-*manylinux_2_17_aarch64*.whl"
        "raptorq-$version-*-macosx_*_x86_64.whl"
        "raptorq-$version-*-macosx_*_arm64.whl"
        "raptorq-$version-*-win_amd64.whl"
        "raptorq-$version.tar.gz"
    )
    for pattern in "${expected[@]}"; do
        matches=("$dir"/$pattern)
        if [ "${#matches[@]}" -ne 1 ]; then
            echo "Expected exactly one $pattern in $dir, found ${#matches[@]}" >&2
            exit 1
        fi
    done
    all=("$dir"/*)
    if [ "${#all[@]}" -ne "${#expected[@]}" ]; then
        echo "Expected ${#expected[@]} files in $dir, found ${#all[@]}:" >&2
        ls -l "$dir" >&2
        exit 1
    fi
    for wheel in "$dir"/*.whl; do
        # testzip() returns the first entry whose CRC doesn't match (`python3 -m zipfile -t` exits 0 even then)
        if ! python3 -c 'import sys, zipfile; sys.exit(zipfile.ZipFile(sys.argv[1]).testzip() is not None)' "$wheel" > /dev/null 2>&1; then
            echo "Corrupt wheel: $wheel" >&2
            exit 1
        fi
    done
    if ! tar -tzf "$dir/raptorq-$version.tar.gz" > /dev/null 2>&1; then
        echo "Corrupt sdist: $dir/raptorq-$version.tar.gz" >&2
        exit 1
    fi
    ls -l "$dir"
    echo "OK: $dir has all release files for raptorq $version"

install_py: pre
    maturin develop

test_py: install_py
    python3 -m unittest discover
