use std::fs;
use std::path::Path;

use crate::encoding::{detect_and_decode, encode_string};
use crate::errors::ConversionError;
use crate::models::{ConversionOptions, CsvDelimiter, FileFormat, TextEncoding};

use super::Converter;

/// テキスト変換器
pub struct TextConverter;

impl TextConverter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TextConverter {
    fn default() -> Self {
        Self::new()
    }
}

/// サポートする全テキストフォーマット一覧（入力）
const TEXT_INPUT_FORMATS: &[FileFormat] = &[
    FileFormat::Json,
    FileFormat::Yaml,
    FileFormat::Toml,
    FileFormat::Xml,
    FileFormat::Csv,
    FileFormat::Markdown,
    FileFormat::PlainText,
];

/// サポートする全テキストフォーマット一覧（出力）
const TEXT_OUTPUT_FORMATS: &[FileFormat] = &[
    FileFormat::Json,
    FileFormat::Yaml,
    FileFormat::Toml,
    FileFormat::Xml,
    FileFormat::Csv,
    FileFormat::Markdown,
    FileFormat::PlainText,
];

impl Converter for TextConverter {
    fn supported_input_formats(&self) -> &[FileFormat] {
        TEXT_INPUT_FORMATS
    }

    fn supported_output_formats(&self) -> &[FileFormat] {
        TEXT_OUTPUT_FORMATS
    }

    fn can_convert(&self, from: &FileFormat, to: &FileFormat) -> bool {
        if from == to {
            return false;
        }
        matches!(
            (from, to),
            (FileFormat::Json, FileFormat::Yaml)
                | (FileFormat::Json, FileFormat::Toml)
                | (FileFormat::Json, FileFormat::Xml)
                | (FileFormat::Json, FileFormat::Csv)
                | (FileFormat::Yaml, FileFormat::Json)
                | (FileFormat::Yaml, FileFormat::Toml)
                | (FileFormat::Yaml, FileFormat::Xml)
                | (FileFormat::Yaml, FileFormat::Csv)
                | (FileFormat::Toml, FileFormat::Json)
                | (FileFormat::Toml, FileFormat::Yaml)
                | (FileFormat::Toml, FileFormat::Xml)
                | (FileFormat::Toml, FileFormat::Csv)
                | (FileFormat::Xml, FileFormat::Json)
                | (FileFormat::Xml, FileFormat::Yaml)
                | (FileFormat::Xml, FileFormat::Toml)
                | (FileFormat::Xml, FileFormat::Csv)
                | (FileFormat::Csv, FileFormat::Json)
                | (FileFormat::Markdown, FileFormat::PlainText)
                | (FileFormat::PlainText, FileFormat::Markdown)
        )
    }

    async fn convert(
        &self,
        input: &Path,
        output: &Path,
        options: &ConversionOptions,
    ) -> Result<(), ConversionError> {
        let file_name = input
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        // File size check (50MB limit for text files)
        let metadata = fs::metadata(input).map_err(|e| ConversionError::FileReadError {
            file_name: file_name.clone(),
            detail: e.to_string(),
        })?;
        let size_bytes = metadata.len();
        let limit_bytes: u64 = 50 * 1024 * 1024;
        if size_bytes > limit_bytes {
            return Err(ConversionError::FileSizeLimitError {
                file_name,
                size_mb: size_bytes / (1024 * 1024),
                limit_mb: 50,
            });
        }

        // Read input file with encoding detection
        let raw_bytes =
            fs::read(input).map_err(|e| ConversionError::FileReadError {
                file_name: file_name.clone(),
                detail: e.to_string(),
            })?;

        let content = detect_and_decode(&raw_bytes).map_err(|e| match e {
            ConversionError::DecodeError { detail, .. } => ConversionError::DecodeError {
                file_name: file_name.clone(),
                detail,
            },
            other => other,
        })?;

        // Determine source and target formats from file extensions
        let source_format =
            detect_format_from_path(input).ok_or_else(|| ConversionError::UnsupportedConversion {
                from: "unknown".to_string(),
                to: "unknown".to_string(),
            })?;
        let target_format = detect_format_from_path(output).ok_or_else(|| {
            ConversionError::UnsupportedConversion {
                from: format!("{:?}", source_format),
                to: "unknown".to_string(),
            }
        })?;

        // Dispatch to appropriate conversion handler
        let output_content = match (&source_format, &target_format) {
            (FileFormat::Markdown, FileFormat::PlainText) => markdown_to_plaintext(&content),
            (FileFormat::PlainText, FileFormat::Markdown) => plaintext_to_markdown(&content),
            _ => {
                let value =
                    parse_to_json_value(&content, &source_format, &file_name, options)?;
                serialize_from_json_value(&value, &target_format, &file_name, options)?
            }
        };

        // Encode output with specified encoding (default: UTF-8)
        let target_encoding = options.encoding.as_ref().unwrap_or(&TextEncoding::Utf8);
        let output_bytes = encode_string(&output_content, target_encoding).map_err(|e| match e {
            ConversionError::EncodeError { detail, .. } => ConversionError::EncodeError {
                file_name: file_name.clone(),
                detail,
            },
            other => other,
        })?;

        // Write output file
        fs::write(output, &output_bytes).map_err(|e| {
            ConversionError::FileWriteError {
                file_name,
                detail: e.to_string(),
            }
        })?;

        Ok(())
    }
}

/// Detect text format from file extension
fn detect_format_from_path(path: &Path) -> Option<FileFormat> {
    let ext = path.extension()?.to_string_lossy().to_lowercase();
    match ext.as_str() {
        "json" => Some(FileFormat::Json),
        "yaml" | "yml" => Some(FileFormat::Yaml),
        "toml" => Some(FileFormat::Toml),
        "xml" => Some(FileFormat::Xml),
        "csv" | "tsv" => Some(FileFormat::Csv),
        "md" | "markdown" => Some(FileFormat::Markdown),
        "txt" | "text" => Some(FileFormat::PlainText),
        _ => None,
    }
}

/// Parse input content into serde_json::Value intermediate representation
fn parse_to_json_value(
    content: &str,
    format: &FileFormat,
    file_name: &str,
    options: &ConversionOptions,
) -> Result<serde_json::Value, ConversionError> {
    match format {
        FileFormat::Json => serde_json::from_str(content).map_err(|e| {
            ConversionError::ParseError {
                file_name: file_name.to_string(),
                line: Some(e.line() as u32),
                detail: e.to_string(),
            }
        }),
        FileFormat::Yaml => serde_yaml::from_str(content).map_err(|e| {
            ConversionError::ParseError {
                file_name: file_name.to_string(),
                line: extract_yaml_line(&e),
                detail: e.to_string(),
            }
        }),
        FileFormat::Toml => toml::from_str(content).map_err(|e| {
            ConversionError::ParseError {
                file_name: file_name.to_string(),
                line: extract_toml_line(&e),
                detail: e.to_string(),
            }
        }),
        FileFormat::Xml => parse_xml_to_json(content, file_name),
        FileFormat::Csv => parse_csv_to_json(content, file_name, options),
        _ => Err(ConversionError::UnsupportedConversion {
            from: format!("{:?}", format),
            to: "JSON".to_string(),
        }),
    }
}

/// Serialize serde_json::Value to output format string
fn serialize_from_json_value(
    value: &serde_json::Value,
    format: &FileFormat,
    file_name: &str,
    options: &ConversionOptions,
) -> Result<String, ConversionError> {
    match format {
        FileFormat::Json => {
            serde_json::to_string_pretty(value).map_err(|e| ConversionError::EncodeError {
                file_name: file_name.to_string(),
                detail: e.to_string(),
            })
        }
        FileFormat::Yaml => {
            serde_yaml::to_string(value).map_err(|e| ConversionError::EncodeError {
                file_name: file_name.to_string(),
                detail: e.to_string(),
            })
        }
        FileFormat::Toml => serialize_to_toml(value, file_name),
        FileFormat::Xml => serialize_to_xml(value, file_name),
        FileFormat::Csv => serialize_to_csv(value, file_name, options),
        _ => Err(ConversionError::UnsupportedConversion {
            from: "JSON".to_string(),
            to: format!("{:?}", format),
        }),
    }
}

/// Serialize JSON value to TOML string
/// Note: TOML doesn't support top-level arrays, handle gracefully
fn serialize_to_toml(
    value: &serde_json::Value,
    file_name: &str,
) -> Result<String, ConversionError> {
    // TOML requires a top-level table (object), not arrays or primitives
    if !value.is_object() {
        return Err(ConversionError::UnsupportedConversion {
            from: "JSON".to_string(),
            to: "TOML (トップレベルがオブジェクトではありません)".to_string(),
        });
    }
    toml::to_string_pretty(value).map_err(|e| ConversionError::EncodeError {
        file_name: file_name.to_string(),
        detail: e.to_string(),
    })
}

/// Serialize JSON value to XML string
fn serialize_to_xml(
    value: &serde_json::Value,
    file_name: &str,
) -> Result<String, ConversionError> {
    let mut xml_output = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    json_value_to_xml(value, "root", &mut xml_output, 0);
    let _ = file_name; // used for error context if needed
    Ok(xml_output)
}

/// Recursively convert a JSON value to XML elements
fn json_value_to_xml(value: &serde_json::Value, tag: &str, output: &mut String, indent: usize) {
    let pad = "  ".repeat(indent);
    match value {
        serde_json::Value::Object(map) => {
            output.push_str(&format!("{}<{}>\n", pad, tag));
            for (key, val) in map {
                json_value_to_xml(val, key, output, indent + 1);
            }
            output.push_str(&format!("{}</{}>\n", pad, tag));
        }
        serde_json::Value::Array(arr) => {
            for item in arr {
                json_value_to_xml(item, tag, output, indent);
            }
        }
        serde_json::Value::String(s) => {
            let escaped = xml_escape(s);
            output.push_str(&format!("{}<{}>{}</{}>\n", pad, tag, escaped, tag));
        }
        serde_json::Value::Number(n) => {
            output.push_str(&format!("{}<{}>{}</{}>\n", pad, tag, n, tag));
        }
        serde_json::Value::Bool(b) => {
            output.push_str(&format!("{}<{}>{}</{}>\n", pad, tag, b, tag));
        }
        serde_json::Value::Null => {
            output.push_str(&format!("{}<{}/>\n", pad, tag));
        }
    }
}

/// Escape special XML characters
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Parse XML content to JSON value
fn parse_xml_to_json(
    content: &str,
    file_name: &str,
) -> Result<serde_json::Value, ConversionError> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut reader = Reader::from_str(content);
    let mut stack: Vec<(String, serde_json::Value)> = Vec::new();
    // Start with a root container
    stack.push(("$root".to_string(), serde_json::Value::Object(serde_json::Map::new())));

    let mut current_text = String::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                stack.push((tag_name, serde_json::Value::Object(serde_json::Map::new())));
                current_text.clear();
            }
            Ok(Event::End(_)) => {
                let (tag_name, mut element_value) = stack.pop().unwrap_or_default();
                // If the element has no children (is an empty object), use collected text
                if let serde_json::Value::Object(ref map) = element_value {
                    if map.is_empty() && !current_text.is_empty() {
                        let text = current_text.trim().to_string();
                        element_value = parse_xml_text_value(&text);
                        current_text.clear();
                    }
                }

                // Add this element to its parent
                if let Some((_parent_name, parent_value)) = stack.last_mut() {
                    if let serde_json::Value::Object(ref mut map) = parent_value {
                        if let Some(existing) = map.get_mut(&tag_name) {
                            // Convert to array if multiple same-named elements
                            match existing {
                                serde_json::Value::Array(arr) => {
                                    arr.push(element_value);
                                }
                                _ => {
                                    let prev = existing.clone();
                                    *existing =
                                        serde_json::Value::Array(vec![prev, element_value]);
                                }
                            }
                        } else {
                            map.insert(tag_name, element_value);
                        }
                    }
                }
                current_text.clear();
            }
            Ok(Event::Text(e)) => {
                current_text.push_str(&e.unescape().unwrap_or_default());
            }
            Ok(Event::CData(e)) => {
                current_text
                    .push_str(&String::from_utf8_lossy(e.as_ref()));
            }
            Ok(Event::Empty(e)) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if let Some((_parent_name, parent_value)) = stack.last_mut() {
                    if let serde_json::Value::Object(ref mut map) = parent_value {
                        if let Some(existing) = map.get_mut(&tag_name) {
                            match existing {
                                serde_json::Value::Array(arr) => {
                                    arr.push(serde_json::Value::Null);
                                }
                                _ => {
                                    let prev = existing.clone();
                                    *existing = serde_json::Value::Array(vec![
                                        prev,
                                        serde_json::Value::Null,
                                    ]);
                                }
                            }
                        } else {
                            map.insert(tag_name, serde_json::Value::Null);
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Ok(_) => {} // Skip comments, PIs, etc.
            Err(e) => {
                return Err(ConversionError::ParseError {
                    file_name: file_name.to_string(),
                    line: None,
                    detail: format!("XML parse error: {}", e),
                });
            }
        }
    }

    // The root element should be the only element in $root
    // Unwrap the $root container - return its single child value
    let (_root_name, root_value) = stack.pop().unwrap_or_default();
    if let serde_json::Value::Object(map) = &root_value {
        if map.len() == 1 {
            // Unwrap the single root element
            return Ok(map.values().next().unwrap().clone());
        }
    }
    Ok(root_value)
}

/// Try to parse XML text as number/bool/null, otherwise keep as string
fn parse_xml_text_value(text: &str) -> serde_json::Value {
    if text == "true" {
        return serde_json::Value::Bool(true);
    }
    if text == "false" {
        return serde_json::Value::Bool(false);
    }
    if text == "null" {
        return serde_json::Value::Null;
    }
    if let Ok(n) = text.parse::<i64>() {
        return serde_json::Value::Number(n.into());
    }
    if let Ok(n) = text.parse::<f64>() {
        if let Some(num) = serde_json::Number::from_f64(n) {
            return serde_json::Value::Number(num);
        }
    }
    serde_json::Value::String(text.to_string())
}

/// Parse CSV content to JSON array of objects
fn parse_csv_to_json(
    content: &str,
    file_name: &str,
    options: &ConversionOptions,
) -> Result<serde_json::Value, ConversionError> {
    let delimiter = match options.csv_delimiter {
        Some(CsvDelimiter::Tab) => b'\t',
        Some(CsvDelimiter::Semicolon) => b';',
        _ => b',', // Default: Comma
    };

    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .has_headers(true)
        .from_reader(content.as_bytes());

    let headers = rdr
        .headers()
        .map_err(|e| ConversionError::ParseError {
            file_name: file_name.to_string(),
            line: Some(1),
            detail: format!("CSV header parse error: {}", e),
        })?
        .clone();

    let mut records: Vec<serde_json::Value> = Vec::new();
    for (idx, result) in rdr.records().enumerate() {
        let record = result.map_err(|e| ConversionError::ParseError {
            file_name: file_name.to_string(),
            line: Some((idx + 2) as u32), // +2 because line 1 is header
            detail: format!("CSV parse error: {}", e),
        })?;

        let mut obj = serde_json::Map::new();
        for (i, field) in record.iter().enumerate() {
            let key = headers
                .get(i)
                .unwrap_or(&format!("column_{}", i))
                .to_string();
            obj.insert(key, serde_json::Value::String(field.to_string()));
        }
        records.push(serde_json::Value::Object(obj));
    }

    Ok(serde_json::Value::Array(records))
}

/// Serialize JSON value to CSV string
/// Only supports flat array of objects (no nested structures)
fn serialize_to_csv(
    value: &serde_json::Value,
    file_name: &str,
    options: &ConversionOptions,
) -> Result<String, ConversionError> {
    let arr = match value {
        serde_json::Value::Array(arr) => arr,
        serde_json::Value::Object(_) => {
            // Wrap single object in array
            return serialize_to_csv(
                &serde_json::Value::Array(vec![value.clone()]),
                file_name,
                options,
            );
        }
        _ => {
            return Err(ConversionError::UnsupportedConversion {
                from: "structured data".to_string(),
                to: "CSV (データが配列またはオブジェクトではありません)".to_string(),
            });
        }
    };

    if arr.is_empty() {
        return Ok(String::new());
    }

    // Check for nested structures and collect headers
    let mut headers: Vec<String> = Vec::new();
    for item in arr {
        match item {
            serde_json::Value::Object(map) => {
                for (key, val) in map {
                    // Reject nested structures
                    if val.is_object() || val.is_array() {
                        return Err(ConversionError::UnsupportedConversion {
                            from: "structured data".to_string(),
                            to: "CSV (ネストされたオブジェクトや配列を含むデータはCSVに変換できません)".to_string(),
                        });
                    }
                    if !headers.contains(key) {
                        headers.push(key.clone());
                    }
                }
            }
            _ => {
                return Err(ConversionError::UnsupportedConversion {
                    from: "structured data".to_string(),
                    to: "CSV (配列内の要素がオブジェクトではありません)".to_string(),
                });
            }
        }
    }

    let delimiter = match options.csv_delimiter {
        Some(CsvDelimiter::Tab) => b'\t',
        Some(CsvDelimiter::Semicolon) => b';',
        _ => b',',
    };

    let mut wtr = csv::WriterBuilder::new()
        .delimiter(delimiter)
        .from_writer(Vec::new());

    // Write headers
    wtr.write_record(&headers)
        .map_err(|e| ConversionError::EncodeError {
            file_name: file_name.to_string(),
            detail: e.to_string(),
        })?;

    // Write rows
    for item in arr {
        if let serde_json::Value::Object(map) = item {
            let row: Vec<String> = headers
                .iter()
                .map(|h| match map.get(h) {
                    Some(serde_json::Value::String(s)) => s.clone(),
                    Some(serde_json::Value::Number(n)) => n.to_string(),
                    Some(serde_json::Value::Bool(b)) => b.to_string(),
                    Some(serde_json::Value::Null) => String::new(),
                    None => String::new(),
                    _ => String::new(),
                })
                .collect();
            wtr.write_record(&row)
                .map_err(|e| ConversionError::EncodeError {
                    file_name: file_name.to_string(),
                    detail: e.to_string(),
                })?;
        }
    }

    let bytes = wtr
        .into_inner()
        .map_err(|e| ConversionError::EncodeError {
            file_name: file_name.to_string(),
            detail: e.to_string(),
        })?;

    String::from_utf8(bytes).map_err(|e| ConversionError::EncodeError {
        file_name: file_name.to_string(),
        detail: e.to_string(),
    })
}

/// Convert Markdown to PlainText by stripping markdown syntax
fn markdown_to_plaintext(content: &str) -> String {
    let mut result = String::with_capacity(content.len());

    for line in content.lines() {
        let processed = strip_markdown_line(line);
        result.push_str(&processed);
        result.push('\n');
    }

    // Remove trailing newline if original didn't have one
    if !content.ends_with('\n') && result.ends_with('\n') {
        result.pop();
    }

    result
}

/// Strip markdown syntax from a single line
fn strip_markdown_line(line: &str) -> String {
    let mut s = line.to_string();

    // Strip headers (# ## ### etc.)
    if s.starts_with('#') {
        s = s.trim_start_matches('#').trim_start().to_string();
    }

    // Strip bold **text** and __text__
    s = strip_paired_markers(&s, "**");
    s = strip_paired_markers(&s, "__");

    // Strip italic *text* and _text_ (careful not to strip already-stripped bold)
    s = strip_single_markers(&s, '*');
    s = strip_single_markers(&s, '_');

    // Strip strikethrough ~~text~~
    s = strip_paired_markers(&s, "~~");

    // Strip inline code `text`
    s = strip_single_markers(&s, '`');

    // Strip links [text](url) -> text
    s = strip_links(&s);

    // Strip images ![alt](url) -> alt
    s = strip_images(&s);

    // Strip horizontal rules (---, ***, ___)
    let trimmed = s.trim();
    if (trimmed.chars().all(|c| c == '-' || c == ' ') && trimmed.contains("---"))
        || (trimmed.chars().all(|c| c == '*' || c == ' ') && trimmed.contains("***"))
        || (trimmed.chars().all(|c| c == '_' || c == ' ') && trimmed.contains("___"))
    {
        return String::new();
    }

    // Strip list markers (- item, * item, + item, 1. item)
    let list_stripped = s.trim_start();
    if list_stripped.starts_with("- ")
        || list_stripped.starts_with("* ")
        || list_stripped.starts_with("+ ")
    {
        let indent = s.len() - list_stripped.len();
        s = format!(
            "{}{}",
            " ".repeat(indent),
            &list_stripped[2..]
        );
    } else if let Some(rest) = strip_numbered_list(list_stripped) {
        let indent = s.len() - list_stripped.len();
        s = format!("{}{}", " ".repeat(indent), rest);
    }

    // Strip blockquote markers (> )
    if s.starts_with("> ") {
        s = s[2..].to_string();
    } else if s.starts_with('>') {
        s = s[1..].to_string();
    }

    s
}

/// Strip paired markers like ** or ~~
fn strip_paired_markers(s: &str, marker: &str) -> String {
    let mut result = s.to_string();
    while let Some(start) = result.find(marker) {
        if let Some(end) = result[start + marker.len()..].find(marker) {
            let end_pos = start + marker.len() + end;
            let inner = &result[start + marker.len()..end_pos];
            result = format!("{}{}{}", &result[..start], inner, &result[end_pos + marker.len()..]);
        } else {
            break;
        }
    }
    result
}

/// Strip single character markers like * or `
fn strip_single_markers(s: &str, marker: char) -> String {
    let marker_str = marker.to_string();
    let mut result = s.to_string();
    while let Some(start) = result.find(marker) {
        if let Some(end) = result[start + 1..].find(marker) {
            let end_pos = start + 1 + end;
            let inner = &result[start + 1..end_pos];
            result = format!("{}{}{}", &result[..start], inner, &result[end_pos + 1..]);
        } else {
            break;
        }
    }
    let _ = marker_str;
    result
}

/// Strip markdown links [text](url) -> text
fn strip_links(s: &str) -> String {
    let mut result = s.to_string();
    while let Some(bracket_start) = result.find('[') {
        if let Some(bracket_end) = result[bracket_start..].find("](") {
            let bracket_end_abs = bracket_start + bracket_end;
            if let Some(paren_end) = result[bracket_end_abs + 2..].find(')') {
                let paren_end_abs = bracket_end_abs + 2 + paren_end;
                let text = &result[bracket_start + 1..bracket_end_abs];
                result = format!(
                    "{}{}{}",
                    &result[..bracket_start],
                    text,
                    &result[paren_end_abs + 1..]
                );
            } else {
                break;
            }
        } else {
            break;
        }
    }
    result
}

/// Strip markdown images ![alt](url) -> alt
fn strip_images(s: &str) -> String {
    let mut result = s.to_string();
    while let Some(img_start) = result.find("![") {
        if let Some(bracket_end) = result[img_start + 2..].find("](") {
            let bracket_end_abs = img_start + 2 + bracket_end;
            if let Some(paren_end) = result[bracket_end_abs + 2..].find(')') {
                let paren_end_abs = bracket_end_abs + 2 + paren_end;
                let alt = &result[img_start + 2..bracket_end_abs];
                result = format!(
                    "{}{}{}",
                    &result[..img_start],
                    alt,
                    &result[paren_end_abs + 1..]
                );
            } else {
                break;
            }
        } else {
            break;
        }
    }
    result
}

/// Strip numbered list prefix (e.g., "1. item" -> "item")
fn strip_numbered_list(s: &str) -> Option<&str> {
    let bytes = s.as_bytes();
    let mut i = 0;
    // Skip digits
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    // Must have at least one digit followed by ". "
    if i > 0 && i < bytes.len() - 1 && bytes[i] == b'.' && bytes[i + 1] == b' ' {
        Some(&s[i + 2..])
    } else {
        None
    }
}

/// Convert PlainText to Markdown
/// Paragraphs are separated by blank lines
fn plaintext_to_markdown(content: &str) -> String {
    // Plain text is already valid markdown content.
    // We just return as-is since the text already has paragraph separation.
    content.to_string()
}

/// Extract line number from serde_yaml error (best effort)
fn extract_yaml_line(e: &serde_yaml::Error) -> Option<u32> {
    e.location().map(|loc| loc.line() as u32)
}

/// Extract line number from toml error (best effort)
fn extract_toml_line(e: &toml::de::Error) -> Option<u32> {
    e.span().map(|span| {
        // span gives byte range; estimate line from the start
        span.start as u32
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_formats_count() {
        let converter = TextConverter::new();
        assert_eq!(converter.supported_input_formats().len(), 7);
        assert_eq!(converter.supported_output_formats().len(), 7);
    }

    #[test]
    fn test_json_yaml_bidirectional() {
        let converter = TextConverter::new();
        assert!(converter.can_convert(&FileFormat::Json, &FileFormat::Yaml));
        assert!(converter.can_convert(&FileFormat::Yaml, &FileFormat::Json));
    }

    #[test]
    fn test_json_toml_bidirectional() {
        let converter = TextConverter::new();
        assert!(converter.can_convert(&FileFormat::Json, &FileFormat::Toml));
        assert!(converter.can_convert(&FileFormat::Toml, &FileFormat::Json));
    }

    #[test]
    fn test_json_xml_bidirectional() {
        let converter = TextConverter::new();
        assert!(converter.can_convert(&FileFormat::Json, &FileFormat::Xml));
        assert!(converter.can_convert(&FileFormat::Xml, &FileFormat::Json));
    }

    #[test]
    fn test_csv_to_json_only() {
        let converter = TextConverter::new();
        assert!(converter.can_convert(&FileFormat::Csv, &FileFormat::Json));
        assert!(!converter.can_convert(&FileFormat::Csv, &FileFormat::Yaml));
    }

    #[test]
    fn test_structured_to_csv_flat_only() {
        let converter = TextConverter::new();
        assert!(converter.can_convert(&FileFormat::Json, &FileFormat::Csv));
        assert!(converter.can_convert(&FileFormat::Yaml, &FileFormat::Csv));
        assert!(converter.can_convert(&FileFormat::Toml, &FileFormat::Csv));
        assert!(converter.can_convert(&FileFormat::Xml, &FileFormat::Csv));
    }

    #[test]
    fn test_markdown_plaintext_bidirectional() {
        let converter = TextConverter::new();
        assert!(converter.can_convert(&FileFormat::Markdown, &FileFormat::PlainText));
        assert!(converter.can_convert(&FileFormat::PlainText, &FileFormat::Markdown));
    }

    #[test]
    fn test_unsupported_pairs() {
        let converter = TextConverter::new();
        assert!(!converter.can_convert(&FileFormat::Markdown, &FileFormat::Json));
        assert!(!converter.can_convert(&FileFormat::PlainText, &FileFormat::Yaml));
        assert!(!converter.can_convert(&FileFormat::Csv, &FileFormat::Markdown));
    }

    #[test]
    fn test_cannot_convert_same_format() {
        let converter = TextConverter::new();
        assert!(!converter.can_convert(&FileFormat::Json, &FileFormat::Json));
        assert!(!converter.can_convert(&FileFormat::Yaml, &FileFormat::Yaml));
        assert!(!converter.can_convert(&FileFormat::Csv, &FileFormat::Csv));
    }

    #[test]
    fn test_detect_format_from_path() {
        assert_eq!(detect_format_from_path(Path::new("f.json")), Some(FileFormat::Json));
        assert_eq!(detect_format_from_path(Path::new("f.yaml")), Some(FileFormat::Yaml));
        assert_eq!(detect_format_from_path(Path::new("f.yml")), Some(FileFormat::Yaml));
        assert_eq!(detect_format_from_path(Path::new("f.toml")), Some(FileFormat::Toml));
        assert_eq!(detect_format_from_path(Path::new("f.xml")), Some(FileFormat::Xml));
        assert_eq!(detect_format_from_path(Path::new("f.csv")), Some(FileFormat::Csv));
        assert_eq!(detect_format_from_path(Path::new("f.md")), Some(FileFormat::Markdown));
        assert_eq!(detect_format_from_path(Path::new("f.txt")), Some(FileFormat::PlainText));
        assert_eq!(detect_format_from_path(Path::new("f.png")), None);
    }

    #[test]
    fn test_parse_json_to_value() {
        let json = r#"{"name": "test", "value": 42}"#;
        let opts = default_options();
        let val = parse_to_json_value(json, &FileFormat::Json, "test.json", &opts).unwrap();
        assert_eq!(val["name"], "test");
        assert_eq!(val["value"], 42);
    }

    #[test]
    fn test_parse_yaml_to_value() {
        let yaml = "name: test\nvalue: 42\n";
        let opts = default_options();
        let val = parse_to_json_value(yaml, &FileFormat::Yaml, "test.yaml", &opts).unwrap();
        assert_eq!(val["name"], "test");
        assert_eq!(val["value"], 42);
    }

    #[test]
    fn test_parse_toml_to_value() {
        let toml_str = "name = \"test\"\nvalue = 42\n";
        let opts = default_options();
        let val = parse_to_json_value(toml_str, &FileFormat::Toml, "test.toml", &opts).unwrap();
        assert_eq!(val["name"], "test");
        assert_eq!(val["value"], 42);
    }

    #[test]
    fn test_parse_csv_to_json_array() {
        let csv = "name,age\nAlice,30\nBob,25\n";
        let opts = default_options();
        let val = parse_to_json_value(csv, &FileFormat::Csv, "test.csv", &opts).unwrap();
        let arr = val.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["name"], "Alice");
        assert_eq!(arr[0]["age"], "30");
        assert_eq!(arr[1]["name"], "Bob");
    }

    #[test]
    fn test_csv_tab_delimiter() {
        let csv = "name\tage\nAlice\t30\n";
        let mut opts = default_options();
        opts.csv_delimiter = Some(CsvDelimiter::Tab);
        let val = parse_to_json_value(csv, &FileFormat::Csv, "test.csv", &opts).unwrap();
        let arr = val.as_array().unwrap();
        assert_eq!(arr[0]["name"], "Alice");
        assert_eq!(arr[0]["age"], "30");
    }

    #[test]
    fn test_nested_structure_csv_rejection() {
        let json_val = serde_json::json!([
            {"name": "Alice", "address": {"city": "Tokyo"}}
        ]);
        let opts = default_options();
        let result = serialize_to_csv(&json_val, "test.json", &opts);
        assert!(result.is_err());
        if let Err(ConversionError::UnsupportedConversion { to, .. }) = result {
            assert!(to.contains("ネスト"));
        }
    }

    #[test]
    fn test_flat_structure_csv_success() {
        let json_val = serde_json::json!([
            {"name": "Alice", "age": 30},
            {"name": "Bob", "age": 25}
        ]);
        let opts = default_options();
        let result = serialize_to_csv(&json_val, "test.json", &opts).unwrap();
        assert!(result.contains("name"));
        assert!(result.contains("Alice"));
        assert!(result.contains("Bob"));
    }

    #[test]
    fn test_toml_top_level_array_rejection() {
        let json_val = serde_json::json!([1, 2, 3]);
        let result = serialize_to_toml(&json_val, "test.json");
        assert!(result.is_err());
    }

    #[test]
    fn test_markdown_to_plaintext_headers() {
        let md = "# Hello\n## World\n### Test";
        let result = markdown_to_plaintext(md);
        assert_eq!(result, "Hello\nWorld\nTest");
    }

    #[test]
    fn test_markdown_to_plaintext_bold_italic() {
        let md = "This is **bold** and *italic* text";
        let result = markdown_to_plaintext(md);
        assert_eq!(result, "This is bold and italic text");
    }

    #[test]
    fn test_markdown_to_plaintext_links() {
        let md = "Click [here](https://example.com) for more";
        let result = markdown_to_plaintext(md);
        assert_eq!(result, "Click here for more");
    }

    #[test]
    fn test_plaintext_to_markdown() {
        let text = "Hello world\n\nNew paragraph";
        let result = plaintext_to_markdown(text);
        assert_eq!(result, "Hello world\n\nNew paragraph");
    }

    #[test]
    fn test_json_to_yaml_roundtrip() {
        let json = r#"{"name": "test", "values": [1, 2, 3], "nested": {"a": true}}"#;
        let opts = default_options();
        let val = parse_to_json_value(json, &FileFormat::Json, "f.json", &opts).unwrap();
        let yaml_str = serialize_from_json_value(&val, &FileFormat::Yaml, "f.yaml", &opts).unwrap();
        let val2 = parse_to_json_value(&yaml_str, &FileFormat::Yaml, "f.yaml", &opts).unwrap();
        assert_eq!(val, val2);
    }

    #[test]
    fn test_json_to_toml_roundtrip() {
        let json = r#"{"name": "test", "value": 42, "flag": true}"#;
        let opts = default_options();
        let val = parse_to_json_value(json, &FileFormat::Json, "f.json", &opts).unwrap();
        let toml_str = serialize_from_json_value(&val, &FileFormat::Toml, "f.toml", &opts).unwrap();
        let val2 = parse_to_json_value(&toml_str, &FileFormat::Toml, "f.toml", &opts).unwrap();
        assert_eq!(val, val2);
    }

    #[tokio::test]
    async fn test_convert_json_to_yaml_file() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("input.json");
        let output = dir.path().join("output.yaml");
        fs::write(&input, r#"{"hello": "world"}"#).unwrap();

        let converter = TextConverter::new();
        let opts = default_options();
        converter.convert(&input, &output, &opts).await.unwrap();

        let result = fs::read_to_string(&output).unwrap();
        assert!(result.contains("hello"));
        assert!(result.contains("world"));
    }

    #[tokio::test]
    async fn test_file_size_limit_small_file_succeeds() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("small.json");
        fs::write(&input, r#"{"small": true}"#).unwrap();
        let output = dir.path().join("output.yaml");

        let converter = TextConverter::new();
        let opts = default_options();
        // Should succeed since file is small
        let result = converter.convert(&input, &output, &opts).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_file_size_limit_exceeds_50mb() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("huge.json");
        // Create a file just over 50MB by writing repeated content
        // 50MB = 52_428_800 bytes, write 52_428_801 bytes
        let chunk = "a".repeat(1024 * 1024); // 1MB chunk
        let mut file = std::fs::File::create(&input).unwrap();
        use std::io::Write;
        for _ in 0..50 {
            file.write_all(chunk.as_bytes()).unwrap();
        }
        // Write 1 more byte to exceed exactly 50MB
        file.write_all(b"x").unwrap();
        drop(file);

        let output = dir.path().join("output.yaml");
        let converter = TextConverter::new();
        let opts = default_options();
        let result = converter.convert(&input, &output, &opts).await;
        assert!(result.is_err());
        if let Err(ConversionError::FileSizeLimitError { file_name, size_mb, limit_mb }) = result {
            assert_eq!(file_name, "huge.json");
            assert_eq!(limit_mb, 50);
            assert!(size_mb >= 50);
        } else {
            panic!("Expected FileSizeLimitError, got {:?}", result);
        }
    }

    #[tokio::test]
    async fn test_file_size_limit_exactly_50mb_succeeds() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("exact50.json");
        // Create a file of exactly 50MB (50 * 1024 * 1024 bytes)
        let chunk = "a".repeat(1024 * 1024); // 1MB chunk
        let mut file = std::fs::File::create(&input).unwrap();
        use std::io::Write;
        for _ in 0..50 {
            file.write_all(chunk.as_bytes()).unwrap();
        }
        drop(file);

        let output = dir.path().join("output.yaml");
        let converter = TextConverter::new();
        let opts = default_options();
        // Exactly 50MB should NOT exceed the limit (condition is > not >=)
        let result = converter.convert(&input, &output, &opts).await;
        // This will fail on parse (content is 'aaa...') but should NOT fail on size check
        assert!(!matches!(result, Err(ConversionError::FileSizeLimitError { .. })));
    }

    #[test]
    fn test_xml_parse_simple() {
        let xml = "<root><name>test</name><value>42</value></root>";
        let val = parse_xml_to_json(xml, "test.xml").unwrap();
        assert_eq!(val["name"], "test");
        assert_eq!(val["value"], 42);
    }

    #[test]
    fn test_json_to_xml_serialize() {
        let val = serde_json::json!({"name": "test", "value": 42});
        let xml = serialize_to_xml(&val, "test.xml").unwrap();
        assert!(xml.contains("<name>test</name>"));
        assert!(xml.contains("<value>42</value>"));
        assert!(xml.starts_with("<?xml"));
    }

    /// Helper: default ConversionOptions for testing
    fn default_options() -> ConversionOptions {
        use crate::models::{ConflictResolution, FileNamingPattern};
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
    // 1. Conversion pair roundtrip/basic tests (file-based via Converter trait)
    // =========================================================================

    #[tokio::test]
    async fn test_convert_json_to_yaml_roundtrip_file() {
        let dir = tempfile::tempdir().unwrap();
        let json_input = dir.path().join("data.json");
        let yaml_output = dir.path().join("data.yaml");
        let json_roundtrip = dir.path().join("data_roundtrip.json");

        let original = r#"{"name": "test", "count": 5, "active": true}"#;
        fs::write(&json_input, original).unwrap();

        let converter = TextConverter::new();
        let opts = default_options();

        // JSON → YAML
        converter.convert(&json_input, &yaml_output, &opts).await.unwrap();
        let yaml_content = fs::read_to_string(&yaml_output).unwrap();
        assert!(yaml_content.contains("name"));
        assert!(yaml_content.contains("test"));

        // YAML → JSON (roundtrip)
        converter.convert(&yaml_output, &json_roundtrip, &opts).await.unwrap();
        let roundtrip_content = fs::read_to_string(&json_roundtrip).unwrap();
        let original_val: serde_json::Value = serde_json::from_str(original).unwrap();
        let roundtrip_val: serde_json::Value = serde_json::from_str(&roundtrip_content).unwrap();
        assert_eq!(original_val, roundtrip_val);
    }

    #[tokio::test]
    async fn test_convert_json_to_toml_roundtrip_file() {
        let dir = tempfile::tempdir().unwrap();
        let json_input = dir.path().join("data.json");
        let toml_output = dir.path().join("data.toml");
        let json_roundtrip = dir.path().join("data_rt.json");

        let original = r#"{"server": "localhost", "port": 8080, "debug": false}"#;
        fs::write(&json_input, original).unwrap();

        let converter = TextConverter::new();
        let opts = default_options();

        // JSON → TOML
        converter.convert(&json_input, &toml_output, &opts).await.unwrap();
        let toml_content = fs::read_to_string(&toml_output).unwrap();
        assert!(toml_content.contains("server"));
        assert!(toml_content.contains("localhost"));

        // TOML → JSON (roundtrip)
        converter.convert(&toml_output, &json_roundtrip, &opts).await.unwrap();
        let roundtrip_content = fs::read_to_string(&json_roundtrip).unwrap();
        let original_val: serde_json::Value = serde_json::from_str(original).unwrap();
        let roundtrip_val: serde_json::Value = serde_json::from_str(&roundtrip_content).unwrap();
        assert_eq!(original_val, roundtrip_val);
    }

    #[tokio::test]
    async fn test_convert_json_to_xml_file() {
        let dir = tempfile::tempdir().unwrap();
        let json_input = dir.path().join("data.json");
        let xml_output = dir.path().join("data.xml");

        fs::write(&json_input, r#"{"city": "Tokyo", "population": 14000000}"#).unwrap();

        let converter = TextConverter::new();
        let opts = default_options();

        converter.convert(&json_input, &xml_output, &opts).await.unwrap();
        let xml_content = fs::read_to_string(&xml_output).unwrap();
        assert!(xml_content.contains("<?xml"));
        assert!(xml_content.contains("<city>Tokyo</city>"));
        assert!(xml_content.contains("<population>14000000</population>"));
    }

    #[tokio::test]
    async fn test_convert_xml_to_json_file() {
        let dir = tempfile::tempdir().unwrap();
        let xml_input = dir.path().join("data.xml");
        let json_output = dir.path().join("data.json");

        fs::write(&xml_input, r#"<?xml version="1.0"?><root><lang>Rust</lang><version>1.77</version></root>"#).unwrap();

        let converter = TextConverter::new();
        let opts = default_options();

        converter.convert(&xml_input, &json_output, &opts).await.unwrap();
        let json_content = fs::read_to_string(&json_output).unwrap();
        let val: serde_json::Value = serde_json::from_str(&json_content).unwrap();
        assert_eq!(val["lang"], "Rust");
    }

    #[tokio::test]
    async fn test_convert_csv_to_json_file() {
        let dir = tempfile::tempdir().unwrap();
        let csv_input = dir.path().join("data.csv");
        let json_output = dir.path().join("data.json");

        fs::write(&csv_input, "id,name,score\n1,Alice,95\n2,Bob,87\n").unwrap();

        let converter = TextConverter::new();
        let opts = default_options();

        converter.convert(&csv_input, &json_output, &opts).await.unwrap();
        let json_content = fs::read_to_string(&json_output).unwrap();
        let val: serde_json::Value = serde_json::from_str(&json_content).unwrap();
        let arr = val.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["name"], "Alice");
        assert_eq!(arr[1]["name"], "Bob");
        assert_eq!(arr[0]["score"], "95");
    }

    #[tokio::test]
    async fn test_convert_csv_with_tab_delimiter_file() {
        let dir = tempfile::tempdir().unwrap();
        let csv_input = dir.path().join("data.csv");
        let json_output = dir.path().join("data.json");

        fs::write(&csv_input, "name\tcity\tage\nAlice\tTokyo\t30\nBob\tOsaka\t25\n").unwrap();

        let converter = TextConverter::new();
        let mut opts = default_options();
        opts.csv_delimiter = Some(CsvDelimiter::Tab);

        converter.convert(&csv_input, &json_output, &opts).await.unwrap();
        let json_content = fs::read_to_string(&json_output).unwrap();
        let val: serde_json::Value = serde_json::from_str(&json_content).unwrap();
        let arr = val.as_array().unwrap();
        assert_eq!(arr[0]["name"], "Alice");
        assert_eq!(arr[0]["city"], "Tokyo");
        assert_eq!(arr[1]["city"], "Osaka");
    }

    #[tokio::test]
    async fn test_convert_csv_with_semicolon_delimiter_file() {
        let dir = tempfile::tempdir().unwrap();
        let csv_input = dir.path().join("data.csv");
        let json_output = dir.path().join("data.json");

        fs::write(&csv_input, "product;price;qty\nApple;1.50;10\nBanana;0.80;20\n").unwrap();

        let converter = TextConverter::new();
        let mut opts = default_options();
        opts.csv_delimiter = Some(CsvDelimiter::Semicolon);

        converter.convert(&csv_input, &json_output, &opts).await.unwrap();
        let json_content = fs::read_to_string(&json_output).unwrap();
        let val: serde_json::Value = serde_json::from_str(&json_content).unwrap();
        let arr = val.as_array().unwrap();
        assert_eq!(arr[0]["product"], "Apple");
        assert_eq!(arr[0]["price"], "1.50");
        assert_eq!(arr[1]["qty"], "20");
    }

    #[tokio::test]
    async fn test_convert_json_to_csv_flat_file() {
        let dir = tempfile::tempdir().unwrap();
        let json_input = dir.path().join("data.json");
        let csv_output = dir.path().join("data.csv");

        fs::write(
            &json_input,
            r#"[{"name": "Alice", "age": 30}, {"name": "Bob", "age": 25}]"#,
        )
        .unwrap();

        let converter = TextConverter::new();
        let opts = default_options();

        converter.convert(&json_input, &csv_output, &opts).await.unwrap();
        let csv_content = fs::read_to_string(&csv_output).unwrap();
        assert!(csv_content.contains("name"));
        assert!(csv_content.contains("age"));
        assert!(csv_content.contains("Alice"));
        assert!(csv_content.contains("Bob"));
    }

    #[tokio::test]
    async fn test_convert_markdown_to_plaintext_file() {
        let dir = tempfile::tempdir().unwrap();
        let md_input = dir.path().join("doc.md");
        let txt_output = dir.path().join("doc.txt");

        fs::write(&md_input, "# Title\n\nThis is **bold** and [a link](http://example.com).\n").unwrap();

        let converter = TextConverter::new();
        let opts = default_options();

        converter.convert(&md_input, &txt_output, &opts).await.unwrap();
        let txt_content = fs::read_to_string(&txt_output).unwrap();
        assert!(txt_content.contains("Title"));
        assert!(txt_content.contains("bold"));
        assert!(txt_content.contains("a link"));
        assert!(!txt_content.contains("#"));
        assert!(!txt_content.contains("**"));
        assert!(!txt_content.contains("]("));
    }

    #[tokio::test]
    async fn test_convert_plaintext_to_markdown_file() {
        let dir = tempfile::tempdir().unwrap();
        let txt_input = dir.path().join("doc.txt");
        let md_output = dir.path().join("doc.md");

        let content = "Hello world\n\nThis is a paragraph.\n";
        fs::write(&txt_input, content).unwrap();

        let converter = TextConverter::new();
        let opts = default_options();

        converter.convert(&txt_input, &md_output, &opts).await.unwrap();
        let md_content = fs::read_to_string(&md_output).unwrap();
        assert_eq!(md_content, content);
    }

    // =========================================================================
    // 2. Parse error tests (verify error type and line number)
    // =========================================================================

    #[test]
    fn test_parse_error_invalid_json() {
        let invalid_json = "{invalid}";
        let opts = default_options();
        let result = parse_to_json_value(invalid_json, &FileFormat::Json, "bad.json", &opts);
        assert!(result.is_err());
        if let Err(ConversionError::ParseError { file_name, line, detail }) = result {
            assert_eq!(file_name, "bad.json");
            assert!(line.is_some()); // serde_json provides line numbers
            assert_eq!(line.unwrap(), 1);
            assert!(!detail.is_empty());
        } else {
            panic!("Expected ParseError, got {:?}", result);
        }
    }

    #[test]
    fn test_parse_error_invalid_json_multiline() {
        let invalid_json = "{\n  \"name\": \"test\",\n  \"value\": \n}";
        let opts = default_options();
        let result = parse_to_json_value(invalid_json, &FileFormat::Json, "multi.json", &opts);
        assert!(result.is_err());
        if let Err(ConversionError::ParseError { file_name, line, .. }) = result {
            assert_eq!(file_name, "multi.json");
            assert!(line.is_some());
            // Error should be on line 4 where the incomplete value is
            assert!(line.unwrap() >= 3);
        } else {
            panic!("Expected ParseError");
        }
    }

    #[test]
    fn test_parse_error_invalid_yaml() {
        let invalid_yaml = "key: [unclosed";
        let opts = default_options();
        let result = parse_to_json_value(invalid_yaml, &FileFormat::Yaml, "bad.yaml", &opts);
        assert!(result.is_err());
        if let Err(ConversionError::ParseError { file_name, detail, .. }) = result {
            assert_eq!(file_name, "bad.yaml");
            assert!(!detail.is_empty());
        } else {
            panic!("Expected ParseError, got {:?}", result);
        }
    }

    #[test]
    fn test_parse_error_invalid_toml() {
        let invalid_toml = "= no_key";
        let opts = default_options();
        let result = parse_to_json_value(invalid_toml, &FileFormat::Toml, "bad.toml", &opts);
        assert!(result.is_err());
        if let Err(ConversionError::ParseError { file_name, detail, .. }) = result {
            assert_eq!(file_name, "bad.toml");
            assert!(!detail.is_empty());
        } else {
            panic!("Expected ParseError, got {:?}", result);
        }
    }

    #[test]
    fn test_parse_error_invalid_xml() {
        let invalid_xml = "<unclosed>";
        let result = parse_xml_to_json(invalid_xml, "bad.xml");
        // quick-xml may or may not error on unclosed tags depending on version
        // But "<unclosed" without closing > would definitely fail
        let invalid_xml2 = "<root><broken";
        let result2 = parse_xml_to_json(invalid_xml2, "bad2.xml");
        assert!(result2.is_err());
        if let Err(ConversionError::ParseError { file_name, detail, .. }) = result2 {
            assert_eq!(file_name, "bad2.xml");
            assert!(!detail.is_empty());
        } else {
            panic!("Expected ParseError, got {:?}", result2);
        }
        // Also test with mismatched tags
        let invalid_xml3 = "<root><a>text</b></root>";
        let result3 = parse_xml_to_json(invalid_xml3, "mismatch.xml");
        assert!(result3.is_err());
        let _ = result; // suppress unused warning
    }

    #[tokio::test]
    async fn test_parse_error_via_convert_invalid_json_file() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("invalid.json");
        let output = dir.path().join("output.yaml");
        fs::write(&input, "{ not valid json }").unwrap();

        let converter = TextConverter::new();
        let opts = default_options();
        let result = converter.convert(&input, &output, &opts).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConversionError::ParseError { .. }));
    }

    // =========================================================================
    // 3. CSV delimiter option tests
    // =========================================================================

    #[test]
    fn test_csv_semicolon_delimiter_parse() {
        let csv = "name;age;city\nAlice;30;Tokyo\nBob;25;Osaka\n";
        let mut opts = default_options();
        opts.csv_delimiter = Some(CsvDelimiter::Semicolon);
        let val = parse_to_json_value(csv, &FileFormat::Csv, "test.csv", &opts).unwrap();
        let arr = val.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["name"], "Alice");
        assert_eq!(arr[0]["age"], "30");
        assert_eq!(arr[0]["city"], "Tokyo");
        assert_eq!(arr[1]["name"], "Bob");
    }

    #[test]
    fn test_csv_tab_delimiter_serialize() {
        let json_val = serde_json::json!([
            {"name": "Alice", "score": 95},
            {"name": "Bob", "score": 87}
        ]);
        let mut opts = default_options();
        opts.csv_delimiter = Some(CsvDelimiter::Tab);
        let result = serialize_to_csv(&json_val, "test.json", &opts).unwrap();
        // Tab-separated output should contain tabs
        assert!(result.contains('\t'));
        assert!(result.contains("Alice"));
        assert!(result.contains("Bob"));
        // Should NOT contain commas as separator (data values may contain commas but headers won't)
        let lines: Vec<&str> = result.lines().collect();
        assert!(lines[0].contains('\t')); // header uses tab
    }

    #[test]
    fn test_csv_semicolon_delimiter_serialize() {
        let json_val = serde_json::json!([
            {"fruit": "Apple", "color": "Red"},
            {"fruit": "Banana", "color": "Yellow"}
        ]);
        let mut opts = default_options();
        opts.csv_delimiter = Some(CsvDelimiter::Semicolon);
        let result = serialize_to_csv(&json_val, "test.json", &opts).unwrap();
        assert!(result.contains(';'));
        assert!(result.contains("Apple"));
        assert!(result.contains("Banana"));
        let lines: Vec<&str> = result.lines().collect();
        assert!(lines[0].contains(';'));
    }

    #[test]
    fn test_csv_default_comma_delimiter() {
        let csv = "a,b\n1,2\n";
        let opts = default_options(); // csv_delimiter is None → defaults to comma
        let val = parse_to_json_value(csv, &FileFormat::Csv, "test.csv", &opts).unwrap();
        let arr = val.as_array().unwrap();
        assert_eq!(arr[0]["a"], "1");
        assert_eq!(arr[0]["b"], "2");
    }

    // =========================================================================
    // 4. Nested structure CSV rejection tests
    // =========================================================================

    #[test]
    fn test_nested_object_csv_rejection() {
        let json_val = serde_json::json!([
            {"name": "Alice", "address": {"street": "Main St", "city": "Tokyo"}}
        ]);
        let opts = default_options();
        let result = serialize_to_csv(&json_val, "nested.json", &opts);
        assert!(result.is_err());
        if let Err(ConversionError::UnsupportedConversion { from, to }) = &result {
            assert_eq!(from, "structured data");
            assert!(to.contains("ネスト"));
        } else {
            panic!("Expected UnsupportedConversion error, got {:?}", result);
        }
    }

    #[test]
    fn test_nested_array_csv_rejection() {
        let json_val = serde_json::json!([
            {"name": "Alice", "scores": [95, 87, 92]}
        ]);
        let opts = default_options();
        let result = serialize_to_csv(&json_val, "nested_arr.json", &opts);
        assert!(result.is_err());
        if let Err(ConversionError::UnsupportedConversion { from, to }) = &result {
            assert_eq!(from, "structured data");
            assert!(to.contains("ネスト"));
        } else {
            panic!("Expected UnsupportedConversion error, got {:?}", result);
        }
    }

    #[tokio::test]
    async fn test_nested_json_to_csv_file_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let json_input = dir.path().join("nested.json");
        let csv_output = dir.path().join("output.csv");

        fs::write(
            &json_input,
            r#"[{"name": "Alice", "meta": {"role": "admin"}}]"#,
        )
        .unwrap();

        let converter = TextConverter::new();
        let opts = default_options();
        let result = converter.convert(&json_input, &csv_output, &opts).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConversionError::UnsupportedConversion { .. }
        ));
        // Output file should NOT exist since conversion failed
        assert!(!csv_output.exists());
    }

    // =========================================================================
    // 5. Additional conversion pair edge cases
    // =========================================================================

    #[test]
    fn test_json_yaml_roundtrip_with_nested_arrays() {
        let json = r#"{"data": [{"id": 1, "tags": ["a", "b"]}, {"id": 2, "tags": ["c"]}]}"#;
        let opts = default_options();
        let val = parse_to_json_value(json, &FileFormat::Json, "f.json", &opts).unwrap();
        let yaml_str = serialize_from_json_value(&val, &FileFormat::Yaml, "f.yaml", &opts).unwrap();
        let val2 = parse_to_json_value(&yaml_str, &FileFormat::Yaml, "f.yaml", &opts).unwrap();
        assert_eq!(val, val2);
    }

    #[test]
    fn test_json_yaml_roundtrip_preserves_types() {
        let json = r#"{"str": "hello", "num": 42, "float": 3.14, "bool": true, "null_val": null}"#;
        let opts = default_options();
        let val = parse_to_json_value(json, &FileFormat::Json, "f.json", &opts).unwrap();
        let yaml_str = serialize_from_json_value(&val, &FileFormat::Yaml, "f.yaml", &opts).unwrap();
        let val2 = parse_to_json_value(&yaml_str, &FileFormat::Yaml, "f.yaml", &opts).unwrap();
        assert_eq!(val["str"], val2["str"]);
        assert_eq!(val["num"], val2["num"]);
        assert_eq!(val["bool"], val2["bool"]);
        assert_eq!(val["null_val"], val2["null_val"]);
    }

    #[test]
    fn test_json_to_xml_to_json_basic() {
        let json = r#"{"name": "test", "value": 42}"#;
        let opts = default_options();
        let val = parse_to_json_value(json, &FileFormat::Json, "f.json", &opts).unwrap();
        let xml_str = serialize_from_json_value(&val, &FileFormat::Xml, "f.xml", &opts).unwrap();
        let val2 = parse_xml_to_json(&xml_str, "f.xml").unwrap();
        // XML roundtrip: numbers become parsed back correctly
        assert_eq!(val2["name"], "test");
        assert_eq!(val2["value"], 42);
    }

    #[test]
    fn test_csv_to_json_preserves_column_count() {
        let csv = "col1,col2,col3\na,b,c\nd,e,f\ng,h,i\n";
        let opts = default_options();
        let val = parse_to_json_value(csv, &FileFormat::Csv, "test.csv", &opts).unwrap();
        let arr = val.as_array().unwrap();
        assert_eq!(arr.len(), 3); // 3 data rows
        for item in arr {
            let obj = item.as_object().unwrap();
            assert_eq!(obj.len(), 3); // 3 columns per row
            assert!(obj.contains_key("col1"));
            assert!(obj.contains_key("col2"));
            assert!(obj.contains_key("col3"));
        }
    }
}
