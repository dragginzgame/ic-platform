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
	  'qualify-registry-settings  Test and check the bounded Registry projection' \
	  'qualify-registry-runtime   Execute Registry Wasm and native heap fixtures' \
	  'qualify-marketplace    Test marketplace rows, natural numbers and public consumer' \
	  'qualify-marketplace-wasm  Execute marketplace and availability admission in Wasm' \
	  'qualify-types          Test and check local Principal and Account identities' \
	  'qualify-types-consumer  Compare published identities and execute native/Wasm fixtures' \
	  'qualify-runtime-call   Check inert call preparation and ownership' \
	  'qualify-runtime-consumer  Execute the local Types/Runtime boundary on native and Wasm' \
	  'qualify-runtime-ic0    Execute the IC0 cost and one-way submission ABI harness' \
	  'qualify-runtime-awaited  Execute first-poll and callback custody in the Wasm host' \
	  'qualify-runtime-guard  Check exact guard versions and mixed-executor refusal' \
	  'qualify-runtime-exports  Build and execute explicit generated IC update exports' \
	  'install-runtime-server / runtime-server-check  Prepare or verify Testkit PocketIC' \
	  'qualify-runtime-installed  Execute installed initial-update rollback cases' \
	  'install-ci-tools       Prepare pinned Linux workflow/security tools' \
	  'lint-workflows / check-secrets  Validate workflows or scan Git history' \
	  'fmt / fmt-check        Format or check Rust and Cargo manifests'

test-engine-request: toko-tooling-check
	cargo test --offline --locked -p ic-platform-protocol --features cloud-engine --lib cloud_engine::tests

.PHONY: qualify-runtime-call
qualify-runtime-call: toko-tooling-check
	cargo check --offline --locked -p ic-platform-runtime --no-default-features
	cargo test --offline --locked -p ic-platform-runtime --features call
	cargo clippy --offline --locked -p ic-platform-runtime --features call --all-targets -- -D warnings
	RUSTDOCFLAGS='-D warnings' cargo doc --offline --locked -p ic-platform-runtime --features call --no-deps
	cargo check --offline --locked -p ic-platform-runtime --features call --tests --target wasm32-unknown-unknown
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)" --package ic-platform-runtime --no-default-features --forbid-pattern '^(?!ic-platform-runtime$$)'
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)" --package ic-platform-runtime --no-default-features --features call --forbid-pattern '^(?!(ic-platform-runtime|ic-platform-types|cfg-if|crc32fast|data-encoding|serde|serde_core|serde_derive|serde_bytes|thiserror|thiserror-impl|proc-macro2|quote|syn|unicode-ident)$$)' --require ic-platform-types=1

.PHONY: qualify-types
qualify-types: toko-tooling-check
	cargo test --offline --locked --manifest-path crates/ic-platform-types/Cargo.toml
	cargo clippy --offline --locked --manifest-path crates/ic-platform-types/Cargo.toml --all-targets -- -D warnings
	RUSTDOCFLAGS='-D warnings' cargo doc --offline --locked --manifest-path crates/ic-platform-types/Cargo.toml --no-deps
	cargo check --offline --locked --manifest-path crates/ic-platform-types/Cargo.toml --tests --target wasm32-unknown-unknown
	cargo test --offline --locked --manifest-path crates/ic-platform-types/Cargo.toml --features principal-self-authenticating
	cargo clippy --offline --locked --manifest-path crates/ic-platform-types/Cargo.toml --features principal-self-authenticating --all-targets -- -D warnings
	RUSTDOCFLAGS='-D warnings' cargo doc --offline --locked --manifest-path crates/ic-platform-types/Cargo.toml --features principal-self-authenticating --no-deps
	cargo check --offline --locked --manifest-path crates/ic-platform-types/Cargo.toml --features principal-self-authenticating --tests --target wasm32-unknown-unknown

.PHONY: qualify-types-consumer check-types-graphs
qualify-types-consumer: qualification-tooling-check
	cargo test --offline --locked --manifest-path qualification/Cargo.toml -p types-consumer
	cargo test --offline --locked --manifest-path qualification/Cargo.toml -p types-consumer --features self-authentication
	cargo clippy --offline --locked --manifest-path qualification/Cargo.toml -p types-consumer --all-targets -- -D warnings
	cargo clippy --offline --locked --manifest-path qualification/Cargo.toml -p types-consumer --features self-authentication --all-targets -- -D warnings
	$(ENGINE_WASM_ENV) cargo build --offline --locked --manifest-path qualification/Cargo.toml -p types-consumer --release --target wasm32-unknown-unknown --target-dir target/types-wasm/default
	$(ENGINE_WASM_ENV) cargo build --offline --locked --manifest-path qualification/Cargo.toml -p types-consumer --features self-authentication --release --target wasm32-unknown-unknown --target-dir target/types-wasm/self-authentication
	node qualification/types-wasm.mjs

TYPES_CONSUMER_PACKAGES := types-consumer|ic-platform-types|cfg-if|crc32fast|data-encoding|serde|serde_core|serde_derive|serde_bytes|thiserror|thiserror-impl|proc-macro2|quote|syn|unicode-ident|serde_json|itoa|memchr|zmij
.PHONY: qualify-runtime-consumer check-runtime-consumer-graph
qualify-runtime-consumer: qualification-tooling-check
	cargo test --offline --locked --manifest-path qualification/Cargo.toml -p types-consumer --features runtime-call
	cargo clippy --offline --locked --manifest-path qualification/Cargo.toml -p types-consumer --features runtime-call --all-targets -- -D warnings
	$(ENGINE_WASM_ENV) cargo build --offline --locked --manifest-path qualification/Cargo.toml -p types-consumer --features runtime-call --release --target wasm32-unknown-unknown --target-dir target/types-wasm/runtime-call
	node qualification/types-wasm.mjs runtime-call

check-runtime-consumer-graph: qualification-tooling-check
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)/qualification" --package types-consumer --no-default-features --features runtime-call --forbid-pattern '^(?!(ic-platform-runtime|$(TYPES_CONSUMER_PACKAGES))$$)' --require ic-platform-types=1 --require ic-platform-runtime=1

.PHONY: lint-runtime-ic0 qualify-runtime-ic0
lint-runtime-ic0: qualification-tooling-check
	cargo clippy --offline --locked -p ic-platform-runtime --features call-ic0 --all-targets --target wasm32-unknown-unknown -- -D warnings
	cargo clippy --offline --locked --manifest-path qualification/Cargo.toml -p types-consumer --features runtime-ic0 --all-targets --target wasm32-unknown-unknown -- -D warnings

qualify-runtime-ic0: lint-runtime-ic0
	RUSTDOCFLAGS='-D warnings' cargo doc --offline --locked -p ic-platform-runtime --features call-ic0 --target wasm32-unknown-unknown --no-deps
	$(ENGINE_WASM_ENV) cargo build --offline --locked --manifest-path qualification/Cargo.toml -p types-consumer --features runtime-ic0 --release --target wasm32-unknown-unknown --target-dir target/types-wasm/runtime-ic0
	node qualification/runtime-ic0.mjs
	node qualification/runtime-arguments.mjs
	node qualification/runtime-entrypoints.mjs
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)/qualification" --package types-consumer --target wasm32-unknown-unknown --no-default-features --features runtime-ic0 --forbid-pattern '^(?!(ic-platform-runtime|ic-platform-runtime-guard|$(TYPES_CONSUMER_PACKAGES))$$)' --require ic-platform-types=1 --require ic-platform-runtime=1 --require ic-platform-runtime-guard=1

.PHONY: qualify-runtime-awaited
qualify-runtime-awaited: lint-runtime-ic0
	$(ENGINE_WASM_ENV) CARGO_ENCODED_RUSTFLAGS='-Clink-arg=--export-table' cargo build --offline --locked --manifest-path qualification/Cargo.toml -p types-consumer --features runtime-ic0 --bin runtime-awaited --release --target wasm32-unknown-unknown --target-dir target/runtime-awaited
	node qualification/runtime-awaited.mjs

.PHONY: install-runtime-server runtime-server-check qualify-runtime-installed
install-runtime-server:
	cargo run --offline --locked --manifest-path qualification/Cargo.toml -p runtime-installed -- setup

runtime-server-check:
	cargo run --offline --locked --manifest-path qualification/Cargo.toml -p runtime-installed -- check

qualify-runtime-installed: qualification-tooling-check runtime-server-check
	cargo clippy --offline --locked --manifest-path qualification/Cargo.toml -p runtime-installed --all-targets -- -D warnings
	cargo clippy --offline --locked --manifest-path qualification/Cargo.toml -p runtime-rollback --target wasm32-unknown-unknown --all-targets -- -D warnings
	$(ENGINE_WASM_ENV) cargo build --offline --locked --manifest-path qualification/Cargo.toml -p runtime-rollback --target wasm32-unknown-unknown --release --target-dir target/runtime-rollback
	cargo run --offline --locked --manifest-path qualification/Cargo.toml -p runtime-installed
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)/qualification" --package runtime-rollback --target wasm32-unknown-unknown --forbid-pattern '^(?!(runtime-rollback|ic-platform-runtime|ic-platform-runtime-guard|ic-platform-macros|ic-platform-types|cfg-if|crc32fast|data-encoding|serde|serde_core|serde_derive|serde_bytes|thiserror|thiserror-impl|proc-macro2|quote|syn|unicode-ident)$$)' --require ic-platform-runtime=1 --require ic-platform-runtime-guard=1 --require ic-platform-macros=1 --require ic-platform-types=1

.PHONY: qualify-runtime-guard
qualify-runtime-guard: qualification-tooling-check
	cargo test --offline --locked -p ic-platform-runtime-guard
	cargo clippy --offline --locked -p ic-platform-runtime-guard --all-targets -- -D warnings
	RUSTDOCFLAGS='-D warnings' cargo doc --offline --locked -p ic-platform-runtime-guard --no-deps
	cargo check --offline --locked -p ic-platform-runtime-guard --target wasm32-unknown-unknown
	node qualification/runtime-guard.mjs

.PHONY: qualify-runtime-input-ui
qualify-runtime-input-ui: qualification-tooling-check
	node qualification/runtime-ui.mjs input

.PHONY: qualify-runtime-exports
qualify-runtime-exports: qualification-tooling-check
	cargo test --offline --locked -p ic-platform-macros
	cargo clippy --offline --locked -p ic-platform-macros --all-targets -- -D warnings
	RUSTDOCFLAGS='-D warnings' cargo doc --offline --locked -p ic-platform-macros --no-deps
	cargo clippy --offline --locked -p ic-platform-runtime --features entrypoint-macros --all-targets --target wasm32-unknown-unknown -- -D warnings
	RUSTDOCFLAGS='-D warnings' cargo doc --offline --locked -p ic-platform-runtime --features entrypoint-macros --target wasm32-unknown-unknown --no-deps
	cargo clippy --offline --locked --manifest-path qualification/Cargo.toml -p runtime-exports --target wasm32-unknown-unknown --all-targets -- -D warnings
	$(ENGINE_WASM_ENV) cargo build --offline --locked --manifest-path qualification/Cargo.toml -p runtime-exports --target wasm32-unknown-unknown --release --target-dir target/runtime-exports
	node qualification/runtime-exports.mjs
	node qualification/runtime-ui.mjs exports
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)/qualification" --package runtime-exports --target wasm32-unknown-unknown --forbid-pattern '^(?!(runtime-exports|ic-platform-runtime|ic-platform-runtime-guard|ic-platform-macros|ic-platform-types|cfg-if|crc32fast|data-encoding|serde|serde_core|serde_derive|serde_bytes|thiserror|thiserror-impl|proc-macro2|quote|syn|unicode-ident)$$)' --require ic-platform-runtime=1 --require ic-platform-runtime-guard=1 --require ic-platform-macros=1 --require ic-platform-types=1

check-types-graphs: qualification-tooling-check
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)/qualification" --package types-consumer --no-default-features --forbid-pattern '^(?!($(TYPES_CONSUMER_PACKAGES))$$)' --require ic-platform-types=1
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)/qualification" --package types-consumer --no-default-features --features self-authentication --forbid-pattern '^(?!($(TYPES_CONSUMER_PACKAGES)|sha2|digest|block-buffer|crypto-common|hybrid-array|typenum|cpufeatures|libc)$$)' --require ic-platform-types=1 --require sha2=1

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
.PHONY: qualify-registry-settings
qualify-registry-settings: qualification-tooling-check
	cargo test --offline --locked -p ic-platform-protocol --features registry-settings --lib registry_settings::tests
	cargo clippy --offline --locked -p ic-platform-protocol --features registry-settings --all-targets -- -D warnings
	RUSTDOCFLAGS='-D warnings' cargo doc --offline --locked -p ic-platform-protocol --features registry-settings --no-deps
	cargo check --offline --locked -p ic-platform-protocol --features registry-settings --lib --target wasm32-unknown-unknown
	cargo test --offline --locked --manifest-path qualification/Cargo.toml -p engine-view-consumer --features registry-settings registry_settings
	cargo clippy --offline --locked --manifest-path qualification/Cargo.toml -p engine-view-consumer --features registry-settings --all-targets -- -D warnings
	cargo check --offline --locked --manifest-path qualification/Cargo.toml -p engine-view-consumer --features registry-settings --tests --target wasm32-unknown-unknown

GRAPH_TARGET ?= wasm32-unknown-unknown
.PHONY: qualify-marketplace
qualify-marketplace: qualification-tooling-check
	cargo test --offline --locked -p ic-platform-protocol --features cloud-engine-marketplace --lib
	cargo clippy --offline --locked -p ic-platform-protocol --features cloud-engine-marketplace --all-targets -- -D warnings
	RUSTDOCFLAGS='-D warnings' cargo doc --offline --locked -p ic-platform-protocol --features cloud-engine-marketplace --no-deps
	cargo test --offline --locked --manifest-path qualification/Cargo.toml -p engine-view-consumer --features marketplace marketplace
	cargo clippy --offline --locked --manifest-path qualification/Cargo.toml -p engine-view-consumer --features marketplace --all-targets -- -D warnings
	cargo check --offline --locked --manifest-path qualification/Cargo.toml -p engine-view-consumer --features marketplace --tests --target wasm32-unknown-unknown
	cargo clippy --offline --locked --manifest-path qualification/Cargo.toml -p engine-view-profile --features marketplace --test page_arguments -- -D warnings
	cargo test --offline --locked --manifest-path qualification/Cargo.toml -p engine-view-profile --features marketplace --release --test page_arguments -- --test-threads=1

.PHONY: qualify-registry-runtime
.PHONY: qualify-marketplace-wasm
qualify-marketplace-wasm: qualification-tooling-check
	cargo clippy --offline --locked --manifest-path qualification/Cargo.toml -p engine-view-wasm --features marketplace --bin marketplace-admission -- -D warnings
	$(ENGINE_WASM_ENV) cargo build --offline --locked --manifest-path qualification/Cargo.toml -p engine-view-wasm --bin marketplace-admission --release --target wasm32-unknown-unknown --no-default-features --features marketplace --target-dir target/engine-view-wasm/marketplace
	node qualification/engine-view-wasm.mjs marketplace

qualify-registry-runtime: qualification-tooling-check
	cargo clippy --offline --locked --manifest-path qualification/Cargo.toml -p engine-view-profile --features registry-settings --test registry -- -D warnings
	cargo clippy --offline --locked --manifest-path qualification/Cargo.toml -p engine-view-wasm --features registry-settings --bin registry-admission -- -D warnings
	cargo test --offline --locked --manifest-path qualification/Cargo.toml -p engine-view-profile --features registry-settings --release --test registry -- --nocapture --test-threads=1
	$(ENGINE_WASM_ENV) cargo build --offline --locked --manifest-path qualification/Cargo.toml -p engine-view-wasm --bin registry-admission --release --target wasm32-unknown-unknown --no-default-features --features registry-settings --target-dir target/engine-view-wasm/registry
	node qualification/engine-view-wasm.mjs registry

REGISTRY_GRAPH_PACKAGES := ic-platform-protocol|serde|serde_core|serde_derive|proc-macro2|quote|syn|unicode-ident
PROTOCOL_GRAPH_PACKAGES := ic-platform-protocol|ic-platform-types|cfg-if|crc32fast|data-encoding|serde|serde_core|serde_derive|serde_bytes|thiserror|thiserror-impl|proc-macro2|quote|syn|unicode-ident
MARKETPLACE_GRAPH_PACKAGES := $(PROTOCOL_GRAPH_PACKAGES)|num-bigint|num-integer|num-traits|autocfg
PROTOCOL_GRAPH_CHECK = perl "$(TOKO_TOOLING_ROOT)/scripts/ci/check-cargo-graph.pl" --target "$(GRAPH_TARGET)" --edges normal,build --explain
.PHONY: check-protocol-graphs
check-protocol-graphs: qualification-tooling-check
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)/qualification" --package engine-view-wasm --no-default-features --features marketplace --forbid-pattern '^(?!(engine-view-wasm|$(MARKETPLACE_GRAPH_PACKAGES))$$)' --require ic-platform-types=1 --require ic-platform-protocol=1 --require num-bigint=1
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)" --package ic-platform-protocol --no-default-features --features cloud-engine-marketplace --forbid-pattern '^(?!($(MARKETPLACE_GRAPH_PACKAGES))$$)' --require ic-platform-types=1 --require num-bigint=1
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)/qualification" --package engine-view-consumer --features marketplace --forbid-pattern '^(?!(engine-view-consumer|$(MARKETPLACE_GRAPH_PACKAGES))$$)' --require ic-platform-types=1 --require ic-platform-protocol=1 --require num-bigint=1
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)/qualification" --package engine-view-wasm --no-default-features --features registry-settings --forbid-pattern '^(?!(engine-view-wasm|$(PROTOCOL_GRAPH_PACKAGES))$$)' --require ic-platform-types=1 --require ic-platform-protocol=1
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)" --package ic-platform-protocol --no-default-features --features registry-settings --forbid-pattern '^(?!($(REGISTRY_GRAPH_PACKAGES))$$)'
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)" --package ic-platform-protocol --forbid-pattern '^(?!ic-platform-protocol$$)'
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)" --package ic-platform-protocol --no-default-features --features cloud-engine --forbid-pattern '^(?!ic-platform-protocol$$)'
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)" --package ic-platform-protocol --no-default-features --features cloud-engine-response --forbid-pattern '^(?!($(PROTOCOL_GRAPH_PACKAGES))$$)' --require ic-platform-types=1
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)/qualification" --package engine-view-consumer --forbid-pattern '^(?!(engine-view-consumer|$(PROTOCOL_GRAPH_PACKAGES))$$)' --require ic-platform-types=1 --require ic-platform-protocol=1
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)/qualification" --package engine-view-consumer --features registry-settings --forbid-pattern '^(?!(engine-view-consumer|$(PROTOCOL_GRAPH_PACKAGES))$$)' --require ic-platform-types=1 --require ic-platform-protocol=1
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)/qualification" --package engine-view-wasm --no-default-features --features candidate --forbid-pattern '^(?!(engine-view-wasm|$(PROTOCOL_GRAPH_PACKAGES))$$)' --require ic-platform-types=1 --require ic-platform-protocol=1
