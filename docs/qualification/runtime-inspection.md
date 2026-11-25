# Runtime ingress inspection

[Platform #42](https://github.com/dragginzgame/ic-platform/issues/42) adds
`#[inspect_message]` under the existing `entrypoint-macros` feature. It generates
the fixed `canister_inspect_message` Wasm export and takes an explicit
`max_method_bytes`. The Rust handler receives an `Inspection<'_>` with the
requested method name, caller Principal and argument byte count.

The IC0 adapter checks the method-name length before reserving or copying bytes.
Over-limit names, allocation failures and malformed UTF-8 leave ingress unaccepted.
The Runtime invokes `ic0.accept_message` only when the handler returns
`InspectionDecision::Accept`. Inspection reads the argument length without
copying payload bytes, decoding them, scheduling an executor task or replying.

The IC runs this hook for external update ingress. Query and inter-canister
calls bypass it. Applications must enforce authorization and resource limits
in the ordinary handler as well. See the
[IC ingress inspection specification](https://docs.internetcomputer.org/references/ic-interface-spec/canister-interface/#ingress-message-inspection).

`make qualify-runtime-exports` exercises the generated Wasm export and its
imports. The host fixture checks exact-limit acceptance, caller, method and
argument-size refusals, over-limit refusal before copying, and malformed UTF-8.
It also checks that inspection never calls the payload-copy or reply imports.
`make qualify-runtime-installed` runs the same artifact in PocketIC. It checks
controller ingress acceptance, non-controller, argument-size and method-name
refusals at submission, and a real canister call reaching the same method
refused for ingress.
