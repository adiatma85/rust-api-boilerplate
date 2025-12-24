.PHONY: lint
lint:
	cargo clippy -- -D warnings

.PHONY: format
format: lint
	cargo +nightly fmt

.PHONY: check
check:
	cargo check

.PHONY: clean
clean:
	cargo clean

.PHONY: run
run:
	cargo run
