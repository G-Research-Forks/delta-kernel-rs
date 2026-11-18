//! Validation for time of day in nanoseconds feature support

use super::TableFeature;
use crate::schema::{PrimitiveType, Schema};
use crate::table_configuration::TableConfiguration;
use crate::transforms::{transform_output_type, SchemaTransform};
use crate::utils::require;
use crate::{KernelError, KernelResult};

/// Validates that if a table schema contains `time(9)` columns, the table must have the
/// `timeNanos-dev` feature in both reader and writer features.
pub(crate) fn validate_time_nanos_feature_support(tc: &TableConfiguration) -> KernelResult<()> {
    let protocol = tc.protocol();
    if !protocol.has_table_feature(&TableFeature::TimeNanos) {
        require!(
            !schema_contains_time_nanos(&tc.logical_schema()),
            KernelError::unsupported(
                "Table contains time(9) columns but does not have the required \
                 'timeNanos-dev' feature in reader and writer features"
            )
        );
    }
    Ok(())
}

/// Checks if any column in the schema (including nested structs, arrays, maps) uses the
/// `time(9)` primitive type.
pub(crate) fn schema_contains_time_nanos(schema: &Schema) -> bool {
    UsesTimeNanos.transform_struct(schema).is_err()
}

struct UsesTimeNanos;

impl<'a> SchemaTransform<'a> for UsesTimeNanos {
    transform_output_type!(|'a, T| Result<(), ()>);

    fn transform_primitive(&mut self, ptype: &'a PrimitiveType) -> Result<(), ()> {
        match ptype {
            PrimitiveType::TimeNanos => Err(()),
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
    fn test_time_nanos_feature_validation() {
        let schema_with = schema! {
            not_null "id": INTEGER,
            nullable "value": TIME_NANOS,
        };
        let schema_without = schema! {
            not_null "id": INTEGER,
            nullable "name": STRING,
        };
        let nested_struct = schema! {
            not_null "id": INTEGER,
            nullable "nested": {
                nullable "inner": TIME_NANOS,
            },
        };
        let in_array = schema! {
            nullable "arr": [ nullable TIME_NANOS ],
        };
        let in_map_key = schema! {
            nullable "map": { TIME_NANOS => nullable STRING },
        };
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
            "Table contains time(9) columns but does not have the required \
             'timeNanos-dev' feature in reader and writer features",
        );
    }
}
