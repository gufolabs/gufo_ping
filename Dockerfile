FROM python:3.10-slim-bullseye AS dev
COPY .requirements /tmp
ENV \
    PATH=/usr/local/cargo/bin:$PATH\
    RUSTUP_HOME=/usr/local/rustup\
    CARGO_HOME=/usr/local/cargo\
    RUST_VERSION=1.60.0\
    RUST_ARCH=x86_64-unknown-linux-gnu\
    RUSTUP_SHA=3dc5ef50861ee18657f9db2eeb7392f9c2a6c95c90ab41e45ab4ca71476b4338
RUN \
    apt-get update \
    && apt-get install -y --no-install-recommends\
    git\
    ca-certificates\
    gcc\
    libc6-dev\
    curl\
    && curl -o rustup-init https://sh.rustup.rs \
    && chmod a+x rustup-init\
    && ./rustup-init -y --no-modify-path --profile minimal\
    --default-toolchain ${RUST_VERSION} --default-host ${RUST_ARCH}\
    && rm rustup-init\
    && cargo --version\
    && rustup --version\
    && rustc --version\
    && rustup component add\
    rust-analysis\
    rust-src \
    rls\
    clippy\
    rustfmt\
    && pip install --upgrade pip\
    && pip install --upgrade build\
    && pip install\
    -r /tmp/build.txt\
    -r /tmp/docs.txt\
    -r /tmp/ipython.txt\
    -r /tmp/lint.txt\
    -r /tmp/test.txt