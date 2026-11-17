//! Validation for time of day in nanoseconds feature support

use std::borrow::Cow;

use super::TableFeature;
use crate::schema::{PrimitiveType, Schema};
use crate::table_configuration::TableConfiguration;
use crate::transforms::SchemaTransform;
use crate::utils::require;
use crate::{transform_output_type, DeltaResult, Error};

/// Validates that if a table schema contains `time_nanos` columns, the table must have the
/// `timeNanos-dev` feature in both reader and writer features.
pub(crate) fn validate_time_nanos_feature_support(tc: &TableConfiguration) -> DeltaResult<()> {
    let protocol = tc.protocol();
    if !protocol.has_table_feature(&TableFeature::TimeNanos) {
        require!(
            !schema_contains_time_nanos(&tc.logical_schema()),
            Error::unsupported(
                "Table contains time_nanos columns but does not have the required \
                 'timeNanos-dev' feature in reader and writer features"
            )
        );
    }
    Ok(())
}

/// Checks if any column in the schema (including nested structs, arrays, maps) uses the
/// `time_nanos` primitive type.
pub(crate) fn schema_contains_time_nanos(schema: &Schema) -> bool {
    let mut uses_time_nanos = UsesTimeNanos(false);
    let _ = uses_time_nanos.transform_struct(schema);
    uses_time_nanos.0
}

struct UsesTimeNanos(bool);

impl<'a> SchemaTransform<'a> for UsesTimeNanos {
    transform_output_type!(|'a, T| Option<Cow<'a, T>>);

    fn transform_primitive(&mut self, ptype: &'a PrimitiveType) -> Option<Cow<'a, PrimitiveType>> {
        if *ptype == PrimitiveType::TimeNanos {
            self.0 = true;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::actions::Protocol;
    use crate::schema::{ArrayType, DataType, MapType, StructField, StructType};
    use crate::table_features::TableFeature;
    use crate::utils::test_utils::assert_schema_feature_validation;

    #[test]
    fn test_time_nanos_feature_validation() {
        let schema_with = StructType::new_unchecked([
            StructField::not_null("id", DataType::INTEGER),
            StructField::nullable("value", DataType::TIME_NANOS),
        ]);
        let schema_without = StructType::new_unchecked([
            StructField::not_null("id", DataType::INTEGER),
            StructField::nullable("name", DataType::STRING),
        ]);
        let nested_struct = StructType::new_unchecked([
            StructField::not_null("id", DataType::INTEGER),
            StructField::nullable(
                "nested",
                StructType::new_unchecked([StructField::nullable("inner", DataType::TIME_NANOS)]),
            ),
        ]);
        let in_array = StructType::new_unchecked([StructField::nullable(
            "arr",
            ArrayType::new(DataType::TIME_NANOS, true),
        )]);
        let in_map_key = StructType::new_unchecked([StructField::nullable(
            "map",
            MapType::new(DataType::TIME_NANOS, DataType::STRING, true),
        )]);
        let protocol_with =
            Protocol::try_new_modern([TableFeature::TimeNanos], [TableFeature::TimeNanos]).unwrap();
        let protocol_without =
            Protocol::try_new_modern(TableFeature::EMPTY_LIST, TableFeature::EMPTY_LIST).unwrap();

        assert_schema_feature_validation(
            &schema_with,
            &schema_without,
            &protocol_with,
            &protocol_without,
            &[&nested_struct, &in_array, &in_map_key],
            "Table contains time_nanos columns but does not have the required \
             'timeNanos-dev' feature in reader and writer features",
        );
    }
}
