//! Parser for pre-compiled binary resource files referenced by
//! `#RESOURCE RES, "file.RES"` (standard Win32 resource compiler output) and
//! `#RESOURCE PBR, "file.PBR"` (legacy PowerBASIC PBRES format).
//!
//! Standard .RES layout (measured from rc.exe output and Microsoft's
//! "Resource File Formats" reference).  Each entry is:
//! ```text
//! DWORD DataSize        // size of the resource data
//! DWORD HeaderSize      // total header size from the Type field to the data
//! DWORD Type            // low word 0xFFFF -> integer type in high word;
//!                       // otherwise inline UTF-16 string + NUL WORD
//! DWORD Name            // same encoding as Type
//! DWORD DataVersion
//! WORD  MemoryFlags
//! WORD  LanguageId
//! DWORD Version
//! DWORD Characteristics
//! [HeaderSize-24 bytes of extra header]
//! [DataSize bytes of resource data]
//! ```
//! Entries are DWORD aligned.  A leading entry with DataSize 0 is an rc.exe
//! alignment placeholder and is skipped.

/// One resource entry decoded from a .RES/.PBR file, ready to be written with
/// the Win32 UpdateResource API.
#[derive(Debug, Clone)]
pub struct ParsedResEntry {
    /// Numeric RT_* type id when the type is an integer, else None.
    pub type_int: Option<u32>,
    /// Unicode type name when the type is a string, else None.
    pub type_str: Option<String>,
    /// Numeric resource id when the name is an integer, else None.
    pub name_int: Option<u32>,
    /// Unicode resource name when the name is a string, else None.
    pub name_str: Option<String>,
    /// Language id (e.g. 1033 = LANG_ENGLISH/SUBLANG_ENGLISH_US).
    pub lang: u16,
    /// Resource payload, byte-for-byte as compiled.
    pub data: Vec<u8>,
}

struct Cursor<'a> {
    b: &'a [u8],
}

impl<'a> Cursor<'a> {
    fn u32(&self, at: usize) -> u32 {
        u32::from_le_bytes([self.b[at], self.b[at + 1], self.b[at + 2], self.b[at + 3]])
    }
    fn u16(&self, at: usize) -> u16 {
        u16::from_le_bytes([self.b[at], self.b[at + 1]])
    }
    /// Read a Type/Name identifier: integer (0xFFFF marker) or inline UTF-16
    /// string terminated by a NUL WORD.  Returns (integer, string, new pos).
    fn read_id(&self, start: usize) -> (Option<u32>, Option<String>, usize) {
        let dw = self.u32(start);
        if (dw & 0xFFFF) == 0xFFFF {
            return (Some(dw >> 16), None, start + 4);
        }
        let mut p = start;
        while self.u16(p) != 0 {
            p += 2;
        }
        let words: Vec<u16> = self.b[start..p]
            .as_chunks::<2>()
            .0
            .iter()
            .map(|c| u16::from_le_bytes(*c))
            .collect();
        let s = String::from_utf16_lossy(&words);
        (None, Some(s), p + 2)
    }
}

/// Parse a legacy .PBR (PowerBASIC PBRES) binary.
///
/// Layout measured from a real PBRWin10-produced sample
/// (`SEDP_Lookup.pbr`, 3,340 bytes) and cross-checked against the resource
/// directory of the EXE that embedded it:
///
/// ```text
/// 0x00  WORD 0x0500, ASCII "CR"       // TrID signature 00 05 43 52
/// 0x04  DWORD entry_count
/// 0x08  DWORD data_entry_offset[entry_count]   // relative to BASE
///       ... zero padding ...
/// BASE  IMAGE_RESOURCE_DIRECTORY tree (the same 16-byte headers and
///       8-byte entries used by a PE .rsrc section)
///       each leaf is a 16-byte data entry:
///         DWORD DataRva (relative to BASE)
///         DWORD DataSize
///         DWORD Codepage
///         DWORD Reserved
/// ```
///
/// Unlike a PE resource section, every file offset is relative to `BASE`,
/// and the directory tree is stored in the file rather than at an RVA.  The
/// string form of a type/name is a WORD character count followed by that many
/// UTF-16LE code units (no NUL terminator).
pub fn parse_pbr(bytes: &[u8]) -> Result<Vec<ParsedResEntry>, String> {
    // Magic: WORD 0x0500 followed by ASCII "CR" (TrID signature
    // 00 05 43 52).  Reject anything that does not carry it.
    if bytes.len() < 8 || bytes[0] != 0x00 || bytes[1] != 0x05 || &bytes[2..4] != b"CR" {
        return Err("not a PowerBASIC PBR file (missing 00 05 43 52 signature)".to_string());
    }
    let count = pbr_u32(bytes, 4)? as usize;
    if count == 0 {
        return Err("PBR contains no resource entries".to_string());
    }
    if count > 1_000_000 || 8 + count * 4 > bytes.len() {
        return Err(format!("invalid PBR entry count {}", count));
    }
    let mut declared = Vec::with_capacity(count);
    for i in 0..count {
        declared.push(pbr_u32(bytes, 8 + i * 4)? as usize);
    }

    // The sample's header index ends exactly at 0x20.  Keep 0x20 as the
    // primary base and also try the naturally aligned end of the index so a
    // file with a different entry count is not rejected solely on layout.
    let aligned = (8 + count * 4 + 15) & !15;
    let mut bases = vec![0x20usize];
    if aligned != 0x20 {
        bases.push(aligned);
    }
    let mut last_err = String::new();
    for base in bases {
        match pbr_walk(bytes, base, count, &declared) {
            Ok(v) => return Ok(v),
            Err(e) => last_err = e,
        }
    }
    Err(format!(
        "cannot decode PBR resource directory: {}",
        last_err
    ))
}

fn pbr_u16(bytes: &[u8], at: usize) -> Result<u16, String> {
    if at + 2 > bytes.len() {
        return Err(format!("truncated PBR at offset {}", at));
    }
    Ok(u16::from_le_bytes([bytes[at], bytes[at + 1]]))
}

fn pbr_u32(bytes: &[u8], at: usize) -> Result<u32, String> {
    if at + 4 > bytes.len() {
        return Err(format!("truncated PBR at offset {}", at));
    }
    Ok(u32::from_le_bytes([
        bytes[at],
        bytes[at + 1],
        bytes[at + 2],
        bytes[at + 3],
    ]))
}

/// Read a length-prefixed UTF-16LE PBR string at `base + rel`.
fn pbr_string(bytes: &[u8], base: usize, rel: usize) -> Result<String, String> {
    let at = base
        .checked_add(rel)
        .ok_or_else(|| "PBR string offset overflow".to_string())?;
    let units = pbr_u16(bytes, at)? as usize;
    let start = at + 2;
    let end = start
        .checked_add(units * 2)
        .ok_or_else(|| "PBR string length overflow".to_string())?;
    if end > bytes.len() {
        return Err(format!("truncated PBR string at offset {}", at));
    }
    let words: Vec<u16> = bytes[start..end]
        .as_chunks::<2>()
        .0
        .iter()
        .map(|c| u16::from_le_bytes(*c))
        .collect();
    Ok(String::from_utf16_lossy(&words))
}

fn pbr_walk(
    bytes: &[u8],
    base: usize,
    count: usize,
    declared: &[usize],
) -> Result<Vec<ParsedResEntry>, String> {
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    pbr_walk_dir(
        bytes, base, 0, 0, None, None, None, None, &mut out, &mut seen,
    )?;
    if out.len() != count || seen.len() != count {
        return Err(format!(
            "directory yielded {} entries / {} data refs, header says {}",
            out.len(),
            seen.len(),
            count
        ));
    }
    for off in declared {
        if !seen.contains(off) {
            return Err(format!(
                "header data-entry offset {:#x} is not reachable from the directory tree",
                off
            ));
        }
    }
    Ok(out)
}

#[allow(clippy::too_many_arguments)]
fn pbr_walk_dir(
    bytes: &[u8],
    base: usize,
    rel: usize,
    depth: u8,
    type_int: Option<u32>,
    type_str: Option<String>,
    name_int: Option<u32>,
    name_str: Option<String>,
    out: &mut Vec<ParsedResEntry>,
    seen: &mut std::collections::HashSet<usize>,
) -> Result<(), String> {
    if depth > 3 {
        return Err("PBR resource directory is nested too deeply".to_string());
    }
    let at = base
        .checked_add(rel)
        .ok_or_else(|| "PBR directory offset overflow".to_string())?;
    if at + 16 > bytes.len() {
        return Err(format!("truncated PBR directory at offset {}", at));
    }
    let named = pbr_u16(bytes, at + 12)? as usize;
    let ids = pbr_u16(bytes, at + 14)? as usize;
    let total = named
        .checked_add(ids)
        .ok_or_else(|| "PBR directory entry count overflow".to_string())?;
    let entries = at + 16;
    let end = entries
        .checked_add(total * 8)
        .ok_or_else(|| "PBR directory table overflow".to_string())?;
    if end > bytes.len() {
        return Err(format!(
            "truncated PBR directory table at offset {}",
            entries
        ));
    }

    for i in 0..total {
        let pos = entries + i * 8;
        let raw_name = pbr_u32(bytes, pos)?;
        let raw_off = pbr_u32(bytes, pos + 4)?;
        let (this_int, this_str) = if raw_name & 0x8000_0000 != 0 {
            (
                None,
                Some(pbr_string(bytes, base, (raw_name & 0x7fff_ffff) as usize)?),
            )
        } else {
            (Some(raw_name), None)
        };

        if raw_off & 0x8000_0000 != 0 {
            let sub = (raw_off & 0x7fff_ffff) as usize;
            match depth {
                0 => pbr_walk_dir(
                    bytes, base, sub, 1, this_int, this_str, None, None, out, seen,
                )?,
                1 => pbr_walk_dir(
                    bytes,
                    base,
                    sub,
                    2,
                    type_int,
                    type_str.clone(),
                    this_int,
                    this_str,
                    out,
                    seen,
                )?,
                _ => return Err("a PBR language node cannot contain a subdirectory".to_string()),
            }
            continue;
        }

        if depth < 2 {
            return Err("PBR resource data appears before the language level".to_string());
        }
        let data_at = base
            .checked_add(raw_off as usize)
            .ok_or_else(|| "PBR data-entry offset overflow".to_string())?;
        if data_at + 16 > bytes.len() {
            return Err(format!("truncated PBR data entry at offset {}", data_at));
        }
        let data_rva = pbr_u32(bytes, data_at)? as usize;
        let data_size = pbr_u32(bytes, data_at + 4)? as usize;
        let payload = base
            .checked_add(data_rva)
            .ok_or_else(|| "PBR data payload offset overflow".to_string())?;
        let payload_end = payload
            .checked_add(data_size)
            .ok_or_else(|| "PBR data payload length overflow".to_string())?;
        if payload_end > bytes.len() {
            return Err(format!(
                "PBR payload {:#x}..{:#x} exceeds file length {}",
                payload,
                payload_end,
                bytes.len()
            ));
        }
        let lang = this_int
            .ok_or_else(|| "PBR language id is a string, expected a numeric id".to_string())?;
        if lang > u16::MAX as u32 {
            return Err(format!("PBR language id {} out of range", lang));
        }
        seen.insert(raw_off as usize);
        if data_size > 0 {
            out.push(ParsedResEntry {
                type_int,
                type_str: type_str.clone(),
                name_int,
                name_str: name_str.clone(),
                lang: lang as u16,
                data: bytes[payload..payload_end].to_vec(),
            });
        }
    }
    Ok(())
}

/// Parse a standard .RES binary into its resource entries.
/// Returns an error only on a structurally truncated file.
pub fn parse_res(bytes: &[u8]) -> Result<Vec<ParsedResEntry>, String> {
    let cur = Cursor { b: bytes };
    let mut out = Vec::new();
    let mut off = 0usize;
    while off + 8 <= bytes.len() {
        let data_size = cur.u32(off) as usize;
        let header_size = cur.u32(off + 4) as usize;
        if data_size == 0 && header_size == 0 {
            // Terminator / rc.exe alignment placeholder.
            break;
        }
        if header_size < 24 {
            return Err(format!(
                "invalid RES header size {} at offset {}",
                header_size, off
            ));
        }
        let mut p = off + 8;
        let (type_int, type_str, p2) = cur.read_id(p);
        p = p2;
        let (name_int, name_str, p2) = cur.read_id(p);
        p = p2;
        // DataVersion (DWORD) then MemoryFlags (WORD) + LanguageId (WORD).
        p += 4;
        let lang = cur.u16(p + 2);
        // Version + Characteristics (two DWORDs) bring the fixed header to 24;
        // the data start is derived from header_size rather than p.
        let data_start = off + header_size;
        let data_end = data_start + data_size;
        if data_end > bytes.len() {
            return Err(format!("truncated RES entry at offset {}", off));
        }
        // rc.exe emits a leading DataSize 0 placeholder; skip empty entries.
        if data_size > 0 {
            out.push(ParsedResEntry {
                type_int,
                type_str,
                name_int,
                name_str,
                lang,
                data: bytes[data_start..data_end].to_vec(),
            });
        }
        off = (data_end + 3) & !3;
    }
    Ok(out)
}

#[cfg(test)]
mod pbr_tests {
    fn put_u16(b: &mut [u8], at: usize, v: u16) {
        b[at..at + 2].copy_from_slice(&v.to_le_bytes());
    }
    fn put_u32(b: &mut [u8], at: usize, v: u32) {
        b[at..at + 4].copy_from_slice(&v.to_le_bytes());
    }
    fn put_str(b: &mut [u8], at: usize, s: &str) {
        let units: Vec<u16> = s.encode_utf16().collect();
        put_u16(b, at, units.len() as u16);
        for (i, u) in units.iter().enumerate() {
            put_u16(b, at + 2 + i * 2, *u);
        }
    }

    /// One RCDATA resource, numeric type 10 and numeric name 1.
    fn numeric_fixture() -> Vec<u8> {
        let mut b = vec![0u8; 0x94];
        b[0] = 0x00;
        b[1] = 0x05;
        b[2..4].copy_from_slice(b"CR");
        put_u32(&mut b, 4, 1);
        put_u32(&mut b, 8, 0x60); // data entry, relative to base 0x20
                                  // root directory at 0x20
        put_u16(&mut b, 0x2e, 1);
        put_u32(&mut b, 0x30, 10); // type RT_RCDATA
        put_u32(&mut b, 0x34, 0x8000_0020); // type subdir at base+0x20
                                            // type directory at 0x40
        put_u16(&mut b, 0x4e, 1);
        put_u32(&mut b, 0x50, 1); // resource id
        put_u32(&mut b, 0x54, 0x8000_0040); // name subdir at base+0x40
                                            // name directory at 0x60
        put_u16(&mut b, 0x6e, 1);
        put_u32(&mut b, 0x70, 1033); // language
        put_u32(&mut b, 0x74, 0x60); // data entry at base+0x60
                                     // data entry at 0x80
        put_u32(&mut b, 0x80, 0x70); // payload at base+0x70 = 0x90
        put_u32(&mut b, 0x84, 4);
        b[0x90..0x94].copy_from_slice(&[1, 2, 3, 4]);
        b
    }

    /// One resource with a string type ("TST") and string name ("NAMED").
    fn named_fixture() -> Vec<u8> {
        let mut b = vec![0u8; 0x120];
        b[0] = 0x00;
        b[1] = 0x05;
        b[2..4].copy_from_slice(b"CR");
        put_u32(&mut b, 4, 1);
        put_u32(&mut b, 8, 0x60);
        // root: type string at base+0xE0 = 0x100
        put_u16(&mut b, 0x2e, 1);
        put_u32(&mut b, 0x30, 0x8000_00E0);
        put_u32(&mut b, 0x34, 0x8000_0020);
        // type dir: name string at base+0xF0 = 0x110
        put_u16(&mut b, 0x4e, 1);
        put_u32(&mut b, 0x50, 0x8000_00F0);
        put_u32(&mut b, 0x54, 0x8000_0040);
        // name dir: language 1033 -> data entry 0x60
        put_u16(&mut b, 0x6e, 1);
        put_u32(&mut b, 0x70, 1033);
        put_u32(&mut b, 0x74, 0x60);
        // data entry at 0x80, payload at 0x90
        put_u32(&mut b, 0x80, 0x70);
        put_u32(&mut b, 0x84, 4);
        b[0x90..0x94].copy_from_slice(&[9, 8, 7, 6]);
        put_str(&mut b, 0x100, "TST");
        put_str(&mut b, 0x110, "NAMED");
        b
    }

    #[test]
    fn parses_numeric_pbr() {
        let out = super::parse_pbr(&numeric_fixture()).expect("numeric PBR");
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].type_int, Some(10));
        assert_eq!(out[0].name_int, Some(1));
        assert_eq!(out[0].lang, 1033);
        assert_eq!(out[0].data, [1, 2, 3, 4]);
    }

    #[test]
    fn parses_named_pbr() {
        let out = super::parse_pbr(&named_fixture()).expect("named PBR");
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].type_str.as_deref(), Some("TST"));
        assert_eq!(out[0].name_str.as_deref(), Some("NAMED"));
        assert_eq!(out[0].lang, 1033);
        assert_eq!(out[0].data, [9, 8, 7, 6]);
    }

    #[test]
    fn rejects_bad_signature_and_truncation() {
        assert!(super::parse_pbr(b"not a pbr").is_err());
        let mut b = numeric_fixture();
        b.truncate(0x80);
        assert!(super::parse_pbr(&b).is_err());
    }
}
