cargo := $(env) cargo
rustup := $(env) rustup
nightly := $(shell cat rust-nightly-version)

clippy:
	$(cargo) +$(nightly) clippy --all-targets -D warnings

fmt:
	$(cargo) +$(nightly) fmt --all

clippy-fix:
	$(cargo) +$(nightly) clippy --fix --allow-dirty --allow-staged

check:
	$(cargo) check