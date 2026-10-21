.DEFAULT_GOAL := help
SHARED_TOOLING_RUST := 1
FORMAT_WRITE_COMMAND = bash -ec '"$$1" sort --workspace --grouped; "$$1" fmt --all; rustfmt --edition 2024 scripts/fixtures/cdk-runtime/*.rs scripts/fixtures/secp-scaffolding/main.rs scripts/fixtures/type-benchmark/main.rs scripts/fixtures/parser-fuzz/main.rs scripts/benchmarks/0.4/principal-source/main.rs' -- "$(FORMAT_CARGO)"
FORMAT_CHECK_COMMAND = bash -ec '"$$1" sort --workspace --grouped --check; "$$1" fmt --all -- --check; rustfmt --edition 2024 --check scripts/fixtures/cdk-runtime/*.rs scripts/fixtures/secp-scaffolding/main.rs scripts/fixtures/type-benchmark/main.rs scripts/fixtures/parser-fuzz/main.rs scripts/benchmarks/0.4/principal-source/main.rs' -- "$(FORMAT_CARGO)"
export RELEASE_DELIVERY := direct
export RELEASE_DELIVERY_POLICY := $(CURDIR)/ci/release-delivery.txt
LOCAL_TOOL_INSTALL_TARGETS += install-release-editor
LOCAL_TOOL_CHECK_TARGETS += release-editor-check
include make/consumer.mk
export YQ := $(CURDIR)/.tools/host/bin/yq
release-patch release-minor release-major release-resume: export IC_PLATFORM_RELEASE_PREPARE := 1

.PHONY: help check test clippy docs-check wasm-check msrv check-boundary check-doc-links check-pins ci check-consumer check-features
.PHONY: version release-version release-preflight release-verify release-prepare-version release-files
.PHONY: release-prepared-check release-commit-check release-committed-check release-tagged-check release-push-check
.PHONY: check-release-commands
.PHONY: publish publish-check

help:
	@echo 'Setup: install-tools, tools-check, install-hooks'
	@echo 'Focused: check, test, clippy, docs-check, wasm-check, msrv, check-boundary'
	@echo 'Full release gate: ci; authorized releases: release-patch, release-minor, release-major'
	@echo 'Feature graphs: check-features (routine); check-features-exhaustive (manual audit)'
	@echo 'Registry: publish-check (dry run), publish (upload current committed version)'
	@echo 'Release editor: install-release-editor; release-editor-check (offline)'

.PHONY: install-release-editor release-editor-check
install-release-editor:
	+bash scripts/release/editor.sh install
release-editor-check:
	+bash scripts/release/editor.sh check

publish: host-tools-check
	bash scripts/release/publish.sh publish
publish-check: host-tools-check
	bash scripts/release/publish.sh check

check:
	cargo check -p ic-platform -p ic-platform-macros --all-targets --all-features --locked
test:
	cargo test -p ic-platform -p ic-platform-macros --all-features --locked
clippy:
	cargo clippy -p ic-platform -p ic-platform-macros --all-targets --all-features --locked -- -D warnings
docs-check:
	RUSTDOCFLAGS='-D warnings' cargo doc -p ic-platform -p ic-platform-macros --all-features --no-deps --locked
wasm-check:
	cargo check -p ic-platform --lib --all-features --target wasm32-unknown-unknown --locked
msrv:
	cargo +1.88.0 test -p ic-platform -p ic-platform-macros --all-features --locked
	cargo +1.88.0 check -p ic-platform --lib --all-features --target wasm32-unknown-unknown --locked
check-boundary:
	bash scripts/ci/check-cargo-dependency-boundaries.sh --consumer "$(CURDIR)"
check-consumer:
	bash scripts/ci/test-platform-consumer-admission.sh
	bash scripts/ci/check-platform-consumer.sh
check-features:
	bash scripts/ci/test-platform-feature-selections.sh
	bash scripts/ci/test-platform-feature-closures.sh
	bash scripts/ci/test-platform-feature-graphs.sh
	bash scripts/ci/check-platform-features.sh
.PHONY: check-features-exhaustive
check-features-exhaustive:
	bash scripts/ci/test-platform-feature-selections.sh
	bash scripts/ci/test-platform-feature-closures.sh
	bash scripts/ci/test-platform-feature-graphs.sh
	bash scripts/ci/check-platform-features.sh --exhaustive
.PHONY: check-upstream-scaffolding
check-upstream-scaffolding:
	bash scripts/ci/check-upstream-scaffolding.sh
check-doc-links:
	perl scripts/ci/check-documentation-links.pl --root . README.md AGENTS.md CHANGELOG.md docs/status/current.md docs/migration.md crates/ic-platform/README.md crates/ic-platform-macros/README.md
check-pins:
	bash scripts/ci/check-dependency-pins.sh --consumer "$(CURDIR)" --cargo-inheritance
check-release-commands:
	bash scripts/ci/check-release-commands.sh . make/consumer.mk make/tools.mk make/release.mk make/rust-format.mk make/execution.mk scripts/ci/check-make-execution.sh scripts/ci/check-release-delivery.sh ci/release-delivery.txt
	bash scripts/ci/test-release-changelog.sh
ci:
	+bash scripts/ci/run-validation-targets.sh --fail-fast shared-tooling-check check-doc-links check-pins check-boundary fmt-check clippy check-runtime-lock check-release-commands check test docs-check check-consumer check-features wasm-check msrv

version release-version:
	@bash scripts/release/adapter.sh version
release-preflight:
	@bash scripts/release/adapter.sh preflight
release-verify:
	+VALIDATION_FAILURE_LOG_DIR="$$(git rev-parse --git-path release-state)/validation-failures" bash scripts/ci/run-validation-targets.sh ci
release-prepare-version:
	@bash scripts/release/adapter.sh prepare
release-prepared-check release-committed-check release-tagged-check release-push-check:
	@bash scripts/release/adapter.sh check
release-commit-check:
	@bash scripts/release/adapter.sh commit-check
release-files:
	@printf 'Cargo.toml\0Cargo.lock\0CHANGELOG.md\0scripts/fixtures/cdk-runtime/Cargo.lock\0'

.PHONY: check-runtime
check-runtime:
	bash scripts/ci/check-platform-runtime.sh

.PHONY: check-runtime-lock
check-runtime-lock:
	bash scripts/ci/test-published-runtime.sh
	bash scripts/ci/check-owned-runtime-source.sh
	bash scripts/ci/test-owned-runtime-source.sh
	bash scripts/release/runtime-lock.sh check
	bash scripts/ci/test-runtime-lock.sh
	bash scripts/ci/test-platform-runtime-graph.sh
