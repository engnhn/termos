.PHONY: check format lint test release

check: format lint test

format:
	cargo fmt --check

lint:
	cargo clippy --all-targets --locked -- -D warnings

test:
	cargo test --locked

release: check
	cargo build --locked --release
