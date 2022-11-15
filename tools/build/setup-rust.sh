#!/bin/sh
# ------------------------------------------------------------------------
# Gufo Labs: Install and setup rust
# ------------------------------------------------------------------------
# Copyright (C) 2022, Gufo Labs
# ------------------------------------------------------------------------

set -x
set -e

if [ -z "${RUST_ARCH}" ]; then
    echo "RUST_ARCH is not set"
    exit 2
fi

RUST_VERSION=1.65.0

# @todo: Allow override
export RUSTUP_HOME=${RUSTUP_HOME:-/usr/local/rustup}
export CARGO_HOME=${CARGO_HOME:-/usr/local/cargo}
export PATH=${CARGO_HOME}/bin:${PATH}

echo "Install Rust ${RUST_ARCH}"
echo "PATH        = ${PATH}"
echo "RUSTUP_HOME = ${RUSTUP_HOME}"
echo "CARGO_HOME  = ${CARGO_HOME}"

# Install rust
curl -s --tlsv1.2 -o rustup-init https://sh.rustup.rs
# rustup-init tries to check /proc/self/exe
# which is not accessible during Docker build
# on aarch64
# echo "Patching rustup"
# sed -i.bak 's#/proc/self/exe#/bin/sh#g' rustup-init
# rm rustup-init.bak
#
chmod a+x rustup-init
./rustup-init -y --no-modify-path --profile minimal\
    --default-toolchain ${RUST_VERSION}\
    --default-host ${RUST_ARCH}
rm rustup-init
# Check
cargo --version
rustc --version
