nightly_version := prepend('+', `cat rust-nightly-version`)

help:
    @echo fmt, clippy, clippy-fix, check or clean

clean:
    cargo clean

fmt:
    cargo {{nightly_version}} fmt --all

clippy:
    cargo {{nightly_version}} clippy

clippy-fix:
    cargo {{nightly_version}} clippy --fix --allow-dirty --allow-staged

check:
    cargo check

build:
    cargo build 

build-release:
    cargo build --release