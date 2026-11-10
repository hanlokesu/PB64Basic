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

/// Parse a legacy .PBR (PowerBASIC PBRES) binary.  The PBRES format uses a
/// private internal offset-table layout; see the dedicated parser below.
pub fn parse_pbr(bytes: &[u8]) -> Result<Vec<ParsedResEntry>, String> {
    // Magic: WORD 0x0500 followed by ASCII "CR" (TrID signature
    // 00 05 43 52).  Reject anything that does not carry it.
    if bytes.len() < 4 || &bytes[2..4] != b"CR" {
        return Err("not a PowerBASIC PBR file (missing CR signature)".to_string());
    }
    // The legacy PBRES private offset-table layout is not yet fully decoded:
    // the data payloads can be located, but the directory tree that assigns
    // each (type, name) pair is not reliably traversable.  Fail loudly rather
    // than embedding entries with empty identifiers or silently dropping the
    // resource.  Convert the file to a standard .RES with the Win32 resource
    // compiler (or PBWin) and use `#RESOURCE RES, "file.RES"` instead.
    Err(
        "PBR (legacy PBRES) resources are not yet supported; convert the file \
         to a standard .RES and use #RESOURCE RES instead"
            .to_string(),
    )
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
