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

/// Replace every occurrence of `$NAME` (or `$$NAME`) in `text` with `value`,
/// guarding against matching a longer identifier (`$NAME_EXTRA` stays intact).
fn substitute_equate_text(text: &str, name: &str, value: &str) -> String {
    let bytes = text.as_bytes();
    let nlen = name.len();
    let mut out = String::with_capacity(text.len());
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'$' && text[i..].starts_with(name) {
            let before_ok = i == 0
                || !(bytes[i - 1].is_ascii_alphanumeric()
                    || bytes[i - 1] == b'_'
                    || bytes[i - 1] == b'$');
            let after_ok = i + nlen >= bytes.len()
                || !(bytes[i + nlen].is_ascii_alphanumeric()
                    || bytes[i + nlen] == b'_'
                    || bytes[i + nlen] == b'$');
            if before_ok && after_ok {
                out.push_str(value);
                i += nlen;
                continue;
            }
        }
        // Copy one UTF-8 char
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
    macros: HashMap<String, MacroDef>,
    pub resources: Vec<(u32, PathBuf)>,
    /// #RESOURCE MANIFEST, id, "file.xml" — embedded as RT_MANIFEST so the
    /// EXE gets comctl32 v6 visual styles (flat buttons) like a PBWin build.
    pub manifests: Vec<(u32, PathBuf)>,
    pub version_info: VersionInfo,
    /// Official PB: `#DIM ALL` == `OPTION EXPLICIT` (every variable must be
    /// declared). Recorded here so main.rs can enforce it at codegen time.
    dim_all: bool,
}

impl Default for Preprocessor {
    fn default() -> Self {
        Self::new()
    }
}

impl Preprocessor {
    pub fn new() -> Self {
        Preprocessor {
            constants: HashMap::new(),
            string_constants: HashMap::new(),
            included: HashSet::new(),
            macros: HashMap::new(),
            resources: Vec::new(),
            manifests: Vec::new(),
            version_info: VersionInfo::default(),
            dim_all: false,
        }
    }

    /// Process a source file, returning all resolved lines (with includes inlined).
    pub fn process_file(&mut self, path: &Path) -> PbResult<Vec<SourceLine>> {
        let canonical = std::fs::canonicalize(path)
            .map_err(|e| PbError::io(format!("Cannot resolve path {}: {}", path.display(), e)))?;

        if self.included.contains(&canonical) {
            return Ok(Vec::new()); // cycle detection
        }
        self.included.insert(canonical.clone());

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

        while i < raw_lines.len() {
            let line_num = i + 1;
            let raw = raw_lines[i];
            let trimmed = raw.trim();

            // Check if we're inside an inactive #IF block
            let active = if_stack.iter().all(|&b| b);

            // Handle #IF / #ELSEIF / #ELSE / #ENDIF
            let upper = trimmed.to_uppercase();
            if upper.starts_with("#IF ") || upper.starts_with("#IF\t") {
                let condition = self.evaluate_if_condition(trimmed[3..].trim());
                if active {
                    if_stack.push(condition);
                } else {
                    if_stack.push(false); // nested in inactive block
                }
                i += 1;
                continue;
            }
            if upper == "#ELSE" {
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
                let cond = self.evaluate_if_condition(trimmed[8..].trim());
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
            if upper == "#ENDIF" {
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
                let code_part = strip_trailing_comment(&full_line);
                let trimmed_code = code_part.trim_end();
                if !trimmed_code.ends_with('_') || trimmed_code.len() <= 1 {
                    break;
                }
                // Remove the trailing underscore from the code (not the comment)
                let before_underscore = &trimmed_code[..trimmed_code.len() - 1];
                full_line = before_underscore.to_string();
                i += 1;
                if i < raw_lines.len() {
                    full_line.push(' ');
                    full_line.push_str(raw_lines[i].trim());
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

            // #CONSOLE OFF / #CONSOLE ON are legal PB directives; accept and skip
            // them.  The old console_off flag (batch 129) was removed in
            // batch 250: `?` always lowers to MSGBOX regardless of console mode.
            if upper_full.starts_with("#CONSOLE OFF") {
                i += 1;
                continue;
            }
            if upper_full.starts_with("#CONSOLE ON") {
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
                    let resolved = resolve_include_path(file, &inc_path);
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

            // Regular line — emit it (prepend active PREFIX source code, batch 26)
            // DATA rewrite (batch 275): the lexer uppercases bare identifiers,
            // which corrupts unquoted DATA items (CHW: unquoted items are trimmed
            // of leading/trailing spaces but keep their text).  Rewrite unquoted
            // items as quoted string literals so the lexer keeps them verbatim.
            let data_rewritten: Option<String> =
                if upper_full.starts_with("DATA ") || upper_full == "DATA" {
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
            let map: Vec<(String, String)> = self
                .string_constants
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            for sl in lines.iter_mut() {
                let t = sl.text.trim_start();
                if t.starts_with('$') && sl.text.contains('=') {
                    continue; // definition line
                }
                let mut out = sl.text.clone();
                for (name, value) in &map {
                    out = substitute_equate_text(&out, name, value);
                }
                sl.text = out;
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
        // Single-line macros: anywhere in the line.
        let mut result = full.to_string();
        for (name, def) in &self.macros {
            if def.body.len() == 1 {
                result = replace_macro_refs(&result, name, def);
            }
        }
        result
    }

    fn evaluate_if_condition(&self, condition: &str) -> bool {
        let cond = condition.trim().to_uppercase();

        // Handle NOT %DEF(%WINAPI) etc.
        if let Some(stripped) = cond.strip_prefix("NOT ") {
            return !self.evaluate_if_condition(stripped);
        }

        // Comparison forms: "%CONST = 0", "%CONST <> 5", "(X AND Y) = &H1000".
        // (batch 229: without these the %MY_PBVER discriminator blocks evaluate
        // "0 = 0" as the bare-constant truth test "is the %CONST non-zero?" ->
        // false, silently dropping every fork-only statement inside the block,
        // e.g. WINDOW SET/GET TEXT, ARRAY ASSIGN, TYPE SET.)
        if let Some(eq) = cond.find(" = ") {
            return self.eval_operand(&cond[..eq]) == self.eval_operand(&cond[eq + 3..]);
        }
        if let Some(ne) = cond.find(" <> ") {
            return self.eval_operand(&cond[..ne]) != self.eval_operand(&cond[ne + 4..]);
        }

        // Handle %DEF(%CONSTANT)
        if cond.starts_with("%DEF(") {
            if let Some(end) = cond.find(')') {
                let const_name = cond[5..end].trim().to_string();
                return self.constants.contains_key(&const_name);
            }
            return false;
        }

        // Handle bare %CONSTANT (true if non-zero)
        if cond.starts_with('%') {
            if let Some(&val) = self.constants.get(&cond) {
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

    pub fn get_constant(&self, name: &str) -> Option<i64> {
        self.constants.get(&name.to_uppercase()).copied()
    }

    pub fn constants(&self) -> &HashMap<String, i64> {
        &self.constants
    }

    pub fn resources(&self) -> &[(u32, PathBuf)] {
        &self.resources
    }

    pub fn manifests(&self) -> &[(u32, PathBuf)] {
        &self.manifests
    }

    pub fn dim_all(&self) -> bool {
        self.dim_all
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

/// Replace single-line macro references (NAME or NAME(args)) in `line`.
fn replace_macro_refs(line: &str, name: &str, def: &MacroDef) -> String {
    let name_upper = name.to_uppercase();
    let body = def.body.first().cloned().unwrap_or_default();
    let mut out = String::new();
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i] as char;
        if c.is_ascii_alphabetic() || c == '_' {
            let start = i;
            while i < bytes.len()
                && ((bytes[i] as char).is_ascii_alphanumeric()
                    || bytes[i] == b'_'
                    || bytes[i] == b'$')
            {
                i += 1;
            }
            let word = &line[start..i];
            if word.to_uppercase() == name_upper {
                let mut j = i;
                while j < bytes.len() && (bytes[j] as char).is_ascii_whitespace() {
                    j += 1;
                }
                if j < bytes.len() && bytes[j] == b'(' {
                    if let Some(close_rel) = find_matching_paren(&line[j..]) {
                        let close = j + close_rel;
                        let args = split_args(&line[j + 1..close]);
                        out.push_str(&substitute_params(&body, &def.params, &args));
                        i = close + 1;
                        continue;
                    }
                }
                out.push_str(&substitute_params(&body, &def.params, &[]));
                continue;
            }
            out.push_str(word);
        } else {
            out.push(c);
            i += 1;
        }
    }
    out
}

fn resolve_include_path(current_file: &Path, include_path: &str) -> PathBuf {
    // Convert backslashes to forward slashes for cross-platform
    let normalized = include_path.replace('\\', "/");
    let parent = current_file.parent().unwrap_or_else(|| Path::new("."));
    let local = parent.join(&normalized);
    if local.exists() {
        return local;
    }
    // CHW #INCLUDE: "The compiler scans the search path for each #INCLUDE
    // file before checking the current (default) directory."  Batch 276:
    // official samples do `#INCLUDE ONCE "Win32api.inc"`; the file lives in
    // the PBWin10 WINAPI folder, not next to the source.  Auto-detect the
    // installed PBWin include directory (same spirit as the Windows SDK lib
    // auto-detection) and fall back to it so official samples compile as-is.
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
    local
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
