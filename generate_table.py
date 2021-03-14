from typing import Optional, List, Dict
from io import TextIOBase
import re
import requests
from sys import stdout, argv
from pathlib import Path
import json


def convert_to_reverse_map(table: List[Optional[int]]) -> Dict[int, int]:
    ret = {}
    for cp_codepoint, u8_codepoint in enumerate(table):
        if u8_codepoint is None:
            continue
        ret[u8_codepoint] = cp_codepoint
    return ret


def print_header(timestamp: str, f: TextIOBase = stdout):
    print(
        f"""\
//! Code table
//! Generated at {timestamp}
use super::OEMCPHashMap;
use super::code_table_type::TableType;
use lazy_static::lazy_static;
use TableType::*;""",
        file=f,
    )


def print_codepage_table(
    codepage: int, table: List[Optional[int]], f: TextIOBase = stdout
):
    print(
        f"/// Decoding table (CP{codepage} to Unicode)\n"
        f"pub static DECODING_TABLE_CP{codepage}",
        file=f,
        end="",
    )
    if None in table:
        print(": [Option<char>; 128] = [", file=f)
        for codepoint in table[128:]:
            print(
                "    None,"
                if codepoint is None
                else f"    Some('\\u{{{codepoint:04X}}}'),",
                file=f,
            )
        print("];", file=f)
    else:
        count = 0
        print(": [char; 128] = [", file=f, end="")
        for codepoint in table[128:]:
            print("\n    " if count == 0 else " ", end="", file=f)
            print(f"'\\u{{{codepoint:04X}}}',", end="", file=f)
            if count == 7:
                count = 0
            else:
                count += 1
        print("\n];", file=f)


def print_reverse_map(reverse_map: Dict[int, Dict[int, int]], f: TextIOBase = stdout):
    print(
        """\
lazy_static! {""",
        file=f,
    )
    for codepage, m in reverse_map.items():
        print(
            f"    /// Encoding table (Unicode to CP{codepage})\n"
            f"    pub static ref ENCODING_TABLE_CP{codepage}"
            ": OEMCPHashMap<char, u8> = {\n"
            "        let mut m = OEMCPHashMap::new();",
            file=f,
        )
        for unicode, dest in m.items():
            if unicode == dest and unicode < 128:
                continue
            print(
                f"        m.insert('\\u{{{unicode:04X}}}', 0x{dest:02X});", file=f,
            )
        print(
            """\
        return m;
    };""",
            file=f,
        )
    print("}", file=f)


def print_codepage_table_map(
    table_map: Dict[int, List[Optional[int]]], f: TextIOBase = stdout
):
    print(
        """\
lazy_static! {
    /// map from codepage to decoding table
    ///
    /// `.get` returns `code_table_type::{Complete,Incomplete}`.
    ///
    /// * `Complete`: the decoding table doesn't have undefined mapping.
    /// * `Incomplete`:  it have some undefined mapping.
    ///
    /// This enumerate provides methods `decode_string_lossy` and `decode_string_checked`.
    /// The following examples show the use of them.  `if let Some(decoder) = *snip* decoder.decode_string_*snip*` is convenient for practical use.
    ///
    /// # Examples
    ///
    /// ```
    /// use oem_cp::code_table::{DECODING_TABLE_CP_MAP, DECODING_TABLE_CP437};
    /// use oem_cp::code_table_type::TableType::*;
    /// assert_eq!((*DECODING_TABLE_CP_MAP).get(&437).unwrap().decode_string_lossy(vec![0x31, 0xF6, 0xAB, 0x3D, 0x32]), "1÷½=2".to_string());
    /// if let Some(cp874_table) = (*DECODING_TABLE_CP_MAP).get(&874) {
    ///     // means shrimp in Thai (U+E49 => 0xE9)
    ///     assert_eq!(cp874_table.decode_string_checked(vec![0xA1, 0xD8, 0xE9, 0xA7]), Some("กุ้ง".to_string()));
    ///     // undefined mapping 0xDB for CP874 Windows dialect (strict mode with MB_ERR_INVALID_CHARS)
    ///     assert_eq!(cp874_table.decode_string_checked(vec![0xDB]), None);
    /// } else {
    ///     panic!("CP874 must be defined in DECODING_TABLE_CP_MAP");
    /// }
    /// ```
    pub static ref DECODING_TABLE_CP_MAP: OEMCPHashMap<u16, TableType> = {
        let mut map = OEMCPHashMap::new();""",
        file=f,
    )
    for (codepage, table) in table_map.items():
        print(
            f"        map.insert({codepage}, {'Incomplete' if None in table else 'Complete'}(&DECODING_TABLE_CP{codepage}));",
            file=f,
        )
    print(
        """\
        return map;
    };
}""",
        file=f,
    )


def print_codepage_reverse_map_table(
    reverse_map: Dict[int, Dict[int, int]], f: TextIOBase = stdout
):
    print(
        """\
lazy_static! {
    /// map from codepage to encoding table
    ///
    /// # Examples
    ///
    /// ```
    /// use oem_cp::code_table::{ENCODING_TABLE_CP_MAP, ENCODING_TABLE_CP437};
    /// assert_eq!((*ENCODING_TABLE_CP_MAP).get(&437), Some(&&*ENCODING_TABLE_CP437));
    /// // CP932 (Shift-JIS; Japanese MBCS) is unsupported
    /// assert_eq!((*ENCODING_TABLE_CP_MAP).get(&932), None);
    ///
    /// use oem_cp::encode_string_checked;
    ///
    /// if let Some(cp437_table) = (*ENCODING_TABLE_CP_MAP).get(&437) {
    ///     assert_eq!(encode_string_checked("π≈22/7", cp437_table), Some(vec![0xE3, 0xF7, 0x32, 0x32, 0x2F, 0x37]));
    /// } else {
    ///     panic!("CP437 must be registerd in ENCODING_TABLE_CP_MAP");
    /// }
    /// ```
    pub static ref ENCODING_TABLE_CP_MAP: OEMCPHashMap<u16, &'static OEMCPHashMap<char, u8>> = {
        let mut m = OEMCPHashMap::new();""",
        file=f,
    )
    for codepage, m in reverse_map.items():
        print(
            f"        m.insert({codepage}, &*ENCODING_TABLE_CP{codepage});", file=f,
        )
    print(
        """\
        return m;
    };
}""",
        file=f,
    )


if __name__ == "__main__":
    raw_json = {}
    with (Path(argv[0]).parent / "assets" / "code_tables.json").open(
        encoding="UTF-8", newline="\n"
    ) as f:
        raw_json = json.load(f)

    created_timestamp = raw_json["created"]
    table_map = {
        int(codepage_str): table for codepage_str, table in raw_json["tables"].items()
    }
    reverse_map = {
        codepage: convert_to_reverse_map(table_map[codepage])
        for codepage in table_map.keys()
    }

    with open("src/code_table.rs", "w", encoding="utf-8", newline="\n") as f:
        print_header(created_timestamp, f)
        for codepage in table_map.keys():
            table = table_map[codepage]
            print_codepage_table(codepage, table, f)
        print_reverse_map(reverse_map, f)
        print_codepage_table_map(table_map, f)
        print_codepage_reverse_map_table(reverse_map, f)
