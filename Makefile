# log-sender makefile

.PHONY: all test bump

all: test

test:
	cargo fmt -- --check
	cargo clippy --locked -- -D warnings
	RUSTFLAGS="-D warnings" cargo test --locked --all-features --all-targets
	@if [ "${CI}x" != "x" ]; then git diff --exit-code; fi
