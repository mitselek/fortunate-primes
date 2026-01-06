.PHONY: build release test fmt lint clean bench run help

build:
	cargo build

release:
	cargo build --release

test:
	cargo test

test-verbose:
	cargo test -- --nocapture

fmt:
	cargo fmt

fmt-check:
	cargo fmt -- --check

lint:
	cargo clippy -- -D warnings

check:
	cargo check

clean:
	cargo clean

bench: release
	./benchmark.sh

run: release
	./target/release/fortunate

help:
	@echo "Available targets:"
	@echo "  build       - Debug build"
	@echo "  release     - Optimized release build"
	@echo "  test        - Run all tests"
	@echo "  test-verbose - Run tests with output"
	@echo "  fmt         - Format code"
	@echo "  fmt-check   - Check formatting"
	@echo "  lint        - Run clippy linter"
	@echo "  check       - Check without building"
	@echo "  clean       - Remove build artifacts"
	@echo "  bench       - Run benchmarks"
	@echo "  run         - Build and run calculator"
