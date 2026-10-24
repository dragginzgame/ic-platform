//! Metadata follows Account's existing ordered Serde fields without normalization.

impl ic_wire_metadata::typescript::Wire for super::Account {
    fn native_schema(
        context: &mut ic_wire_metadata::typescript::SchemaContext,
    ) -> Result<serde_json::Value, ic_wire_metadata::typescript::GenerationError> {
        Ok(serde_json::json!({
            "k": "record",
            "fields": [
                {"name": "owner", "value": context.describe::<crate::Principal>()?},
                {"name": "subaccount", "value": context.describe::<Option<super::Subaccount>>()?}
            ]
        }))
    }
}
