# --- Configuration ---
# Versions to install
AGE_VERSION := v1.2.0
SOPS_VERSION := v3.9.1
RATCHET_VERSION := 0.11.4

# Environment for the Age File
# Check if the SOPS_AGE_KEY environment variable is set or not, if set, then use that.
# If not then we are using SOPS_AGE_KEY_FILE
ifndef SOPS_AGE_KEY
    export SOPS_AGE_KEY_FILE := $(shell pwd)/.secrets/keys.txt
endif

# APP Configuration
APP_NAME := gemini-assisted-axum

# Directory to place executables
BIN_DIR := ./bin

# Define a default, but allow the environment to override it
EDITOR ?= vim

# --- Auto-Detection ---
# Detect OS (linux/darwin)
OS := $(shell uname -s | tr '[:upper:]' '[:lower:]')

# Detect Architecture and map to standard names (amd64/arm64)
ARCH := $(shell uname -m)
ifeq ($(ARCH),x86_64)
    ARCH := amd64
else ifeq ($(ARCH),aarch64)
    ARCH := arm64
endif

# --- Download URLs for Tools ---
# Age uses tarballs
AGE_URL := https://github.com/FiloSottile/age/releases/download/$(AGE_VERSION)/age-$(AGE_VERSION)-$(OS)-$(ARCH).tar.gz

# Sops uses standalone binaries
SOPS_URL := https://github.com/getsops/sops/releases/download/$(SOPS_VERSION)/sops-$(SOPS_VERSION).$(OS).$(ARCH)

# Ratchet uses tarballs
RATCHET_URL := https://github.com/sethvargo/ratchet/releases/download/v$(RATCHET_VERSION)/ratchet_$(RATCHET_VERSION)_$(OS)_$(ARCH).tar.gz

# --- Tools ---

.PHONY: install-tools clean-tools ensure-bin-dir install-ratchet sort-install

# Main target to run
install-tools: ensure-bin-dir install-age install-sops install-ratchet sort-install
	@echo "✅ Installation complete! Executables are in $(BIN_DIR)"
	@echo "👉 usage: $(BIN_DIR)/sops --version"

ensure-bin-dir:
	@mkdir -p $(BIN_DIR)

install-age:
	@echo "⬇️  Downloading age $(AGE_VERSION) for $(OS)/$(ARCH)..."
	@curl -L -o $(BIN_DIR)/age.tar.gz $(AGE_URL)
	@echo "📦 Extracting age..."
	@tar -xzf $(BIN_DIR)/age.tar.gz -C $(BIN_DIR) --strip-components=1 age/age age/age-keygen
	@rm $(BIN_DIR)/age.tar.gz
	@chmod +x $(BIN_DIR)/age $(BIN_DIR)/age-keygen

install-sops:
	@echo "⬇️  Downloading sops $(SOPS_VERSION) for $(OS)/$(ARCH)..."
	@curl -L -o $(BIN_DIR)/sops $(SOPS_URL)
	@chmod +x $(BIN_DIR)/sops

install-ratchet:
	@echo "⬇️  Downloading ratchet v$(RATCHET_VERSION) for $(OS)/$(ARCH)..."
	@curl -L -o $(BIN_DIR)/ratchet.tar.gz $(RATCHET_URL)
	@echo "📦 Extracting ratchet..."
	@tar -xzf $(BIN_DIR)/ratchet.tar.gz -C $(BIN_DIR) ratchet
	@rm $(BIN_DIR)/ratchet.tar.gz
	@chmod +x $(BIN_DIR)/ratchet
	@echo "✨ Ratchet installed."

sort-install:
	cargo install cargo-sort@2.0.2

clean-tools:
	@rm -rf $(BIN_DIR)/age $(BIN_DIR)/age-keygen $(BIN_DIR)/sops
	@echo "🧹 Cleaned up executables."


# --- Initialization commands ---

# Run this once after cloning the repo
# This is used to hooks initialization and other tools that related to projects
.PHONY: init
init: install-tools
	git config core.hooksPath .githooks
	chmod +x .githooks/pre-commit
	@echo "✅ Git hooks configured successfully!"

# --- Configuration build commands
# Define the path to your sops binary
SOPS := ./bin/sops
CFG_DIR := ./etc/cfg

decrypt-conf-%:
	@echo "🔓 Decrypting configuration for environment: $*"
	@$(SOPS) --decrypt $(CFG_DIR)/conf.$*.enc.json > $(CFG_DIR)/conf.json

.PHONY: decrypt-local decrypt-staging decrypt-prod

decrypt-local: decrypt-conf-local
decrypt-staging: decrypt-conf-staging
decrypt-prod: decrypt-conf-production

encrypt-conf-%:
	@echo "🔒 Encrypting current conf.json into environment: $*"
	@$(SOPS) --encrypt $(CFG_DIR)/conf.json > $(CFG_DIR)/conf.$*.enc.json
	@echo "✅ Saved to $(CFG_DIR)/conf.$*.enc.json"

.PHONY: encrypt-local encrypt-staging encrypt-pro

encrypt-local: encrypt-conf-local
encrypt-staging: encrypt-conf-staging
encrypt-prod: encrypt-conf-production

edit-conf-%:
	@echo "📝 Opening $* config using $(EDITOR)..."
	@EDITOR="$(EDITOR)" $(SOPS) $(CFG_DIR)/conf.$*.enc.json
	@echo "✅ Changes encrypted and saved."

.PHONY: conf-local conf-staging conf-prod

conf-local: edit-conf-local
conf-staging: edit-conf-staging
conf-prod: edit-conf-prod

.PHONY: ratchet-update ratchet-check ratchet-upgrade ratchet-unpin

# Updates (pins) all your workflow files to the latest SHAs
ratchet-update:
	@echo "🔒 Pinning GitHub Actions to latest SHAs..."
	@# We use \( -name ... -o -name ... \) to find both .yml and .yaml
	@find .github/workflows \( -name "*.yml" -o -name "*.yaml" \) -exec $(BIN_DIR)/ratchet pin {} \;
	@echo "✅ Workflows pinned successfully."

# Checks if workflows are unpinned (fails the build if they are)
ratchet-check:
	@echo "🕵️  Checking for unpinned actions..."
	@# We use 'xargs' here because standard 'find -exec' swallows exit codes.
	@# 'xargs' ensures that if ratchet fails on ANY file, the make command fails.
	@find .github/workflows \( -name "*.yml" -o -name "*.yaml" \) -print0 | xargs -0 -n1 $(BIN_DIR)/ratchet lint
	@echo "✅ All actions are securely pinned."

# Always check the compatibility
ratchet-upgrade:
	@echo "🚀 Upgrading GitHub Actions to NEWEST versions (Major version bumps possible)..."
	@find .github/workflows \( -name "*.yml" -o -name "*.yaml" \) -exec $(BIN_DIR)/ratchet upgrade {} \;
	@echo "⚠️  Workflows upgraded. Please manually verify compatibility!"

ratchet-unpin:
	@echo "🔓 Unpinning workflows..."
	@find .github/workflows \( -name "*.yml" -o -name "*.yaml" \) -exec $(BIN_DIR)/ratchet unpin {} \;
	@echo "✅ Workflows unpinned."

# --- Daily Usage commands

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
check: ratchet-check
	cargo check

.PHONY: clean
clean: clean-tools
	cargo clean

.PHONY: run
run:
	cargo run

.PHONY: sort
sort:
	cargo sort

# --- Github Action builds commands

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
	cp ./target/x86_64-unknown-linux-musl/release/$(APP_NAME) ./build/app
