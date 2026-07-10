//! Property-based tests for text conversion
//!
//! Feature: universal-file-converter

use proptest::prelude::*;

use crate::converter::text::TextConverter;
use crate::converter::Converter;
use crate::models::{
    ConflictResolution, ConversionOptions, FileNamingPattern, TextEncoding,
};

/// Helper: Create default conversion options for text tests
fn default_options() -> ConversionOptions {
    ConversionOptions {
        quality: 80,
        resize: None,
        encoding: None,
        csv_delimiter: None,
        file_naming: FileNamingPattern::Original,
        conflict_resolution: ConflictResolution::Overwrite,
        delete_source: false,
    }
}

// =========================================================================
// Strategies for generating test data
// =========================================================================

/// Strategy to generate a simple JSON-safe string (no control chars that break YAML)
fn json_safe_string() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9_]{0,14}"
        .prop_filter("non-empty key", |s| !s.is_empty())
}

/// Strategy to generate a JSON leaf value (types that survive YAML roundtrip)
fn json_leaf_value() -> impl Strategy<Value = serde_json::Value> {
    prop_oneof![
        // String (avoid values that YAML might interpret as bool/null)
        json_safe_string()
            .prop_filter("not yaml keyword", |s| {
                let lower = s.to_lowercase();
                lower != "true" && lower != "false" && lower != "null"
                    && lower != "yes" && lower != "no" && lower != "on" && lower != "off"
            })
            .prop_map(serde_json::Value::String),
        // Integer
        (-1000i64..1000i64).prop_map(|n| serde_json::Value::Number(n.into())),
        // Boolean
        any::<bool>().prop_map(serde_json::Value::Bool),
        // Null
        Just(serde_json::Value::Null),
    ]
}

/// Strategy to generate a JSON value with possible nesting (for roundtrip tests)
fn json_value_strategy() -> impl Strategy<Value = serde_json::Value> {
    json_leaf_value().prop_recursive(
        3,  // max depth
        32, // max nodes
        4,  // items per collection
        |inner| {
            prop_oneof![
                // Object with nested values
                prop::collection::vec(
                    (json_safe_string(), inner.clone()),
                    1..=3
                )
                .prop_map(|entries| {
                    let mut map = serde_json::Map::new();
                    for (k, v) in entries {
                        if !k.is_empty() {
                            map.insert(k, v);
                        }
                    }
                    serde_json::Value::Object(map)
                }),
                // Array of nested values
                prop::collection::vec(inner, 1..=3)
                    .prop_map(serde_json::Value::Array),
            ]
        },
    )
}

/// Strategy to generate a JSON object (top-level must be object for YAML roundtrip)
fn json_object_strategy() -> impl Strategy<Value = serde_json::Value> {
    prop::collection::vec(
        (json_safe_string(), json_value_strategy()),
        1..=5,
    )
    .prop_map(|entries| {
        let mut map = serde_json::Map::new();
        for (k, v) in entries {
            if !k.is_empty() {
                map.insert(k, v);
            }
        }
        // Ensure at least one key exists
        if map.is_empty() {
            map.insert("key".to_string(), serde_json::Value::String("value".to_string()));
        }
        serde_json::Value::Object(map)
    })
}

/// Strategy to generate a nested JSON structure that contains at least one nested
/// object or array (guaranteed to fail CSV conversion)
fn nested_json_strategy() -> impl Strategy<Value = serde_json::Value> {
    // Generate objects that contain at least one nested object or array
    prop_oneof![
        // Object with nested object
        json_safe_string().prop_map(|key| {
            serde_json::json!([
                {key: {"inner": "value"}}
            ])
        }),
        // Object with nested array
        json_safe_string().prop_map(|key| {
            serde_json::json!([
                {key: [1, 2, 3]}
            ])
        }),
        // Multiple keys, one nested
        (json_safe_string(), json_safe_string()).prop_map(|(k1, k2)| {
            let mut map = serde_json::Map::new();
            map.insert(k1, serde_json::Value::String("flat".to_string()));
            map.insert(k2 + "_nested", serde_json::json!({"deep": true}));
            serde_json::Value::Array(vec![serde_json::Value::Object(map)])
        }),
    ]
}

/// Strategy to generate a simple CSV cell value (no commas, no quotes, no newlines)
fn csv_cell_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9]{1,10}"
}

/// Strategy to generate valid ASCII text that is valid in all target encodings
fn ascii_text_strategy() -> impl Strategy<Value = String> {
    // Only printable ASCII characters (0x20-0x7E) plus newlines
    prop::collection::vec(
        prop_oneof![
            (0x20u8..=0x7Eu8).prop_map(|b| b as char),
            Just('\n'),
        ],
        1..=80,
    )
    .prop_map(|chars| chars.into_iter().collect::<String>())
    .prop_filter("non-empty content", |s| !s.trim().is_empty())
}

// =========================================================================
// Property 4: JSON↔YAML ラウンドトリップ保存
//
// Feature: universal-file-converter, Property 4: JSON↔YAML roundtrip preservation
// For any valid JSON document, after JSON→YAML→JSON roundtrip conversion,
// key names, value types (number, string, boolean, null), nesting structure,
// and array element order are preserved.
//
// **Validates: Requirements 3.2, 3.6**
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn property_4_json_yaml_roundtrip(json_obj in json_object_strategy()) {
        let dir = tempfile::tempdir().unwrap();
        let json_input = dir.path().join("input.json");
        let yaml_output = dir.path().join("intermediate.yaml");
        let json_roundtrip = dir.path().join("roundtrip.json");

        // Write original JSON
        let json_str = serde_json::to_string_pretty(&json_obj).unwrap();
        std::fs::write(&json_input, &json_str).unwrap();

        let converter = TextConverter::new();
        let opts = default_options();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        // Step 1: JSON → YAML
        let result1 = rt.block_on(converter.convert(&json_input, &yaml_output, &opts));
        prop_assert!(result1.is_ok(), "JSON→YAML failed: {:?}", result1.err());

        // Step 2: YAML → JSON
        let result2 = rt.block_on(converter.convert(&yaml_output, &json_roundtrip, &opts));
        prop_assert!(result2.is_ok(), "YAML→JSON failed: {:?}", result2.err());

        // Step 3: Compare original and roundtripped values
        let roundtrip_str = std::fs::read_to_string(&json_roundtrip).unwrap();
        let roundtripped: serde_json::Value = serde_json::from_str(&roundtrip_str).unwrap();

        prop_assert_eq!(
            &json_obj, &roundtripped,
            "JSON→YAML→JSON roundtrip failed.\nOriginal: {}\nRoundtripped: {}",
            serde_json::to_string_pretty(&json_obj).unwrap(),
            serde_json::to_string_pretty(&roundtripped).unwrap()
        );
    }
}

// =========================================================================
// Property 5: CSV→JSON 変換の構造保存
//
// Feature: universal-file-converter, Property 5: CSV→JSON structural preservation
// For any valid CSV data (header row + N data rows, M columns), the JSON conversion
// result is an array of length N where each object's keys match the M headers.
//
// **Validates: Requirements 3.3**
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn property_5_csv_to_json_structure(
        num_rows in 1usize..=8,
        num_cols in 2usize..=6,
        cells in prop::collection::vec(csv_cell_strategy(), 48..=48),
    ) {
        let headers = (0..num_cols).map(|i| format!("col{}", i)).collect::<Vec<_>>();

        // Build CSV content
        let mut csv_content = headers.join(",");
        csv_content.push('\n');

        for row_idx in 0..num_rows {
            let row: Vec<&str> = (0..num_cols)
                .map(|col_idx| {
                    let idx = (row_idx * num_cols + col_idx) % cells.len();
                    cells[idx].as_str()
                })
                .collect();
            csv_content.push_str(&row.join(","));
            csv_content.push('\n');
        }

        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("test.csv");
        let output_path = dir.path().join("output.json");
        std::fs::write(&input_path, &csv_content).unwrap();

        let converter = TextConverter::new();
        let opts = default_options();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let result = rt.block_on(converter.convert(&input_path, &output_path, &opts));
        prop_assert!(result.is_ok(), "CSV→JSON conversion failed: {:?}", result.err());

        // Read and parse the output JSON
        let output_content = std::fs::read_to_string(&output_path).unwrap();
        let json_val: serde_json::Value = serde_json::from_str(&output_content).unwrap();

        // Verify: result is an array of length N
        let arr = json_val.as_array();
        prop_assert!(arr.is_some(), "Output is not a JSON array");
        let arr = arr.unwrap();
        prop_assert_eq!(
            arr.len(), num_rows,
            "Expected {} rows, got {}",
            num_rows, arr.len()
        );

        // Verify: each object's keys match the M headers
        for (i, item) in arr.iter().enumerate() {
            let obj = item.as_object();
            prop_assert!(obj.is_some(), "Row {} is not a JSON object", i);
            let obj = obj.unwrap();
            prop_assert_eq!(
                obj.len(), num_cols,
                "Row {} has {} keys, expected {}. Keys: {:?}",
                i, obj.len(), num_cols, obj.keys().collect::<Vec<_>>()
            );
            for header in &headers {
                prop_assert!(
                    obj.contains_key(header),
                    "Row {} missing key '{}'. Actual keys: {:?}",
                    i, header, obj.keys().collect::<Vec<_>>()
                );
            }
        }
    }
}

// =========================================================================
// Property 6: ネスト構造の CSV 変換拒否
//
// Feature: universal-file-converter, Property 6: Nested structure CSV conversion rejection
// For any structured data (JSON, YAML, TOML, XML) containing nested objects or arrays,
// attempting CSV conversion should return an error and produce no output file.
//
// **Validates: Requirements 3.8**
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn property_6_nested_csv_rejection(nested_json in nested_json_strategy()) {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("nested.json");
        let output_path = dir.path().join("output.csv");

        // Write nested JSON to file
        let json_str = serde_json::to_string_pretty(&nested_json).unwrap();
        std::fs::write(&input_path, &json_str).unwrap();

        let converter = TextConverter::new();
        let opts = default_options();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let result = rt.block_on(converter.convert(&input_path, &output_path, &opts));

        // Should return an error
        prop_assert!(
            result.is_err(),
            "Expected error for nested JSON→CSV conversion, but got Ok.\nInput: {}",
            json_str
        );

        // Output file should not exist
        prop_assert!(
            !output_path.exists(),
            "Output file should not exist after failed nested→CSV conversion"
        );
    }
}

// =========================================================================
// Property 20: エンコーディング変換の正確性
//
// Feature: universal-file-converter, Property 20: Encoding conversion accuracy
// For any valid text content and output encoding specification, decoding the output
// file with the specified encoding produces the original text content.
//
// **Validates: Requirements 3.4**
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn property_20_encoding_conversion_accuracy(
        text in ascii_text_strategy(),
        encoding_choice in prop_oneof![
            Just(TextEncoding::Utf8),
            Just(TextEncoding::ShiftJis),
            Just(TextEncoding::EucJp),
        ],
    ) {
        let dir = tempfile::tempdir().unwrap();
        // Use PlainText → Markdown conversion (identity transform) to exercise encoding
        let input_path = dir.path().join("input.txt");
        let output_path = dir.path().join("output.md");

        // Write input as UTF-8
        std::fs::write(&input_path, text.as_bytes()).unwrap();

        // Convert PlainText → Markdown with specified output encoding
        let mut opts = default_options();
        opts.encoding = Some(encoding_choice.clone());

        let converter = TextConverter::new();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let result = rt.block_on(converter.convert(&input_path, &output_path, &opts));
        prop_assert!(result.is_ok(), "Conversion failed: {:?}", result.err());

        // Read output bytes
        let output_bytes = std::fs::read(&output_path).unwrap();

        // Decode with the specified encoding
        let decoded = match &encoding_choice {
            TextEncoding::Utf8 => {
                String::from_utf8(output_bytes).unwrap()
            }
            TextEncoding::ShiftJis => {
                let (cow, _, had_errors) = encoding_rs::SHIFT_JIS.decode(&output_bytes);
                prop_assert!(!had_errors, "Shift_JIS decoding had errors");
                cow.into_owned()
            }
            TextEncoding::EucJp => {
                let (cow, _, had_errors) = encoding_rs::EUC_JP.decode(&output_bytes);
                prop_assert!(!had_errors, "EUC-JP decoding had errors");
                cow.into_owned()
            }
        };

        // plaintext_to_markdown is an identity transform, so decoded should equal original
        prop_assert_eq!(
            &text, &decoded,
            "Encoding roundtrip failed for {:?}.\nOriginal ({} bytes): {:?}\nDecoded ({} bytes): {:?}",
            encoding_choice, text.len(), &text[..text.len().min(50)],
            decoded.len(), &decoded[..decoded.len().min(50)]
        );
    }
}
