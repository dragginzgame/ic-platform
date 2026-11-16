//! Independent native Candid schemas; no product projection macros are reused.
use candid::{CandidType, Deserialize, Nat, Principal};
use serde::Serialize;

#[derive(Serialize, CandidType, Deserialize, Debug, Clone)]
pub struct SubnetFrame {
    pub replica_version: String,
    pub maximum_state_size: Option<u64>,
    pub halted: bool,
    pub node_ids: Vec<Principal>,
}
#[derive(Serialize, CandidType, Deserialize, Debug, Clone)]
pub struct ViewFrame {
    #[serde(rename = "")]
    pub extra: Vec<String>,
    pub id: String,
    pub subnet_id: Option<Principal>,
    pub engine_operator_id: Option<Principal>,
    pub subnet: Option<SubnetFrame>,
    pub deleted_at: Option<i64>,
    pub node_ids: Vec<Principal>,
}
#[derive(Serialize, CandidType, Deserialize, Debug, Clone)]
pub struct FullViewFrame {
    pub id: String,
    pub subnet_id: Option<Principal>,
    pub engine_operator_id: Option<Principal>,
    pub subnet: Option<SubnetFrame>,
    pub deleted_at: Option<i64>,
    pub node_ids: Vec<Principal>,
}
#[derive(Serialize, CandidType, Deserialize, Debug, Clone)]
pub enum NodeClass {
    #[serde(rename = "type4")]
    Type4,
    #[serde(rename = "type4_1")]
    Type4_1,
    #[serde(rename = "type4_2")]
    Type4_2,
    #[serde(rename = "type4_3")]
    Type4_3,
    #[serde(rename = "type4_4")]
    Type4_4,
    #[serde(rename = "type4_5")]
    Type4_5,
}
#[derive(Serialize, CandidType, Deserialize, Debug, Clone)]
pub struct Price {
    pub gross_cycles: Nat,
    pub net_cycles: Nat,
}
#[derive(Serialize, CandidType, Deserialize, Debug, Clone)]
pub struct PriceEntry {
    pub dc: Option<String>,
    pub updated_at: i64,
    pub node_type: NodeClass,
    pub provider: Option<Principal>,
    pub price: Price,
}
#[derive(Serialize, CandidType, Deserialize, Debug, Clone)]
pub struct MarketplaceEntry {
    pub key: String,
    pub entry: PriceEntry,
}
#[derive(Serialize, CandidType, Deserialize, Debug, Clone)]
pub struct PageArgs {
    pub limit: Option<Nat>,
    pub start: Option<Principal>,
    pub rejoining: Option<String>,
}
#[derive(Serialize, CandidType, Deserialize, Debug, Clone)]
pub struct AvailableNode {
    pub dc: Option<String>,
    pub node_type: NodeClass,
    pub node_id: Principal,
    pub provider: Principal,
    pub operator: Principal,
    pub sev_enabled: bool,
}
#[derive(Serialize, CandidType, Deserialize, Debug, Clone)]
pub struct Page {
    pub next: Option<Principal>,
    pub registry_version: u64,
    pub nodes: Vec<AvailableNode>,
}
#[derive(Serialize, CandidType, Deserialize, Debug, Clone)]
pub enum EngineError {
    Internal(String),
    ControllerError(String),
    InvalidTransition(String),
    SlugConflict(String),
    NotFound,
    Unauthorized,
    AlreadyExists,
    NotVetted,
    BadRequest(String),
}
#[derive(Serialize, CandidType, Deserialize, Debug, Clone)]
pub enum PageResult {
    #[serde(rename = "ok")]
    Ok(Option<Page>),
    #[serde(rename = "err")]
    Err(EngineError),
}

/// Upstream schema codec for isolated real-consumer qualification only.
pub use candid;
/// Encode an independent fixture and refuse a fixture exceeding its selected size.
/// This is fixture setup, not a product allocation-bound claim.
///
/// # Errors
/// Returns upstream encoding diagnostics or a fixture-size refusal.
pub fn encode_fixture<T: CandidType>(value: &T, maximum: usize) -> Result<Vec<u8>, String> {
    let bytes = candid::encode_one(value).map_err(|e| e.to_string())?;
    if bytes.len() > maximum {
        return Err("fixture output bound".into());
    }
    Ok(bytes)
}
/// Independent Registry-copy status schema.
#[derive(Serialize, CandidType, Deserialize)]
pub struct RegistryCopyStatus {
    /// Source version.
    pub version: u64,
    /// Latest reported version.
    pub latest: u64,
    /// Source status.
    pub caught_up: bool,
}
/// Independent placement result record.
#[derive(Serialize, CandidType, Deserialize)]
pub struct Placement {
    /// Optional returned subnet.
    pub subnet_id: Option<Principal>,
}
