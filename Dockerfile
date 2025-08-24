FROM python:3.13-slim-bullseye AS dev
COPY . /workspaces/gufo_ping
WORKDIR /workspaces/gufo_ping
ENV \
    PATH=/usr/local/cargo/bin:$PATH\
    RUSTUP_HOME=/usr/local/rustup\
    CARGO_HOME=/usr/local/cargo\
    RUST_ARCH=x86_64-unknown-linux-gnu
RUN \
    set -x \
    && apt-get update \
    && apt-get install -y --no-install-recommends\
    git\
    ca-certificates\
    gcc\
    libc6-dev\
    curl\
    && ./tools/build/setup-rust.sh \
    && rustup component add\
    rust-analysis\
    rust-src \
    rls\
    clippy\
    rustfmt\
    && pip install --upgrade pip\
    && pip install --upgrade build\
    && pip install -e .[build,docs,ipython,lint,test]
