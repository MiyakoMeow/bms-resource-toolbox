use std::borrow::Cow;

use encoding_rs::Encoding;

pub static BOFTT_ID_SPECIFIC_ENCODING_TABLE: &[(&str, &str)] = &[
    ("134", "utf-8"),
    ("191", "gbk"),
    ("435", "gbk"),
    ("439", "gbk"),
];

#[must_use] 
pub fn default_encodings() -> Vec<&'static Encoding> {
    vec![
        encoding_rs::SHIFT_JIS,
        encoding_rs::GBK,
        encoding_rs::UTF_8,
        encoding_rs::GB18030,
    ]
}

#[must_use] 
pub fn lookup_encoding(name: &str) -> Option<&'static Encoding> {
    Encoding::for_label(name.as_bytes())
}

pub enum ErrorHandling {
    Strict,
    Replace,
    Ignore,
}

pub struct PriorityDecoder {
    encodings: Vec<&'static Encoding>,
}

impl PriorityDecoder {
    #[must_use] 
    pub fn new(encodings: Vec<&'static Encoding>) -> Self {
        Self { encodings }
    }

    fn decode_byte_sequence(
        &self,
        bytes: &[u8],
        pos: usize,
    ) -> (Option<char>, usize) {
        let remaining = bytes.len() - pos;

        for enc in &self.encodings {
            let max_len = remaining.min(4);
            for len in 1..=max_len {
                let slice = &bytes[pos..pos + len];
                let (cow, had_replacements) =
                    enc.decode_without_bom_handling(slice);
                if had_replacements {
                    continue;
                }
                let s: String = match cow {
                    Cow::Owned(s) => s,
                    Cow::Borrowed(s) => s.to_owned(),
                };
                if s.contains('\u{FFFD}') {
                    continue;
                }
                let mut chars = s.chars();
                if let (Some(ch), None) = (chars.next(), chars.next()) {
                    return (Some(ch), len);
                }
            }
        }

        (None, 1)
    }

    pub fn decode(&self, bytes: &[u8], error_handling: ErrorHandling) -> Result<String, String> {
        let mut result = String::with_capacity(bytes.len());
        let mut pos = 0;
        let total = bytes.len();

        while pos < total {
            let (ch, consumed) = self.decode_byte_sequence(bytes, pos);

            match ch {
                Some(c) => result.push(c),
                None => match error_handling {
                    ErrorHandling::Strict => {
                        return Err(format!(
                            "Failed to decode byte at position {pos}: {:02X?}",
                            &bytes[pos..pos.min(total)]
                        ));
                    }
                    ErrorHandling::Replace => result.push('\u{FFFD}'),
                    ErrorHandling::Ignore => {}
                },
            }

            pos += consumed;
        }

        Ok(result)
    }
}

#[must_use] 
pub fn get_bms_file_str(bytes: &[u8], encoding: Option<&str>) -> String {
    let mut encodings = default_encodings();
    if let Some(enc) = encoding
        && let Some(e) = lookup_encoding(enc) {
            encodings.insert(0, e);
        }
    let decoder = PriorityDecoder::new(encodings);
    match decoder.decode(bytes, ErrorHandling::Strict) {
        Ok(s) => s,
        Err(_) => String::from_utf8_lossy(bytes).into_owned(),
    }
}
