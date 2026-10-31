# Changelog

Subsequent releases share the workspace version and root changelog. The initial
independent publication is retained below as historical context.

## [0.1.0]

### ➕ Added

- Establish the canonical owned Principal, Account, Subaccount and identity errors
  independently of Platform runtime releases. Preserve existing representations,
  Serde, bounded text admission and account equality/order/hash semantics.
- Retain explicit registry Principal conversions and separately selected upstream
  self-authentication. Platform retains external ledger DTO conversions.
