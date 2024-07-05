nightly_version := prepend('+', `cat rust-nightly-version`)

help:
    @echo fmt, clippy, clippy-fix, check or clean

clean:
    cargo clean

fmt *CHECK:
    cargo {{nightly_version}} fmt --all {{ if CHECK == "check" { "-- --check" } else { "" } }}

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
