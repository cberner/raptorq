#!/bin/bash

cd /raptorq
yum install -y python3-pip
pip3 install maturin
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain=nightly-2020-03-12
source $HOME/.cargo/env

# xargs is just to merge the lines together into a single line
RUSTUP_TOOLCHAIN="nightly" maturin publish --cargo-extra-args="--features python" \
 -i $(ls -1 /opt/python/*/bin/python3 | xargs | sed 's/ / -i /g')
