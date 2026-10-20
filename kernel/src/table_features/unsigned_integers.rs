//! Validation for unsigned integer types feature support

use super::TableFeature;
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
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::actions::Protocol;
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
}
