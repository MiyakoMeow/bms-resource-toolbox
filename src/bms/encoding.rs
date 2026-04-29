//! Encoding detection and priority decoding.
//!
//! This module provides multi-encoding detection for BMS files,
//! supporting Shift-JIS, GBK, UTF-8, and other encodings.

#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::items_after_statements
)]

use std::path::Path;

/// Encoding priority order (from Python ENCODINGS)
pub const ENCODINGS: [&str; 6] = [
    "shift-jis",
    "shift-jis-2004",
    "gb2312",
    "utf-8",
    "gb18030",
    "shift-jisx0213",
];

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
            }
        }
        Self { codecs }
    }

    /// Decode a single byte sequence using the encoding priority
    /// Returns (`decoded_char`, `bytes_consumed`) or (None, 1) if all fail
    fn decode_byte_sequence(&self, byte_data: &[u8], start: usize) -> (Option<char>, usize) {
        for encoding in &self.codecs {
            // Try lengths 1-4 (CJK encodings rarely exceed 4 bytes)
            for length in 1..=4 {
                if start + length > byte_data.len() {
                    break;
                }
                let (decoded, _, had_error) = encoding.decode(&byte_data[start..start + length]);
                if !had_error && !decoded.is_empty() {
                    let ch = decoded.chars().next();
                    if ch.is_some() {
                        return (ch, length);
                    }
                }
            }
        }
        // All encodings failed, consume 1 byte
        (None, 1)
    }

    /// Decode byte data using encoding priority
    pub fn decode(&self, byte_data: &[u8], errors: &str) -> Result<String, std::io::Error> {
        let mut result = Vec::new();
        let mut position = 0;

        while position < byte_data.len() {
            let (ch, consumed) = self.decode_byte_sequence(byte_data, position);

            match ch {
                Some(c) => result.push(c),
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

        Ok(result.into_iter().collect())
    }
}

/// Read file with encoding priority decoding
#[expect(dead_code)]
pub(crate) fn read_file_with_priority<P: AsRef<Path>>(
    file_path: P,
    encoding_priority: &[&str],
    errors: &str,
) -> Result<Option<String>, std::io::Error> {
    match std::fs::File::open(file_path.as_ref()) {
        Ok(mut file) => {
            let mut byte_data = Vec::new();
            use std::io::Read;
            file.read_to_end(&mut byte_data)?;
            let decoder = PriorityDecoder::new(encoding_priority);
            match decoder.decode(&byte_data, errors) {
                Ok(s) => Ok(Some(s)),
                Err(e) => {
                    eprintln!("Error: {e}");
                    Ok(None)
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {e}");
            Ok(None)
        }
    }
}

/// Get BMS file string with optional forced encoding
#[must_use]
pub fn get_bms_file_str(file_bytes: &[u8], encoding: Option<&str>) -> String {
    let mut encodings: Vec<&str> = ENCODINGS.to_vec();
    if let Some(enc) = encoding {
        encodings.insert(0, enc);
    }
    let decoder = PriorityDecoder::new(&encodings);
    match decoder.decode(file_bytes, "strict") {
        Ok(s) => s,
        Err(_) => {
            // Fallback to UTF-8 ignoring errors
            String::from_utf8_lossy(file_bytes).into_owned()
        }
    }
}

/// Read BMS file auto-detecting encoding
pub fn read_bms_file<P: AsRef<Path>>(path: P) -> Result<String, std::io::Error> {
    let path = path.as_ref();
    let mut file = std::fs::File::open(path)?;
    let len = file.metadata()?.len();
    let mut bytes = Vec::with_capacity(usize::try_from(len).unwrap_or(usize::MAX));
    use std::io::Read;
    file.read_to_end(&mut bytes)?;

    let content = get_bms_file_str(&bytes, None);
    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_decoder() {
        // Test Shift-JIS encoded "こんにちは"
        let test_bytes: &[u8] = &[0x82, 0xb1, 0x82, 0xf1, 0x82, 0xc9, 0x82, 0xbf, 0x82, 0xcd];
        let decoder = PriorityDecoder::new(&["shift-jis", "utf-8"]);
        let result = decoder.decode(test_bytes, "replace").unwrap();
        assert_eq!(result, "こんにちは");
    }
}
