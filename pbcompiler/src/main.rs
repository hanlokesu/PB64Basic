// =====================================================================
// Modified by the hanlokesu/PB64Basic fork (https://github.com/hanlokesu/PB64Basic).
// Derived from benstopics/powerbasilisk (https://github.com/benstopics/powerbasilisk), Apache-2.0.
// See LICENSE for the full license terms.
// =====================================================================
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process;

use pb::error::{PbError, PbResult};
use pb::lexer::Lexer;
use pb::parser::Parser;
use pb::preprocessor::Preprocessor;

mod codegen;
mod llvm_ir;
mod resfile;
mod symbols;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 || args[1] != "build" {
        eprintln!("Usage: PB64Basic build <file.bas> [-o output] [--dll] [--exe] [--obj-only]");
        eprintln!("       [--session-struct]");
        eprintln!("       [--emit-llvm] [--parse-only] [--runtime-lib path] [--lib-dir path]");
        eprintln!("       [--include-dir path] [--debug] [--target triple] [--console]");
        process::exit(2);
    }

    let file_path = &args[2];
    // Validate every token up front and collect any bare positional ".obj"
    // (runtime library). Fails loudly on unknown arguments instead of silently
    // ignoring them (msg1404).
    let positional_obj = match scan_positional_runtime(&args) {
        Ok(obj) => obj,
        Err(msg) => {
            eprintln!("[PB64Basic] Error: {}", msg);
            process::exit(2);
        }
    };
    let dll_mode = args.iter().any(|a| a == "--dll");
    // Source-level `#COMPILE DLL` (official PBWin form) implies DLL mode even
    // without --dll on the command line (batch 276: official DLL samples use
    // `#COMPILE DLL` only; run_examples/release flows pass no --dll).
    // `#COMPILE SLL` (batch 277) is a *Static Link Library*: it must NOT be
    // built as a DLL — the object is archived into <name>.SLL instead.
    let mut sll_mode = false;
    let mut sll_name: Option<String> = None;
    let mut exe_name: Option<(String, bool)> = None;
    let mut dll_name: Option<(String, bool)> = None;
    // Scan the first 64 lines for the single #COMPILE directive. The parser
    // returns the case-preserved name and whether the exact-name '=' is used.
    let compile_dir = std::fs::read_to_string(file_path)
        .ok()
        .and_then(|s| s.lines().take(64).find_map(parse_compile_directive));
    let src_has_compile_dll = if let Some(cd) = &compile_dir {
        match cd.kind.as_str() {
            "SLL" => {
                sll_mode = true;
                if !cd.name.is_empty() {
                    sll_name = Some(cd.name.clone());
                }
                false
            }
            "DLL" => {
                if !cd.name.is_empty() {
                    dll_name = Some((cd.name.clone(), cd.has_eq));
                }
                true
            }
            _ => {
                if !cd.name.is_empty() {
                    exe_name = Some((cd.name.clone(), cd.has_eq));
                }
                false
            }
        }
    } else {
        false
    };
    let dll_mode = dll_mode || (src_has_compile_dll && !sll_mode);
    // A source-level `#COMPILE DLL` (or --dll) wins over the EXE link: a DLL
    // source must never also run the EXE link (undefined `main` from libcmt.exe,
    // batch 276b).
    // Building a .bas links a runnable EXE by default, matching PBWin's
    // "build produces a runnable program" behavior (an explicit --exe is
    // accepted and equivalent). The link is skipped for DLL/SLL targets and
    // for --obj-only / --parse-only / --emit-llvm, which intentionally stop at
    // an object/IR. Previously the EXE link required --exe and a bare runtime
    // .obj was silently ignored, so a build reported success with no EXE
    // (msg1404).
    let obj_only = args.iter().any(|a| a == "--obj-only");
    let exe_mode = !dll_mode
        && !sll_mode
        && !obj_only
        && !args
            .iter()
            .any(|a| a == "--parse-only" || a == "--emit-llvm");
    let session_mode = args.iter().any(|a| a == "--session-struct");
    let parse_only = args.iter().any(|a| a == "--parse-only");
    let emit_llvm = args.iter().any(|a| a == "--emit-llvm");
    let debug_mode = args.iter().any(|a| a == "--debug");
    // --console: force the console PE subsystem regardless of the source's
    // #CONSOLE directive (batch 310).
    let cli_console = args.iter().any(|a| a == "--console");

    let output = get_flag_value(&args, "-o").unwrap_or_else(|| {
        let p = Path::new(file_path);
        p.with_extension("obj").to_string_lossy().into_owned()
    });

    // Runtime library: an explicit --runtime-lib wins; otherwise a bare
    // positional ".obj" collected above is used. Two different runtime objects
    // is a hard error rather than a silent choice.
    let runtime_lib = match (get_flag_value(&args, "--runtime-lib"), positional_obj) {
        (Some(flag), Some(bare)) if flag != bare => {
            eprintln!(
                "[PB64Basic] Error: conflicting runtime libraries: --runtime-lib '{}' and bare '{}'.",
                flag, bare
            );
            process::exit(2);
        }
        (Some(flag), _) => Some(flag),
        (None, bare) => bare,
    };
    let target =
        get_flag_value(&args, "--target").unwrap_or_else(|| codegen::DEFAULT_TARGET.to_string());
    let lib_dir = get_flag_value(&args, "--lib-dir").or_else(|| {
        let d = detect_sdk_lib_dir(&target);
        match &d {
            Some(path) => {
                eprintln!(
                    "[PB64Basic] Auto-detected Windows SDK lib dir: {}",
                    path
                )
            }
            None => {
                if exe_mode {
                    eprintln!(
                        "[PB64Basic] Warning: --lib-dir not given and no Windows SDK found under C:\\Program Files (x86)\\Windows Kits\\10\\Lib; linking may fail."
                    );
                }
            }
        }
        d
    });
    let split_threshold = get_flag_value(&args, "--split-threshold")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);
    let include_dirs = collect_include_dirs(&args);

    let mut opts = codegen::CompileOptions {
        dll_mode,
        exe_mode,
        sll_mode,
        sll_name,
        exe_name,
        dll_name,
        session_mode,
        emit_llvm,
        debug_mode,
        runtime_lib,
        lib_dir,
        split_threshold,
        target,
        option_explicit: false,
        stack_size: None,
        console_mode: None,
    };

    match compile_file(
        file_path,
        &output,
        parse_only,
        cli_console,
        &mut opts,
        &include_dirs,
    ) {
        Ok(()) => {
            eprintln!("[PB64Basic] Success: {}", output);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            // Exit with the PBWin10-compatible error code (PBWin.exe returns the
            // compile error number as its exit code, e.g. 477 = syntax error).
            process::exit(e.pbwin_code());
        }
    }
}

/// Get the value following a flag like "--runtime-lib /path/to/file"
fn get_flag_value(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|pos| args.get(pos + 1).cloned())
}

/// True for the PB-style include-directory switch `/I`, either attached
/// (`/IC:\WinAPI`) or separate (`/I C:\WinAPI`).
fn is_slash_i(tok: &str) -> bool {
    tok.len() >= 2 && tok[..2].eq_ignore_ascii_case("/I")
}

/// Collect every `--include-dir <path>` and `/I <path>` / `/I<path>` value,
/// in command-line order.  The switch is repeatable; duplicates are left to
/// the preprocessor to collapse.
fn collect_include_dirs(args: &[String]) -> Vec<String> {
    let mut dirs: Vec<String> = Vec::new();
    let mut i = 3; // args[0]=exe, args[1]=build, args[2]=source
    while i < args.len() {
        let tok = args[i].as_str();
        if tok == "--include-dir" {
            if let Some(v) = args.get(i + 1) {
                dirs.push(v.clone());
            }
            i += 2;
        } else if is_slash_i(tok) {
            if tok.len() == 2 {
                if let Some(v) = args.get(i + 1) {
                    dirs.push(v.clone());
                }
                i += 2;
            } else {
                dirs.push(tok[2..].to_string());
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    dirs
}

// Every CLI flag the parser understands. Value flags consume the token that
// follows them; boolean flags do not. Any positional token outside these sets
// is either a bare ".obj" runtime library or an error - it must never be
// silently swallowed (the bug in msg1404: a bare "pb_runtime_x86.obj" was
// ignored and the build reported success without producing an EXE).
const BOOL_FLAGS: &[&str] = &[
    "--dll",
    "--exe",
    "--session-struct",
    "--parse-only",
    "--emit-llvm",
    "--debug",
    "--console",
    "--obj-only",
];
const VALUE_FLAGS: &[&str] = &[
    "-o",
    "--runtime-lib",
    "--lib-dir",
    "--target",
    "--split-threshold",
    "--include-dir",
];

/// Scan the arguments after the source file. A bare token ending in ".obj" is
/// treated as the runtime library (the natural positional form users type);
/// any other unrecognized token is a hard error rather than a silent no-op.
fn scan_positional_runtime(args: &[String]) -> Result<Option<String>, String> {
    let mut bare_obj: Option<String> = None;
    let mut i = 3; // args[0]=exe, args[1]=build, args[2]=source
    while i < args.len() {
        let tok = &args[i];
        if BOOL_FLAGS.contains(&tok.as_str()) {
            i += 1;
        } else if VALUE_FLAGS.contains(&tok.as_str()) {
            // The next token is this flag's value, not a positional argument.
            i += 2;
        } else if is_slash_i(tok) {
            // PB-style include dir: `/I <dir>` (separate) or `/I<dir>`
            // (attached). The value is collected by collect_include_dirs();
            // here it is only recognised so it is not reported as unknown.
            i += if tok.len() == 2 { 2 } else { 1 };
        } else if tok.to_ascii_lowercase().ends_with(".obj") {
            if bare_obj.is_some() {
                return Err(format!(
                    "More than one positional runtime object given: '{}' and '{}'. Pass a single runtime .obj, or use --runtime-lib.",
                    bare_obj.as_deref().unwrap_or(""),
                    tok
                ));
            }
            bare_obj = Some(tok.clone());
            i += 1;
        } else {
            return Err(format!(
                "Unrecognized command-line argument: '{}'. Check the flag spelling (runtime objects use --runtime-lib, or a bare .obj).",
                tok
            ));
        }
    }
    Ok(bare_obj)
}

/// One parsed `#COMPILE {EXE|DLL|SLL} [=] ["name"]` directive.
struct CompileDirective {
    kind: String, // "EXE" | "DLL" | "SLL"
    has_eq: bool, // optional '=' present: name is exact, nothing appended
    name: String, // case-preserved target name (may be empty)
}

fn parse_compile_directive(line: &str) -> Option<CompileDirective> {
    let t = line.trim();
    let upper = t.to_uppercase();
    if !upper.starts_with("#COMPILE ") {
        return None;
    }
    let rest = t["#COMPILE ".len()..].trim();
    let kind_end = rest
        .find(|c: char| c.is_whitespace() || c == '=')
        .unwrap_or(rest.len());
    let kind = rest[..kind_end].to_uppercase();
    if kind != "EXE" && kind != "DLL" && kind != "SLL" {
        return None;
    }
    let mut tail = rest[kind_end..].trim();
    let has_eq = tail.starts_with('=');
    if has_eq {
        tail = tail[1..].trim();
    }
    let name = if tail.starts_with('"') || tail.starts_with('\'') {
        let q = tail.as_bytes()[0];
        match tail[1..].find(q as char) {
            Some(c) => tail[1..c + 1].to_string(),
            None => String::new(),
        }
    } else {
        let e = tail.find(char::is_whitespace).unwrap_or(tail.len());
        tail[..e].to_string()
    };
    Some(CompileDirective { kind, has_eq, name })
}
/// Auto-detect the Windows SDK import-library directory so users do not have
/// to pass --lib-dir by hand. Scans the standard install roots and picks the
/// newest installed version whose `um\x64` folder exists.
fn detect_sdk_lib_dir(target: &str) -> Option<String> {
    let roots = [
        r"C:\Program Files (x86)\Windows Kits\10\Lib",
        r"C:\Program Files\Windows Kits\10\Lib",
    ];
    let mut best: Option<(Vec<u64>, String)> = None;
    for root in roots.iter() {
        let root_path = Path::new(root);
        let entries = match std::fs::read_dir(root_path) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            // Version folders look like "10.0.26100.0"; skip non-version names.
            let ver: Vec<u64> = name
                .split('.')
                .filter_map(|s| s.parse::<u64>().ok())
                .collect();
            if ver.is_empty() {
                continue;
            }
            let arch = if target.contains("i686") || target.contains("i386") {
                "x86"
            } else {
                "x64"
            };
            let um_dir = root_path.join(&name).join("um").join(arch);
            if !um_dir.is_dir() {
                continue;
            }
            if best.as_ref().is_none_or(|(bv, _)| ver > *bv) {
                best = Some((ver, um_dir.to_string_lossy().into_owned()));
            }
        }
    }
    best.map(|(_, path)| path)
}

fn compile_file(
    path: &str,
    output: &str,
    parse_only: bool,
    force_console: bool,
    opts: &mut codegen::CompileOptions,
    include_dirs: &[String],
) -> PbResult<()> {
    let path = Path::new(path);

    // Phase 1: Preprocess
    let t0 = std::time::Instant::now();
    let mut preprocessor = Preprocessor::new();
    preprocessor.set_target(&opts.target);
    preprocessor.set_build_mode(opts.exe_mode, opts.dll_mode, opts.sll_mode);
    for dir in include_dirs {
        preprocessor.add_include_dir(dir);
    }
    let source_lines = preprocessor.preprocess(path)?;
    eprintln!(
        "[PB64Basic] Preprocessed {} lines ({:.1}s)",
        source_lines.len(),
        t0.elapsed().as_secs_f64()
    );
    // Batch306 debug: PB64DUMP_PP=<file> writes the preprocessed text (one
    // line per entry) so combined-stream line numbers can be mapped back.
    if let Ok(f) = std::env::var("PB64DUMP_PP") {
        let text = source_lines
            .iter()
            .map(|s| s.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(&f, text).ok();
    }

    // Reassemble into single source string for lexing
    // Build line map: combined_line (1-based) -> original file line
    let mut combined = String::new();
    let mut line_map: Vec<usize> = Vec::new(); // index 0 = combined line 1
    for sl in &source_lines {
        combined.push_str(&sl.text);
        combined.push('\n');
        line_map.push(sl.line_num);
    }

    // Phase 2: Lex
    let t1 = std::time::Instant::now();
    let mut lexer = Lexer::new(&combined, path.to_str());
    let mut tokens = lexer.tokenize()?;
    // Remap token line numbers from preprocessed line to original file line
    for tok in &mut tokens {
        if tok.line > 0 && tok.line <= line_map.len() {
            tok.line = line_map[tok.line - 1];
        }
    }
    eprintln!(
        "[PB64Basic] Lexed {} tokens ({:.1}s)",
        tokens.len(),
        t1.elapsed().as_secs_f64()
    );

    // Phase 3: Parse
    let t2 = std::time::Instant::now();
    let mut parser = Parser::new(tokens);
    let program = parser.parse()?;
    eprintln!(
        "[PB64Basic] Parsed {} top-level items ({:.1}s)",
        program.items.len(),
        t2.elapsed().as_secs_f64()
    );

    if parser.error_count > 0 {
        // Append the first error message so pbwin_code() can resolve the precise
        // PBWin expected-code (411..459) instead of always falling back to 477.
        let suffix = parser
            .first_error_message
            .map(|m| format!(" First error: {}", m))
            .unwrap_or_default();
        return Err(pb::error::PbError::parser(
            format!("Build failed with {} parse error(s). Search for unexpected tokens in your .bas file.{}",
                    parser.error_count, suffix),
            None, 0));
    }

    // #DIM ALL (official meta-directive) == OPTION EXPLICIT. The parser only
    // sees the OPTION EXPLICIT statement form; the preprocessor records #DIM ALL.
    opts.option_explicit = parser.option_explicit || preprocessor.dim_all();
    // #STACK num_expr (batch 299): requested runtime stack size; the
    // preprocessor rounds it up (64 Kb boundary, min 128 Kb). Passed through
    // to the linker as /STACK:bytes in codegen::link_exe.
    opts.stack_size = preprocessor.stack_size();
    // #CONSOLE ON/OFF (batch 310): the source directive selects the PE
    // subsystem at link time; an explicit --console on the command line
    // overrides the source.
    opts.console_mode = preprocessor.console_mode();
    if force_console {
        opts.console_mode = Some(true);
    }

    if parse_only {
        eprintln!("[PB64Basic] Parse-only mode, skipping codegen");
        return Ok(());
    }

    // Phase 4: Codegen
    let t3 = std::time::Instant::now();
    let output_path = Path::new(output);
    let pp_constants: HashMap<String, i64> = preprocessor.constants().clone();
    codegen::compile(&program, output_path, opts, &pp_constants)?;
    eprintln!(
        "[PB64Basic] Codegen complete ({:.1}s)",
        t3.elapsed().as_secs_f64()
    );

    // Phase 4b: #COMPILE SLL → archive the compiled object(s) into a static
    // link library named after the source (ABC.BAS → ABC.SLL), or the
    // explicit `#COMPILE SLL = "name.SLL"` name. The .SLL is a COFF archive
    // (llvm-ar), structurally identical to a .lib, so a host program can
    // `#LINK "ABC.SLL"` and lld pulls the members in (batch 277).
    if opts.sll_mode {
        let mut obj_inputs: Vec<std::path::PathBuf> = Vec::new();
        let base = output_path.with_extension("");
        for cand in ["main.obj", "split.obj", "obj"] {
            let p = output_path.with_extension(cand);
            if p.exists() {
                obj_inputs.push(p);
                break;
            }
        }
        if obj_inputs.is_empty() {
            return Err(PbError::generic(
                "SLL build: no object file produced (expected <name>.obj)",
            ));
        }
        let sll_path = match &opts.sll_name {
            Some(n) => {
                let mut p = PathBuf::from(n);
                if p.extension().is_none() {
                    p.set_extension("sll");
                }
                if p.is_absolute() {
                    p
                } else {
                    output_path.parent().unwrap_or(Path::new(".")).join(p)
                }
            }
            None => base.with_extension("sll"),
        };
        let ar = if std::process::Command::new("llvm-ar")
            .arg("--version")
            .output()
            .is_ok()
        {
            "llvm-ar".to_string()
        } else {
            r"C:\Program Files\LLVM\bin\llvm-ar.exe".to_string()
        };
        let mut cmd = std::process::Command::new(&ar);
        cmd.arg("cr").arg(&sll_path);
        for o in &obj_inputs {
            cmd.arg(o);
        }
        match cmd.output() {
            Ok(out) if out.status.success() => {
                eprintln!(
                    "[PB64Basic] Wrote SLL: {} ({} object(s))",
                    sll_path.display(),
                    obj_inputs.len()
                );
            }
            Ok(out) => {
                return Err(PbError::generic(format!(
                    "SLL archive failed: {}",
                    String::from_utf8_lossy(&out.stderr)
                )));
            }
            Err(e) => {
                return Err(PbError::generic(format!(
                    "SLL archive failed (cannot run {}): {}",
                    ar, e
                )));
            }
        }
    }

    // Phase 5: Embed resources (icons, version info, etc.) into EXE
    if opts.exe_mode {
        let exe_path = codegen::resolve_target_path(output_path, opts.exe_name.as_ref(), "exe");
        if exe_path.exists() {
            let has_icons = !preprocessor.resources().is_empty();
            let has_raw = !preprocessor.raw_resources().is_empty();
            let has_strings = !preprocessor.string_resources().is_empty();
            let has_manifest = !preprocessor.manifests().is_empty();
            let has_version = !preprocessor.version_info().strings.is_empty();
            let has_res_files = !preprocessor.res_files().is_empty();
            if has_icons {
                embed_resources(&exe_path, preprocessor.resources());
            }
            if has_res_files {
                embed_res_files(&exe_path, preprocessor.res_files());
            }
            if has_raw {
                embed_raw_resources(&exe_path, preprocessor.raw_resources());
            }
            if has_strings {
                embed_string_resources(&exe_path, preprocessor.string_resources());
            }
            if has_manifest {
                embed_manifest(&exe_path, preprocessor.manifests());
            }
            if has_version {
                embed_version_info(&exe_path, preprocessor.version_info());
            }
        } else {
            eprintln!(
                "[PB64Basic] Warning: EXE not found for resource embedding: {}",
                exe_path.display()
            );
        }
    }

    Ok(())
}

/// Embed icon resources into the compiled EXE using Win32 UpdateResourceW.
/// Properly splits .ico into RT_ICON (type 3) + RT_GROUP_ICON (type 14).
fn embed_resources(exe_path: &Path, resources: &[(u32, std::path::PathBuf)]) {
    use std::os::windows::process::CommandExt;
    let mut script = String::new();
    script.push_str("Add-Type -TypeDefinition @'\n");
    script.push_str("using System;\n");
    script.push_str("using System.Runtime.InteropServices;\n");
    script.push_str("public class ResEm {\n");
    script.push_str("  [DllImport(\"kernel32.dll\", SetLastError=true, CharSet=CharSet.Unicode)] public static extern IntPtr BeginUpdateResource(string p, bool b);\n");
    script.push_str("  [DllImport(\"kernel32.dll\", SetLastError=true)] public static extern bool UpdateResource(IntPtr h, IntPtr t, IntPtr n, ushort l, byte[] d, uint cb);\n");
    script.push_str("  [DllImport(\"kernel32.dll\", SetLastError=true)] public static extern bool EndUpdateResource(IntPtr h, bool d);\n");
    script.push_str("}\n'@\n");
    let exe_q = exe_path.to_string_lossy().replace('\'', "''");
    script.push_str(&format!("$exe = '{}'\n", exe_q));
    script.push_str("$h = [ResEm]::BeginUpdateResource($exe, $false)\n");
    script.push_str("$RT_ICON = [IntPtr]3\n");
    script.push_str("$RT_GROUP_ICON = [IntPtr]14\n");
    for (id, path) in resources {
        let path_q = path.to_string_lossy().replace('\'', "''");
        script.push_str(&format!("$ico = [IO.File]::ReadAllBytes('{}')\n", path_q));
        script.push_str("$count = [BitConverter]::ToUInt16($ico, 4)\n");
        script.push_str(&format!("$grpId = [IntPtr]{}\n", id));
        script.push_str("$grp = New-Object byte[] (6 + 14*$count)\n");
        script.push_str("[Array]::Copy($ico, 0, $grp, 0, 6)\n");
        script.push_str("for ($i = 0; $i -lt $count; $i++) {\n");
        script.push_str("  $srcOff = 6 + $i * 16\n");
        script.push_str("  $dstOff = 6 + $i * 14\n");
        script.push_str("  [Array]::Copy($ico, $srcOff, $grp, $dstOff, 12)\n");
        script.push_str("  $imgOff = [BitConverter]::ToUInt32($ico, $srcOff + 12)\n");
        script.push_str("  $imgSize = [BitConverter]::ToUInt32($ico, $srcOff + 8)\n");
        script.push_str("  $imgData = New-Object byte[] $imgSize\n");
        script.push_str("  [Array]::Copy($ico, $imgOff, $imgData, 0, $imgSize)\n");
        script.push_str(&format!("  $iconId = [IntPtr]({} + $i + 1)\n", id));
        script.push_str("  [void][ResEm]::UpdateResource($h, $RT_ICON, $iconId, 0, $imgData, $imgData.Length)\n");
        script.push_str("  $grp[$dstOff + 12] = [byte]($iconId.ToInt64() -band 0xFF)\n");
        script.push_str("  $grp[$dstOff + 13] = [byte](($iconId.ToInt64() -shr 8) -band 0xFF)\n");
        script.push_str("}\n");
        script.push_str(
            "[void][ResEm]::UpdateResource($h, $RT_GROUP_ICON, $grpId, 0, $grp, $grp.Length)\n",
        );
    }
    script.push_str("[void][ResEm]::EndUpdateResource($h, $false)\n");

    match std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .creation_flags(0x08000000)
        .output()
    {
        Ok(out) => {
            if out.status.success() {
                eprintln!(
                    "[PB64Basic] Embedded {} icon resource(s) into {}",
                    resources.len(),
                    exe_path.display()
                );
            } else {
                eprintln!(
                    "[PB64Basic] Warning: icon embedding failed: {}",
                    String::from_utf8_lossy(&out.stderr)
                );
            }
        }
        Err(e) => eprintln!(
            "[PB64Basic] Warning: cannot run powershell for icon embedding: {}",
            e
        ),
    }
}

/// Embed #RESOURCE MANIFEST XML as RT_MANIFEST (type 24) with the given id.
/// This is what gives the EXE comctl32 v6 visual styles — flat, themed
/// buttons — exactly like a PBWin build that carries
/// `#RESOURCE MANIFEST, 1, "XPTheme.xml"`.
fn embed_manifest(exe_path: &Path, manifests: &[(u32, std::path::PathBuf)]) {
    use std::os::windows::process::CommandExt;
    let mut script = String::new();
    script.push_str("Add-Type -TypeDefinition @'\n");
    script.push_str("using System;\n");
    script.push_str("using System.Runtime.InteropServices;\n");
    script.push_str("public class ResMf {\n");
    script.push_str("  [DllImport(\"kernel32.dll\", SetLastError=true, CharSet=CharSet.Unicode)] public static extern IntPtr BeginUpdateResource(string p, bool b);\n");
    script.push_str("  [DllImport(\"kernel32.dll\", SetLastError=true)] public static extern bool UpdateResource(IntPtr h, IntPtr t, IntPtr n, ushort l, byte[] d, uint cb);\n");
    script.push_str("  [DllImport(\"kernel32.dll\", SetLastError=true)] public static extern bool EndUpdateResource(IntPtr h, bool d);\n");
    script.push_str("}\n'@\n");
    let exe_q = exe_path.to_string_lossy().replace('\'', "''");
    script.push_str(&format!("$exe = '{}'\n", exe_q));
    script.push_str("$h = [ResMf]::BeginUpdateResource($exe, $false)\n");
    script.push_str("$RT_MANIFEST = [IntPtr]24\n");
    for (id, path) in manifests {
        let path_q = path.to_string_lossy().replace('\'', "''");
        script.push_str(&format!("$xml = [IO.File]::ReadAllText('{}')\n", path_q));
        /* The PBWin sample manifests (XPTheme.xml) declare
        processorArchitecture="x86" because PBWin builds 32-bit EXEs.
        This fork builds x64, and an x64 EXE whose manifest pins the
        comctl32 dependency to x86 fails to start with 0xc000007b.
        Neutralise the architecture so the manifest applies on any
        build. */
        script.push_str(
            "$xml = $xml -replace 'processorArchitecture=\"x86\"', 'processorArchitecture=\"*\"'\n",
        );
        script.push_str("$bytes = [Text.Encoding]::UTF8.GetBytes($xml)\n");
        script.push_str(&format!("[void][ResMf]::UpdateResource($h, $RT_MANIFEST, [IntPtr]{}, 0, $bytes, $bytes.Length)\n", id));
    }
    script.push_str("[void][ResMf]::EndUpdateResource($h, $false)\n");

    match std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .creation_flags(0x08000000)
        .output()
    {
        Ok(out) => {
            if out.status.success() {
                eprintln!(
                    "[PB64Basic] Embedded {} manifest resource(s) into {}",
                    manifests.len(),
                    exe_path.display()
                );
            } else {
                eprintln!(
                    "[PB64Basic] Warning: manifest embedding failed: {}",
                    String::from_utf8_lossy(&out.stderr)
                );
            }
        }
        Err(e) => eprintln!(
            "[PB64Basic] Warning: cannot run powershell for manifest embedding: {}",
            e
        ),
    }
}

/// Embed pre-compiled #RESOURCE RES/PBR files.  Each binary is parsed into
/// its resource entries and every entry is written through Win32
/// UpdateResource, preserving its type (integer or Unicode name), resource
/// id, language and byte-for-byte data.
fn embed_res_files(exe_path: &Path, files: &[pb::preprocessor::ResFileEntry]) {
    use std::os::windows::process::CommandExt;
    let mut script = String::new();
    script.push_str("Add-Type -TypeDefinition @'\n");
    script.push_str("using System;\n");
    script.push_str("using System.Runtime.InteropServices;\n");
    script.push_str("public class ResFile {\n");
    script.push_str("  [DllImport(\"kernel32.dll\", SetLastError=true, CharSet=CharSet.Unicode)] public static extern IntPtr BeginUpdateResource(string p, bool b);\n");
    script.push_str("  [DllImport(\"kernel32.dll\", SetLastError=true, CharSet=CharSet.Unicode)] public static extern bool UpdateResource(IntPtr h, IntPtr t, IntPtr n, ushort l, byte[] d, uint cb);\n");
    script.push_str("  [DllImport(\"kernel32.dll\", SetLastError=true)] public static extern bool EndUpdateResource(IntPtr h, bool d);\n");
    script.push_str("}\n'@\n");
    let exe_q = exe_path.to_string_lossy().replace('\'', "''");
    script.push_str(&format!("$exe = '{}'\n", exe_q));
    script.push_str("$h = [ResFile]::BeginUpdateResource($exe, $false)\n");

    let mut total_entries = 0usize;
    for entry in files {
        let bytes = match std::fs::read(&entry.path) {
            Ok(b) => b,
            Err(e) => {
                eprintln!(
                    "[PB64Basic] Warning: cannot read resource file {}: {}",
                    entry.path.display(),
                    e
                );
                continue;
            }
        };
        let parsed = if entry.is_pbr {
            resfile::parse_pbr(&bytes)
        } else {
            resfile::parse_res(&bytes)
        };
        let parsed = match parsed {
            Ok(p) => p,
            Err(e) => {
                eprintln!(
                    "[PB64Basic] Warning: cannot parse resource file {}: {}",
                    entry.path.display(),
                    e
                );
                continue;
            }
        };
        for item in &parsed {
            total_entries += 1;
            let b64 = base64_encode(&item.data);
            script.push_str(&format!(
                "$bytes = [Convert]::FromBase64String('{}')\n",
                b64
            ));
            // Type pointer: integer RT_* or Unicode string name.
            if let Some(t) = item.type_int {
                script.push_str(&format!("$tp = [IntPtr]{}\n", t));
            } else if let Some(s) = &item.type_str {
                let sq = s.replace('\'', "''");
                script.push_str(&format!(
                    "$tp = [Runtime.InteropServices.Marshal]::StringToHGlobalUni('{}')\n",
                    sq
                ));
            }
            // Name pointer: integer id or Unicode string name.
            if let Some(n) = item.name_int {
                script.push_str(&format!("$nm = [IntPtr]{}\n", n));
            } else if let Some(s) = &item.name_str {
                let sq = s.replace('\'', "''");
                script.push_str(&format!(
                    "$nm = [Runtime.InteropServices.Marshal]::StringToHGlobalUni('{}')\n",
                    sq
                ));
            }
            script.push_str(&format!(
                "[void][ResFile]::UpdateResource($h, $tp, $nm, {}, $bytes, $bytes.Length)\n",
                item.lang
            ));
        }
    }
    script.push_str("[void][ResFile]::EndUpdateResource($h, $false)\n");

    match std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .creation_flags(0x08000000)
        .output()
    {
        Ok(out) => {
            if out.status.success() {
                eprintln!(
                    "[PB64Basic] Embedded {} resource entry/entries from {} file(s) into {}",
                    total_entries,
                    files.len(),
                    exe_path.display()
                );
            } else {
                eprintln!(
                    "[PB64Basic] Warning: RES/PBR embedding failed: {}",
                    String::from_utf8_lossy(&out.stderr)
                );
            }
        }
        Err(e) => eprintln!(
            "[PB64Basic] Warning: cannot run powershell for RES/PBR embedding: {}",
            e
        ),
    }
}

/// Embed file-based #RESOURCE items (BITMAP/RCDATA/WAVE/TYPELIB) via Win32
/// UpdateResource.  Numeric RT_* types are passed as integer pointers;
/// custom WAVE/TYPELIB types are passed as Unicode string-name pointers.
/// BITMAP data drops the 14-byte BITMAPFILEHEADER (RT_BITMAP stores only
/// the BITMAPINFO section).  All mappings match a PBWin10 build.
fn embed_raw_resources(exe_path: &Path, raw: &[pb::preprocessor::RawResource]) {
    use std::os::windows::process::CommandExt;
    let mut script = String::new();
    script.push_str("Add-Type -TypeDefinition @'\n");
    script.push_str("using System;\n");
    script.push_str("using System.Runtime.InteropServices;\n");
    script.push_str("public class ResRaw {\n");
    script.push_str("  [DllImport(\"kernel32.dll\", SetLastError=true, CharSet=CharSet.Unicode)] public static extern IntPtr BeginUpdateResource(string p, bool b);\n");
    script.push_str("  [DllImport(\"kernel32.dll\", SetLastError=true)] public static extern bool UpdateResource(IntPtr h, IntPtr t, IntPtr n, ushort l, byte[] d, uint cb);\n");
    script.push_str("  [DllImport(\"kernel32.dll\", SetLastError=true)] public static extern bool EndUpdateResource(IntPtr h, bool d);\n");
    script.push_str("}\n'@\n");
    let exe_q = exe_path.to_string_lossy().replace('\'', "''");
    script.push_str(&format!("$exe = '{}'\n", exe_q));
    script.push_str("$h = [ResRaw]::BeginUpdateResource($exe, $false)\n");
    for item in raw {
        let path_q = item.path.to_string_lossy().replace('\'', "''");
        script.push_str(&format!("$bytes = [IO.File]::ReadAllBytes('{}')\n", path_q));
        if item.type_kind == pb::preprocessor::RawResType::Bitmap {
            // RT_BITMAP: drop the 14-byte BITMAPFILEHEADER.
            script.push_str("$bytes = $bytes[14..($bytes.Length-1)]\n");
        }
        // Build the type pointer (numeric RT_* or Unicode string name).
        if let Some(num) = item.type_kind.numeric_id() {
            script.push_str(&format!("$tp = [IntPtr]{}\n", num));
        } else if let Some(name) = item.type_kind.type_name() {
            script.push_str(&format!(
                "$tp = [Runtime.InteropServices.Marshal]::StringToHGlobalUni('{}')\n",
                name
            ));
        }
        script.push_str(&format!("$nm = [IntPtr]{}\n", item.id));
        script.push_str("[void][ResRaw]::UpdateResource($h, $tp, $nm, 0, $bytes, $bytes.Length)\n");
    }
    script.push_str("[void][ResRaw]::EndUpdateResource($h, $false)\n");

    match std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .creation_flags(0x08000000)
        .output()
    {
        Ok(out) => {
            if out.status.success() {
                eprintln!(
                    "[PB64Basic] Embedded {} raw resource(s) into {}",
                    raw.len(),
                    exe_path.display()
                );
            } else {
                eprintln!(
                    "[PB64Basic] Warning: raw resource embedding failed: {}",
                    String::from_utf8_lossy(&out.stderr)
                );
            }
        }
        Err(e) => eprintln!(
            "[PB64Basic] Warning: cannot run powershell for raw resource embedding: {}",
            e
        ),
    }
}

/// Group #RESOURCE STRING entries into Windows RT_STRING blocks and embed
/// them.  Each block holds 16 IDs: resource name = (id>>4)+1, slot inside
/// the block = id&15.  Block data is 16 consecutive (u16 length-in-WCHARs
/// followed by that many UTF-16 code units); empty slots have length 0.
/// Layout matches a PBWin10 build byte-for-byte.
fn embed_string_resources(exe_path: &Path, entries: &[pb::preprocessor::StringResource]) {
    use std::os::windows::process::CommandExt;

    // Determine the set of block indices present.
    let mut blocks: std::collections::BTreeSet<u32> = std::collections::BTreeSet::new();
    for e in entries {
        blocks.insert(e.id >> 4);
    }

    let mut script = String::new();
    script.push_str("Add-Type -TypeDefinition @'\n");
    script.push_str("using System;\n");
    script.push_str("using System.Runtime.InteropServices;\n");
    script.push_str("public class ResStr {\n");
    script.push_str("  [DllImport(\"kernel32.dll\", SetLastError=true, CharSet=CharSet.Unicode)] public static extern IntPtr BeginUpdateResource(string p, bool b);\n");
    script.push_str("  [DllImport(\"kernel32.dll\", SetLastError=true)] public static extern bool UpdateResource(IntPtr h, IntPtr t, IntPtr n, ushort l, byte[] d, uint cb);\n");
    script.push_str("  [DllImport(\"kernel32.dll\", SetLastError=true)] public static extern bool EndUpdateResource(IntPtr h, bool d);\n");
    script.push_str("}\n'@\n");
    let exe_q = exe_path.to_string_lossy().replace('\'', "''");
    script.push_str(&format!("$exe = '{}'\n", exe_q));
    script.push_str("$h = [ResStr]::BeginUpdateResource($exe, $false)\n");
    script.push_str("$RT_STRING = [IntPtr]6\n");

    for block in &blocks {
        // Build the 16-slot block in Rust.
        let mut buf: Vec<u8> = Vec::new();
        let mut block_lang: u16 = 1033;
        for slot in 0u32..16 {
            let want_id = block * 16 + slot;
            // Find the entry for this exact id (first match wins).
            let entry = entries.iter().find(|e| e.id == want_id);
            if let Some(en) = entry {
                let wchars: Vec<u16> = en.text.encode_utf16().collect();
                buf.extend_from_slice(&(wchars.len() as u16).to_le_bytes());
                for w in &wchars {
                    buf.extend_from_slice(&w.to_le_bytes());
                }
                block_lang = en.lang_id;
            } else {
                buf.extend_from_slice(&0u16.to_le_bytes());
            }
        }
        let b64 = base64_encode(&buf);
        let block_name = block + 1; // resource name is 1-based block index
        script.push_str(&format!("$b = [Convert]::FromBase64String('{}')\n", b64));
        script.push_str(&format!(
            "[void][ResStr]::UpdateResource($h, $RT_STRING, [IntPtr]{}, {}, $b, $b.Length)\n",
            block_name, block_lang
        ));
    }
    script.push_str("[void][ResStr]::EndUpdateResource($h, $false)\n");

    match std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .creation_flags(0x08000000)
        .output()
    {
        Ok(out) => {
            if out.status.success() {
                eprintln!(
                    "[PB64Basic] Embedded {} string resource(s) in {} block(s) into {}",
                    entries.len(),
                    blocks.len(),
                    exe_path.display()
                );
            } else {
                eprintln!(
                    "[PB64Basic] Warning: string resource embedding failed: {}",
                    String::from_utf8_lossy(&out.stderr)
                );
            }
        }
        Err(e) => eprintln!(
            "[PB64Basic] Warning: cannot run powershell for string resource embedding: {}",
            e
        ),
    }
}

/// Build and embed VS_VERSION_INFO resource into the EXE.
fn embed_version_info(exe_path: &Path, vi: &pb::preprocessor::VersionInfo) {
    use std::os::windows::process::CommandExt;

    // Build the VS_VERSIONINFO binary blob in little-endian UTF-16
    let _buf: Vec<u8> = Vec::new();
    let _pos: usize = 0;

    // We'll compute lengths after building. Use a helper to push aligned UTF-16 strings.
    fn align_to_4(buf: &mut Vec<u8>) {
        while !buf.len().is_multiple_of(4) {
            buf.push(0);
        }
    }
    fn push_u16(buf: &mut Vec<u8>, v: u16) {
        buf.extend_from_slice(&v.to_le_bytes());
    }
    fn push_u32(buf: &mut Vec<u8>, v: u32) {
        buf.extend_from_slice(&v.to_le_bytes());
    }
    fn push_utf16(buf: &mut Vec<u8>, s: &str) {
        for ch in s.encode_utf16() {
            buf.extend_from_slice(&ch.to_le_bytes());
        }
        buf.push(0);
        buf.push(0); // null terminator
    }

    // --- StringFileInfo block ---
    let mut sfi_buf: Vec<u8> = Vec::new();
    // String table header placeholder, fill later
    let _sfi_start = sfi_buf.len();
    push_u16(&mut sfi_buf, 0); // wLength placeholder
    push_u16(&mut sfi_buf, 0); // wValueLength
    push_u16(&mut sfi_buf, 1); // wType = text
    push_utf16(&mut sfi_buf, "StringFileInfo");
    align_to_4(&mut sfi_buf);

    // String table
    let mut st_buf: Vec<u8> = Vec::new();
    let _st_start = st_buf.len();
    push_u16(&mut st_buf, 0);
    push_u16(&mut st_buf, 0);
    push_u16(&mut st_buf, 1);
    let lang_hex = format!("{:04X}{:04X}", vi.lang_id, vi.codepage);
    push_utf16(&mut st_buf, &lang_hex);
    align_to_4(&mut st_buf);

    // String entries
    for (key, val) in &vi.strings {
        let mut se_buf: Vec<u8> = Vec::new();
        push_u16(&mut se_buf, 0); // length placeholder
        let val_w: Vec<u16> = val.encode_utf16().chain(std::iter::once(0)).collect();
        push_u16(&mut se_buf, (val_w.len() * 2) as u16); // wValueLength in bytes
        push_u16(&mut se_buf, 1); // wType = text
        push_utf16(&mut se_buf, key);
        align_to_4(&mut se_buf);
        for w in &val_w {
            se_buf.extend_from_slice(&w.to_le_bytes());
        }
        align_to_4(&mut se_buf);
        // Fix length
        let len = se_buf.len() as u16;
        se_buf[0..2].copy_from_slice(&len.to_le_bytes());
        st_buf.extend_from_slice(&se_buf);
    }

    // Fix string table length
    let st_len = st_buf.len() as u16;
    st_buf[0..2].copy_from_slice(&st_len.to_le_bytes());
    sfi_buf.extend_from_slice(&st_buf);

    // Fix SFI length
    let sfi_len = sfi_buf.len() as u16;
    sfi_buf[0..2].copy_from_slice(&sfi_len.to_le_bytes());

    // --- VarFileInfo block ---
    let mut vfi_buf: Vec<u8> = Vec::new();
    push_u16(&mut vfi_buf, 0);
    push_u16(&mut vfi_buf, 0);
    push_u16(&mut vfi_buf, 0); // wType = binary
    push_utf16(&mut vfi_buf, "VarFileInfo");
    align_to_4(&mut vfi_buf);
    // Var entry
    let mut var_buf: Vec<u8> = Vec::new();
    push_u16(&mut var_buf, 0);
    push_u16(&mut var_buf, 4); // value length = 4 bytes
    push_u16(&mut var_buf, 0); // binary
    push_utf16(&mut var_buf, "Translation");
    align_to_4(&mut var_buf);
    push_u16(&mut var_buf, vi.lang_id);
    push_u16(&mut var_buf, vi.codepage);
    align_to_4(&mut var_buf);
    let var_len = var_buf.len() as u16;
    var_buf[0..2].copy_from_slice(&var_len.to_le_bytes());
    vfi_buf.extend_from_slice(&var_buf);
    let vfi_len = vfi_buf.len() as u16;
    vfi_buf[0..2].copy_from_slice(&vfi_len.to_le_bytes());

    // --- VS_VERSIONINFO top-level ---
    let mut vi_buf: Vec<u8> = Vec::new();
    push_u16(&mut vi_buf, 0); // total length placeholder
    push_u16(&mut vi_buf, 52); // wValueLength = sizeof(VS_FIXEDFILEINFO)
    push_u16(&mut vi_buf, 0); // wType = binary
    push_utf16(&mut vi_buf, "VS_VERSION_INFO");
    align_to_4(&mut vi_buf);

    // VS_FIXEDFILEINFO (52 bytes)
    push_u32(&mut vi_buf, 0xFEEF04BD); // dwSignature
    push_u32(&mut vi_buf, 0x00010000); // dwStrucVersion
    let (a, b, c, d) = vi.file_version;
    push_u32(&mut vi_buf, (a << 16) | b); // dwFileVersionMS
    push_u32(&mut vi_buf, (c << 16) | d); // dwFileVersionLS
    let (pa, pb, pc, pd) = vi.product_version;
    push_u32(&mut vi_buf, (pa << 16) | pb); // dwProductVersionMS
    push_u32(&mut vi_buf, (pc << 16) | pd); // dwProductVersionLS
    push_u32(&mut vi_buf, 0x00000037); // dwFileFlagsMask
    push_u32(&mut vi_buf, 0x0); // dwFileFlags
    push_u32(&mut vi_buf, 0x40004); // dwFileOS = VOS_NT_WINDOWS32
    push_u32(&mut vi_buf, 0x1); // dwFileType = VFT_APP
    push_u32(&mut vi_buf, 0x0); // dwFileSubtype
    push_u32(&mut vi_buf, 0x0); // dwFileDateMS
    push_u32(&mut vi_buf, 0x0); // dwFileDateLS

    align_to_4(&mut vi_buf);
    vi_buf.extend_from_slice(&sfi_buf);
    vi_buf.extend_from_slice(&vfi_buf);

    // Fix total length
    let total_len = vi_buf.len() as u16;
    vi_buf[0..2].copy_from_slice(&total_len.to_le_bytes());

    // Embed via PowerShell UpdateResource
    let b64 = base64_encode(&vi_buf);
    let exe_q = exe_path.to_string_lossy().replace('\'', "''");
    let script = format!(
        r#"
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public class VerRes {{
  [DllImport("kernel32.dll", SetLastError=true, CharSet=CharSet.Unicode)] public static extern IntPtr BeginUpdateResource(string p, bool b);
  [DllImport("kernel32.dll", SetLastError=true)] public static extern bool UpdateResource(IntPtr h, IntPtr t, IntPtr n, ushort l, byte[] d, uint cb);
  [DllImport("kernel32.dll", SetLastError=true)] public static extern bool EndUpdateResource(IntPtr h, bool d);
}}
'@
$exe = '{exe}'
$b64 = '{data}'
$bytes = [Convert]::FromBase64String($b64)
$h = [VerRes]::BeginUpdateResource($exe, $false)
if ($h -eq [IntPtr]::Zero) {{ Write-Error "BeginUpdateResource failed"; exit 1 }}
$RT_VERSION = [IntPtr]16
$RT_VERSION_NAME = [IntPtr]1
[void][VerRes]::UpdateResource($h, $RT_VERSION, $RT_VERSION_NAME, 0, $bytes, $bytes.Length)
[void][VerRes]::EndUpdateResource($h, $false)
Write-Host "Version info embedded: $($bytes.Length) bytes"
"#,
        exe = exe_q,
        data = b64
    );

    match std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .creation_flags(0x08000000)
        .output()
    {
        Ok(out) => {
            if out.status.success() {
                eprintln!(
                    "[PB64Basic] Embedded VERSIONINFO into {}",
                    exe_path.display()
                );
            } else {
                eprintln!(
                    "[PB64Basic] Warning: VERSIONINFO embed failed: {}",
                    String::from_utf8_lossy(&out.stderr)
                );
            }
        }
        Err(e) => eprintln!("[PB64Basic] Warning: VERSIONINFO embed error: {e}"),
    }
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(CHARS[(n >> 18) as usize & 63] as char);
        out.push(CHARS[(n >> 12) as usize & 63] as char);
        if chunk.len() > 1 {
            out.push(CHARS[(n >> 6) as usize & 63] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(CHARS[n as usize & 63] as char);
        } else {
            out.push('=');
        }
    }
    out
}
