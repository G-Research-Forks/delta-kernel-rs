//! Validation for FLOAT16 feature support

use std::borrow::Cow;

use super::TableFeature;
use crate::schema::{PrimitiveType, Schema};
use crate::table_configuration::TableConfiguration;
use crate::transforms::SchemaTransform;
use crate::utils::require;
use crate::{transform_output_type, DeltaResult, Error};

/// Validates that if a table schema contains FLOAT16 columns, the table must
/// have the Float16 feature in both reader and writer features.
pub(crate) fn validate_float16_feature_support(tc: &TableConfiguration) -> DeltaResult<()> {
    let protocol = tc.protocol();
    if !protocol.has_table_feature(&TableFeature::Float16) {
        require!(
            !schema_contains_float16(&tc.logical_schema()),
            Error::unsupported(
                "Table contains FLOAT16 columns but does not have the required 'float16' feature in reader and writer features"
            )
        );
    }
    Ok(())
}

/// Checks if any column in the schema (including nested structs, arrays, maps) uses
/// the FLOAT16 primitive type.
pub(crate) fn schema_contains_float16(schema: &Schema) -> bool {
    let mut uses_float16 = UsesFloat16(false);
    let _ = uses_float16.transform_struct(schema);
    uses_float16.0
}

struct UsesFloat16(bool);

impl<'a> SchemaTransform<'a> for UsesFloat16 {
    transform_output_type!(|'a, T| Option<Cow<'a, T>>);

    fn transform_primitive(&mut self, ptype: &'a PrimitiveType) -> Option<Cow<'a, PrimitiveType>> {
        if *ptype == PrimitiveType::Float16 {
            self.0 = true;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::actions::Protocol;
    use crate::schema::{DataType, PrimitiveType, StructField, StructType};
    use crate::table_features::TableFeature;
    use crate::utils::test_utils::assert_schema_feature_validation;

    #[test]
    fn test_float16_feature_validation() {
        let schema_with = StructType::new_unchecked([
            StructField::new("id", DataType::INTEGER, false),
            StructField::new("ts", DataType::Primitive(PrimitiveType::Float16), true),
        ]);
        let schema_without = StructType::new_unchecked([
            StructField::new("id", DataType::INTEGER, false),
            StructField::new("name", DataType::STRING, true),
        ]);
        let nested_schema_with = StructType::new_unchecked([
            StructField::new("id", DataType::INTEGER, false),
            StructField::new(
                "nested",
                DataType::Struct(Box::new(StructType::new_unchecked([StructField::new(
                    "inner_ts",
                    DataType::Primitive(PrimitiveType::Float16),
                    true,
                )]))),
                true,
            ),
        ]);
        let protocol_with =
            Protocol::try_new_modern([TableFeature::Float16], [TableFeature::Float16]).unwrap();
        let protocol_without =
            Protocol::try_new_modern(TableFeature::EMPTY_LIST, TableFeature::EMPTY_LIST).unwrap();

        assert_schema_feature_validation(
            &schema_with,
            &schema_without,
            &protocol_with,
            &protocol_without,
            &[&nested_schema_with],
            "Table contains FLOAT16 columns but does not have the required 'float16' feature in reader and writer features",
        );
    }
}
