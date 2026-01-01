# Run this once after cloning the repo
.PHONY: init
init:
	git config core.hooksPath .githooks
	chmod +x .githooks/pre-commit
	@echo "✅ Git hooks configured successfully!"

.PHONY: lint
lint:
	cargo clippy -- -D warnings

.PHONY: format
format:
	cargo +nightly fmt

.PHONY: format-check
format-check:
	cargo +nightly fmt -- --check

.PHONY: format-lint
format-lint: lint
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

.PHONY: sort-install
sort-install:
	cargo install cargo-sort@2.0.2

.PHONY: sort
sort:
	cargo sort
