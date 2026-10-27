//! Standalone compiled catalog CLI, including exact filtering/content identity.
use ic_crypto_sha2::Sha256;
use std::io::Write;

#[derive(Debug, clap::Args)]
pub struct ExplainArgs {
    /// Include only this exact charge-rule ID, preserving the catalog metadata.
    #[arg(long, value_name = "RULE_ID")]
    pub rule: Option<String>,
    /// Print JSON on one line instead of using indentation.
    #[arg(long)]
    pub compact: bool,
}

pub fn write_computation_catalog(
    mut catalog: serde_json::Value,
    args: &ExplainArgs,
    mut output: impl Write,
) -> Result<(), String> {
    if let Some(rule_id) = &args.rule {
        let rules = catalog
            .get_mut("charge_rules")
            .and_then(serde_json::Value::as_object_mut)
            .ok_or_else(|| {
                "compiled computation catalog has no charge-rule dictionary".to_string()
            })?;
        let selected = rules
            .remove(rule_id)
            .ok_or_else(|| format!("unknown charge rule '{rule_id}'"))?;
        rules.clear();
        rules.insert(rule_id.clone(), selected);
        let object = catalog.as_object_mut().unwrap();
        if let Some(full_hash) = object.remove("catalog_sha256") {
            object
                .entry("full_catalog_sha256".to_string())
                .or_insert(full_hash);
        }
        let bytes = serde_json::to_vec(&catalog)
            .map_err(|error| format!("failed to hash computation catalog: {error}"))?;
        catalog["catalog_sha256"] = serde_json::Value::String(hex::encode(Sha256::hash(&bytes)));
    }
    if args.compact {
        serde_json::to_writer(&mut output, &catalog)
    } else {
        serde_json::to_writer_pretty(&mut output, &catalog)
    }
    .map_err(|error| format!("failed to write computation catalog: {error}"))?;
    output
        .write_all(b"\n")
        .and_then(|_| output.flush())
        .map_err(|error| format!("failed to write computation catalog: {error}"))
}
