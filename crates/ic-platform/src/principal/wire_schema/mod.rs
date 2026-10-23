//! Native metadata only; Principal's existing Serde implementation owns bytes.

impl ic_wire_metadata::typescript::Wire for super::Principal {
    fn native_schema(
        context: &mut ic_wire_metadata::typescript::SchemaContext,
    ) -> Result<serde_json::Value, ic_wire_metadata::typescript::GenerationError> {
        context.describe::<ic_wire_metadata::Bytes<{ Self::MAX_LENGTH_IN_BYTES }>>()
    }
}
