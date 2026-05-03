//! Encoding detection and priority decoding.
//!
//! This module provides multi-encoding detection for BMS files,
//! supporting Shift-JIS, GBK, UTF-8, and other encodings.

use std::sync::{LazyLock, RwLock};

/// Encoding priority order (from Python ENCODINGS)
pub static ENCODINGS: LazyLock<RwLock<Vec<&'static str>>> = LazyLock::new(|| {
    RwLock::new(vec![
        "shift-jis",
        "shift-jis-2004",
        "gb2312",
        "utf-8",
        "gb18030",
        "shift-jisx0213",
    ])
});

/// BOFTT-specific encoding overrides
const BOFTT_ID_ENCODING: [(&str, &str); 4] = [
    ("134", "utf-8"),
    ("191", "gbk"),
    ("435", "gbk"),
    ("439", "gbk"),
];

/// Get BOFTT encoding for a given ID
#[must_use]
pub fn get_boftt_encoding(id: &str) -> Option<&'static str> {
    BOFTT_ID_ENCODING
        .iter()
        .find(|(k, _)| *k == id)
        .map(|(_, v)| *v)
}

/// `PriorityDecoder` attempts to decode byte sequences using multiple encodings
/// in priority order, trying 1-4 bytes at a time for each character
pub struct PriorityDecoder {
    codecs: Vec<&'static encoding_rs::Encoding>,
}

impl PriorityDecoder {
    /// Create a new `PriorityDecoder` with encoding priority list.
    #[must_use]
    pub fn new(encoding_priority: &[&str]) -> Self {
        let mut codecs: Vec<&'static encoding_rs::Encoding> = Vec::new();
        for enc in encoding_priority {
            if let Some(encoding) = encoding_rs::Encoding::for_label(enc.as_bytes()) {
                codecs.push(encoding);
            } else if let Some(fallback) = Self::fallback_encoding(enc) {
                codecs.push(fallback);
            }
        }
        Self { codecs }
    }

    fn fallback_encoding(label: &str) -> Option<&'static encoding_rs::Encoding> {
        match label {
            "shift-jis-2004" | "shift-jisx0213" => Some(encoding_rs::SHIFT_JIS),
            _ => None,
        }
    }

    /// Decode a single byte sequence using the encoding priority
    /// Returns (`decoded_char`, `bytes_consumed`) or (None, 1) if all fail
    fn decode_byte_sequence(&self, byte_data: &[u8], start: usize) -> (Option<String>, usize) {
        for encoding in &self.codecs {
            for length in 1..=4 {
                if start + length > byte_data.len() {
                    break;
                }
                let (decoded, _, had_error) = encoding.decode(&byte_data[start..start + length]);
                if !had_error && !decoded.is_empty() {
                    return (Some(decoded.into_owned()), length);
                }
            }
        }
        (None, 1)
    }

    /// Decode byte data using encoding priority
    ///
    /// # Errors
    ///
    /// Returns `std::io::Error` with `InvalidData` when `errors` is `"strict"`
    /// and a byte sequence cannot be decoded by any known encoding.
    pub fn decode(&self, byte_data: &[u8], errors: &str) -> Result<String, std::io::Error> {
        let mut result = String::new();
        let mut position = 0;

        while position < byte_data.len() {
            let (ch, consumed) = self.decode_byte_sequence(byte_data, position);

            match ch {
                Some(s) => result.push_str(&s),
                None => match errors {
                    "strict" => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!(
                                "Cannot decode byte sequence: {:02x?}",
                                &byte_data[position..=position]
                            ),
                        ));
                    }
                    "replace" => result.push('\u{FFFD}'),
                    _ => {}
                },
            }
            position += consumed;
        }

        Ok(result)
    }
}

/// Get BMS file string with optional forced encoding
///
/// # Panics
///
/// Panics if the global `ENCODINGS` `RwLock` is poisoned (i.e., another
/// thread panicked while holding the lock).
#[must_use]
#[allow(clippy::similar_names)]
pub fn get_bms_file_str(file_bytes: &[u8], encoding: Option<&str>) -> String {
    if let Some(enc) = encoding {
        let enc_static: &'static str = Box::leak(enc.to_string().into_boxed_str());
        {
            let mut list = ENCODINGS.write().unwrap();
            if let Some(pos) = list.iter().position(|e| *e == enc_static) {
                list.remove(pos);
            }
            list.insert(0, enc_static);
        }
    }
    let encodings = ENCODINGS.read().unwrap();
    let decoder = PriorityDecoder::new(&encodings);
    if let Ok(s) = decoder.decode(file_bytes, "strict") {
        s
    } else {
        let (decoded, _) = encoding_rs::UTF_8.decode_without_bom_handling(file_bytes);
        // Match Python errors="ignore" behavior: drop undecodable bytes instead
        // of inserting U+FFFD replacement characters
        let s = decoded.into_owned();
        if s.contains('\u{FFFD}') {
            s.replace('\u{FFFD}', "")
        } else {
            s
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[test]
    async fn test_priority_decoder() {
        // Test Shift-JIS encoded "こんにちは"
        let test_bytes: &[u8] = &[0x82, 0xb1, 0x82, 0xf1, 0x82, 0xc9, 0x82, 0xbf, 0x82, 0xcd];
        let decoder = PriorityDecoder::new(&["shift-jis", "utf-8"]);
        let result = decoder.decode(test_bytes, "replace").unwrap();
        assert_eq!(result, "こんにちは");
    }
}
