use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt::Write;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::{env, fmt, io};

use serde::Deserialize;

enum Table {
    Complete([char; 128]),
    Incomplete([Option<char>; 128]),
}

/// Parsed code tables from `assets/code_tables.json`
struct CodeTables {
    /// The file creation time as a ISO 8601 Timestamp
    created: String,
    /// The code tables
    ///
    /// `(code_page, table)`
    tables: Vec<(u16, Table)>,
}

/// Generates `src/code_table.generated.rs` from `./assets/code_tables.json`
#[test]
fn generate_tables() -> Result<(), Box<dyn std::error::Error>> {
    let code_tables = parse_code_tables()?;
    let mut output = String::new();

    write_header(&mut output, code_tables.created)?;

    for (code_page, table) in &code_tables.tables {
        write_decoding(&mut output, *code_page, table)?;
    }

    for (code_page, table) in &code_tables.tables {
        write_encoding(&mut output, *code_page, table)?;
    }

    write_decoding_table_cp_map(&mut output, &code_tables.tables)?;
    write_encoding_table_cp_map(&mut output, &code_tables.tables)?;

    write_footer(&mut output)?;

    // NOTE: normalizes line endings to `\n` regardless of platform
    output = output.lines().collect::<Vec<_>>().join("\n");
    output.push('\n');

    snapshot_testing::assert_eq_or_update(output, Path::new("src").join("code_table.generated.rs"));

    Ok(())
}

/// Opens `assets/code_tables.json`, and organizes and returns its contents
fn parse_code_tables() -> io::Result<CodeTables> {
    let (path, patch_path) = {
        let mut path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        path.push("assets");
        let mut path2 = path.clone();
        path.push("code_tables.json");
        path2.push("code_tables_patch_win.json");
        (path, path2)
    };
    let file = BufReader::new(File::open(path)?);
    let patch_file = BufReader::new(File::open(patch_path)?);

    /// Raw data structure defined in `assets/code_tables.json`
    #[derive(Deserialize)]
    struct JsonCodeTables {
        created: String,
        tables: HashMap<String, Vec<Option<u32>>>,
    }

    let JsonCodeTables { created, tables } = serde_json::from_reader(file).unwrap();
    let raw_patch: HashMap<String, HashMap<String, u32>> =
        serde_json::from_reader(patch_file).unwrap();

    let patch: HashMap<String, HashMap<u8, u32>> = raw_patch
        .into_iter()
        .map(|(k, v)| {
            let table = v
                .into_iter()
                .map(|(k, v)| (k.parse().unwrap(), v))
                .collect::<HashMap<u8, u32>>();
            (k, table)
        })
        .collect::<HashMap<String, HashMap<u8, u32>>>();

    let mut tables = tables
        .into_iter()
        .map(|(code_page, table)| {
            // Apply patches
            let table = if let Some(patch_for_codepage) = patch.get(&code_page) {
                table
                    .into_iter()
                    .enumerate()
                    .map(|(i, c)| c.or_else(|| patch_for_codepage.get(&(i as u8)).copied()))
                    .collect()
            } else {
                table
            };
            // After here, `table` has been patched
            let complete = table.iter().all(Option::is_some);
            let code_page = code_page.parse().unwrap();
            let table = table
                .into_iter()
                .skip(128)
                .map(|i| i.map(|i| char::from_u32(i).unwrap()));
            let table = if complete {
                Table::Complete(
                    table
                        .map(Option::unwrap)
                        .collect::<Vec<_>>()
                        .try_into()
                        .unwrap(),
                )
            } else {
                Table::Incomplete(table.collect::<Vec<_>>().try_into().unwrap())
            };
            (code_page, table)
        })
        .collect::<Vec<_>>();

    tables.sort_unstable_by_key(|(code_page, _table)| *code_page);

    Ok(CodeTables { created, tables })
}

fn write_header(mut dst: impl Write, created: String) -> fmt::Result {
    writeln!(
        &mut dst,
        "/// Code table
/// Generated at {created}
pub mod code_table {{

use super::code_table_type::TableType;
use super::OEMCPHashMap;
use TableType::*;
"
    )
}

fn write_decoding(mut dst: impl Write, code_page: u16, table: &Table) -> fmt::Result {
    writeln!(&mut dst, "/// Decoding table (CP{code_page} to Unicode)")?;
    match table {
        Table::Complete(table) => {
            writeln!(
                &mut dst,
                "pub static DECODING_TABLE_CP{code_page}: [char; 128] = {table:?};"
            )?;
        }
        Table::Incomplete(table) => {
            writeln!(
                &mut dst,
                "pub static DECODING_TABLE_CP{code_page}: [Option<char>; 128] = {table:?};"
            )?;
        }
    }

    writeln!(&mut dst)?;

    Ok(())
}

fn write_encoding(mut dst: impl Write, code_page: u16, table: &Table) -> fmt::Result {
    let mut map = phf_codegen::Map::new();

    match table {
        Table::Complete(table) => {
            for (i, c) in table
                .iter()
                .copied()
                .enumerate()
                .map(|(i, c)| (i + 0x80, c))
            {
                map.entry(c, &i.to_string());
            }
        }
        Table::Incomplete(table) => {
            for (i, c) in table
                .iter()
                .copied()
                .enumerate()
                .filter_map(|(i, c)| c.map(|c| (i + 0x80, c)))
            {
                map.entry(c, &i.to_string());
            }
        }
    }

    write!(
        &mut dst,
        "/// Encoding table (Unicode to CP{code_page})
pub static ENCODING_TABLE_CP{code_page}: OEMCPHashMap<char, u8> = {map};",
        map = map.build()
    )?;

    Ok(())
}

fn write_decoding_table_cp_map(mut dst: impl Write, tables: &[(u16, Table)]) -> fmt::Result {
    let mut map = phf_codegen::Map::new();

    for (code_page, table) in tables {
        let ty = match table {
            Table::Complete(_) => "Complete",
            Table::Incomplete(_) => "Incomplete",
        };
        map.entry(code_page, &format!("{ty}(&DECODING_TABLE_CP{code_page})"));
    }

    writeln!(
        &mut dst,
        r#"/// map from codepage to decoding table
///
/// `.get` returns `code_table_type::{{Complete,Incomplete}}`.
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
/// use oem_cp::code_table::{{DECODING_TABLE_CP_MAP, DECODING_TABLE_CP437}};
/// use oem_cp::code_table_type::TableType::*;
/// assert_eq!(DECODING_TABLE_CP_MAP.get(&437).unwrap().decode_string_lossy(&[0x31, 0xF6, 0xAB, 0x3D, 0x32]), "1÷½=2".to_string());
/// if let Some(cp874_table) = DECODING_TABLE_CP_MAP.get(&874) {{
///     // means shrimp in Thai (U+E49 => 0xE9)
///     assert_eq!(cp874_table.decode_string_checked(&[0xA1, 0xD8, 0xE9, 0xA7]), Some("กุ้ง".to_string()));
///     // undefined mapping 0xDB for CP874 Windows dialect (strict mode with MB_ERR_INVALID_CHARS)
///     assert_eq!(cp874_table.decode_string_checked(&[0xDB]), None);
/// }} else {{
///     panic!("CP874 must be defined in DECODING_TABLE_CP_MAP");
/// }}
/// ```
pub static DECODING_TABLE_CP_MAP: OEMCPHashMap<u16, TableType> = {map};"#,
        map = map.build()
    )?;

    Ok(())
}

fn write_encoding_table_cp_map(mut dst: impl Write, tables: &[(u16, Table)]) -> fmt::Result {
    let mut map = phf_codegen::Map::new();

    for (code_page, _table) in tables {
        map.entry(*code_page, &format!("&ENCODING_TABLE_CP{code_page}"));
    }

    writeln!(
        &mut dst,
        r#"/// map from codepage to encoding table
///
/// # Examples
///
/// ```
/// # use std::ptr;
/// use oem_cp::code_table::{{ENCODING_TABLE_CP_MAP, ENCODING_TABLE_CP437}};
/// assert!(ptr::eq(*ENCODING_TABLE_CP_MAP.get(&437).unwrap(), &ENCODING_TABLE_CP437));
/// // CP932 (Shift-JIS; Japanese MBCS) is unsupported
/// assert!(ENCODING_TABLE_CP_MAP.get(&932).is_none());
///
/// use oem_cp::encode_string_checked;
///
/// if let Some(cp437_table) = ENCODING_TABLE_CP_MAP.get(&437) {{
///     assert_eq!(encode_string_checked("π≈22/7", cp437_table), Some(vec![0xE3, 0xF7, 0x32, 0x32, 0x2F, 0x37]));
/// }} else {{
///     panic!("CP437 must be registered in ENCODING_TABLE_CP_MAP");
/// }}
/// ```
pub static ENCODING_TABLE_CP_MAP: OEMCPHashMap<u16, &'static OEMCPHashMap<char, u8>> = {map};"#,
        map = map.build()
    )?;

    Ok(())
}

fn write_footer(mut dst: impl Write) -> fmt::Result {
    writeln!(&mut dst, "}}")
}
