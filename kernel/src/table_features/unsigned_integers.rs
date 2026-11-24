//! Validation for unsigned integer types feature support

use super::TableFeature;
#[cfg(feature = "udt-in-dev")]
use crate::schema::UserDefinedType;
use crate::schema::{PrimitiveType, Schema};
use crate::table_configuration::TableConfiguration;
use crate::transforms::{transform_output_type, SchemaTransform};
use crate::utils::require;
use crate::{KernelError, KernelResult};

/// Validates that if a table schema contains unsigned integer columns, the table must have the
/// `unsignedIntegerTypes-dev` feature in both reader and writer features.
pub(crate) fn validate_unsigned_integer_types_feature_support(
    tc: &TableConfiguration,
) -> KernelResult<()> {
    let protocol = tc.protocol();
    if !protocol.has_table_feature(&TableFeature::UnsignedIntegerTypes) {
        require!(
            !schema_contains_unsigned_integers(&tc.logical_schema()),
            KernelError::unsupported(
                "Table contains unsigned integer columns but does not have the required \
                 'unsignedIntegerTypes-dev' feature in reader and writer features"
            )
        );
    }
    Ok(())
}

/// Checks if any column in the schema (including nested structs, arrays, maps) uses an unsigned
/// integer primitive type.
pub(crate) fn schema_contains_unsigned_integers(schema: &Schema) -> bool {
    UsesUnsignedIntegers.transform_struct(schema).is_err()
}

struct UsesUnsignedIntegers;

impl<'a> SchemaTransform<'a> for UsesUnsignedIntegers {
    transform_output_type!(|'a, T| Result<(), ()>);

    fn transform_primitive(&mut self, ptype: &'a PrimitiveType) -> Result<(), ()> {
        match ptype {
            PrimitiveType::Uint8
            | PrimitiveType::Uint16
            | PrimitiveType::Uint32
            | PrimitiveType::Uint64 => Err(()),
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

    use rstest::rstest;

    use crate::actions::Protocol;
    #[cfg(feature = "udt-in-dev")]
    use crate::schema::{schema, UserDefinedType};
    use crate::schema::{ArrayType, DataType, MapType, StructField, StructType};
    use crate::table_features::TableFeature;
    use crate::unit_test_utils::assert_schema_feature_validation;

    #[rstest]
    fn test_unsigned_integer_types_feature_validation(
        #[values(DataType::UINT8, DataType::UINT16, DataType::UINT32, DataType::UINT64)]
        uint_type: DataType,
    ) {
        let schema_with = StructType::new_unchecked([
            StructField::not_null("id", DataType::INTEGER),
            StructField::nullable("value", uint_type.clone()),
        ]);
        let schema_without = StructType::new_unchecked([
            StructField::not_null("id", DataType::INTEGER),
            StructField::nullable("name", DataType::STRING),
        ]);
        let nested_struct = StructType::new_unchecked([
            StructField::not_null("id", DataType::INTEGER),
            StructField::nullable(
                "nested",
                StructType::new_unchecked([StructField::nullable("inner", uint_type.clone())]),
            ),
        ]);
        let in_array = StructType::new_unchecked([StructField::nullable(
            "arr",
            ArrayType::new(uint_type.clone(), true),
        )]);
        let in_map_key = StructType::new_unchecked([StructField::nullable(
            "map",
            MapType::new(uint_type, DataType::STRING, true),
        )]);
        let protocol_with = Protocol::try_new_modern(
            [TableFeature::UnsignedIntegerTypes],
            [TableFeature::UnsignedIntegerTypes],
        )
        .unwrap();
        let protocol_without =
            Protocol::try_new_modern(TableFeature::EMPTY_LIST, TableFeature::EMPTY_LIST).unwrap();

        assert_schema_feature_validation(
            &schema_with,
            &schema_without,
            &protocol_with,
            &protocol_without,
            &[&nested_struct, &in_array, &in_map_key],
            "Table contains unsigned integer columns but does not have the required \
             'unsignedIntegerTypes-dev' feature in reader and writer features",
        );
    }

    #[cfg(feature = "udt-in-dev")]
    #[rstest]
    fn test_unsigned_integers_feature_validation_inside_udt(
        #[values(DataType::UINT8, DataType::UINT16, DataType::UINT32, DataType::UINT64)] data_type: DataType,
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
        let protocol_with = Protocol::try_new_modern(
            [TableFeature::UnsignedIntegerTypes],
            [TableFeature::UnsignedIntegerTypes],
        )
        .unwrap();
        let protocol_without =
            Protocol::try_new_modern(TableFeature::EMPTY_LIST, TableFeature::EMPTY_LIST).unwrap();

        assert_schema_feature_validation(
            &schema_with,
            &schema_without,
            &protocol_with,
            &protocol_without,
            &[&struct_schema_with, &array_schema_with, &map_schema_with],
            "does not have the required 'unsignedIntegerTypes-dev' feature",
        );
    }
}
