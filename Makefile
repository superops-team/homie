.PHONY: fmt fmt-check lint test test-cli-runtime-clean security smoke package-shell-test parity-lock ui-screenshot-gate module-inventory-check spec-diri-mapping-check app package dmg pre-commit full-check

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

lint:
	cargo clippy --workspace --all-targets -- -D warnings

test:
	cargo test --workspace

test-cli-runtime-clean:
	cargo clean
	cargo build -p homie-runtime --bin homie-runtime-daemon --bin homie-runtime-holder
	cargo test -p homie-cli -- --test-threads=1

security:
	.githooks/pre-commit

smoke:
	@# Real gate: assemble -> verify one app closure -> run one packaged runtime smoke.
	@set -eu; \
	package_output="$$(scripts/package/package.sh)"; \
	printf '%s\n' "$$package_output"; \
	app_path="$$(printf '%s\n' "$$package_output" | sed -n 's/^APP_PATH=//p')"; \
	test -n "$$app_path"; \
	sh scripts/package/tests/verify-app-binary.sh "$$app_path"; \
	sh scripts/package/tests/smoke-packaged-runtime.sh "$$app_path"

package-shell-test:
	@# Fast contract tests for package assembly/verification and smoke orchestration.
	sh scripts/package/tests/package-closure-test.sh
	sh scripts/package/tests/smoke-packaged-runtime-test.sh

parity-lock:
	scripts/quality/loopx-diri-parity-lock.sh

ui-screenshot-gate:
	python3 scripts/quality/check-ui-screenshot-evidence.py

module-inventory-check:
	python3 scripts/quality/check-diri-module-inventory.py

spec-diri-mapping-check:
	python3 scripts/quality/check-diri-spec-mapping.py

app:
	scripts/package/package.sh

package: app

dmg:
	scripts/package/dmg.sh

pre-commit: fmt-check lint test security

full-check: pre-commit package-shell-test smoke
