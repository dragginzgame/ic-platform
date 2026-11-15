.DEFAULT_GOAL := help

override TOKO_TOOLING_ROOT := $(CURDIR)/vendor/toko-tooling
FORMAT_RUST_ARGS := --extra-workspace qualification
include vendor/toko-tooling/make/consumer.mk
include vendor/toko-tooling/make/cargo-dependencies.mk

.PHONY: help test-engine-request lint-engine-request doc-engine-request check-engine-request-wasm
help:
	@printf '%s\n' \
	  'deps-list              List local packages and versions (offline)' \
	  'deps-update            Update Cargo locks; optional DEPENDENCY_PACKAGE / DEPENDENCY_PRECISE' \
	  'install-tools          Prepare the selected Toko host and Rust tools' \
	  'tools-check            Check prepared tools without installing' \
	  'toko-tooling-check     Verify the installed snapshot against Cargo.lock' \
	  'test-engine-request    Test Cloud Engine request bytes and limits' \
	  'lint-engine-request    Clippy for the request encoder and tests' \
	  'doc-engine-request     Check public encoder documentation' \
	  'check-engine-request-wasm  Check the encoder on wasm32-unknown-unknown' \
	  'test-engine-response   Test response projection and malformed frames' \
	  'lint-engine-response / doc-engine-response  Check decoder code and docs' \
	  'qualify-engine-response  Native consumer/Serde proof and Wasm compilation' \
	  'measure-engine-response  Report fixed-workload logical resource charges' \
	  'profile-engine-response  Compare native heap allocations with the published decoder' \
	  'measure-engine-wasm    Build, execute and compare standalone decoder Wasm sizes' \
	  'qualify-engine-wasm    Execute old/new Wasm admission and refusal fixtures' \
	  'check-protocol-graphs  Enforce selected closures (GRAPH_TARGET defaults to Wasm)' \
	  'install-ci-tools       Prepare pinned Linux workflow/security tools' \
	  'lint-workflows / check-secrets  Validate workflows or scan Git history' \
	  'fmt / fmt-check        Format or check Rust and Cargo manifests'

test-engine-request: toko-tooling-check
	cargo test --offline --locked -p ic-platform-protocol --features cloud-engine --lib cloud_engine::tests

lint-engine-request: toko-tooling-check
	cargo clippy --offline --locked -p ic-platform-protocol --features cloud-engine --all-targets -- -D warnings

doc-engine-request: toko-tooling-check
	RUSTDOCFLAGS='-D warnings' cargo doc --offline --locked -p ic-platform-protocol --features cloud-engine --no-deps

check-engine-request-wasm: toko-tooling-check
	cargo check --offline --locked -p ic-platform-protocol --features cloud-engine --lib --target wasm32-unknown-unknown

.PHONY: test-engine-response lint-engine-response doc-engine-response qualify-engine-response
test-engine-response: toko-tooling-check
	cargo test --offline --locked -p ic-platform-protocol --features cloud-engine-response --lib cloud_engine::response::tests

lint-engine-response: toko-tooling-check
	cargo clippy --offline --locked -p ic-platform-protocol --features cloud-engine-response --all-targets -- -D warnings

doc-engine-response: toko-tooling-check
	RUSTDOCFLAGS='-D warnings' cargo doc --offline --locked -p ic-platform-protocol --features cloud-engine-response --no-deps

.PHONY: qualification-tooling-check
qualification-tooling-check: toko-tooling-check
	@root="$$(perl "$(TOKO_TOOLING_ROOT)/scripts/dev/tooling-lock.pl" "$(CURDIR)")" && \
	  qualification="$$(perl "$(TOKO_TOOLING_ROOT)/scripts/dev/tooling-lock.pl" "$(CURDIR)/qualification")" && \
	  test "$$root" = "$$qualification" || { echo 'Product and qualification tooling selections must agree.' >&2; exit 1; }

qualify-engine-response: qualification-tooling-check
	cargo test --offline --locked --manifest-path qualification/Cargo.toml -p engine-view-consumer
	cargo clippy --offline --locked --manifest-path qualification/Cargo.toml -p engine-view-consumer --all-targets -- -D warnings
	cargo check --offline --locked --manifest-path qualification/Cargo.toml -p engine-view-consumer --tests --target wasm32-unknown-unknown

.PHONY: measure-engine-response
measure-engine-response: qualification-tooling-check
	cargo test --offline --locked --manifest-path qualification/Cargo.toml -p engine-view-consumer comparison::resources -- --nocapture

.PHONY: profile-engine-response
profile-engine-response: qualification-tooling-check
	cargo test --offline --locked --manifest-path qualification/Cargo.toml -p engine-view-profile --release --lib -- --nocapture --test-threads=1
	cargo clippy --offline --locked --manifest-path qualification/Cargo.toml -p engine-view-profile --all-targets -- -D warnings

# Fixed library comparison settings, independent of a future deployment recipe.
ENGINE_WASM_ENV = RUSTFLAGS= CARGO_ENCODED_RUSTFLAGS= RUSTC_WRAPPER= RUSTC_WORKSPACE_WRAPPER= CARGO_PROFILE_RELEASE_OPT_LEVEL=3 CARGO_PROFILE_RELEASE_LTO=false CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16 CARGO_PROFILE_RELEASE_DEBUG=false CARGO_PROFILE_RELEASE_STRIP=none CARGO_PROFILE_RELEASE_PANIC=unwind CARGO_PROFILE_RELEASE_INCREMENTAL=false CARGO_PROFILE_RELEASE_OVERFLOW_CHECKS=false CARGO_PROFILE_RELEASE_DEBUG_ASSERTIONS=false
.PHONY: measure-engine-wasm
measure-engine-wasm: qualification-tooling-check
	$(ENGINE_WASM_ENV) cargo build --offline --locked --manifest-path qualification/Cargo.toml -p engine-view-wasm --bin engine-view-wasm --release --target wasm32-unknown-unknown --no-default-features --features published --target-dir target/engine-view-wasm/published
	$(ENGINE_WASM_ENV) cargo build --offline --locked --manifest-path qualification/Cargo.toml -p engine-view-wasm --bin engine-view-wasm --release --target wasm32-unknown-unknown --no-default-features --features candidate --target-dir target/engine-view-wasm/candidate
	node qualification/engine-view-wasm.mjs

.PHONY: qualify-engine-wasm
qualify-engine-wasm: qualification-tooling-check
	$(ENGINE_WASM_ENV) cargo build --offline --locked --manifest-path qualification/Cargo.toml -p engine-view-wasm --bin engine-view-admission --release --target wasm32-unknown-unknown --no-default-features --features published --target-dir target/engine-view-wasm/published
	$(ENGINE_WASM_ENV) cargo build --offline --locked --manifest-path qualification/Cargo.toml -p engine-view-wasm --bin engine-view-admission --release --target wasm32-unknown-unknown --no-default-features --features candidate --target-dir target/engine-view-wasm/candidate
	node qualification/engine-view-wasm.mjs admission
	cargo clippy --offline --locked --manifest-path qualification/Cargo.toml -p engine-view-wasm --features published --all-targets -- -D warnings
	cargo clippy --offline --locked --manifest-path qualification/Cargo.toml -p engine-view-wasm --features candidate --all-targets -- -D warnings

.PHONY: install-ci-tools lint-workflows check-secrets
install-ci-tools:
	bash "$(TOKO_TOOLING_ROOT)/scripts/ci/install-ci-tool.sh" gitleaks --install-dir "$(CURDIR)/.tools/ci/bin"
	bash "$(TOKO_TOOLING_ROOT)/scripts/ci/install-ci-tool.sh" actionlint --install-dir "$(CURDIR)/.tools/ci/bin"
	bash "$(TOKO_TOOLING_ROOT)/scripts/ci/install-ci-tool.sh" shellcheck --install-dir "$(CURDIR)/.tools/ci/bin"

lint-workflows:
	PATH="$(CURDIR)/.tools/ci/bin:$(PATH)" "$(CURDIR)/.tools/ci/bin/actionlint" .github/workflows/ci.yml

check-secrets:
	"$(CURDIR)/.tools/ci/bin/gitleaks" git --redact=100 --no-banner --no-color --log-level error

# Initial projection's admitted normal/build closure. New suppliers require review.
GRAPH_TARGET ?= wasm32-unknown-unknown
PROTOCOL_GRAPH_PACKAGES := ic-platform-protocol|ic-platform-types|cfg-if|crc32fast|data-encoding|serde|serde_core|serde_derive|serde_bytes|thiserror|thiserror-impl|proc-macro2|quote|syn|unicode-ident
PROTOCOL_GRAPH_CHECK = perl "$(TOKO_TOOLING_ROOT)/scripts/ci/check-cargo-graph.pl" --target "$(GRAPH_TARGET)" --edges normal,build --explain
.PHONY: check-protocol-graphs
check-protocol-graphs: qualification-tooling-check
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)" --package ic-platform-protocol --forbid-pattern '^(?!ic-platform-protocol$$)'
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)" --package ic-platform-protocol --no-default-features --features cloud-engine --forbid-pattern '^(?!ic-platform-protocol$$)'
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)" --package ic-platform-protocol --no-default-features --features cloud-engine-response --forbid-pattern '^(?!($(PROTOCOL_GRAPH_PACKAGES))$$)' --require ic-platform-types=1
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)/qualification" --package engine-view-consumer --forbid-pattern '^(?!(engine-view-consumer|$(PROTOCOL_GRAPH_PACKAGES))$$)' --require ic-platform-types=1 --require ic-platform-protocol=1
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)/qualification" --package engine-view-wasm --no-default-features --features candidate --forbid-pattern '^(?!(engine-view-wasm|$(PROTOCOL_GRAPH_PACKAGES))$$)' --require ic-platform-types=1 --require ic-platform-protocol=1
