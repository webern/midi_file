.PHONY: clean
clean:
	cargo clean
	rm -rf target

.PHONY: lint
lint:
	cargo fmt -- --check
	cargo clippy --locked -- -D warnings
	cargo clippy --tests --locked -- -D warnings

.PHONY: build
build:
	cargo build

.PHONY: test
test:
	cargo test

.PHONY: ci
ci: lint build test lint_example run_example

.PHONY: run_example
run_example:
	cargo run --example main

.PHONY: run_example
lint_example:
	cargo clippy --example main -- -D warnings