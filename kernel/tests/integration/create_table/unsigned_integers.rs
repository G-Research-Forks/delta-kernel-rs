//! Unsigned integer integration tests for the CreateTable API.
//!
//! Tests that creating a table with unsigned integer columns in the schema automatically adds the
//! `unsignedIntegerTypes-dev` feature to the protocol.

use std::sync::Arc;

use buoyant_kernel as delta_kernel;
use delta_kernel::committer::FileSystemCommitter;
use delta_kernel::schema::{ArrayType, DataType, MapType, SchemaRef, StructField, StructType};
use delta_kernel::snapshot::Snapshot;
use delta_kernel::table_features::{
    ColumnMappingMode, TableFeature, TABLE_FEATURES_MIN_READER_VERSION,
    TABLE_FEATURES_MIN_WRITER_VERSION,
};
use delta_kernel::transaction::create_table::create_table;
use delta_kernel::DeltaResult;
use test_utils::{
    cm_properties, multi_schema_with_type, nested_schema_with_type, schema_with_type,
    test_table_setup,
};

fn create_and_load(schema: SchemaRef, cm_mode: &str) -> DeltaResult<Arc<Snapshot>> {
    let (_temp_dir, table_path, engine) = test_table_setup()?;
    let _ = create_table(&table_path, schema, "Test/1.0")
        .with_table_properties(cm_properties(cm_mode))
        .build(engine.as_ref(), Box::new(FileSystemCommitter::new()))?
        .commit(engine.as_ref())?;
    let table_url = delta_kernel::try_parse_uri(&table_path)?;
    Snapshot::builder_for(table_url).build(engine.as_ref())
}

/// Unsigned integer schemas auto-enable unsignedIntegerTypes-dev across schema shapes and column
/// mapping modes.
#[rstest::rstest]
fn test_create_table_with_unsigned_integers(
    #[values(DataType::UINT8, DataType::UINT16, DataType::UINT32, DataType::UINT64)]
    dtype: DataType,
    #[values(schema_with_type, nested_schema_with_type, multi_schema_with_type)]
    make_schema: fn(DataType) -> SchemaRef,
    #[values("none", "name", "id")] cm_mode: &str,
) -> DeltaResult<()> {
    let schema = make_schema(dtype);
    let snapshot = create_and_load(schema.clone(), cm_mode)?;

    let table_config = snapshot.table_configuration();
    assert!(
        table_config.is_feature_supported(&TableFeature::UnsignedIntegerTypes),
        "unsignedIntegerTypes-dev feature should be supported"
    );
    let protocol = table_config.protocol();
    assert!(protocol.min_reader_version() >= TABLE_FEATURES_MIN_READER_VERSION);
    assert!(protocol.min_writer_version() >= TABLE_FEATURES_MIN_WRITER_VERSION);
    assert!(protocol
        .reader_features()
        .is_some_and(|f| f.contains(&TableFeature::UnsignedIntegerTypes)));
    assert!(protocol
        .writer_features()
        .is_some_and(|f| f.contains(&TableFeature::UnsignedIntegerTypes)));

    if cm_mode != "none" {
        let expected_mode = match cm_mode {
            "name" => ColumnMappingMode::Name,
            "id" => ColumnMappingMode::Id,
            _ => unreachable!(),
        };
        assert_eq!(table_config.column_mapping_mode(), expected_mode);
    }

    let read_schema = snapshot.schema();
    let stripped = super::column_mapping::strip_column_mapping_metadata(&read_schema);
    assert_eq!(
        &stripped,
        schema.as_ref(),
        "Schema should round-trip through create table"
    );
    Ok(())
}

/// Unsigned integers inside arrays and maps also auto-enable the feature.
#[rstest::rstest]
#[case::array_element(DataType::from(ArrayType::new(DataType::UINT32, true)))]
#[case::map_key(DataType::from(MapType::new(DataType::UINT64, DataType::STRING, true)))]
#[case::map_value(DataType::from(MapType::new(DataType::STRING, DataType::UINT8, true)))]
fn test_create_table_with_unsigned_integers_in_collections(
    #[case] dtype: DataType,
) -> DeltaResult<()> {
    let snapshot = create_and_load(schema_with_type(dtype), "none")?;
    assert!(snapshot
        .table_configuration()
        .is_feature_supported(&TableFeature::UnsignedIntegerTypes));
    Ok(())
}

/// A schema without unsigned integer columns should not add the feature.
#[test]
fn test_create_table_no_unsigned_integers_no_feature() -> DeltaResult<()> {
    let schema = Arc::new(StructType::try_new(vec![
        StructField::not_null("id", DataType::LONG),
        StructField::nullable("name", DataType::STRING),
    ])?);
    let snapshot = create_and_load(schema, "none")?;
    assert!(!snapshot
        .table_configuration()
        .is_feature_supported(&TableFeature::UnsignedIntegerTypes));
    Ok(())
}
