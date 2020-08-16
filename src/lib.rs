pub mod code_table;
pub mod code_table_type;
use hashbrown::HashMap;
use std::borrow::Cow;
use std::convert::Into;

/// Decode SBCS (single byte character set) bytes (no undefined codepoints)
///
/// # Arguments
///
/// * `src` - bytes encoded in SBCS
/// * `decoding_table` - table for decoding SBCS (with**out** undefined codepoints)
///
/// # Examples
///
/// ```
/// use oem_cp::decode_string_complete_table;
/// use oem_cp::code_table::DECODING_TABLE_CP437;
///
/// assert_eq!(&decode_string_complete_table(vec![0xFB, 0xAC, 0x3D, 0xAB], &DECODING_TABLE_CP437), "√¼=½");
/// ```
pub fn decode_string_complete_table<'a, T: Into<Cow<'a, [u8]>>>(
    src: T,
    decoding_table: &[char; 128],
) -> String {
    src.into()
        .iter()
        .map(|byte| {
            if *byte < 128 {
                *byte as char
            } else {
                decoding_table[(*byte & 127) as usize]
            }
        })
        .collect()
}

/// Decode SBCS (single byte character set) bytes (with undefined codepoints)
///
/// If some undefined codepoints are found, returns `None`.
///
/// # Arguments
///
/// * `src` - bytes encoded in SBCS
/// * `decoding_table` - table for decoding SBCS (**with** undefined codepoints)
///
/// # Examples
///
/// ```
/// use oem_cp::decode_string_incomplete_table_checked;
/// use oem_cp::code_table::DECODING_TABLE_CP874;
///
/// // means shrimp in Thai (U+E49 => 0xE9)
/// assert_eq!(decode_string_incomplete_table_checked(vec![0xA1, 0xD8, 0xE9, 0xA7], &DECODING_TABLE_CP874), Some("กุ้ง".to_string()));
/// // 0x81-0x84,0x86-0x90,0x98-0x9F is invalid in CP874
/// assert_eq!(decode_string_incomplete_table_checked(vec![0x30, 0x81], &DECODING_TABLE_CP874), None);
/// ```
pub fn decode_string_incomplete_table_checked<'a, T: Into<Cow<'a, [u8]>>>(
    src: T,
    decoding_table: &[Option<char>; 128],
) -> Option<String> {
    let mut ret = String::new();
    for byte in src.into().iter() {
        ret.push(if *byte < 128 {
            *byte as char
        } else {
            decoding_table[(*byte & 127) as usize]?
        });
    }
    return Some(ret);
}

/// Decode SBCS (single byte character set) bytes (with undefined codepoints)
///
/// Undefined codepoints are replaced with `U+FFFD` (replacement character).
///
/// # Arguments
///
/// * `src` - bytes encoded in SBCS
/// * `decoding_table` - table for decoding SBCS (**with** undefined codepoints)
///
/// # Examples
///
/// ```
/// use oem_cp::decode_string_incomplete_table_lossy;
/// use oem_cp::code_table::DECODING_TABLE_CP874;
///
/// // means shrimp in Thai (U+E49 => 0xE9)
/// assert_eq!(&decode_string_incomplete_table_lossy(vec![0xA1, 0xD8, 0xE9, 0xA7], &DECODING_TABLE_CP874), "กุ้ง");
/// // 0x81-0x84,0x86-0x90,0x98-0x9F is invalid in CP874
/// assert_eq!(&decode_string_incomplete_table_lossy(vec![0x30, 0x81], &DECODING_TABLE_CP874), "0\u{FFFD}");
/// ```
pub fn decode_string_incomplete_table_lossy<'a, T: Into<Cow<'a, [u8]>>>(
    src: T,
    decoding_table: &[Option<char>; 128],
) -> String {
    src.into()
        .iter()
        .map(|byte| {
            if *byte < 128 {
                *byte as char
            } else {
                decoding_table[(*byte & 127) as usize].unwrap_or('\u{FFFD}')
            }
        })
        .collect()
}

/// Encode Unicode string in SBCS (single byte character set)
///
/// If some undefined codepoints are found, returns `None`.
///
/// # Arguments
///
/// * `src` - Unicode string
/// * `encoding_table` - table for encoding in SBCS
///
/// # Examples
///
/// ```
/// use oem_cp::encode_string_checked;
/// use oem_cp::code_table::{ENCODING_TABLE_CP437, ENCODING_TABLE_CP737};
/// assert_eq!(encode_string_checked("π≈22/7", &*ENCODING_TABLE_CP437), Some(vec![0xE3, 0xF7, 0x32, 0x32, 0x2F, 0x37]));
/// // Archimedes in Greek
/// assert_eq!(encode_string_checked("Αρχιμήδης", &*ENCODING_TABLE_CP737), Some(vec![0x80, 0xA8, 0xAE, 0xA0, 0xA3, 0xE3, 0x9B, 0x9E, 0xAA]));
/// // Japanese characters are not defined in CP437
/// assert_eq!(encode_string_checked("日本語ja_jp", &*ENCODING_TABLE_CP437), None);
/// ```
pub fn encode_string_checked<'a, T: Into<Cow<'a, str>>>(
    src: T,
    encoding_table: &HashMap<char, u8>,
) -> Option<Vec<u8>> {
    let mut ret = Vec::new();
    for c in src.into().chars() {
        ret.push(if (c as u32) < 128 {
            c as u8
        } else {
            *encoding_table.get(&c)?
        });
    }
    return Some(ret);
}

/// Encode Unicode string in SBCS (single byte character set)
///
/// Undefined codepoints are replaced with `0x3F` (`?`).
///
/// # Arguments
///
/// * `src` - Unicode string
/// * `encoding_table` - table for encoding in SBCS
///
/// # Examples
///
/// ```
/// use oem_cp::encode_string_lossy;
/// use oem_cp::code_table::{ENCODING_TABLE_CP437, ENCODING_TABLE_CP737};
/// assert_eq!(encode_string_lossy("π≈22/7", &*ENCODING_TABLE_CP437), vec![0xE3, 0xF7, 0x32, 0x32, 0x2F, 0x37]);
/// // Archimedes in Greek
/// assert_eq!(encode_string_lossy("Αρχιμήδης", &*ENCODING_TABLE_CP737), vec![0x80, 0xA8, 0xAE, 0xA0, 0xA3, 0xE3, 0x9B, 0x9E, 0xAA]);
/// // Japanese characters are not defined in CP437 and replaced with `?` (0x3F)
/// // "日本語ja_jp" => "???ja_jp"
/// assert_eq!(encode_string_lossy("日本語ja_jp", &*ENCODING_TABLE_CP437), vec![0x3F, 0x3F, 0x3F, 0x6A, 0x61, 0x5F, 0x6A, 0x70]);
/// ```
pub fn encode_string_lossy<'a, T: Into<Cow<'a, str>>>(
    src: T,
    encoding_table: &HashMap<char, u8>,
) -> Vec<u8> {
    src.into()
        .chars()
        .map(|c| {
            if (c as u32) < 128 {
                c as u8
            } else {
                encoding_table
                    .get(&c)
                    .map(|byte_ref| *byte_ref)
                    .unwrap_or('?' as u8)
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use code_table::*;
    use lazy_static::lazy_static;

    lazy_static! {
        static ref CP437_VALID_PAIRS: Vec<(&'static str, Vec<u8>)> = vec![
            ("√α²±ß²", vec![0xFB, 0xE0, 0xFD, 0xF1, 0xE1, 0xFD]),
            ("és", vec![0x82, 0x73]),
            ("più", vec![0x70, 0x69, 0x97]),
            ("½÷¼=2", vec![0xAB, 0xF6, 0xAC, 0x3D, 0x32])
        ];
        static ref CP874_VALID_PAIRS: Vec<(&'static str, Vec<u8>)> = vec![
            (
                "ราชอาณาจักรไท",
                vec![0xC3, 0xD2, 0xAA, 0xCD, 0xD2, 0xB3, 0xD2, 0xA8, 0xD1, 0xA1, 0xC3, 0xE4, 0xB7]
            ),
            (
                "ต้มยำกุ้ง",
                vec![0xB5, 0xE9, 0xC1, 0xC2, 0xD3, 0xA1, 0xD8, 0xE9, 0xA7]
            )
        ];
    }
    #[test]
    fn cp437_encoding_test() {
        for (utf8_ref, cp437_ref) in &*CP437_VALID_PAIRS {
            assert_eq!(
                &encode_string_lossy(*utf8_ref, &*ENCODING_TABLE_CP437),
                cp437_ref
            );
            assert_eq!(
                &(encode_string_checked(*utf8_ref, &*ENCODING_TABLE_CP437).unwrap()),
                cp437_ref
            );
        }
    }
    #[test]
    fn cp437_decoding_test() {
        for (utf8_ref, cp437_ref) in &*CP437_VALID_PAIRS {
            assert_eq!(
                &decode_string_complete_table(cp437_ref, &DECODING_TABLE_CP437),
                *utf8_ref
            );
        }
    }
    #[test]
    fn cp874_encoding_test() {
        for (utf8_ref, cp437_ref) in &*CP874_VALID_PAIRS {
            assert_eq!(
                &encode_string_lossy(*utf8_ref, &*ENCODING_TABLE_CP874),
                cp437_ref
            );
            assert_eq!(
                &(encode_string_checked(*utf8_ref, &*ENCODING_TABLE_CP874).unwrap()),
                cp437_ref
            );
        }
    }
    #[test]
    fn cp874_decoding_test() {
        for (utf8_ref, cp437_ref) in &*CP874_VALID_PAIRS {
            assert_eq!(
                &decode_string_incomplete_table_lossy(cp437_ref, &DECODING_TABLE_CP874),
                *utf8_ref
            );
            assert_eq!(
                &decode_string_incomplete_table_checked(cp437_ref, &DECODING_TABLE_CP874).unwrap(),
                *utf8_ref
            );
        }
    }
}
