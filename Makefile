.PHONY: fmt fmt-check lint test security pre-commit

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

lint:
	cargo clippy --workspace --all-targets -- -D warnings

test:
	cargo test --workspace

security:
	.githooks/pre-commit

pre-commit: fmt-check lint test security
