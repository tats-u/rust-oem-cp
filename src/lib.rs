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
        /// OEM SBCSs used in some languages (locales)
        static ref WINDOWS_USED_CODEPAGES: Vec<u16> = vec![
            437,
            // 720, // TODO: implement for locales using Arabic alphabets
            737,
            775,
            850,
            852,
            855,
            857,
            862,
            866,
            874,
        ];
        static ref WINDOWS_CONVERSION_VALID_TESTCASES: Vec<(u16, Vec<(u8, char)>)> = vec![
            (437, vec![(0x82, 'é'), (0x9D, '¥'), (0xFB, '√')]),
            (850, vec![(0xD0, 'ð'), (0xF3, '¾'), (0x9E, '×')]),
            (874, vec![(0x80, '€'), (0xDF, '฿'), (0xA1, 'ก')]),
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

    #[test]
    fn windows_codepages_coverage_test() {
        for cp in &*WINDOWS_USED_CODEPAGES {
            assert!(
                ENCODING_TABLE_CP_MAP.get(&cp).is_some(),
                "Encoding table for cp{} is not defined",
                cp
            );
            assert!(
                DECODING_TABLE_CP_MAP.get(&cp).is_some(),
                "Decoding table for cp{} is not defined",
                cp
            );
        }
    }

    /// Convert codepoint to Unicode via WindowsAPI
    ///
    /// # Arguments
    ///
    /// * `byte` - code point to convert to Unicode
    /// * `codepage` - code page
    #[cfg(windows)]
    fn windows_to_unicode_char(byte: u8, codepage: u16) -> Option<char> {
        let input_buf = [byte];
        let mut win_decode_buf: Vec<u16>;
        unsafe {
            use std::ptr::null_mut;
            use winapi::shared::winerror::ERROR_NO_UNICODE_TRANSLATION;
            use winapi::um::errhandlingapi::GetLastError;
            use winapi::um::stringapiset::MultiByteToWideChar;
            use winapi::um::winnls::MB_ERR_INVALID_CHARS;
            let win_decode_len = MultiByteToWideChar(
                codepage as u32,
                MB_ERR_INVALID_CHARS,
                input_buf.as_ptr() as *const i8,
                1,
                null_mut(),
                0,
            );
            if win_decode_len <= 0 {
                if GetLastError() == ERROR_NO_UNICODE_TRANSLATION {
                    return None;
                }
                panic!(
                    "MultiByteToWideChar (size checking) for 0x{:X} failed in cp{}",
                    byte, codepage
                );
            }
            win_decode_buf = vec![0; win_decode_len as usize];
            let win_decode_status = MultiByteToWideChar(
                codepage as u32,
                MB_ERR_INVALID_CHARS,
                input_buf.as_ptr() as *const i8,
                1,
                win_decode_buf.as_mut_ptr(),
                win_decode_len,
            );
            assert_eq!(
                win_decode_status, win_decode_len,
                "MultiByteToWideChar (writing) failed for 0x{:X} in cp{} (size checking returned {} / writing returned {})",
                byte,
                codepage,
                win_decode_len,
                win_decode_status
            );
        }
        let string_buf = String::from_utf16(&win_decode_buf).unwrap();
        if string_buf.chars().count() != 1 {
            return None;
        }
        return Some(string_buf.chars().next().unwrap());
    }

    #[cfg(windows)]
    #[test]
    fn windows_to_unicode_char_test() {
        lazy_static! {
            static ref WINDOWS_CONVERSION_INVALID_TESTCASES: Vec<(u16, Vec<u8>)> = vec![
                (857, vec![0xE7, 0xF2]),
                (874, vec![0xDB, 0xDC, 0xDD, 0xDE, 0xFC, 0xFD, 0xFE, 0xFF])
            ];
        }
        use itertools::join;
        for (codepage, testcases) in &*WINDOWS_CONVERSION_VALID_TESTCASES {
            let result = testcases
                .iter()
                .map(|(source, _)| windows_to_unicode_char(*source, *codepage))
                .collect::<Vec<Option<char>>>();
            assert!(
                testcases
                    .iter()
                    .zip(result.iter())
                    .all(|((_, target), converted)| converted
                        .map(|c| c == *target)
                        .unwrap_or(false)),
                "failed in cp{}:\n{}",
                codepage,
                join(
                    testcases
                        .iter()
                        .zip(result.iter())
                        .filter(|((_, target), converted)| converted
                            .map(|c| c != *target)
                            .unwrap_or(true))
                        .map(|((from, target), converted)| format!(
                            "0x{:X} => {:?} (target) / {:?} (Windows)",
                            from, target, converted
                        )),
                    ", "
                )
            );
        }
        for (codepage, testcases) in &*WINDOWS_CONVERSION_INVALID_TESTCASES {
            let result = testcases
                .iter()
                .map(|source| windows_to_unicode_char(*source, *codepage))
                .collect::<Vec<Option<char>>>();
            assert!(
                result.iter().all(|r| r.is_none()),
                "Some codepoints in cp{} weren't None: {}",
                codepage,
                join(
                    testcases
                        .iter()
                        .zip(result.iter())
                        .filter(|(_, r)| r.is_some())
                        .map(|(t, r)| format!("0x{:X} => {:?}", t, r.unwrap())),
                    ", "
                )
            );
        }
    }

    #[cfg(windows)]
    #[test]
    fn compare_to_winapi_decoding_test() {
        let windows_testing_codepages: Vec<(u16, Option<Vec<std::ops::Range<u8>>>)> = vec![
            (437, None),
            // (720, None),
            (737, None),
            (775, None),
            (850, None),
            (852, None),
            (855, None),
            (857, None),
            (862, None),
            (866, None),
            // CP437 is broken in Windows (0x81-0x84,0x86-0x90,0x98-9F are mapped to U+XX as are, but they must be undefined)
            (874, Some(vec![0x85..0x85, 0x91..0x97, 0xA0..0xFF])),
        ];
        use std::borrow::Cow;
        let default_range = Cow::from(vec![(128..255).collect::<Vec<u8>>()]);
        use itertools::join;
        for (codepage, testing_ranges) in &*windows_testing_codepages {
            let testing_ranges = testing_ranges
                .as_ref()
                .map(|v| {
                    Cow::from(
                        v.iter()
                            .map(|r| r.clone().collect::<Vec<u8>>())
                            .collect::<Vec<Vec<u8>>>(),
                    )
                })
                .unwrap_or(default_range.clone());
            for testing in testing_ranges.as_ref() {
                let msg = format!("Decoding table for cp{} is not defined", codepage);
                let library_result = DECODING_TABLE_CP_MAP
                    .get(codepage)
                    .expect(&*msg)
                    .decode_string_lossy(testing);
                let windows_result = testing
                    .iter()
                    .map(|codepoint| {
                        windows_to_unicode_char(*codepoint, *codepage)
                            .and_then(|ch| {
                                if 0xE000 <= ch as u32 && ch as u32 <= 0xF8FF {
                                    None
                                } else {
                                    Some(ch)
                                }
                            })
                            .unwrap_or('\u{FFFD}')
                    })
                    .collect::<String>();
                assert_eq!(
                    library_result,
                    windows_result,
                    "Different in cp{}:\n {}",
                    codepage,
                    join(
                        testing
                            .iter()
                            .zip(library_result.chars().zip(windows_result.chars()))
                            .filter(|(_, (l, w))| l != w)
                            .map(|(from, (lib, win))| format!(
                                "0x{:X} => {:?} (library) != {:?} (Windows)",
                                from, lib, win
                            )),
                        ", "
                    )
                );
            }
        }
    }
}
