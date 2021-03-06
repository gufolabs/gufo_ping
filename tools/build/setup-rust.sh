#!/bin/sh
# ------------------------------------------------------------------------
# Gufo Labs: Install and setup rust
# ------------------------------------------------------------------------
# Copyright (C) 2022, Gufo Labs
# ------------------------------------------------------------------------

set -x
set -e

RUST_VERSION=1.60.0

# @todo: Allow override
export RUSTUP_HOME=${RUSTUP_HOME:-/usr/local/rustup}
export CARGO_HOME=${CARGO_HOME:-/usr/local/cargo}
export PATH=${CARGO_HOME}/bin:${PATH}

# Detect RUST_ARCH
case "${SETUP_RUST_PLATFORM}" in
    # manylinux
    "manylinux-x86_64")
        RUST_ARCH=x86_64-unknown-linux-gnu
        ;;
    "manylinux-aarch64")
        RUST_ARCH=aarch64-unknown-linux-gnu
        ;;
    # musllinux
    "musllinux-x86_64")
        RUST_ARCH=x86_64-unknown-linux-musl
        ;;
    "musllinux-aarch64")
        RUST_ARCH=aarch64-unknown-linux-musl
        ;;
    # macosx
    "macosx-x86_64")
        RUST_ARCH=x86_64-apple-darwin
        ;;
    # Rust not ready yet
    # "macosx-arm64")
    #     RUST_ARCH=aarch64-unknown-linux-musl
    #     ;;
    # default
    *)
        RUST_ARCH=x86_64-unknown-linux-gnu
        ;;
esac

echo "Install Rust ${RUST_ARCH}"
echo "PATH        = ${PATH}"
echo "RUSTUP_HOME = ${RUSTUP_HOME}"
echo "CARGO_HOME  = ${CARGO_HOME}"

# Install rust
curl -o rustup-init https://sh.rustup.rs
chmod a+x rustup-init
./rustup-init -y --no-modify-path --profile minimal\
    --default-toolchain ${RUST_VERSION}\
    --default-host ${RUST_ARCH}
rm rustup-init
# Check
cargo --version
rustc --version
