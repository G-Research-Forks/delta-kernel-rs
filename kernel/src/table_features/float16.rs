//! Validation for FLOAT16 feature support

use super::TableFeature;
use crate::schema::{PrimitiveType, Schema};
use crate::table_configuration::TableConfiguration;
use crate::transforms::{transform_output_type, SchemaTransform};
use crate::utils::require;
use crate::{KernelError, KernelResult};

/// Validates that if a table schema contains FLOAT16 columns, the table must
/// have the Float16 feature in both reader and writer features.
pub(crate) fn validate_float16_feature_support(tc: &TableConfiguration) -> KernelResult<()> {
    let protocol = tc.protocol();
    if !protocol.has_table_feature(&TableFeature::Float16) {
        require!(
            !schema_contains_float16(&tc.logical_schema()),
            KernelError::unsupported(
                "Table contains FLOAT16 columns but does not have the required 'float16' feature in reader and writer features"
            )
        );
    }
    Ok(())
}

/// Checks if any column in the schema (including nested structs, arrays, maps) uses
/// the FLOAT16 primitive type.
pub(crate) fn schema_contains_float16(schema: &Schema) -> bool {
    UsesFloat16.transform_struct(schema).is_err()
}

struct UsesFloat16;

impl<'a> SchemaTransform<'a> for UsesFloat16 {
    transform_output_type!(|'a, T| Result<(), ()>);

    fn transform_primitive(&mut self, ptype: &'a PrimitiveType) -> Result<(), ()> {
        match ptype {
            PrimitiveType::Float16 => Err(()),
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::actions::Protocol;
    use crate::schema::schema;
    use crate::table_features::TableFeature;
    use crate::unit_test_utils::assert_schema_feature_validation;

    #[test]
    fn test_float16_feature_validation() {
        let schema_with = schema! {
            not_null "id": INTEGER,
            nullable "value": FLOAT16,
        };
        let schema_without = schema! {
            not_null "id": INTEGER,
            nullable "name": STRING,
        };
        let nested_schema_with = schema! {
            not_null "id": INTEGER,
            nullable "nested": {
                nullable "inner_value": FLOAT16,
            },
        };
        let array_schema_with = schema! {
            not_null "id": INTEGER,
            nullable "arr": [ nullable FLOAT16 ],
        };
        let map_value_schema_with = schema! {
            not_null "id": INTEGER,
            nullable "map": { STRING => nullable FLOAT16 },
        };
        let protocol_with =
            Protocol::try_new_modern([TableFeature::Float16], [TableFeature::Float16]).unwrap();
        let protocol_without =
            Protocol::try_new_modern(TableFeature::EMPTY_LIST, TableFeature::EMPTY_LIST).unwrap();

        assert_schema_feature_validation(
            &schema_with,
            &schema_without,
            &protocol_with,
            &protocol_without,
            &[
                &nested_schema_with,
                &array_schema_with,
                &map_value_schema_with,
            ],
            "Table contains FLOAT16 columns but does not have the required 'float16' feature in reader and writer features",
        );
    }
}
