.DEFAULT_GOAL := help

override TOKO_TOOLING_ROOT := $(CURDIR)/vendor/toko-tooling
FORMAT_RUST_ARGS := --extra-workspace qualification
IC_TOOLS := icp
LOCAL_TOOL_INSTALL_TARGETS += install-ic-tools
LOCAL_TOOL_CHECK_TARGETS += ic-tools-check
# Normal qualification always uses the admitted checkout-local executable.
override ICP_CLI := $(CURDIR)/.tools/ic/bin/icp
export ICP_CLI
# Explicit independent product groups, published one package at a time.
CARGO_PUBLISH_PACKAGES := ic-platform-types ic-platform-runtime-guard ic-platform-macros ic-platform-runtime ic-platform-protocol ic-platform
CARGO_PUBLISH_REGISTRY := crates-io
CARGO_PUBLISH_ALL_FEATURES := 1
CARGO_PUBLISH_VERIFY_TARGETS := release-publication-admission
include vendor/toko-tooling/make/consumer.mk
include vendor/toko-tooling/make/cargo-dependencies.mk

.PHONY: types-publication-admission
types-publication-admission: qualification-tooling-check
	CARGO_PUBLISH_SELECTION=ic-platform-types RELEASE_GROUP=types node scripts/release-publication-admission.mjs

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
	  'profile-engine-response  Check native heap allocations against retained bounds' \
	  'measure-engine-wasm    Build, execute and compare standalone decoder Wasm sizes' \
	  'qualify-engine-wasm    Execute old/new Wasm admission and refusal fixtures' \
	  'check-protocol-graphs  Enforce selected closures (GRAPH_TARGET defaults to Wasm)' \
	  'qualify-registry-settings  Test and check the bounded Registry projection' \
	  'qualify-registry-runtime   Execute Registry Wasm and native heap fixtures' \
	  'qualify-marketplace    Test marketplace rows, natural numbers and public consumer' \
	  'qualify-marketplace-wasm  Execute marketplace and availability admission in Wasm' \
	  'qualify-platform-certificate  Check strict unverified certificate decoding' \
	  'qualify-platform-authentication  Check direct and delegated certificate authentication' \
	  'qualify-canister-signature-authentication  Check key, witness and certificate binding' \
	  'qualify-canister-signature-decode  Check bounded unverified signature envelopes' \
	  'qualify-canister-signature-observed  Capture and authenticate replica signatures' \
	  'qualify-canister-signature-wasm  Replay observed signatures on native and Wasm' \
	  'qualify-auth-consumer  Exercise isolated Auth issuer policy on native and Wasm' \
	  'qualify-types-auth-archive  Test packaged Types through pinned Auth passive source' \
	  'qualify-platform-wasm  Execute certificate authentication and refusals in Wasm' \
	  'qualify-platform-observed  Verify replica-issued certificates through public APIs' \
	  'qualify-types          Test and check local Principal and Account identities' \
	  'qualify-types-consumer  Compare published identities and execute native/Wasm fixtures' \
	  'qualify-runtime-call   Check inert call preparation and ownership' \
	  'qualify-runtime-consumer  Execute the local Types/Runtime boundary on native and Wasm' \
	  'qualify-runtime-ic0    Execute the IC0 cost and one-way submission ABI harness' \
	  'qualify-runtime-awaited  Execute first-poll and callback custody in the Wasm host' \
	  'qualify-runtime-guard  Check exact guard versions and mixed-executor refusal' \
	  'qualify-runtime-exports  Build and execute explicit generated IC update exports' \
	  'install-runtime-server / runtime-server-check  Prepare or verify Testkit PocketIC' \
	  'qualify-runtime-installed  Execute installed calls, rollback and drained upgrades' \
	  'qualify-runtime-timer  Execute timer roots, trap cleanup and version observations' \
	  'qualify-auth-runtime   Test pinned actual Auth endpoints in PocketIC' \
	  'qualify-runtime-raw    Check synchronous system ABI and installed behavior' \
	  'qualify-storage-runtime  Compile pinned Storage against synchronous Runtime' \
	  'qualify-http-request   Check bounded management GET argument encoding' \
	  'upgrade-help / check-upgrade-tooling  Preview operator usage or test failure handling' \
	  'install-ci-tools       Prepare pinned Linux workflow/security tools' \
	  'lint-workflows / check-secrets  Validate workflows or scan Git history' \
	  'fmt / fmt-check        Format or check Rust and Cargo manifests'

test-engine-request: toko-tooling-check
	cargo test --locked -p ic-platform-protocol --features cloud-engine --lib cloud_engine::tests

.PHONY: qualify-runtime-call
qualify-runtime-call: toko-tooling-check
	cargo check --locked -p ic-platform-runtime --no-default-features
	cargo test --locked -p ic-platform-runtime --features call
	cargo clippy --locked -p ic-platform-runtime --features call --all-targets -- -D warnings
	RUSTDOCFLAGS='-D warnings' cargo doc --locked -p ic-platform-runtime --features call --no-deps
	cargo check --locked -p ic-platform-runtime --features call --tests --target wasm32-unknown-unknown
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)" --package ic-platform-runtime --no-default-features --forbid-pattern '^(?!ic-platform-runtime$$)'
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)" --package ic-platform-runtime --no-default-features --features call --forbid-pattern '^(?!(ic-platform-runtime|ic-platform-types|cfg-if|crc32fast|data-encoding|serde|serde_core|serde_derive|serde_bytes|thiserror|thiserror-impl|proc-macro2|quote|syn|unicode-ident)$$)' --require ic-platform-types=1

.PHONY: qualify-types
qualify-types: toko-tooling-check
	cargo clippy --locked --manifest-path crates/ic-platform-types/Cargo.toml --all-targets -- -D warnings
	cargo clippy --locked --manifest-path crates/ic-platform-types/Cargo.toml --features principal-self-authenticating --all-targets -- -D warnings
	cargo test --locked --manifest-path crates/ic-platform-types/Cargo.toml
	RUSTDOCFLAGS='-D warnings' cargo doc --locked --manifest-path crates/ic-platform-types/Cargo.toml --no-deps
	cargo check --locked --manifest-path crates/ic-platform-types/Cargo.toml --tests --target wasm32-unknown-unknown
	cargo test --locked --manifest-path crates/ic-platform-types/Cargo.toml --features principal-self-authenticating
	RUSTDOCFLAGS='-D warnings' cargo doc --locked --manifest-path crates/ic-platform-types/Cargo.toml --features principal-self-authenticating --no-deps
	cargo check --locked --manifest-path crates/ic-platform-types/Cargo.toml --features principal-self-authenticating --tests --target wasm32-unknown-unknown

.PHONY: qualify-types-consumer check-types-graphs
qualify-types-consumer: qualification-tooling-check
	cargo clippy --locked --manifest-path qualification/Cargo.toml -p types-consumer --all-targets -- -D warnings
	cargo clippy --locked --manifest-path qualification/Cargo.toml -p types-consumer --features self-authentication --all-targets -- -D warnings
	cargo test --locked --manifest-path qualification/Cargo.toml -p types-consumer
	cargo test --locked --manifest-path qualification/Cargo.toml -p types-consumer --features self-authentication
	$(ENGINE_WASM_ENV) cargo build --locked --manifest-path qualification/Cargo.toml -p types-consumer --release --target wasm32-unknown-unknown --target-dir target/types-wasm/default
	$(ENGINE_WASM_ENV) cargo build --locked --manifest-path qualification/Cargo.toml -p types-consumer --features self-authentication --release --target wasm32-unknown-unknown --target-dir target/types-wasm/self-authentication
	node qualification/types-wasm.mjs

TYPES_CONSUMER_PACKAGES := types-consumer|ic-platform-types|cfg-if|crc32fast|data-encoding|serde|serde_core|serde_derive|serde_bytes|thiserror|thiserror-impl|proc-macro2|quote|syn|unicode-ident|serde_json|itoa|memchr|zmij

.PHONY: qualify-types-auth-archive
qualify-types-auth-archive: qualification-tooling-check
	node qualification/types-auth/qualify.mjs

.PHONY: qualify-runtime-consumer check-runtime-consumer-graph
qualify-runtime-consumer: qualification-tooling-check
	cargo test --locked --manifest-path qualification/Cargo.toml -p types-consumer --features runtime-call
	cargo clippy --locked --manifest-path qualification/Cargo.toml -p types-consumer --features runtime-call --all-targets -- -D warnings
	$(ENGINE_WASM_ENV) cargo build --locked --manifest-path qualification/Cargo.toml -p types-consumer --features runtime-call --release --target wasm32-unknown-unknown --target-dir target/types-wasm/runtime-call
	node qualification/types-wasm.mjs runtime-call

check-runtime-consumer-graph: qualification-tooling-check
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)/qualification" --package types-consumer --no-default-features --features runtime-call --forbid-pattern '^(?!(ic-platform-runtime|$(TYPES_CONSUMER_PACKAGES))$$)' --require ic-platform-types=1 --require ic-platform-runtime=1

.PHONY: lint-runtime-ic0 qualify-runtime-ic0
lint-runtime-ic0: qualification-tooling-check
	cargo clippy --locked -p ic-platform-runtime --features call-ic0 --all-targets --target wasm32-unknown-unknown -- -D warnings
	cargo clippy --locked --manifest-path qualification/Cargo.toml -p types-consumer --features runtime-ic0 --all-targets --target wasm32-unknown-unknown -- -D warnings

qualify-runtime-ic0: lint-runtime-ic0
	RUSTDOCFLAGS='-D warnings' cargo doc --locked -p ic-platform-runtime --features call-ic0 --target wasm32-unknown-unknown --no-deps
	$(ENGINE_WASM_ENV) cargo build --locked --manifest-path qualification/Cargo.toml -p types-consumer --features runtime-ic0 --release --target wasm32-unknown-unknown --target-dir target/types-wasm/runtime-ic0
	node qualification/runtime-ic0.mjs
	node qualification/runtime-arguments.mjs
	node qualification/runtime-entrypoints.mjs
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)/qualification" --package types-consumer --target wasm32-unknown-unknown --no-default-features --features runtime-ic0 --forbid-pattern '^(?!(ic-platform-runtime|ic-platform-runtime-guard|$(TYPES_CONSUMER_PACKAGES))$$)' --require ic-platform-types=1 --require ic-platform-runtime=1 --require ic-platform-runtime-guard=1

.PHONY: qualify-runtime-awaited
qualify-runtime-awaited: lint-runtime-ic0
	$(ENGINE_WASM_ENV) CARGO_ENCODED_RUSTFLAGS='-Clink-arg=--export-table' cargo build --locked --manifest-path qualification/Cargo.toml -p types-consumer --features runtime-ic0 --bin runtime-awaited --release --target wasm32-unknown-unknown --target-dir target/runtime-awaited
	node qualification/runtime-awaited.mjs

.PHONY: install-runtime-server runtime-server-check qualify-runtime-installed qualify-runtime-timer
install-runtime-server:
	node qualification/server.mjs setup

runtime-server-check:
	cargo clippy --locked --manifest-path qualification/Cargo.toml -p platform-qualification-server --all-targets -- -D warnings
	cargo build --locked --manifest-path qualification/Cargo.toml -p platform-qualification-server
	node qualification/server.test.mjs
	node qualification/server.mjs check

qualify-runtime-timer: qualification-tooling-check runtime-server-check
	cargo clippy --locked --manifest-path qualification/Cargo.toml -p runtime-installed --all-targets -- -D warnings
	cargo clippy --locked --manifest-path qualification/Cargo.toml -p runtime-rollback --target wasm32-unknown-unknown --all-targets -- -D warnings
	$(ENGINE_WASM_ENV) cargo build --locked --manifest-path qualification/Cargo.toml -p runtime-rollback --target wasm32-unknown-unknown --release --target-dir target/runtime-rollback
	node qualification/server.mjs run cargo run --locked --manifest-path qualification/Cargo.toml -p runtime-installed -- timer

qualify-runtime-installed: qualification-tooling-check runtime-server-check
	cargo test --locked -p ic-platform-runtime --features raw-rand raw_rand::
	$(ENGINE_WASM_ENV) CARGO_ENCODED_RUSTFLAGS=--cfg=ic_platform_qualification_accounting cargo build --locked --manifest-path qualification/Cargo.toml -p runtime-rollback --target wasm32-unknown-unknown --release --target-dir target/runtime-accounting
	cargo clippy --locked --manifest-path qualification/Cargo.toml -p runtime-installed --all-targets -- -D warnings
	cargo clippy --locked --manifest-path qualification/Cargo.toml -p runtime-rollback --target wasm32-unknown-unknown --all-targets -- -D warnings
	$(ENGINE_WASM_ENV) cargo build --locked --manifest-path qualification/Cargo.toml -p runtime-rollback --target wasm32-unknown-unknown --release --target-dir target/runtime-rollback
	node qualification/runtime-system.mjs
	node qualification/server.mjs run cargo run --locked --manifest-path qualification/Cargo.toml -p runtime-installed
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)/qualification" --package runtime-rollback --target wasm32-unknown-unknown --forbid-pattern '^(?!(runtime-rollback|ic-platform-protocol|ic-platform-runtime|ic-platform-runtime-guard|ic-platform-macros|ic-platform-types|cfg-if|crc32fast|data-encoding|serde|serde_core|serde_derive|serde_bytes|thiserror|thiserror-impl|proc-macro2|quote|syn|unicode-ident)$$)' --require ic-platform-protocol=1 --require ic-platform-runtime=1 --require ic-platform-runtime-guard=1 --require ic-platform-macros=1 --require ic-platform-types=1

.PHONY: qualify-runtime-guard
qualify-runtime-guard: qualification-tooling-check
	cargo test --locked -p ic-platform-runtime-guard
	cargo clippy --locked -p ic-platform-runtime-guard --all-targets -- -D warnings
	RUSTDOCFLAGS='-D warnings' cargo doc --locked -p ic-platform-runtime-guard --no-deps
	cargo check --locked -p ic-platform-runtime-guard --target wasm32-unknown-unknown
	node qualification/runtime-guard.mjs

.PHONY: qualify-runtime-input-ui
qualify-runtime-input-ui: qualification-tooling-check
	node qualification/runtime-ui.mjs input

.PHONY: qualify-runtime-exports
qualify-runtime-exports: qualification-tooling-check
	cargo test --locked -p ic-platform-macros
	cargo clippy --locked -p ic-platform-macros --all-targets -- -D warnings
	RUSTDOCFLAGS='-D warnings' cargo doc --locked -p ic-platform-macros --no-deps
	cargo clippy --locked -p ic-platform-runtime --features entrypoint-macros --all-targets --target wasm32-unknown-unknown -- -D warnings
	RUSTDOCFLAGS='-D warnings' cargo doc --locked -p ic-platform-runtime --features entrypoint-macros --target wasm32-unknown-unknown --no-deps
	cargo clippy --locked --manifest-path qualification/Cargo.toml -p runtime-exports --target wasm32-unknown-unknown --all-targets -- -D warnings
	$(ENGINE_WASM_ENV) cargo build --locked --manifest-path qualification/Cargo.toml -p runtime-exports --target wasm32-unknown-unknown --release --target-dir target/runtime-exports
	node qualification/runtime-exports.mjs
	node qualification/runtime-ui.mjs exports
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)/qualification" --package runtime-exports --target wasm32-unknown-unknown --forbid-pattern '^(?!(runtime-exports|ic-platform-runtime|ic-platform-runtime-guard|ic-platform-macros|ic-platform-types|cfg-if|crc32fast|data-encoding|serde|serde_core|serde_derive|serde_bytes|thiserror|thiserror-impl|proc-macro2|quote|syn|unicode-ident)$$)' --require ic-platform-runtime=1 --require ic-platform-runtime-guard=1 --require ic-platform-macros=1 --require ic-platform-types=1

check-types-graphs: qualification-tooling-check
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)/qualification" --package types-consumer --no-default-features --forbid-pattern '^(?!($(TYPES_CONSUMER_PACKAGES))$$)' --require ic-platform-types=1
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)/qualification" --package types-consumer --no-default-features --features self-authentication --forbid-pattern '^(?!($(TYPES_CONSUMER_PACKAGES)|sha2|digest|block-buffer|crypto-common|hybrid-array|typenum|cpufeatures|libc)$$)' --require ic-platform-types=1 --require sha2=1

lint-engine-request: toko-tooling-check
	cargo clippy --locked -p ic-platform-protocol --features cloud-engine --all-targets -- -D warnings

doc-engine-request: toko-tooling-check
	RUSTDOCFLAGS='-D warnings' cargo doc --locked -p ic-platform-protocol --features cloud-engine --no-deps

check-engine-request-wasm: toko-tooling-check
	cargo check --locked -p ic-platform-protocol --features cloud-engine --lib --target wasm32-unknown-unknown

.PHONY: test-engine-response lint-engine-response doc-engine-response qualify-engine-response
test-engine-response: toko-tooling-check
	cargo test --locked -p ic-platform-protocol --features cloud-engine-response --lib cloud_engine::response::tests

lint-engine-response: toko-tooling-check
	cargo clippy --locked -p ic-platform-protocol --features cloud-engine-response --all-targets -- -D warnings

doc-engine-response: toko-tooling-check
	RUSTDOCFLAGS='-D warnings' cargo doc --locked -p ic-platform-protocol --features cloud-engine-response --no-deps

.PHONY: qualification-tooling-check
qualification-tooling-check: toko-tooling-check
	@root="$$(perl "$(TOKO_TOOLING_ROOT)/scripts/dev/tooling-lock.pl" "$(CURDIR)")" && \
	  qualification="$$(perl "$(TOKO_TOOLING_ROOT)/scripts/dev/tooling-lock.pl" "$(CURDIR)/qualification")" && \
	  test "$$root" = "$$qualification" || { echo 'Product and qualification tooling selections must agree.' >&2; exit 1; }

qualify-engine-response: qualification-tooling-check
	cargo test --locked --manifest-path qualification/Cargo.toml -p engine-view-consumer
	cargo clippy --locked --manifest-path qualification/Cargo.toml -p engine-view-consumer --all-targets -- -D warnings
	cargo check --locked --manifest-path qualification/Cargo.toml -p engine-view-consumer --tests --target wasm32-unknown-unknown

.PHONY: measure-engine-response
measure-engine-response: qualification-tooling-check
	cargo test --locked --manifest-path qualification/Cargo.toml -p engine-view-consumer comparison::resources -- --nocapture

.PHONY: profile-engine-response
profile-engine-response: qualification-tooling-check
	cargo test --locked --manifest-path qualification/Cargo.toml -p engine-view-profile --release --lib -- --nocapture --test-threads=1
	cargo clippy --locked --manifest-path qualification/Cargo.toml -p engine-view-profile --all-targets -- -D warnings

# Fixed library comparison settings, independent of a future deployment recipe.
ENGINE_WASM_ENV = RUSTFLAGS= CARGO_ENCODED_RUSTFLAGS= RUSTC_WRAPPER= RUSTC_WORKSPACE_WRAPPER= CARGO_PROFILE_RELEASE_OPT_LEVEL=3 CARGO_PROFILE_RELEASE_LTO=false CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16 CARGO_PROFILE_RELEASE_DEBUG=false CARGO_PROFILE_RELEASE_STRIP=none CARGO_PROFILE_RELEASE_PANIC=unwind CARGO_PROFILE_RELEASE_INCREMENTAL=false CARGO_PROFILE_RELEASE_OVERFLOW_CHECKS=false CARGO_PROFILE_RELEASE_DEBUG_ASSERTIONS=false
.PHONY: measure-engine-wasm
measure-engine-wasm: qualification-tooling-check
	$(ENGINE_WASM_ENV) cargo build --locked --manifest-path qualification/Cargo.toml -p engine-view-wasm --bin engine-view-wasm --release --target wasm32-unknown-unknown --no-default-features --features candidate --target-dir target/engine-view-wasm/candidate
	node qualification/engine-view-wasm.mjs

.PHONY: qualify-engine-wasm
qualify-engine-wasm: qualification-tooling-check
	$(ENGINE_WASM_ENV) cargo build --locked --manifest-path qualification/Cargo.toml -p engine-view-wasm --bin engine-view-admission --release --target wasm32-unknown-unknown --no-default-features --features candidate --target-dir target/engine-view-wasm/candidate
	node qualification/engine-view-wasm.mjs admission
	cargo clippy --locked --manifest-path qualification/Cargo.toml -p engine-view-wasm --features candidate --all-targets -- -D warnings

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
	cargo test --locked -p ic-platform-protocol --features registry-settings --lib registry_settings::tests
	cargo clippy --locked -p ic-platform-protocol --features registry-settings --all-targets -- -D warnings
	RUSTDOCFLAGS='-D warnings' cargo doc --locked -p ic-platform-protocol --features registry-settings --no-deps
	cargo check --locked -p ic-platform-protocol --features registry-settings --lib --target wasm32-unknown-unknown
	cargo test --locked --manifest-path qualification/Cargo.toml -p engine-view-consumer --features registry-settings registry_settings
	cargo clippy --locked --manifest-path qualification/Cargo.toml -p engine-view-consumer --features registry-settings --all-targets -- -D warnings
	cargo check --locked --manifest-path qualification/Cargo.toml -p engine-view-consumer --features registry-settings --tests --target wasm32-unknown-unknown

GRAPH_TARGET ?= wasm32-unknown-unknown
.PHONY: qualify-marketplace
qualify-marketplace: qualification-tooling-check
	cargo test --locked -p ic-platform-protocol --features cloud-engine-marketplace --lib
	cargo clippy --locked -p ic-platform-protocol --features cloud-engine-marketplace --all-targets -- -D warnings
	RUSTDOCFLAGS='-D warnings' cargo doc --locked -p ic-platform-protocol --features cloud-engine-marketplace --no-deps
	cargo test --locked --manifest-path qualification/Cargo.toml -p engine-view-consumer --features marketplace marketplace
	cargo clippy --locked --manifest-path qualification/Cargo.toml -p engine-view-consumer --features marketplace --all-targets -- -D warnings
	cargo check --locked --manifest-path qualification/Cargo.toml -p engine-view-consumer --features marketplace --tests --target wasm32-unknown-unknown
	cargo clippy --locked --manifest-path qualification/Cargo.toml -p engine-view-profile --features marketplace --test page_arguments -- -D warnings
	cargo test --locked --manifest-path qualification/Cargo.toml -p engine-view-profile --features marketplace --release --test page_arguments -- --test-threads=1

.PHONY: qualify-registry-runtime
.PHONY: qualify-marketplace-wasm
qualify-marketplace-wasm: qualification-tooling-check
	cargo clippy --locked --manifest-path qualification/Cargo.toml -p engine-view-wasm --features marketplace --bin marketplace-admission -- -D warnings
	$(ENGINE_WASM_ENV) cargo build --locked --manifest-path qualification/Cargo.toml -p engine-view-wasm --bin marketplace-admission --release --target wasm32-unknown-unknown --no-default-features --features marketplace --target-dir target/engine-view-wasm/marketplace
	node qualification/engine-view-wasm.mjs marketplace

qualify-registry-runtime: qualification-tooling-check
	cargo clippy --locked --manifest-path qualification/Cargo.toml -p engine-view-profile --features registry-settings --test registry -- -D warnings
	cargo clippy --locked --manifest-path qualification/Cargo.toml -p engine-view-wasm --features registry-settings --bin registry-admission -- -D warnings
	cargo test --locked --manifest-path qualification/Cargo.toml -p engine-view-profile --features registry-settings --release --test registry -- --nocapture --test-threads=1
	$(ENGINE_WASM_ENV) cargo build --locked --manifest-path qualification/Cargo.toml -p engine-view-wasm --bin registry-admission --release --target wasm32-unknown-unknown --no-default-features --features registry-settings --target-dir target/engine-view-wasm/registry
	node qualification/engine-view-wasm.mjs registry

REGISTRY_GRAPH_PACKAGES := ic-platform-protocol|serde|serde_core|serde_derive|proc-macro2|quote|syn|unicode-ident
PROTOCOL_GRAPH_PACKAGES := ic-platform-protocol|ic-platform-types|cfg-if|crc32fast|data-encoding|serde|serde_core|serde_derive|serde_bytes|thiserror|thiserror-impl|proc-macro2|quote|syn|unicode-ident
MARKETPLACE_GRAPH_PACKAGES := $(PROTOCOL_GRAPH_PACKAGES)|num-bigint|num-integer|num-traits|autocfg
# The native comparison fixture intentionally exposes independent Candid schemas.
# Its oracle graph is excluded on Wasm and never admitted into product graphs.
CONSUMER_ORACLE_GRAPH_PACKAGES :=
CONSUMER_MARKETPLACE_BIGINT_IDENTITIES := 1
ifeq ($(findstring wasm,$(GRAPH_TARGET)),)
CONSUMER_ORACLE_GRAPH_PACKAGES := |anyhow|ar_archive_writer|array-init|arrayvec|autocfg|binrw|binrw_derive|block-buffer|bytemuck|byteorder|candid|candid_derive|cc|cpufeatures|crypto-common|digest|either|find-msvc-tools|generic-array|hex|ic_principal|lazy_static|leb128|libc|memchr|num-bigint|num-integer|num-traits|object|paste|pretty|psm|sha2|shlex|stacker|typed-arena|typenum|unicode-width|version_check
CONSUMER_MARKETPLACE_BIGINT_IDENTITIES := 2
endif
PROTOCOL_GRAPH_CHECK = perl "$(TOKO_TOOLING_ROOT)/scripts/ci/check-cargo-graph.pl" --target "$(GRAPH_TARGET)" --edges normal,build --explain
.PHONY: check-protocol-graphs
check-protocol-graphs: qualification-tooling-check
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)/qualification" --package engine-view-wasm --no-default-features --features marketplace --forbid-pattern '^(?!(engine-view-wasm|$(MARKETPLACE_GRAPH_PACKAGES))$$)' --require ic-platform-types=1 --require ic-platform-protocol=1 --require num-bigint=1
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)" --package ic-platform-protocol --no-default-features --features cloud-engine-marketplace --forbid-pattern '^(?!($(MARKETPLACE_GRAPH_PACKAGES))$$)' --require ic-platform-types=1 --require num-bigint=1
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)/qualification" --package engine-view-consumer --features marketplace --forbid-pattern '^(?!(engine-view-consumer|$(MARKETPLACE_GRAPH_PACKAGES)$(CONSUMER_ORACLE_GRAPH_PACKAGES))$$)' --require ic-platform-types=1 --require ic-platform-protocol=1 --require num-bigint=$(CONSUMER_MARKETPLACE_BIGINT_IDENTITIES)
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)/qualification" --package engine-view-wasm --no-default-features --features registry-settings --forbid-pattern '^(?!(engine-view-wasm|$(PROTOCOL_GRAPH_PACKAGES))$$)' --require ic-platform-types=1 --require ic-platform-protocol=1
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)" --package ic-platform-protocol --no-default-features --features registry-settings --forbid-pattern '^(?!($(REGISTRY_GRAPH_PACKAGES))$$)'
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)" --package ic-platform-protocol --forbid-pattern '^(?!ic-platform-protocol$$)'
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)" --package ic-platform-protocol --no-default-features --features cloud-engine --forbid-pattern '^(?!ic-platform-protocol$$)'
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)" --package ic-platform-protocol --no-default-features --features cloud-engine-response --forbid-pattern '^(?!($(PROTOCOL_GRAPH_PACKAGES))$$)' --require ic-platform-types=1
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)/qualification" --package engine-view-consumer --forbid-pattern '^(?!(engine-view-consumer|$(PROTOCOL_GRAPH_PACKAGES)$(CONSUMER_ORACLE_GRAPH_PACKAGES))$$)' --require ic-platform-types=1 --require ic-platform-protocol=1
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)/qualification" --package engine-view-consumer --features registry-settings --forbid-pattern '^(?!(engine-view-consumer|$(PROTOCOL_GRAPH_PACKAGES)$(CONSUMER_ORACLE_GRAPH_PACKAGES))$$)' --require ic-platform-types=1 --require ic-platform-protocol=1
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)/qualification" --package engine-view-wasm --no-default-features --features candidate --forbid-pattern '^(?!(engine-view-wasm|$(PROTOCOL_GRAPH_PACKAGES))$$)' --require ic-platform-types=1 --require ic-platform-protocol=1

.PHONY: upgrade-help check-upgrade-tooling
upgrade-help:
	node scripts/upgrade-canister.mjs --help

check-upgrade-tooling:
	node --test qualification/upgrade-tooling.test.mjs

.PHONY: qualify-platform-certificate
qualify-platform-certificate: toko-tooling-check
	cargo check --locked -p ic-platform --no-default-features
	cargo test --locked -p ic-platform --features certificate-decode
	cargo clippy --locked -p ic-platform --features certificate-decode --all-targets -- -D warnings
	RUSTDOCFLAGS='-D warnings' cargo doc --locked -p ic-platform --features certificate-decode --no-deps
	cargo clippy --locked -p ic-platform --features certificate-decode --all-targets --target wasm32-unknown-unknown -- -D warnings
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)" --package ic-platform --no-default-features --forbid-pattern '^(?!ic-platform$$)'
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)" --package ic-platform --features certificate-decode --forbid-pattern '^(?!(ic-platform|ic-certification|serde|serde_core|serde_derive|serde_cbor|half|thiserror|thiserror-impl|proc-macro2|quote|syn|unicode-ident|hex|sha2|digest|block-buffer|crypto-common|generic-array|typenum|version_check|cfg-if|cpufeatures|libc)$$)' --require ic-certification=1

.PHONY: qualify-platform-authentication
qualify-platform-authentication: qualify-platform-certificate
	cargo test --locked -p ic-platform --features certificate-authentication
	cargo clippy --locked -p ic-platform --features certificate-authentication --all-targets -- -D warnings
	RUSTDOCFLAGS='-D warnings' cargo doc --locked -p ic-platform --features certificate-authentication --no-deps
	cargo clippy --locked -p ic-platform --features certificate-authentication --all-targets --target wasm32-unknown-unknown -- -D warnings
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)" --package ic-platform --features certificate-authentication --forbid-pattern '^(?!(ic-platform|ic-platform-types|ic-certification|ic-verify-bls-signature|ic_bls12_381|ff|group|pairing|rand_core|subtle|lazy_static|crc32fast|data-encoding|serde|serde_core|serde_derive|serde_bytes|serde_cbor|half|thiserror|thiserror-impl|proc-macro2|quote|syn|unicode-ident|hex|sha2|digest|block-buffer|crypto-common|generic-array|typenum|version_check|cfg-if|cpufeatures|libc)$$)' --require ic-platform-types=1 --require ic-certification=1 --require ic-verify-bls-signature=1

.PHONY: qualify-platform-observed
qualify-platform-observed: qualification-tooling-check runtime-server-check
	cargo clippy --locked --manifest-path qualification/Cargo.toml -p platform-certificates --all-targets -- -D warnings
	node qualification/server.mjs run cargo run --locked --manifest-path qualification/Cargo.toml -p platform-certificates

.PHONY: qualify-platform-wasm
qualify-platform-wasm: qualification-tooling-check
	cargo clippy --locked --manifest-path qualification/Cargo.toml -p platform-certificates --bin certificate-admission --bin mainnet-admission -- -D warnings
	cargo clippy --locked --manifest-path qualification/Cargo.toml -p platform-certificates --bin certificate-admission --bin mainnet-admission --target wasm32-unknown-unknown -- -D warnings
	cargo run --locked --manifest-path qualification/Cargo.toml -p platform-certificates --bin certificate-admission
	$(ENGINE_WASM_ENV) cargo build --locked --manifest-path qualification/Cargo.toml -p platform-certificates --bin certificate-admission --bin mainnet-admission --release --target wasm32-unknown-unknown --target-dir target/platform-certificate-wasm
	cargo run --locked --manifest-path qualification/Cargo.toml -p platform-certificates --bin mainnet-admission
	node qualification/platform-certificate-wasm.mjs

.PHONY: qualify-canister-signature-decode
qualify-canister-signature-decode: toko-tooling-check
	cargo test --locked -p ic-platform --features canister-signature-decode
	cargo clippy --locked -p ic-platform --features canister-signature-decode --all-targets -- -D warnings
	cargo clippy --locked -p ic-platform --features canister-signature-decode --all-targets --target wasm32-unknown-unknown -- -D warnings
	RUSTDOCFLAGS='-D warnings' cargo doc --locked -p ic-platform --features canister-signature-decode --no-deps
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)" --package ic-platform --features canister-signature-decode --forbid-pattern '^(?!(ic-platform|ic-certification|serde|serde_core|serde_derive|serde_cbor|half|thiserror|thiserror-impl|proc-macro2|quote|syn|unicode-ident|hex|sha2|digest|block-buffer|crypto-common|generic-array|typenum|version_check|cfg-if|cpufeatures|libc)$$)' --require ic-certification=1

.PHONY: qualify-canister-signature-authentication
qualify-canister-signature-authentication: toko-tooling-check
	cargo test --locked -p ic-platform --features canister-signature-authentication
	cargo clippy --locked -p ic-platform --features canister-signature-authentication --all-targets -- -D warnings
	cargo clippy --locked -p ic-platform --features canister-signature-authentication --all-targets --target wasm32-unknown-unknown -- -D warnings
	RUSTDOCFLAGS='-D warnings' cargo doc --locked -p ic-platform --features canister-signature-authentication --no-deps
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)" --package ic-platform --features canister-signature-authentication --forbid-pattern '^(?!(ic-platform|ic-platform-types|ic-certification|ic-verify-bls-signature|ic_bls12_381|ff|group|pairing|rand_core|subtle|lazy_static|crc32fast|data-encoding|serde|serde_core|serde_derive|serde_bytes|serde_cbor|half|thiserror|thiserror-impl|proc-macro2|quote|syn|unicode-ident|hex|sha2|digest|block-buffer|crypto-common|generic-array|typenum|version_check|cfg-if|cpufeatures|libc|spki|der|const-oid|hybrid-array)$$)' --require ic-platform-types=1 --require ic-certification=1 --require ic-verify-bls-signature=1 --require spki=1

.PHONY: qualify-canister-signature-observed
qualify-canister-signature-observed: qualification-tooling-check runtime-server-check
	cargo clippy --locked --manifest-path qualification/Cargo.toml -p platform-certificates --features canister-signature --bin signature-observed -- -D warnings
	node qualification/server.mjs run cargo run --locked --manifest-path qualification/Cargo.toml -p platform-certificates --features canister-signature --bin signature-observed

.PHONY: qualify-canister-signature-wasm
qualify-canister-signature-wasm: qualification-tooling-check
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)/qualification" --package platform-certificates --features canister-signature --forbid ic-testkit --forbid candid --forbid wat --forbid ic-platform-runtime --forbid ic-platform-protocol --forbid ic-agent --require ic-platform-types=1 --require ic-platform=1
	cargo clippy --locked --manifest-path qualification/Cargo.toml -p platform-certificates --features canister-signature --bin signature-admission -- -D warnings
	cargo clippy --locked --manifest-path qualification/Cargo.toml -p platform-certificates --features canister-signature --bin signature-admission --target wasm32-unknown-unknown -- -D warnings
	cargo run --locked --manifest-path qualification/Cargo.toml -p platform-certificates --features canister-signature --bin signature-admission
	$(ENGINE_WASM_ENV) cargo build --locked --manifest-path qualification/Cargo.toml -p platform-certificates --features canister-signature --bin signature-admission --release --target wasm32-unknown-unknown --target-dir target/platform-certificate-wasm
	node qualification/platform-certificate-wasm.mjs signature-admission

.PHONY: qualify-auth-consumer
qualify-auth-consumer: qualification-tooling-check
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)/qualification" --package platform-certificates --features auth-consumer --forbid ic-testkit --forbid candid --forbid wat --forbid ic-platform-runtime --forbid ic-platform-protocol --forbid ic-agent --forbid ic-auth --forbid canic --require ic-platform-types=1 --require ic-platform=1
	cargo clippy --locked --manifest-path qualification/Cargo.toml -p platform-certificates --features auth-consumer --bin auth-admission -- -D warnings
	cargo clippy --locked --manifest-path qualification/Cargo.toml -p platform-certificates --features auth-consumer --bin auth-admission --target wasm32-unknown-unknown -- -D warnings
	cargo run --locked --manifest-path qualification/Cargo.toml -p platform-certificates --features auth-consumer --bin auth-admission
	$(ENGINE_WASM_ENV) cargo build --locked --manifest-path qualification/Cargo.toml -p platform-certificates --features auth-consumer --bin auth-admission --release --target wasm32-unknown-unknown --target-dir target/platform-certificate-wasm
	node qualification/platform-certificate-wasm.mjs auth-admission

.PHONY: qualify-platform-crypto qualify-auth-crypto qualify-query-settings
qualify-platform-crypto: qualification-tooling-check
	cargo clippy --locked -p ic-platform --no-default-features --features ed25519,ecdsa --all-targets -- -D warnings
	cargo clippy --locked -p ic-platform --no-default-features --features ed25519,ecdsa --target wasm32-unknown-unknown -- -D warnings
	RUSTDOCFLAGS='-D warnings' cargo doc --locked -p ic-platform --features ed25519,ecdsa --no-deps
	cargo clippy --locked --manifest-path qualification/Cargo.toml -p platform-certificates --features crypto --bin crypto-admission -- -D warnings
	cargo clippy --locked --manifest-path qualification/Cargo.toml -p platform-certificates --features crypto --bin crypto-admission --target wasm32-unknown-unknown -- -D warnings
	cargo run --locked --manifest-path qualification/Cargo.toml -p platform-certificates --features crypto --bin crypto-admission
	$(ENGINE_WASM_ENV) cargo build --locked --manifest-path qualification/Cargo.toml -p platform-certificates --features crypto --bin crypto-admission --release --target wasm32-unknown-unknown --target-dir target/platform-certificate-wasm
	node qualification/platform-certificate-wasm.mjs crypto-admission

qualify-auth-crypto:
	node qualification/auth-crypto.mjs

qualify-query-settings:
	node qualification/query-settings.mjs

.PHONY: qualify-release release-publication-admission
qualify-release: qualification-tooling-check ic-tools-check
	node scripts/qualify-release.mjs

release-publication-admission: qualification-tooling-check
	node scripts/release-publication-admission.mjs

.PHONY: qualify-auth-runtime
qualify-auth-runtime: qualification-tooling-check runtime-server-check
	node qualification/server.mjs run node qualification/auth-runtime/run.mjs

.PHONY: qualify-http-request
qualify-http-request: qualification-tooling-check
	cargo clippy --locked -p ic-platform-protocol --no-default-features --features http-request --all-targets -- -D warnings
	cargo clippy --locked --manifest-path qualification/Cargo.toml -p engine-view-consumer --features http-request --test http_request -- -D warnings
	cargo test --locked --manifest-path qualification/Cargo.toml -p engine-view-consumer --features http-request --test http_request
	cargo check --locked -p ic-platform-protocol --no-default-features --features http-request --target wasm32-unknown-unknown
	RUSTDOCFLAGS='-D warnings' cargo doc --locked -p ic-platform-protocol --no-default-features --features http-request --no-deps
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)" --package ic-platform-protocol --no-default-features --features http-request --forbid-pattern '^(?!ic-platform-protocol$$)'

.PHONY: qualify-runtime-raw qualify-storage-runtime
qualify-runtime-raw: qualification-tooling-check runtime-server-check
	cargo clippy --locked -p ic-platform-runtime --no-default-features --features system-raw --all-targets --target wasm32-unknown-unknown -- -D warnings
	RUSTDOCFLAGS='-D warnings' cargo doc --locked -p ic-platform-runtime --no-default-features --features system-raw --target wasm32-unknown-unknown --no-deps
	cargo clippy --locked --manifest-path qualification/Cargo.toml -p runtime-exports --no-default-features --features raw --target wasm32-unknown-unknown -- -D warnings
	cargo clippy --locked --manifest-path qualification/Cargo.toml -p runtime-installed --all-targets -- -D warnings
	$(ENGINE_WASM_ENV) cargo build --locked --manifest-path qualification/Cargo.toml -p runtime-exports --no-default-features --features raw --target wasm32-unknown-unknown --release --target-dir target/runtime-raw
	node qualification/runtime-raw.mjs
	node qualification/server.mjs run cargo run --locked --manifest-path qualification/Cargo.toml -p runtime-installed -- raw
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)" --package ic-platform-runtime --no-default-features --features system-raw --forbid-pattern '^(?!ic-platform-runtime$$)'
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)/qualification" --package runtime-exports --no-default-features --features raw --forbid-pattern '^(?!(runtime-exports|ic-platform-runtime)$$)'

qualify-storage-runtime:
	node qualification/storage-runtime/check.mjs

.PHONY: qualify-query-hard-cut qualify-native-read
qualify-query-hard-cut:
	QUERY_COHORT=candidate node qualification/query-hard-cut.mjs
	node qualification/query-hard-cut.mjs

.PHONY: test-native
test-native: qualification-tooling-check
	cargo test --locked -p ic-platform --all-features native::

qualify-native-read: test-native
	cargo test --locked --manifest-path qualification/Cargo.toml -p platform-certificates --features native-read --test native_transport
	cargo clippy --locked -p ic-platform --all-features --all-targets -- -D warnings

.PHONY: qualify-registry-codecs
qualify-registry-codecs:
	cargo test --locked -p ic-platform-protocol --all-features
	cargo clippy --locked -p ic-platform-protocol --all-features --all-targets -- -D warnings
	cargo test --locked --manifest-path qualification/Cargo.toml -p engine-view-consumer --all-features
	cargo clippy --locked --manifest-path qualification/Cargo.toml -p engine-view-consumer --all-features --all-targets -- -D warnings

.PHONY: qualify-raw-rand
qualify-raw-rand: qualification-tooling-check
	cargo test --locked -p ic-platform-protocol --no-default-features --features raw-rand raw_rand
	cargo clippy --locked -p ic-platform-protocol --no-default-features --features raw-rand --all-targets -- -D warnings
	RUSTDOCFLAGS='-D warnings' cargo doc --locked -p ic-platform-protocol --no-default-features --features raw-rand --no-deps
	cargo test --locked --manifest-path qualification/Cargo.toml -p engine-view-consumer --features raw-rand --test raw_rand
	cargo clippy --locked --manifest-path qualification/Cargo.toml -p engine-view-wasm --features raw-rand --bin raw-rand-admission -- -D warnings
	$(ENGINE_WASM_ENV) cargo build --locked --manifest-path qualification/Cargo.toml -p engine-view-wasm --bin raw-rand-admission --release --target wasm32-unknown-unknown --no-default-features --features raw-rand --target-dir target/engine-view-wasm/raw-rand
	node qualification/engine-view-wasm.mjs raw-rand
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)" --package ic-platform-protocol --no-default-features --features raw-rand --forbid-pattern '^(?!ic-platform-protocol$$)'

.PHONY: qualify-management-chunks
qualify-management-chunks: qualification-tooling-check
	cargo clippy --locked -p ic-platform-protocol --no-default-features --features management-chunks --all-targets -- -D warnings
	RUSTDOCFLAGS='-D warnings' cargo doc --locked -p ic-platform-protocol --no-default-features --features management-chunks --no-deps
	cargo clippy --locked --manifest-path qualification/Cargo.toml -p engine-view-consumer --features management-chunks --test management_chunks -- -D warnings
	cargo test --locked --manifest-path qualification/Cargo.toml -p engine-view-consumer --features management-chunks --test management_chunks
	cargo clippy --locked --manifest-path qualification/Cargo.toml -p engine-view-wasm --features management-chunks --bin management-chunks-admission -- -D warnings
	$(ENGINE_WASM_ENV) cargo build --locked --manifest-path qualification/Cargo.toml -p engine-view-wasm --bin management-chunks-admission --release --target wasm32-unknown-unknown --no-default-features --features management-chunks --target-dir target/engine-view-wasm/management-chunks
	node qualification/engine-view-wasm.mjs management-chunks
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)" --package ic-platform-protocol --no-default-features --features management-chunks --forbid-pattern '^(ic-platform-runtime|ic-agent|candid|ic-management-canister-types)$$' --require ic-platform-types=1

.PHONY: qualify-ingress-packets
qualify-ingress-packets: toko-tooling-check
	cargo test --locked -p ic-platform --no-default-features --features ingress-packets ingress
	cargo clippy --locked -p ic-platform --no-default-features --features ingress-packets --all-targets -- -D warnings
	RUSTDOCFLAGS='-D warnings' cargo doc --locked -p ic-platform --no-default-features --features ingress-packets --no-deps
	cargo check --locked -p ic-platform --all-features --target wasm32-unknown-unknown
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)" --package ic-platform --target x86_64-unknown-linux-gnu --no-default-features --features ingress-packets --forbid-pattern '^(ic-agent|reqwest|ic-platform-runtime)$$' --require ic-platform-types=1 --require ic-transport-types=1

.PHONY: qualify-management-snapshots
qualify-management-snapshots: qualification-tooling-check
	cargo clippy --locked -p ic-platform-protocol --no-default-features --features management-snapshot-fixtures --all-targets -- -D warnings
	cargo clippy --locked -p ic-platform-protocol --no-default-features --features management-snapshots --all-targets -- -D warnings
	RUSTDOCFLAGS='-D warnings' cargo doc --locked -p ic-platform-protocol --no-default-features --features management-snapshot-fixtures --no-deps
	cargo clippy --locked --manifest-path qualification/Cargo.toml -p engine-view-consumer --features management-snapshots --test management_snapshots -- -D warnings
	cargo test --locked --manifest-path qualification/Cargo.toml -p engine-view-consumer --features management-snapshots --test management_snapshots
	cargo clippy --locked --manifest-path qualification/Cargo.toml -p engine-view-wasm --features management-snapshots --bin management-snapshots-admission -- -D warnings
	$(ENGINE_WASM_ENV) cargo build --locked --manifest-path qualification/Cargo.toml -p engine-view-wasm --bin management-snapshots-admission --release --target wasm32-unknown-unknown --no-default-features --features management-snapshots --target-dir target/engine-view-wasm/management-snapshots
	node qualification/engine-view-wasm.mjs management-snapshots
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)" --package ic-platform-protocol --no-default-features --features management-snapshots --forbid-pattern '^(ic-platform-runtime|ic-agent|candid|ic-management-canister-types)$$' --require ic-platform-types=1
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)" --package ic-platform-protocol --no-default-features --features management-snapshot-fixtures --forbid-pattern '^(ic-platform-runtime|ic-agent|candid|ic-management-canister-types)$$' --require ic-platform-types=1

.PHONY: qualify-caffeine qualify-native-signed
qualify-caffeine: qualification-tooling-check
	cargo clippy --locked -p ic-platform-protocol --no-default-features --features caffeine --all-targets -- -D warnings
	RUSTDOCFLAGS='-D warnings' cargo doc --locked -p ic-platform-protocol --no-default-features --features caffeine --no-deps
	cargo clippy --locked --manifest-path qualification/Cargo.toml -p engine-view-consumer --features caffeine --test caffeine -- -D warnings
	cargo test --locked --manifest-path qualification/Cargo.toml -p engine-view-consumer --features caffeine --test caffeine
	cargo clippy --locked --manifest-path qualification/Cargo.toml -p engine-view-wasm --features caffeine --bin caffeine-admission -- -D warnings
	$(ENGINE_WASM_ENV) cargo build --locked --manifest-path qualification/Cargo.toml -p engine-view-wasm --bin caffeine-admission --release --target wasm32-unknown-unknown --no-default-features --features caffeine --target-dir target/engine-view-wasm/caffeine
	node qualification/engine-view-wasm.mjs caffeine
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)" --package ic-platform-protocol --no-default-features --features caffeine --forbid-pattern '^(ic-platform-runtime|ic-agent|candid|ic-management-canister-types)$$' --require ic-platform-types=1

qualify-native-signed: test-native ic-tools-check
	cargo clippy --locked -p ic-platform --no-default-features --features native-signed --all-targets -- -D warnings
	RUSTDOCFLAGS='-D warnings' cargo doc --locked -p ic-platform --no-default-features --features native-signed --no-deps
	cargo clippy --locked --manifest-path qualification/Cargo.toml -p platform-certificates --features native-signed-secp256k1 --test native_signed --bin native-signed-installed --bin native-anonymous-installed -- -D warnings
	cargo test --locked --manifest-path qualification/Cargo.toml -p platform-certificates --features native-signed-secp256k1 --test native_signed
	node qualification/server.mjs run cargo run --locked --manifest-path qualification/Cargo.toml -p platform-certificates --features native-signed-secp256k1 --bin native-signed-installed
	node qualification/server.mjs run cargo run --locked --manifest-path qualification/Cargo.toml -p platform-certificates --features native-signed --bin native-anonymous-installed

.PHONY: qualify-blob-suppliers
qualify-blob-suppliers: qualification-tooling-check
	node qualification/blob-supplier/run.mjs

.PHONY: qualify-management-ledger
qualify-management-ledger: qualification-tooling-check
	cargo clippy --locked -p ic-platform-protocol --no-default-features --features ledger --all-targets -- -D warnings
	RUSTDOCFLAGS='-D warnings' cargo doc --locked -p ic-platform-protocol --no-default-features --features ledger --no-deps
	cargo clippy --locked --manifest-path qualification/Cargo.toml -p engine-view-consumer --features ledger --test ledger -- -D warnings
	cargo test --locked --manifest-path qualification/Cargo.toml -p engine-view-consumer --features ledger --test ledger
	cargo clippy --locked --manifest-path qualification/Cargo.toml -p engine-view-wasm --features ledger --bin ledger-admission -- -D warnings
	$(ENGINE_WASM_ENV) cargo build --locked --manifest-path qualification/Cargo.toml -p engine-view-wasm --features ledger --bin ledger-admission --release --target wasm32-unknown-unknown --target-dir target/engine-view-wasm/ledger
	node qualification/engine-view-wasm.mjs ledger
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)" --package ic-platform-protocol --no-default-features --features ledger --forbid-pattern '^(ic-platform-runtime|ic-agent|candid|ic-management-canister-types)$$' --require ic-platform-types=1

.PHONY: qualify-sns-neurons
qualify-sns-neurons: qualification-tooling-check
	cargo clippy --locked -p ic-platform-protocol --no-default-features --features sns-neurons --all-targets -- -D warnings
	RUSTDOCFLAGS='-D warnings' cargo doc --locked -p ic-platform-protocol --no-default-features --features sns-neurons --no-deps
	cargo clippy --locked --manifest-path qualification/Cargo.toml -p engine-view-consumer --features sns-neurons --test sns -- -D warnings
	cargo test --locked --manifest-path qualification/Cargo.toml -p engine-view-consumer --features sns-neurons --test sns
	cargo clippy --locked --manifest-path qualification/Cargo.toml -p engine-view-wasm --features sns-neurons --bin sns-admission -- -D warnings
	$(ENGINE_WASM_ENV) cargo build --locked --manifest-path qualification/Cargo.toml -p engine-view-wasm --features sns-neurons --bin sns-admission --release --target wasm32-unknown-unknown --target-dir target/engine-view-wasm/sns-neurons
	node qualification/engine-view-wasm.mjs sns-neurons
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)" --package ic-platform-protocol --no-default-features --features sns-neurons --forbid-pattern '^(ic-platform-runtime|ic-agent|candid|ic-management-canister-types|num-bigint|sha2)$$' --require ic-platform-types=1

.PHONY: check-release-evidence
check-release-evidence: qualification-tooling-check
	node --test scripts/tests/release-audit.test.mjs scripts/tests/qualification-wiring.test.mjs scripts/tests/release-flow.test.mjs

.PHONY: qualify-stable-structures
qualify-stable-structures: qualification-tooling-check
	cargo test --locked -p ic-platform --no-default-features --features stable-structures --test stable_structures
	cargo clippy --locked -p ic-platform --no-default-features --features stable-structures --test stable_structures -- -D warnings
	cargo clippy --locked -p ic-platform --no-default-features --features stable-structures --test stable_structures --target wasm32-unknown-unknown -- -D warnings
	RUSTDOCFLAGS='-D warnings' cargo doc --locked -p ic-platform --no-default-features --features stable-structures --no-deps
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)" --package ic-platform --no-default-features --features stable-structures --require ic-stable-structures=1 --forbid ic-platform-runtime --forbid ic-platform-protocol
	$(PROTOCOL_GRAPH_CHECK) --consumer "$(CURDIR)" --package ic-platform --no-default-features --forbid-pattern '^(?!ic-platform$$)'
