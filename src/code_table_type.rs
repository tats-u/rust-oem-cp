use super::{
    decode_string_complete_table, decode_string_incomplete_table_checked,
    decode_string_incomplete_table_lossy,
};
use std::borrow::Cow;
use std::convert::Into;

/// Wrapper enumerate for decoding tables
///
/// It has 2 types: `Complete`, complete tables (it doesn't have undefined codepoints) / `Incomplete`, incomplete tables (does have ones)
pub enum TableType {
    /// complete table, which doen't have any undefined codepoints
    Complete(&'static [char; 128]),
    /// incomplete table, which has some undefined codepoints
    Incomplete(&'static [Option<char>; 128]),
}
use TableType::*;

impl TableType {
    /// Wrapper function for decoding bytes encoded in SBCSs
    ///
    /// This function returns `None` if any bytes bumps into undefined codepoints
    ///
    /// # Arguments
    ///
    /// * `src` - bytes encoded in SBCS
    ///
    /// # Examples
    ///
    /// ```
    /// use oem_cp::code_table::{DECODING_TABLE_CP437, DECODING_TABLE_CP874};
    /// use oem_cp::code_table_type::TableType;
    /// use TableType::{Complete,Incomplete};
    ///
    /// assert_eq!(Complete(&DECODING_TABLE_CP437).decode_string_checked(vec![0xFB, 0xAC, 0x3D, 0xAB]), Some("√¼=½".to_string()));
    /// // means shrimp in Thai (U+E49 => 0xE9)
    /// assert_eq!(Incomplete(&DECODING_TABLE_CP874).decode_string_checked(vec![0xA1, 0xD8, 0xE9, 0xA7]), Some("กุ้ง".to_string()));
    /// // 0x81-0x84,0x86-0x90,0x98-0x9F is invalid in CP874
    /// assert_eq!(Incomplete(&DECODING_TABLE_CP874).decode_string_checked(vec![0x30, 0x81]), None);
    /// ```
    pub fn decode_string_checked<'a, T: Into<Cow<'a, [u8]>>>(&self, src: T) -> Option<String> {
        match self {
            Complete(table_ref) => Some(decode_string_complete_table(src, table_ref)),
            Incomplete(table_ref) => decode_string_incomplete_table_checked(src, table_ref),
        }
    }
    /// Wrapper function for decoding bytes encoded in SBCSs
    ///
    /// Undefined codepoints are replaced with U+FFFD.
    ///
    /// # Arguments
    ///
    /// * `src` - bytes encoded in SBCS
    ///
    /// # Examples
    ///
    /// ```
    /// use oem_cp::code_table::{DECODING_TABLE_CP437, DECODING_TABLE_CP874};
    /// use oem_cp::code_table_type::TableType;
    /// use TableType::{Complete,Incomplete};
    ///
    /// assert_eq!(Complete(&DECODING_TABLE_CP437).decode_string_lossy(vec![0xFB, 0xAC, 0x3D, 0xAB]), "√¼=½".to_string());
    /// // means shrimp in Thai (U+E49 => 0xE9)
    /// assert_eq!(Incomplete(&DECODING_TABLE_CP874).decode_string_lossy(vec![0xA1, 0xD8, 0xE9, 0xA7]), "กุ้ง".to_string());
    /// // 0x81-0x84,0x86-0x90,0x98-0x9F is invalid in CP874
    /// assert_eq!(Incomplete(&DECODING_TABLE_CP874).decode_string_lossy(vec![0x30, 0x81]), "0\u{FFFD}".to_string());
    /// ```
    pub fn decode_string_lossy<'a, T: Into<Cow<'a, [u8]>>>(&self, src: T) -> String {
        match self {
            Complete(table_ref) => decode_string_complete_table(src, table_ref),
            Incomplete(table_ref) => decode_string_incomplete_table_lossy(src, table_ref),
        }
    }
}
