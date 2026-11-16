# IC Platform Types

Owned Internet Computer `Principal`, `Account` and `Subaccount` identities,
with bounded text/byte admission and Serde representations. Identity validity
does not establish authorization.

Defaults are empty. Enable `principal-self-authenticating` for upstream SHA-224
derivation; it does not parse keys or prove possession. There is no Runtime,
Candid, network transport or implicit trust policy in this crate.

```rust
use ic_platform_types::{Account, Principal};

let owner = Principal::anonymous();
let account = Account::from(owner);
let mut output = [0; Account::MAX_LENGTH_IN_TEXT];
assert_eq!(account.write_text(&mut output), "2vxsx-fae");
```

The 0.21 line retains the 0.20.4 Principal/Account API and external text/Serde
contracts, and adds rendering into caller-owned buffers. An absent subaccount
and an explicit zero subaccount have the same effective identity; Serde retains
their distinct presence. Default subaccounts are omitted from canonical text.

Select one package source and version at public Rust type crossings. Updating
this crate alone does not migrate Runtime, Protocol, Platform or applications.
The independently qualified Types release does not claim the complete Platform
rebuild or downstream production migrations are finished.
