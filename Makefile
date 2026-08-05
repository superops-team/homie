.PHONY: fmt fmt-check lint test security smoke package pre-commit full-check

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

smoke:
	@tmpdir="$$(mktemp -d)"; \
	cargo run -q -p homie-cli -- doctor --data-dir "$$tmpdir" --json >/dev/null; \
	cargo run -q -p homie-cli -- runtime status --data-dir "$$tmpdir" --json >/dev/null; \
	cargo run -q -p homie-cli -- session create --data-dir "$$tmpdir" --workspace "$$(pwd)" --title Smoke --json >/dev/null; \
	cargo run -q -p homie-cli -- session list --data-dir "$$tmpdir" --json >/dev/null

package:
	scripts/package/package.sh

pre-commit: fmt-check lint test security

full-check: pre-commit smoke package
