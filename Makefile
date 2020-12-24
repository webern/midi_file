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
ci: lint build test
