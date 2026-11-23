//! Validation for time of day in nanoseconds feature support

use super::TableFeature;
#[cfg(feature = "udt-in-dev")]
use crate::schema::UserDefinedType;
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

    // The feature is required for any use of the type, including as a UDT's physical type.
    #[cfg(feature = "udt-in-dev")]
    fn transform_user_defined(&mut self, udt: &'a UserDefinedType) -> Result<(), ()> {
        self.transform(udt.sql_type())
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "udt-in-dev")]
    use std::collections::HashMap;

    use crate::actions::Protocol;
    use crate::schema::schema;
    #[cfg(feature = "udt-in-dev")]
    use crate::schema::{ArrayType, DataType, MapType, UserDefinedType};
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

    #[cfg(feature = "udt-in-dev")]
    #[rstest::rstest]
    fn test_time_nanos_feature_validation_inside_udt(
        #[values(DataType::TIME_NANOS)] data_type: DataType,
    ) {
        let udt = |sql_type: DataType| -> DataType {
            UserDefinedType::try_new(sql_type, HashMap::new())
                .unwrap()
                .into()
        };
        let schema_with = schema! {
            not_null "id": INTEGER,
            nullable "udt": (udt(data_type.clone())),
        };
        let schema_without = schema! {
            not_null "id": INTEGER,
            nullable "udt": (udt(DataType::LONG)),
        };
        let struct_schema_with = schema! {
            nullable "udt": (udt(schema! { nullable "inner": (data_type.clone()) }.into())),
        };
        let array_schema_with = schema! {
            nullable "udt": (udt(ArrayType::new(data_type.clone(), true).into())),
        };
        let map_schema_with = schema! {
            nullable "udt": (udt(MapType::new(DataType::STRING, data_type, true).into())),
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
            &[&struct_schema_with, &array_schema_with, &map_schema_with],
            "does not have the required 'timeNanos-dev' feature",
        );
    }
}
