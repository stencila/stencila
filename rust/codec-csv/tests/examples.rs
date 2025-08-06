#![allow(clippy::print_stderr)]

use std::{fs, path::Path};

use codec::{
    DecodeOptions, EncodeOptions,
    common::{
        eyre::{Context, Result, eyre},
        serde_json, tempfile,
    },
    format::Format,
    schema::{Datatable, Node},
};

/// Load a test datatable from JSON file
fn load_test_datatable(filename: &str) -> Result<Node> {
    let json_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join(filename);

    let json_content = fs::read_to_string(&json_path)?;

    let node = serde_json::from_str::<Datatable>(&json_content)
        .wrap_err_with(|| format!("Unable to decode {filename} as datatable"))?;
    Ok(Node::Datatable(node))
}

/// Test roundtrip encoding/decoding for all formats with all test datatables
#[test]
fn test_roundtrip_all_formats() -> Result<()> {
    let test_files = ["datatable-1.json", "datatable-2.json", "datatable-3.json"];
    let formats = [
        ("csv", Format::Csv),
        ("tsv", Format::Tsv),
        ("parquet", Format::Parquet),
        ("arrow", Format::Arrow),
    ];

    // Test each datatable with each format
    for test_file in &test_files {
        let original_node = load_test_datatable(test_file)?;
        eprintln!("Testing {test_file} with all formats");

        for (extension, format) in &formats {
            eprintln!("  Testing {extension} format");

            // Create temporary file
            let temp_file = tempfile::Builder::new()
                .prefix(&format!("test_{}_", test_file.replace(".json", "")))
                .suffix(&format!(".{extension}"))
                .tempfile()?;
            let temp_path = temp_file.path();

            // Encode to file
            let encode_options = EncodeOptions {
                format: Some(format.clone()),
                ..Default::default()
            };

            codec_csv::encode_to_path(&original_node, temp_path, Some(encode_options))?;

            // Decode from file
            let decode_options = DecodeOptions {
                format: Some(format.clone()),
                ..Default::default()
            };

            let decoded_node = codec_csv::decode_from_path(temp_path, Some(decode_options))?;

            // Verify structure is preserved
            match (&original_node, &decoded_node) {
                (Node::Datatable(orig_dt), Node::Datatable(decoded_dt)) => {
                    assert_eq!(
                        orig_dt.columns.len(),
                        decoded_dt.columns.len(),
                        "Column count mismatch for {test_file} in {extension} format"
                    );

                    for (i, (orig_col, decoded_col)) in orig_dt
                        .columns
                        .iter()
                        .zip(decoded_dt.columns.iter())
                        .enumerate()
                    {
                        assert_eq!(
                            orig_col.name, decoded_col.name,
                            "Column {i} name mismatch for {test_file} in {extension} format"
                        );
                        assert_eq!(
                            orig_col.values.len(),
                            decoded_col.values.len(),
                            "Column {i} value count mismatch for {test_file} in {extension} format"
                        );
                    }
                }
                _ => {
                    return Err(eyre!(
                        "Expected Datatable nodes for {test_file} in {extension} format",
                    ));
                }
            }
        }
    }

    Ok(())
}
