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

.PHONY: prepare
prepare:
	# Installs target if not present (redundant if done in CI, but good for local)
	rustup target add x86_64-unknown-linux-musl

.PHONY: build-alpine
build-alpine:
	# Build release version using musl (static linking for Alpine)
	# We output specifically to ./build/app to match your Dockerfile expectation
	cargo build --release --target x86_64-unknown-linux-musl
	mkdir -p ./build
	# DO NOTE that release/<package-name-in-cargo-toml>
	cp ./target/x86_64-unknown-linux-musl/release/gemini-assisted-axum ./build/app
