# EDR Desktop Makefile
# Provides convenient commands for development, testing, and golden bundle management

.PHONY: all build test check clean
.PHONY: verify-golden verify-golden-fast gen-golden
.PHONY: server run help

# Default target
all: build

# Build all crates
build:
	cargo build

# Run all tests
test:
	cargo test

# Check all crates (faster than build)
check:
	cargo check

# Clean build artifacts
clean:
	cargo clean

# ============================================================================
# Golden Bundle Commands
# ============================================================================

# Generate all golden bundles
gen-golden:
	@echo "Generating golden bundles..."
	cargo run -p edr-server --bin golden-cli -- generate bundles/golden --regenerate
	@echo "✅ Golden bundles generated in bundles/golden/"

# Verify all golden bundles (strict mode - CI equivalent)
verify-golden:
	@echo "Verifying golden bundles (strict mode)..."
	cargo run -p edr-server --bin golden-cli -- verify --strict bundles/golden

# Verify golden bundles with JSON output
verify-golden-json:
	@echo "Verifying golden bundles with JSON output..."
	cargo run -p edr-server --bin golden-cli -- verify --strict --json golden-verify-results.json bundles/golden
	@echo "Results written to golden-verify-results.json"

# Fast verification (edge cases only, best-effort mode)
verify-golden-fast:
	@echo "Fast golden bundle verification (edge_cases only)..."
	cargo run -p edr-server --bin golden-cli -- verify --families edge_cases bundles/golden

# Verify specific family
verify-golden-family:
	@test -n "$(FAMILY)" || (echo "Usage: make verify-golden-family FAMILY=credential_access" && exit 1)
	cargo run -p edr-server --bin golden-cli -- verify --families $(FAMILY) bundles/golden

# ============================================================================
# Server Commands
# ============================================================================

# Run the server
server:
	cargo run -p edr-server --bin edr-server

# Run the server in release mode
server-release:
	cargo run -p edr-server --bin edr-server --release

# ============================================================================
# Test Commands
# ============================================================================

# Run golden bundle unit tests
test-golden:
	cargo test -p edr-server golden_bundle --lib

# Run all server tests
test-server:
	cargo test -p edr-server

# Run tests with verbose output
test-verbose:
	cargo test -- --nocapture

# ============================================================================
# CI Commands
# ============================================================================

# Full CI validation (what CI runs)
ci: check test verify-golden
	@echo "✅ CI validation passed"

# ============================================================================
# Help
# ============================================================================

help:
	@echo "EDR Desktop Makefile"
	@echo ""
	@echo "Build Commands:"
	@echo "  make build            Build all crates"
	@echo "  make check            Check all crates (faster)"
	@echo "  make clean            Clean build artifacts"
	@echo ""
	@echo "Golden Bundle Commands:"
	@echo "  make gen-golden       Generate all golden bundles"
	@echo "  make verify-golden    Verify all bundles (strict mode)"
	@echo "  make verify-golden-json  Verify with JSON output"
	@echo "  make verify-golden-fast  Fast verify (edge_cases only)"
	@echo "  make verify-golden-family FAMILY=<name>"
	@echo ""
	@echo "Test Commands:"
	@echo "  make test             Run all tests"
	@echo "  make test-golden      Run golden bundle tests"
	@echo "  make test-server      Run server tests"
	@echo ""
	@echo "Server Commands:"
	@echo "  make server           Run the server"
	@echo "  make server-release   Run in release mode"
	@echo ""
	@echo "CI Commands:"
	@echo "  make ci               Full CI validation"
