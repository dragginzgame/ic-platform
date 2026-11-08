.DEFAULT_GOAL := help
SHARED_TOOLING_RUST := 1
FORMAT_EXTRA_SOURCES := $(wildcard scripts/fixtures/cdk-runtime/*.rs scripts/fixtures/cdk-runtime/transport/*.rs scripts/fixtures/prepared-call-consumer/*.rs scripts/fixtures/type-benchmark/main.rs scripts/fixtures/parser-fuzz/main.rs scripts/fixtures/wasm-size/lib.rs)
FORMAT_RUST_ARGS = --rustfmt-edition 2024 $(foreach source,$(FORMAT_EXTRA_SOURCES),--rustfmt-source "$(source)")
FORMAT_EXTRA_SOURCES += $(wildcard scripts/fixtures/stable-identities/*.rs)
FORMAT_EXTRA_SOURCES += $(wildcard scripts/fixtures/ic0-consumer/*.rs)
FORMAT_EXTRA_SOURCES += $(wildcard scripts/fixtures/canic-wasm-benchmark/*.rs)
FORMAT_EXTRA_SOURCES += $(wildcard scripts/fixtures/raw-rand/canister/*.rs)
FORMAT_EXTRA_SOURCES += $(wildcard crates/ic-platform-macros/src/export/tests/linking/*.rs)
FORMAT_EXTRA_SOURCES += crates/ic-platform/tests/fixtures/candid_malformed.rs
FORMAT_EXTRA_SOURCES += crates/ic-platform/tests/fixtures/cbor_malformed.rs
# Fresh grouped releases capture their selected roster once. The shared runner
# invokes the ordinary group runner inside that checkout with this flag set.
RELEASE_GROUPS ?= $(RELEASE_GROUP)
RELEASE_SETUP_TARGET := release-candidate-setup
_platform_batch_targets := release-batch-status release-batch-resume release-batch-reconcile release-batch-retire
_platform_fresh_release := $(filter release-patch release-minor release-major,$(MAKECMDGOALS))
# Independent groups require a visible selection before any recipe or setup.
_platform_comma := ,
ifneq ($(_platform_fresh_release),)
ifneq ($(RELEASE_BATCH_ACTIVE),1)
_platform_batch_groups := $(strip $(subst $(_platform_comma), ,$(RELEASE_GROUPS)))
_platform_batch_order := $(strip $(foreach group,types runtime platform,$(if $(filter $(group),$(_platform_batch_groups)),$(group))))
ifeq ($(_platform_batch_groups),)
$(error Select RELEASE_GROUP=platform, RELEASE_GROUP=runtime or RELEASE_GROUP=types, or RELEASE_GROUPS in supplier order)
endif
ifneq ($(_platform_batch_order),$(_platform_batch_groups))
$(error Select RELEASE_GROUP=platform, RELEASE_GROUP=runtime or RELEASE_GROUP=types; batch groups must be unique and in supplier order: types,runtime,platform)
endif
ifneq ($(strip $(RELEASE_GROUP)),)
ifneq ($(strip $(RELEASE_GROUPS)),$(strip $(RELEASE_GROUP)))
$(error Select RELEASE_GROUP alone or the complete RELEASE_GROUPS roster)
endif
endif
ifneq ($(filter publish publish-check,$(MAKECMDGOALS)),)
$(error Select RELEASE_PUBLISH_TARGET instead of a second normal-checkout publication target)
endif
endif
endif
ifneq ($(filter-out release-editor-check $(_platform_batch_targets) $(_platform_fresh_release),$(filter release-% version publish publish-check,$(MAKECMDGOALS))),)
ifeq ($(RELEASE_GROUP),platform)
else ifeq ($(RELEASE_GROUP),types)
else ifeq ($(RELEASE_GROUP),runtime)
else
$(error Select RELEASE_GROUP=platform, RELEASE_GROUP=runtime or RELEASE_GROUP=types)
endif
endif
# Captured release commands still require their one saved group.
ifeq ($(RELEASE_BATCH_ACTIVE),1)
ifneq ($(_platform_fresh_release),)
ifneq ($(words $(RELEASE_GROUP)),1)
$(error Select RELEASE_GROUP=platform, RELEASE_GROUP=runtime or RELEASE_GROUP=types)
endif
ifneq ($(filter types runtime platform,$(RELEASE_GROUP)),$(RELEASE_GROUP))
$(error Select RELEASE_GROUP=platform, RELEASE_GROUP=runtime or RELEASE_GROUP=types)
endif
endif
endif
export RELEASE_GROUP
export RELEASE_DELIVERY := direct
export RELEASE_DELIVERY_POLICY := $(CURDIR)/ci/release-delivery.txt
LOCAL_TOOL_INSTALL_TARGETS += install-release-editor
LOCAL_TOOL_CHECK_TARGETS += release-editor-check
LOCAL_TOOL_INSTALL_TARGETS += install-browser-tools
LOCAL_TOOL_CHECK_TARGETS += browser-tools-check
include make/consumer.mk
# Keep the shared pre-capture cleanup behind tooling admission, including -j.
clean-empty-source-dirs: shared-tooling-check
release-batch-resume: shared-tooling-check
export YQ := $(CURDIR)/.tools/host/bin/yq
release-patch release-minor release-major release-resume: export IC_PLATFORM_RELEASE_PREPARE := 1

.PHONY: release-candidate-setup
release-candidate-setup:
	+bash scripts/release/candidate-setup.sh

.PHONY: help check test clippy docs-check wasm-check msrv check-boundary check-doc-links check-pins ci check-consumer check-features
.PHONY: version release-version release-preflight release-verify release-prepare-version release-files
.PHONY: release-prepared-check release-commit-check release-committed-check release-tagged-check release-push-check
.PHONY: check-release-commands
.PHONY: publish publish-check
.PHONY: check-retained-evidence
.PHONY: check-formatting

check-formatting:
	@bash scripts/ci/test-platform-formatting.sh

help:
	@echo 'Setup: install-tools, tools-check, install-hooks'
	@echo 'Consumer cache setup: prepare-consumer-dependencies (explicit registry fetch)'
	@echo 'Tool diagnostics: tools-report (offline, no installation)'
	@echo 'Focused: check, test, clippy, docs-check, wasm-check, msrv, check-boundary'
	@echo 'Full release gate: ci; authorized releases: release-patch, release-minor, release-major'
	@echo 'Exact saved recovery: release-resume VERSION=X.Y.Z'
	@echo 'Frozen cohort: release-patch RELEASE_GROUPS=types,runtime,platform RELEASE_PUBLISH_TARGET=publish'
	@echo 'Frozen recovery/custody: release-batch-status, release-batch-resume, release-batch-reconcile, release-batch-retire BATCH=batch-XXXXXXXX'
	@echo 'Feature graphs: check-features; check-features-exhaustive FEATURES=a,b (1–12 features)'
	@echo 'Release/publish requires RELEASE_GROUP=platform, RELEASE_GROUP=runtime or RELEASE_GROUP=types'
	@echo 'Registry: publish-check (dry run), publish (selected group); PACKAGE= selects an explicit batch'
	@echo 'Release editor: install-release-editor; release-editor-check (offline)'
	@echo 'Browser SDK: install-browser-tools, check-browser; check-browser-runtime needs prepared Chromium'

.PHONY: install-release-editor release-editor-check
install-release-editor:
	+bash scripts/release/editor.sh install
release-editor-check:
	+bash scripts/release/editor.sh check

.PHONY: install-browser-tools browser-tools-check check-browser check-browser-runtime
install-browser-tools:
	node scripts/ci/browser-toolchain.mjs
	npm ci --prefix packages/browser --ignore-scripts --no-audit --no-fund
	npm ci --prefix scripts/fixtures/browser-compiler-floor --ignore-scripts --no-audit --no-fund
browser-tools-check:
	node scripts/ci/check-browser-tools.mjs
check-browser: browser-tools-check
	npm run check --prefix packages/browser
check-browser-runtime: browser-tools-check
	npm test --prefix packages/browser
	node scripts/ci/check-browser-consumer.mjs --browser

publish: shared-tooling-check
	+$(MAKE) --no-print-directory host-tools-check check-retained-evidence
	bash scripts/release/publish.sh publish
publish-check: shared-tooling-check
	+$(MAKE) --no-print-directory host-tools-check check-retained-evidence
	bash scripts/release/publish.sh check

.PHONY: check-compiler
check-compiler: host-tools-check
	bash scripts/ci/check-platform-compiler.sh
check: check-compiler
	cargo check -p ic-platform -p ic-platform-types -p ic-platform-runtime -p ic-platform-macros -p ic-platform-runtime-guard --all-targets --all-features --locked
test: check-compiler
	bash scripts/ci/test-canic-benchmark-capture.sh
	bash scripts/ci/test-platform-documentation.sh
	bash scripts/ci/test-advisories.sh
	bash scripts/ci/test-platform-inventory.sh
	cargo test -p ic-platform -p ic-platform-types -p ic-platform-runtime -p ic-platform-macros -p ic-platform-runtime-guard --all-features --locked
PLATFORM_CLIPPY_PACKAGES ?= ic-platform ic-platform-types ic-platform-runtime ic-platform-macros ic-platform-runtime-guard
clippy: check-compiler
	cargo clippy $(foreach package,$(PLATFORM_CLIPPY_PACKAGES),--package $(package)) --all-targets --all-features --locked -- -D warnings
docs-check: check-compiler
	RUSTDOCFLAGS='-D warnings' cargo doc -p ic-platform -p ic-platform-types -p ic-platform-runtime -p ic-platform-macros -p ic-platform-runtime-guard --all-features --no-deps --locked
wasm-check: check-compiler
	cargo check -p ic-platform --lib --all-features --target wasm32-unknown-unknown --locked
# Compatibility command: these are the same supported-compiler proofs, once.
msrv: test wasm-check
check-boundary: check-candid-boundary
	bash scripts/ci/check-cargo-dependency-boundaries.sh --consumer "$(CURDIR)" --external-ic
.PHONY: check-candid-boundary
check-candid-boundary:
	perl scripts/ci/check-candid-boundary.pl --cbor "$(CURDIR)" ci/candid-boundary.json crates scripts native/ic-profiler/profiler native/ic-profiler/overlay
	bash scripts/ci/test-platform-candid-boundary.sh
.PHONY: check-primitives
check-primitives:
	@for target in "$$(rustc -vV | sed -n 's/^host: //p')" wasm32-unknown-unknown; do \
		for features in - executor runtime; do \
			set --; if [ "$$features" != - ]; then set -- --features "$$features"; fi; \
			perl scripts/ci/check-cargo-graph.pl --consumer "$(CURDIR)" --package ic-platform --target "$$target" --no-default-features --forbid blake3 "$$@" || exit $$?; \
		done; \
	done
.PHONY: check-stable-identities
check-stable-identities:
	bash scripts/ci/test-stable-identities.sh

.PHONY: prepare-consumer-dependencies
prepare-consumer-dependencies:
	bash scripts/ci/test-stable-identities.sh --prepare
	bash scripts/ci/test-runtime-singleton.sh --prepare
	bash scripts/ci/check-prepared-call-consumer.sh --prepare
check-consumer:
	bash scripts/ci/test-runtime-singleton.sh
	bash scripts/ci/check-runtime-ic0.sh
	bash scripts/ci/test-platform-msrv.sh
	bash scripts/ci/test-platform-consumer-admission.sh
	bash scripts/ci/test-prepared-call-consumer-admission.sh
	bash scripts/ci/check-prepared-call-consumer.sh
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
	bash scripts/ci/check-platform-features.sh --exhaustive "$(FEATURES)"
check-doc-links:
	@set -eu; mkdir -p target; \
		records="$$(mktemp "$(CURDIR)/target/doc-link-records.XXXXXX")"; \
		trap 'rm -f "$$records"' EXIT; \
		bash scripts/ci/verify-shared-tooling-snapshot.sh --read-manifest .shared-tooling.snapshot > "$$records"; \
		set -- README.md docs/development-and-qualification.md AGENTS.md CHANGELOG.md releases/pocket-ic/CHANGELOG.md releases/runtime/CHANGELOG.md releases/platform/CHANGELOG.md releases/types/CHANGELOG.md releases/history.md docs/status/current.md docs/migration.md docs/history/migrations-through-0.26.md scripts/benchmarks/0.9/README.md scripts/benchmarks/0.4/README.md scripts/benchmarks/0.22/README.md scripts/benchmarks/canic-wasm/README.md docs/ic-inventory.md docs/external-contracts.md docs/pocket-ic-client.md docs/platform-release-validation.md packages/browser/README.md packages/browser/CHANGELOG.md crates/ic-platform/README.md crates/ic-platform-runtime/README.md crates/ic-platform-runtime/src/call/README.md crates/ic-platform-macros/README.md; \
		while IFS="$$(printf '\t')" read -r kind checksum mode path; do \
			case "$$kind:$$path" in file:*.md) set -- "$$@" "$$path" ;; esac; \
		done < "$$records"; \
		perl scripts/ci/check-documentation-links.pl --root . "$$@"
.PHONY: check-inventory
.PHONY: check-native-provider
check-native-provider:
	python3 -m unittest discover -s native/ic-profiler/tests -p 'test_*.py'
	bash scripts/ci/check-native-profiler-http.sh

check-inventory:
	bash scripts/ci/check-platform-inventory.sh
check-pins:
	bash scripts/ci/check-dependency-pins.sh --consumer "$(CURDIR)" --cargo-inheritance --npm-root scripts/fixtures/browser-compiler-floor --node-version "$$(node -p 'require("./scripts/fixtures/browser-compiler-floor/package.json").engines.node')" --npm-version "$$(node -p 'require("./scripts/fixtures/browser-compiler-floor/package.json").engines.npm')"
	bash scripts/ci/check-dependency-pins.sh --consumer "$(CURDIR)" --npm-root packages/browser --node-version "$$(node -p 'require("./scripts/fixtures/browser-compiler-floor/package.json").engines.node')" --npm-version "$$(node -p 'require("./scripts/fixtures/browser-compiler-floor/package.json").engines.npm')"
check-release-commands:
	bash scripts/ci/test-resume-policy.sh
	perl scripts/ci/test-platform-evidence.pl
	bash scripts/ci/test-platform-frozen-setup.sh
	bash scripts/ci/test-platform-group-projection.sh
	bash scripts/ci/test-platform-ci-selection.sh
	bash scripts/release/check-policy-fixtures.sh
	bash scripts/ci/test-platform-release-groups.sh
	bash scripts/ci/test-platform-group-preparation.sh
	bash scripts/release/test-editor-bootstrap.sh
.PHONY: check-policy-fixture-evidence
check-policy-fixture-evidence:
	@perl scripts/release/policy-fixture-evidence.pl seal "$$PLATFORM_POLICY_ADMISSION" "$$PLATFORM_POLICY_SOURCE" "$$PLATFORM_POLICY_GROUP"
ci:
	+bash scripts/ci/run-validation-targets.sh --fail-fast shared-tooling-check check-doc-links check-pins check-boundary check-inventory fmt-check check-formatting check-primitives clippy check-browser check-runtime-lock check-release-commands check test docs-check check-consumer check-features wasm-check

release-resume: check-release-resume-policy
.PHONY: check-release-resume-policy
check-release-resume-policy:
	@bash scripts/release/check-resume-policy.sh "$(VERSION)"

version release-version:
	@bash scripts/release/adapter.sh version
check-retained-evidence:
	perl scripts/ci/check-retained-state.pl --root recovery-evidence --max-bytes 335544320 --max-file-bytes 268435456 --max-age-days 30
release-preflight: check-retained-evidence
	@bash scripts/release/adapter.sh preflight
release-verify:
	+bash scripts/release/validation-gate.sh verify
release-prepare-version:
	@bash scripts/release/adapter.sh prepare
release-prepared-check:
	@bash scripts/release/adapter.sh check
	@bash scripts/ci/check-platform-runtime.sh --timers --prepared-call --check-graph
release-committed-check release-tagged-check release-push-check:
	@bash scripts/release/adapter.sh check
release-commit-check:
	@bash scripts/release/adapter.sh commit-check
release-files:
	@bash scripts/release/adapter.sh files

.PHONY: check-runtime
check-runtime:
	bash scripts/ci/check-platform-runtime.sh --prepared-call
	bash scripts/ci/check-platform-raw-rand.sh

.PHONY: check-raw-rand
check-raw-rand:
	bash scripts/ci/check-platform-raw-rand.sh

.PHONY: check-runtime-composition check-runtime-all check-runtime-independence
check-runtime-composition:
	bash scripts/ci/check-platform-runtime.sh --timers --prepared-call
check-runtime-all:
	+bash scripts/ci/run-validation-targets.sh check-runtime check-runtime-composition
check-runtime-independence:
	bash scripts/ci/test-runtime-independence.sh

.PHONY: check-runtime-lock
check-runtime-lock:
	bash scripts/ci/test-runtime-selection.sh
	bash scripts/ci/check-icp-account-source.sh
	bash scripts/ci/test-published-runtime.sh
	bash scripts/ci/check-owned-runtime-source.sh
	bash scripts/ci/test-owned-runtime-source.sh
	bash scripts/ci/test-principal-source.sh
	bash scripts/ci/check-icrc-source.sh
	bash scripts/ci/test-icrc-source.sh
	bash scripts/release/runtime-lock.sh check
	bash scripts/release/runtime-pin.sh check
	bash scripts/ci/test-runtime-lock.sh
	bash scripts/ci/test-platform-runtime-graph.sh
