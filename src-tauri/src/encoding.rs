use crate::errors::ConversionError;
use crate::models::TextEncoding;

/// 入力バイト列のエンコーディングを自動検出し、UTF-8 文字列にデコードする
pub fn detect_and_decode(bytes: &[u8]) -> Result<String, ConversionError> {
    // UTF-8 BOM チェック
    let data = if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        &bytes[3..]
    } else {
        bytes
    };

    // chardetng でエンコーディングを検出
    let mut detector = chardetng::EncodingDetector::new();
    detector.feed(data, true);
    let encoding = detector.guess(None, true);

    // encoding_rs でデコード
    let (decoded, _, had_errors) = encoding.decode(data);

    if had_errors {
        return Err(ConversionError::DecodeError {
            file_name: String::new(),
            detail: format!(
                "エンコーディング '{}' でのデコード中にエラーが発生しました",
                encoding.name()
            ),
        });
    }

    Ok(decoded.into_owned())
}

/// UTF-8 文字列を指定エンコーディングのバイト列に変換する
pub fn encode_string(content: &str, encoding: &TextEncoding) -> Result<Vec<u8>, ConversionError> {
    match encoding {
        TextEncoding::Utf8 => Ok(content.as_bytes().to_vec()),
        TextEncoding::ShiftJis => {
            let (encoded, _, had_errors) = encoding_rs::SHIFT_JIS.encode(content);
            if had_errors {
                return Err(ConversionError::EncodeError {
                    file_name: String::new(),
                    detail: "Shift_JIS へのエンコード中にエラーが発生しました（変換不能な文字が含まれています）".to_string(),
                });
            }
            Ok(encoded.into_owned())
        }
        TextEncoding::EucJp => {
            let (encoded, _, had_errors) = encoding_rs::EUC_JP.encode(content);
            if had_errors {
                return Err(ConversionError::EncodeError {
                    file_name: String::new(),
                    detail: "EUC-JP へのエンコード中にエラーが発生しました（変換不能な文字が含まれています）".to_string(),
                });
            }
            Ok(encoded.into_owned())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_and_decode_utf8() {
        let content = "Hello, 世界！";
        let bytes = content.as_bytes();
        let result = detect_and_decode(bytes).unwrap();
        assert_eq!(result, content);
    }

    #[test]
    fn test_detect_and_decode_utf8_with_bom() {
        let content = "Hello, 世界！";
        let mut bytes = vec![0xEF, 0xBB, 0xBF];
        bytes.extend_from_slice(content.as_bytes());
        let result = detect_and_decode(&bytes).unwrap();
        assert_eq!(result, content);
    }

    #[test]
    fn test_detect_and_decode_shift_jis() {
        // "こんにちは" in Shift_JIS
        let (encoded, _, _) = encoding_rs::SHIFT_JIS.encode("こんにちは");
        let result = detect_and_decode(&encoded).unwrap();
        assert_eq!(result, "こんにちは");
    }

    #[test]
    fn test_detect_and_decode_euc_jp() {
        // "こんにちは" in EUC-JP
        let (encoded, _, _) = encoding_rs::EUC_JP.encode("こんにちは");
        let result = detect_and_decode(&encoded).unwrap();
        assert_eq!(result, "こんにちは");
    }

    #[test]
    fn test_encode_string_utf8() {
        let content = "Hello, 世界！";
        let result = encode_string(content, &TextEncoding::Utf8).unwrap();
        assert_eq!(result, content.as_bytes());
    }

    #[test]
    fn test_encode_string_shift_jis() {
        let content = "こんにちは";
        let result = encode_string(content, &TextEncoding::ShiftJis).unwrap();
        // Verify by decoding back
        let (decoded, _, _) = encoding_rs::SHIFT_JIS.decode(&result);
        assert_eq!(decoded, content);
    }

    #[test]
    fn test_encode_string_euc_jp() {
        let content = "こんにちは";
        let result = encode_string(content, &TextEncoding::EucJp).unwrap();
        // Verify by decoding back
        let (decoded, _, _) = encoding_rs::EUC_JP.decode(&result);
        assert_eq!(decoded, content);
    }

    #[test]
    fn test_encode_string_roundtrip_shift_jis() {
        let content = "日本語テスト文字列";
        let encoded = encode_string(content, &TextEncoding::ShiftJis).unwrap();
        let decoded = detect_and_decode(&encoded).unwrap();
        assert_eq!(decoded, content);
    }

    #[test]
    fn test_encode_string_roundtrip_euc_jp() {
        let content = "日本語テスト文字列";
        let encoded = encode_string(content, &TextEncoding::EucJp).unwrap();
        let decoded = detect_and_decode(&encoded).unwrap();
        assert_eq!(decoded, content);
    }

    #[test]
    fn test_detect_and_decode_empty() {
        let result = detect_and_decode(&[]).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_encode_string_ascii_all_encodings() {
        let content = "Hello World 123";
        // ASCII is valid in all three encodings
        let utf8 = encode_string(content, &TextEncoding::Utf8).unwrap();
        let sjis = encode_string(content, &TextEncoding::ShiftJis).unwrap();
        let euc = encode_string(content, &TextEncoding::EucJp).unwrap();
        // ASCII bytes are the same across all encodings
        assert_eq!(utf8, sjis);
        assert_eq!(utf8, euc);
    }
}
