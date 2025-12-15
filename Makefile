# Makefile for code-analyzer

# Configuration
BINARY_NAME := code-analyzer
INSTALL_DIR := $(HOME)/.local/bin

.PHONY: all build release check test lint fmt install uninstall clean help

# Default target
all: build

# Development build
build:
	cargo build

# Optimized release build
release:
	cargo build --release

# Fast syntax check
check:
	cargo check

# Run all tests
test:
	cargo test

# Run clippy linter (strict mode)
lint:
	cargo clippy -- -D warnings

# Format code
fmt:
	cargo fmt

# Full quality check (run before committing)
quality: fmt
	cargo fmt --check
	cargo clippy -- -D warnings
	cargo test

# Install binary to ~/.local/bin
install: release
	@mkdir -p $(INSTALL_DIR)
	@cp target/release/$(BINARY_NAME) $(INSTALL_DIR)/
	@chmod +x $(INSTALL_DIR)/$(BINARY_NAME)
	@echo "Installed $(BINARY_NAME) to $(INSTALL_DIR)"
	@echo "Make sure $(INSTALL_DIR) is in your PATH"

# Uninstall binary
uninstall:
	@rm -f $(INSTALL_DIR)/$(BINARY_NAME)
	@echo "Removed $(BINARY_NAME) from $(INSTALL_DIR)"

# Clean build artifacts
clean:
	cargo clean

# Show help
help:
	@echo "Available targets:"
	@echo "  build     - Development build"
	@echo "  release   - Optimized release build"
	@echo "  check     - Fast syntax check"
	@echo "  test      - Run all tests"
	@echo "  lint      - Run clippy linter"
	@echo "  fmt       - Format code"
	@echo "  quality   - Full quality check (fmt + lint + test)"
	@echo "  install   - Install binary to ~/.local/bin"
	@echo "  uninstall - Remove installed binary"
	@echo "  clean     - Clean build artifacts"
	@echo "  help      - Show this help"
