// =====================================================================
// Modified by the hanlokesu/PB64Basic fork (https://github.com/hanlokesu/PB64Basic).
// Derived from benstopics/powerbasilisk (https://github.com/benstopics/powerbasilisk), Apache-2.0.
// See LICENSE for the full license terms.
// =====================================================================
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::error::{PbError, PbResult};

/// A resolved source line with file origin tracking.
#[derive(Debug, Clone)]
pub struct SourceLine {
    pub text: String,
    pub file: PathBuf,
    pub line_num: usize,
}

/// Collected #RESOURCE VERSIONINFO data
#[derive(Debug, Clone, Default)]
pub struct VersionInfo {
    pub file_version: (u32, u32, u32, u32),
    pub product_version: (u32, u32, u32, u32),
    pub lang_id: u16,
    pub codepage: u16,
    pub strings: Vec<(String, String)>,
}

/// Win32 resource type for a file-based #RESOURCE item.
/// Numeric types are the standard RT_* integers; named types are custom
/// string type names (measured from PBWin10: WAVE and TYPELIB are stored
/// under their own string type names, not RT_RCDATA).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawResType {
    /// RT_BITMAP (2); the 14-byte BITMAPFILEHEADER is stripped at embed.
    Bitmap,
    /// RT_RCDATA (10); the whole file is stored byte-for-byte.
    RcData,
    /// Custom string type "WAVE"; whole file.
    Wave,
    /// Custom string type "TYPELIB"; whole file.
    TypeLib,
}

impl RawResType {
    /// Numeric RT_* id for integer types, or None for string-named types.
    pub fn numeric_id(&self) -> Option<u32> {
        match self {
            RawResType::Bitmap => Some(2),
            RawResType::RcData => Some(10),
            RawResType::Wave | RawResType::TypeLib => None,
        }
    }
    /// String type name for custom types, or None for integer types.
    pub fn type_name(&self) -> Option<&'static str> {
        match self {
            RawResType::Wave => Some("WAVE"),
            RawResType::TypeLib => Some("TYPELIB"),
            RawResType::Bitmap | RawResType::RcData => None,
        }
    }
}

/// A file-based #RESOURCE item (BITMAP/RCDATA/WAVE/TYPELIB) collected for
/// post-link embedding via UpdateResource.
#[derive(Debug, Clone)]
pub struct RawResource {
    pub id: u32,
    pub type_kind: RawResType,
    pub path: PathBuf,
}

/// A #RESOURCE STRING table entry.  Windows stores string resources in
/// blocks of 16 IDs (RT_STRING=6, resource name = block index = (id>>4)+1);
/// these individual entries are grouped into blocks at embed time.
#[derive(Debug, Clone)]
pub struct StringResource {
    pub id: u32,
    pub text: String,
    pub lang_id: u16,
}

/// A pre-compiled binary resource file referenced by
/// `#RESOURCE RES, "file.RES"` or `#RESOURCE PBR, "file.PBR"`.  The binary
/// is parsed into individual resource entries at embed time.  CHM hard rule:
/// RES/PBR cannot be mixed with any other #RESOURCE form.
#[derive(Debug, Clone)]
pub struct ResFileEntry {
    pub path: PathBuf,
    /// true = legacy PBRES .PBR format, false = standard .RES format.
    pub is_pbr: bool,
}

/// A MACRO definition (batch 27). Single-line macros have a 1-line body and
/// expand anywhere in a line; multi-line macros expand at statement position.
#[derive(Debug, Clone)]
struct MacroDef {
    params: Vec<String>,
    body: Vec<String>,
}

/// Strip trailing comment from a line, respecting string literals.
/// Returns the code portion (everything before the comment marker).
/// e.g. `IF X=1 _ ' old code` → `IF X=1 _`
///      `A$ = "it's fine"` → `A$ = "it's fine"` (no strip)
fn strip_trailing_comment(line: &str) -> &str {
    let bytes = line.as_bytes();
    let mut in_string = false;
    for i in 0..bytes.len() {
        if bytes[i] == b'"' {
            in_string = !in_string;
        } else if bytes[i] == b'\'' && !in_string {
            return &line[..i];
        }
    }
    line
}

/// Find a standalone `_` (not part of an identifier) that is followed by
/// non-blank content on the same line.  PBWin's continuation rule ignores
/// that trailing text (official ObjBase.inc: `AS ANY, _ IStorage PTR` /
/// `AS ANY _ FAR * LPVOID`); the underscore itself still continues the line.
/// Returns the byte index of the underscore, or None.
fn standalone_underscore_with_tail(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut in_string = false;
    for i in 0..bytes.len() {
        let b = bytes[i];
        if b == b'"' {
            in_string = !in_string;
            continue;
        }
        if b != b'_' || in_string {
            continue;
        }
        let prev_ok = i == 0 || !(bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'_');
        if !prev_ok {
            continue;
        }
        // Skip blank after the underscore; a bare trailing `_` (line
        // continuation) has nothing after it and is not this case.
        let mut j = i + 1;
        while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t') {
            j += 1;
        }
        if j < bytes.len() && bytes[j] != b'\r' && bytes[j] != b'\n' {
            return Some(i);
        }
    }
    None
}

/// Fast path for `$equate` substitution: instead of scanning the whole line
/// once per equate name (O(lines * equates * line_len)), scan the line once,
/// extract each `$IDENT` token and look it up in the map. Token-boundary rules
/// (prev char not alnum/_/`$`; the token run already ends at the next
/// non-identifier char, so the trailing boundary check is implied) match the
/// old per-name sweep. Single pass, no recursive expansion of substituted
/// values — same behavior as before.
fn substitute_equates_fast(text: &str, map: &HashMap<String, String>) -> String {
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'$' {
            let start = i;
            let mut j = i + 1;
            while j < bytes.len()
                && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_' || bytes[j] == b'$')
            {
                j += 1;
            }
            let before_ok = i == 0
                || !(bytes[i - 1].is_ascii_alphanumeric()
                    || bytes[i - 1] == b'_'
                    || bytes[i - 1] == b'$');
            if before_ok {
                let key = text[start..j].to_uppercase();
                if let Some(value) = map.get(&key) {
                    out.push_str(value);
                    i = j;
                    continue;
                }
            }
        }
        let ch_len = utf8_char_len(bytes[i]);
        out.push_str(&text[i..i + ch_len]);
        i += ch_len;
    }
    out
}

fn utf8_char_len(b: u8) -> usize {
    if b < 0x80 {
        1
    } else if b < 0xE0 {
        2
    } else if b < 0xF0 {
        3
    } else {
        4
    }
}

/// Preprocessor: resolves #INCLUDE, %CONSTANTS, MACRO blocks, and strips directives.
pub struct Preprocessor {
    constants: HashMap<String, i64>,
    /// String equates: `$CLSID_XXX = "..."` / `$$NAME` (batch 278).  The key
    /// is the full uppercase equate name (e.g. "$CLSID_FOO"); the value is the
    /// raw right-hand side of the definition line (quotes included), which is
    /// text-substituted where the equate name appears later in the source.
    string_constants: HashMap<String, String>,
    included: HashSet<PathBuf>,
    /// Second-pass include set: the macro-collection pass (batch 306) walks
    /// the include graph with its own cycle set so the emit pass can inline
    /// the same files again (with the now-complete macro table).
    collect_included: HashSet<PathBuf>,
    /// True while running the first (macro-collection) pass over the source.
    /// Regular lines are skipped; only MACRO definitions, %/$ constants,
    /// #IF stacks and #INCLUDE expansion run, so the macro table is complete
    /// before any line is expanded (PBWin semantics: WindowsX.inc defines
    /// ListBox_GetCurSel AFTER the function that calls it).
    collect_only: bool,
    /// Constants defined *during this pass* (plus the built-in SDK defaults),
    /// consulted by `%DEF(name)` in #IF guards.  The two-pass preprocess (batch
    /// 306) must NOT let the collection pass's definitions leak into the emit
    /// pass: official headers guard their defaults with
    /// `#IF NOT %DEF(%ANY_SIZE)` / `#IF NOT %DEF(%WIN32_WINNT)` — if the emit
    /// pass sees them as already defined, the `%ANY_SIZE = 1` line is skipped
    /// and the parser never registers it, so TYPE members like
    /// `table(%ANY_SIZE-1)` fail constant evaluation (38 parse errors, build44).
    defined_this_pass: HashSet<String>,
    macros: HashMap<String, MacroDef>,
    pub resources: Vec<(u32, PathBuf)>,
    /// #RESOURCE BITMAP/RCDATA/WAVE/TYPELIB — file-based raw resources.
    pub raw_resources: Vec<RawResource>,
    /// #RESOURCE STRING — individual string-table entries (grouped at embed).
    pub string_resources: Vec<StringResource>,
    /// #RESOURCE RES/PBR — pre-compiled binary resource files (parsed at
    /// embed; cannot be mixed with any other resource form).
    pub res_files: Vec<ResFileEntry>,
    /// #RESOURCE MANIFEST, id, "file.xml" — embedded as RT_MANIFEST so the
    /// EXE gets comctl32 v6 visual styles (flat buttons) like a PBWin build.
    pub manifests: Vec<(u32, PathBuf)>,
    pub version_info: VersionInfo,
    /// Official PB: `#DIM ALL` == `OPTION EXPLICIT` (every variable must be
    /// declared). Recorded here so main.rs can enforce it at codegen time.
    dim_all: bool,
    /// `#STACK num_expr` — requested runtime stack size in bytes (CHW
    /// _STACK_metastatement.htm).  Recorded here (rounded up to the next
    /// 64 Kb boundary, minimum 128 Kb) so main.rs can pass it to the linker.
    stack_size: Option<u64>,
    /// `#CONSOLE ON` / `#CONSOLE OFF` (CHW meta directives).  Some(true)
    /// selects a console-subsystem EXE (PRINT/WRITE go to the owning console),
    /// Some(false) forces the GUI subsystem; None leaves the historical
    /// default (GUI, see codegen::link_exe).  Recorded here so main.rs can
    /// pass it to the linker as /SUBSYSTEM:CONSOLE.
    console_mode: Option<bool>,
    /// Extra user-supplied `#INCLUDE` search directories, in command-line
    /// order (`--include-dir <path>`, repeatable).  Consulted before the
    /// built-in PBWin install roots and the source directory for a bare
    /// include file name (msg1413: headers installed outside the hard-coded
    /// `C:\PBWin10\WINAPI` roots still resolve).
    include_dirs: Vec<PathBuf>,
}

impl Default for Preprocessor {
    fn default() -> Self {
        Self::new()
    }
}

impl Preprocessor {
    pub fn new() -> Self {
        Preprocessor {
            constants: HashMap::from([
                // Batch 306: SDK version constants supplied before the official
                // headers run.  Note: the official includes DO define
                // %WIN32_WINNT themselves when unset — Win32Api.inc defaults
                // it to 0x0500, ObjBase.inc/ObjIdl.inc to 0x0600 via
                // `#IF (%NTDDI_VERSION >= ... AND NOT %DEF(%WIN32_WINNT))`.
                // Pre-defining 0x0600 keeps ws2def.inc's SOCKADDR_STORAGE on
                // the LongHorn alias path, which the recursive boolean
                // evaluator now reproduces faithfully.
                ("%WIN32_WINNT".to_string(), 0x0600),
                ("%WINVER".to_string(), 0x0600),
                ("%WIN32_IE".to_string(), 0x0700),
                // Hash family step 2: CHM built-in compiler-identity equates.
                // fork is an independent 64-bit compiler, so following the
                // PBxB64 reference it defines all of them but assigns its real
                // identity instead of claiming the 32-bit PB/Win product.
                // PB_WIN32/PB_DLL32/PB_CC32 = 0 (not that product, not TRUE).
                ("%PB_WIN32".to_string(), 0),
                ("%PB_DLL32".to_string(), 0),
                ("%PB_CC32".to_string(), 0),
                // PB_REVISION: high byte = 2 (0.2 line), low byte = 119 (patch).
                // 0x0277 = 631; its &H0FF00 part (0x0200) never matches the
                // PBWin10 &H1000 or PBWin9 &H0900 guards, so the existing
                // examples' fork branches are preserved.
                ("%PB_REVISION".to_string(), 0x0277),
                // PB_REVLETTER: no revision letter -> 0x20.
                ("%PB_REVLETTER".to_string(), 0x20),
                // PB_EXE: TRUE for an EXE target; set_build_mode flips it to
                // 0 for a DLL. Defaults to EXE (the documented default).
                ("%PB_EXE".to_string(), -1),
            ]),
            string_constants: HashMap::new(),
            included: HashSet::new(),
            collect_included: HashSet::new(),
            collect_only: false,
            defined_this_pass: HashSet::from([
                "%WIN32_WINNT".to_string(),
                "%WINVER".to_string(),
                "%WIN32_IE".to_string(),
                "%PB_WIN32".to_string(),
                "%PB_DLL32".to_string(),
                "%PB_CC32".to_string(),
                "%PB_REVISION".to_string(),
                "%PB_REVLETTER".to_string(),
                "%PB_EXE".to_string(),
            ]),
            macros: HashMap::new(),
            resources: Vec::new(),
            raw_resources: Vec::new(),
            string_resources: Vec::new(),
            res_files: Vec::new(),
            manifests: Vec::new(),
            version_info: VersionInfo::default(),
            dim_all: false,
            stack_size: None,
            console_mode: None,
            include_dirs: Vec::new(),
        }
    }

    /// Configure the compilation target before preprocessing so #IF guards can
    /// select target-specific code (e.g. linking a 32-bit vs 64-bit COFF object
    /// produced by bin2coff).  Defines %TARGET_64 / %TARGET_32 as PB truth
    /// (-1) on the matching target and 0 on the other.  These live in the
    /// cross-pass `constants` table, so they are available in both
    /// preprocessing passes and are not cleared by reset_defined_this_pass.
    pub fn set_target(&mut self, target: &str) {
        let t = target.to_ascii_lowercase();
        let is_64 = t.contains("x86_64") || t.contains("aarch64");
        self.constants
            .insert("%TARGET_64".to_string(), if is_64 { -1 } else { 0 });
        self.constants
            .insert("%TARGET_32".to_string(), if is_64 { 0 } else { -1 });
    }

    /// Configure the build output type before preprocessing so %PB_EXE carries
    /// the CHM value: TRUE (non-zero) for an EXE, FALSE (zero) for a DLL. SLL
    /// builds are not executables, so %PB_EXE is set to 0 there as well. The
    /// equate stays defined in every mode (CHM: %DEF(%PB_EXE) is always TRUE);
    /// only its assigned value changes.
    pub fn set_build_mode(&mut self, exe_mode: bool, dll_mode: bool, sll_mode: bool) {
        let val = if exe_mode { -1 } else { 0 };
        let _ = (dll_mode, sll_mode);
        self.constants.insert("%PB_EXE".to_string(), val);
    }

    /// Register one extra `#INCLUDE` search directory (repeatable from the
    /// command line).  Blank/whitespace entries are ignored; duplicates are
    /// collapsed so a directory passed twice does not lose search order or
    /// waste a probe.  User directories are consulted before the built-in
    /// PBWin install roots and the source directory.
    pub fn add_include_dir(&mut self, dir: &str) {
        let trimmed = dir.trim();
        if trimmed.is_empty() {
            return;
        }
        let path = PathBuf::from(trimmed);
        if !self.include_dirs.contains(&path) {
            self.include_dirs.push(path);
        }
    }

    /// Run the full two-pass preprocess (batch 306).  Pass 1 walks the source
    /// and include graph collecting every MACRO definition (with the correct
    /// #IF/constant state); pass 2 re-runs the same walk with the complete
    /// macro table so "use before macro definition" (WindowsX.inc:
    /// ListBox_GetCurSel) expands exactly like PBWin.
    pub fn preprocess(&mut self, path: &Path) -> PbResult<Vec<SourceLine>> {
        // Each pass sees a fresh "defined this pass" set: the guarded
        // `#IF NOT %DEF(%ANY_SIZE)` definition lines must be emitted to the
        // parser in BOTH passes (collection pass builds the macro/constant
        // state; emit pass re-emits the same lines so the parser registers
        // them), exactly like a single PBWin pass.
        self.collect_only = true;
        self.collect_included.clear();
        self.macros.clear();
        self.reset_defined_this_pass();
        let _ = self.process_file(path)?;
        self.collect_only = false;
        self.reset_defined_this_pass();
        let lines = self.process_file(path)?;
        Ok(lines)
    }

    /// Restore `defined_this_pass` to just the built-in SDK defaults.
    fn reset_defined_this_pass(&mut self) {
        self.defined_this_pass.clear();
        for k in [
            "%WIN32_WINNT",
            "%WINVER",
            "%WIN32_IE",
            "%PB_WIN32",
            "%PB_DLL32",
            "%PB_CC32",
            "%PB_REVISION",
            "%PB_REVLETTER",
            "%PB_EXE",
        ] {
            self.defined_this_pass.insert(k.to_string());
        }
    }

    /// Process a source file, returning all resolved lines (with includes inlined).
    pub fn process_file(&mut self, path: &Path) -> PbResult<Vec<SourceLine>> {
        let canonical = std::fs::canonicalize(path)
            .map_err(|e| PbError::io(format!("Cannot resolve path {}: {}", path.display(), e)))?;

        let included_set = if self.collect_only {
            &mut self.collect_included
        } else {
            &mut self.included
        };
        if included_set.contains(&canonical) {
            return Ok(Vec::new()); // cycle detection
        }
        included_set.insert(canonical.clone());

        // PB sources are often ANSI/GBK (not UTF-8). Try UTF-8 first; on
        // failure fall back to lossy decode (comments/strings may show
        // replacement chars, but ASCII keywords & structure are unaffected).
        let bytes = std::fs::read(&canonical)
            .map_err(|e| PbError::io(format!("Cannot read {}: {}", canonical.display(), e)))?;
        let content = match String::from_utf8(bytes.clone()) {
            Ok(s) => s,
            Err(_) => String::from_utf8_lossy(&bytes).into_owned(),
        };

        self.process_source(&content, &canonical)
    }

    fn process_source(&mut self, source: &str, file: &Path) -> PbResult<Vec<SourceLine>> {
        let mut lines = Vec::new();
        let raw_lines: Vec<&str> = source.lines().collect();
        let mut i = 0;
        let mut if_stack: Vec<bool> = Vec::new(); // stack of "are we in an active block?"
        let mut prefix: Option<String> = None; // active PREFIX "source code" (batch 26)
        let mut struct_depth: usize = 0; // TYPE/UNION nesting (batch 306)

        while i < raw_lines.len() {
            let line_num = i + 1;
            let raw = raw_lines[i];
            let trimmed = raw.trim();

            // Check if we're inside an inactive #IF block
            let active = if_stack.iter().all(|&b| b);

            // Handle #IF / #ELSEIF / #ELSE / #ENDIF
            let upper = trimmed.to_uppercase();
            if upper.starts_with("#IF ") || upper.starts_with("#IF\t") {
                let raw_cond = trimmed[3..].trim();
                if Self::contains_float_literal(&raw_cond.to_uppercase()) {
                    return Err(PbError::preprocessor(
                        "Error 427: Integer constant expected",
                        file.to_str(),
                        line_num,
                    ));
                }
                let condition = self.evaluate_if_condition(raw_cond);
                if active {
                    if_stack.push(condition);
                } else {
                    if_stack.push(false); // nested in inactive block
                }
                i += 1;
                continue;
            }
            if upper == "#ELSE"
                || upper.starts_with("#ELSE ")
                || upper.starts_with("#ELSE\t")
                || upper.starts_with("#ELSE'")
            {
                let len = if_stack.len();
                let parent_active = if len > 1 {
                    if_stack[..len - 1].iter().all(|&b| b)
                } else {
                    true
                };
                if let Some(last) = if_stack.last_mut() {
                    if parent_active {
                        *last = !*last;
                    }
                }
                i += 1;
                continue;
            }
            if upper.starts_with("#ELSEIF ") || upper.starts_with("#ELSEIF\t") {
                let len = if_stack.len();
                let parent_active = if len > 1 {
                    if_stack[..len - 1].iter().all(|&b| b)
                } else {
                    true
                };
                let raw_cond = trimmed[8..].trim();
                if Self::contains_float_literal(&raw_cond.to_uppercase()) {
                    return Err(PbError::preprocessor(
                        "Error 427: Integer constant expected",
                        file.to_str(),
                        line_num,
                    ));
                }
                let cond = self.evaluate_if_condition(raw_cond);
                if let Some(last) = if_stack.last_mut() {
                    if parent_active {
                        if *last {
                            *last = false;
                        } else {
                            *last = cond;
                        }
                    }
                }
                i += 1;
                continue;
            }
            if upper == "#ENDIF"
                || upper.starts_with("#ENDIF ")
                || upper.starts_with("#ENDIF\t")
                || upper.starts_with("#ENDIF'")
            {
                if_stack.pop();
                i += 1;
                continue;
            }

            if !active {
                i += 1;
                continue;
            }

            // Handle line continuations: join lines ending with _
            // Must strip comments BEFORE checking for _, because a comment may
            // contain a trailing _ that isn't a real continuation marker.
            // e.g.: IF X=1 _ ' old code was: IF X=1 AND Y=2 _
            //           OR Z=3 THEN
            let mut full_line = raw.to_string();
            loop {
                let mut code_owned = strip_trailing_comment(&full_line).to_string();
                // Batch 306: PBWin's continuation rule allows a standalone `_`
                // to be followed by text on the same line — the official
                // ObjBase.inc writes `AS ANY, _ IStorage PTR` (L1245) and
                // `AS ANY _ FAR * LPVOID` (BindMoniker, L1346) where the text
                // after the underscore is a documentation hint, not a real
                // continuation.  PBWin ignores everything after the underscore
                // (to end of line); the underscore itself still continues the
                // line into the next physical row.  Strip that trailing text so
                // the emitted line never carries a mid-line underscore.
                if let Some(pos) = standalone_underscore_with_tail(&code_owned) {
                    code_owned.truncate(pos + 1);
                }
                let trimmed_code = code_owned.trim_end();
                if !trimmed_code.ends_with('_') || trimmed_code.len() <= 1 {
                    break;
                }
                // Remove the trailing underscore from the code (not the comment)
                let before_underscore = &trimmed_code[..trimmed_code.len() - 1];
                full_line = before_underscore.to_string();
                i += 1;
                if i < raw_lines.len() {
                    let nxt = raw_lines[i].trim();
                    // Batch 306: an official include (WinDNS.inc) places a
                    // #IF / #ELSE / #ENDIF directive INSIDE a continued
                    // DECLARE parameter list (`... AS DNS_FREE_TYPE _` followed
                    // by a lone `_` then `#ENDIF ' ...`).  The directive line
                    // must NOT be glued onto the continued statement — it has
                    // to stay a standalone preprocessor line so the #IF stack
                    // stays balanced (PBWin treats directives the same way).
                    if nxt.starts_with('#') {
                        break;
                    }
                    full_line.push(' ');
                    full_line.push_str(nxt);
                }
            }

            let trimmed_full = full_line.trim();

            // Skip empty lines and comments
            if trimmed_full.is_empty() || trimmed_full.starts_with('\'') {
                i += 1;
                continue;
            }
            // REM comment
            if trimmed_full.len() >= 3
                && trimmed_full[..3].to_uppercase() == "REM"
                && (trimmed_full.len() == 3 || !trimmed_full.as_bytes()[3].is_ascii_alphanumeric())
            {
                i += 1;
                continue;
            }

            let upper_full = trimmed_full.to_uppercase();

            // #CONSOLE OFF / #CONSOLE ON are legal PB directives.  The old
            // console_off flag (batch 129) was removed in batch 250: `?`
            // always lowers to MSGBOX regardless of console mode.  The
            // directive now selects the PE subsystem (batch 310): ON records
            // the console subsystem, OFF keeps the GUI subsystem.
            if upper_full.starts_with("#CONSOLE OFF") {
                self.console_mode = Some(false);
                i += 1;
                continue;
            }
            if upper_full.starts_with("#CONSOLE ON") {
                self.console_mode = Some(true);
                i += 1;
                continue;
            }
            // (batch 129 note: #INCLUDE win32api.inc / windows.inc used to set
            //  console_off = true; that flag is gone, the directive itself is
            //  still handled normally below.)

            // Batch 129 revisited (batch 250): `?` at line start is the PB
            // official MSGBOX statement abbreviation (CHW msgbox_s.htm:
            // `? txt$ [, [style&], title$]`).  PBWin compiles it to a real
            // message box even without #CONSOLE OFF / #INCLUDE WIN32API
            // (batch 250 probe p4/p5: PBWin exe stayed running on a message
            // box).  The old console_off split (PRINT when console) is wrong
            // for PBWin compatibility, so `?` always lowers to MSGBOX; the
            // lexer/parser Question token path is a second safety net for
            // statement positions the preprocessor does not rewrite.
            let replaced: Option<String> = if trimmed_full.starts_with('?') {
                let rest = trimmed_full.strip_prefix('?').unwrap().trim_start();
                Some(format!("MSGBOX {}", rest))
            } else {
                None
            };
            let trimmed_full: &str = replaced.as_deref().unwrap_or(trimmed_full);

            // MACRO / END MACRO (batch 27): collect definitions, skip their lines
            if upper_full.starts_with("MACRO") {
                i = self.collect_macro(&raw_lines, i, trimmed_full);
                continue;
            }
            if upper_full.starts_with("END MACRO") {
                // stray END MACRO — skip
                i += 1;
                continue;
            }

            // PREFIX / END PREFIX (batch 26): prepend "source code" to every
            // following line until END PREFIX. Text-level transform only.
            if upper_full == "END PREFIX" {
                prefix = None;
                i += 1;
                continue;
            }
            if upper_full.starts_with("PREFIX") {
                let rest = trimmed_full[6..].trim();
                if let Some(code) = extract_string(rest) {
                    prefix = Some(code);
                }
                i += 1;
                continue;
            }

            // #COMPILE / #COMPILER — skip entirely
            if upper_full.starts_with("#COMPILE") || upper_full.starts_with("#COMPILER") {
                i += 1;
                continue;
            }

            // #DIM ALL — official meta-directive == OPTION EXPLICIT.
            // Record it (main.rs enforces at codegen); other #DIM forms skip.
            if upper_full.starts_with("#DIM") {
                if upper_full.starts_with("#DIM ALL") {
                    self.dim_all = true;
                }
                i += 1;
                continue;
            }

            // #REGISTER — skip
            if upper_full.starts_with("#REGISTER") {
                i += 1;
                continue;
            }

            // #MESSAGES COMMAND / NOTIFY — CHW _MESSAGES_metastatement.htm:
            // selects whether %WM_NOTIFY is also dispatched to Control Callback
            // Functions.  The fork's callback dispatch already passes both
            // message classes through (PBWin's NOTIFY is the default), so the
            // directive is accepted and recorded as a no-op.
            if upper_full.starts_with("#MESSAGES") {
                i += 1;
                continue;
            }

            // #OPTIMIZE CODE [ON|OFF] / {SIZE|SPEED} — CHW
            // _OPTIMIZE_metastatement.htm: compiler optimization preference.
            // The fork's LLVM backend drives optimization via -O flags, so the
            // directive is accepted and ignored (SIZE/SPEED have no codegen
            // equivalent here).
            if upper_full.starts_with("#OPTIMIZE") {
                i += 1;
                continue;
            }

            // #STACK num_expr — CHW _STACK_metastatement.htm: requested runtime
            // stack size in bytes, rounded up to the next 64 Kb boundary, minimum
            // 128 Kb, default 1 Mb.  Recorded and passed to the linker as
            // /STACK:bytes by main.rs.
            if upper_full.starts_with("#STACK") {
                let rest = trimmed_full[6..].trim();
                if let Ok(n) = rest.parse::<u64>() {
                    let mut n = n.max(128 * 1024);
                    n = n.div_ceil(65536) * 65536;
                    self.stack_size = Some(n);
                }
                i += 1;
                continue;
            }

            // #TOOLS [ON|+|OFF|-] — CHW _TOOLS_metastatement.htm: disables the
            // TRACE / PROFILE / CALLSTK instrumentation.  The fork generates no
            // such instrumentation, so accept and ignore.
            if upper_full.starts_with("#TOOLS") {
                i += 1;
                continue;
            }

            // #UTILITY "text" — CHW _UTILITY_metastatement.htm: "The entire
            // line is ignored by the PowerBASIC compiler."  Swallow it.
            if upper_full.starts_with("#UTILITY") {
                i += 1;
                continue;
            }

            // #ALIGN / #BLOAT / #COM / #EXPORT / #PAGE / #UNIQUE — CHW
            // metastatements with no 64-bit fork semantics:
            //   #ALIGN  NOP-pads instruction boundaries (assembler-level)
            //   #BLOAT  inflates the disk image (running size unaffected)
            //   #COM    COM type-library info (CLASS/DOC/HELP/NAME/GUID/TLIB)
            //   #EXPORT adds the EXPORT attribute to Sub/Functions defined
            //           elsewhere (DLL/SLL interop; fork links statically)
            //   #PAGE   IDE print pagination — CHW: "no effect on your
            //           compiled code"
            //   #UNIQUE VAR [ON|OFF] — unique-name check (default OFF)
            // All are accepted and swallowed so official sources compile
            // (#PAGE is exactly equivalent to CHW; the rest are documented as
            // not implemented in the coverage CSV).  (A/B/C re-audit.)
            if upper_full.starts_with("#ALIGN")
                || upper_full.starts_with("#BLOAT")
                || upper_full.starts_with("#COM")
                || upper_full.starts_with("#EXPORT")
                || upper_full.starts_with("#PAGE")
                || upper_full.starts_with("#UNIQUE")
            {
                i += 1;
                continue;
            }

            // #DEBUG — skip (bounds/display/error/numeric all covered by this)
            if upper_full.starts_with("#DEBUG") {
                i += 1;
                continue;
            }

            // #OPTION — skip (explicit/ansi etc., we don't enforce)
            if upper_full.starts_with("#OPTION") {
                i += 1;
                continue;
            }

            // DEBUG
            // #RESOURCE STRING, ResID, "text" [, LangID] — string-table entry.
            // ResID must be numeric; text is 1..127 chars, no embedded nul.
            // Collected individually and grouped into 16-ID RT_STRING blocks
            // at embed time (block name = (id>>4)+1, slot = id&15).
            if upper_full.starts_with("#RESOURCE STRING") {
                let rest = trimmed_full
                    .strip_prefix("#RESOURCE STRING")
                    .or_else(|| trimmed_full.strip_prefix("#resource string"))
                    .unwrap_or("")
                    .trim()
                    .trim_start_matches(",")
                    .trim();
                let parts: Vec<&str> = rest.split(',').map(|s| s.trim()).collect();
                if parts.len() >= 2 {
                    if let Ok(sid) = parts[0].parse::<u32>() {
                        if let Some(text) = extract_string(parts[1]) {
                            // Optional LangID (4th comma field); default 1033.
                            let lang = parts
                                .get(2)
                                .and_then(|l| l.trim().parse::<u16>().ok())
                                .unwrap_or(1033);
                            if !self.collect_only {
                                self.string_resources.push(StringResource {
                                    id: sid,
                                    text,
                                    lang_id: lang,
                                });
                            }
                        }
                    }
                }
                i += 1;
                continue;
            }
            // #RESOURCE BITMAP/RCDATA/WAVE/TYPELIB, ResID, "file" — file-based
            // raw resources, collected for post-link UpdateResource embedding.
            // Type mapping measured from PBWin10 (llvm-readobj): BITMAP ->
            // RT_BITMAP(2, 14-byte header stripped), RCDATA -> RT_RCDATA(10),
            // WAVE -> custom "WAVE", TYPELIB -> custom "TYPELIB".
            let file_kind = ["BITMAP", "RCDATA", "WAVE", "TYPELIB"]
                .iter()
                .find(|k| upper_full.starts_with(&format!("#RESOURCE {}", k)));
            if let Some(kind_word) = file_kind {
                let prefix_len = "#RESOURCE ".len() + kind_word.len();
                let rest = trimmed_full[prefix_len..]
                    .trim()
                    .trim_start_matches(",")
                    .trim();
                let parts: Vec<&str> = rest.split(',').map(|s| s.trim()).collect();
                if parts.len() >= 2 {
                    if let Ok(id) = parts[0].parse::<u32>() {
                        if let Some(res_path) = extract_string(parts[1]) {
                            let resolved = file.parent().unwrap_or(Path::new(".")).join(&res_path);
                            if resolved.exists() && !self.collect_only {
                                let type_kind = match *kind_word {
                                    "BITMAP" => RawResType::Bitmap,
                                    "RCDATA" => RawResType::RcData,
                                    "WAVE" => RawResType::Wave,
                                    _ => RawResType::TypeLib,
                                };
                                self.raw_resources.push(RawResource {
                                    id,
                                    type_kind,
                                    path: resolved,
                                });
                            }
                        }
                    }
                }
                i += 1;
                continue;
            }
            // #RESOURCE RES, "file.RES" / #RESOURCE PBR, "file.PBR" — a
            // pre-compiled binary resource file.  Only the file path is
            // collected here; the binary is parsed into individual resource
            // entries at embed time.  CHM hard rule: RES/PBR cannot be mixed
            // with any other #RESOURCE form (enforced after preprocessing).
            let res_kind = if upper_full.starts_with("#RESOURCE RES") {
                Some(false)
            } else if upper_full.starts_with("#RESOURCE PBR") {
                Some(true)
            } else {
                None
            };
            if let Some(is_pbr) = res_kind {
                let marker = if is_pbr {
                    "#RESOURCE PBR"
                } else {
                    "#RESOURCE RES"
                };
                let rest = trimmed_full
                    .strip_prefix(marker)
                    .or_else(|| {
                        trimmed_full.strip_prefix(if is_pbr {
                            "#resource pbr"
                        } else {
                            "#resource res"
                        })
                    })
                    .unwrap_or("")
                    .trim()
                    .trim_start_matches(",")
                    .trim();
                if let Some(res_path) = extract_string(rest) {
                    let resolved = file.parent().unwrap_or(Path::new(".")).join(&res_path);
                    if resolved.exists() && !self.collect_only {
                        self.res_files.push(ResFileEntry {
                            path: resolved,
                            is_pbr,
                        });
                    }
                }
                i += 1;
                continue;
            }
            // #RESOURCE ICON, id, "file.ico" — collect for post-link embedding
            if upper_full.starts_with("#RESOURCE ICON") {
                let rest = trimmed_full[14..].trim().trim_start_matches(",").trim();
                // rest is like: 100, "Hello.ico"
                let parts: Vec<&str> = rest.split(',').map(|s| s.trim()).collect();
                if parts.len() >= 2 {
                    if let Ok(id) = parts[0].parse::<u32>() {
                        if let Some(icon_path) = extract_string(parts[1]) {
                            let resolved = file.parent().unwrap_or(Path::new(".")).join(&icon_path);
                            if resolved.exists() {
                                self.resources.push((id, resolved));
                            }
                        }
                    }
                }
                i += 1;
                continue;
            }
            // #RESOURCE MANIFEST, id, "file.xml" — visual-styles manifest,
            // embedded post-link as RT_MANIFEST (same layout as ICON).
            if upper_full.starts_with("#RESOURCE MANIFEST") {
                let rest = trimmed_full[18..].trim().trim_start_matches(",").trim();
                let parts: Vec<&str> = rest.split(',').map(|s| s.trim()).collect();
                if parts.len() >= 2 {
                    if let Ok(id) = parts[0].parse::<u32>() {
                        if let Some(mpath) = extract_string(parts[1]) {
                            let resolved = file.parent().unwrap_or(Path::new(".")).join(&mpath);
                            if resolved.exists() {
                                self.manifests.push((id, resolved));
                            } else {
                                eprintln!(
                                    "[PB64Basic] warning: #RESOURCE MANIFEST file not found: {}",
                                    resolved.display()
                                );
                            }
                        }
                    }
                }
                i += 1;
                continue;
            }
            // #RESOURCE VERSIONINFO / FILEVERSION / PRODUCTVERSION / STRINGINFO / VERSION$
            if upper_full.starts_with("#RESOURCE FILEVERSION") {
                let nums: Vec<u32> = trimmed_full[19..]
                    .split(',')
                    .filter_map(|x| x.trim().parse().ok())
                    .collect();
                if nums.len() >= 4 {
                    self.version_info.file_version = (nums[0], nums[1], nums[2], nums[3]);
                }
                i += 1;
                continue;
            }
            if upper_full.starts_with("#RESOURCE PRODUCTVERSION") {
                let nums: Vec<u32> = trimmed_full[22..]
                    .split(',')
                    .filter_map(|x| x.trim().parse().ok())
                    .collect();
                if nums.len() >= 4 {
                    self.version_info.product_version = (nums[0], nums[1], nums[2], nums[3]);
                }
                i += 1;
                continue;
            }
            if upper_full.starts_with("#RESOURCE STRINGINFO") {
                let _q: Vec<Option<String>> = [trimmed_full[19..].trim()]
                    .iter()
                    .map(|s| extract_string(s))
                    .collect();
                // Parse two quoted strings: "0409", "04B0"
                let mut strs: Vec<String> = Vec::new();
                for part in trimmed_full[19..].split(',') {
                    if let Some(v) = extract_string(part.trim()) {
                        strs.push(v);
                    }
                }
                if strs.len() >= 2 {
                    self.version_info.lang_id = u16::from_str_radix(&strs[1], 16).unwrap_or(0x04B0);
                    self.version_info.codepage =
                        u16::from_str_radix(&strs[0], 16).unwrap_or(0x0409);
                }
                i += 1;
                continue;
            }
            if upper_full.starts_with("#RESOURCE VERSION") && trimmed_full.contains("$") {
                // #RESOURCE VERSION$ "Key", "Value"
                let rest = trimmed_full[18..].trim();
                let parts: Vec<&str> = rest.splitn(2, ',').map(|s| s.trim()).collect();
                if parts.len() == 2 {
                    if let (Some(k), Some(v)) = (extract_string(parts[0]), extract_string(parts[1]))
                    {
                        self.version_info.strings.push((k, v));
                    }
                }
                i += 1;
                continue;
            }
            // #RESOURCE VERSIONINFO (just a marker, skip)
            if upper_full.starts_with("#RESOURCE") {
                i += 1;
                continue;
            }

            // #INCLUDE "file"  (also `#INCLUDE ONCE "file"`, the PBWin-official
            // form; batch 276: official samples always use ONCE.  ONCE-once
            // semantics are the same here since included paths are tracked
            // per file, so just strip the keyword.)
            if upper_full.starts_with("#INCLUDE") {
                let mut rest = trimmed_full[8..].trim();
                if let Some(no_once) = rest
                    .strip_prefix("ONCE")
                    .or_else(|| rest.strip_prefix("once"))
                    .or_else(|| rest.strip_prefix("Once"))
                {
                    rest = no_once.trim();
                }
                if let Some(inc_path) = extract_string(rest) {
                    let resolved = resolve_include_path(file, &inc_path, &self.include_dirs);
                    match self.process_file(&resolved) {
                        Ok(inc_lines) => {
                            lines.extend(inc_lines);
                        }
                        Err(e) => {
                            eprintln!("Warning: failed to include {}: {}", resolved.display(), e);
                        }
                    }
                }
                i += 1;
                continue;
            }

            // $NAME = "..." — string equate (CHW equate directive; the
            // official MyDLL.bas uses $CLSID_xxx for its COM class IDs).
            // Definitions are recorded and later occurrences are text-
            // substituted with the raw definition text (quotes included).
            // The definition line itself is NOT emitted (pure preprocessor
            // directive — the parser has no $NAME statement).
            if trimmed_full.starts_with('$') {
                if let Some(eq_pos) = trimmed_full.find('=') {
                    let name = trimmed_full[..eq_pos].trim().to_uppercase();
                    if name.starts_with('$') && name.len() > 1 {
                        let val_str = strip_comment(trimmed_full[eq_pos + 1..].trim());
                        self.string_constants
                            .insert(name.clone(), val_str.to_string());
                    }
                }
                i += 1;
                continue;
            }

            // %CONSTANT = value  (define a constant).  Official form is
            // `%CONSTANT %name = value` (CHW constant_directive.htm); the
            // leading `%CONSTANT` keyword must be stripped or the recorded
            // key becomes "%CONSTANT %NAME" and lookups of %NAME miss (they
            // fell back to 0 — batch 276 traced the LIBMAIN SELECT CASE
            // matching nothing through exactly this).
            if trimmed_full.starts_with('%') {
                if let Some(eq_pos) = trimmed_full.find('=') {
                    let mut name = trimmed_full[..eq_pos].trim().to_uppercase();
                    if name.starts_with("%CONSTANT") {
                        name = name["%CONSTANT".len()..].trim().to_string();
                    }
                    let val_str = trimmed_full[eq_pos + 1..].trim();
                    // Strip trailing comments
                    let val_str = strip_comment(val_str);
                    let value = self.parse_constant_value(&val_str);
                    self.constants.insert(name.to_string(), value);
                    self.defined_this_pass.insert(name.to_string());
                }
                // Emit the line anyway so the parser can also see it
                lines.push(SourceLine {
                    text: full_line.clone(),
                    file: file.to_path_buf(),
                    line_num,
                });
                i += 1;
                continue;
            }

            // Macro-collection pass (batch 306): every directive handler above
            // (MACRO collect, #IF stack, %/$ constants, #INCLUDE expansion) has
            // run; ordinary lines are skipped entirely so the macro table is
            // complete before the emit pass expands any line.  PBWin semantics:
            // WindowsX.inc defines ListBox_GetCurSel AFTER the function that
            // calls it, and official headers do this 237 files wide.
            if self.collect_only {
                i += 1;
                continue;
            }

            // Regular line — emit it (prepend active PREFIX source code, batch 26)
            // DATA rewrite (batch 275): the lexer uppercases bare identifiers,
            // which corrupts unquoted DATA items (CHW: unquoted items are trimmed
            // of leading/trailing spaces but keep their text).  Rewrite unquoted
            // items as quoted string literals so the lexer keeps them verbatim.
            // TYPE/UNION block nesting (batch 306): inside a structure block
            // "DATA AS SomeType" is a MEMBER name (WinDNS.inc unions use DATA as
            // a field name), not the DATA statement — suppress the rewrite there
            // or the member line gets corrupted into quoted string literals.
            if struct_depth == 0 {
                if upper_full == "TYPE"
                    || upper_full.starts_with("TYPE ")
                    || upper_full == "UNION"
                    || upper_full.starts_with("UNION ")
                {
                    struct_depth += 1;
                }
            } else {
                if upper_full == "END TYPE"
                    || upper_full.starts_with("END TYPE ")
                    || upper_full.starts_with("END TYPE'")
                    || upper_full == "END UNION"
                    || upper_full.starts_with("END UNION ")
                    || upper_full.starts_with("END UNION'")
                {
                    struct_depth -= 1;
                }
            }
            let data_rewritten: Option<String> =
                if struct_depth == 0 && (upper_full.starts_with("DATA ") || upper_full == "DATA") {
                    Some(rewrite_data_line(trimmed_full))
                } else {
                    None
                };
            let expanded = self.expand_macros(
                data_rewritten.as_deref().unwrap_or(trimmed_full),
                data_rewritten.as_deref().unwrap_or(&full_line),
            );
            let mut emitted_text = expanded;
            // Second rewrite site (batch 129 legacy): `expand_macros` seeds
            // from the ORIGINAL line (`full`), so a `?`-leading line can come
            // back out of it as `? ...` again.  Same rule as the first site:
            // `?` -> MSGBOX unconditionally (batch 250, PBWin probe-verified).
            {
                let t = emitted_text.trim_start();
                if t.starts_with('?') {
                    let indent_len = emitted_text.len() - t.len();
                    let indent = &emitted_text[..indent_len];
                    let rest = t.strip_prefix('?').unwrap().trim_start();
                    emitted_text = format!("{}{}{}", indent, "MSGBOX ", rest);
                }
            }
            if let Some(p) = &prefix {
                emitted_text = format!("{}{}", p, emitted_text);
            }
            for piece in emitted_text.split('\n') {
                lines.push(SourceLine {
                    text: piece.to_string(),
                    file: file.to_path_buf(),
                    line_num,
                });
            }

            i += 1;
        }

        // String equate substitution (batch 278): every `$NAME` occurrence is
        // text-replaced with the definition's right-hand side, except on the
        // definition lines themselves (they were pushed raw above).
        if !self.string_constants.is_empty() {
            for sl in lines.iter_mut() {
                let t = sl.text.trim_start();
                if t.starts_with('$') && sl.text.contains('=') {
                    continue; // definition line
                }
                sl.text = substitute_equates_fast(&sl.text, &self.string_constants);
            }
        }

        Ok(lines)
    }

    /// Collect a MACRO definition starting at raw_lines[start_i] (whose trimmed
    /// text is `first_line`). Returns the index of the next unprocessed line.
    /// Single-line: `MACRO name(params) = text`; multi-line: `MACRO name(params)`
    /// ... `END MACRO`. MACROTEMP / EXIT MACRO / macro functions are not
    /// expanded in this batch; their definition lines are skipped.
    fn collect_macro(&mut self, raw_lines: &[&str], start_i: usize, first_line: &str) -> usize {
        let rest = first_line[5..].trim();
        let upper_rest = rest.to_uppercase();
        // MACRO FUNCTION name ... END MACRO = expr — macro function, not
        // supported this batch: consume the whole block.
        if upper_rest.starts_with("FUNCTION") {
            let mut j = start_i + 1;
            while j < raw_lines.len() {
                if raw_lines[j].trim().to_uppercase().starts_with("END MACRO") {
                    return j + 1;
                }
                j += 1;
            }
            return j;
        }
        let (name, params) = parse_macro_proto(rest);
        let eq_pos = find_eq_outside_parens(rest);
        if let Some(eqpos) = eq_pos {
            // Single-line macro: body is everything after '='
            let body_text = rest[eqpos + 1..].trim().to_string();
            if !name.is_empty() {
                self.macros.insert(
                    name,
                    MacroDef {
                        params,
                        body: vec![body_text],
                    },
                );
            }
            return start_i + 1;
        }
        // Multi-line macro: collect until END MACRO
        let mut body: Vec<String> = Vec::new();
        let mut j = start_i + 1;
        while j < raw_lines.len() {
            let t = raw_lines[j].trim();
            let u = t.to_uppercase();
            if u.starts_with("END MACRO") {
                break;
            }
            if u.starts_with("MACROTEMP") {
                // MACROTEMP unique-identifier renaming not supported this
                // batch; skip the declaration line.
                j += 1;
                continue;
            }
            body.push(t.to_string());
            j += 1;
        }
        if !name.is_empty() {
            self.macros.insert(name, MacroDef { params, body });
        }
        j + 1
    }

    /// Expand macros in a line. Multi-line macros expand at statement position
    /// (line start); single-line macros expand anywhere in the line. Returns
    /// the expanded text, possibly containing embedded newlines.
    fn expand_macros(&self, trimmed: &str, full: &str) -> String {
        // Multi-line macros: statement position (line start).
        let trimmed_upper = trimmed.to_uppercase();
        for (name, def) in &self.macros {
            if def.body.len() > 1 {
                if let Some(rest) = trimmed_upper.strip_prefix(&name.to_uppercase()) {
                    let ok = rest.is_empty()
                        || rest.starts_with('(')
                        || rest
                            .chars()
                            .next()
                            .map(|c| c.is_whitespace())
                            .unwrap_or(false);
                    if ok {
                        let args = extract_paren_args(trimmed, name.len());
                        let mut out: Vec<String> = Vec::new();
                        for bl in &def.body {
                            out.push(substitute_params(bl, &def.params, &args));
                        }
                        return out.join("\n");
                    }
                }
            }
        }
        // Single-line macros: anywhere in the line.  Macro bodies may alias
        // other macros (NtDdndis.inc: NDIS_802_11_ENCRYPTION_STATUS_enum =
        // NDIS_802_11_WEP_STATUS_enum = LONG), so replacement is iterated to
        // a fixpoint (batch306 #5) — PBWin resolves the full alias chain.
        let mut result = full.to_string();
        for _ in 0..8 {
            let (next, changed) = Self::expand_single_macros_once(&result, &self.macros);
            result = next;
            if !changed {
                break;
            }
        }
        result
    }

    /// One fixpoint round of single-line macro expansion: scan the line once,
    /// extract each identifier token and look it up in the macro map, instead
    /// of walking every macro over the whole line (O(lines * macros * line_len)).
    /// Only body.len()==1 macros are matched here; multi-line macros are handled
    /// by the statement-position pass above. Returns (new text, changed).
    fn expand_single_macros_once(
        result: &str,
        macros: &HashMap<String, MacroDef>,
    ) -> (String, bool) {
        let bytes = result.as_bytes();
        let mut out = String::new();
        let mut changed = false;
        let mut i = 0usize;
        while i < bytes.len() {
            let c = bytes[i] as char;
            if c.is_ascii_alphabetic() || c == b'_' as char {
                let start = i;
                while i < bytes.len()
                    && ((bytes[i] as char).is_ascii_alphanumeric()
                        || bytes[i] == b'_'
                        || bytes[i] == b'$')
                {
                    i += 1;
                }
                let word = &result[start..i];
                let key = word.to_uppercase();
                let hit = macros.get(&key).filter(|d| d.body.len() == 1);
                if let Some(def) = hit {
                    let body = def.body.first().cloned().unwrap_or_default();
                    let mut j = i;
                    while j < bytes.len() && (bytes[j] as char).is_ascii_whitespace() {
                        j += 1;
                    }
                    if j < bytes.len() && bytes[j] == b'(' {
                        if let Some(close_rel) = find_matching_paren(&result[j..]) {
                            let close = j + close_rel;
                            let args = split_args(&result[j + 1..close]);
                            out.push_str(&substitute_params(&body, &def.params, &args));
                            i = close + 1;
                            changed = true;
                            continue;
                        }
                    }
                    out.push_str(&substitute_params(&body, &def.params, &[]));
                    changed = true;
                    continue;
                }
                out.push_str(word);
            } else {
                out.push(c);
                i += 1;
            }
        }
        (out, changed)
    }

    /// Evaluate a #IF / #ELSEIF condition.
    ///
    /// Batch 306 rewrite: full recursive boolean evaluation.  The old
    /// implementation handled only `NOT expr`, one bare comparison operator
    /// and `%DEF(x)` as an entire line, so official guards such as
    ///
    ///   #IF (%NTDDI_VERSION >= %NTDDI_VISTA AND NOT %DEF(%WIN32_WINNT))
    ///   #IF %DEF(%STATUS_SUCCESS) OR (%WIN32_WINNT > &H0500)
    ///
    /// were mis-evaluated: ObjBase.inc's four conditional `%WIN32_WINNT =`
    /// defaults all fired (the right operand `... AND NOT %DEF(...)` parsed as
    /// 0, so `0 >= 0` was true), the last one overwrote the constant with
    /// &H0500, and ws2def.inc's `MACRO SOCKADDR_STORAGE=` alias was never
    /// defined -> clang "use of undefined type named 'SOCKADDR_STORAGE'".
    fn evaluate_if_condition(&self, condition: &str) -> bool {
        let cond = strip_comment(condition.trim()).to_uppercase();
        self.eval_bool_expr(&cond)
    }

    /// Detect a floating-point literal (e.g. `3.5`) in a #IF condition.
    /// PBWin rejects these with Error 427 "Integer constant expected".
    /// String contents are masked so a '.' inside CVQ("..") is not flagged.
    fn contains_float_literal(condition: &str) -> bool {
        let bytes = condition.as_bytes();
        let mut masked = Vec::with_capacity(bytes.len());
        let mut quote: Option<u8> = None;
        for &b in bytes {
            match quote {
                Some(q) => {
                    masked.push(b' ');
                    if b == q {
                        quote = None;
                    }
                }
                None => {
                    if b == b'"' || b == b'\'' {
                        quote = Some(b);
                        masked.push(b' ');
                    } else {
                        masked.push(b);
                    }
                }
            }
        }
        let s = masked;
        for w in s.windows(3) {
            if w[0].is_ascii_digit() && w[1] == b'.' && w[2].is_ascii_digit() {
                return true;
            }
        }
        false
    }

    /// Recursive boolean expression evaluator over an upper-cased condition.
    /// Grammar: expr ::= expr OR expr | expr AND expr | NOT expr
    ///                | '(' expr ')' | %DEF(name) | cmp | atom
    /// where cmp ::= operand (' = ' | ' <> ' | ' >= ' | ' <= ' | ' > ' | ' < ') operand.
    /// AND/OR are only split at paren depth 0, so `(A AND B) = C` keeps its
    /// inner AND inside the operand (handled by eval_operand).
    fn eval_bool_expr(&self, s: &str) -> bool {
        let s = s.trim();
        if s.is_empty() {
            return false;
        }
        // Strip fully balanced outer parentheses (repeat).
        let mut stripped = s.to_string();
        loop {
            if stripped.starts_with('(') && stripped.ends_with(')') {
                let inner = &stripped[1..stripped.len() - 1];
                let mut depth = 0i32;
                let mut ok = true;
                for b in inner.bytes() {
                    match b {
                        b'(' => depth += 1,
                        b')' => {
                            depth -= 1;
                            if depth < 0 {
                                ok = false;
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                if ok && depth == 0 {
                    stripped = inner.to_string();
                    continue;
                }
            }
            break;
        }
        let s = stripped.as_str();

        // OR binds loosest.
        if let Some(parts) = split_top_level(s, " OR ") {
            if parts.len() > 1 {
                return parts.iter().any(|p| self.eval_bool_expr(p));
            }
        }
        // AND binds tighter than OR.
        if let Some(parts) = split_top_level(s, " AND ") {
            if parts.len() > 1 {
                return parts.iter().all(|p| self.eval_bool_expr(p));
            }
        }
        // NOT prefix.
        if let Some(stripped) = s.strip_prefix("NOT ") {
            return !self.eval_bool_expr(stripped);
        }
        // %DEF(%CONSTANT) — defined *in this pass* (plus the built-in SDK
        // defaults).  Matching normalises the leading '%' so keys stored as
        // "%ANY_SIZE" and lookups written "%ANY_SIZE" / "ANY_SIZE" agree.
        // Batch 306: must NOT consult the cross-pass `constants` table, or the
        // collection pass's definitions would suppress the emit pass's
        // guarded `%ANY_SIZE = 1` / `%WIN32_WINNT = ...` definition lines and
        // the parser would never register them (38 parse errors, build44).
        if s.starts_with("%DEF(") {
            if let Some(end) = s.find(')') {
                let const_name = s[5..end].trim().to_string();
                let bare = const_name.trim_start_matches('%');
                return self
                    .defined_this_pass
                    .iter()
                    .any(|k| k.trim_start_matches('%').eq_ignore_ascii_case(bare));
            }
            return false;
        }
        // Comparison forms: "%CONST = 0", "%CONST <> 5", "(X AND Y) = &H1000",
        // ">= " / "<= " / "> " / "< " relational guards.
        // (batch 229: without "= " the %MY_PBVER discriminator blocks evaluate
        // "0 = 0" as the bare-constant truth test "is the %CONST non-zero?" ->
        // false, silently dropping every fork-only statement inside the block,
        // e.g. WINDOW SET/GET TEXT, ARRAY ASSIGN, TYPE SET.)
        for (op, cmpf, cmpi) in [
            (
                " <> ",
                (|a: f64, b: f64| a != b) as fn(f64, f64) -> bool,
                (|a: i64, b: i64| a != b) as fn(i64, i64) -> bool,
            ),
            (
                " = ",
                (|a: f64, b: f64| a == b) as fn(f64, f64) -> bool,
                (|a: i64, b: i64| a == b) as fn(i64, i64) -> bool,
            ),
            (
                " >= ",
                (|a: f64, b: f64| a >= b) as fn(f64, f64) -> bool,
                (|a: i64, b: i64| a >= b) as fn(i64, i64) -> bool,
            ),
            (
                " <= ",
                (|a: f64, b: f64| a <= b) as fn(f64, f64) -> bool,
                (|a: i64, b: i64| a <= b) as fn(i64, i64) -> bool,
            ),
            (
                " > ",
                (|a: f64, b: f64| a > b) as fn(f64, f64) -> bool,
                (|a: i64, b: i64| a > b) as fn(i64, i64) -> bool,
            ),
            (
                " < ",
                (|a: f64, b: f64| a < b) as fn(f64, f64) -> bool,
                (|a: i64, b: i64| a < b) as fn(i64, i64) -> bool,
            ),
        ] {
            if let Some(pos) = find_top_level(s, op) {
                let lhs = &s[..pos];
                let rhs = &s[pos + op.len()..];
                // Arithmetic may produce fractional values (floating-point
                // divide), so compare in f64 when both sides are expressions.
                if let (Some(a), Some(b)) = (self.eval_arith_expr(lhs), self.eval_arith_expr(rhs)) {
                    return cmpf(a, b);
                }
                return cmpi(self.eval_operand(lhs), self.eval_operand(rhs));
            }
        }

        // Handle bare %CONSTANT (true if non-zero)
        if s.starts_with('%') {
            if let Some(&val) = self.constants.get(s) {
                return val != 0;
            }
            return false;
        }

        // Handle expr - expr (non-zero is true)
        // Simple: just check if it's a defined constant
        false
    }

    /// Evaluate one #IF operand: a plain number, an &H hex literal, a
    /// %CONSTANT reference, or a parenthesized bitwise-AND expression such as
    /// `(%PB_REVISION AND &H0FF00)`. Unknown %CONSTANTs evaluate to 0 (same
    /// rule parse_constant_value uses). A top-level split may leave a single
    /// trailing ')' on the right operand, which is stripped here.
    fn eval_operand(&self, s: &str) -> i64 {
        let mut s = s.trim();
        let opens = s.matches('(').count();
        let mut closes = s.matches(')').count();
        while closes > opens && s.ends_with(')') {
            s = &s[..s.len() - 1];
            closes -= 1;
        }
        if s.starts_with('(') && s.ends_with(')') {
            let inner = &s[1..s.len() - 1];
            // Find the top-level "AND" (depth 0) and split on it.
            let mut depth = 0i32;
            let bytes = inner.as_bytes();
            let mut i = 0usize;
            while i < bytes.len() {
                match bytes[i] {
                    b'(' => depth += 1,
                    b')' => depth -= 1,
                    b'A' if depth == 0
                        && inner[i..].starts_with("AND ")
                        && (i == 0 || bytes[i - 1] == b' ' || bytes[i - 1] == b'\t') =>
                    {
                        return self.eval_operand(&inner[..i]) & self.eval_operand(&inner[i + 4..]);
                    }
                    _ => {}
                }
                i += 1;
            }
            return self.eval_operand(inner);
        }
        // Strip unmatched leading '(' / trailing ')' — a comparison split like
        // `(%WIN32_IE >= &H0300)` yields operands "(%WIN32_IE" and "&H0300)"
        // (batch306).  Also normalises `(X) = Y` style operands.
        let bytes = s.as_bytes();
        let mut start = 0usize;
        let mut end = s.len();
        while start < end && bytes[start] == b'(' {
            start += 1;
        }
        while end > start && bytes[end - 1] == b')' {
            end -= 1;
        }
        s = &s[start..end];
        // Arithmetic expression (2 + 3, 17 \ 3, CVQ(".."), ...); a plain
        // constant or number also parses to its own value.
        if let Some(v) = self.eval_arith_expr(s) {
            return v as i64;
        }
        self.parse_constant_value(s)
    }

    fn parse_constant_value(&self, s: &str) -> i64 {
        let s = s.trim();

        // Hex literal
        if s.starts_with("&H") || s.starts_with("&h") {
            let hex = s[2..].trim_end_matches(['&', '%', '?']);
            return i64::from_str_radix(hex, 16).unwrap_or(0);
        }

        // Reference to another constant
        if s.starts_with('%') {
            if let Some(&val) = self.constants.get(&s.to_uppercase()) {
                return val;
            }
            return 0;
        }

        // Simple integer
        s.parse::<i64>().unwrap_or(0)
    }

    /// Evaluate a compile-time arithmetic expression (hash family step 2).
    /// Returns `None` when the text is not an arithmetic expression, so the
    /// caller can fall back to the legacy integer/constant path.
    ///
    /// PBWin #IF semantics reproduced here:
    ///
    /// - `+ - * /` ordinary floating-point arithmetic
    /// - `\` floating-point divide followed by banker's rounding
    ///   (round half to even), NOT run-time integer truncation
    /// - `CVQ("..")` bytes packed little-endian into a 64-bit integer
    ///
    /// Operator precedence: `+,-` < `*,/,\`; parentheses override.
    fn eval_arith_expr(&self, raw: &str) -> Option<f64> {
        let mut p = ArithParser {
            s: raw,
            pos: 0,
            pre: self,
        };
        p.skip_ws();
        let v = p.parse_expr().ok()?;
        p.skip_ws();
        if p.pos != p.s.len() {
            return None;
        }
        Some(v)
    }

    pub fn get_constant(&self, name: &str) -> Option<i64> {
        self.constants.get(&name.to_uppercase()).copied()
    }

    pub fn constants(&self) -> &HashMap<String, i64> {
        &self.constants
    }

    pub fn resources(&self) -> &[(u32, PathBuf)] {
        &self.resources
    }

    pub fn raw_resources(&self) -> &[RawResource] {
        &self.raw_resources
    }

    pub fn string_resources(&self) -> &[StringResource] {
        &self.string_resources
    }

    pub fn res_files(&self) -> &[ResFileEntry] {
        &self.res_files
    }

    pub fn manifests(&self) -> &[(u32, PathBuf)] {
        &self.manifests
    }

    pub fn dim_all(&self) -> bool {
        self.dim_all
    }

    pub fn stack_size(&self) -> Option<u64> {
        self.stack_size
    }

    pub fn console_mode(&self) -> Option<bool> {
        self.console_mode
    }

    pub fn version_info(&self) -> &VersionInfo {
        &self.version_info
    }
}

fn extract_string(s: &str) -> Option<String> {
    let s = s.trim();
    if let Some(stripped) = s.strip_prefix('"') {
        if let Some(end) = stripped.find('"') {
            return Some(stripped[..end].to_string());
        }
    }
    None
}

/// Parse `name(params)` or `name` from the text after the MACRO keyword.
fn parse_macro_proto(rest: &str) -> (String, Vec<String>) {
    let rest = rest.trim();
    if rest.is_empty() {
        return (String::new(), Vec::new());
    }
    let name_end = rest
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .unwrap_or(rest.len());
    let name = rest[..name_end].to_uppercase();
    let tail = rest[name_end..].trim();
    if tail.starts_with('(') {
        if let Some(close) = find_matching_paren(tail) {
            let inner = &tail[1..close];
            let params: Vec<String> = split_args(inner)
                .into_iter()
                .map(|p| p.trim().to_uppercase())
                .filter(|p| !p.is_empty())
                .collect();
            return (name, params);
        }
    }
    (name, Vec::new())
}

/// Find the position of '=' outside any parentheses in `s`.
fn find_eq_outside_parens(s: &str) -> Option<usize> {
    let mut depth = 0i32;
    let mut in_string = false;
    for (i, ch) in s.char_indices() {
        match ch {
            '"' => in_string = !in_string,
            '(' if !in_string => depth += 1,
            ')' if !in_string => depth -= 1,
            '=' if !in_string && depth == 0 => return Some(i),
            _ => {}
        }
    }
    None
}

/// Given `s` starting with '(', return the index of the matching ')'.
fn find_matching_paren(s: &str) -> Option<usize> {
    let mut depth = 0i32;
    let mut in_string = false;
    for (i, ch) in s.char_indices() {
        match ch {
            '"' => in_string = !in_string,
            '(' if !in_string => depth += 1,
            ')' if !in_string => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

/// Split a comma-separated argument list (depth 0 commas only).
fn split_args(inner: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut in_string = false;
    let mut cur = String::new();
    for ch in inner.chars() {
        match ch {
            '"' => {
                in_string = !in_string;
                cur.push(ch);
            }
            '(' if !in_string => {
                depth += 1;
                cur.push(ch);
            }
            ')' if !in_string => {
                depth -= 1;
                cur.push(ch);
            }
            ',' if !in_string && depth == 0 => {
                out.push(cur.trim().to_string());
                cur = String::new();
            }
            _ => cur.push(ch),
        }
    }
    if !cur.trim().is_empty() {
        out.push(cur.trim().to_string());
    }
    out
}

/// Extract the parenthesised argument list text after `name` in `s`.
/// Returns the args split at depth-0 commas.
fn extract_paren_args(s: &str, name_len: usize) -> Vec<String> {
    let tail = &s[name_len..];
    let tail = tail.trim_start();
    if tail.starts_with('(') {
        if let Some(close) = find_matching_paren(tail) {
            return split_args(&tail[1..close]);
        }
    }
    Vec::new()
}

/// Replace occurrences of parameter `param` (word-boundary) with `arg`.
fn replace_word(body: &str, param: &str, arg: &str) -> String {
    let mut out = String::new();
    let bytes = body.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i] as char;
        if c.is_ascii_alphanumeric() || c == '_' {
            let start = i;
            while i < bytes.len()
                && ((bytes[i] as char).is_ascii_alphanumeric()
                    || bytes[i] == b'_'
                    || bytes[i] == b'$')
            {
                i += 1;
            }
            let word = &body[start..i];
            if word.to_uppercase() == param {
                out.push_str(arg);
            } else {
                out.push_str(word);
            }
        } else {
            out.push(c);
            i += 1;
        }
    }
    out
}

/// Split `s` at top-level (paren depth 0) occurrences of `sep`, returning the
/// pieces when at least one split happened.  Parentheses (including %DEF(...))
/// keep their interiors from being split.
/// Round half to even (banker's rounding), the PBWin #IF `\` semantics.
/// PBWin performs a floating-point divide then rounds .5 to the nearest even
/// integer rather than always away from zero.
fn banker_round(x: f64) -> f64 {
    if !x.is_finite() {
        return x;
    }
    let floor = x.floor();
    let diff = x - floor;
    if diff < 0.5 {
        floor
    } else if diff > 0.5 {
        floor + 1.0
    } else {
        // exactly halfway: pick the even neighbour
        let fl = floor as i64;
        if fl % 2 == 0 {
            floor
        } else {
            floor + 1.0
        }
    }
}

/// Recursive-descent parser for #IF compile-time arithmetic. The input is
/// already upper-cased. Whitespace is allowed around tokens.
struct ArithParser<'a> {
    s: &'a str,
    pos: usize,
    pre: &'a Preprocessor,
}

impl<'a> ArithParser<'a> {
    fn skip_ws(&mut self) {
        while self.pos < self.s.len() {
            let b = self.s.as_bytes()[self.pos];
            if b == b' ' || b == b'\t' {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn peek(&self) -> Option<u8> {
        self.s.as_bytes().get(self.pos).copied()
    }

    // expr := term (('+' | '-') term)*
    fn parse_expr(&mut self) -> Result<f64, ()> {
        let mut v = self.parse_term()?;
        loop {
            self.skip_ws();
            match self.peek() {
                Some(b'+') => {
                    self.pos += 1;
                    v += self.parse_term()?;
                }
                Some(b'-') => {
                    self.pos += 1;
                    v -= self.parse_term()?;
                }
                _ => break,
            }
        }
        Ok(v)
    }

    // term := factor (('*' | '/' | '\') factor)*
    fn parse_term(&mut self) -> Result<f64, ()> {
        let mut v = self.parse_factor()?;
        loop {
            self.skip_ws();
            match self.peek() {
                Some(b'*') => {
                    self.pos += 1;
                    v *= self.parse_factor()?;
                }
                Some(b'/') => {
                    // PBWin #IF: both / and \ are a floating-point divide
                    // followed immediately by banker's rounding; they are
                    // equivalent at compile time (verified against PBWin 10).
                    self.pos += 1;
                    let r = self.parse_factor()?;
                    v = banker_round(v / r);
                }
                Some(b'\\') => {
                    self.pos += 1;
                    let r = self.parse_factor()?;
                    v = banker_round(v / r);
                }
                _ => break,
            }
        }
        Ok(v)
    }

    // factor := unary '-' factor | atom
    fn parse_factor(&mut self) -> Result<f64, ()> {
        self.skip_ws();
        if self.peek() == Some(b'-') {
            self.pos += 1;
            return Ok(-self.parse_factor()?);
        }
        if self.peek() == Some(b'+') {
            self.pos += 1;
            return self.parse_factor();
        }
        self.parse_atom()
    }

    // atom := CVQ('(' string ')') | '(' expr ')' | hex | integer | %constant
    fn parse_atom(&mut self) -> Result<f64, ()> {
        self.skip_ws();
        let bytes = self.s.as_bytes();
        // CVQ("...")
        if self.s[self.pos..].starts_with("CVQ(") {
            self.pos += 4;
            self.skip_ws();
            // read a quoted string literal
            let q = self.peek().ok_or(())?;
            if q != b'"' && q != b'\'' {
                return Err(());
            }
            self.pos += 1;
            let start = self.pos;
            while self.pos < self.s.len() && bytes[self.pos] != q {
                self.pos += 1;
            }
            let lit = &self.s[start..self.pos];
            if self.pos >= self.s.len() {
                return Err(());
            }
            self.pos += 1; // closing quote
            self.skip_ws();
            if self.peek() != Some(b')') {
                return Err(());
            }
            self.pos += 1;
            // little-endian byte packing (up to 8 bytes, high bytes zero)
            let mut val: i64 = 0;
            for (i, c) in lit.bytes().take(8).enumerate() {
                val |= (c as i64) << (8 * i);
            }
            return Ok(val as f64);
        }
        // parenthesised sub-expression
        if self.peek() == Some(b'(') {
            self.pos += 1;
            let v = self.parse_expr()?;
            self.skip_ws();
            if self.peek() != Some(b')') {
                return Err(());
            }
            self.pos += 1;
            return Ok(v);
        }
        // hex literal &H....
        if self.s[self.pos..].starts_with("&H") {
            let start = self.pos + 2;
            let mut e = start;
            while e < self.s.len() {
                let b = bytes[e];
                if b.is_ascii_hexdigit() {
                    e += 1;
                } else {
                    break;
                }
            }
            let hex = &self.s[start..e];
            let val = i64::from_str_radix(hex, 16).map_err(|_| ())?;
            self.pos = e;
            // skip an optional trailing type suffix
            if matches!(self.peek(), Some(b'&') | Some(b'%') | Some(b'?')) {
                self.pos += 1;
            }
            return Ok(val as f64);
        }
        // %constant reference
        if self.peek() == Some(b'%') {
            let start = self.pos;
            self.pos += 1;
            while self.pos < self.s.len() {
                let b = bytes[self.pos];
                if b.is_ascii_alphanumeric() || b == b'_' {
                    self.pos += 1;
                } else {
                    break;
                }
            }
            let name = &self.s[start..self.pos];
            return Ok(self.pre.parse_constant_value(name) as f64);
        }
        // plain integer
        let start = self.pos;
        while self.pos < self.s.len() && bytes[self.pos].is_ascii_digit() {
            self.pos += 1;
        }
        if self.pos == start {
            return Err(());
        }
        let txt = &self.s[start..self.pos];
        txt.parse::<i64>().map_err(|_| ()).map(|v| v as f64)
    }
}

fn split_top_level<'a>(s: &'a str, sep: &str) -> Option<Vec<&'a str>> {
    let bytes = s.as_bytes();
    let mut depth = 0i32;
    let mut starts: Vec<usize> = vec![0];
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'(' => depth += 1,
            b')' => depth -= 1,
            _ => {
                if depth == 0 && s[i..].starts_with(sep) {
                    starts.push(i + sep.len());
                    i += sep.len();
                    continue;
                }
            }
        }
        i += 1;
    }
    if starts.len() < 2 {
        return None;
    }
    starts.push(s.len());
    Some(starts.windows(2).map(|w| &s[w[0]..w[1]]).collect())
}

/// Find `needle` at top-level (paren depth 0).  Used to locate comparison
/// operators without splitting inside nested parentheses.
fn find_top_level(s: &str, needle: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut depth = 0i32;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'(' => depth += 1,
            b')' => depth -= 1,
            _ => {
                if depth == 0 && s[i..].starts_with(needle) {
                    return Some(i);
                }
            }
        }
        i += 1;
    }
    None
}

/// Substitute macro parameters into a body line.
fn substitute_params(body: &str, params: &[String], args: &[String]) -> String {
    if params.is_empty() {
        return body.to_string();
    }
    let mut result = body.to_string();
    for (idx, p) in params.iter().enumerate() {
        let arg = args.get(idx).map(|s| s.as_str()).unwrap_or("");
        result = replace_word(&result, p, arg);
    }
    result
}

fn resolve_include_path(
    current_file: &Path,
    include_path: &str,
    include_dirs: &[PathBuf],
) -> PathBuf {
    // Convert backslashes to forward slashes for cross-platform
    let normalized = include_path.replace('\\', "/");
    // Absolute path or drive-qualified path: use it directly.
    if include_path.contains(':') || Path::new(&normalized).is_absolute() {
        return PathBuf::from(&normalized);
    }
    let parent = current_file.parent().unwrap_or_else(|| Path::new("."));
    // A name containing a directory separator is a project-relative path:
    // resolve it against the including file's directory.
    if normalized.contains('/') {
        return parent.join(&normalized);
    }
    // CHW #INCLUDE: "The compiler scans the search path for each #INCLUDE
    // file before checking the current (default) directory."  Batch 276:
    // official samples do `#INCLUDE ONCE "Win32api.inc"`; the file lives in
    // the PBWin10 WINAPI folder, not next to the source.  Auto-detect the
    // installed PBWin include directory (same spirit as the Windows SDK lib
    // auto-detection) and search it BEFORE the local directory, so a
    // same-named local file (e.g. the repo's 70-byte win32api.inc stub)
    // cannot hijack the official SDK header (batch 306).
    // User-supplied `--include-dir` roots are consulted first, in the order
    // given on the command line (msg1413: headers installed outside the
    // hard-coded PBWin roots still resolve).
    for dir in include_dirs {
        let p = dir.join(&normalized);
        if p.exists() {
            return p;
        }
    }
    for cand in [
        "C:\\PBWin10\\WINAPI",
        "C:\\PBWin10\\WINAPI\\INC",
        "C:\\PBWin9\\WINAPI",
    ] {
        let p = std::path::Path::new(cand).join(&normalized);
        if p.exists() {
            return p;
        }
    }
    parent.join(&normalized)
}

fn strip_comment(s: &str) -> String {
    // Find a comment marker that's not inside a string
    let mut in_string = false;
    for (i, ch) in s.char_indices() {
        if ch == '"' {
            in_string = !in_string;
        } else if ch == '\'' && !in_string {
            return s[..i].trim().to_string();
        }
    }
    s.trim().to_string()
}

/// Rewrite a `DATA ...` line so every *unquoted* item becomes a quoted string
/// literal, keeping its text verbatim (CHW DATA_statement.htm: unquoted items
/// are trimmed of leading/trailing spaces but otherwise preserved).  The
/// lexer uppercases bare identifiers, which would corrupt items like `plain`;
/// quoting them sidesteps that.  Quoted items are left untouched (they keep
/// their spaces).  An unquoted `:` ends the data list (statement separator,
/// CHW); anything after it is left as-is.
fn rewrite_data_line(line: &str) -> String {
    let head = "DATA";
    let rest = line[head.len()..].trim_start();
    let mut out = String::new();
    out.push_str(head);
    let mut data_part_end = rest.len();
    let mut in_string = false;
    for (i, ch) in rest.char_indices() {
        if ch == '"' {
            in_string = !in_string;
        } else if ch == ':' && !in_string {
            data_part_end = i;
            break;
        }
    }
    let data = &rest[..data_part_end];
    let tail = &rest[data_part_end..];
    let mut items: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut in_q = false;
    for ch in data.chars() {
        if ch == '"' {
            in_q = !in_q;
            cur.push(ch);
        } else if ch == ',' && !in_q {
            items.push(cur.clone());
            cur.clear();
        } else {
            cur.push(ch);
        }
    }
    if !cur.is_empty() || !items.is_empty() {
        items.push(cur);
    }
    let mut first = true;
    for item in &items {
        let it = item.trim();
        if it.is_empty() {
            continue;
        }
        if !first {
            out.push_str(", ");
        }
        first = false;
        if it.starts_with('"') && it.ends_with('"') && it.len() >= 2 {
            out.push_str(it);
        } else {
            out.push('"');
            out.push_str(it);
            out.push('"');
        }
    }
    if first {
        out.push(' ');
    }
    if !tail.is_empty() {
        out.push_str(tail);
    }
    out
}
