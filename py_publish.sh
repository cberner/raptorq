#!/bin/bash

PYTHON3=/opt/python/cp311-cp311/bin/python3

cp -r /raptorq-ro /raptorq
cd /raptorq

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain=$(cat ./rust-toolchain)
source $HOME/.cargo/env

cd /tmp
$PYTHON3 -m venv venv
cd /raptorq
source /tmp/venv/bin/activate
python3 -m pip install --upgrade pip
python3 -m pip install 'maturin>=1.0,<2.0'

python3 -m maturin publish --username __token__
