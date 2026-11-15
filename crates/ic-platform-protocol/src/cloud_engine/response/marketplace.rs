//! Selected daily-price declarations; consumer interpretation stays outside codecs.

use crate::candid::{self, Decode, Error, Limits, Nat, Reader, Schema};
use ic_platform_types::Principal;
use serde::{Deserialize, Serialize};

projection! {
    /// Raw source class, without an inferred hardware or capacity specification.
    #[derive(Copy)]
    enum NodeType {
        /// Unnumbered type4 class.
        #[serde(rename = "type4")]
        449_540_154 => Type4,
        /// Source type4_1 class.
        #[serde(rename = "type4_1")]
        4_172_531_116 => Type4_1,
        /// Source type4_2 class.
        #[serde(rename = "type4_2")]
        4_172_531_117 => Type4_2,
        /// Source type4_3 class.
        #[serde(rename = "type4_3")]
        4_172_531_118 => Type4_3,
        /// Source type4_4 class.
        #[serde(rename = "type4_4")]
        4_172_531_119 => Type4_4,
        /// Source type4_5 class.
        #[serde(rename = "type4_5")]
        4_172_531_120 => Type4_5,
    }
}
projection! {
    /// Raw whole-cycle amounts per node per day, without tariff selection.
    Price {
        /// Source gross daily cycles; never recalculated with a newer fee.
        2_625_678_760 => gross_cycles: Nat [required],
        /// Source net daily cycles.
        3_039_038_447 => net_cycles: Nat [required],
    }
}
projection! {
    /// Public price scope and source observation, not a binding quote.
    PriceEntry {
        /// Data-center scope; absent differs from present empty text.
        22_399 => dc: Option<String> [strict_optional],
        /// Raw signed Unix nanoseconds.
        272_465_847 => updated_at: i64 [required],
        /// Raw source class.
        287_472_983 => node_type: NodeType [required],
        /// Optional provider scope, using the canonical Principal.
        590_999_857 => provider: Option<Principal> [strict_optional],
        /// Raw daily amounts.
        3_364_572_809 => price: Price [required],
    }
}
projection! {
    /// One source declaration, with no key normalization or uniqueness policy.
    MarketplaceEntry {
        /// Opaque source key.
        5_343_647 => key: String [required],
        /// Source price declaration.
        1_887_860_850 => entry: PriceEntry [required],
    }
}

/// Decode exactly one vector of marketplace declarations under shared bounds.
///
/// Preserves arbitrary naturals, signed times, source order and optional scopes.
/// Unknown selected class tags refuse; unknown record fields are validated and
/// skipped. Prices remain daily amounts; caller policy owns key binding, limits
/// on decimal digits, uniqueness, gross/net relationships and scope precedence.
///
/// # Errors
/// Returns shared framing, type, arity, allocation and resource refusals.
pub fn decode_list_marketplace_prices_response(
    bytes: &[u8],
    limits: Limits,
) -> Result<Vec<MarketplaceEntry>, Error> {
    candid::decode(bytes, limits)
}
