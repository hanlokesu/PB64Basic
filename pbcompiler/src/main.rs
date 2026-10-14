// =====================================================================
// Modified by the hanlokesu/powerbasilisk fork (https://github.com/hanlokesu/powerbasilisk).
// Derived from benstopics/powerbasilisk (https://github.com/benstopics/powerbasilisk), Apache-2.0.
// See LICENSE for the full license terms.
// =====================================================================
use std::collections::HashMap;
use std::path::Path;
use std::process;

use pb::error::PbResult;
use pb::lexer::Lexer;
use pb::parser::Parser;
use pb::preprocessor::Preprocessor;

mod codegen;
mod llvm_ir;
mod symbols;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 || args[1] != "build" {
        eprintln!(
            "Usage: pbcompiler build <file.bas> [-o output] [--dll] [--exe] [--session-struct]"
        );
        eprintln!("       [--emit-llvm] [--parse-only] [--runtime-lib path] [--lib-dir path]");
        eprintln!("       [--debug] [--target triple]");
        process::exit(2);
    }

    let file_path = &args[2];
    let dll_mode = args.iter().any(|a| a == "--dll");
    let exe_mode = args.iter().any(|a| a == "--exe");
    let session_mode = args.iter().any(|a| a == "--session-struct");
    let parse_only = args.iter().any(|a| a == "--parse-only");
    let emit_llvm = args.iter().any(|a| a == "--emit-llvm");
    let debug_mode = args.iter().any(|a| a == "--debug");

    let output = get_flag_value(&args, "-o").unwrap_or_else(|| {
        let p = Path::new(file_path);
        p.with_extension("obj").to_string_lossy().into_owned()
    });

    let runtime_lib = get_flag_value(&args, "--runtime-lib");
    let target =
        get_flag_value(&args, "--target").unwrap_or_else(|| codegen::DEFAULT_TARGET.to_string());
    let lib_dir = get_flag_value(&args, "--lib-dir").or_else(|| {
        let d = detect_sdk_lib_dir(&target);
        match &d {
            Some(path) => {
                eprintln!(
                    "[pbcompiler] Auto-detected Windows SDK lib dir: {}",
                    path
                )
            }
            None => {
                if exe_mode {
                    eprintln!(
                        "[pbcompiler] Warning: --lib-dir not given and no Windows SDK found under C:\\Program Files (x86)\\Windows Kits\\10\\Lib; linking may fail."
                    );
                }
            }
        }
        d
    });
    let split_threshold = get_flag_value(&args, "--split-threshold")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);

    let mut opts = codegen::CompileOptions {
        dll_mode,
        exe_mode,
        session_mode,
        emit_llvm,
        debug_mode,
        runtime_lib,
        lib_dir,
        split_threshold,
        target,
        option_explicit: false,
    };

    match compile_file(file_path, &output, parse_only, &mut opts) {
        Ok(()) => {
            eprintln!("[pbcompiler] Success: {}", output);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

/// Get the value following a flag like "--runtime-lib /path/to/file"
fn get_flag_value(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|pos| args.get(pos + 1).cloned())
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
    opts: &mut codegen::CompileOptions,
) -> PbResult<()> {
    let path = Path::new(path);

    // Phase 1: Preprocess
    let t0 = std::time::Instant::now();
    let mut preprocessor = Preprocessor::new();
    let source_lines = preprocessor.process_file(path)?;
    eprintln!(
        "[pbcompiler] Preprocessed {} lines ({:.1}s)",
        source_lines.len(),
        t0.elapsed().as_secs_f64()
    );

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
        "[pbcompiler] Lexed {} tokens ({:.1}s)",
        tokens.len(),
        t1.elapsed().as_secs_f64()
    );

    // Phase 3: Parse
    let t2 = std::time::Instant::now();
    let mut parser = Parser::new(tokens);
    let program = parser.parse()?;
    eprintln!(
        "[pbcompiler] Parsed {} top-level items ({:.1}s)",
        program.items.len(),
        t2.elapsed().as_secs_f64()
    );

    if parser.error_count > 0 {
        return Err(pb::error::PbError::parser(format!("Build failed with {} parse error(s). Search for unexpected tokens in your .bas file.", parser.error_count), None, 0));
    }

    // #DIM ALL (official meta-directive) == OPTION EXPLICIT. The parser only
    // sees the OPTION EXPLICIT statement form; the preprocessor records #DIM ALL.
    opts.option_explicit = parser.option_explicit || preprocessor.dim_all();

    if parse_only {
        eprintln!("[pbcompiler] Parse-only mode, skipping codegen");
        return Ok(());
    }

    // Phase 4: Codegen
    let t3 = std::time::Instant::now();
    let output_path = Path::new(output);
    let pp_constants: HashMap<String, i64> = preprocessor.constants().clone();
    codegen::compile(&program, output_path, opts, &pp_constants)?;
    eprintln!(
        "[pbcompiler] Codegen complete ({:.1}s)",
        t3.elapsed().as_secs_f64()
    );

    // Phase 5: Embed resources (icons, version info, etc.) into EXE
    if opts.exe_mode {
        let exe_path = output_path.with_extension("exe");
        if exe_path.exists() {
            let has_icons = !preprocessor.resources().is_empty();
            let has_version = !preprocessor.version_info().strings.is_empty();
            if has_icons {
                embed_resources(&exe_path, preprocessor.resources());
            }
            if has_version {
                embed_version_info(&exe_path, preprocessor.version_info());
            }
        } else {
            eprintln!(
                "[pbcompiler] Warning: EXE not found for resource embedding: {}",
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
                    "[pbcompiler] Embedded {} icon resource(s) into {}",
                    resources.len(),
                    exe_path.display()
                );
            } else {
                eprintln!(
                    "[pbcompiler] Warning: icon embedding failed: {}",
                    String::from_utf8_lossy(&out.stderr)
                );
            }
        }
        Err(e) => eprintln!(
            "[pbcompiler] Warning: cannot run powershell for icon embedding: {}",
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
                    "[pbcompiler] Embedded VERSIONINFO into {}",
                    exe_path.display()
                );
            } else {
                eprintln!(
                    "[pbcompiler] Warning: VERSIONINFO embed failed: {}",
                    String::from_utf8_lossy(&out.stderr)
                );
            }
        }
        Err(e) => eprintln!("[pbcompiler] Warning: VERSIONINFO embed error: {e}"),
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
