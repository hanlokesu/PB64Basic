// =====================================================================
// Modified by the hanlokesu/PB64Basic fork (https://github.com/hanlokesu/PB64Basic).
// Derived from benstopics/powerbasilisk (https://github.com/benstopics/powerbasilisk), Apache-2.0.
// See LICENSE for the full license terms.
// =====================================================================
use std::collections::{HashMap, HashSet};
use std::path::Path;

use pb::ast::*;
use pb::error::{PbError, PbResult};

use crate::llvm_ir::{FunctionBuilder, IrType, ModuleBuilder, Val};
use crate::symbols::{ArrayInfo, SymbolTable};

/// Options for the compilation pipeline.
pub const DEFAULT_TARGET: &str = "i686-pc-windows-msvc";

/// Built-in Win32 equates (PBWin's equates.inc equivalents) used as a
/// fallback when a %CONSTANT is neither #CONST-defined nor %X= defined by
/// the program.  Values are from the Windows SDK headers.  batch 243:
/// without this, "%BS_DEFAULT OR %WS_TABSTOP" silently lost WS_TABSTOP
/// (unknown equates resolved to 0).
const PB_BUILTIN_EQUATES: &[(&str, i64)] = &[
    ("%WS_BORDER", 0x00800000),
    ("%WS_CAPTION", 0x00C00000),
    ("%WS_CHILD", 0x40000000),
    ("%WS_CHILDWINDOW", 0x40000000),
    ("%WS_CLIPCHILDREN", 0x02000000),
    ("%WS_CLIPSIBLINGS", 0x04000000),
    ("%WS_DISABLED", 0x08000000),
    ("%WS_DLGFRAME", 0x00400000),
    ("%WS_GROUP", 0x00020000),
    ("%WS_HSCROLL", 0x00100000),
    ("%WS_MAXIMIZE", 0x01000000),
    ("%WS_MAXIMIZEBOX", 0x00010000),
    ("%WS_MINIMIZE", 0x20000000),
    ("%WS_MINIMIZEBOX", 0x00020000),
    ("%WS_OVERLAPPED", 0x00000000),
    ("%WS_OVERLAPPEDWINDOW", 0x00CF0000),
    ("%WS_POPUP", 0x80000000),
    ("%WS_POPUPWINDOW", 0x80880000),
    ("%WS_SYSMENU", 0x00080000),
    ("%WS_TABSTOP", 0x00010000),
    ("%WS_THICKFRAME", 0x00040000),
    ("%WS_VISIBLE", 0x10000000),
    ("%WS_VSCROLL", 0x00200000),
    ("%WS_EX_ACCEPTFILES", 0x00000010),
    ("%WS_EX_CLIENTEDGE", 0x00000200),
    ("%WS_EX_CONTEXTHELP", 0x00000400),
    ("%WS_EX_DLGMODALFRAME", 0x00000001),
    ("%WS_EX_LEFT", 0x00000000),
    ("%WS_EX_LTRREADING", 0x00000000),
    ("%WS_EX_OVERLAPPEDWINDOW", 0x00000300),
    ("%WS_EX_RIGHTSCROLLBAR", 0x00000000),
    ("%WS_EX_STATICEDGE", 0x00020000),
    ("%WS_EX_TOPMOST", 0x00000008),
    ("%WS_EX_TRANSPARENT", 0x00000020),
    ("%WS_EX_WINDOWEDGE", 0x00000100),
    ("%BS_BOTTOM", 0x00000800),
    ("%BS_CENTER", 0x00000300),
    ("%BS_DEFAULT", 0x00000001),
    ("%BS_DEFPUSHBUTTON", 0x00000001),
    ("%BS_LEFT", 0x00000100),
    ("%BS_MULTILINE", 0x00002000),
    ("%BS_PUSHBUTTON", 0x00000000),
    ("%BS_RIGHT", 0x00000200),
    ("%BS_TOP", 0x00000400),
    ("%BS_VCENTER", 0x00000C00),
    ("%DS_3DLOOK", 0x00000004),
    ("%DS_CENTER", 0x00000800),
    ("%DS_CENTERMOUSE", 0x00001000),
    ("%DS_MODALFRAME", 0x00000080),
    ("%DS_NOFAILCREATE", 0x00000800),
    ("%DS_SETFONT", 0x00000040),
    ("%DS_SETFOREGROUND", 0x00000200),
    ("%ES_AUTOHSCROLL", 0x00000080),
    ("%ES_AUTOVSCROLL", 0x00000040),
    ("%ES_CENTER", 0x00000001),
    ("%ES_LEFT", 0x00000000),
    ("%ES_MULTILINE", 0x00000004),
    ("%ES_NOHIDESEL", 0x00000100),
    ("%ES_PASSWORD", 0x00000020),
    ("%ES_READONLY", 0x00000800),
    ("%ES_RIGHT", 0x00000002),
    ("%ES_WANTRETURN", 0x00001000),
    ("%SS_BLACKFRAME", 0x00000007),
    ("%SS_BLACKRECT", 0x00000003),
    ("%SS_CENTER", 0x00000001),
    ("%SS_GRAYFRAME", 0x00000008),
    ("%SS_LEFT", 0x00000000),
    ("%SS_LEFTNOWORDWRAP", 0x0000000C),
    ("%SS_RIGHT", 0x00000002),
    ("%SS_SIMPLE", 0x0000000B),
    ("%WM_COMMAND", 0x00000111),
    ("%WM_CTLCOLORMSGBOX", 0x00000132),
    ("%WM_CTLCOLOREDIT", 0x00000133),
    ("%WM_CTLCOLORLISTBOX", 0x00000134),
    ("%WM_CTLCOLORBTN", 0x00000135),
    ("%WM_CTLCOLORDLG", 0x00000136),
    ("%WM_CTLCOLORSCROLLBAR", 0x00000137),
    ("%WM_CTLCOLORSTATIC", 0x00000138),
    ("%WM_CLOSE", 0x00000010),
    ("%WM_CREATE", 0x00000001),
    ("%WM_DESTROY", 0x00000002),
    ("%WM_GETTEXT", 0x0000000D),
    ("%WM_GETTEXTLENGTH", 0x0000000E),
    ("%WM_KEYDOWN", 0x00000100),
    ("%WM_KEYUP", 0x00000101),
    ("%WM_LBUTTONDOWN", 0x00000201),
    ("%WM_LBUTTONUP", 0x00000202),
    ("%WM_MOUSEMOVE", 0x00000200),
    ("%WM_PAINT", 0x0000000F),
    ("%WM_QUIT", 0x00000012),
    ("%WM_SETTEXT", 0x0000000C),
    ("%WM_SETFONT", 0x00000030),
    ("%WM_SIZE", 0x00000005),
    ("%WM_TIMER", 0x00000113),
    ("%BN_CLICKED", 0x00000000),
    ("%CN_CLICKED", 0x00000000),
    ("%EN_CHANGE", 0x00000300),
    ("%EN_UPDATE", 0x00000400),
    ("%LBN_SELCHANGE", 0x00000001),
    ("%CBN_SELCHANGE", 0x00000001),
    ("%CB_ADDSTRING", 0x00000143),
    ("%CB_GETCURSEL", 0x00000147),
    ("%CB_GETLBTEXT", 0x00000148),
    ("%CB_SETCURSEL", 0x0000014E),
    ("%LB_ADDSTRING", 0x00000180),
    ("%LB_GETCURSEL", 0x00000188),
    ("%LB_GETSEL", 0x00000187),
    ("%LB_GETTEXT", 0x00000189),
    ("%LB_GETTEXTLEN", 0x0000018A),
    ("%LB_GETTOPINDEX", 0x0000010E),
    ("%LB_SETCURSEL", 0x00000186),
    ("%LB_SETTOPINDEX", 0x00000197),
    ("%LBN_DBLCLK", 0x00000002),
    ("%MB_OK", 0x00000000),
    ("%MB_OKCANCEL", 0x00000001),
    ("%MB_YESNO", 0x00000004),
    ("%MB_YESNOCANCEL", 0x00000003),
    ("%MB_ICONERROR", 0x00000010),
    ("%MB_ICONINFORMATION", 0x00000040),
    ("%MB_ICONQUESTION", 0x00000020),
    ("%MB_ICONWARNING", 0x00000030),
    ("%MB_DEFBUTTON1", 0x00000000),
    ("%MB_DEFBUTTON2", 0x00000100),
    ("%MB_DEFBUTTON3", 0x00000200),
    ("%MB_DEFBUTTON4", 0x00000300),
    ("%MB_ABORTRETRYIGNORE", 0x00000002),
    ("%MB_RETRYCANCEL", 0x00000005),
    ("%MB_CANCELTRYCONTINUE", 0x00000006),
    ("%MB_ICONHAND", 0x00000010),
    ("%MB_ICONSTOP", 0x00000010),
    ("%MB_ICONEXCLAMATION", 0x00000030),
    ("%MB_ICONASTERISK", 0x00000040),
    ("%MB_APPLMODAL", 0x00000000),
    ("%MB_SYSTEMMODAL", 0x00001000),
    ("%MB_TASKMODAL", 0x00002000),
    ("%MB_HELP", 0x00004000),
    ("%MB_NOFOCUS", 0x00008000),
    ("%MB_SETFOREGROUND", 0x00010000),
    ("%MB_DEFAULT_DESKTOP_ONLY", 0x00020000),
    ("%MB_TOPMOST", 0x00040000),
    ("%MB_RIGHT", 0x00080000),
    ("%MB_RTLREADING", 0x00100000),
    ("%MB_SERVICE_NOTIFICATION", 0x00200000),
    ("%IDOK", 1),
    ("%IDCANCEL", 2),
    ("%IDABORT", 3),
    ("%IDRETRY", 4),
    ("%IDIGNORE", 5),
    ("%IDYES", 6),
    ("%IDNO", 7),
    ("%IDCLOSE", 8),
    ("%IDHELP", 9),
    ("%IDTRYAGAIN", 10),
    ("%IDCONTINUE", 11),
    ("%TRUE", 1),
    ("%FALSE", 0),
];

pub struct CompileOptions {
    pub dll_mode: bool,
    pub exe_mode: bool,
    pub session_mode: bool,
    pub emit_llvm: bool,
    pub debug_mode: bool, // enable verbose debug logging (modal text, assertions)
    pub runtime_lib: Option<String>, // path to pb_runtime.obj
    pub lib_dir: Option<String>, // path to directory containing import libraries (.lib)
    pub split_threshold: usize, // split functions exceeding this many IR lines (0 = disabled)
    pub target: String, // LLVM target triple (e.g. "i686-pc-windows-msvc", "x86_64-pc-windows-msvc")
    /// `OPTION EXPLICIT` is in effect: every variable must be declared.
    pub option_explicit: bool,
}

/// Compile a parsed PB program to LLVM IR, then optionally to object code via clang.
pub fn compile(
    program: &Program,
    output_path: &Path,
    opts: &CompileOptions,
    pp_constants: &HashMap<String, i64>,
) -> PbResult<()> {
    let mut compiler = Compiler::with_target(&opts.target);
    compiler.option_explicit = opts.option_explicit;
    compiler.session_mode = opts.session_mode;
    compiler.debug_mode = opts.debug_mode;
    compiler.pp_constants = pp_constants.clone();
    compiler.compile_program(program)?;
    // Report statements that were silently dropped (no code generated)
    if !compiler.warnings.is_empty() {
        let warn_path = output_path.with_extension("unimplemented.log");
        let mut report = String::new();
        report.push_str("PB64Basic - unimplemented / silently-dropped statement report\n");
        report.push_str(&format!(
            "{} statement(s) parsed but produced NO code. These make the .exe run but do nothing.\n",
            compiler.warnings.len()
        ));
        report.push_str("Check each line below against your source:\n");
        for w in &compiler.warnings {
            report.push_str("  ");
            report.push_str(w);
            report.push('\n');
        }
        std::fs::write(&warn_path, &report)
            .map_err(|e| PbError::io(format!("Failed to write unimplemented report: {}", e)))?;
        eprintln!(
            "[PB64Basic] WARNING: {} statement(s) not implemented (silently dropped) - see {}",
            compiler.warnings.len(),
            warn_path.display()
        );
    }

    let ir = compiler.module.emit();

    // Write .ll file
    let ll_path = output_path.with_extension("ll");
    std::fs::write(&ll_path, &ir)
        .map_err(|e| PbError::io(format!("Failed to write .ll: {}", e)))?;
    eprintln!(
        "[PB64Basic] Wrote LLVM IR: {} ({} lines)",
        ll_path.display(),
        ir.lines().count()
    );

    if opts.emit_llvm {
        return Ok(());
    }

    // Split large functions if threshold is set
    let threshold = if opts.split_threshold > 0 {
        opts.split_threshold
    } else {
        20000
    };
    let (main_ir, split_ir) = split_large_functions(&ir, threshold);

    let mut obj_paths: Vec<std::path::PathBuf> = Vec::new();

    if let Some(ref split_ir_content) = split_ir {
        // Write the split main .ll (large functions removed)
        let main_ll_path = output_path.with_extension("main.ll");
        std::fs::write(&main_ll_path, &main_ir)
            .map_err(|e| PbError::io(format!("Failed to write main .ll: {}", e)))?;

        let main_obj_path = output_path.with_extension("main.obj");
        // Use -O0 to avoid clang 17.0.1 crashes (Register Coalescer, DAG ISel)
        compile_with_clang(&main_ll_path, &main_obj_path, "-O0", &opts.target)?;
        eprintln!("[PB64Basic] Wrote main object: {}", main_obj_path.display());
        obj_paths.push(main_obj_path);

        // Write and compile the split .ll (large functions only)
        let split_ll_path = output_path.with_extension("split.ll");
        std::fs::write(&split_ll_path, split_ir_content)
            .map_err(|e| PbError::io(format!("Failed to write split .ll: {}", e)))?;
        eprintln!(
            "[PB64Basic] Split large functions to: {}",
            split_ll_path.display()
        );

        let split_obj_path = output_path.with_extension("split.obj");
        compile_split_module(&split_ll_path, &split_obj_path, &opts.target)?;
        eprintln!(
            "[PB64Basic] Wrote split object: {}",
            split_obj_path.display()
        );
        obj_paths.push(split_obj_path);
    } else {
        // No split needed → compile as one unit
        let obj_path = output_path.with_extension("obj");
        compile_with_clang(&ll_path, &obj_path, "-O2", &opts.target)?;
        eprintln!("[PB64Basic] Wrote object: {}", obj_path.display());
        obj_paths.push(obj_path);
    }

    // Link if requested
    if opts.dll_mode {
        let dll_path = output_path.with_extension("dll");
        link_dll(&obj_paths[0], &dll_path, opts)?;
        eprintln!("[PB64Basic] Linked DLL: {}", dll_path.display());
    }

    if opts.exe_mode {
        let exe_path = output_path.with_extension("exe");
        let obj_refs: Vec<&Path> = obj_paths.iter().map(|p| p.as_path()).collect();
        link_exe(&obj_refs, &exe_path, opts)?;
        eprintln!("[PB64Basic] Linked EXE: {}", exe_path.display());
    }

    Ok(())
}

fn compile_with_clang(
    ll_path: &Path,
    obj_path: &Path,
    opt_level: &str,
    target: &str,
) -> PbResult<()> {
    let target_flag = format!("--target={}", target);
    let ll_str = ll_path.to_string_lossy();
    let obj_str = obj_path.to_string_lossy();
    let output = std::process::Command::new("clang")
        .args([
            "-c",
            opt_level,
            &target_flag,
            ll_str.as_ref(),
            "-o",
            obj_str.as_ref(),
        ])
        .output()
        .map_err(|e| PbError::io(format!("Failed to run clang: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(PbError::io(format!("clang failed:\n{}", stderr)));
    }

    Ok(())
}

/// Compile a split module containing large functions.
/// Tries multiple strategies since these functions crash clang's default ISel.
/// If all strategies fail, generates stubs for uncompilable functions.
fn compile_split_module(ll_path: &Path, obj_path: &Path, target: &str) -> PbResult<()> {
    let target_flag = format!("--target={}", target);
    let ll_str = ll_path.to_string_lossy();
    let obj_str = obj_path.to_string_lossy();

    // Strategy 1: Try -O1 (lighter optimization, may avoid DAG explosion)
    eprintln!("[PB64Basic] Compiling split module with -O1...");
    let result = std::process::Command::new("clang")
        .args([
            "-c",
            "-O1",
            &target_flag,
            ll_str.as_ref(),
            "-o",
            obj_str.as_ref(),
        ])
        .output();

    if let Ok(output) = &result {
        if output.status.success() {
            return Ok(());
        }
    }

    // Strategy 2: -O0 with fast-isel
    eprintln!("[PB64Basic] -O1 failed, trying -O0 -mllvm -fast-isel...");
    let result = std::process::Command::new("clang")
        .args([
            "-c",
            "-O0",
            "-mllvm",
            "-fast-isel",
            &target_flag,
            ll_str.as_ref(),
            "-o",
            obj_str.as_ref(),
        ])
        .output();

    if let Ok(output) = &result {
        if output.status.success() {
            return Ok(());
        }
    }

    // Strategy 3: Plain -O0
    eprintln!("[PB64Basic] fast-isel failed, trying plain -O0...");
    let result = std::process::Command::new("clang")
        .args([
            "-c",
            "-O0",
            &target_flag,
            ll_str.as_ref(),
            "-o",
            obj_str.as_ref(),
        ])
        .output();

    if let Ok(output) = &result {
        if output.status.success() {
            return Ok(());
        }
    }

    // Strategy 4: Generate stub module for all functions
    // BIGLOOP and similar functions crash clang at ALL optimization levels.
    // Emit stub functions that return default values so linking can proceed.
    eprintln!("[PB64Basic] All compilation strategies failed for split module.");
    eprintln!("[PB64Basic] Generating stub functions for uncompilable functions...");
    generate_stub_module(ll_path, obj_path, target)
}

/// Generate a stub .ll module where each function just returns a default value.
/// This allows linking to succeed even when clang can't compile the full function.
fn generate_stub_module(original_ll: &Path, obj_path: &Path, target: &str) -> PbResult<()> {
    let content = std::fs::read_to_string(original_ll)
        .map_err(|e| PbError::io(format!("Failed to read split .ll: {}", e)))?;

    let mut stub = String::new();
    use std::fmt::Write;

    // Copy module header
    for line in content.lines() {
        if line.starts_with("; Generated") || line.starts_with("target ") || line.is_empty() {
            writeln!(stub, "{}", line).unwrap();
        } else {
            break;
        }
    }

    // Copy type definitions
    for line in content.lines() {
        if line.starts_with('%') && line.contains(" = type ") {
            writeln!(stub, "{}", line).unwrap();
        }
    }

    // Copy declare statements
    for line in content.lines() {
        if line.starts_with("declare ") {
            writeln!(stub, "{}", line).unwrap();
        }
    }

    // Copy external global declarations
    for line in content.lines() {
        if line.starts_with('@') && line.contains(" = external global ") {
            writeln!(stub, "{}", line).unwrap();
        }
    }

    writeln!(stub).unwrap();

    // For each define, emit a stub that returns default
    for line in content.lines() {
        if line.starts_with("define ") {
            // Extract return type and generate stub
            let rest = line.strip_prefix("define ").unwrap();
            let rest = rest.strip_prefix("dllexport ").unwrap_or(rest);
            // rest is "TYPE @NAME(PARAMS) {"
            let ret_type = rest.split_whitespace().next().unwrap_or("void");

            // Extract function name
            let name = if let Some(at_pos) = rest.find('@') {
                let after_at = &rest[at_pos + 1..];
                if let Some(paren_pos) = after_at.find('(') {
                    &after_at[..paren_pos]
                } else {
                    after_at
                }
            } else {
                "unknown"
            };

            eprintln!(
                "[PB64Basic] Stubbing function: {} (returns {})",
                name, ret_type
            );

            // Emit stub: strip param names for the define
            let export = if line.contains("dllexport") {
                "dllexport "
            } else {
                ""
            };

            // Get the parameter part
            if let (Some(open), Some(close)) = (line.rfind('('), line.rfind(')')) {
                let params = &line[open..=close];
                writeln!(stub, "define {}{} @{}{} {{", export, ret_type, name, params).unwrap();
            } else {
                writeln!(stub, "define {}{} @{}() {{", export, ret_type, name).unwrap();
            }
            writeln!(stub, "entry:").unwrap();
            match ret_type {
                "void" => writeln!(stub, "  ret void").unwrap(),
                "i32" => writeln!(stub, "  ret i32 0").unwrap(),
                "i16" => writeln!(stub, "  ret i16 0").unwrap(),
                "i64" => writeln!(stub, "  ret i64 0").unwrap(),
                "double" => writeln!(stub, "  ret double 0.0").unwrap(),
                "float" => writeln!(stub, "  ret float 0.0").unwrap(),
                "ptr" => writeln!(stub, "  ret ptr null").unwrap(),
                _ => writeln!(stub, "  ret {} zeroinitializer", ret_type).unwrap(),
            }
            writeln!(stub, "}}").unwrap();
            writeln!(stub).unwrap();
        }
    }

    // Write and compile the stub module
    let stub_ll = original_ll.with_extension("stub.ll");
    std::fs::write(&stub_ll, &stub)
        .map_err(|e| PbError::io(format!("Failed to write stub .ll: {}", e)))?;

    compile_with_clang(&stub_ll, obj_path, "-O0", target)
}

/// Locate `legacy_stdio_definitions.lib` for the target architecture.
///
/// UCRT moved `printf`/`scanf` out of the static CRT into the universal CRT;
/// MSVC's own link line pulls this shim in to provide the legacy names.  clang
/// does not, so linking a 32-bit program that uses `PRINT` failed with
/// `lld-link: error: undefined symbol: _printf` while the same source linked
/// fine for x64.  The archive is passed by full path: the clang driver treats a
/// bare `foo.lib` argument as an input file rather than a library to search for,
/// so there is no name-only fallback to fall back on.  The toolset is discovered
/// by reading the Visual Studio install level rather than a hard-coded year - CI
/// moved to Visual Studio 2026 and the old year list found nothing there.
fn detect_legacy_stdio_lib(target: &str) -> Option<String> {
    let arch = if target.contains("i686") || target.contains("i386") {
        "x86"
    } else {
        "x64"
    };
    if arch != "x86" {
        // x64 already resolves the CRT stdio symbols; keep that link line unchanged.
        return None;
    }
    // Windows SDK-style environment (vcvars sets this); it already points at the
    // toolset folder, so its lib directory is checked directly.
    if let Ok(vct) = std::env::var("VCToolsInstallDir") {
        let lib = std::path::PathBuf::from(vct)
            .join("lib")
            .join(arch)
            .join("legacy_stdio_definitions.lib");
        if lib.is_file() {
            return Some(lib.to_string_lossy().into_owned());
        }
    }
    // Read the Visual Studio install level instead of hard-coding a year: the
    // layout is <base>\<year>\<edition>\VC\Tools\MSVC, and the year changes with
    // the runner image (VS 2026 in CI at the time of writing).  Hard-coding it
    // meant the scan silently found nothing there and the link failed.
    let mut roots: Vec<std::path::PathBuf> =
        vec![std::path::PathBuf::from(r"C:\BuildTools\VC\Tools\MSVC")];
    for base in [
        r"C:\Program Files\Microsoft Visual Studio",
        r"C:\Program Files (x86)\Microsoft Visual Studio",
    ] {
        let years = match std::fs::read_dir(base) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for year in years.flatten() {
            let editions = match std::fs::read_dir(year.path()) {
                Ok(e) => e,
                Err(_) => continue,
            };
            for edition in editions.flatten() {
                let candidate = edition.path().join("VC").join("Tools").join("MSVC");
                if candidate.is_dir() {
                    roots.push(candidate);
                }
            }
        }
    }
    let mut best: Option<(Vec<u64>, String)> = None;
    for root in roots.iter() {
        let entries = match std::fs::read_dir(root) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            // Toolset folders look like "14.44.35207"; skip non-version names.
            let ver: Vec<u64> = name
                .split('.')
                .filter_map(|s| s.parse::<u64>().ok())
                .collect();
            if ver.is_empty() {
                continue;
            }
            let lib = entry
                .path()
                .join("lib")
                .join(arch)
                .join("legacy_stdio_definitions.lib");
            if !lib.is_file() {
                continue;
            }
            if best.as_ref().is_none_or(|(bv, _)| ver > *bv) {
                best = Some((ver, lib.to_string_lossy().into_owned()));
            }
        }
    }
    best.map(|(_, p)| p)
}

fn link_dll(obj_path: &Path, dll_path: &Path, opts: &CompileOptions) -> PbResult<()> {
    let mut args: Vec<String> = vec![
        "-shared".to_string(),
        format!("--target={}", opts.target),
        obj_path.to_string_lossy().to_string(),
    ];

    // Runtime library (pb_debug_enter, pb_bstr_alloc, pb_str_concat, ...)
    if let Some(ref runtime) = opts.runtime_lib {
        args.push(runtime.clone());
    }

    args.push("-o".to_string());
    args.push(dll_path.to_string_lossy().to_string());
    args.push("-loleaut32".to_string());
    // 32-bit: UCRT keeps `printf`/`scanf` in the legacy stdio shim, which MSVC
    // links but clang does not.  Without it a program that uses PRINT fails with
    // `undefined symbol: _printf`.
    if let Some(ref legacy) = detect_legacy_stdio_lib(&opts.target) {
        args.push(legacy.clone());
    }

    // Windows system libraries (same set as EXE linking)
    if let Some(ref lib_dir) = opts.lib_dir {
        args.push(format!("-L{}", lib_dir));
        args.extend([
            "-lkernel32".to_string(),
            "-luser32".to_string(),
            "-lgdi32".to_string(),
            "-lshell32".to_string(),
            "-lcomctl32".to_string(),
            "-lcomdlg32".to_string(),
            "-ladvapi32".to_string(),
            "-lole32".to_string(),
            "-lwinmm".to_string(),
            "-lws2_32".to_string(),
            "-lwinspool".to_string(),
        ]);
    }

    let output = std::process::Command::new("clang")
        .args(&args)
        .output()
        .map_err(|e| PbError::io(format!("Failed to run clang for DLL linking: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(PbError::io(format!("clang DLL link failed:\n{}", stderr)));
    }

    Ok(())
}

/// Link object files into an executable.
fn link_exe(obj_paths: &[&Path], exe_path: &Path, opts: &CompileOptions) -> PbResult<()> {
    let mut args: Vec<String> = vec![format!("--target={}", opts.target)];

    // Add all object files
    for obj in obj_paths {
        args.push(obj.to_string_lossy().to_string());
    }

    // Add runtime library if specified
    if let Some(ref runtime) = opts.runtime_lib {
        args.push(runtime.clone());
    }

    // Output path
    args.push("-o".to_string());
    args.push(exe_path.to_string_lossy().to_string());

    // Always link oleaut32 (needed by pb_runtime for BSTR/SysAllocString)
    args.push("-loleaut32".to_string());
    // 32-bit: UCRT keeps `printf`/`scanf` in the legacy stdio shim, which MSVC
    // links but clang does not.  Without it a program that uses PRINT fails with
    // `undefined symbol: _printf`.
    if let Some(ref legacy) = detect_legacy_stdio_lib(&opts.target) {
        args.push(legacy.clone());
    }

    // Use Windows GUI subsystem (no console window)
    args.push("-Wl,/SUBSYSTEM:WINDOWS,/ENTRY:mainCRTStartup".to_string());

    // Link additional Windows libraries only when --lib-dir is provided
    if let Some(ref lib_dir) = opts.lib_dir {
        args.push(format!("-L{}", lib_dir));
        args.extend([
            "-lkernel32".to_string(),
            "-luser32".to_string(),
            "-lgdi32".to_string(),
            "-lshell32".to_string(),
            "-lcomctl32".to_string(),
            "-lcomdlg32".to_string(),
            "-ladvapi32".to_string(),
            "-lole32".to_string(),
            "-lwinmm".to_string(),
            "-lws2_32".to_string(),
            "-lwinspool".to_string(),
        ]);
    }

    let output = std::process::Command::new("clang")
        .args(&args)
        .output()
        .map_err(|e| PbError::io(format!("Failed to run clang for EXE linking: {}", e)))?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        // Print warnings but don't fail on them
        for line in stderr.lines() {
            if line.contains("error") && !line.contains("warning") {
                return Err(PbError::io(format!("clang EXE link failed:\n{}", stderr)));
            }
        }
        // Print any warnings
        for line in stderr.lines() {
            if line.contains("warning") {
                eprintln!("[PB64Basic] Link warning: {}", line);
            }
        }
    }

    if !output.status.success() {
        return Err(PbError::io(format!("clang EXE link failed:\n{}", stderr)));
    }

    Ok(())
}

/// Split large functions from the main IR module into a separate module.
///
/// Functions exceeding `threshold` lines are moved to a split module.
/// The main module gets `declare` stubs for those functions.
/// The split module gets the module header, all type definitions, global
/// declarations (as `external`), and `declare` stubs for all other functions.
///
/// Returns (main_ir, Some(split_ir)) if functions were split, or (ir, None) if not.
fn split_large_functions(ir: &str, threshold: usize) -> (String, Option<String>) {
    let lines: Vec<&str> = ir.lines().collect();

    // Find all function definition boundaries: (start_line, end_line, signature_line)
    // A function definition starts with "define " and ends with "}"
    struct FnRange {
        start: usize,
        end: usize,       // inclusive (the "}" line)
        sig_line: String, // the "define ..." line
        name: String,     // @FunctionName
    }

    let mut fn_ranges: Vec<FnRange> = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if lines[i].starts_with("define ") {
            let start = i;
            let sig_line = lines[i].to_string();

            // Extract function name from "define ... @NAME(..."
            let name = if let Some(at_pos) = sig_line.find('@') {
                let after_at = &sig_line[at_pos + 1..];
                if let Some(paren_pos) = after_at.find('(') {
                    after_at[..paren_pos].to_string()
                } else {
                    after_at.to_string()
                }
            } else {
                String::new()
            };

            // Find closing brace (at column 0)
            i += 1;
            while i < lines.len() && lines[i] != "}" {
                i += 1;
            }
            let end = i; // the "}" line

            fn_ranges.push(FnRange {
                start,
                end,
                sig_line,
                name,
            });
        }
        i += 1;
    }

    // Find functions exceeding threshold
    let large_fn_indices: Vec<usize> = fn_ranges
        .iter()
        .enumerate()
        .filter(|(_, fr)| fr.end - fr.start > threshold)
        .map(|(idx, _)| idx)
        .collect();

    if large_fn_indices.is_empty() {
        return (ir.to_string(), None);
    }

    let large_names: Vec<&str> = large_fn_indices
        .iter()
        .map(|&idx| fn_ranges[idx].name.as_str())
        .collect();
    eprintln!(
        "[PB64Basic] Splitting {} large function(s): {}",
        large_names.len(),
        large_names.join(", ")
    );

    // Build a set of line ranges to exclude from the main module
    let mut exclude_ranges: Vec<(usize, usize)> = Vec::new();
    for &idx in &large_fn_indices {
        let fr = &fn_ranges[idx];
        exclude_ranges.push((fr.start, fr.end));
    }

    // --- Build main module ---
    // Everything except large function bodies, with `declare` stubs added
    let mut main_lines: Vec<String> = Vec::new();
    let mut declare_stubs: Vec<String> = Vec::new();

    // Generate declare stubs for large functions
    for &idx in &large_fn_indices {
        let fr = &fn_ranges[idx];
        // Convert "define [dllexport] TYPE @NAME(PARAMS) {" to "declare TYPE @NAME(PARAMS)"
        let stub = fn_define_to_declare(&fr.sig_line);
        declare_stubs.push(stub);
    }

    let mut skip_until: Option<usize> = None;
    for (line_idx, line) in lines.iter().enumerate() {
        if let Some(skip_end) = skip_until {
            if line_idx <= skip_end {
                continue;
            }
            skip_until = None;
        }

        // Check if this line starts an excluded function
        let mut excluded = false;
        for &(start, end) in &exclude_ranges {
            if line_idx == start {
                skip_until = Some(end);
                excluded = true;
                break;
            }
        }

        if excluded {
            // Skip the next blank line after "}" too
            continue;
        }

        main_lines.push(line.to_string());
    }

    // Insert declare stubs after the globals section (before first define)
    let first_define_pos = main_lines.iter().position(|l| l.starts_with("define "));
    if let Some(pos) = first_define_pos {
        for (i, stub) in declare_stubs.iter().enumerate() {
            main_lines.insert(pos + i, stub.clone());
        }
        main_lines.insert(pos + declare_stubs.len(), String::new());
    }

    // --- Build split module ---
    let mut split_lines: Vec<String> = Vec::new();

    // Copy module header (target triple, datalayout)
    for line in &lines {
        if line.starts_with("; Generated") || line.starts_with("target ") || line.is_empty() {
            split_lines.push(line.to_string());
        } else {
            break;
        }
    }
    split_lines.push(String::new());

    // Copy type definitions
    for line in &lines {
        if line.starts_with('%') && line.contains(" = type ") {
            split_lines.push(line.to_string());
        }
    }

    // Add external declarations for all global variables
    for line in &lines {
        if line.starts_with('@') && line.contains(" = global ") {
            // Convert "@NAME = global TYPE init" to "@NAME = external global TYPE"
            if let Some(decl) = global_to_external(line) {
                split_lines.push(decl);
            }
        } else if line.starts_with('@') && line.contains(" = dllexport global ") {
            if let Some(decl) = global_to_external(line) {
                split_lines.push(decl);
            }
        } else if line.starts_with('@') && line.contains(" = external global ") {
            // Already in external form (e.g., @pb_debug_line = external global i32)
            split_lines.push(line.to_string());
        }
    }

    // Add external declarations for string constants referenced by large functions
    // (private unnamed_addr constants) - these need to be in the split module
    // Actually, string constants are private so they can't be cross-referenced.
    // We need to copy them into the split module too.
    let large_fn_text: Vec<&str> = large_fn_indices
        .iter()
        .flat_map(|&idx| {
            let fr = &fn_ranges[idx];
            &lines[fr.start..=fr.end]
        })
        .copied()
        .collect();

    let large_fn_text_joined = large_fn_text.join("\n");
    for line in &lines {
        if line.starts_with('@') && line.contains("private unnamed_addr constant") {
            // Check if this constant is referenced in any large function
            let const_name = if let Some(eq_pos) = line.find(" = ") {
                &line[..eq_pos]
            } else {
                continue;
            };
            if large_fn_text_joined.contains(const_name) {
                split_lines.push(line.to_string());
            }
        }
    }

    // Copy all declare statements (external functions and dllimport)
    for line in &lines {
        if line.starts_with("declare ") {
            split_lines.push(line.to_string());
        }
    }

    // Add declare stubs for all non-large defined functions
    for (idx, fr) in fn_ranges.iter().enumerate() {
        if !large_fn_indices.contains(&idx) {
            let stub = fn_define_to_declare(&fr.sig_line);
            split_lines.push(stub);
        }
    }

    split_lines.push(String::new());

    // Copy large function definitions
    for &idx in &large_fn_indices {
        let fr = &fn_ranges[idx];
        for line in &lines[fr.start..=fr.end] {
            split_lines.push(line.to_string());
        }
        split_lines.push(String::new());
    }

    let main_ir = main_lines.join("\n");
    let split_ir = split_lines.join("\n");

    (main_ir, Some(split_ir))
}

/// Convert a `define` line to a `declare` stub.
/// "define dllexport i32 @FOO(i32 %x, ptr %y) {" → "declare i32 @FOO(i32, ptr)"
fn fn_define_to_declare(define_line: &str) -> String {
    // Remove "define " prefix and optional "dllexport "
    let rest = define_line.strip_prefix("define ").unwrap_or(define_line);
    let rest = rest.strip_prefix("dllexport ").unwrap_or(rest);

    // Find the opening "{" and remove it
    let rest = if let Some(brace_pos) = rest.rfind('{') {
        rest[..brace_pos].trim()
    } else {
        rest.trim()
    };

    // Strip parameter names from the signature: "i32 %x" → "i32"
    // Find params between ( and )
    if let (Some(open), Some(close)) = (rest.find('('), rest.rfind(')')) {
        let before_params = &rest[..open];
        let params_str = &rest[open + 1..close];

        // Strip parameter names
        let stripped_params: Vec<&str> = params_str
            .split(',')
            .map(|p| {
                let p = p.trim();
                if p.is_empty() {
                    return p;
                }
                // "i32 %name" → "i32", "ptr %name" → "ptr", "..." → "..."
                if p == "..." {
                    return p;
                }
                // Take just the type (first word)
                if let Some(space_pos) = p.find(' ') {
                    p[..space_pos].trim()
                } else {
                    p
                }
            })
            .collect();

        format!("declare {}({})", before_params, stripped_params.join(", "))
    } else {
        format!("declare {}", rest)
    }
}

/// Convert a global definition to an external declaration.
/// "@NAME = global i32 0" → "@NAME = external global i32"
/// "@NAME = dllexport global %TYPE zeroinitializer" → "@NAME = external global %TYPE"
fn global_to_external(line: &str) -> Option<String> {
    // Find "@NAME = "
    let eq_pos = line.find(" = ")?;
    let name = &line[..eq_pos];

    // Find "global TYPE ..."
    let global_pos = line.find(" global ")?;
    let after_global = &line[global_pos + 8..]; // skip " global "

    // Type is everything up to the initializer (first space after type, or end)
    // Types can be: i32, double, ptr, [N x T], %TYPENAME, { ... }
    let ir_type = extract_ir_type(after_global);

    Some(format!("{} = external global {}", name, ir_type))
}

/// Extract the IR type from the start of a string like "i32 0" or "[1600 x i32] zeroinitializer"
fn extract_ir_type(s: &str) -> &str {
    let s = s.trim();

    // Handle array types: [N x T]
    if s.starts_with('[') {
        if let Some(bracket_end) = s.find(']') {
            return &s[..bracket_end + 1];
        }
    }

    // Handle struct types: %TYPENAME
    if s.starts_with('%') {
        // %TYPENAME followed by space
        if let Some(space) = s.find(' ') {
            return &s[..space];
        }
        return s;
    }

    // Handle simple types: i32, double, ptr, etc.
    if let Some(space) = s.find(' ') {
        return &s[..space];
    }
    s
}

struct Compiler {
    module: ModuleBuilder,
    symbols: SymbolTable,
    /// True when the source declared `OPTION EXPLICIT`.
    option_explicit: bool,
    /// Line of the statement currently being compiled (0 = unknown).
    current_line: usize,

    // Function/sub declarations collected during first pass
    functions: HashMap<String, FuncInfo>,
    subs: HashMap<String, FuncInfo>,

    // Current function context
    current_fn_name: Option<String>,
    current_fn_return_type: Option<(PbType, IrType)>,
    current_fn_retval_ptr: Option<Val>,

    // Loop control: stack of (continue_label, exit_label, is_for)
    loop_stack: Vec<LoopInfo>,

    // GLOBAL arr() AS TYPE awaiting DIM with bounds
    pending_global_arrays: HashMap<String, PbType>,

    // THREADED vars already declared (global to every Sub/Function, so a
    // repeated THREADED declaration in another proc must not re-emit the
    // module global -- LLVM would reject the duplicate symbol).
    threaded_declared: std::collections::HashSet<String>,

    // ASMDATA blocks: canonical uppercase name -> IR global symbol name.
    // CODEPTR(BlockName) and ASM Offset/name references resolve through this.
    asmdata_blocks: std::collections::HashMap<String, String>,

    // Name of the global empty string constant (e.g., "@.str.empty")
    empty_string_name: String,

    // Name of the global "PowerBASIC" constant used as the default MSGBOX
    // title when title$ is omitted (CHW msgbox.htm / msgbox_s.htm; batch 250).
    msgbox_title_name: String,

    // User-defined TYPE layouts
    type_layouts: HashMap<String, TypeLayout>,

    // GOSUB/RETURN context for the current function being compiled
    gosub_context: Option<GosubContext>,

    // ON ERROR GOTO context for the current function
    // handler block name (None = error trapping disabled)
    onerror_handler: Option<String>,
    // (stmt_start_block, stmt_next_block) per checked call statement
    onerror_checkpoints: Vec<(String, String)>,
    // TRY/END TRY: stack of END TRY block names for EXIT TRY (batch 26)
    try_exit_stack: Vec<String>,

    // Session struct mode: wrap all globals in a single struct
    session_mode: bool,
    session_fields: Vec<SessionFieldInfo>,

    // Debug mode: emit extra logging calls (modal text, state transitions)
    debug_mode: bool,

    // Preprocessor %CONSTANTS (name → value)
    pp_constants: HashMap<String, i64>,

    // Unimplemented-statement warnings collected during codegen (name / line)
    warnings: Vec<String>,
    #[allow(dead_code)]
    xprint_dc: Option<String>,

    // OOP event bus (batch 222): class -> EVENT SOURCE interface names, and
    // event method name -> integer id (allocated lazily so RAISEEVENT and
    // EVENTS FROM agree on the same id for the same method name).
    event_sources: std::collections::HashMap<String, Vec<String>>,
    event_method_ids: std::collections::HashMap<String, i32>,

    // String variables bound via FIELD dyn$ — their assignments must copy
    // into a fresh mutable buffer (a plain store would point at a read-only
    // string constant and pb_field_set would crash writing through it).
    field_bound_strings: std::collections::HashSet<String>,
}

/// Tracks a global variable or array that will become a session struct field.
struct SessionFieldInfo {
    name: String,    // normalized global name
    ir_type: IrType, // field IR type (scalar or array type)
    pb_type: PbType, // PB type
    is_array: bool,
    elem_ir_type: Option<IrType>,        // for arrays: element type
    array_dims: Option<Vec<(i64, i64)>>, // for arrays: (lower_bound, count) per dim
    total_elements: Option<usize>,       // for arrays: total element count
}

/// Which target a RESUME statement jumps to.
enum ResumeMode {
    /// Re-execute the statement that errored
    Retry,
    /// Continue with the statement after the one that errored
    Next,
}

/// Tracks GOSUB/RETURN info for one function's compilation.
struct GosubContext {
    /// Map from label name → LLVM block name
    label_blocks: HashMap<String, String>,
    /// Map from GOSUB site ID → (return-point label name)
    return_points: Vec<String>,
    /// Alloca for the gosub return address (i32)
    ret_addr_ptr: Val,
    /// Labels already emitted (to prevent duplicate LLVM blocks)
    emitted_labels: HashSet<String>,
}

#[allow(dead_code)]
#[derive(Clone)]
struct FuncInfo {
    ir_name: String,
    ret_type: IrType,
    params: Vec<ParamInfo>,
    export: bool,
    is_external: bool, // true if DECLARE LIB (dllimport)
    is_stdcall: bool,  // true if Win32 API (stdcall on 32-bit), false for cdecl
}

#[derive(Clone)]
struct ParamInfo {
    name: String,
    pb_type: PbType,
    ir_type: IrType,
    is_byval: bool,
    is_optional: bool,
}

struct LoopInfo {
    continue_label: String,
    exit_label: String,
    is_for: bool,
}

/// Layout of a user-defined TYPE (struct).
#[allow(dead_code)]
struct TypeLayout {
    ir_name: String,                   // normalized TYPE name
    fields: Vec<FieldInfo>,            // ordered fields
    field_map: HashMap<String, usize>, // normalized field name → index
}

#[allow(dead_code)]
struct FieldInfo {
    name: String,
    pb_type: PbType,
    ir_type: IrType,
}

impl Compiler {
    /// Returns true if the DLL is stubbed in pb_runtime.c (not available at link time).
    fn is_stubbed_dll(lib_name: &str) -> bool {
        let name = lib_name.trim_end_matches(".DLL").trim_end_matches(".dll");
        matches!(name, "VARICHEK" | "REMARKSLIB")
    }

    /// Returns true if the DLL name is a known Win32 API library that uses stdcall calling convention.
    fn is_win32_stdcall_dll(lib_name: &str) -> bool {
        // Strip .DLL extension if present
        let name = lib_name.trim_end_matches(".DLL").trim_end_matches(".dll");
        matches!(
            name,
            "KERNEL32"
                | "USER32"
                | "GDI32"
                | "COMCTL32"
                | "COMDLG32"
                | "SHELL32"
                | "OLEAUT32"
                | "OLE32"
                | "ADVAPI32"
                | "WINMM"
                | "WS2_32"
                | "WSOCK32"
                | "NETAPI32"
                | "SHLWAPI"
                | "VERSION"
                | "WININET"
                | "WINSPOOL"
                | "MAPI32"
                | "PSAPI"
                | "SETUPAPI"
                | "IPHLPAPI"
                | "SECUR32"
                | "CRYPT32"
                | "MPR"
                | "DBGHELP"
        )
    }

    fn with_target(target: &str) -> Self {
        Compiler {
            module: ModuleBuilder::new(target),
            symbols: SymbolTable::new(),
            option_explicit: false,
            current_line: 0,
            functions: HashMap::new(),
            subs: HashMap::new(),
            current_fn_name: None,
            current_fn_return_type: None,
            current_fn_retval_ptr: None,
            loop_stack: Vec::new(),
            pending_global_arrays: HashMap::new(),
            threaded_declared: std::collections::HashSet::new(),
            asmdata_blocks: std::collections::HashMap::new(),
            empty_string_name: String::new(),
            msgbox_title_name: String::new(),
            type_layouts: HashMap::new(),
            gosub_context: None,
            onerror_handler: None,
            onerror_checkpoints: Vec::new(),
            try_exit_stack: Vec::new(),
            session_mode: false,
            session_fields: Vec::new(),
            debug_mode: false,
            pp_constants: HashMap::new(),
            warnings: Vec::new(),
            xprint_dc: None,
            event_sources: std::collections::HashMap::new(),
            event_method_ids: std::collections::HashMap::new(),
            field_bound_strings: std::collections::HashSet::new(),
        }
    }

    // ========== Type mapping ==========

    fn ir_type_for(pb_type: &PbType) -> IrType {
        match pb_type {
            PbType::Long | PbType::Dword => IrType::I32,
            PbType::Integer | PbType::Word => IrType::I16,
            PbType::Byte => IrType::I8,
            PbType::Quad => IrType::I64,
            PbType::Double | PbType::Ext | PbType::Cur => IrType::Double,
            PbType::Single => IrType::Float,
            PbType::String | PbType::FixedString(_) => IrType::Ptr,
            PbType::Field => IrType::Struct("pb.field".to_string()),
            PbType::UserDefined(name) => IrType::Struct(normalize_name(name)),
            PbType::Variant => IrType::I32, // default to LONG
        }
    }

    /// Byte size of an IR type (for TYPE SET memcpy sizing).
    fn ir_size_bytes(ir: &IrType) -> u32 {
        match ir {
            IrType::I1 | IrType::I8 => 1,
            IrType::I16 => 2,
            IrType::I32 | IrType::Float => 4,
            IrType::I64 | IrType::Double | IrType::Ptr => 8,
            IrType::Array(n, elem) => (*n as u32) * Self::ir_size_bytes(elem),
            _ => 4,
        }
    }

    /// IR type for a struct field → FixedString becomes [N x i8] instead of ptr
    fn ir_type_for_field(pb_type: &PbType) -> IrType {
        match pb_type {
            PbType::FixedString(n) => IrType::Array(*n, Box::new(IrType::I8)),
            other => Self::ir_type_for(other),
        }
    }

    fn is_string_pb(pb_type: &PbType) -> bool {
        matches!(pb_type, PbType::String | PbType::FixedString(_))
    }

    fn is_string_name(name: &str) -> bool {
        name.ends_with('$')
    }

    fn is_float_pb(pb_type: &PbType) -> bool {
        matches!(
            pb_type,
            PbType::Double | PbType::Single | PbType::Ext | PbType::Cur
        )
    }

    // ========== Top-level compilation ==========

    fn compile_program(&mut self, program: &Program) -> PbResult<()> {
        // %pb.field = type { ptr, i32, i32 } — FIELD variable (16 bytes)
        self.module.define_struct_type(
            "pb.field",
            &[IrType::Ptr, IrType::I32, IrType::I32, IrType::I32],
        );
        self.module.declare_function(
            "pb_open_random",
            &IrType::I32,
            &[IrType::I32, IrType::I32, IrType::I32],
            false,
        );
        self.module
            .declare_function("pb_put_record", &IrType::Void, &[IrType::I32], false);
        self.module
            .declare_function("pb_get_record", &IrType::Void, &[IrType::I32], false);
        self.module.declare_function(
            "pb_seek_record",
            &IrType::Void,
            &[IrType::I32, IrType::I64],
            false,
        );
        self.module.declare_function(
            "pb_control_add_groupbox",
            &IrType::Ptr,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );
        self.module.declare_function(
            "pb_control_add_radiobutton",
            &IrType::Ptr,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );
        self.module.declare_function(
            "pb_field_bind_file",
            &IrType::I32,
            &[IrType::I32, IrType::I64, IrType::I32, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_field_bind_str",
            &IrType::I32,
            &[IrType::Ptr, IrType::I64, IrType::I32, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_field_set",
            &IrType::Void,
            &[IrType::Ptr, IrType::Ptr, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_str_assign_copy",
            &IrType::Void,
            &[IrType::Ptr, IrType::Ptr, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_mat_fill",
            &IrType::Void,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I64,
                IrType::Double,
            ],
            false,
        );
        self.module.declare_function(
            "pb_mat_copy",
            &IrType::Void,
            &[IrType::Ptr, IrType::Ptr, IrType::I32, IrType::I64],
            false,
        );
        self.module.declare_function(
            "pb_mat_add",
            &IrType::Void,
            &[
                IrType::Ptr,
                IrType::Ptr,
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I64,
                IrType::I32,
            ],
            false,
        );
        self.module.declare_function(
            "pb_mat_scale",
            &IrType::Void,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I64,
                IrType::Double,
                IrType::Ptr,
            ],
            false,
        );
        self.module.declare_function(
            "pb_mat_identity",
            &IrType::Void,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );
        self.module.declare_function(
            "pb_mat_trn",
            &IrType::Void,
            &[
                IrType::Ptr,
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );
        self.module.declare_function(
            "pb_mat_mul",
            &IrType::Void,
            &[
                IrType::Ptr,
                IrType::Ptr,
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );
        self.module.declare_function(
            "pb_mat_inv",
            &IrType::I32,
            &[
                IrType::Ptr,
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );
        self.module
            .declare_function("pb_callstk_push", &IrType::Void, &[IrType::Ptr], false);
        self.module
            .declare_function("pb_callstk_pop", &IrType::Void, &[], false);
        self.module
            .declare_function("pb_callstk_count", &IrType::I64, &[], false);
        self.module
            .declare_function("pb_callstk_get", &IrType::Ptr, &[IrType::I64], false);
        self.module
            .declare_function("pb_callstk_dump", &IrType::Void, &[IrType::Ptr], false);
        self.module
            .declare_function("pb_profile_enable", &IrType::Void, &[], false);
        self.module
            .declare_function("pb_profile_dump", &IrType::Void, &[IrType::Ptr], false);
        self.module.declare_function(
            "pb_regex_scan",
            &IrType::I32,
            &[
                IrType::Ptr,
                IrType::Ptr,
                IrType::I64,
                IrType::Ptr,
                IrType::Ptr,
            ],
            false,
        );
        self.module.declare_function(
            "pb_regex_replace",
            &IrType::Ptr,
            &[
                IrType::Ptr,
                IrType::Ptr,
                IrType::Ptr,
                IrType::I64,
                IrType::Ptr,
            ],
            false,
        );
        self.module
            .declare_function("pb_field_get", &IrType::Ptr, &[IrType::Ptr], false);
        self.module
            .declare_function("pb_field_reset", &IrType::Void, &[IrType::Ptr], false);
        self.module
            .declare_function("pb_field_tostr", &IrType::Void, &[IrType::Ptr], false);

        // Declare printf
        self.module
            .declare_function("printf", &IrType::I32, &[IrType::Ptr], true);
        self.module
            .declare_function("fflush", &IrType::I32, &[IrType::Ptr], false);

        // LLVM intrinsics for math
        self.module.declare_intrinsic(
            "llvm.pow.f64",
            &IrType::Double,
            &[IrType::Double, IrType::Double],
        );
        self.module
            .declare_intrinsic("llvm.fabs.f64", &IrType::Double, &[IrType::Double]);
        self.module
            .declare_intrinsic("llvm.floor.f64", &IrType::Double, &[IrType::Double]);
        self.module
            .declare_intrinsic("llvm.ceil.f64", &IrType::Double, &[IrType::Double]);
        self.module
            .declare_intrinsic("llvm.sqrt.f64", &IrType::Double, &[IrType::Double]);
        self.module
            .declare_intrinsic("llvm.round.f64", &IrType::Double, &[IrType::Double]);
        self.module
            .declare_intrinsic("llvm.log.f64", &IrType::Double, &[IrType::Double]);
        self.module
            .declare_intrinsic("llvm.exp.f64", &IrType::Double, &[IrType::Double]);
        self.module
            .declare_intrinsic("llvm.sin.f64", &IrType::Double, &[IrType::Double]);
        self.module
            .declare_intrinsic("llvm.cos.f64", &IrType::Double, &[IrType::Double]);

        // C math library (no LLVM intrinsic)
        self.module
            .declare_function("tan", &IrType::Double, &[IrType::Double], false);
        self.module
            .declare_function("atan", &IrType::Double, &[IrType::Double], false);
        self.module
            .declare_function("asin", &IrType::Double, &[IrType::Double], false);
        self.module
            .declare_function("acos", &IrType::Double, &[IrType::Double], false);
        self.module
            .declare_function("sinh", &IrType::Double, &[IrType::Double], false);
        self.module
            .declare_function("cosh", &IrType::Double, &[IrType::Double], false);
        self.module
            .declare_function("tanh", &IrType::Double, &[IrType::Double], false);
        self.module.declare_function(
            "atan2",
            &IrType::Double,
            &[IrType::Double, IrType::Double],
            false,
        );
        self.module
            .declare_function("asinh", &IrType::Double, &[IrType::Double], false);
        self.module
            .declare_function("acosh", &IrType::Double, &[IrType::Double], false);
        self.module
            .declare_function("atanh", &IrType::Double, &[IrType::Double], false);
        self.module.declare_function(
            "hypot",
            &IrType::Double,
            &[IrType::Double, IrType::Double],
            false,
        );
        self.module
            .declare_function("cbrt", &IrType::Double, &[IrType::Double], false);
        self.module
            .declare_function("expm1", &IrType::Double, &[IrType::Double], false);
        self.module
            .declare_function("log1p", &IrType::Double, &[IrType::Double], false);
        self.module
            .declare_function("erf", &IrType::Double, &[IrType::Double], false);

        // C stdlib for RND
        self.module
            .declare_function("rand", &IrType::I32, &[], false);

        // PB runtime library (pb_runtime.c)
        self.module.declare_function(
            "pb_format",
            &IrType::Ptr,
            &[IrType::Double, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_parse",
            &IrType::Ptr,
            &[IrType::Ptr, IrType::Ptr, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_parsecount",
            &IrType::I32,
            &[IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_replace",
            &IrType::Void,
            &[IrType::Ptr, IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module
            .declare_function("pb_error_message", &IrType::Ptr, &[IrType::I32], false);
        self.module.declare_function(
            "pb_remove",
            &IrType::Ptr,
            &[IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_using",
            &IrType::Ptr,
            &[IrType::Ptr, IrType::Double],
            false,
        );

        // oleaut32: BSTR allocation for bridge compatibility
        self.module.declare_function(
            "pb_bstr_alloc",
            &IrType::Ptr,
            &[IrType::Ptr, IrType::I32],
            false,
        );
        self.module
            .declare_function("pb_str_len", &IrType::I32, &[IrType::Ptr], false);
        self.module
            .declare_function("pb_str_cstr_len", &IrType::I32, &[IrType::Ptr], false);
        self.module
            .declare_function("pb_bstr_free", &IrType::Void, &[IrType::Ptr], false);
        self.module.declare_function(
            "pb_str_concat",
            &IrType::Ptr,
            &[IrType::Ptr, IrType::Ptr],
            false,
        );

        // Debug/crash facilities (declared unconditionally — every function
        // body calls pb_debug_enter, including library/DLL sources without
        // a PBMAIN entry point)
        self.module
            .declare_function("pb_install_crash_handler", &IrType::Void, &[], false);
        self.module
            .declare_function("pb_debug_enter", &IrType::Void, &[IrType::Ptr], false);
        self.module.declare_function(
            "pb_debug_log_msg",
            &IrType::Void,
            &[IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_debug_modal",
            &IrType::Void,
            &[IrType::Ptr, IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module
            .declare_external_global("pb_debug_line", &IrType::I32);

        // File I/O runtime
        self.module
            .declare_function("pb_freefile", &IrType::I32, &[], false);
        self.module
            .declare_function("pb_fre", &IrType::I64, &[], false);
        self.module.declare_function(
            "pb_open",
            &IrType::I32,
            &[IrType::Ptr, IrType::I32, IrType::I32],
            false,
        );
        self.module
            .declare_function("pb_close", &IrType::Void, &[IrType::I32], false);
        self.module.declare_function(
            "pb_print_file",
            &IrType::Void,
            &[IrType::I32, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_print_file_newline",
            &IrType::Void,
            &[IrType::I32],
            false,
        );
        self.module
            .declare_function("pb_line_input", &IrType::Ptr, &[IrType::I32], false);
        self.module
            .declare_function("pb_eof", &IrType::I32, &[IrType::I32], false);
        self.module.declare_function(
            "pb_input_file_str",
            &IrType::Void,
            &[IrType::I32, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_input_console",
            &IrType::Ptr,
            &[IrType::Ptr, IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_line_input_console",
            &IrType::Ptr,
            &[IrType::Ptr, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_input_file_int",
            &IrType::Void,
            &[IrType::I32, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_input_file_dbl",
            &IrType::Void,
            &[IrType::I32, IrType::Ptr],
            false,
        );
        self.module
            .declare_function("pb_kill", &IrType::I32, &[IrType::Ptr], false);
        self.module
            .declare_function("pb_cls", &IrType::Void, &[], false);
        self.module
            .declare_function("pb_environ_set", &IrType::Void, &[IrType::Ptr], false);
        self.module.declare_function(
            "pb_filecopy",
            &IrType::I32,
            &[IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_setattr",
            &IrType::Void,
            &[IrType::Ptr, IrType::I32],
            false,
        );
        self.module
            .declare_function("_errno", &IrType::Ptr, &[], false);

        // C string library (size_t = i32 on 32-bit)
        self.module
            .declare_function("strlen", &IrType::I32, &[IrType::Ptr], false);
        self.module
            .declare_function("strcmp", &IrType::I32, &[IrType::Ptr, IrType::Ptr], false);
        self.module
            .declare_function("strcpy", &IrType::Ptr, &[IrType::Ptr, IrType::Ptr], false);
        self.module.declare_function(
            "strncpy",
            &IrType::Ptr,
            &[IrType::Ptr, IrType::Ptr, IrType::I32],
            false,
        );
        self.module
            .declare_function("strcat", &IrType::Ptr, &[IrType::Ptr, IrType::Ptr], false);
        self.module
            .declare_function("strstr", &IrType::Ptr, &[IrType::Ptr, IrType::Ptr], false);
        self.module
            .declare_function("malloc", &IrType::Ptr, &[IrType::I32], false);
        self.module
            .declare_function("free", &IrType::Void, &[IrType::Ptr], false);
        self.module.declare_function(
            "snprintf",
            &IrType::I32,
            &[IrType::Ptr, IrType::I32, IrType::Ptr],
            true,
        );
        self.module
            .declare_function("atof", &IrType::Double, &[IrType::Ptr], false);
        self.module.declare_function(
            "memcpy",
            &IrType::Ptr,
            &[IrType::Ptr, IrType::Ptr, IrType::I32],
            false,
        );
        self.module
            .declare_function("toupper", &IrType::I32, &[IrType::I32], false);
        self.module
            .declare_function("tolower", &IrType::I32, &[IrType::I32], false);

        // System functions
        self.module
            .declare_function("time", &IrType::I32, &[IrType::Ptr], false);
        self.module
            .declare_function("pb_date", &IrType::Ptr, &[], false);
        self.module
            .declare_function("pb_time", &IrType::Ptr, &[], false);
        self.module
            .declare_function("pb_waitkey", &IrType::Ptr, &[], false);
        self.module
            .declare_function("pb_environ", &IrType::Ptr, &[IrType::Ptr], false);
        self.module
            .declare_function("pb_exe_path", &IrType::Ptr, &[], false);
        self.module
            .declare_function("pb_exe_name", &IrType::Ptr, &[], false);
        self.module
            .declare_function("srand", &IrType::Void, &[IrType::I32], false);
        self.module
            .declare_function("rand", &IrType::I32, &[], false);
        self.module
            .declare_function("_mkdir", &IrType::I32, &[IrType::Ptr], false);
        self.module
            .declare_function("_rmdir", &IrType::I32, &[IrType::Ptr], false);
        self.module
            .declare_function("_chdir", &IrType::I32, &[IrType::Ptr], false);
        self.module
            .declare_function("_access", &IrType::I32, &[IrType::Ptr, IrType::I32], false);
        self.module.declare_function(
            "pb_lset",
            &IrType::Void,
            &[IrType::Ptr, IrType::Ptr, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_rset",
            &IrType::Void,
            &[IrType::Ptr, IrType::Ptr, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_lset_buf",
            &IrType::Void,
            &[IrType::Ptr, IrType::Ptr, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_rset_buf",
            &IrType::Void,
            &[IrType::Ptr, IrType::Ptr, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_erase_array",
            &IrType::Void,
            &[IrType::Ptr, IrType::I32, IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_get",
            &IrType::Void,
            &[IrType::I32, IrType::I64, IrType::Ptr, IrType::I64],
            false,
        );
        self.module.declare_function(
            "pb_put",
            &IrType::Void,
            &[IrType::I32, IrType::I64, IrType::Ptr, IrType::I64],
            false,
        );
        self.module.declare_function(
            "pb_array_sort",
            &IrType::Void,
            &[IrType::Ptr, IrType::I32, IrType::I32, IrType::I32],
            false,
        );
        self.module
            .declare_function("pb_console_set_title", &IrType::Void, &[IrType::Ptr], false);
        self.module
            .declare_function("pb_console_get_title", &IrType::Ptr, &[], false);
        self.module.declare_function(
            "pb_window_new",
            &IrType::Ptr,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );
        self.module.declare_function(
            "pb_window_new_dlg",
            &IrType::Ptr,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );
        self.module
            .declare_function("pb_message_loop", &IrType::Void, &[], false);
        self.module.declare_function(
            "pb_control_add_button",
            &IrType::Ptr,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );
        self.module.declare_function(
            "pb_control_add_button_ex",
            &IrType::Ptr,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::Ptr,
            ],
            false,
        );
        self.module.declare_function(
            "pb_control_add_checkbox_ex",
            &IrType::Ptr,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::Ptr,
            ],
            false,
        );
        self.module.declare_function(
            "pb_control_add_radiobutton_ex",
            &IrType::Ptr,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::Ptr,
            ],
            false,
        );
        self.module.declare_function(
            "pb_control_add_groupbox_ex",
            &IrType::Ptr,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::Ptr,
            ],
            false,
        );
        self.module.declare_function(
            "pb_control_add_option_ex",
            &IrType::Ptr,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::Ptr,
            ],
            false,
        );
        self.module.declare_function(
            "pb_control_add_check3state_ex",
            &IrType::Ptr,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::Ptr,
            ],
            false,
        );
        self.module.declare_function(
            "pb_control_add_frame_ex",
            &IrType::Ptr,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::Ptr,
            ],
            false,
        );
        self.module.declare_function(
            "pb_control_add_textbox_ex",
            &IrType::Ptr,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::Ptr,
            ],
            false,
        );
        self.module.declare_function(
            "pb_control_add_line_ex",
            &IrType::Ptr,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::Ptr,
            ],
            false,
        );
        self.module.declare_function(
            "pb_control_add_label_ex",
            &IrType::Ptr,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::Ptr,
            ],
            false,
        );
        self.module.declare_function(
            "pb_control_add_editbox_ex",
            &IrType::Ptr,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::Ptr,
            ],
            false,
        );
        self.module.declare_function(
            "pb_control_add_scrollbar_ex",
            &IrType::Ptr,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::Ptr,
            ],
            false,
        );
        self.module.declare_function(
            "pb_control_add_tab_ex",
            &IrType::Ptr,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::Ptr,
            ],
            false,
        );
        self.module.declare_function(
            "pb_control_add_listbox_ex",
            &IrType::Ptr,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::Ptr,
                IrType::I64,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::Ptr,
            ],
            false,
        );
        self.module.declare_function(
            "pb_control_add_combobox_ex",
            &IrType::Ptr,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::Ptr,
                IrType::I64,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::Ptr,
            ],
            false,
        );
        self.module.declare_function(
            "pb_control_add_editbox",
            &IrType::Ptr,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );
        self.module.declare_function(
            "pb_control_add_combobox",
            &IrType::Ptr,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );
        self.module.declare_function(
            "pb_control_get_text_by_id",
            &IrType::Void,
            &[IrType::Ptr, IrType::I32, IrType::Ptr, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_control_get_text",
            &IrType::Void,
            &[IrType::Ptr, IrType::Ptr, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_control_set_text",
            &IrType::Void,
            &[IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_register_callback_hwnd",
            &IrType::Void,
            &[IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module
            .declare_function("pb_control_check", &IrType::Void, &[IrType::Ptr], false);
        self.module
            .declare_function("pb_control_uncheck", &IrType::Void, &[IrType::Ptr], false);
        self.module
            .declare_function("pb_control_get_check", &IrType::I64, &[IrType::Ptr], false);
        self.module.declare_function(
            "pb_progressbar_set_range",
            &IrType::I64,
            &[IrType::Ptr, IrType::I64, IrType::I64, IrType::I64],
            false,
        );
        self.module.declare_function(
            "pb_progressbar_set_pos",
            &IrType::I64,
            &[IrType::Ptr, IrType::I64, IrType::I64],
            false,
        );
        self.module.declare_function(
            "pb_progressbar_set_step",
            &IrType::I64,
            &[IrType::Ptr, IrType::I64, IrType::I64],
            false,
        );
        self.module.declare_function(
            "pb_progressbar_step",
            &IrType::I64,
            &[IrType::Ptr, IrType::I64, IrType::I64],
            false,
        );
        self.module.declare_function(
            "pb_progressbar_get_pos",
            &IrType::I64,
            &[IrType::Ptr, IrType::I64],
            false,
        );
        self.module.declare_function(
            "pb_progressbar_get_lo",
            &IrType::I64,
            &[IrType::Ptr, IrType::I64],
            false,
        );
        self.module.declare_function(
            "pb_progressbar_get_hi",
            &IrType::I64,
            &[IrType::Ptr, IrType::I64],
            false,
        );
        self.module.declare_function(
            "pb_header_send",
            &IrType::I64,
            &[
                IrType::Ptr,
                IrType::I64,
                IrType::I64,
                IrType::I64,
                IrType::I64,
            ],
            false,
        );
        self.module.declare_function(
            "pb_header_get_count",
            &IrType::I64,
            &[IrType::Ptr, IrType::I64],
            false,
        );
        self.module.declare_function(
            "pb_header_get_item",
            &IrType::I64,
            &[IrType::Ptr, IrType::I64, IrType::I64, IrType::I64],
            false,
        );
        self.module.declare_function(
            "pb_header_set_item",
            &IrType::I64,
            &[IrType::Ptr, IrType::I64, IrType::I64, IrType::I64],
            false,
        );
        self.module.declare_function(
            "pb_control_add_scrollbar",
            &IrType::Ptr,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );
        self.module.declare_function(
            "pb_debug_dump",
            &IrType::Void,
            &[IrType::Ptr, IrType::I64],
            false,
        );
        self.module.declare_function(
            "pb_control_add_label",
            &IrType::Ptr,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );
        self.module.declare_function(
            "pb_control_add_monthcal",
            &IrType::Ptr,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );
        self.module.declare_function(
            "pb_control_add_animate",
            &IrType::Ptr,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );
        self.module.declare_function(
            "pb_control_add_richedit",
            &IrType::Ptr,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );
        self.module.declare_function(
            "pb_control_add_trackbar",
            &IrType::Ptr,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );
        self.module.declare_function(
            "pb_control_add_updown",
            &IrType::Ptr,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );
        self.module.declare_function(
            "pb_control_add_hotkey",
            &IrType::Ptr,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );
        self.module.declare_function(
            "pb_control_add_ipaddress",
            &IrType::Ptr,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );
        self.module.declare_function(
            "pb_control_add_progressbar",
            &IrType::Ptr,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );
        self.module.declare_function(
            "pb_control_add_listview",
            &IrType::Ptr,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );
        self.module.declare_function(
            "pb_control_add_treeview",
            &IrType::Ptr,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );

        // ---- TOOLBAR / STATUSBAR (batch 164) ----
        for ctl in [
            "pb_control_add_toolbar",
            "pb_control_add_statusbar",
            "pb_control_add_graphic",
            "pb_control_add_header",
        ] {
            self.module.declare_function(
                ctl,
                &IrType::Ptr,
                &[
                    IrType::Ptr,
                    IrType::I32,
                    IrType::Ptr,
                    IrType::I32,
                    IrType::I32,
                    IrType::I32,
                    IrType::I32,
                    IrType::I32,
                    IrType::I32,
                ],
                false,
            );
        }
        // CONTROL SET COLOR hDlg, id&, foreclr&, backclr&
        self.module.declare_function(
            "pb_control_set_color",
            &IrType::Void,
            &[IrType::Ptr, IrType::I64, IrType::I64, IrType::I64],
            false,
        );
        self.module.declare_function(
            "pb_toolbar_add_button",
            &IrType::I64,
            &[
                IrType::I64,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::Ptr,
                IrType::I32,
            ],
            false,
        );
        self.module.declare_function(
            "pb_toolbar_add_separator",
            &IrType::I64,
            &[
                IrType::I64,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );
        self.module.declare_function(
            "pb_toolbar_delete_button",
            &IrType::I64,
            &[IrType::I64, IrType::I32, IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_toolbar_get_state",
            &IrType::I64,
            &[IrType::I64, IrType::I32, IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_toolbar_get_count",
            &IrType::I64,
            &[IrType::I64, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_toolbar_set_imagelist",
            &IrType::I64,
            &[IrType::I64, IrType::I32, IrType::I64, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_toolbar_set_state",
            &IrType::I64,
            &[
                IrType::I64,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );
        self.module.declare_function(
            "pb_statusbar_set_parts",
            &IrType::Void,
            &[IrType::I64, IrType::I32, IrType::Ptr, IrType::I64],
            false,
        );
        self.module.declare_function(
            "pb_statusbar_set_text",
            &IrType::I64,
            &[
                IrType::I64,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::Ptr,
            ],
            false,
        );
        self.module.declare_function(
            "pb_listview_insert_column",
            &IrType::Void,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );
        self.module.declare_function(
            "pb_listview_insert_item",
            &IrType::Void,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::Ptr,
            ],
            false,
        );
        self.module.declare_function(
            "pb_listview_get_count",
            &IrType::I64,
            &[IrType::Ptr, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_listview_get_text",
            &IrType::Void,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::Ptr,
                IrType::I32,
            ],
            false,
        );
        self.module.declare_function(
            "pb_listview_set_text",
            &IrType::Void,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::Ptr,
            ],
            false,
        );
        self.module.declare_function(
            "pb_listview_delete_item",
            &IrType::Void,
            &[IrType::Ptr, IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_listview_reset",
            &IrType::Void,
            &[IrType::Ptr, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_listview_delete_column",
            &IrType::Void,
            &[IrType::Ptr, IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_listview_find",
            &IrType::I64,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::Ptr,
                IrType::I32,
            ],
            false,
        );
        self.module.declare_function(
            "pb_listview_fit_content",
            &IrType::Void,
            &[IrType::Ptr, IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_listview_fit_header",
            &IrType::Void,
            &[IrType::Ptr, IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_listview_get_column",
            &IrType::I64,
            &[IrType::Ptr, IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_listview_set_column",
            &IrType::Void,
            &[IrType::Ptr, IrType::I32, IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_listview_get_header",
            &IrType::Void,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::Ptr,
                IrType::I32,
            ],
            false,
        );
        self.module.declare_function(
            "pb_listview_set_header",
            &IrType::Void,
            &[IrType::Ptr, IrType::I32, IrType::I32, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_listview_get_headerid",
            &IrType::Void,
            &[IrType::Ptr, IrType::I32, IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_listview_get_mode",
            &IrType::I64,
            &[IrType::Ptr, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_listview_set_mode",
            &IrType::Void,
            &[IrType::Ptr, IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_listview_get_selcount",
            &IrType::I64,
            &[IrType::Ptr, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_listview_get_select",
            &IrType::I64,
            &[IrType::Ptr, IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_listview_get_state",
            &IrType::I64,
            &[IrType::Ptr, IrType::I32, IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_listview_select",
            &IrType::Void,
            &[IrType::Ptr, IrType::I32, IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_listview_unselect",
            &IrType::Void,
            &[IrType::Ptr, IrType::I32, IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_listview_get_stylexx",
            &IrType::I64,
            &[IrType::Ptr, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_listview_set_stylexx",
            &IrType::Void,
            &[IrType::Ptr, IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_listview_get_user",
            &IrType::I64,
            &[IrType::Ptr, IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_listview_set_user",
            &IrType::Void,
            &[IrType::Ptr, IrType::I32, IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_listview_set_image",
            &IrType::Void,
            &[IrType::Ptr, IrType::I32, IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_listview_set_image2",
            &IrType::Void,
            &[IrType::Ptr, IrType::I32, IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_listview_set_overlay",
            &IrType::Void,
            &[IrType::Ptr, IrType::I32, IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_listview_set_imagelist",
            &IrType::Void,
            &[IrType::Ptr, IrType::I32, IrType::Ptr, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_listview_visible",
            &IrType::Void,
            &[IrType::Ptr, IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_listview_sort",
            &IrType::Void,
            &[IrType::Ptr, IrType::I32, IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_treeview_insert_item",
            &IrType::Void,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::Ptr,
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::Ptr,
                IrType::Ptr,
            ],
            false,
        );
        self.module.declare_function(
            "pb_treeview_get_count",
            &IrType::I64,
            &[IrType::Ptr, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_treeview_get_text",
            &IrType::Void,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::Ptr,
                IrType::Ptr,
                IrType::I32,
            ],
            false,
        );
        self.module.declare_function(
            "pb_scrollbar_get_pos",
            &IrType::I64,
            &[IrType::Ptr, IrType::I32],
            false,
        );

        self.module.declare_function(
            "pb_scrollbar_get_pagesize",
            &IrType::I64,
            &[IrType::Ptr, IrType::I32],
            false,
        );

        self.module.declare_function(
            "pb_scrollbar_get_trackpos",
            &IrType::I64,
            &[IrType::Ptr, IrType::I32],
            false,
        );

        self.module.declare_function(
            "pb_scrollbar_get_lo",
            &IrType::I64,
            &[IrType::Ptr, IrType::I32],
            false,
        );

        self.module.declare_function(
            "pb_scrollbar_get_hi",
            &IrType::I64,
            &[IrType::Ptr, IrType::I32],
            false,
        );

        self.module.declare_function(
            "pb_scrollbar_set_range",
            &IrType::I64,
            &[IrType::Ptr, IrType::I32, IrType::I32, IrType::I32],
            false,
        );

        self.module.declare_function(
            "pb_scrollbar_set_pagesize",
            &IrType::I64,
            &[IrType::Ptr, IrType::I32, IrType::I32],
            false,
        );

        self.module.declare_function(
            "pb_scrollbar_set_pos",
            &IrType::I64,
            &[IrType::Ptr, IrType::I32, IrType::I32],
            false,
        );

        self.module.declare_function(
            "pb_cblb_add",
            &IrType::I64,
            &[IrType::Ptr, IrType::I32, IrType::Ptr, IrType::I32],
            false,
        );

        self.module.declare_function(
            "pb_cblb_delete",
            &IrType::I64,
            &[IrType::Ptr, IrType::I32, IrType::I32, IrType::I32],
            false,
        );

        self.module.declare_function(
            "pb_cblb_find",
            &IrType::I64,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );

        self.module.declare_function(
            "pb_cblb_get_count",
            &IrType::I64,
            &[IrType::Ptr, IrType::I32, IrType::I32],
            false,
        );

        self.module.declare_function(
            "pb_cblb_get_selcount",
            &IrType::I64,
            &[IrType::Ptr, IrType::I32, IrType::I32],
            false,
        );

        self.module.declare_function(
            "pb_cblb_get_select",
            &IrType::I64,
            &[IrType::Ptr, IrType::I32, IrType::I32, IrType::I32],
            false,
        );

        self.module.declare_function(
            "pb_cblb_get_state",
            &IrType::I64,
            &[IrType::Ptr, IrType::I32, IrType::I32, IrType::I32],
            false,
        );

        self.module.declare_function(
            "pb_cblb_get_text",
            &IrType::Void,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );

        self.module.declare_function(
            "pb_cblb_get_user",
            &IrType::I64,
            &[IrType::Ptr, IrType::I32, IrType::I32, IrType::I32],
            false,
        );

        self.module.declare_function(
            "pb_cblb_insert",
            &IrType::I64,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::Ptr,
                IrType::I32,
            ],
            false,
        );

        self.module.declare_function(
            "pb_cblb_reset",
            &IrType::I64,
            &[IrType::Ptr, IrType::I32, IrType::I32],
            false,
        );

        self.module.declare_function(
            "pb_cblb_select",
            &IrType::I64,
            &[IrType::Ptr, IrType::I32, IrType::I32, IrType::I32],
            false,
        );

        self.module.declare_function(
            "pb_cblb_set_text",
            &IrType::I64,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::Ptr,
                IrType::I32,
            ],
            false,
        );

        self.module.declare_function(
            "pb_cblb_set_user",
            &IrType::I64,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I64,
                IrType::I32,
            ],
            false,
        );

        self.module.declare_function(
            "pb_control_add_tab",
            &IrType::Ptr,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );

        self.module.declare_function(
            "pb_tab_delete",
            &IrType::I64,
            &[IrType::Ptr, IrType::I32, IrType::I32],
            false,
        );

        self.module.declare_function(
            "pb_tab_get_count",
            &IrType::I64,
            &[IrType::Ptr, IrType::I32],
            false,
        );

        self.module.declare_function(
            "pb_tab_get_dialog",
            &IrType::I64,
            &[IrType::Ptr, IrType::I32, IrType::I32],
            false,
        );

        self.module.declare_function(
            "pb_tab_get_image",
            &IrType::I64,
            &[IrType::Ptr, IrType::I32, IrType::I32],
            false,
        );

        self.module
            .declare_function("pb_tab_get_page", &IrType::I64, &[IrType::Ptr], false);

        self.module.declare_function(
            "pb_tab_get_select",
            &IrType::I64,
            &[IrType::Ptr, IrType::I32],
            false,
        );

        self.module.declare_function(
            "pb_tab_get_text",
            &IrType::I64,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::Ptr,
                IrType::I32,
            ],
            false,
        );

        self.module.declare_function(
            "pb_tab_insert_page",
            &IrType::I64,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::Ptr,
                IrType::Ptr,
            ],
            false,
        );

        self.module.declare_function(
            "pb_tab_reset",
            &IrType::I64,
            &[IrType::Ptr, IrType::I32],
            false,
        );

        self.module.declare_function(
            "pb_tab_select",
            &IrType::I64,
            &[IrType::Ptr, IrType::I32, IrType::I32],
            false,
        );

        self.module.declare_function(
            "pb_tab_set_image",
            &IrType::I64,
            &[IrType::Ptr, IrType::I32, IrType::I32, IrType::I32],
            false,
        );

        self.module.declare_function(
            "pb_tab_set_imagelist",
            &IrType::I64,
            &[IrType::Ptr, IrType::I32, IrType::Ptr],
            false,
        );

        self.module.declare_function(
            "pb_tab_set_text",
            &IrType::I64,
            &[IrType::Ptr, IrType::I32, IrType::I32, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_cblb_unselect",
            &IrType::I64,
            &[IrType::Ptr, IrType::I32, IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_treeview_delete",
            &IrType::Void,
            &[IrType::Ptr, IrType::I32, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_treeview_reset",
            &IrType::Void,
            &[IrType::Ptr, IrType::I32],
            false,
        );

        self.module.declare_function(
            "pb_treeview_get_attr",
            &IrType::I64,
            &[IrType::Ptr, IrType::I32, IrType::Ptr, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_treeview_set_attr",
            &IrType::Void,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::Ptr,
                IrType::I32,
                IrType::I64,
            ],
            false,
        );
        self.module.declare_function(
            "pb_treeview_select",
            &IrType::Void,
            &[IrType::Ptr, IrType::I32, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_treeview_unselect",
            &IrType::Void,
            &[IrType::Ptr, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_treeview_set_text",
            &IrType::Void,
            &[IrType::Ptr, IrType::I32, IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_treeview_set_imagelist",
            &IrType::Void,
            &[IrType::Ptr, IrType::I32, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_progress_set_range",
            &IrType::Void,
            &[IrType::Ptr, IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_progress_set_pos",
            &IrType::Void,
            &[IrType::Ptr, IrType::I32],
            false,
        );
        self.module
            .declare_function("pb_get_cb_msg", &IrType::I32, &[], false);
        self.module
            .declare_function("pb_get_cb_hwnd", &IrType::Ptr, &[], false);
        self.module
            .declare_function("pb_get_cb_ctl", &IrType::I32, &[], false);
        self.module
            .declare_function("pb_get_cb_ctlmsg", &IrType::I32, &[], false);
        self.module
            .declare_function("pb_get_cb_wparam", &IrType::I64, &[], false);
        self.module
            .declare_function("pb_get_cb_lparam", &IrType::I64, &[], false);
        self.module.declare_function(
            "pb_register_dialog_cb",
            &IrType::Void,
            &[IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_dialog_set_text",
            &IrType::Void,
            &[IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_combobox_add",
            &IrType::Void,
            &[IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_listbox_add",
            &IrType::Void,
            &[IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_dialog_end",
            &IrType::Void,
            &[IrType::Ptr, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_dialog_menu",
            &IrType::I32,
            &[IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_control_get_text_by_id",
            &IrType::Void,
            &[IrType::Ptr, IrType::I32, IrType::Ptr, IrType::I32],
            false,
        );
        self.module
            .declare_function("pb_message_loop", &IrType::Void, &[], false);
        self.module
            .declare_function("pb_show_modal_to", &IrType::I32, &[IrType::Ptr], false);
        self.module.declare_function(
            "pb_msgbox_impl",
            &IrType::I32,
            &[IrType::Ptr, IrType::Ptr, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_control_add_hscrollbar",
            &IrType::Ptr,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );
        self.module.declare_function(
            "pb_control_set_pos",
            &IrType::Void,
            &[IrType::Ptr, IrType::I64],
            false,
        );
        self.module
            .declare_function("pb_control_get_pos", &IrType::I64, &[IrType::Ptr], false);
        for n in [
            "pb_control_show",
            "pb_control_hide",
            "pb_control_enable",
            "pb_control_disable",
            "pb_control_focus",
            "pb_control_kill",
        ] {
            self.module
                .declare_function(n, &IrType::Void, &[IrType::Ptr], false);
        }
        self.module
            .declare_function("pb_dialog_doevents", &IrType::Void, &[], false);
        self.module.declare_function(
            "pb_dialog_get_text",
            &IrType::Void,
            &[IrType::Ptr, IrType::Ptr, IrType::I64],
            false,
        );
        self.module.declare_function(
            "pb_dialog_show_state",
            &IrType::Void,
            &[IrType::Ptr, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_control_set_check",
            &IrType::Void,
            &[IrType::Ptr, IrType::I32],
            false,
        );
        self.module
            .declare_function("pb_dialog_center", &IrType::Void, &[IrType::Ptr], false);
        self.module.declare_function(
            "pb_control_add_custom",
            &IrType::Ptr,
            &[
                IrType::Ptr,
                IrType::Ptr,
                IrType::I32,
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );
        self.module.declare_function(
            "pb_control_add_listbox",
            &IrType::Ptr,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );
        self.module.declare_function(
            "pb_control_add_checkbox",
            &IrType::Ptr,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );
        self.module.declare_function(
            "pb_type_set",
            &IrType::Void,
            &[IrType::Ptr, IrType::Ptr, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_type_set_str",
            &IrType::Void,
            &[IrType::Ptr, IrType::Ptr, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_dir_first",
            &IrType::Ptr,
            &[IrType::Ptr, IrType::I32, IrType::I32],
            false,
        );
        self.module
            .declare_function("pb_dir_next", &IrType::Ptr, &[], false);
        self.module
            .declare_function("pb_dir_close", &IrType::Void, &[], false);
        self.module
            .declare_function("pb_tix", &IrType::I64, &[], false);
        self.module
            .declare_function("pb_mkbyt", &IrType::Ptr, &[IrType::I32], false);
        self.module
            .declare_function("pb_mkint", &IrType::Ptr, &[IrType::I32], false);
        self.module
            .declare_function("pb_mklong", &IrType::Ptr, &[IrType::I32], false);
        self.module
            .declare_function("pb_mkquad", &IrType::Ptr, &[IrType::I64], false);
        self.module
            .declare_function("pb_mksingle", &IrType::Ptr, &[IrType::Float], false);
        self.module
            .declare_function("pb_mkdouble", &IrType::Ptr, &[IrType::Double], false);
        self.module.declare_function(
            "pb_cv_int",
            &IrType::I64,
            &[IrType::Ptr, IrType::I64, IrType::I64, IrType::I64],
            false,
        );
        self.module.declare_function(
            "pb_cv_dbl",
            &IrType::Double,
            &[IrType::Ptr, IrType::I64, IrType::I64],
            false,
        );
        self.module
            .declare_function("pb_tally", &IrType::I64, &[IrType::Ptr, IrType::Ptr], false);
        self.module
            .declare_function("pb_strreverse", &IrType::Ptr, &[IrType::Ptr], false);
        self.module.declare_function(
            "pb_strinsert",
            &IrType::Ptr,
            &[IrType::Ptr, IrType::Ptr, IrType::I64],
            false,
        );
        self.module.declare_function(
            "pb_strdelete",
            &IrType::Ptr,
            &[IrType::Ptr, IrType::I64, IrType::I64],
            false,
        );
        self.module.declare_function(
            "pb_repeat",
            &IrType::Ptr,
            &[IrType::I64, IrType::Ptr],
            false,
        );
        self.module
            .declare_function("pb_frac", &IrType::Double, &[IrType::Double], false);
        self.module
            .declare_function("pb_isfolder", &IrType::I64, &[IrType::Ptr], false);
        self.module
            .declare_function("pb_exp2", &IrType::Double, &[IrType::Double], false);
        self.module
            .declare_function("pb_exp10", &IrType::Double, &[IrType::Double], false);
        self.module
            .declare_function("pb_log2", &IrType::Double, &[IrType::Double], false);
        self.module
            .declare_function("pb_log10", &IrType::Double, &[IrType::Double], false);
        self.module
            .declare_function("pb_bin", &IrType::Ptr, &[IrType::I64], false);
        self.module
            .declare_function("pb_oct", &IrType::Ptr, &[IrType::I64], false);
        self.module
            .declare_function("pb_dec", &IrType::Ptr, &[IrType::I64], false);
        self.module.declare_function(
            "pb_verify",
            &IrType::I64,
            &[IrType::Ptr, IrType::Ptr, IrType::I64],
            false,
        );
        self.module
            .declare_function("pb_getattr", &IrType::I64, &[IrType::Ptr], false);
        self.module
            .declare_function("pb_diskfree", &IrType::I64, &[IrType::Ptr], false);
        self.module
            .declare_function("pb_disksize", &IrType::I64, &[IrType::Ptr], false);
        self.module.declare_function(
            "pb_clip",
            &IrType::Ptr,
            &[IrType::Ptr, IrType::Ptr, IrType::I64, IrType::I64],
            false,
        );
        self.module.declare_function(
            "pb_wrap",
            &IrType::Ptr,
            &[IrType::Ptr, IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_unwrap",
            &IrType::Ptr,
            &[IrType::Ptr, IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_shrink",
            &IrType::Ptr,
            &[IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_remove_string",
            &IrType::Ptr,
            &[IrType::Ptr, IrType::Ptr, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_retain_string",
            &IrType::Ptr,
            &[IrType::Ptr, IrType::Ptr, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_remain_string",
            &IrType::Ptr,
            &[IrType::Ptr, IrType::Ptr, IrType::I64, IrType::I32],
            false,
        );
        self.module
            .declare_function("pb_mcase_string", &IrType::Ptr, &[IrType::Ptr], false);
        self.module
            .declare_function("pb_build", &IrType::Ptr, &[IrType::Ptr, IrType::I64], false);
        self.module
            .declare_function("pb_dayname", &IrType::Ptr, &[IrType::I64], false);
        self.module.declare_function(
            "pb_bits_str",
            &IrType::Ptr,
            &[IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_pathname",
            &IrType::Ptr,
            &[IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module
            .declare_function("pb_printer_count", &IrType::I64, &[], false);
        self.module.declare_function(
            "pb_fileattr",
            &IrType::I64,
            &[IrType::I32, IrType::I32],
            false,
        );
        self.module
            .declare_function("pb_filename", &IrType::Ptr, &[IrType::I32], false);
        self.module
            .declare_function("pb_erl_str", &IrType::Ptr, &[], false);
        self.module.declare_function(
            "pb_extract",
            &IrType::Ptr,
            &[IrType::I64, IrType::Ptr, IrType::Ptr, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_rgb3",
            &IrType::I64,
            &[IrType::I64, IrType::I64, IrType::I64],
            false,
        );
        self.module.declare_function(
            "pb_bgr3",
            &IrType::I64,
            &[IrType::I64, IrType::I64, IrType::I64],
            false,
        );
        self.module
            .declare_function("pb_rgb_swap", &IrType::I64, &[IrType::I64], false);
        self.module.declare_function(
            "pb_mem_copy",
            &IrType::Void,
            &[IrType::I64, IrType::I64, IrType::I64],
            false,
        );
        self.module.declare_function(
            "pb_mem_swap",
            &IrType::Void,
            &[IrType::I64, IrType::I64, IrType::I64],
            false,
        );
        self.module.declare_function(
            "pb_mem_fill",
            &IrType::Void,
            &[IrType::I64, IrType::I64, IrType::I64, IrType::I64],
            false,
        );
        self.module.declare_function(
            "pb_mem_fill_str",
            &IrType::Void,
            &[IrType::I64, IrType::I64, IrType::Ptr],
            false,
        );

        self.module.declare_function(
            "pb_font_new",
            &IrType::I32,
            &[
                IrType::Ptr,
                IrType::Float,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );
        self.module
            .declare_function("pb_font_end", &IrType::I32, &[IrType::I32], false);
        self.module.declare_function(
            "pb_imagelist_new",
            &IrType::I64,
            &[
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );
        self.module.declare_function(
            "pb_imagelist_add_bitmap",
            &IrType::I32,
            &[IrType::I64, IrType::I64, IrType::I64],
            false,
        );
        self.module.declare_function(
            "pb_imagelist_add_bitmap_file",
            &IrType::I32,
            &[IrType::I64, IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_imagelist_add_icon",
            &IrType::I32,
            &[IrType::I64, IrType::I64],
            false,
        );
        self.module.declare_function(
            "pb_imagelist_add_icon_file",
            &IrType::I32,
            &[IrType::I64, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_imagelist_add_masked",
            &IrType::I32,
            &[IrType::I64, IrType::I64, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_imagelist_add_masked_file",
            &IrType::I32,
            &[IrType::I64, IrType::Ptr, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_imagelist_set_overlay",
            &IrType::I32,
            &[IrType::I64, IrType::I32, IrType::I32],
            false,
        );
        self.module
            .declare_function("pb_imagelist_count", &IrType::I32, &[IrType::I64], false);
        self.module
            .declare_function("pb_imagelist_kill", &IrType::I32, &[IrType::I64], false);
        // batch 211: ACCEL ATTACH.  The call arm emits pb_accel_attach, so the
        // symbol has to be declared here too - a call without a declaration is
        // "use of undefined value" at clang time, which is how this was caught.
        self.module.declare_function(
            "pb_accel_attach",
            &IrType::I64,
            &[IrType::Ptr, IrType::Ptr, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_color",
            &IrType::Void,
            &[IrType::I32, IrType::I32],
            false,
        );
        self.module
            .declare_function("pb_menu_new_bar", &IrType::I64, &[], false);
        self.module
            .declare_function("pb_menu_new_popup", &IrType::I64, &[], false);
        self.module.declare_function(
            "pb_menu_add_string",
            &IrType::I32,
            &[IrType::I64, IrType::Ptr, IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_menu_add_popup",
            &IrType::I32,
            &[IrType::I64, IrType::I64, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_menu_delete",
            &IrType::I32,
            &[IrType::I64, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_menu_get_state",
            &IrType::I32,
            &[IrType::I64, IrType::I32, IrType::I32, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_menu_attach",
            &IrType::I32,
            &[IrType::I64, IrType::I64],
            false,
        );
        self.module
            .declare_function("pb_menu_draw_bar", &IrType::I32, &[IrType::I64], false);
        self.module.declare_function(
            "pb_menu_context",
            &IrType::I32,
            &[
                IrType::I64,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::Ptr,
            ],
            false,
        );
        self.module.declare_function(
            "pb_menu_set_state",
            &IrType::I32,
            &[IrType::I64, IrType::I32, IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_menu_get_text",
            &IrType::I32,
            &[IrType::I64, IrType::I32, IrType::I32, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_menu_set_text",
            &IrType::I32,
            &[IrType::I64, IrType::I32, IrType::I32, IrType::Ptr],
            false,
        );
        self.module
            .declare_function("pb_menu_destroy", &IrType::I32, &[IrType::I64], false);
        self.module.declare_function(
            "pb_gdi_bitmap_new",
            &IrType::I64,
            &[IrType::I32, IrType::I32],
            false,
        );
        self.module
            .declare_function("pb_gdi_bitmap_end", &IrType::I32, &[IrType::I64], false);
        // batch 179 - the two-operand GRAPHIC ATTACH form needs its own helper
        self.module.declare_function(
            "pb_graphic_attach_ctl",
            &IrType::I32,
            &[IrType::Ptr, IrType::I64],
            false,
        );
        // batch 181 - GRAPHIC REDRAW / SET FOCUS / SET LOC / SET CLIENT / SET OVERLAP / GET OVERLAP
        self.module
            .declare_function("pb_graphic_redraw", &IrType::I32, &[], false);
        self.module
            .declare_function("pb_graphic_set_focus", &IrType::I32, &[], false);
        self.module.declare_function(
            "pb_graphic_set_loc",
            &IrType::I32,
            &[IrType::I64, IrType::I64],
            false,
        );
        self.module.declare_function(
            "pb_graphic_set_client",
            &IrType::I32,
            &[IrType::I64, IrType::I64],
            false,
        );
        self.module.declare_function(
            "pb_graphic_set_overlap",
            &IrType::I32,
            &[IrType::I32],
            false,
        );
        self.module
            .declare_function("pb_graphic_get_overlap", &IrType::I32, &[], false);
        // batch 180 - the GRAPHIC WINDOW family
        self.module.declare_function(
            "pb_graphic_window_new",
            &IrType::Ptr,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::Ptr,
                IrType::I32,
            ],
            false,
        );
        self.module.declare_function(
            "pb_graphic_window_text",
            &IrType::Ptr,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::Ptr,
                IrType::I32,
            ],
            false,
        );
        self.module
            .declare_function("pb_graphic_window_end", &IrType::I32, &[IrType::Ptr], false);
        self.module.declare_function(
            "pb_graphic_window_hide",
            &IrType::I32,
            &[IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_graphic_window_minimize",
            &IrType::I32,
            &[IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_graphic_window_normalize",
            &IrType::I32,
            &[IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_graphic_window_stabilize",
            &IrType::I32,
            &[IrType::Ptr, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_graphic_window_click",
            &IrType::I32,
            &[IrType::Ptr, IrType::Ptr, IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module
            .declare_function("pb_graphic_attach", &IrType::I32, &[IrType::I64], false);
        self.module
            .declare_function("pb_graphic_detach", &IrType::I32, &[], false);
        self.module.declare_function(
            "pb_xprint_attach",
            &IrType::I32,
            &[IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module
            .declare_function("pb_xprint_close", &IrType::I32, &[], false);
        self.module.declare_function(
            "pb_xprint_get_ppi",
            &IrType::I32,
            &[IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_xprint_get_size",
            &IrType::I32,
            &[IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module
            .declare_function("pb_xprint_get_dc", &IrType::I32, &[IrType::Ptr], false);
        self.module
            .declare_function("pb_xprint_cancel", &IrType::I32, &[], false);
        self.module
            .declare_function("pb_xprint_formfeed", &IrType::I32, &[], false);
        self.module.declare_function(
            "pb_xprint_line",
            &IrType::I32,
            &[IrType::I32, IrType::I32, IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_xprint_box",
            &IrType::I32,
            &[IrType::I32, IrType::I32, IrType::I32, IrType::I32],
            false,
        );
        self.module
            .declare_function("pb_xprint_width", &IrType::I32, &[IrType::I32], false);
        self.module
            .declare_function("pb_xprint_style", &IrType::I32, &[IrType::I32], false);
        self.module
            .declare_function("pb_xprint_set_color", &IrType::I32, &[IrType::I32], false);
        self.module.declare_function(
            "pb_xprint_set_color_rgb",
            &IrType::I32,
            &[IrType::I32, IrType::I32, IrType::I32],
            false,
        );
        self.module
            .declare_function("pb_xprint_get_color", &IrType::I32, &[IrType::Ptr], false);
        self.module.declare_function(
            "pb_xprint_set_pos",
            &IrType::I32,
            &[IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_xprint_get_pos",
            &IrType::I32,
            &[IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_xprint_set_pixel",
            &IrType::I32,
            &[IrType::I32, IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_xprint_get_pixel",
            &IrType::I32,
            &[IrType::I32, IrType::I32, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_xprint_set_textalign",
            &IrType::I32,
            &[IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_xprint_get_textalign",
            &IrType::I32,
            &[IrType::Ptr],
            false,
        );
        self.module
            .declare_function("pb_xprint_get_attach", &IrType::I32, &[IrType::Ptr], false);
        self.module
            .declare_function("pb_xprint_print_str", &IrType::I32, &[IrType::Ptr], false);
        self.module.declare_function(
            "pb_xprint_arc",
            &IrType::I32,
            &[
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::Double,
                IrType::Double,
            ],
            false,
        );
        self.module.declare_function(
            "pb_xprint_ellipse",
            &IrType::I32,
            &[IrType::I32, IrType::I32, IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_xprint_pie",
            &IrType::I32,
            &[
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::Double,
                IrType::Double,
            ],
            false,
        );
        self.module
            .declare_function("pb_xprint_set_font", &IrType::I32, &[IrType::I32], false);
        self.module
            .declare_function("pb_xprint_set_mix", &IrType::I32, &[IrType::I32], false);
        self.module
            .declare_function("pb_xprint_get_mix", &IrType::I32, &[IrType::Ptr], false);
        self.module.declare_function(
            "pb_xprint_set_stretchmode",
            &IrType::I32,
            &[IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_xprint_get_stretchmode",
            &IrType::I32,
            &[IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_xprint_text_size",
            &IrType::I32,
            &[IrType::Ptr, IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_xprint_get_client",
            &IrType::I32,
            &[IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_xprint_get_canvas",
            &IrType::I32,
            &[IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module
            .declare_function("pb_xprint_set_wrap", &IrType::I32, &[IrType::I32], false);
        self.module
            .declare_function("pb_xprint_get_wrap", &IrType::I32, &[IrType::Ptr], false);
        self.module.declare_function(
            "pb_xprint_set_wordwrap",
            &IrType::I32,
            &[IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_xprint_get_wordwrap",
            &IrType::I32,
            &[IrType::Ptr],
            false,
        );
        self.module
            .declare_function("pb_xprint_set_overlap", &IrType::I32, &[IrType::I32], false);
        self.module
            .declare_function("pb_xprint_get_overlap", &IrType::I32, &[IrType::Ptr], false);
        self.module.declare_function(
            "pb_xprint_set_clip",
            &IrType::I32,
            &[IrType::I32, IrType::I32, IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_xprint_get_clip",
            &IrType::I32,
            &[IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_xprint_scale",
            &IrType::I32,
            &[IrType::I32, IrType::I32, IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_xprint_get_scale",
            &IrType::I32,
            &[IrType::Ptr, IrType::Ptr, IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module
            .declare_function("pb_xprint_get_lines", &IrType::I32, &[IrType::Ptr], false);
        self.module.declare_function(
            "pb_xprint_cell_size",
            &IrType::I32,
            &[IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_xprint_chr_size",
            &IrType::I32,
            &[IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_xprint_copy",
            &IrType::I32,
            &[
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );
        self.module.declare_function(
            "pb_xprint_polygon",
            &IrType::I32,
            &[IrType::Ptr, IrType::I32, IrType::I64],
            false,
        );
        self.module.declare_function(
            "pb_xprint_polyline",
            &IrType::I32,
            &[IrType::Ptr, IrType::I32, IrType::I64],
            false,
        );
        self.module
            .declare_function("pb_xprint_set_copies", &IrType::I32, &[IrType::I32], false);
        self.module
            .declare_function("pb_xprint_get_copies", &IrType::I32, &[IrType::Ptr], false);
        self.module.declare_function(
            "pb_xprint_set_orientation",
            &IrType::I32,
            &[IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_xprint_get_orientation",
            &IrType::I32,
            &[IrType::Ptr],
            false,
        );
        self.module
            .declare_function("pb_xprint_set_quality", &IrType::I32, &[IrType::I32], false);
        self.module
            .declare_function("pb_xprint_get_quality", &IrType::I32, &[IrType::Ptr], false);
        self.module
            .declare_function("pb_xprint_set_duplex", &IrType::I32, &[IrType::I32], false);
        self.module
            .declare_function("pb_xprint_get_duplex", &IrType::I32, &[IrType::Ptr], false);
        self.module
            .declare_function("pb_xprint_set_collate", &IrType::I32, &[IrType::I32], false);
        self.module
            .declare_function("pb_xprint_get_collate", &IrType::I32, &[IrType::Ptr], false);
        self.module.declare_function(
            "pb_xprint_set_colormode",
            &IrType::I32,
            &[IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_xprint_get_colormode",
            &IrType::I32,
            &[IrType::Ptr],
            false,
        );
        self.module
            .declare_function("pb_xprint_set_pages", &IrType::I32, &[IrType::I32], false);
        self.module
            .declare_function("pb_xprint_get_pages", &IrType::I32, &[IrType::Ptr], false);
        self.module.declare_function(
            "pb_xprint_set_cell",
            &IrType::I32,
            &[IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_xprint_get_cell",
            &IrType::I32,
            &[IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_xprint_get_selection",
            &IrType::I32,
            &[IrType::Ptr],
            false,
        );
        self.module
            .declare_function("pb_xprint_set_paper", &IrType::I32, &[IrType::I32], false);
        self.module
            .declare_function("pb_xprint_get_paper", &IrType::I32, &[IrType::Ptr], false);
        self.module
            .declare_function("pb_xprint_set_tray", &IrType::I32, &[IrType::I32], false);
        self.module
            .declare_function("pb_xprint_get_tray", &IrType::I32, &[IrType::Ptr], false);
        self.module.declare_function(
            "pb_resource_save_file",
            &IrType::I32,
            &[IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module
            .declare_function("pb_xprint_get_papers", &IrType::I32, &[IrType::Ptr], false);
        self.module
            .declare_function("pb_xprint_get_trays", &IrType::I32, &[IrType::Ptr], false);
        self.module
            .declare_function("pb_xprint_preview", &IrType::I32, &[IrType::I32], false);
        self.module
            .declare_function("pb_xprint_preview_close", &IrType::I32, &[], false);
        self.module
            .declare_function("pb_xprint_render", &IrType::I32, &[], false);
        self.module.declare_function(
            "pb_xprint_split",
            &IrType::I32,
            &[IrType::Ptr, IrType::I32, IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_xprint_stretch",
            &IrType::I32,
            &[
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );
        self.module.declare_function(
            "pb_xprint_imagelist",
            &IrType::I32,
            &[IrType::I32, IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_tcp_notify",
            &IrType::I32,
            &[IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_udp_notify",
            &IrType::I32,
            &[IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_dialog_get_size",
            &IrType::Void,
            &[IrType::Ptr, IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_dialog_set_size",
            &IrType::Void,
            &[IrType::Ptr, IrType::I64, IrType::I64],
            false,
        );
        // Batch 165 - the DIALOG statement family.
        self.module.declare_function(
            "pb_dialog_enable",
            &IrType::Void,
            &[IrType::Ptr, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_dialog_show",
            &IrType::Void,
            &[IrType::Ptr, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_dialog_stabilize",
            &IrType::Void,
            &[IrType::Ptr, IrType::I32],
            false,
        );
        self.module
            .declare_function("pb_dialog_redraw", &IrType::Void, &[IrType::Ptr], false);
        self.module.declare_function(
            "pb_dialog_send",
            &IrType::I64,
            &[IrType::Ptr, IrType::I64, IrType::I64, IrType::I64],
            false,
        );
        self.module.declare_function(
            "pb_dialog_post",
            &IrType::Void,
            &[IrType::Ptr, IrType::I64, IrType::I64, IrType::I64],
            false,
        );
        self.module.declare_function(
            "pb_dialog_set_user",
            &IrType::Void,
            &[IrType::Ptr, IrType::I64, IrType::I64],
            false,
        );
        self.module.declare_function(
            "pb_dialog_get_user",
            &IrType::I64,
            &[IrType::Ptr, IrType::I64],
            false,
        );
        self.module.declare_function(
            "pb_dialog_set_icon",
            &IrType::Void,
            &[IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_dialog_get_client",
            &IrType::Void,
            &[IrType::Ptr, IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_dialog_set_client",
            &IrType::Void,
            &[IrType::Ptr, IrType::I64, IrType::I64],
            false,
        );
        self.module.declare_function(
            "pb_dialog_get_loc",
            &IrType::Void,
            &[IrType::Ptr, IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_dialog_set_loc",
            &IrType::Void,
            &[IrType::Ptr, IrType::I64, IrType::I64],
            false,
        );
        self.module.declare_function(
            "pb_dialog_pixels",
            &IrType::Void,
            &[
                IrType::Ptr,
                IrType::I64,
                IrType::I64,
                IrType::Ptr,
                IrType::Ptr,
            ],
            false,
        );
        self.module.declare_function(
            "pb_dialog_units",
            &IrType::Void,
            &[
                IrType::Ptr,
                IrType::I64,
                IrType::I64,
                IrType::Ptr,
                IrType::Ptr,
            ],
            false,
        );

        // Batch 175 - CONTROL geometry: by (hDlg, id), dialog units.
        for (fname, params) in [
            (
                "pb_control_get_client",
                &[IrType::Ptr, IrType::I64, IrType::Ptr, IrType::Ptr][..],
            ),
            (
                "pb_control_get_size",
                &[IrType::Ptr, IrType::I64, IrType::Ptr, IrType::Ptr][..],
            ),
            (
                "pb_control_get_loc",
                &[IrType::Ptr, IrType::I64, IrType::Ptr, IrType::Ptr][..],
            ),
            (
                "pb_control_set_client",
                &[IrType::Ptr, IrType::I64, IrType::I64, IrType::I64][..],
            ),
            (
                "pb_control_set_loc",
                &[IrType::Ptr, IrType::I64, IrType::I64, IrType::I64][..],
            ),
            (
                "pb_control_set_size",
                &[IrType::Ptr, IrType::I64, IrType::I64, IrType::I64][..],
            ),
        ] {
            self.module
                .declare_function(fname, &IrType::Void, params, false);
        }
        // Batch 176 - CONTROL messages / state: by (hDlg, id).
        for (fname, ret, params) in [
            (
                "pb_control_handle",
                &IrType::Ptr,
                &[IrType::Ptr, IrType::I64][..],
            ),
            (
                "pb_control_send",
                &IrType::I64,
                &[
                    IrType::Ptr,
                    IrType::I64,
                    IrType::I64,
                    IrType::I64,
                    IrType::I64,
                ][..],
            ),
            (
                "pb_control_post",
                &IrType::Void,
                &[
                    IrType::Ptr,
                    IrType::I64,
                    IrType::I64,
                    IrType::I64,
                    IrType::I64,
                ][..],
            ),
            (
                "pb_control_redraw",
                &IrType::Void,
                &[IrType::Ptr, IrType::I64][..],
            ),
            (
                "pb_control_set_focus",
                &IrType::Void,
                &[IrType::Ptr, IrType::I64][..],
            ),
            (
                "pb_control_set_font",
                &IrType::Void,
                &[IrType::Ptr, IrType::I64, IrType::I64][..],
            ),
            (
                "pb_control_show_state",
                &IrType::I64,
                &[IrType::Ptr, IrType::I64, IrType::I64][..],
            ),
            (
                "pb_control_normalize",
                &IrType::Void,
                &[IrType::Ptr, IrType::I64][..],
            ),
            (
                "pb_control_set_user",
                &IrType::Void,
                &[IrType::Ptr, IrType::I64, IrType::I64, IrType::I64][..],
            ),
            (
                "pb_control_get_user",
                &IrType::I64,
                &[IrType::Ptr, IrType::I64, IrType::I64][..],
            ),
            (
                "pb_control_add_option",
                &IrType::Ptr,
                &[
                    IrType::Ptr,
                    IrType::I32,
                    IrType::Ptr,
                    IrType::I32,
                    IrType::I32,
                    IrType::I32,
                    IrType::I32,
                ][..],
            ),
            (
                "pb_control_add_check3state",
                &IrType::Ptr,
                &[
                    IrType::Ptr,
                    IrType::I32,
                    IrType::Ptr,
                    IrType::I32,
                    IrType::I32,
                    IrType::I32,
                    IrType::I32,
                ][..],
            ),
            (
                "pb_control_add_frame",
                &IrType::Ptr,
                &[
                    IrType::Ptr,
                    IrType::I32,
                    IrType::Ptr,
                    IrType::I32,
                    IrType::I32,
                    IrType::I32,
                    IrType::I32,
                ][..],
            ),
            (
                "pb_control_add_textbox",
                &IrType::Ptr,
                &[
                    IrType::Ptr,
                    IrType::I32,
                    IrType::Ptr,
                    IrType::I32,
                    IrType::I32,
                    IrType::I32,
                    IrType::I32,
                ][..],
            ),
            (
                "pb_control_add_line",
                &IrType::Ptr,
                &[
                    IrType::Ptr,
                    IrType::I32,
                    IrType::Ptr,
                    IrType::I32,
                    IrType::I32,
                    IrType::I32,
                    IrType::I32,
                ][..],
            ),
            (
                "pb_control_add_image",
                &IrType::Ptr,
                &[
                    IrType::Ptr,
                    IrType::I32,
                    IrType::Ptr,
                    IrType::I32,
                    IrType::I32,
                    IrType::I32,
                    IrType::I32,
                    IrType::I32,
                    IrType::I32,
                    IrType::I32,
                    IrType::I32,
                ][..],
            ),
            (
                "pb_control_set_image",
                &IrType::Void,
                &[
                    IrType::Ptr,
                    IrType::I32,
                    IrType::Ptr,
                    IrType::I32,
                    IrType::I32,
                ][..],
            ),
            (
                "pb_control_set_option",
                &IrType::Void,
                &[IrType::Ptr, IrType::I64, IrType::I64, IrType::I64][..],
            ),
        ] {
            self.module.declare_function(fname, ret, params, false);
        }
        self.module.declare_function(
            "pb_dialog_set_color",
            &IrType::Void,
            &[IrType::Ptr, IrType::I64, IrType::I64],
            false,
        );
        self.module.declare_function(
            "pb_dialog_default_font",
            &IrType::Void,
            &[IrType::Ptr, IrType::I32, IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_array_select",
            &IrType::I32,
            &[IrType::Ptr, IrType::I32, IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_array_tagarray",
            &IrType::I32,
            &[IrType::Ptr, IrType::I32, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_array_tagarray_erase",
            &IrType::I32,
            &[IrType::Ptr, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_xprint_get_margin",
            &IrType::I32,
            &[IrType::Ptr, IrType::Ptr, IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_display_openfile",
            &IrType::I32,
            &[IrType::Ptr, IrType::Ptr, IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_display_savefile",
            &IrType::I32,
            &[IrType::Ptr, IrType::Ptr, IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module
            .declare_function("pb_display_color", &IrType::I32, &[IrType::Ptr], false);
        self.module
            .declare_function("pb_display_font", &IrType::I32, &[IrType::Ptr], false);
        self.module.declare_function(
            "pb_display_browse",
            &IrType::I32,
            &[IrType::Ptr, IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module
            .declare_function("pb_class_create", &IrType::Ptr, &[IrType::Ptr], false);
        self.module
            .declare_function("pb_class_destroy", &IrType::Void, &[IrType::Ptr], false);
        self.module.declare_function(
            "pb_method_call",
            &IrType::I32,
            &[IrType::Ptr, IrType::Ptr],
            false,
        );
        // OOP event bus (batch 222): per-class handler table + subscribe bus
        self.module
            .declare_function("pb_events_new_table", &IrType::Ptr, &[], false);
        self.module.declare_function(
            "pb_events_add",
            &IrType::Void,
            &[IrType::Ptr, IrType::I32, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_events_attach",
            &IrType::I32,
            &[IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module
            .declare_function("pb_events_detach", &IrType::I32, &[IrType::Ptr], false);
        self.module.declare_function(
            "pb_raise_event",
            &IrType::I32,
            &[IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_array_redim_incr",
            &IrType::I32,
            &[IrType::Ptr, IrType::I32, IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_array_redim_decr",
            &IrType::I32,
            &[IrType::Ptr, IrType::I32, IrType::I32, IrType::I32],
            false,
        );
        self.module
            .declare_function("pb_graphic_clear", &IrType::I32, &[IrType::I32], false);
        self.module.declare_function(
            "pb_graphic_line",
            &IrType::I32,
            &[
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );
        self.module.declare_function(
            "pb_graphic_box",
            &IrType::I32,
            &[
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );
        self.module.declare_function(
            "pb_graphic_ellipse",
            &IrType::I32,
            &[
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );
        self.module
            .declare_function("pb_graphic_width", &IrType::I32, &[IrType::I32], false);
        self.module.declare_function(
            "pb_graphic_arc",
            &IrType::I32,
            &[
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );
        self.module.declare_function(
            "pb_graphic_pie",
            &IrType::I32,
            &[
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );
        self.module.declare_function(
            "pb_graphic_polyline",
            &IrType::I32,
            &[IrType::Ptr, IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_graphic_paint",
            &IrType::I32,
            &[
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );
        self.module
            .declare_function("pb_graphic_style", &IrType::I32, &[IrType::I32], false);
        self.module
            .declare_function("pb_graphic_save", &IrType::I32, &[IrType::Ptr], false);
        self.module.declare_function(
            "pb_graphic_color",
            &IrType::I32,
            &[IrType::I32, IrType::I32],
            false,
        );
        self.module
            .declare_function("pb_graphic_print_str", &IrType::I32, &[IrType::Ptr], false);
        self.module.declare_function(
            "pb_graphic_get_pixel",
            &IrType::I32,
            &[IrType::I32, IrType::I32, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_graphic_copy",
            &IrType::I32,
            &[
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );
        self.module.declare_function(
            "pb_graphic_circle",
            &IrType::I32,
            &[IrType::I32, IrType::I32, IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_graphic_polygon",
            &IrType::I32,
            &[IrType::Ptr, IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_graphic_get_client",
            &IrType::I32,
            &[IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_graphic_get_loc",
            &IrType::I32,
            &[IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_graphic_set_pixel",
            &IrType::I32,
            &[IrType::I32, IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_graphic_get_size",
            &IrType::I32,
            &[IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_graphic_set_textalign",
            &IrType::I32,
            &[IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_graphic_get_textalign",
            &IrType::I32,
            &[IrType::Ptr],
            false,
        );
        self.module
            .declare_function("pb_graphic_get_ppi", &IrType::I32, &[IrType::Ptr], false);
        self.module.declare_function(
            "pb_graphic_get_pos",
            &IrType::I32,
            &[IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_graphic_get_clip",
            &IrType::I32,
            &[IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_graphic_get_view",
            &IrType::I32,
            &[IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_graphic_set_view",
            &IrType::I32,
            &[IrType::Float, IrType::Float],
            false,
        );
        self.module
            .declare_function("pb_graphic_get_lines", &IrType::I32, &[IrType::Ptr], false);
        self.module
            .declare_function("pb_graphic_get_wrap", &IrType::I32, &[IrType::Ptr], false);
        self.module
            .declare_function("pb_graphic_set_wrap", &IrType::I32, &[IrType::I32], false);
        self.module
            .declare_function("pb_graphic_get_bits", &IrType::I32, &[IrType::Ptr], false);
        self.module
            .declare_function("pb_graphic_set_bits", &IrType::I32, &[IrType::Ptr], false);
        self.module.declare_function(
            "pb_graphic_get_scale",
            &IrType::I32,
            &[IrType::Ptr, IrType::Ptr, IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_graphic_scale",
            &IrType::I32,
            &[IrType::Float, IrType::Float, IrType::Float, IrType::Float],
            false,
        );
        self.module
            .declare_function("pb_graphic_scale_pixels", &IrType::I32, &[], false);
        self.module.declare_function(
            "pb_graphic_set_autosize",
            &IrType::I32,
            &[IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_graphic_set_size",
            &IrType::I32,
            &[IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_graphic_set_clip",
            &IrType::I32,
            &[IrType::Float, IrType::Float, IrType::Float, IrType::Float],
            false,
        );
        self.module.declare_function(
            "pb_graphic_set_virtual",
            &IrType::I32,
            &[IrType::I32, IrType::I32],
            false,
        );
        self.module
            .declare_function("pb_graphic_set_fixed", &IrType::I32, &[], false);
        self.module
            .declare_function("pb_graphic_set_font", &IrType::I32, &[IrType::I64], false);
        self.module.declare_function(
            "pb_graphic_set_wordwrap",
            &IrType::I32,
            &[IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_graphic_get_wordwrap",
            &IrType::I32,
            &[IrType::Ptr],
            false,
        );

        self.module.declare_function(
            "pb_graphic_set_pos",
            &IrType::I32,
            &[IrType::Float, IrType::Float],
            false,
        );
        self.module.declare_function(
            "pb_graphic_text_size",
            &IrType::I32,
            &[IrType::Ptr, IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_graphic_get_stretchmode",
            &IrType::I32,
            &[IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_graphic_set_stretchmode",
            &IrType::I32,
            &[IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_graphic_get_caption",
            &IrType::I32,
            &[IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_graphic_set_caption",
            &IrType::I32,
            &[IrType::Ptr],
            false,
        );
        self.module
            .declare_function("pb_graphic_get_canvas", &IrType::I64, &[], false);
        self.module
            .declare_function("pb_graphic_get_dc", &IrType::I64, &[], false);
        self.module
            .declare_function("pb_graphic_set_mix", &IrType::I32, &[IrType::I32], false);
        self.module
            .declare_function("pb_graphic_get_mix", &IrType::I32, &[IrType::Ptr], false);
        self.module.declare_function(
            "pb_graphic_bitmap_load",
            &IrType::I64,
            &[IrType::Ptr],
            false,
        );
        self.module
            .declare_function("pb_graphic_instat", &IrType::I32, &[], false);
        self.module
            .declare_function("pb_graphic_input_flush", &IrType::Void, &[], false);
        self.module
            .declare_function("pb_graphic_inkey", &IrType::Ptr, &[], false);
        self.module.declare_function(
            "pb_graphic_waitkey",
            &IrType::Ptr,
            &[IrType::Ptr, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_graphic_split",
            &IrType::Void,
            &[IrType::Ptr, IrType::I32, IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_graphic_split_word",
            &IrType::Void,
            &[IrType::Ptr, IrType::I32, IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module
            .declare_function("pb_graphic_line_input", &IrType::Ptr, &[IrType::Ptr], false);
        self.module.declare_function(
            "pb_graphic_input_begin",
            &IrType::Void,
            &[IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_graphic_input_field",
            &IrType::Ptr,
            &[IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_graphic_input_field_num",
            &IrType::Double,
            &[IrType::I32],
            false,
        );
        self.module
            .declare_function("pb_graphic_bitmap_capture", &IrType::I64, &[], false);
        self.module.declare_function(
            "pb_graphic_set_scrolltext",
            &IrType::Void,
            &[IrType::I32],
            false,
        );
        self.module
            .declare_function("pb_graphic_get_scrolltext", &IrType::I32, &[], false);
        self.module.declare_function(
            "pb_graphic_imagelist",
            &IrType::Void,
            &[
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );
        self.module.declare_function(
            "pb_graphic_render",
            &IrType::Void,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );
        self.module.declare_function(
            "pb_graphic_render_icon",
            &IrType::Void,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );
        self.module.declare_function(
            "pb_graphic_stretch",
            &IrType::Void,
            &[
                IrType::I64,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );
        self.module.declare_function(
            "pb_graphic_stretch_page",
            &IrType::Void,
            &[IrType::I64, IrType::I32, IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_graphic_chr_size",
            &IrType::I32,
            &[IrType::Ptr, IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_graphic_cell_size",
            &IrType::I32,
            &[IrType::I32, IrType::I32, IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_graphic_cell",
            &IrType::I32,
            &[IrType::I32, IrType::I32, IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_pathscan",
            &IrType::Ptr,
            &[IrType::Ptr, IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module
            .declare_function("pb_monthname", &IrType::Ptr, &[IrType::I64], false);
        self.module
            .declare_function("pb_data_count", &IrType::I64, &[], false);
        self.module
            .declare_function("pb_thread_count", &IrType::I64, &[], false);
        self.module
            .declare_function("pb_chr_to_oem", &IrType::Ptr, &[IrType::Ptr], false);
        self.module
            .declare_function("pb_oem_to_chr", &IrType::Ptr, &[IrType::Ptr], false);
        self.module
            .declare_function("pb_chr_to_utf8", &IrType::Ptr, &[IrType::Ptr], false);
        self.module
            .declare_function("pb_utf8_to_chr", &IrType::Ptr, &[IrType::Ptr], false);
        self.module.declare_function(
            "pb_desktop_get_client",
            &IrType::Void,
            &[IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_desktop_get_loc",
            &IrType::Void,
            &[IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_desktop_get_ppi",
            &IrType::Void,
            &[IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module
            .declare_function("pb_globalmem_alloc", &IrType::I32, &[IrType::I32], false);
        self.module
            .declare_function("pb_globalmem_free", &IrType::I32, &[IrType::I32], false);
        self.module
            .declare_function("pb_globalmem_lock", &IrType::Ptr, &[IrType::I32], false);
        self.module
            .declare_function("pb_globalmem_size", &IrType::I32, &[IrType::I32], false);
        self.module
            .declare_function("pb_globalmem_unlock", &IrType::I32, &[IrType::I32], false);
        self.module
            .declare_function("pb_mouseptr", &IrType::I32, &[IrType::I32], false);
        self.module
            .declare_function("pb_ucodepage", &IrType::I32, &[IrType::I32], false);
        self.module
            .declare_function("pb_ucode", &IrType::Ptr, &[IrType::Ptr, IrType::I32], false);
        self.module
            .declare_function("pb_acode", &IrType::Ptr, &[IrType::Ptr, IrType::I32], false);
        self.module
            .declare_function("pb_isinfinite", &IrType::I32, &[IrType::Double], false);
        self.module
            .declare_function("pb_isnormal", &IrType::I32, &[IrType::Double], false);
        self.module
            .declare_function("pb_chdrive", &IrType::I32, &[IrType::Ptr], false);
        self.module
            .declare_function("pb_seteof", &IrType::I32, &[IrType::I32], false);
        self.module
            .declare_function("pb_play_wave", &IrType::I32, &[IrType::Ptr], false);
        self.module.declare_function(
            "pb_array_reverse",
            &IrType::Void,
            &[IrType::Ptr, IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_put_string",
            &IrType::I32,
            &[IrType::I32, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_get_string",
            &IrType::I32,
            &[IrType::I32, IrType::I64, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_put_wstring",
            &IrType::I32,
            &[IrType::I32, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_get_wstring",
            &IrType::I32,
            &[IrType::I32, IrType::I64, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_cset",
            &IrType::Void,
            &[IrType::Ptr, IrType::Ptr, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_cset_buf",
            &IrType::Void,
            &[IrType::Ptr, IrType::Ptr, IrType::I32],
            false,
        );
        self.module
            .declare_function("pb_mkbyt", &IrType::Ptr, &[IrType::I32], false);
        self.module.declare_function(
            "pb_shift_left",
            &IrType::I64,
            &[IrType::I64, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_shift_right",
            &IrType::I64,
            &[IrType::I64, IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_rotate_left",
            &IrType::I64,
            &[IrType::I64, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_rotate_right",
            &IrType::I64,
            &[IrType::I64, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_play_sound",
            &IrType::I32,
            &[IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_split",
            &IrType::Void,
            &[IrType::Ptr, IrType::I32, IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_array_shuffle",
            &IrType::Void,
            &[IrType::Ptr, IrType::I32, IrType::I32],
            false,
        );
        self.module
            .declare_function("pb_data_append", &IrType::Void, &[IrType::Ptr], false);
        self.module
            .declare_function("pb_read_data_str", &IrType::Ptr, &[], false);
        self.module
            .declare_function("pb_read_data_num", &IrType::Double, &[], false);
        self.module
            .declare_function("pb_data_reset", &IrType::Void, &[], false);
        self.module.declare_function(
            "pb_array_scan_num",
            &IrType::I64,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::I64,
                IrType::I64,
                IrType::I64,
                IrType::I64,
                IrType::I32,
            ],
            false,
        );
        self.module.declare_function(
            "pb_array_scan_str",
            &IrType::I64,
            &[
                IrType::Ptr,
                IrType::I64,
                IrType::I64,
                IrType::I64,
                IrType::Ptr,
            ],
            false,
        );
        self.module.declare_function(
            "pb_array_insert_num",
            &IrType::Void,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::I64,
                IrType::I64,
                IrType::I64,
            ],
            false,
        );
        self.module.declare_function(
            "pb_array_insert_str",
            &IrType::Void,
            &[IrType::Ptr, IrType::I64, IrType::I64, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_array_delete",
            &IrType::Void,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::I64,
                IrType::I64,
                IrType::I64,
                IrType::I32,
            ],
            false,
        );
        self.module.declare_function(
            "pb_array_arrayix",
            &IrType::Void,
            &[IrType::Ptr, IrType::I32, IrType::I64, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_filescan",
            &IrType::Void,
            &[IrType::I32, IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module
            .declare_function("pb_clipboard_set_text", &IrType::I32, &[IrType::Ptr], false);
        self.module
            .declare_function("pb_clipboard_get_text", &IrType::Ptr, &[], false);
        self.module
            .declare_function("pb_clipboard_reset", &IrType::I32, &[], false);
        self.module
            .declare_function("pb_input_flush", &IrType::Void, &[], false);
        self.module.declare_function(
            "pb_array_add",
            &IrType::Void,
            &[
                IrType::Ptr,
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I64,
            ],
            false,
        );
        self.module.declare_function(
            "pb_array_copy",
            &IrType::Void,
            &[IrType::Ptr, IrType::Ptr, IrType::I32, IrType::I64],
            false,
        );
        self.module.declare_function(
            "pb_array_swap",
            &IrType::Void,
            &[IrType::Ptr, IrType::Ptr, IrType::I32, IrType::I64],
            false,
        );
        self.module.declare_function(
            "pb_array_unique",
            &IrType::I64,
            &[IrType::Ptr, IrType::I32, IrType::I64, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_host_addr",
            &IrType::Void,
            &[IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module
            .declare_function("pb_host_name", &IrType::Ptr, &[IrType::I32], false);
        self.module
            .declare_function("pb_lof", &IrType::I64, &[IrType::I32], false);
        self.module
            .declare_function("pb_loc", &IrType::I64, &[IrType::I32], false);
        self.module
            .declare_function("pb_process_get_priority", &IrType::I32, &[], false);
        self.module.declare_function(
            "pb_process_set_priority",
            &IrType::I32,
            &[IrType::I32],
            false,
        );
        self.module
            .declare_function("pb_peek8", &IrType::I32, &[IrType::Ptr], false);
        self.module
            .declare_function("pb_peek16", &IrType::I32, &[IrType::Ptr], false);
        self.module
            .declare_function("pb_peek32", &IrType::I32, &[IrType::Ptr], false);
        self.module
            .declare_function("pb_peek64", &IrType::I64, &[IrType::Ptr], false);
        self.module
            .declare_function("pb_peekf", &IrType::Float, &[IrType::Ptr], false);
        self.module
            .declare_function("pb_peekd", &IrType::Double, &[IrType::Ptr], false);
        self.module.declare_function(
            "pb_tcp_open",
            &IrType::I32,
            &[
                IrType::I32,
                IrType::I32,
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );
        self.module.declare_function(
            "pb_tcp_accept",
            &IrType::I32,
            &[IrType::I32, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_tcp_send",
            &IrType::I32,
            &[IrType::I32, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_tcp_recv",
            &IrType::I32,
            &[IrType::I32, IrType::I32, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_tcp_line_input",
            &IrType::I32,
            &[IrType::I32, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_tcp_print",
            &IrType::I32,
            &[IrType::I32, IrType::Ptr, IrType::I32],
            false,
        );
        self.module
            .declare_function("pb_tcp_close", &IrType::I32, &[IrType::I32], false);
        self.module.declare_function(
            "pb_udp_open",
            &IrType::I32,
            &[IrType::I32, IrType::I32, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_udp_send",
            &IrType::I32,
            &[IrType::I32, IrType::I32, IrType::I32, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_udp_send_str",
            &IrType::I32,
            &[IrType::I32, IrType::Ptr, IrType::I32, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_udp_recv",
            &IrType::I32,
            &[IrType::I32, IrType::Ptr, IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module
            .declare_function("pb_udp_close", &IrType::I32, &[IrType::I32], false);
        self.module.declare_function(
            "pb_poke8",
            &IrType::Void,
            &[IrType::Ptr, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_poke16",
            &IrType::Void,
            &[IrType::Ptr, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_poke32",
            &IrType::Void,
            &[IrType::Ptr, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_poke64",
            &IrType::Void,
            &[IrType::Ptr, IrType::I64],
            false,
        );
        self.module.declare_function(
            "pb_pokef",
            &IrType::Void,
            &[IrType::Ptr, IrType::Float],
            false,
        );
        self.module.declare_function(
            "pb_poked",
            &IrType::Void,
            &[IrType::Ptr, IrType::Double],
            false,
        );
        self.module
            .declare_function("pb_reset", &IrType::Void, &[], false);
        self.module
            .declare_function("pb_flush", &IrType::Void, &[IrType::I32], false);
        self.module
            .declare_function("pb_name", &IrType::I32, &[IrType::Ptr, IrType::Ptr], false);
        self.module
            .declare_function("pb_write_file_begin", &IrType::Void, &[IrType::I32], false);
        self.module.declare_function(
            "pb_write_file_str",
            &IrType::Void,
            &[IrType::I32, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_write_file_int",
            &IrType::Void,
            &[IrType::I32, IrType::I64],
            false,
        );
        self.module.declare_function(
            "pb_write_file_dbl",
            &IrType::Void,
            &[IrType::I32, IrType::Double],
            false,
        );
        self.module.declare_function(
            "pb_write_file_newline",
            &IrType::Void,
            &[IrType::I32],
            false,
        );
        self.module
            .declare_function("pb_seek", &IrType::Void, &[IrType::I32, IrType::I64], false);
        self.module.declare_function(
            "pb_lock",
            &IrType::Void,
            &[IrType::I32, IrType::I64, IrType::I64],
            false,
        );
        self.module.declare_function(
            "pb_unlock",
            &IrType::Void,
            &[IrType::I32, IrType::I64, IrType::I64],
            false,
        );
        // COMM serial port (batch 21)
        self.module.declare_function(
            "pb_comm_open",
            &IrType::I32,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
            ],
            false,
        );
        self.module
            .declare_function("pb_comm_close", &IrType::I32, &[IrType::I32], false);
        self.module
            .declare_function("pb_comm_reset", &IrType::Void, &[], false);
        self.module.declare_function(
            "pb_comm_send",
            &IrType::I32,
            &[IrType::I32, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_comm_recv",
            &IrType::I32,
            &[IrType::I32, IrType::I64, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_comm_line_input",
            &IrType::I32,
            &[IrType::I32, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_comm_print_str",
            &IrType::Void,
            &[IrType::I32, IrType::Ptr],
            false,
        );
        self.module.declare_function(
            "pb_comm_print_int",
            &IrType::Void,
            &[IrType::I32, IrType::I64],
            false,
        );
        self.module.declare_function(
            "pb_comm_print_dbl",
            &IrType::Void,
            &[IrType::I32, IrType::Double],
            false,
        );
        self.module.declare_function(
            "pb_comm_set",
            &IrType::I32,
            &[IrType::I32, IrType::Ptr, IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_comm_timeout",
            &IrType::I32,
            &[IrType::I32, IrType::I64],
            false,
        );
        // THREAD (batch 21)
        self.module.declare_function(
            "pb_thread_create",
            &IrType::I32,
            &[IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module
            .declare_function("pb_thread_close", &IrType::I32, &[IrType::I32], false);
        self.module
            .declare_function("pb_thread_suspend", &IrType::I32, &[IrType::I32], false);
        self.module
            .declare_function("pb_thread_resume", &IrType::I32, &[IrType::I32], false);
        self.module
            .declare_function("pb_thread_status", &IrType::I32, &[IrType::I32], false);
        self.module.declare_function(
            "pb_thread_get_priority",
            &IrType::I32,
            &[IrType::I32],
            false,
        );
        self.module.declare_function(
            "pb_thread_set_priority",
            &IrType::I32,
            &[IrType::I32, IrType::I32],
            false,
        );
        // LPRINT (batch 22)
        self.module
            .declare_function("pb_lprint_attach", &IrType::I32, &[IrType::Ptr], false);
        self.module
            .declare_function("pb_lprint_close", &IrType::Void, &[], false);
        self.module
            .declare_function("pb_lprint_flush", &IrType::Void, &[], false);
        self.module
            .declare_function("pb_lprint_formfeed", &IrType::Void, &[], false);
        self.module
            .declare_function("pb_lprint_bstr", &IrType::Void, &[IrType::Ptr], false);
        self.module
            .declare_function("pb_lprint_int", &IrType::Void, &[IrType::I32], false);
        self.module
            .declare_function("pb_lprint_i64", &IrType::Void, &[IrType::I64], false);
        self.module
            .declare_function("pb_lprint_dbl", &IrType::Void, &[IrType::Double], false);
        self.module
            .declare_function("pb_lprint_crlf", &IrType::Void, &[], false);
        self.module
            .declare_function("pb_lprint_tab", &IrType::Void, &[], false);
        // TRACE (batch 22)
        self.module
            .declare_function("pb_trace_new", &IrType::I32, &[IrType::Ptr], false);
        self.module
            .declare_function("pb_trace_on", &IrType::Void, &[], false);
        self.module
            .declare_function("pb_trace_off", &IrType::Void, &[], false);
        self.module
            .declare_function("pb_trace_print", &IrType::Void, &[IrType::Ptr], false);
        self.module
            .declare_function("pb_trace_close", &IrType::Void, &[], false);
        // IMPORT (batch 22)
        self.module.declare_function(
            "pb_import_addr",
            &IrType::I32,
            &[IrType::Ptr, IrType::Ptr, IrType::Ptr, IrType::Ptr],
            false,
        );
        self.module
            .declare_function("pb_import_close", &IrType::Void, &[IrType::Ptr], false);
        self.module
            .declare_dllimport("Sleep", &IrType::Void, &[IrType::I32]);
        self.module
            .declare_dllimport("Beep", &IrType::Void, &[IrType::I32, IrType::I32]);
        // PB-compatible ERR system variable (provided by pb_runtime.c)
        self.module.declare_external_global("pb_err", &IrType::I32);
        // ON ERROR GOTO runtime state (provided by pb_runtime.c)
        self.module
            .declare_external_global("pb_err_stmt_id", &IrType::I32);
        self.module
            .declare_external_global("pb_err_active", &IrType::I32);
        self.module
            .declare_dllimport("GetCommandLineA", &IrType::Ptr, &[]);
        self.module
            .declare_dllimport("GetSystemMetrics", &IrType::I32, &[IrType::I32]);
        self.module.declare_dllimport(
            "MessageBoxA",
            &IrType::I32,
            &[IrType::Ptr, IrType::Ptr, IrType::Ptr, IrType::I32],
        );
        self.module.declare_dllimport(
            "ShellExecuteA",
            &IrType::Ptr,
            &[
                IrType::Ptr,
                IrType::Ptr,
                IrType::Ptr,
                IrType::Ptr,
                IrType::Ptr,
                IrType::I32,
            ],
        );
        self.module.declare_dllimport(
            "CreateFileMappingA",
            &IrType::Ptr,
            &[
                IrType::Ptr,
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::Ptr,
            ],
        );
        self.module.declare_dllimport(
            "MapViewOfFile",
            &IrType::Ptr,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::I32,
                IrType::I32,
            ],
        );
        self.module
            .declare_dllimport("UnmapViewOfFile", &IrType::I32, &[IrType::Ptr]);
        self.module
            .declare_dllimport("CloseHandle", &IrType::I32, &[IrType::Ptr]);
        self.module
            .declare_dllimport("GetFileSize", &IrType::I32, &[IrType::Ptr, IrType::Ptr]);
        self.module.declare_dllimport(
            "CreateDCA",
            &IrType::Ptr,
            &[IrType::Ptr, IrType::Ptr, IrType::Ptr, IrType::Ptr],
        );
        self.module.declare_dllimport(
            "TextOutA",
            &IrType::I32,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::Ptr,
                IrType::I32,
            ],
        );
        self.module
            .declare_dllimport("DeleteDC", &IrType::I32, &[IrType::Ptr]);
        self.module.declare_dllimport(
            "CreateFileA",
            &IrType::Ptr,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::Ptr,
                IrType::I32,
                IrType::I32,
                IrType::Ptr,
            ],
        );
        self.module.declare_dllimport(
            "ReadFile",
            &IrType::I32,
            &[
                IrType::Ptr,
                IrType::Ptr,
                IrType::I32,
                IrType::Ptr,
                IrType::Ptr,
            ],
        );
        self.module.declare_dllimport(
            "WriteFile",
            &IrType::I32,
            &[
                IrType::Ptr,
                IrType::Ptr,
                IrType::I32,
                IrType::Ptr,
                IrType::Ptr,
            ],
        );
        self.module.declare_dllimport(
            "CreateThread",
            &IrType::Ptr,
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::Ptr,
                IrType::Ptr,
                IrType::I32,
                IrType::Ptr,
            ],
        );
        self.module
            .declare_dllimport("SuspendThread", &IrType::I32, &[IrType::Ptr]);
        self.module
            .declare_dllimport("ResumeThread", &IrType::I32, &[IrType::Ptr]);
        self.module
            .declare_dllimport("LoadLibraryA", &IrType::Ptr, &[IrType::Ptr]);
        self.module
            .declare_dllimport("GetProcAddress", &IrType::Ptr, &[IrType::Ptr, IrType::Ptr]);
        self.module
            .declare_dllimport("FreeLibrary", &IrType::I32, &[IrType::Ptr]);
        self.module.declare_dllimport(
            "GetCurrentDirectoryA",
            &IrType::I32,
            &[IrType::I32, IrType::Ptr],
        );
        self.module
            .declare_dllimport("GetCurrentThreadId", &IrType::I32, &[]);
        self.module
            .declare_dllimport("IsWindow", &IrType::I32, &[IrType::Ptr]);
        self.module
            .declare_dllimport("GetDlgItem", &IrType::Ptr, &[IrType::Ptr, IrType::I32]);

        // Empty string constant
        let (empty_name, _) = self.module.add_string_constant("");
        self.empty_string_name = empty_name;

        // Default MSGBOX title when title$ is omitted — official PB uses
        // "PowerBASIC" (CHW msgbox.htm / msgbox_s.htm; batch 250).  User
        // brand decision 2026-10-18: our compiler defaults to "PB64Basic"
        // instead (deliberate divergence from the CHW default, user-picked).
        let (title_name, _) = self.module.add_string_constant("PB64Basic");
        self.msgbox_title_name = title_name;

        // First pass: collect TYPE declarations, function/sub signatures, and globals

        // Pre-scan: find all names that will be DIMmed as arrays, so we don't
        // emit them as scalars when we see GLOBAL name (without parens) first.
        // Also scan function/sub bodies since common PB pattern puts DIM inside SetupDims().
        let mut future_arrays: HashSet<String> = HashSet::new();
        for item in &program.items {
            match item {
                TopLevel::DimDecl(dim) => {
                    if !dim.bounds.is_empty() {
                        future_arrays.insert(normalize_name(&dim.name));
                    }
                }
                TopLevel::DimDeclList(dims) => {
                    for dim in dims {
                        if !dim.bounds.is_empty() {
                            future_arrays.insert(normalize_name(&dim.name));
                        }
                    }
                }
                TopLevel::SubDecl(sd) => {
                    Self::scan_body_for_array_names(&sd.body, &mut future_arrays);
                }
                TopLevel::FunctionDecl(fd) => {
                    Self::scan_body_for_array_names(&fd.body, &mut future_arrays);
                }
                _ => {}
            }
        }

        for item in &program.items {
            if let TopLevel::TypeDecl(td) = item {
                self.register_type(td);
            }
        }

        for item in &program.items {
            match item {
                TopLevel::FunctionDecl(fd) => {
                    self.register_function(fd);
                }
                TopLevel::SubDecl(sd) => {
                    self.register_sub(sd);
                }
                TopLevel::DeclareStmt(ds) => {
                    self.register_declare(ds);
                }
                TopLevel::GlobalDecl(vd) => {
                    if vd.is_array {
                        // GLOBAL arr() AS TYPE → remember for later DIM with bounds
                        self.pending_global_arrays
                            .insert(normalize_name(&vd.name), vd.pb_type.clone());
                    } else if future_arrays.contains(&normalize_name(&vd.name)) {
                        // Will be DIMmed as array later → treat as pending array
                        self.pending_global_arrays
                            .insert(normalize_name(&vd.name), vd.pb_type.clone());
                    } else {
                        self.declare_global(vd);
                    }
                }
                TopLevel::GlobalDeclList(decls) => {
                    for vd in decls {
                        if vd.is_array || future_arrays.contains(&normalize_name(&vd.name)) {
                            self.pending_global_arrays
                                .insert(normalize_name(&vd.name), vd.pb_type.clone());
                        } else {
                            self.declare_global(vd);
                        }
                    }
                }
                TopLevel::DimDecl(dim) => {
                    self.compile_top_level_dim(dim);
                }
                TopLevel::DimDeclList(dims) => {
                    for dim in dims {
                        self.compile_top_level_dim(dim);
                    }
                }
                TopLevel::AsmData(ad) => {
                    self.compile_asmdata(ad);
                }
                _ => {}
            }
        }

        // In session mode, also scan function/sub bodies for pending global DIMs
        // and GLOBAL scalar declarations inside SUBs (e.g., GLOBAL ForceSync in SetupDims)
        if self.session_mode {
            for item in &program.items {
                match item {
                    TopLevel::FunctionDecl(fd) => self.scan_body_for_global_dims(&fd.body),
                    TopLevel::SubDecl(sd) => self.scan_body_for_global_dims(&sd.body),
                    _ => {}
                }
            }
        }

        // In session mode, emit the struct type and global instance
        if self.session_mode {
            self.emit_session_struct();
        }

        // Second pass: compile function/sub bodies
        let mut codegen_errors = 0usize;
        let mut first_codegen_err: Option<PbError> = None;
        for item in &program.items {
            match item {
                TopLevel::FunctionDecl(fd) => {
                    if let Err(e) = self.compile_function(fd) {
                        eprintln!("Codegen warning: FUNCTION {} → {}", fd.name, e);
                        codegen_errors += 1;
                        if first_codegen_err.is_none() {
                            first_codegen_err = Some(e);
                        }
                        // Emit stub declaration so call sites don't fail
                        let name = normalize_name(&fd.name);
                        if let Some(fi) = self.functions.get(&name) {
                            let ir_name = fi.ir_name.clone();
                            let ret_type = fi.ret_type.clone();
                            let param_types: Vec<IrType> =
                                fi.params.iter().map(|p| p.ir_type.clone()).collect();
                            self.module
                                .declare_function(&ir_name, &ret_type, &param_types, false);
                        }
                    }
                }
                TopLevel::SubDecl(sd) => {
                    if let Err(e) = self.compile_sub(sd) {
                        eprintln!("Codegen warning: SUB {} → {}", sd.name, e);
                        codegen_errors += 1;
                        if first_codegen_err.is_none() {
                            first_codegen_err = Some(e);
                        }
                        // Emit stub declaration so call sites don't fail
                        let name = normalize_name(&sd.name);
                        if let Some(fi) = self.subs.get(&name) {
                            let ir_name = fi.ir_name.clone();
                            let param_types: Vec<IrType> =
                                fi.params.iter().map(|p| p.ir_type.clone()).collect();
                            self.module.declare_function(
                                &ir_name,
                                &IrType::Void,
                                &param_types,
                                false,
                            );
                        }
                    }
                }
                _ => {}
            }
        }
        if codegen_errors > 0 {
            eprintln!(
                "[PB64Basic] {} codegen errors (functions skipped)",
                codegen_errors
            );
            // Hard-fail on the first codegen error instead of continuing to a
            // link step whose failure would mask the real semantic error and
            // produce a misleading exit code (PBWin-aligned error codes).
            if let Some(e) = first_codegen_err {
                return Err(e);
            }
        }

        // Emit main() entry point that calls PBMAIN() if it exists
        if let Some(pbmain) = self.functions.get("PBMAIN") {
            let pbmain_ir_name = pbmain.ir_name.clone();
            let mut fb = self
                .module
                .create_function_builder("main", &IrType::I32, &[]);
            fb.call_void("pb_install_crash_handler", &[]);
            fb.call_void("pb_profile_enable", &[]);
            if self.debug_mode {
                let (cat, _) = self.module.add_string_constant("BUILD");
                let (msg, _) = self
                    .module
                    .add_string_constant("Debug mode enabled → modal and state logging active");
                fb.call_void(
                    "pb_debug_log_msg",
                    &[Val::new(cat, IrType::Ptr), Val::new(msg, IrType::Ptr)],
                );
            }
            let ret = fb.call(&IrType::I32, &pbmain_ir_name, &[]);
            fb.ret(&ret);
            self.module.add_function_body(fb.finish());
        } else {
            // PBMAIN not found — suggest close function names
            let close: Vec<&String> = self
                .functions
                .keys()
                .filter(|k| {
                    let u = k.to_uppercase();
                    u.starts_with("PB") || u.contains("MAIN")
                })
                .collect();
            if close.is_empty() {
                eprintln!("Error: No FUNCTION PBMAIN found (entry point missing)");
            } else {
                eprintln!(
                    "Error: No FUNCTION PBMAIN found. Did you mean: {} ?",
                    close
                        .iter()
                        .map(|s| format!("FUNCTION {}", s))
                        .collect::<Vec<_>>()
                        .join(" or ")
                );
            }
            return Err(PbError::parser(
                "No FUNCTION PBMAIN entry point found",
                None,
                0,
            ));
        }

        // In session mode, emit GetSession() export
        if self.session_mode {
            self.emit_get_session();
        }

        Ok(())
    }

    fn register_type(&mut self, td: &TypeDecl) {
        let name = normalize_name(&td.name);
        let mut fields = Vec::new();
        let mut field_map = HashMap::new();
        let mut ir_types = Vec::new();

        for (i, field) in td.fields.iter().enumerate() {
            let field_name = normalize_name(&field.name);
            let ir_type = Self::ir_type_for_field(&field.pb_type);
            ir_types.push(ir_type.clone());
            field_map.insert(field_name.clone(), i);
            fields.push(FieldInfo {
                name: field_name,
                pb_type: field.pb_type.clone(),
                ir_type,
            });
        }

        self.module.define_struct_type(&name, &ir_types);
        self.type_layouts.insert(
            name.clone(),
            TypeLayout {
                ir_name: name.clone(),
                fields,
                field_map,
            },
        );

        // EVENT SOURCE interfaces advertised by this class (batch 222).
        if !td.event_sources.is_empty() {
            let ifaces: Vec<String> = td.event_sources.iter().map(|s| s.to_uppercase()).collect();
            self.event_sources.insert(name.clone(), ifaces);
        }
    }

    fn register_function(&mut self, fd: &FunctionDecl) {
        let name = normalize_name(&fd.name);
        let ir_name = fd.alias.clone().unwrap_or_else(|| name.clone());
        let ret_type = Self::ir_type_for(&fd.return_type);
        let params: Vec<ParamInfo> = fd
            .params
            .iter()
            .map(|p| {
                let ir_type = if p.is_byval {
                    Self::ir_type_for(&p.pb_type)
                } else {
                    IrType::Ptr
                };
                ParamInfo {
                    name: normalize_name(&p.name),
                    pb_type: p.pb_type.clone(),
                    ir_type,
                    is_byval: p.is_byval,
                    is_optional: p.is_optional,
                }
            })
            .collect();

        self.functions.insert(
            name.clone(),
            FuncInfo {
                ir_name,
                ret_type,
                params,
                export: fd.export,
                is_external: false,
                is_stdcall: false,
            },
        );
    }

    fn register_sub(&mut self, sd: &SubDecl) {
        let name = normalize_name(&sd.name);
        let ir_name = sd.alias.clone().unwrap_or_else(|| name.clone());
        let params: Vec<ParamInfo> = sd
            .params
            .iter()
            .map(|p| {
                let ir_type = if p.is_byval {
                    Self::ir_type_for(&p.pb_type)
                } else {
                    IrType::Ptr
                };
                ParamInfo {
                    name: normalize_name(&p.name),
                    pb_type: p.pb_type.clone(),
                    ir_type,
                    is_byval: p.is_byval,
                    is_optional: p.is_optional,
                }
            })
            .collect();

        self.subs.insert(
            name.clone(),
            FuncInfo {
                ir_name,
                ret_type: IrType::Void,
                params,
                export: sd.export,
                is_external: false,
                is_stdcall: false,
            },
        );
    }

    fn register_declare(&mut self, ds: &DeclareStmt) {
        let name = normalize_name(&ds.name);

        if ds.lib.is_none() {
            // Forward declaration within same compilation unit → skip.
            // The actual definition will be registered from FunctionDecl/SubDecl.
            return;
        }

        // External DLL import: emit dllimport declaration
        let ir_name = ds.alias.clone().unwrap_or_else(|| name.clone());
        let params: Vec<ParamInfo> = ds
            .params
            .iter()
            .map(|p| {
                let ir_type = if p.is_byval {
                    Self::ir_type_for(&p.pb_type)
                } else {
                    IrType::Ptr
                };
                ParamInfo {
                    name: normalize_name(&p.name),
                    pb_type: p.pb_type.clone(),
                    ir_type,
                    is_byval: p.is_byval,
                    is_optional: p.is_optional,
                }
            })
            .collect();

        let ir_param_types: Vec<IrType> = params.iter().map(|p| p.ir_type.clone()).collect();

        // Determine calling convention and linkage:
        // - Win32 API DLLs: dllimport + stdcall on 32-bit
        // - Stubbed DLLs: regular extern (stubs provided in pb_runtime.c)
        // - Other DLLs: dllimport + cdecl
        let lib_name = ds.lib.as_deref().unwrap_or("").to_uppercase();
        let is_stdcall = self.module.is_32bit() && Self::is_win32_stdcall_dll(&lib_name);
        let is_stubbed = Self::is_stubbed_dll(&lib_name);

        if ds.is_function {
            let ret_type = Self::ir_type_for(&ds.return_type);
            if is_stubbed {
                // Stubbed DLLs: declare as regular external (resolved by pb_runtime.obj)
                self.module
                    .declare_function(&ir_name, &ret_type, &ir_param_types, false);
            } else if is_stdcall {
                self.module
                    .declare_dllimport(&ir_name, &ret_type, &ir_param_types);
            } else {
                self.module
                    .declare_dllimport_cdecl(&ir_name, &ret_type, &ir_param_types);
            }
            self.functions.insert(
                name,
                FuncInfo {
                    ir_name,
                    ret_type,
                    params,
                    export: false,
                    is_external: !is_stubbed,
                    is_stdcall: is_stdcall && !is_stubbed,
                },
            );
        } else {
            if is_stubbed {
                self.module
                    .declare_function(&ir_name, &IrType::Void, &ir_param_types, false);
            } else if is_stdcall {
                self.module
                    .declare_dllimport(&ir_name, &IrType::Void, &ir_param_types);
            } else {
                self.module
                    .declare_dllimport_cdecl(&ir_name, &IrType::Void, &ir_param_types);
            }
            self.subs.insert(
                name,
                FuncInfo {
                    ir_name,
                    ret_type: IrType::Void,
                    params,
                    export: false,
                    is_external: !is_stubbed,
                    is_stdcall: is_stdcall && !is_stubbed,
                },
            );
        }
    }

    fn declare_global(&mut self, vd: &VarDecl) {
        let name = normalize_name(&vd.name);
        let ir_type = Self::ir_type_for(&vd.pb_type);
        if self.session_mode {
            // Defer: collect into session_fields for struct emission later
            if !self.session_fields.iter().any(|f| f.name == name) {
                self.session_fields.push(SessionFieldInfo {
                    name,
                    ir_type,
                    pb_type: vd.pb_type.clone(),
                    is_array: false,
                    elem_ir_type: None,
                    array_dims: None,
                    total_elements: None,
                });
            }
        } else {
            if Self::is_string_pb(&vd.pb_type) {
                // String globals init to empty string pointer
                self.module
                    .add_global(&name, &ir_type, &self.empty_string_name.clone());
            } else {
                self.module
                    .add_global(&name, &ir_type, &ir_type.zero_literal());
            }
            self.symbols
                .insert_global(name, ir_type, vd.pb_type.clone());
        }
    }

    /// Declare a THREADED (thread-local storage) scalar variable.
    /// Threaded variables are global to every Sub/Function, but each thread
    /// has its own independent copy. Emitted as a module-level `thread_local`
    /// global so loads/stores resolve per-thread automatically.
    fn declare_threaded_global(&mut self, vd: &VarDecl) {
        let name = normalize_name(&vd.name);
        if self.threaded_declared.contains(&name) {
            // Already emitted in this module: THREADED vars are global to
            // every Sub/Function, so repeated declarations are legal and
            // must not re-emit the module global (duplicate symbol).
            return;
        }
        self.threaded_declared.insert(name.clone());
        let ir_type = Self::ir_type_for(&vd.pb_type);
        let tname = format!("__threaded_{}", name);
        if Self::is_string_pb(&vd.pb_type) {
            self.module
                .add_global_thread_local(&tname, &ir_type, &self.empty_string_name.clone());
        } else {
            self.module
                .add_global_thread_local(&tname, &ir_type, &ir_type.zero_literal());
        }
        self.symbols.insert_global_with_ptr(
            name,
            format!("@{}", tname),
            ir_type,
            vd.pb_type.clone(),
        );
    }

    // ========== Function/Sub compilation ==========

    /// Returns true if the function/sub should be nooped (compiled as an empty stub).
    /// EZLIB/EZGUI functions are part of the old Win32 UI → the Electron frontend handles all UI now.
    fn should_noop_function(name: &str) -> bool {
        name.starts_with("EZLIB_") || name.starts_with("EZGUI_")
    }

    fn compile_function(&mut self, fd: &FunctionDecl) -> PbResult<()> {
        let name = normalize_name(&fd.name);
        let info = self.functions.get(&name).unwrap().clone();

        let ir_params: Vec<(String, IrType)> = info
            .params
            .iter()
            .map(|p| (p.name.clone(), p.ir_type.clone()))
            .collect();

        let mut fb = self.module.create_function_builder_ex(
            &info.ir_name,
            &info.ret_type,
            &ir_params,
            info.export,
        );

        // Debug: record current function name
        let (fn_name_str, _) = self.module.add_string_constant(&fd.name);
        fb.call_void(
            "pb_debug_enter",
            &[Val::new(fn_name_str.clone(), IrType::Ptr)],
        );
        // CALLSTK tracing (batch 33): push this procedure's name
        // PBWin10's CALLSTK$(n) returns the frame name WITH trailing "()"
        // (measured 2026-10-14: innermost=TESTA(), frame2=PBMAIN()); match it.
        let (callstk_name, _) = self.module.add_string_constant(&format!("{}()", fd.name));
        fb.call_void("pb_callstk_push", &[Val::new(callstk_name, IrType::Ptr)]);

        // Noop EZLIB/EZGUI functions → old Win32 UI, replaced by Electron
        if Self::should_noop_function(&name) {
            let ret_ir = Self::ir_type_for(&fd.return_type);
            if ret_ir == IrType::Ptr {
                fb.ret(&Val::new(self.empty_string_name.clone(), IrType::Ptr));
            } else {
                fb.ret(&Val::new(ret_ir.zero_literal(), ret_ir.clone()));
            }
            self.module.add_function_body(fb.finish());
            return Ok(());
        }

        self.current_fn_name = Some(name.clone());
        let ret_ir = Self::ir_type_for(&fd.return_type);
        self.current_fn_return_type = Some((fd.return_type.clone(), ret_ir.clone()));
        self.symbols.clear_locals();

        // Allocate return value storage
        let retval_ptr = fb.alloca(&ret_ir);
        if ret_ir == IrType::Ptr {
            fb.store(
                &Val::new(self.empty_string_name.clone(), IrType::Ptr),
                &retval_ptr,
            );
        } else {
            fb.store(
                &Val::new(ret_ir.zero_literal(), ret_ir.clone()),
                &retval_ptr,
            );
        }
        self.current_fn_retval_ptr = Some(retval_ptr);

        // Bind parameters
        self.bind_params(&mut fb, &info.params);

        // Set up GOSUB context (must happen before compile_body)
        self.setup_gosub_context(&mut fb, &fd.body);

        // Allocate REDIM'd arrays at their max size before compiling the body
        self.prescan_redim_arrays(&mut fb, &fd.body)?;

        // Compile body
        self.compile_body(&mut fb, &fd.body)?;

        // Emit RETURN dispatch if we had GOSUBs
        self.emit_return_dispatch(&mut fb);

        // Return
        if !fb.is_terminated() {
            fb.call_void("pb_callstk_pop", &[]);
            let retval_ptr = self.current_fn_retval_ptr.clone().unwrap();
            let ret_val = fb.load(&ret_ir, &retval_ptr);
            fb.ret(&ret_val);
        }

        self.current_fn_name = None;
        self.current_fn_return_type = None;
        self.current_fn_retval_ptr = None;
        self.gosub_context = None;
        self.onerror_handler = None;
        self.onerror_checkpoints.clear();
        self.try_exit_stack.clear();

        self.module.add_function_body(fb.finish());
        Ok(())
    }

    fn compile_sub(&mut self, sd: &SubDecl) -> PbResult<()> {
        let name = normalize_name(&sd.name);
        let info = self.subs.get(&name).unwrap().clone();

        let ir_params: Vec<(String, IrType)> = info
            .params
            .iter()
            .map(|p| (p.name.clone(), p.ir_type.clone()))
            .collect();

        let mut fb = self.module.create_function_builder_ex(
            &info.ir_name,
            &IrType::Void,
            &ir_params,
            info.export,
        );

        // Debug: record current function name
        let (fn_name_str, _) = self.module.add_string_constant(&sd.name);
        fb.call_void(
            "pb_debug_enter",
            &[Val::new(fn_name_str.clone(), IrType::Ptr)],
        );
        // CALLSTK tracing (batch 33): push this procedure's name
        // PBWin10's CALLSTK$(n) returns the frame name WITH trailing "()".
        let (callstk_name, _) = self.module.add_string_constant(&format!("{}()", sd.name));
        fb.call_void("pb_callstk_push", &[Val::new(callstk_name, IrType::Ptr)]);

        // Noop EZLIB/EZGUI subs → old Win32 UI, replaced by Electron
        if Self::should_noop_function(&name) {
            fb.ret_void();
            self.module.add_function_body(fb.finish());
            return Ok(());
        }

        self.current_fn_name = Some(name.clone());
        self.current_fn_return_type = None;
        self.current_fn_retval_ptr = None;
        self.symbols.clear_locals();

        // Bind parameters
        self.bind_params(&mut fb, &info.params);

        // Set up GOSUB context
        self.setup_gosub_context(&mut fb, &sd.body);

        // Compile body
        self.compile_body(&mut fb, &sd.body)?;

        // Emit RETURN dispatch if we had GOSUBs
        self.emit_return_dispatch(&mut fb);

        // Return void
        if !fb.is_terminated() {
            fb.call_void("pb_callstk_pop", &[]);
            fb.ret_void();
        }

        self.current_fn_name = None;
        self.gosub_context = None;
        self.onerror_handler = None;
        self.onerror_checkpoints.clear();
        self.try_exit_stack.clear();

        self.module.add_function_body(fb.finish());
        Ok(())
    }

    fn bind_params(&mut self, fb: &mut FunctionBuilder, params: &[ParamInfo]) {
        for p in params {
            if p.is_byval {
                // Allocate local storage and copy
                let ptr = fb.alloca(&Self::ir_type_for(&p.pb_type));
                let param_val = Val::new(format!("%{}", p.name), p.ir_type.clone());
                fb.store(&param_val, &ptr);
                self.symbols.insert_local(
                    p.name.clone(),
                    ptr.name.clone(),
                    Self::ir_type_for(&p.pb_type),
                    p.pb_type.clone(),
                );
            } else {
                // BYREF: parameter IS a pointer
                self.symbols.insert_local(
                    p.name.clone(),
                    format!("%{}", p.name),
                    Self::ir_type_for(&p.pb_type),
                    p.pb_type.clone(),
                );
            }
        }
    }

    // ========== GOSUB/RETURN setup ==========

    /// Scan a function body for all Label and GoSub statements.
    /// Returns (set of all label names, set of labels that are GOSUB targets).
    fn collect_labels(stmts: &[Statement]) -> (HashSet<String>, HashSet<String>) {
        let mut labels = HashSet::new();
        let mut gosub_targets = HashSet::new();
        Self::collect_labels_recursive(stmts, &mut labels, &mut gosub_targets);
        (labels, gosub_targets)
    }

    fn collect_labels_recursive(
        stmts: &[Statement],
        labels: &mut HashSet<String>,
        gosub_targets: &mut HashSet<String>,
    ) {
        for stmt in stmts {
            match stmt {
                Statement::Label(name) => {
                    labels.insert(name.clone());
                }
                Statement::OnErrorGoto(name) => {
                    labels.insert(name.clone());
                }
                Statement::ResumeLabel(name) => {
                    labels.insert(name.clone());
                }
                Statement::GoSub(name) => {
                    gosub_targets.insert(name.clone());
                }
                Statement::GoTo(_) => {}
                Statement::If(if_stmt) => {
                    Self::collect_labels_recursive(&if_stmt.then_body, labels, gosub_targets);
                    for clause in &if_stmt.elseif_clauses {
                        Self::collect_labels_recursive(&clause.body, labels, gosub_targets);
                    }
                    Self::collect_labels_recursive(&if_stmt.else_body, labels, gosub_targets);
                }
                Statement::For(for_stmt) => {
                    Self::collect_labels_recursive(&for_stmt.body, labels, gosub_targets);
                }
                Statement::DoLoop(do_stmt) => {
                    Self::collect_labels_recursive(&do_stmt.body, labels, gosub_targets);
                }
                Statement::WhileWend(while_stmt) => {
                    Self::collect_labels_recursive(&while_stmt.body, labels, gosub_targets);
                }
                Statement::SelectCase(select_stmt) => {
                    for case in &select_stmt.cases {
                        Self::collect_labels_recursive(&case.body, labels, gosub_targets);
                    }
                    Self::collect_labels_recursive(&select_stmt.else_body, labels, gosub_targets);
                }
                Statement::Block(inner) => {
                    Self::collect_labels_recursive(inner, labels, gosub_targets);
                }
                Statement::Try(try_stmt) => {
                    Self::collect_labels_recursive(&try_stmt.body, labels, gosub_targets);
                    Self::collect_labels_recursive(&try_stmt.catch, labels, gosub_targets);
                    Self::collect_labels_recursive(&try_stmt.finally, labels, gosub_targets);
                }
                _ => {}
            }
        }
    }

    /// Set up GOSUB context for a function body (if it has any labels/gosubs).
    fn setup_gosub_context(&mut self, fb: &mut FunctionBuilder, body: &[Statement]) {
        let (labels, gosub_targets) = Self::collect_labels(body);
        // Filter out labels that are actually known SUB/FUNCTION names (parser ambiguity)
        let labels: HashSet<String> = labels
            .into_iter()
            .filter(|name| !self.functions.contains_key(&normalize_name(name)))
            .collect();
        if labels.is_empty() && gosub_targets.is_empty() {
            self.gosub_context = None;
            return;
        }

        // Allocate return address variable
        let ret_addr_ptr = fb.alloca(&IrType::I32);
        fb.store(&Val::new("0", IrType::I32), &ret_addr_ptr);

        // Create block names for all labels
        let mut label_blocks = HashMap::new();
        for label_name in &labels {
            let block_name = fb.next_label(&format!("label.{}", label_name));
            label_blocks.insert(label_name.clone(), block_name);
        }
        // Also create blocks for gosub targets that might not have been in `labels`
        // (shouldn't happen but be safe)
        for target in &gosub_targets {
            if !label_blocks.contains_key(target) {
                let block_name = fb.next_label(&format!("label.{}", target));
                label_blocks.insert(target.clone(), block_name);
            }
        }

        self.gosub_context = Some(GosubContext {
            label_blocks,
            return_points: Vec::new(),
            ret_addr_ptr,
            emitted_labels: HashSet::new(),
        });
    }

    // ========== Body / Statement compilation ==========

    fn compile_body(&mut self, fb: &mut FunctionBuilder, stmts: &[Statement]) -> PbResult<()> {
        let mut idx = 0usize;
        while idx < stmts.len() {
            let stmt = &stmts[idx];
            if fb.is_terminated() {
                // If terminated but we hit a label, we still need to emit it
                // (labels create new basic blocks and reset the terminated state)
                match stmt {
                    Statement::Label(_) => {
                        // Fall through: compile the label (it creates a new block)
                    }
                    _ => {
                        idx += 1;
                        continue;
                    }
                }
            }
            if let Statement::Asm(t) = stmt {
                // Merge consecutive ASM statements into one inline-asm block so
                // register state is preserved across them (PB semantics).
                let mut seq = vec![t.clone()];
                let mut j = idx + 1;
                while j < stmts.len() {
                    if let Statement::Asm(t2) = &stmts[j] {
                        seq.push(t2.clone());
                        j += 1;
                    } else {
                        break;
                    }
                }
                self.compile_asm_block(fb, &seq)?;
                idx = j;
                continue;
            }
            self.compile_statement(fb, stmt)?;
            idx += 1;
        }
        Ok(())
    }

    fn emit_debug_line(&self, fb: &mut FunctionBuilder, line: usize) {
        if line > 0 {
            let line_val = fb.const_i32(line as i32);
            let debug_ptr = Val::new("@pb_debug_line".to_string(), IrType::Ptr);
            fb.store(&line_val, &debug_ptr);
        }
    }

    fn compile_statement(&mut self, fb: &mut FunctionBuilder, stmt: &Statement) -> PbResult<()> {
        // Emit debug line number for crash reporting
        let line = match stmt {
            Statement::Assign(a) => a.line,
            Statement::Call(c) => c.line,
            Statement::If(i) => i.line,
            Statement::For(f) => f.line,
            Statement::DoLoop(d) => d.line,
            Statement::SelectCase(s) => s.line,
            Statement::Dim(d) => d.line,
            _ => 0,
        };
        self.emit_debug_line(fb, line);
        self.current_line = line;

        match stmt {
            Statement::Assign(a) => self.compile_assign(fb, a),
            Statement::Call(c) => {
                self.compile_call_stmt(fb, c)?;
                // ON ERROR GOTO: test for a run-time error after the statement
                self.emit_error_check(fb)?;
                Ok(())
            }
            Statement::Data(items) => {
                // DATA item1, item2, ... — register constants into the runtime pool
                for item in items {
                    let (str_name, _) = self.module.add_string_constant(item);
                    let sv = Val::new(str_name, IrType::Ptr);
                    fb.call_void("pb_data_append", &[sv]);
                }
                Ok(())
            }
            Statement::Restore => {
                // RESTORE — reset the DATA read cursor
                fb.call_void("pb_data_reset", &[]);
                Ok(())
            }
            Statement::If(i) => self.compile_if(fb, i),
            Statement::For(f) => self.compile_for(fb, f),
            Statement::DoLoop(d) => self.compile_do_loop(fb, d),
            Statement::WhileWend(w) => self.compile_while_wend(fb, w),
            Statement::SelectCase(s) => self.compile_select_case(fb, s),
            Statement::Print(p) => self.compile_print(fb, p),
            Statement::Dim(d) => self.compile_dim(fb, d),
            Statement::FunctionReturn(fr) => self.compile_function_return(fb, fr),
            Statement::ExitSub => {
                // In a FUNCTION context, EXIT SUB should still return the return value
                if let Some((_, ref ret_ir)) = self.current_fn_return_type {
                    let ret_ir = ret_ir.clone();
                    let retval_ptr = self.current_fn_retval_ptr.clone().unwrap();
                    fb.call_void("pb_callstk_pop", &[]);
                    let val = fb.load(&ret_ir, &retval_ptr);
                    fb.ret(&val);
                } else {
                    fb.ret_void();
                }
                Ok(())
            }
            Statement::ExitFunction => {
                if let Some((_, ref ret_ir)) = self.current_fn_return_type {
                    let ret_ir = ret_ir.clone();
                    let retval_ptr = self.current_fn_retval_ptr.clone().unwrap();
                    fb.call_void("pb_callstk_pop", &[]);
                    let val = fb.load(&ret_ir, &retval_ptr);
                    fb.ret(&val);
                }
                Ok(())
            }
            Statement::ExitSelect => {
                // EXIT SELECT → in compiled code, CASE blocks don't fall through,
                // so this is effectively a no-op (control goes to end of select)
                Ok(())
            }
            Statement::ExitFor | Statement::ExitDo => {
                if let Some(info) = self.loop_stack.last() {
                    let exit = info.exit_label.clone();
                    fb.br(&exit);
                }
                Ok(())
            }
            Statement::Iterate(target) => {
                let is_for = matches!(target, IterateTarget::For);
                for info in self.loop_stack.iter().rev() {
                    if info.is_for == is_for {
                        let cont = info.continue_label.clone();
                        fb.br(&cont);
                        break;
                    }
                }
                Ok(())
            }
            Statement::Incr(inc) => self.compile_incr_decr(fb, inc, true),
            Statement::Decr(dec) => self.compile_incr_decr(fb, dec, false),
            Statement::Redim(d) => self.compile_dim(fb, d),
            Statement::Block(stmts) => self.compile_body(fb, stmts),
            // GOSUB/RETURN/Label/GoTo
            Statement::Label(name) => self.compile_label(fb, name),
            Statement::GoSub(label) => self.compile_gosub(fb, label),
            Statement::Return => self.compile_return(fb),
            Statement::GoTo(label) => self.compile_goto(fb, label),
            Statement::OnGoTo { expr, labels } => self.compile_on_goto(fb, expr, labels),
            Statement::OnGoSub { expr, labels } => self.compile_on_gosub(fb, expr, labels),
            Statement::OnCall { expr, targets } => self.compile_on_call(fb, expr, targets),
            Statement::ClipboardSetText { text, result } => {
                self.compile_clipboard_set(fb, text, result.as_ref())
            }
            Statement::ClipboardGetText { target, result } => {
                self.compile_clipboard_get(fb, target, result.as_ref())
            }
            Statement::ClipboardReset { result } => {
                let rc = fb.call(&IrType::I32, "pb_clipboard_reset", &[]);
                if let Some(res) = result {
                    if let Some((ptr, _, _)) = self.lvalue_ptr(fb, res) {
                        fb.store(&rc, &ptr);
                    }
                }
                Ok(())
            }
            Statement::InputFlush => {
                fb.call_void("pb_input_flush", &[]);
                Ok(())
            }
            // ON ERROR GOTO label — enable error trapping with a handler
            Statement::OnErrorGoto(label) => {
                let block = self
                    .gosub_context
                    .as_ref()
                    .and_then(|ctx| ctx.label_blocks.get(label).cloned())
                    .unwrap_or_else(|| {
                        // Safety net: labels were pre-collected, but keep the compiler
                        // robust even if the map is somehow missing an entry.
                        fb.next_label(&format!("label.{}", label))
                    });
                self.onerror_handler = Some(block);
                Ok(())
            }
            // ON ERROR GOTO 0 / ON ERROR RESUME NEXT — disable error trapping
            Statement::OnErrorGotoZero => {
                self.onerror_handler = None;
                Ok(())
            }
            // RESUME (bare) — re-execute the statement that errored
            Statement::Resume => self.compile_resume(fb, ResumeMode::Retry),
            // RESUME NEXT — continue after the statement that errored
            Statement::ResumeNext => self.compile_resume(fb, ResumeMode::Next),
            // RESUME FLUSH — continue with the next statement (no jump)
            Statement::ResumeFlush => Ok(()),
            // TRY ... CATCH ... [FINALLY ...] END TRY (batch 26)
            Statement::Try(try_stmt) => self.compile_try(fb, try_stmt),
            // EXIT TRY: jump to the statement following END TRY
            Statement::ExitTry => {
                if let Some(end_block) = self.try_exit_stack.last().cloned() {
                    if !fb.is_terminated() {
                        fb.br(&end_block);
                    }
                }
                Ok(())
            }
            // RESUME label — jump to a local label
            Statement::ResumeLabel(label) => {
                if let Some(block) = self
                    .gosub_context
                    .as_ref()
                    .and_then(|ctx| ctx.label_blocks.get(label).cloned())
                {
                    fb.store(
                        &fb.const_i32(0),
                        &Val::new("@pb_err_active".to_string(), IrType::Ptr),
                    );
                    fb.br(&block);
                }
                Ok(())
            }
            Statement::Open(o) => self.compile_open(fb, o),
            Statement::Field(f) => self.compile_field(fb, f),
            Statement::Mat(m) => self.compile_mat(fb, m),
            Statement::Close(c) => self.compile_close(fb, c),
            Statement::PrintFile(p) => self.compile_print_file(fb, p),
            Statement::InputFile(inp) => self.compile_input_file(fb, inp),
            Statement::LineInputFile(li) => self.compile_line_input_file(fb, li),
            Statement::InputConsole(inp) => self.compile_input_console(fb, inp),
            Statement::LineInputConsole(li) => self.compile_line_input_console(fb, li),
            Statement::Regexpr {
                mask,
                target,
                start,
                pos_var,
                len_var,
            } => {
                let mask = self.compile_expr(fb, mask)?;
                let target = self.compile_expr(fb, target)?;
                let start_v = if let Some(s) = start {
                    let sv = self.compile_expr(fb, s)?;
                    self.to_i64(fb, &sv)
                } else {
                    fb.const_i64(1)
                };
                let pos_tmp = fb.alloca(&IrType::I64);
                let len_tmp = fb.alloca(&IrType::I64);
                fb.call_void(
                    "pb_regex_scan",
                    &[mask, target, start_v, pos_tmp.clone(), len_tmp.clone()],
                );
                if let Some((ptr, ir, pb)) = self.lvalue_ptr(fb, pos_var) {
                    let v = fb.load(&IrType::I64, &pos_tmp);
                    let converted = self.convert_value(fb, &v, &ir, &pb);
                    fb.store(&converted, &ptr);
                }
                if let Some(len_arg) = len_var {
                    if let Some((ptr, ir, pb)) = self.lvalue_ptr(fb, len_arg) {
                        let v = fb.load(&IrType::I64, &len_tmp);
                        let converted = self.convert_value(fb, &v, &ir, &pb);
                        fb.store(&converted, &ptr);
                    }
                }
                Ok(())
            }
            Statement::Regrepl {
                mask,
                target,
                repl,
                start,
                pos_var,
                out_var,
            } => {
                let mask = self.compile_expr(fb, mask)?;
                let target = self.compile_expr(fb, target)?;
                let repl = self.compile_expr(fb, repl)?;
                let start_v = if let Some(s) = start {
                    let sv = self.compile_expr(fb, s)?;
                    self.to_i64(fb, &sv)
                } else {
                    fb.const_i64(1)
                };
                let pos_tmp = fb.alloca(&IrType::I64);
                let ret = fb.call(
                    &IrType::Ptr,
                    "pb_regex_replace",
                    &[mask, target, repl, start_v, pos_tmp.clone()],
                );
                if let Some((ptr, ir, pb)) = self.lvalue_ptr(fb, pos_var) {
                    let v = fb.load(&IrType::I64, &pos_tmp);
                    let converted = self.convert_value(fb, &v, &ir, &pb);
                    fb.store(&converted, &ptr);
                }
                if let Some((ptr, _, _)) = self.lvalue_ptr(fb, out_var) {
                    let slen = fb.call(&IrType::I32, "strlen", std::slice::from_ref(&ret));
                    fb.call_void("pb_str_assign_copy", &[ptr, ret, slen]);
                }
                Ok(())
            }
            Statement::Profile(filename) => {
                let f = self.compile_expr(fb, filename)?;
                fb.call_void("pb_profile_dump", &[f]);
                Ok(())
            }
            Statement::CallStk(filename) => {
                let f = self.compile_expr(fb, filename)?;
                fb.call_void("pb_callstk_dump", &[f]);
                Ok(())
            }
            Statement::Kill(expr) => {
                // KILL filespec — on failure set ERR: ENOENT -> 53, EACCES -> 70, other -> 75
                let path = self.compile_expr(fb, expr)?;
                let ret = fb.call(&IrType::I32, "pb_kill", &[path]);
                let neg1 = fb.const_i32(-1);
                let is_err = fb.icmp("eq", &ret, &neg1);
                let err_lbl = fb.next_label("kill.err");
                let merge_lbl = fb.next_label("kill.merge");
                fb.condbr(&is_err, &err_lbl, &merge_lbl);
                fb.label(&err_lbl);
                let errno_ptr = fb.call(&IrType::Ptr, "_errno", &[]);
                let eno = fb.load(&IrType::I32, &errno_ptr);
                let enoent = fb.const_i32(2);
                let is_enoent = fb.icmp("eq", &eno, &enoent);
                let enoent_lbl = fb.next_label("kill.enoent");
                let other_lbl = fb.next_label("kill.other");
                fb.condbr(&is_enoent, &enoent_lbl, &other_lbl);
                fb.label(&enoent_lbl);
                let g = Val::new("@pb_err".to_string(), IrType::Ptr);
                fb.store(&fb.const_i32(53), &g);
                fb.br(&merge_lbl);
                fb.label(&other_lbl);
                let g2 = Val::new("@pb_err".to_string(), IrType::Ptr);
                fb.store(&fb.const_i32(75), &g2);
                fb.br(&merge_lbl);
                fb.label(&merge_lbl);
                Ok(())
            }
            Statement::Asm(text) => {
                self.compile_asm_block(fb, std::slice::from_ref(text))?;
                Ok(())
            }
            Statement::Noop(name, line) => {
                // Allow benign/intentional NOOPs (preprocessor artifacts, end markers)
                let benign = matches!(
                    name.as_str(),
                    "END"
                        | "REM"
                        | "REM "
                        | "'"
                        | "LET"
                        | "#INCLUDE"
                        | "%CONSTANT"
                        | "END (mismatched block)"
                        | "<unknown token>"
                );
                if benign {
                    Ok(())
                } else {
                    eprintln!(
                        "Error: statement `{}` on line {} is NOT implemented - no code generated",
                        name, line
                    );
                    Err(pb::error::PbError::parser(
                        format!("Unimplemented statement: `{}`", name),
                        None,
                        *line,
                    ))
                }
            }
        }
    }

    // ========== Inline assembly (batch 29) ==========

    /// Compile a run of consecutive inline-ASM statements (`! opcode` or
    /// `ASM opcode`) into ONE LLVM `call void asm sideeffect inteldialect`
    /// block. Keeping consecutive lines in a single block preserves register
    /// state across them, matching PB semantics (e.g. `! MOV EAX, [x]` then
    /// `! MOV y, EAX`).
    ///
    /// Every PB variable operand is passed by pointer ("r" constraint) and
    /// referenced in the Intel-syntax text as `<width> ptr [$N]`, where N is
    /// the 0-based operand slot. Registers, immediates, brackets and quoted
    /// strings pass through verbatim.
    ///
    /// Automatic register-shuffling (inside the block):
    ///  - mem-to-mem operands (`MOV y, x`): the source is loaded into a
    ///    scratch register first (x86 cannot encode mem,mem).
    ///  - `MOV qwordvar, <big immediate>`: split into two dword stores
    ///    (lo / hi), which works on both 64-bit and 32-bit targets.
    ///
    /// Honest limits: register state is preserved within one run of
    /// consecutive ASM lines only; labels / jumps across ASM lines are not
    /// supported; callee-saved registers are assumed preserved by the user.
    fn compile_asm_block(&mut self, fb: &mut FunctionBuilder, lines: &[String]) -> PbResult<()> {
        #[derive(Clone)]
        enum Op {
            Mem { width: String, slot: usize },
            Imm(i64),
            Raw(String),
            Comma,
        }

        let mut constraints: Vec<String> = Vec::new();
        let mut args: Vec<String> = Vec::new();

        // Resolve one token against the symbol table.
        let resolve_var = |self_: &mut Self,
                           tok: &str,
                           constraints: &mut Vec<String>,
                           args: &mut Vec<String>|
         -> Option<Op> {
            let norm = normalize_name(&tok.to_uppercase());
            if let Some(info) = self_.symbols.lookup(&norm) {
                let width = match &info.ir_type {
                    IrType::I8 => "byte",
                    IrType::I16 => "word",
                    IrType::I32 => "dword",
                    IrType::I64 => "qword",
                    IrType::Float => "dword",
                    IrType::Double => "qword",
                    _ => {
                        self_.warnings.push(format!(
                            "inline asm: variable `{}` has unsupported type {} - operand skipped",
                            tok, info.ir_type
                        ));
                        return None;
                    }
                };
                let slot = constraints.len();
                constraints.push("r".to_string());
                args.push(format!("ptr {}", info.ptr_name));
                Some(Op::Mem {
                    width: width.to_string(),
                    slot,
                })
            } else {
                None
            }
        };

        let mut instruction_texts: Vec<String> = Vec::new();

        for line in lines {
            let t = strip_asm_comment(line);
            let t = t.trim();
            if t.is_empty() {
                continue;
            }
            let toks = split_asm_tokens(t);

            let mut ops: Vec<Op> = Vec::new();
            for (idx, tok) in toks.iter().enumerate() {
                if idx == 0 {
                    ops.push(Op::Raw(tok.to_lowercase()));
                    continue;
                }
                if tok == "," {
                    ops.push(Op::Comma);
                    continue;
                }
                let mut used = false;
                if tok.starts_with('[') && tok.ends_with(']') && tok.len() >= 2 {
                    let inner = tok[1..tok.len() - 1].trim();
                    if let Some(op) = resolve_var(self, inner, &mut constraints, &mut args) {
                        ops.push(op);
                        used = true;
                    }
                }
                if !used {
                    if let Some(op) = resolve_var(self, tok, &mut constraints, &mut args) {
                        ops.push(op);
                        used = true;
                    }
                }
                if used {
                    continue;
                }
                let trimmed = tok.trim();
                let parsed = if let Some(rest) = trimmed.strip_prefix('-') {
                    rest.parse::<i64>().ok().map(|v| -v)
                } else {
                    trimmed.parse::<i64>().ok()
                };
                if let Some(v) = parsed {
                    ops.push(Op::Imm(v));
                    continue;
                }
                ops.push(Op::Raw(tok.to_lowercase()));
            }

            // Special case: `mov qwordmem, <imm > u32::MAX>` -> two dword
            // stores (lo, hi). Works on both x64 and i686 (which has no
            // 64-bit general-purpose register).
            if ops.len() == 4 {
                if let (Op::Raw(opc), Op::Mem { width, slot }, Op::Comma, Op::Imm(v)) =
                    (&ops[0], &ops[1], &ops[2], &ops[3])
                {
                    if opc == "mov" && width == "qword" && (*v as u64) > u32::MAX as u64 {
                        let lo = (*v as u64 & 0xFFFF_FFFF) as u32;
                        let hi = ((*v as u64) >> 32) as u32;
                        let s_lo = constraints.len();
                        constraints.push("r".to_string());
                        args.push(format!("i32 {}", lo));
                        let s_hi = constraints.len();
                        constraints.push("r".to_string());
                        args.push(format!("i32 {}", hi));
                        instruction_texts.push(format!(
                            "mov dword ptr [${}], ${}\nmov dword ptr [${}+4], ${}",
                            slot, s_lo, slot, s_hi
                        ));
                        continue;
                    }
                }
            }

            // Rebuild this instruction's text.
            let mut operand_parts: Vec<String> = Vec::new();
            let mut shuffle_lines: Vec<String> = Vec::new();
            if let Some(Op::Raw(opcode)) = ops.first() {
                operand_parts.push(opcode.clone());
            } else {
                operand_parts.push("nop".to_string());
            }

            let n = ops.len();
            let mut i = 1usize;
            while i < n {
                match &ops[i] {
                    Op::Comma => {
                        operand_parts.push(",".to_string());
                        i += 1;
                    }
                    Op::Imm(v) => {
                        operand_parts.push(v.to_string());
                        i += 1;
                    }
                    Op::Mem { width, slot } => {
                        let is_mem_mem_src = i >= 2
                            && matches!(&ops[i - 1], Op::Comma)
                            && matches!(&ops[i - 2], Op::Mem { .. });
                        if is_mem_mem_src {
                            // Scratch register via a constraint (never a
                            // hard-coded name: on i686 an i64 "r" constraint
                            // uses a register pair and would collide with a
                            // fixed EAX; on x64 qword needs the full 64-bit
                            // register).
                            let scratch_slot = constraints.len();
                            let scratch_ty = if width == "qword" { "i64" } else { "i32" };
                            constraints.push("r".to_string());
                            args.push(format!("{} 0", scratch_ty));
                            shuffle_lines
                                .push(format!("mov ${}, {} ptr [${}]", scratch_slot, width, slot));
                            operand_parts.push(format!("${}", scratch_slot));
                        } else {
                            operand_parts.push(format!("{} ptr [${}]", width, slot));
                        }
                        i += 1;
                    }
                    Op::Raw(txt) => {
                        operand_parts.push(txt.clone());
                        i += 1;
                    }
                }
            }

            let mut text = shuffle_lines.join("\n");
            if !text.is_empty() {
                text.push('\n');
            }
            text.push_str(&operand_parts.join(" "));
            instruction_texts.push(text.trim().to_string());
        }

        if instruction_texts.is_empty() {
            return Ok(());
        }
        let asm_str = instruction_texts.join("\n");
        let c = if constraints.is_empty() {
            "~{memory}".to_string()
        } else {
            format!("{},~{{memory}}", constraints.join(","))
        };
        if args.is_empty() {
            fb.raw_line(&format!(
                "call void asm sideeffect inteldialect \"{}\", \"{}\"()",
                asm_str, c
            ));
        } else {
            fb.raw_line(&format!(
                "call void asm sideeffect inteldialect \"{}\", \"{}\"({})",
                asm_str,
                c,
                args.join(", ")
            ));
        }
        Ok(())
    }

    // ========== Assign ==========

    fn compile_assign(&mut self, fb: &mut FunctionBuilder, assign: &AssignStmt) -> PbResult<()> {
        let value = self.compile_expr(fb, &assign.value)?;
        match &assign.target {
            Expr::Variable(orig_name) => {
                let name = normalize_name(orig_name);
                // batch 249: CALLBACK FUNCTION/SUB synthetic slots — evaluate
                // and discard (the callback dispatcher returns 0).  Bypasses
                // symbol lookup so #DIM ALL never rejects the pseudo-name.
                if name == "__CALLBACK_FUNCTION" || name == "__CALLBACK_SUB" {
                    return Ok(());
                }
                if let Some(info) = self.symbols.lookup(&name) {
                    let ptr_name = info.ptr_name.clone();
                    let target_ir = info.ir_type.clone();
                    let target_pb = info.pb_type.clone();
                    // PBWin10 Error 421 (String operand expected): assigning a
                    // numeric value to a string-typed variable is a hard error
                    // in the official compiler; align instead of silently
                    // converting (which the old convert_value fallback did).
                    if matches!(
                        target_pb,
                        PbType::String | PbType::FixedString(_) | PbType::Field
                    ) && value.ty != IrType::Ptr
                    {
                        return Err(PbError::parser(
                            format!(
                                "String operand expected: cannot assign a numeric value to string variable `{}`",
                                orig_name
                            ),
                            None,
                            self.current_line,
                        ));
                    }
                    // FixedString (ASCIIZ*N): use strncpy instead of store
                    if let PbType::FixedString(n) = &target_pb {
                        let buf_ptr = Val::new(ptr_name, IrType::Ptr);
                        let src = self.compile_expr(fb, &assign.value)?;
                        let size = fb.const_i32((*n as i32) - 1);
                        fb.call_void("strncpy", &[buf_ptr.clone(), src, size]);
                        // Null-terminate at position N-1
                        let term_idx = fb.const_i32((*n as i32) - 1);
                        let term_ptr = fb.gep_byte(&buf_ptr, &term_idx);
                        fb.store(&Val::new("0".to_string(), IrType::I8), &term_ptr);
                    } else if self.field_bound_strings.contains(&name) {
                        // FIELD dyn$-bound string: copy into a fresh mutable
                        // buffer. A plain pointer store would make the variable
                        // point at a read-only string constant and pb_field_set
                        // would crash writing through it.
                        let src = self.compile_expr(fb, &assign.value)?;
                        let slot = Val::new(ptr_name.clone(), IrType::Ptr);
                        let slen = fb.call(&IrType::I32, "strlen", std::slice::from_ref(&src));
                        fb.call_void("pb_str_assign_copy", &[slot, src, slen]);
                    } else if let PbType::Field = &target_pb {
                        // FIELD variable: write into the bound sub-section (pb_field_set).
                        // The value must be a string; pad with blanks to full length.
                        let fv_ptr = Val::new(ptr_name.clone(), IrType::Ptr);
                        let src = self.compile_expr(fb, &assign.value)?;
                        let slen = fb.call(&IrType::I32, "strlen", std::slice::from_ref(&src));
                        fb.call_void("pb_field_set", &[fv_ptr, src, slen]);
                    } else {
                        let converted = self.convert_value(fb, &value, &target_ir, &target_pb);
                        let ptr = Val::new(ptr_name, IrType::Ptr);
                        fb.store(&converted, &ptr);
                    }
                } else {
                    // Implicit member access inside a METHOD: a bare name that is one of
                    // the receiver's INSTANCE fields means this.<name> (batch 216).
                    if let Some(member_expr) = self.implicit_this_member(&name) {
                        let (field_ptr, field_pb) = self.compile_lvalue_ptr(fb, &member_expr)?;
                        if let PbType::FixedString(n) = &field_pb {
                            let src = self.compile_expr(fb, &assign.value)?;
                            let size = fb.const_i32((*n as i32) - 1);
                            fb.call_void("strncpy", &[field_ptr.clone(), src, size]);
                            let term_idx = fb.const_i32((*n as i32) - 1);
                            let term_ptr = fb.gep_byte(&field_ptr, &term_idx);
                            fb.store(&Val::new("0".to_string(), IrType::I8), &term_ptr);
                        } else {
                            let field_ir = Self::ir_type_for(&field_pb);
                            let converted = self.convert_value(fb, &value, &field_ir, &field_pb);
                            fb.store(&converted, &field_ptr);
                        }
                        return Ok(());
                    }
                    if self.option_explicit {
                        return Err(PbError::parser(
                            format!(
                                "OPTION EXPLICIT: variable `{}` must be declared before use",
                                orig_name
                            ),
                            None,
                            self.current_line,
                        ));
                    }
                    // Official PB: a bare undeclared name has no type source
                    // (no DEFxxx support yet) → Error 516, matching the official
                    // compiler instead of silently auto-declaring LONG.
                    if !name_has_type_suffix(orig_name) {
                        return Err(PbError::parser(
                            format!(
                                "DefType, Type id (?%&!#$), or AS... required: {}",
                                orig_name.to_uppercase()
                            ),
                            None,
                            self.current_line,
                        ));
                    }
                    // Auto-declare local → use original name for type inference
                    let pb_type = infer_type_from_name(orig_name);
                    let ir_type = Self::ir_type_for(&pb_type);
                    let ptr = fb.alloca(&ir_type);
                    let converted = self.convert_value(fb, &value, &ir_type, &pb_type);
                    fb.store(&converted, &ptr);
                    self.symbols.insert_local(name, ptr.name, ir_type, pb_type);
                }
            }
            Expr::ArrayAccess(arr_name, indices) | Expr::FunctionCall(arr_name, indices) => {
                let name = normalize_name(arr_name);
                if let Some(arr_info) = self.symbols.lookup_array(&name).cloned() {
                    let mut compiled_indices = Vec::new();
                    for idx in indices {
                        compiled_indices.push(self.compile_expr(fb, idx)?);
                    }
                    let flat_idx = self.compute_flat_index(fb, &compiled_indices, &arr_info);
                    let base = Val::new(arr_info.ptr_name.clone(), IrType::Ptr);
                    let elem_ptr = fb.gep_array(&arr_info.array_ir_type, &base, &flat_idx);
                    let converted =
                        self.convert_value(fb, &value, &arr_info.elem_ir_type, &arr_info.pb_type);
                    fb.store(&converted, &elem_ptr);
                }
            }
            Expr::TypeMember(_, _) => {
                let (field_ptr, field_pb) = self.compile_lvalue_ptr(fb, &assign.target)?;
                // FixedString TYPE fields: buffer is inline [N x i8] 鈫?use strncpy,
                // never store a pointer into the buffer.
                if let PbType::FixedString(n) = &field_pb {
                    let src = self.compile_expr(fb, &assign.value)?;
                    let size = fb.const_i32((*n as i32) - 1);
                    fb.call_void("strncpy", &[field_ptr.clone(), src, size]);
                    let term_idx = fb.const_i32((*n as i32) - 1);
                    let term_ptr = fb.gep_byte(&field_ptr, &term_idx);
                    fb.store(&Val::new("0".to_string(), IrType::I8), &term_ptr);
                } else {
                    let field_ir = Self::ir_type_for(&field_pb);
                    let converted = self.convert_value(fb, &value, &field_ir, &field_pb);
                    fb.store(&converted, &field_ptr);
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// PowerBASIC implicit member access inside a method (batch 216).
    ///
    /// A METHOD inside a CLASS is compiled as `FUNCTION <Class>_<Method>` whose first
    /// parameter is an implicit BYREF receiver named `this`.  Inside such a body a bare
    /// name that matches one of the receiver type's INSTANCE fields means
    /// `this.<name>`.  Before this the name missed the symbol table and fell through to
    /// the auto-declare path, silently becoming a fresh zero-initialised local: the
    /// method ran, reported success and changed nothing on the object.
    ///
    /// The rewritten expression is handed back to the caller so the existing TypeMember
    /// machinery does the field arithmetic, string handling and diagnostics instead of
    /// there being a second copy of them here.
    fn implicit_this_member(&self, name: &str) -> Option<Expr> {
        // The symbol table is keyed by the *normalised* name (uppercase, type
        // suffix stripped), so the receiver injected by the parser is stored under
        // "THIS" -- looking up the literal "this" always missed.
        let this = self.symbols.lookup(&normalize_name("this"))?;
        let tname = match &this.pb_type {
            PbType::UserDefined(t) => normalize_name(t),
            _ => return None,
        };
        let layout = self.type_layouts.get(&tname)?;
        if !layout.field_map.contains_key(name) {
            return None;
        }
        Some(Expr::TypeMember(
            Box::new(Expr::Variable("this".to_string())),
            name.to_string(),
        ))
    }

    /// The TYPE name behind an expression that denotes a struct instance.
    /// Used by the `o.M(args)` sugar (batch 217): the receiver must be a
    /// variable (or a member) whose declared type is a user-defined TYPE.
    fn struct_type_of_expr(&self, expr: &Expr) -> Option<String> {
        match expr {
            Expr::Variable(name) => {
                let info = self.symbols.lookup(&normalize_name(name))?;
                match &info.pb_type {
                    PbType::UserDefined(t) => Some(normalize_name(t)),
                    _ => None,
                }
            }
            Expr::TypeMember(base, member) => {
                let t = self.struct_type_of_expr(base)?;
                let layout = self.type_layouts.get(&t)?;
                let idx = *layout.field_map.get(&normalize_name(member))?;
                match &layout.fields[idx].pb_type {
                    PbType::UserDefined(f) => Some(normalize_name(f)),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    // ========== Lvalue pointer ==========

    /// Return a pointer to an lvalue expression (without loading it).
    fn compile_lvalue_ptr(
        &mut self,
        fb: &mut FunctionBuilder,
        expr: &Expr,
    ) -> PbResult<(Val, PbType)> {
        match expr {
            Expr::Variable(orig_name) => {
                let name = normalize_name(orig_name);
                if let Some(info) = self.symbols.lookup(&name) {
                    Ok((
                        Val::new(info.ptr_name.clone(), IrType::Ptr),
                        info.pb_type.clone(),
                    ))
                } else {
                    // Implicit member access inside a METHOD (batch 216): reuse the
                    // TypeMember path for the receiver's field.
                    if let Some(member_expr) = self.implicit_this_member(&name) {
                        return self.compile_lvalue_ptr(fb, &member_expr);
                    }
                    if self.option_explicit {
                        return Err(PbError::parser(
                            format!(
                                "OPTION EXPLICIT: variable `{}` must be declared before use",
                                orig_name
                            ),
                            None,
                            self.current_line,
                        ));
                    }
                    // Auto-declare
                    let ptr_name = self.ensure_variable_ptr(fb, &name, orig_name)?;
                    let info = self.symbols.lookup(&name).unwrap();
                    let pb_type = info.pb_type.clone();
                    Ok((Val::new(ptr_name, IrType::Ptr), pb_type))
                }
            }
            Expr::ArrayAccess(arr_name, indices) | Expr::FunctionCall(arr_name, indices) => {
                let name = normalize_name(arr_name);
                if let Some(arr_info) = self.symbols.lookup_array(&name).cloned() {
                    let mut compiled_indices = Vec::new();
                    for idx in indices {
                        compiled_indices.push(self.compile_expr(fb, idx)?);
                    }
                    let flat_idx = self.compute_flat_index(fb, &compiled_indices, &arr_info);
                    let base = Val::new(arr_info.ptr_name.clone(), IrType::Ptr);
                    let elem_ptr = fb.gep_array(&arr_info.array_ir_type, &base, &flat_idx);
                    Ok((elem_ptr, arr_info.pb_type.clone()))
                } else {
                    Err(PbError::runtime(format!("Unknown array: {}", name)))
                }
            }
            Expr::TypeMember(base_expr, member) => {
                let (base_ptr, base_pb) = self.compile_lvalue_ptr(fb, base_expr)?;
                if let PbType::UserDefined(type_name) = &base_pb {
                    let type_name = normalize_name(type_name);
                    let layout = self
                        .type_layouts
                        .get(&type_name)
                        .ok_or_else(|| PbError::runtime(format!("Unknown TYPE: {}", type_name)))?;
                    let member_norm = normalize_name(member);
                    let field_idx = *layout.field_map.get(&member_norm).ok_or_else(|| {
                        PbError::runtime(format!("Unknown field: {}.{}", type_name, member_norm))
                    })?;
                    let field_pb = layout.fields[field_idx].pb_type.clone();
                    let field_ptr = fb.gep_struct(&type_name, &base_ptr, field_idx);
                    Ok((field_ptr, field_pb))
                } else {
                    Err(PbError::runtime(
                        "TypeMember on non-TYPE variable".to_string(),
                    ))
                }
            }
            _ => Err(PbError::runtime("Not an lvalue".to_string())),
        }
    }

    /// ASMDATA ... END ASMDATA: pack items into a read-only byte blob global
    /// (`@__asmdata_<NAME>`), register it so CODEPTR(NAME) resolves to its
    /// address. Bytes are packed contiguously and never aligned, per PB docs.
    fn compile_asmdata(&mut self, ad: &AsmDataDecl) {
        let mut bytes: Vec<u8> = Vec::new();
        for item in &ad.items {
            match item {
                AsmDataItem::Db(vals) => {
                    for v in vals {
                        match v {
                            AsmDataValue::Num(n) => bytes.push(*n as u8),
                            AsmDataValue::Str(s) => bytes.extend_from_slice(s.as_bytes()),
                        }
                    }
                }
                AsmDataItem::Dw(vals) => {
                    for v in vals {
                        match v {
                            AsmDataValue::Num(n) => {
                                let n = *n as u16;
                                bytes.extend_from_slice(&n.to_le_bytes());
                            }
                            AsmDataValue::Str(s) => {
                                for ch in s.encode_utf16() {
                                    bytes.extend_from_slice(&ch.to_le_bytes());
                                }
                            }
                        }
                    }
                }
                AsmDataItem::Dd(vals) => {
                    for v in vals {
                        match v {
                            AsmDataValue::Num(n) => {
                                let n = *n as u32;
                                bytes.extend_from_slice(&n.to_le_bytes());
                            }
                            AsmDataValue::Str(_) => {
                                // PB does not allow string literals in DD; keep as zeros to stay deterministic
                                bytes.extend_from_slice(&0u32.to_le_bytes());
                            }
                        }
                    }
                }
                AsmDataItem::Dq(vals) => {
                    for v in vals {
                        match v {
                            AsmDataValue::Num(n) => bytes.extend_from_slice(&n.to_le_bytes()),
                            AsmDataValue::Str(_) => bytes.extend_from_slice(&0u64.to_le_bytes()),
                        }
                    }
                }
            }
        }
        let sym = format!("@__asmdata_{}", ad.name);
        self.module
            .add_byte_array_global(sym.trim_start_matches('@'), &bytes);
        self.asmdata_blocks.insert(ad.name.clone(), sym);
    }

    fn compile_top_level_dim(&mut self, dim: &DimStatement) {
        let name = normalize_name(&dim.name);
        if !dim.bounds.is_empty() {
            // Top-level DIM with bounds → declare as global array
            let pb_type = if let Some(pt) = self.pending_global_arrays.get(&name) {
                pt.clone()
            } else {
                dim.pb_type.clone()
            };
            self.declare_global_array(&name, &pb_type, &dim.bounds);
        } else if dim.scope == DimScope::Global {
            let vd = VarDecl {
                name: dim.name.clone(),
                pb_type: dim.pb_type.clone(),
                is_array: false,
                line: dim.line,
            };
            self.declare_global(&vd);
        } else if dim.scope == DimScope::Threaded {
            // THREADED: thread-local storage variable, global to every
            // Sub/Function but with one independent copy per thread.
            // Scalars only for now; THREADED arr() arrays are not yet
            // supported (they need the pending-array path + TLS arrays).
            let vd = VarDecl {
                name: dim.name.clone(),
                pb_type: dim.pb_type.clone(),
                is_array: false,
                line: dim.line,
            };
            self.declare_threaded_global(&vd);
        }
    }

    // ========== DIM ==========

    /// Pre-scan a function body for every REDIM so each array is stack-allocated
    /// at its maximum declared size. Fixes re-REDIM being silently dropped
    /// (a second REDIM used to hit `lookup_array().is_some() -> return` and the
    /// array kept its first size, causing out-of-bounds writes).
    fn prescan_redim_arrays(
        &mut self,
        fb: &mut FunctionBuilder,
        body: &[Statement],
    ) -> PbResult<()> {
        // Pass 1: element types from array declarations (`LOCAL sa() AS STRING`),
        // so a Variant REDIM can inherit the declared type.
        let mut elem_types: HashMap<String, PbType> = HashMap::new();
        fn collect_types(stmts: &[Statement], map: &mut HashMap<String, PbType>) {
            for st in stmts {
                match st {
                    Statement::Dim(d) => {
                        if d.bounds.is_empty() {
                            let n = normalize_name(&d.name);
                            map.entry(n).or_insert_with(|| d.pb_type.clone());
                        }
                    }
                    Statement::Block(b) => collect_types(b, map),
                    _ => {}
                }
            }
        }
        collect_types(body, &mut elem_types);

        // Pass 2: max element count per array across all REDIM statements.
        let mut max_total: HashMap<String, (PbType, usize)> = HashMap::new();
        let eval_const = Self::eval_const_expr;
        fn collect_dims(
            stmts: &[Statement],
            elem_types: &HashMap<String, PbType>,
            map: &mut HashMap<String, (PbType, usize)>,
            eval_const: fn(&Expr) -> Result<i64, &'static str>,
        ) {
            for st in stmts {
                match st {
                    Statement::Redim(d) if !d.bounds.is_empty() => {
                        let n = normalize_name(&d.name);
                        let pb_type = if matches!(d.pb_type, PbType::Variant) {
                            elem_types.get(&n).cloned().unwrap_or(PbType::Long)
                        } else {
                            d.pb_type.clone()
                        };
                        let mut total = 1usize;
                        for b in &d.bounds {
                            let lo = eval_const(&b.lower).unwrap_or(0);
                            let up = eval_const(&b.upper).unwrap_or(0);
                            total *= (up - lo + 1) as usize;
                        }
                        let pt = pb_type.clone();
                        let e = map.entry(n).or_insert_with(|| (pt.clone(), 0));
                        if total > e.1 {
                            *e = (pt, total);
                        }
                    }
                    Statement::Block(b) => collect_dims(b, elem_types, map, eval_const),
                    _ => {}
                }
            }
        }
        collect_dims(body, &elem_types, &mut max_total, eval_const);

        // Pass 3: allocate each array at its max size (skips globals; those are
        // allocated by declare_global_array when the real REDIM is compiled).
        for (name, (pb_type, total)) in max_total {
            if self.pending_global_arrays.contains_key(&name) {
                continue;
            }
            if self.symbols.lookup_array(&name).is_some() {
                continue;
            }
            let elem_ir = Self::ir_type_for(&pb_type);
            let array_ir = IrType::Array(total, Box::new(elem_ir.clone()));
            let ptr = fb.alloca(&array_ir);
            fb.store(&Val::new("zeroinitializer", array_ir.clone()), &ptr);
            let dims = vec![(1i64, total as i64)];
            self.symbols.insert_local_array(
                name,
                ArrayInfo {
                    ptr_name: ptr.name,
                    array_ir_type: array_ir,
                    elem_ir_type: elem_ir.clone(),
                    pb_type,
                    dims,
                    total_elements: total,
                },
            );
        }
        Ok(())
    }

    fn compile_dim(&mut self, fb: &mut FunctionBuilder, dim: &DimStatement) -> PbResult<()> {
        let name = normalize_name(&dim.name);

        if !dim.bounds.is_empty() {
            // Check if this is a pending global array (GLOBAL arr() + DIM arr(bounds) inside fn)
            if self.pending_global_arrays.contains_key(&name) {
                let pb_type = self.pending_global_arrays.get(&name).unwrap().clone();
                self.declare_global_array(&name, &pb_type, &dim.bounds);
                return Ok(());
            }
            // Already declared (prescan allocated it at max size) → update the
            // bounds/type info in place instead of skipping, so a second REDIM
            // correctly changes the array size.
            if let Some(prev) = self.symbols.lookup_array(&name).cloned() {
                let arr_pb_type = if matches!(dim.pb_type, PbType::Variant) {
                    prev.pb_type.clone()
                } else {
                    dim.pb_type.clone()
                };
                let elem_ir = Self::ir_type_for(&arr_pb_type);
                let mut dims = Vec::new();
                let mut total = 1usize;
                for bound in &dim.bounds {
                    let lower = Self::eval_const_expr(&bound.lower).unwrap_or(0);
                    let upper = Self::eval_const_expr(&bound.upper).unwrap_or(0);
                    let count = (upper - lower + 1) as usize;
                    dims.push((lower, count as i64));
                    total *= count;
                }
                // Keep the original max-size allocation (from prescan); only
                // refresh metadata. Never re-allocate here: prescan already
                // sized every REDIM'd array at its maximum.
                // PBWin10 semantics (measured): REDIM without PRESERVE ZEROES the
                // array (HEX of redim_test: PB prints a(1)=0 after re-REDIM and
                // fails its own assertion). So clear the whole max-size buffer
                // before updating metadata.
                if dim.is_redim {
                    fb.store(
                        &Val::new("zeroinitializer", prev.array_ir_type.clone()),
                        &Val::new(prev.ptr_name.clone(), IrType::Ptr),
                    );
                }
                self.symbols.insert_local_array(
                    name,
                    ArrayInfo {
                        ptr_name: prev.ptr_name.clone(),
                        array_ir_type: prev.array_ir_type.clone(),
                        elem_ir_type: elem_ir.clone(),
                        pb_type: arr_pb_type,
                        dims,
                        total_elements: total,
                    },
                );
                return Ok(());
            }
            // Local array
            // REDIM without an explicit type (parser marks it Variant): inherit
            // the type declared earlier, e.g. `LOCAL sarr() AS STRING` followed
            // by `REDIM sarr(1 TO 4)`. Falls back to LONG when undeclared.
            // batch 248: plain `DIM arr(1 TO 3)` (no AS clause) must ALSO inherit
            // a declared STRING type - before this, `LOCAL items() AS STRING` +
            // `DIM items(1 TO 3)` built a [N x i32] array (element = 32-bit
            // truncated pointer), which made CONTROL ADD LISTBOX/COMBOBOX
            // items$() fills read garbage.  With STRING inherited the array is
            // [N x Ptr] and store/load follow elem_ir_type automatically.
            let declared = self.symbols.lookup(&name).map(|i| i.pb_type.clone());
            let arr_pb_type = match &dim.pb_type {
                PbType::Variant => declared.clone().unwrap_or(PbType::Long),
                PbType::Long => match &declared {
                    Some(PbType::String) | Some(PbType::FixedString(_)) => {
                        declared.clone().unwrap()
                    }
                    _ => PbType::Long,
                },
                other => other.clone(),
            };
            let elem_ir = Self::ir_type_for(&arr_pb_type);
            let mut dims = Vec::new();
            let mut total = 1usize;
            for bound in &dim.bounds {
                let lower = Self::eval_const_expr(&bound.lower).unwrap_or(0);
                let upper = Self::eval_const_expr(&bound.upper).unwrap_or(0);
                let count = (upper - lower + 1) as usize;
                dims.push((lower, count as i64));
                total *= count;
            }
            let array_ir = IrType::Array(total, Box::new(elem_ir.clone()));
            let ptr = fb.alloca(&array_ir);
            fb.store(&Val::new("zeroinitializer", array_ir.clone()), &ptr);
            self.symbols.insert_local_array(
                name,
                ArrayInfo {
                    ptr_name: ptr.name,
                    array_ir_type: array_ir,
                    elem_ir_type: elem_ir.clone(),
                    pb_type: arr_pb_type,
                    dims,
                    total_elements: total,
                },
            );
            return Ok(());
        }

        // Scalar DIM
        // STATIC: allocate in module globals so the value persists across calls
        if dim.scope == DimScope::Static {
            let global_name = match &self.current_fn_name {
                Some(fn_name) => format!("__static_{}_{}", fn_name, name),
                None => format!("__static_{}", name),
            };
            if let PbType::FixedString(n) = &dim.pb_type {
                // +1 keeps the global buffer NUL-terminated after GET fills all N
                let buf_ir = IrType::Array(*n + 1, Box::new(IrType::I8));
                self.module
                    .add_global(&global_name, &buf_ir, "zeroinitializer");
                self.symbols.insert_local(
                    name,
                    format!("@{}", global_name),
                    IrType::Ptr,
                    dim.pb_type.clone(),
                );
                return Ok(());
            }
            let ir_type = Self::ir_type_for(&dim.pb_type);
            if Self::is_string_pb(&dim.pb_type) {
                self.module
                    .add_global(&global_name, &ir_type, &self.empty_string_name.clone());
            } else {
                self.module
                    .add_global(&global_name, &ir_type, &ir_type.zero_literal());
            }
            self.symbols.insert_local(
                name,
                format!("@{}", global_name),
                ir_type,
                dim.pb_type.clone(),
            );
            return Ok(());
        }
        if dim.scope == DimScope::Threaded {
            // THREADED: thread-local storage variable, global to every
            // Sub/Function but with one independent copy per thread.
            // Scalars only for now; THREADED arr() arrays are not yet
            // supported (they need the pending-array path + TLS arrays).
            let vd = VarDecl {
                name: dim.name.clone(),
                pb_type: dim.pb_type.clone(),
                is_array: false,
                line: dim.line,
            };
            self.declare_threaded_global(&vd);
            return Ok(());
        }
        // FixedString(N) / ASCIIZ*N: allocate [N+1 x i8] buffer on stack.
        // The extra byte keeps the buffer NUL-terminated even after a
        // GET/PUT/INPUT fills all N data bytes (pb_get writes exactly N),
        // so strlen/strcmp/& -concatenation never read past the buffer.
        if let PbType::FixedString(n) = &dim.pb_type {
            let buf_ir = IrType::Array(*n + 1, Box::new(IrType::I8));
            let ptr = fb.alloca(&buf_ir);
            fb.store(&Val::new("zeroinitializer", buf_ir.clone()), &ptr);
            // Register as Ptr type so it can be passed to functions expecting char*
            self.symbols
                .insert_local(name, ptr.name, IrType::Ptr, dim.pb_type.clone());
            return Ok(());
        }
        let ir_type = Self::ir_type_for(&dim.pb_type);
        let ptr = fb.alloca(&ir_type);
        if Self::is_string_pb(&dim.pb_type) {
            fb.store(&Val::new(self.empty_string_name.clone(), IrType::Ptr), &ptr);
        } else {
            fb.store(&Val::new(ir_type.zero_literal(), ir_type.clone()), &ptr);
        }
        self.symbols
            .insert_local(name, ptr.name, ir_type, dim.pb_type.clone());
        Ok(())
    }

    // ========== FUNCTION = expr ==========

    fn compile_function_return(
        &mut self,
        fb: &mut FunctionBuilder,
        fr: &FunctionReturnStmt,
    ) -> PbResult<()> {
        if let Some(retval_ptr) = self.current_fn_retval_ptr.clone() {
            let (ref pb_type, ref ir_type) = self
                .current_fn_return_type
                .clone()
                .unwrap_or((PbType::Long, IrType::I32));
            let value = self.compile_expr(fb, &fr.value)?;
            let converted = self.convert_value(fb, &value, ir_type, pb_type);
            let ptr = Val::new(retval_ptr.name, IrType::Ptr);
            fb.store(&converted, &ptr);
        }
        Ok(())
    }

    // ========== CALL ==========

    /// Store the result of an IMAGELIST ADD form into its TO target.
    ///
    /// The TO clause travels as an explicit `has_to` flag at `flag_index`,
    /// with the target one slot after it, instead of being inferred from the
    /// last argument: `IMAGELIST ADD BITMAP h, hbmp` with no TO clause at all
    /// would otherwise look like a TO target named `hbmp`.
    fn store_imagelist_add_result(
        &mut self,
        fb: &mut FunctionBuilder,
        call: &CallStmt,
        val: &Val,
        flag_index: usize,
    ) -> PbResult<()> {
        if !matches!(call.args.get(flag_index), Some(Expr::IntegerLit(1))) {
            return Ok(());
        }
        if let Some(t) = call.args.get(flag_index + 1) {
            let target = self.compile_lvalue_ptr(fb, t)?;
            let converted = self.convert_value(fb, val, &target.0.ty, &target.1);
            fb.store(&converted, &target.0);
        }
        Ok(())
    }

    fn compile_call_stmt(&mut self, fb: &mut FunctionBuilder, call: &CallStmt) -> PbResult<()> {
        let name = normalize_name(&call.name);

        // System statement-builtins
        match name.as_str() {
            "SLEEP" => {
                if !call.args.is_empty() {
                    let ms = self.compile_expr(fb, &call.args[0])?;
                    let ms_i32 = self.to_i32(fb, &ms);
                    if self.module.is_32bit() {
                        fb.call_void_stdcall("Sleep", &[ms_i32]);
                    } else {
                        fb.call_void("Sleep", &[ms_i32]);
                    }
                }
                return Ok(());
            }
            "RANDOMIZE" => {
                if !call.args.is_empty() {
                    let seed = self.compile_expr(fb, &call.args[0])?;
                    let seed_i32 = self.to_i32(fb, &seed);
                    fb.call_void("srand", &[seed_i32]);
                } else {
                    let null = Val::new("null".to_string(), IrType::Ptr);
                    let t = fb.call(&IrType::I32, "time", &[null]);
                    fb.call_void("srand", &[t]);
                }
                return Ok(());
            }
            "CLS" => {
                // CLS — clear the console screen (PB/CC)
                fb.call_void("pb_cls", &[]);
                return Ok(());
            }
            "ERROR" => {
                // ERROR n — set the PB error code (readable via ERR)
                if let Some(arg) = call.args.first() {
                    let v = self.compile_expr(fb, arg)?;
                    let n = self.to_i32(fb, &v);
                    let g = Val::new("@pb_err".to_string(), IrType::Ptr);
                    fb.store(&n, &g);
                }
                return Ok(());
            }
            "ENVIRON" => {
                // ENVIRON "VAR=value" / ENVIRON "VAR" — set/remove env var
                if let Some(arg) = call.args.first() {
                    let s = self.compile_expr(fb, arg)?;
                    fb.call_void("pb_environ_set", &[s]);
                }
                return Ok(());
            }
            "MEMORY_COPY" | "MEMORY_SWAP" | "MEMORY_FILL" => {
                // MEMORY COPY/SWAP src&, dst&, count& | MEMORY FILL dst&, count&, BYTE|WORD|DWORD v
                let fname = if name == "MEMORY_COPY" {
                    "pb_mem_copy"
                } else if name == "MEMORY_SWAP" {
                    "pb_mem_swap"
                } else {
                    "pb_mem_fill"
                };
                if call.args.len() >= 3 {
                    let mut a = Vec::new();
                    for (i, e) in call.args.iter().enumerate() {
                        let v = self.compile_expr(fb, e)?;
                        let vi = self.convert_value(fb, &v, &IrType::I64, &PbType::Long);
                        let _ = i;
                        a.push(vi);
                    }
                    fb.call_void(fname, &a);
                }
                return Ok(());
            }
            "MEMORY_FILLS" => {
                // batch 220: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                        "MEMORY_FILLS requires its arguments: MEMORY FILL Dest& , Count& , BYTE | WORD | DWORD IntegralExpr".to_string(),
                        None,
                        call.line,
                    ));
                }

                // MEMORY FILL dst&, count&, str$
                if call.args.len() >= 3 {
                    let d = self.compile_expr(fb, &call.args[0])?;
                    let d2 = self.convert_value(fb, &d, &IrType::I64, &PbType::Long);
                    let c = self.compile_expr(fb, &call.args[1])?;
                    let c2 = self.convert_value(fb, &c, &IrType::I64, &PbType::Long);
                    let v = self.compile_expr(fb, &call.args[2])?;
                    let v2 = self.convert_value(fb, &v, &IrType::Ptr, &PbType::String);
                    fb.call_void("pb_mem_fill_str", &[d2, c2, v2]);
                }
                return Ok(());
            }
            "FONT_NEW" => {
                // args: fontname, [points, style, charset, pitch, escapement], to_var
                let name_v = self.compile_expr(fb, &call.args[0])?;
                let mut points = fb.const_f32(12.0);
                let mut style = fb.const_i32(0);
                let mut charset = fb.const_i32(0);
                let mut pitch = fb.const_i32(0);
                let mut escapement = fb.const_i32(0);
                let mut to_var: Option<String> = None;
                let mut i = 1;
                while i < call.args.len() {
                    match i {
                        1 => {
                            let p = self.compile_expr(fb, &call.args[i])?;
                            points = self.convert_value(fb, &p, &IrType::Float, &PbType::Single);
                        }
                        2 => {
                            let s = self.compile_expr(fb, &call.args[i])?;
                            style = self.convert_value(fb, &s, &IrType::I32, &PbType::Long);
                        }
                        3 => charset = self.compile_expr(fb, &call.args[i])?,
                        4 => pitch = self.compile_expr(fb, &call.args[i])?,
                        5 => escapement = self.compile_expr(fb, &call.args[i])?,
                        _ => {}
                    }
                    i += 1;
                }
                if let Some(Expr::Variable(tv)) = call.args.last() {
                    to_var = Some(tv.clone());
                }
                let h = fb.call(
                    &IrType::I32,
                    "pb_font_new",
                    &[name_v, points, style, charset, pitch, escapement],
                );
                if let Some(tv) = to_var {
                    let target = self.compile_lvalue_ptr(fb, &Expr::Variable(tv))?;
                    let converted = self.convert_value(fb, &h, &target.0.ty, &target.1);
                    fb.store(&converted, &target.0);
                }
            }
            "FONT_END" => {
                if let Some(h) = call.args.first() {
                    let hv = self.compile_expr(fb, h)?;
                    fb.call_void("pb_font_end", &[hv]);
                }
            }

            "IMAGELIST_NEW" | "IMAGELIST_NEW_ICON" => {
                // args: w, h, depth, initial, has_to [, to]
                let is_icon = name == "IMAGELIST_NEW_ICON";
                let w = self.compile_expr(fb, &call.args[0])?;
                let w2 = self.convert_value(fb, &w, &IrType::I32, &PbType::Long);
                let mut h2 = fb.const_i32(0);
                let mut depth = fb.const_i32(24);
                let mut initial = fb.const_i32(4);
                let mut i = 1;
                while i < call.args.len() && i < 4 {
                    match i {
                        1 => h2 = self.compile_expr(fb, &call.args[i])?,
                        2 => depth = self.compile_expr(fb, &call.args[i])?,
                        3 => initial = self.compile_expr(fb, &call.args[i])?,
                        _ => {}
                    }
                    i += 1;
                }
                let kind = fb.const_i32(if is_icon { 1 } else { 0 });
                let hh = fb.call(
                    &IrType::I64,
                    "pb_imagelist_new",
                    &[w2, h2, depth, initial, kind],
                );
                if matches!(call.args.get(4), Some(Expr::IntegerLit(1))) {
                    if let Some(t) = call.args.get(5) {
                        let target = self.compile_lvalue_ptr(fb, t)?;
                        let converted = self.convert_value(fb, &hh, &target.0.ty, &target.1);
                        fb.store(&converted, &target.0);
                    }
                }
            }
            "IMAGELIST_COUNT" => {
                // args: h, to_var
                let hh = self.compile_expr(fb, &call.args[0])?;
                let h2 = self.convert_value(fb, &hh, &IrType::I64, &PbType::Quad);
                let c = fb.call(&IrType::I32, "pb_imagelist_count", &[h2]);
                if let Some(tv) = call.args.last().and_then(|e| match e {
                    Expr::Variable(v) => Some(v.clone()),
                    _ => None,
                }) {
                    let target = self.compile_lvalue_ptr(fb, &Expr::Variable(tv))?;
                    let converted = self.convert_value(fb, &c, &target.0.ty, &target.1);
                    fb.store(&converted, &target.0);
                }
            }
            "IMAGELIST_KILL" => {
                if let Some(h) = call.args.first() {
                    let hv = self.compile_expr(fb, h)?;
                    let h2 = self.convert_value(fb, &hv, &IrType::I64, &PbType::Quad);
                    fb.call_void("pb_imagelist_kill", &[h2]);
                }
            }

            // ---- IMAGELIST ADD / SET OVERLAY (batch 173) -------------------
            // Every ADD form assigns the 1-based index of the first image
            // added, or 0 when the operation fails; the runtime wrappers do
            // that mapping (the raw Win32 calls are 0-based and return -1 on
            // failure).  IMAGELIST SET OVERLAY has no TO clause.
            "IMAGELIST_ADD_BITMAP" => {
                // args: hLst, hBmp, hMsk(0), has_to [, to]
                let hv = self.compile_expr(fb, &call.args[0])?;
                let hh = self.convert_value(fb, &hv, &IrType::I64, &PbType::Quad);
                let bv = self.compile_expr(fb, &call.args[1])?;
                let bb = self.convert_value(fb, &bv, &IrType::I64, &PbType::Quad);
                let mv = self.compile_expr(fb, &call.args[2])?;
                let mm = self.convert_value(fb, &mv, &IrType::I64, &PbType::Quad);
                let idx = fb.call(&IrType::I32, "pb_imagelist_add_bitmap", &[hh, bb, mm]);
                self.store_imagelist_add_result(fb, call, &idx, 3)?;
            }
            "IMAGELIST_ADD_BITMAP_FILE" => {
                // args: hLst, Bmp$, Msk$(""), has_to [, to]
                let hv = self.compile_expr(fb, &call.args[0])?;
                let hh = self.convert_value(fb, &hv, &IrType::I64, &PbType::Quad);
                let nm = self.compile_expr(fb, &call.args[1])?;
                let mk = self.compile_expr(fb, &call.args[2])?;
                let idx = fb.call(&IrType::I32, "pb_imagelist_add_bitmap_file", &[hh, nm, mk]);
                self.store_imagelist_add_result(fb, call, &idx, 3)?;
            }
            "IMAGELIST_ADD_ICON" => {
                // args: hLst, hIcn, has_to [, to]
                let hv = self.compile_expr(fb, &call.args[0])?;
                let hh = self.convert_value(fb, &hv, &IrType::I64, &PbType::Quad);
                let iv = self.compile_expr(fb, &call.args[1])?;
                let ii = self.convert_value(fb, &iv, &IrType::I64, &PbType::Quad);
                let idx = fb.call(&IrType::I32, "pb_imagelist_add_icon", &[hh, ii]);
                self.store_imagelist_add_result(fb, call, &idx, 2)?;
            }
            "IMAGELIST_ADD_ICON_FILE" => {
                // args: hLst, Icn$, has_to [, to]
                let hv = self.compile_expr(fb, &call.args[0])?;
                let hh = self.convert_value(fb, &hv, &IrType::I64, &PbType::Quad);
                let nm = self.compile_expr(fb, &call.args[1])?;
                let idx = fb.call(&IrType::I32, "pb_imagelist_add_icon_file", &[hh, nm]);
                self.store_imagelist_add_result(fb, call, &idx, 2)?;
            }
            "IMAGELIST_ADD_MASKED" => {
                // args: hLst, hBmp, rgbColor&, has_to [, to]
                let hv = self.compile_expr(fb, &call.args[0])?;
                let hh = self.convert_value(fb, &hv, &IrType::I64, &PbType::Quad);
                let bv = self.compile_expr(fb, &call.args[1])?;
                let bb = self.convert_value(fb, &bv, &IrType::I64, &PbType::Quad);
                let rv = self.compile_expr(fb, &call.args[2])?;
                let rr = self.convert_value(fb, &rv, &IrType::I32, &PbType::Long);
                let idx = fb.call(&IrType::I32, "pb_imagelist_add_masked", &[hh, bb, rr]);
                self.store_imagelist_add_result(fb, call, &idx, 3)?;
            }
            "IMAGELIST_ADD_MASKED_FILE" => {
                // args: hLst, Bmp$, rgbColor&, has_to [, to]
                let hv = self.compile_expr(fb, &call.args[0])?;
                let hh = self.convert_value(fb, &hv, &IrType::I64, &PbType::Quad);
                let nm = self.compile_expr(fb, &call.args[1])?;
                let rv = self.compile_expr(fb, &call.args[2])?;
                let rr = self.convert_value(fb, &rv, &IrType::I32, &PbType::Long);
                let idx = fb.call(&IrType::I32, "pb_imagelist_add_masked_file", &[hh, nm, rr]);
                self.store_imagelist_add_result(fb, call, &idx, 3)?;
            }
            "IMAGELIST_SET_OVERLAY" => {
                // args: hLst, image&, overlay&
                let hv = self.compile_expr(fb, &call.args[0])?;
                let hh = self.convert_value(fb, &hv, &IrType::I64, &PbType::Quad);
                let iv = self.compile_expr(fb, &call.args[1])?;
                let ii = self.convert_value(fb, &iv, &IrType::I32, &PbType::Long);
                let ov = self.compile_expr(fb, &call.args[2])?;
                let oo = self.convert_value(fb, &ov, &IrType::I32, &PbType::Long);
                fb.call_void("pb_imagelist_set_overlay", &[hh, ii, oo]);
            }

            "COLOR" => {
                // args: [fore], [back] — defaults -1 (keep current)
                let fore = if let Some(a0) = call.args.first() {
                    self.compile_expr(fb, a0)?
                } else {
                    fb.const_i32(-1)
                };
                let back = if let Some(a1) = call.args.get(1) {
                    self.compile_expr(fb, a1)?
                } else {
                    fb.const_i32(-1)
                };
                let f2 = self.convert_value(fb, &fore, &IrType::I32, &PbType::Long);
                let b2 = self.convert_value(fb, &back, &IrType::I32, &PbType::Long);
                fb.call_void("pb_color", &[f2, b2]);
                return Ok(());
            }

            "MENU_NEW_BAR" | "MENU_NEW_POPUP" => {
                let f = if call.name == "MENU_NEW_POPUP" {
                    "pb_menu_new_popup"
                } else {
                    "pb_menu_new_bar"
                };
                let h = fb.call(&IrType::I64, f, &[]);
                if let Some(Expr::Variable(tv)) = call.args.first() {
                    let target = self.compile_lvalue_ptr(fb, &Expr::Variable(tv.clone()))?;
                    let converted = self.convert_value(fb, &h, &target.0.ty, &target.1);
                    fb.store(&converted, &target.0);
                }
                return Ok(());
            }
            "MENU_ADD_STRING" => {
                // args: hMenu, txt$, id&, state&
                let h = self.compile_expr(fb, &call.args[0])?;
                let h2 = self.convert_value(fb, &h, &IrType::I64, &PbType::Quad);
                let txt = self.compile_expr(fb, &call.args[1])?;
                let id = self.compile_expr(fb, &call.args[2])?;
                let id2 = self.convert_value(fb, &id, &IrType::I32, &PbType::Long);
                let state = self.compile_expr(fb, &call.args[3])?;
                let s2 = self.convert_value(fb, &state, &IrType::I32, &PbType::Long);
                fb.call_void("pb_menu_add_string", &[h2, txt, id2, s2]);
                return Ok(());
            }
            "MENU_ADD_POPUP" => {
                // args: hMenu, hSub, id&
                let h = self.compile_expr(fb, &call.args[0])?;
                let h2 = self.convert_value(fb, &h, &IrType::I64, &PbType::Quad);
                let hs = self.compile_expr(fb, &call.args[1])?;
                let hs2 = self.convert_value(fb, &hs, &IrType::I64, &PbType::Quad);
                let id = self.compile_expr(fb, &call.args[2])?;
                let id2 = self.convert_value(fb, &id, &IrType::I32, &PbType::Long);
                fb.call_void("pb_menu_add_popup", &[h2, hs2, id2]);
                return Ok(());
            }
            "MENU_DELETE" => {
                // args: hMenu, pos&
                let h = self.compile_expr(fb, &call.args[0])?;
                let h2 = self.convert_value(fb, &h, &IrType::I64, &PbType::Quad);
                let pos = self.compile_expr(fb, &call.args[1])?;
                let p2 = self.convert_value(fb, &pos, &IrType::I32, &PbType::Long);
                fb.call_void("pb_menu_delete", &[h2, p2]);
                return Ok(());
            }

            "MENU_ATTACH" => {
                // args: hMenu, hDlg  (official: MENU ATTACH hMenu, hDlg)
                // Attaches the menu to the dialog, replacing any existing menu; the
                // dialog is redrawn so the new menu bar has room.
                let hm = self.compile_expr(fb, &call.args[0])?;
                let hm2 = self.convert_value(fb, &hm, &IrType::I64, &PbType::Quad);
                let hd = self.compile_expr(fb, &call.args[1])?;
                let hd2 = self.convert_value(fb, &hd, &IrType::I64, &PbType::Quad);
                fb.call_void("pb_menu_attach", &[hm2, hd2]);
                return Ok(());
            }
            "MENU_CONTEXT" => {
                // args: hMenu, x&, y&, flags&, dst   - dst receives the chosen id
                let hm = self.compile_expr(fb, &call.args[0])?;
                let x = self.compile_expr(fb, &call.args[1])?;
                let y = self.compile_expr(fb, &call.args[2])?;
                let fl = self.compile_expr(fb, &call.args[3])?;
                if let Some((dst, _, _)) = self.lvalue_ptr(fb, &call.args[4]) {
                    let hm2 = self.convert_value(fb, &hm, &IrType::I64, &PbType::Quad);
                    let x2 = self.convert_value(fb, &x, &IrType::I32, &PbType::Long);
                    let y2 = self.convert_value(fb, &y, &IrType::I32, &PbType::Long);
                    let f2 = self.convert_value(fb, &fl, &IrType::I32, &PbType::Long);
                    fb.call_void("pb_menu_context", &[hm2, x2, y2, f2, dst]);
                }
                return Ok(());
            }
            "MENU_DRAW_BAR" => {
                // args: hDlg - redraw the dialog's menu bar
                let hd = self.compile_expr(fb, &call.args[0])?;
                let hd2 = self.convert_value(fb, &hd, &IrType::I64, &PbType::Quad);
                fb.call_void("pb_menu_draw_bar", &[hd2]);
                return Ok(());
            }
            "GRAPHIC_BITMAP_NEW" => {
                // args: w&, h& [, hBmp TO target]
                let w = self.compile_expr(fb, &call.args[0])?;
                let w2 = self.convert_value(fb, &w, &IrType::I32, &PbType::Long);
                let h = self.compile_expr(fb, &call.args[1])?;
                let h2 = self.convert_value(fb, &h, &IrType::I32, &PbType::Long);
                let hbmp = fb.call(&IrType::I64, "pb_gdi_bitmap_new", &[w2, h2]);
                if let Some(Expr::Variable(tv)) = call.args.get(2) {
                    let target = self.compile_lvalue_ptr(fb, &Expr::Variable(tv.clone()))?;
                    let converted = self.convert_value(fb, &hbmp, &target.0.ty, &target.1);
                    fb.store(&converted, &target.0);
                }
            }
            "GRAPHIC_BITMAP_END" => {
                // args: [] or [hBmp]
                let h = if let Some(a0) = call.args.first() {
                    self.compile_expr(fb, a0)?
                } else {
                    fb.const_i64(-1)
                };
                let h2 = self.convert_value(fb, &h, &IrType::I64, &PbType::Quad);
                fb.call_void("pb_gdi_bitmap_end", &[h2]);
            }
            "GRAPHIC_ATTACH" => {
                // args: hTarget [, id] - a memory-bitmap handle, or a dialog/control
                //   pair.  The two-operand form used to be parsed and then dropped, so
                //   it attached to nothing; batch 179 routes it to pb_graphic_attach_ctl,
                //   which puts the control's own DC (GetDC) behind the GRAPHIC statements
                //   instead of a bitmap DC.
                if call.args.len() >= 2 {
                    let h = self.compile_expr(fb, &call.args[0])?;
                    // the declaration takes a pointer, so the dialog handle goes
                    // through inttoptr exactly as the DIALOG SET COLOR arm does
                    let hp = fb.inttoptr(&h);
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let id64 = self.convert_value(fb, &id, &IrType::I64, &PbType::Quad);
                    fb.call_void("pb_graphic_attach_ctl", &[hp, id64]);
                } else {
                    let h = self.compile_expr(fb, &call.args[0])?;
                    let h2 = self.convert_value(fb, &h, &IrType::I64, &PbType::Quad);
                    fb.call_void("pb_graphic_attach", &[h2]);
                }
            }
            "GRAPHIC_DETACH" => {
                fb.call_void("pb_graphic_detach", &[]);
            }
            "GRAPHIC_CLEAR" => {
                // args: [] or [color&] or [color&, fillstyle&]
                let c = if let Some(a0) = call.args.first() {
                    self.compile_expr(fb, a0)?
                } else {
                    fb.const_i32(0xFFFFFF)
                };
                let c2 = self.convert_value(fb, &c, &IrType::I32, &PbType::Long);
                fb.call_void("pb_graphic_clear", &[c2]);
            }
            "GRAPHIC_LINE" => {
                // args: x1,y1,x2,y2 [,color]
                let mut ia = Vec::new();
                for i in 0..4 {
                    let v = self.compile_expr(fb, &call.args[i])?;
                    ia.push(self.convert_value(fb, &v, &IrType::I32, &PbType::Long));
                }
                let col = if let Some(a4) = call.args.get(4) {
                    let v = self.compile_expr(fb, a4)?;
                    self.convert_value(fb, &v, &IrType::I32, &PbType::Long)
                } else {
                    fb.const_i32(0)
                };
                fb.call_void(
                    "pb_graphic_line",
                    &[
                        ia[0].clone(),
                        ia[1].clone(),
                        ia[2].clone(),
                        ia[3].clone(),
                        col,
                    ],
                );
            }
            "GRAPHIC_SET_PIXEL" => {
                let x = self.compile_expr(fb, &call.args[0])?;
                let xv = self.convert_value(fb, &x, &IrType::I32, &PbType::Long);
                let y = self.compile_expr(fb, &call.args[1])?;
                let yv = self.convert_value(fb, &y, &IrType::I32, &PbType::Long);
                let c = self.compile_expr(fb, &call.args[2])?;
                let cv = self.convert_value(fb, &c, &IrType::I32, &PbType::Long);
                fb.call_void("pb_graphic_set_pixel", &[xv, yv, cv]);
            }
            "GRAPHIC_GET_SIZE" => {
                if let Some((wp, _, _)) = self.lvalue_ptr(fb, &call.args[0]) {
                    if let Some((hp, _, _)) = self.lvalue_ptr(fb, &call.args[1]) {
                        fb.call_void("pb_graphic_get_size", &[wp, hp]);
                    }
                }
            }
            "GRAPHIC_GET_PPI" => {
                if let Some((xp, _, _)) = self.lvalue_ptr(fb, &call.args[0]) {
                    if let Some((yp, _, _)) = self.lvalue_ptr(fb, &call.args[1]) {
                        fb.call_void("pb_graphic_get_ppi", &[xp, yp]);
                    }
                }
            }
            "XPRINT_ATTACH" => {
                let p = self.compile_expr(fb, &call.args[0])?;
                let j = if let Some(a1) = call.args.get(1) {
                    self.compile_expr(fb, a1)?
                } else {
                    fb.const_null_ptr()
                };
                fb.call_void("pb_xprint_attach", &[p, j]);
            }
            "XPRINT_CLOSE" => {
                fb.call_void("pb_xprint_close", &[]);
            }
            "XPRINT_GET_PPI" => {
                if let Some((xp, _, _)) = self.lvalue_ptr(fb, &call.args[0]) {
                    if let Some((yp, _, _)) = self.lvalue_ptr(fb, &call.args[1]) {
                        fb.call_void("pb_xprint_get_ppi", &[xp, yp]);
                    }
                }
            }
            "XPRINT_GET_SIZE" => {
                if let Some((wp, _, _)) = self.lvalue_ptr(fb, &call.args[0]) {
                    if let Some((hp, _, _)) = self.lvalue_ptr(fb, &call.args[1]) {
                        fb.call_void("pb_xprint_get_size", &[wp, hp]);
                    }
                }
            }
            "XPRINT_GET_DC" => {
                if let Some((dp, _, _)) = self.lvalue_ptr(fb, &call.args[0]) {
                    fb.call_void("pb_xprint_get_dc", &[dp]);
                }
            }
            "XPRINT_CANCEL" => {
                fb.call_void("pb_xprint_cancel", &[]);
            }
            "XPRINT_FORMFEED" => {
                fb.call_void("pb_xprint_formfeed", &[]);
            }
            "XPRINT_LINE" => {
                let mut ia = Vec::new();
                for i in 0..4 {
                    let v = self.compile_expr(fb, &call.args[i])?;
                    ia.push(self.convert_value(fb, &v, &IrType::I32, &PbType::Long));
                }
                fb.call_void(
                    "pb_xprint_line",
                    &[ia[0].clone(), ia[1].clone(), ia[2].clone(), ia[3].clone()],
                );
            }
            "XPRINT_BOX" => {
                let mut ia = Vec::new();
                for i in 0..4 {
                    let v = self.compile_expr(fb, &call.args[i])?;
                    ia.push(self.convert_value(fb, &v, &IrType::I32, &PbType::Long));
                }
                fb.call_void(
                    "pb_xprint_box",
                    &[ia[0].clone(), ia[1].clone(), ia[2].clone(), ia[3].clone()],
                );
            }
            "XPRINT_WIDTH" => {
                let v = self.compile_expr(fb, &call.args[0])?;
                let iv = self.convert_value(fb, &v, &IrType::I32, &PbType::Long);
                fb.call_void("pb_xprint_width", &[iv]);
            }
            "XPRINT_STYLE" => {
                let v = self.compile_expr(fb, &call.args[0])?;
                let iv = self.convert_value(fb, &v, &IrType::I32, &PbType::Long);
                fb.call_void("pb_xprint_style", &[iv]);
            }
            "XPRINT_SET_COLOR" => {
                let v = self.compile_expr(fb, &call.args[0])?;
                let iv = self.convert_value(fb, &v, &IrType::I32, &PbType::Long);
                fb.call_void("pb_xprint_set_color", &[iv]);
            }
            "XPRINT_COLOR" => {
                // XPRINT COLOR fore& (packed COLORREF) OR XPRINT COLOR r, g[, b]
                if call.args.len() == 1 {
                    let v = self.compile_expr(fb, &call.args[0])?;
                    let iv = self.convert_value(fb, &v, &IrType::I32, &PbType::Long);
                    fb.call_void("pb_xprint_set_color", &[iv]);
                } else {
                    let mut cv = Vec::new();
                    for k in 0..3 {
                        if let Some(a) = call.args.get(k) {
                            let v = self.compile_expr(fb, a)?;
                            cv.push(self.convert_value(fb, &v, &IrType::I32, &PbType::Long));
                        } else {
                            cv.push(fb.const_i32(0));
                        }
                    }
                    fb.call_void(
                        "pb_xprint_set_color_rgb",
                        &[cv[0].clone(), cv[1].clone(), cv[2].clone()],
                    );
                }
                return Ok(());
            }
            "XPRINT_GET_COLOR" => {
                if let Some((p, _, _)) = self.lvalue_ptr(fb, &call.args[0]) {
                    fb.call_void("pb_xprint_get_color", &[p]);
                }
            }
            "XPRINT_SET_POS" => {
                let x = self.compile_expr(fb, &call.args[0])?;
                let xv = self.convert_value(fb, &x, &IrType::I32, &PbType::Long);
                let y = self.compile_expr(fb, &call.args[1])?;
                let yv = self.convert_value(fb, &y, &IrType::I32, &PbType::Long);
                fb.call_void("pb_xprint_set_pos", &[xv, yv]);
            }
            "XPRINT_GET_POS" => {
                if let Some((xp, _, _)) = self.lvalue_ptr(fb, &call.args[0]) {
                    if let Some((yp, _, _)) = self.lvalue_ptr(fb, &call.args[1]) {
                        fb.call_void("pb_xprint_get_pos", &[xp, yp]);
                    }
                }
            }
            "XPRINT_SET_PIXEL" => {
                let x = self.compile_expr(fb, &call.args[0])?;
                let xv = self.convert_value(fb, &x, &IrType::I32, &PbType::Long);
                let y = self.compile_expr(fb, &call.args[1])?;
                let yv = self.convert_value(fb, &y, &IrType::I32, &PbType::Long);
                let c = self.compile_expr(fb, &call.args[2])?;
                let cv = self.convert_value(fb, &c, &IrType::I32, &PbType::Long);
                fb.call_void("pb_xprint_set_pixel", &[xv, yv, cv]);
            }
            "XPRINT_GET_PIXEL" => {
                let x = self.compile_expr(fb, &call.args[0])?;
                let xv = self.convert_value(fb, &x, &IrType::I32, &PbType::Long);
                let y = self.compile_expr(fb, &call.args[1])?;
                let yv = self.convert_value(fb, &y, &IrType::I32, &PbType::Long);
                if let Some((cp, _, _)) = self.lvalue_ptr(fb, &call.args[2]) {
                    fb.call_void("pb_xprint_get_pixel", &[xv, yv, cp]);
                }
            }
            "XPRINT_SET_TEXTALIGN" => {
                let v = self.compile_expr(fb, &call.args[0])?;
                let iv = self.convert_value(fb, &v, &IrType::I32, &PbType::Long);
                fb.call_void("pb_xprint_set_textalign", &[iv]);
            }
            "XPRINT_GET_TEXTALIGN" => {
                if let Some((p, _, _)) = self.lvalue_ptr(fb, &call.args[0]) {
                    fb.call_void("pb_xprint_get_textalign", &[p]);
                }
            }
            "XPRINT_GET_ATTACH" => {
                if let Some((p, _, _)) = self.lvalue_ptr(fb, &call.args[0]) {
                    fb.call_void("pb_xprint_get_attach", &[p]);
                }
            }
            "XPRINT_PRINT" => {
                for arg in &call.args {
                    let v = self.compile_expr(fb, arg)?;
                    fb.call_void("pb_xprint_print_str", &[v]);
                }
            }
            "XPRINT_ARC" => {
                // batch 234: official XPRINT ARC (x1,y1)-(x2,y2), start, end
                let mut ia = Vec::new();
                for i in 0..4 {
                    let v = self.compile_expr(fb, &call.args[i])?;
                    ia.push(self.convert_value(fb, &v, &IrType::I32, &PbType::Long));
                }
                let sa = self.compile_expr(fb, &call.args[4])?;
                let sv = self.convert_value(fb, &sa, &IrType::Double, &PbType::Single);
                let ea = self.compile_expr(fb, &call.args[5])?;
                let ev = self.convert_value(fb, &ea, &IrType::Double, &PbType::Single);
                fb.call_void(
                    "pb_xprint_arc",
                    &[
                        ia[0].clone(),
                        ia[1].clone(),
                        ia[2].clone(),
                        ia[3].clone(),
                        sv,
                        ev,
                    ],
                );
            }
            "XPRINT_ELLIPSE" => {
                let mut ia = Vec::new();
                for i in 0..4 {
                    let v = self.compile_expr(fb, &call.args[i])?;
                    ia.push(self.convert_value(fb, &v, &IrType::I32, &PbType::Long));
                }
                fb.call_void(
                    "pb_xprint_ellipse",
                    &[ia[0].clone(), ia[1].clone(), ia[2].clone(), ia[3].clone()],
                );
            }
            "XPRINT_PIE" => {
                // batch 234: official XPRINT PIE (x1,y1)-(x2,y2), start, end
                let mut ia = Vec::new();
                for i in 0..4 {
                    let v = self.compile_expr(fb, &call.args[i])?;
                    ia.push(self.convert_value(fb, &v, &IrType::I32, &PbType::Long));
                }
                let sa = self.compile_expr(fb, &call.args[4])?;
                let sv = self.convert_value(fb, &sa, &IrType::Double, &PbType::Single);
                let ea = self.compile_expr(fb, &call.args[5])?;
                let ev = self.convert_value(fb, &ea, &IrType::Double, &PbType::Single);
                fb.call_void(
                    "pb_xprint_pie",
                    &[
                        ia[0].clone(),
                        ia[1].clone(),
                        ia[2].clone(),
                        ia[3].clone(),
                        sv,
                        ev,
                    ],
                );
            }
            "XPRINT_SET_FONT" => {
                // batch 234: official XPRINT SET FONT fonthndl& (single handle)
                let h = self.compile_expr(fb, &call.args[0])?;
                let hv = self.convert_value(fb, &h, &IrType::I32, &PbType::Long);
                fb.call_void("pb_xprint_set_font", &[hv]);
            }
            "XPRINT_SET_MIX" => {
                let v = self.compile_expr(fb, &call.args[0])?;
                let iv = self.convert_value(fb, &v, &IrType::I32, &PbType::Long);
                fb.call_void("pb_xprint_set_mix", &[iv]);
            }
            "XPRINT_GET_MIX" => {
                if let Some((p, _, _)) = self.lvalue_ptr(fb, &call.args[0]) {
                    fb.call_void("pb_xprint_get_mix", &[p]);
                }
            }
            "XPRINT_SET_STRETCHMODE" => {
                let v = self.compile_expr(fb, &call.args[0])?;
                let iv = self.convert_value(fb, &v, &IrType::I32, &PbType::Long);
                fb.call_void("pb_xprint_set_stretchmode", &[iv]);
            }
            "XPRINT_GET_STRETCHMODE" => {
                if let Some((p, _, _)) = self.lvalue_ptr(fb, &call.args[0]) {
                    fb.call_void("pb_xprint_get_stretchmode", &[p]);
                }
            }
            "XPRINT_TEXT_SIZE" => {
                let txt = self.compile_expr(fb, &call.args[0])?;
                if let Some((wp, _, _)) = self.lvalue_ptr(fb, &call.args[1]) {
                    if let Some((hp, _, _)) = self.lvalue_ptr(fb, &call.args[2]) {
                        fb.call_void("pb_xprint_text_size", &[txt, wp, hp]);
                    }
                }
            }
            "XPRINT_GET_CLIENT" => {
                if let Some((wp, _, _)) = self.lvalue_ptr(fb, &call.args[0]) {
                    if let Some((hp, _, _)) = self.lvalue_ptr(fb, &call.args[1]) {
                        fb.call_void("pb_xprint_get_client", &[wp, hp]);
                    }
                }
            }
            "XPRINT_GET_CANVAS" => {
                if let Some((wp, _, _)) = self.lvalue_ptr(fb, &call.args[0]) {
                    if let Some((hp, _, _)) = self.lvalue_ptr(fb, &call.args[1]) {
                        fb.call_void("pb_xprint_get_canvas", &[wp, hp]);
                    }
                }
            }
            "XPRINT_SET_WRAP" => {
                let v = self.compile_expr(fb, &call.args[0])?;
                let iv = self.convert_value(fb, &v, &IrType::I32, &PbType::Long);
                fb.call_void("pb_xprint_set_wrap", &[iv]);
            }
            "XPRINT_GET_WRAP" => {
                if let Some((p, _, _)) = self.lvalue_ptr(fb, &call.args[0]) {
                    fb.call_void("pb_xprint_get_wrap", &[p]);
                }
            }
            "XPRINT_SET_WORDWRAP" => {
                let v = self.compile_expr(fb, &call.args[0])?;
                let iv = self.convert_value(fb, &v, &IrType::I32, &PbType::Long);
                fb.call_void("pb_xprint_set_wordwrap", &[iv]);
            }
            "XPRINT_GET_WORDWRAP" => {
                if let Some((p, _, _)) = self.lvalue_ptr(fb, &call.args[0]) {
                    fb.call_void("pb_xprint_get_wordwrap", &[p]);
                }
            }
            "XPRINT_SET_OVERLAP" => {
                let v = self.compile_expr(fb, &call.args[0])?;
                let iv = self.convert_value(fb, &v, &IrType::I32, &PbType::Long);
                fb.call_void("pb_xprint_set_overlap", &[iv]);
            }
            "XPRINT_GET_OVERLAP" => {
                if let Some((p, _, _)) = self.lvalue_ptr(fb, &call.args[0]) {
                    fb.call_void("pb_xprint_get_overlap", &[p]);
                }
            }
            "XPRINT_SET_CLIP" => {
                let mut ia = Vec::new();
                for i in 0..4 {
                    let v = self.compile_expr(fb, &call.args[i])?;
                    ia.push(self.convert_value(fb, &v, &IrType::I32, &PbType::Long));
                }
                fb.call_void(
                    "pb_xprint_set_clip",
                    &[ia[0].clone(), ia[1].clone(), ia[2].clone(), ia[3].clone()],
                );
            }
            "XPRINT_GET_CLIP" => {
                // batch 234: official XPRINT GET CLIP TO WidthVar!, HeightVar!
                if let Some((p0, _, _)) = self.lvalue_ptr(fb, &call.args[0]) {
                    if let Some((p1, _, _)) = self.lvalue_ptr(fb, &call.args[1]) {
                        fb.call_void("pb_xprint_get_clip", &[p0, p1]);
                    }
                }
            }
            "XPRINT_SCALE" => {
                // batch 234: official XPRINT SCALE (x1,y1)-(x2,y2)
                let mut ia = Vec::new();
                for i in 0..4 {
                    let v = self.compile_expr(fb, &call.args[i])?;
                    ia.push(self.convert_value(fb, &v, &IrType::I32, &PbType::Long));
                }
                fb.call_void(
                    "pb_xprint_scale",
                    &[ia[0].clone(), ia[1].clone(), ia[2].clone(), ia[3].clone()],
                );
            }
            "XPRINT_GET_SCALE" => {
                // batch 234: official XPRINT GET SCALE TO x1!,y1!,x2!,y2!
                if let Some((p0, _, _)) = self.lvalue_ptr(fb, &call.args[0]) {
                    if let Some((p1, _, _)) = self.lvalue_ptr(fb, &call.args[1]) {
                        if let Some((p2, _, _)) = self.lvalue_ptr(fb, &call.args[2]) {
                            if let Some((p3, _, _)) = self.lvalue_ptr(fb, &call.args[3]) {
                                fb.call_void("pb_xprint_get_scale", &[p0, p1, p2, p3]);
                            }
                        }
                    }
                }
            }
            "XPRINT_GET_LINES" => {
                if let Some((p, _, _)) = self.lvalue_ptr(fb, &call.args[0]) {
                    fb.call_void("pb_xprint_get_lines", &[p]);
                }
            }
            "XPRINT_CELL_SIZE" => {
                if let Some((wp, _, _)) = self.lvalue_ptr(fb, &call.args[0]) {
                    if let Some((hp, _, _)) = self.lvalue_ptr(fb, &call.args[1]) {
                        fb.call_void("pb_xprint_cell_size", &[wp, hp]);
                    }
                }
            }
            "XPRINT_CHR_SIZE" => {
                if let Some((wp, _, _)) = self.lvalue_ptr(fb, &call.args[0]) {
                    if let Some((hp, _, _)) = self.lvalue_ptr(fb, &call.args[1]) {
                        fb.call_void("pb_xprint_chr_size", &[wp, hp]);
                    }
                }
            }
            "XPRINT_COPY" => {
                let mut ia = Vec::new();
                for i in 0..6 {
                    let v = self.compile_expr(fb, &call.args[i])?;
                    ia.push(self.convert_value(fb, &v, &IrType::I32, &PbType::Long));
                }
                fb.call_void(
                    "pb_xprint_copy",
                    &[
                        ia[0].clone(),
                        ia[1].clone(),
                        ia[2].clone(),
                        ia[3].clone(),
                        ia[4].clone(),
                        ia[5].clone(),
                    ],
                );
            }
            "XPRINT_POLYGON" => {
                let ncoords = call.args.len() - if call.args.len() % 2 == 1 { 1 } else { 0 };
                let npts = ncoords / 2;
                if npts >= 3 {
                    let arrty = IrType::Array(npts * 2, Box::new(IrType::I32));
                    let ptr = fb.alloca(&arrty);
                    for i in 0..ncoords {
                        let v = self.compile_expr(fb, &call.args[i])?;
                        let iv = self.convert_value(fb, &v, &IrType::I32, &PbType::Long);
                        let elem = fb.gep_byte(&ptr, &fb.const_i32((i as i32) * 4));
                        fb.store(&iv, &elem);
                    }
                    let col = if call.args.len() % 2 == 1 {
                        let e = self.compile_expr(fb, call.args.last().unwrap())?;
                        self.convert_value(fb, &e, &IrType::I64, &PbType::Long)
                    } else {
                        fb.const_i64(0)
                    };
                    fb.call_void("pb_xprint_polygon", &[ptr, fb.const_i32(npts as i32), col]);
                }
            }
            "XPRINT_POLYLINE" => {
                let ncoords = call.args.len() - if call.args.len() % 2 == 1 { 1 } else { 0 };
                let npts = ncoords / 2;
                if npts >= 2 {
                    let arrty = IrType::Array(npts * 2, Box::new(IrType::I32));
                    let ptr = fb.alloca(&arrty);
                    for i in 0..ncoords {
                        let v = self.compile_expr(fb, &call.args[i])?;
                        let iv = self.convert_value(fb, &v, &IrType::I32, &PbType::Long);
                        let elem = fb.gep_byte(&ptr, &fb.const_i32((i as i32) * 4));
                        fb.store(&iv, &elem);
                    }
                    let col = if call.args.len() % 2 == 1 {
                        let e = self.compile_expr(fb, call.args.last().unwrap())?;
                        self.convert_value(fb, &e, &IrType::I64, &PbType::Long)
                    } else {
                        fb.const_i64(0)
                    };
                    fb.call_void("pb_xprint_polyline", &[ptr, fb.const_i32(npts as i32), col]);
                }
            }
            "XPRINT_SET_COPIES" => {
                let v = self.compile_expr(fb, &call.args[0])?;
                let iv = self.convert_value(fb, &v, &IrType::I32, &PbType::Long);
                fb.call_void("pb_xprint_set_copies", &[iv]);
            }
            "XPRINT_GET_COPIES" => {
                if let Some((p, _, _)) = self.lvalue_ptr(fb, &call.args[0]) {
                    fb.call_void("pb_xprint_get_copies", &[p]);
                }
            }
            "XPRINT_SET_ORIENTATION" => {
                let v = self.compile_expr(fb, &call.args[0])?;
                let iv = self.convert_value(fb, &v, &IrType::I32, &PbType::Long);
                fb.call_void("pb_xprint_set_orientation", &[iv]);
            }
            "XPRINT_GET_ORIENTATION" => {
                if let Some((p, _, _)) = self.lvalue_ptr(fb, &call.args[0]) {
                    fb.call_void("pb_xprint_get_orientation", &[p]);
                }
            }
            "XPRINT_SET_QUALITY" => {
                let v = self.compile_expr(fb, &call.args[0])?;
                let iv = self.convert_value(fb, &v, &IrType::I32, &PbType::Long);
                fb.call_void("pb_xprint_set_quality", &[iv]);
            }
            "XPRINT_GET_QUALITY" => {
                if let Some((p, _, _)) = self.lvalue_ptr(fb, &call.args[0]) {
                    fb.call_void("pb_xprint_get_quality", &[p]);
                }
            }
            "XPRINT_SET_DUPLEX" => {
                let v = self.compile_expr(fb, &call.args[0])?;
                let iv = self.convert_value(fb, &v, &IrType::I32, &PbType::Long);
                fb.call_void("pb_xprint_set_duplex", &[iv]);
            }
            "XPRINT_GET_DUPLEX" => {
                if let Some((p, _, _)) = self.lvalue_ptr(fb, &call.args[0]) {
                    fb.call_void("pb_xprint_get_duplex", &[p]);
                }
            }
            "XPRINT_SET_COLLATE" => {
                let v = self.compile_expr(fb, &call.args[0])?;
                let iv = self.convert_value(fb, &v, &IrType::I32, &PbType::Long);
                fb.call_void("pb_xprint_set_collate", &[iv]);
            }
            "XPRINT_GET_COLLATE" => {
                if let Some((p, _, _)) = self.lvalue_ptr(fb, &call.args[0]) {
                    fb.call_void("pb_xprint_get_collate", &[p]);
                }
            }
            "XPRINT_SET_COLORMODE" => {
                let v = self.compile_expr(fb, &call.args[0])?;
                let iv = self.convert_value(fb, &v, &IrType::I32, &PbType::Long);
                fb.call_void("pb_xprint_set_colormode", &[iv]);
            }
            "XPRINT_GET_COLORMODE" => {
                if let Some((p, _, _)) = self.lvalue_ptr(fb, &call.args[0]) {
                    fb.call_void("pb_xprint_get_colormode", &[p]);
                }
            }
            "XPRINT_SET_PAGES" => {
                let v = self.compile_expr(fb, &call.args[0])?;
                let iv = self.convert_value(fb, &v, &IrType::I32, &PbType::Long);
                fb.call_void("pb_xprint_set_pages", &[iv]);
            }
            "XPRINT_GET_PAGES" => {
                if let Some((p, _, _)) = self.lvalue_ptr(fb, &call.args[0]) {
                    fb.call_void("pb_xprint_get_pages", &[p]);
                }
            }
            "XPRINT_CELL" => {
                let x = self.compile_expr(fb, &call.args[0])?;
                let xv = self.convert_value(fb, &x, &IrType::I32, &PbType::Long);
                let y = self.compile_expr(fb, &call.args[1])?;
                let yv = self.convert_value(fb, &y, &IrType::I32, &PbType::Long);
                fb.call_void("pb_xprint_set_cell", &[xv, yv]);
            }
            "XPRINT_GET_SELECTION" => {
                if let Some((p, _, _)) = self.lvalue_ptr(fb, &call.args[0]) {
                    fb.call_void("pb_xprint_get_selection", &[p]);
                }
            }
            "XPRINT_SET_PAPER" => {
                let v = self.compile_expr(fb, &call.args[0])?;
                let iv = self.convert_value(fb, &v, &IrType::I32, &PbType::Long);
                fb.call_void("pb_xprint_set_paper", &[iv]);
            }
            "XPRINT_GET_PAPER" => {
                if let Some((p, _, _)) = self.lvalue_ptr(fb, &call.args[0]) {
                    fb.call_void("pb_xprint_get_paper", &[p]);
                }
            }
            "XPRINT_SET_TRAY" => {
                let v = self.compile_expr(fb, &call.args[0])?;
                let iv = self.convert_value(fb, &v, &IrType::I32, &PbType::Long);
                fb.call_void("pb_xprint_set_tray", &[iv]);
            }
            "XPRINT_GET_TRAY" => {
                if let Some((p, _, _)) = self.lvalue_ptr(fb, &call.args[0]) {
                    fb.call_void("pb_xprint_get_tray", &[p]);
                }
            }
            "RESOURCE_SAVE_FILE" => {
                let r0 = self.compile_expr(fb, &call.args[0])?;
                let p0 = self.convert_value(fb, &r0, &IrType::Ptr, &PbType::String);
                let r1 = self.compile_expr(fb, &call.args[1])?;
                let p1 = self.convert_value(fb, &r1, &IrType::Ptr, &PbType::String);
                fb.call_void("pb_resource_save_file", &[p0, p1]);
                return Ok(());
            }
            "XPRINT_GET_PAPERS" => {
                if let Some((p, _, _)) = self.lvalue_ptr(fb, &call.args[0]) {
                    fb.call_void("pb_xprint_get_papers", &[p]);
                }
            }
            "XPRINT_GET_TRAYS" => {
                if let Some((p, _, _)) = self.lvalue_ptr(fb, &call.args[0]) {
                    fb.call_void("pb_xprint_get_trays", &[p]);
                }
            }
            "XPRINT_PREVIEW" => {
                let v = self.compile_expr(fb, &call.args[0])?;
                let iv = self.convert_value(fb, &v, &IrType::I32, &PbType::Long);
                fb.call_void("pb_xprint_preview", &[iv]);
            }
            "XPRINT_PREVIEW_CLOSE" => {
                fb.call_void("pb_xprint_preview_close", &[]);
            }
            "XPRINT_RENDER" => {
                fb.call_void("pb_xprint_render", &[]);
            }
            "XPRINT_SPLIT" => {
                // batch 234: official XPRINT SPLIT "str", len TO s1$, s2$
                let src = self.compile_expr(fb, &call.args[0])?;
                let len = self.compile_expr(fb, &call.args[1])?;
                let lv = self.convert_value(fb, &len, &IrType::I32, &PbType::Long);
                if let Some((p1, _, _)) = self.lvalue_ptr(fb, &call.args[2]) {
                    if let Some((p2, _, _)) = self.lvalue_ptr(fb, &call.args[3]) {
                        fb.call_void("pb_xprint_split", &[src, lv, p1, p2]);
                    }
                }
            }
            "XPRINT_STRETCH" => {
                let mut ia = Vec::new();
                for i in 0..8 {
                    let v = self.compile_expr(fb, &call.args[i])?;
                    ia.push(self.convert_value(fb, &v, &IrType::I32, &PbType::Long));
                }
                fb.call_void(
                    "pb_xprint_stretch",
                    &[
                        ia[0].clone(),
                        ia[1].clone(),
                        ia[2].clone(),
                        ia[3].clone(),
                        ia[4].clone(),
                        ia[5].clone(),
                        ia[6].clone(),
                        ia[7].clone(),
                    ],
                );
            }
            "XPRINT_IMAGELIST" => {
                let mut ia = Vec::new();
                for i in 0..3 {
                    let v = self.compile_expr(fb, &call.args[i])?;
                    ia.push(self.convert_value(fb, &v, &IrType::I32, &PbType::Long));
                }
                fb.call_void(
                    "pb_xprint_imagelist",
                    &[ia[0].clone(), ia[1].clone(), ia[2].clone()],
                );
            }
            "TCP_NOTIFY" => {
                let s0 = self.compile_expr(fb, &call.args[0])?;
                let i0 = self.convert_value(fb, &s0, &IrType::I32, &PbType::Long);
                let s1 = self.compile_expr(fb, &call.args[1])?;
                let i1 = self.convert_value(fb, &s1, &IrType::I32, &PbType::Long);
                fb.call_void("pb_tcp_notify", &[i0, i1]);
            }
            "UDP_NOTIFY" => {
                let s0 = self.compile_expr(fb, &call.args[0])?;
                let i0 = self.convert_value(fb, &s0, &IrType::I32, &PbType::Long);
                let s1 = self.compile_expr(fb, &call.args[1])?;
                let i1 = self.convert_value(fb, &s1, &IrType::I32, &PbType::Long);
                fb.call_void("pb_udp_notify", &[i0, i1]);
            }
            // The "PROGRESSBAR" (4-arg legacy form) and "HEADER_CTRL" arms were
            // removed in batch 163.  The parser only ever emits the official
            // syntax names (PROGRESSBAR_SET_POS / HEADER_GET_COUNT / ...), so both
            // arms were unreachable, and their runtime helpers treated the dialog
            // handle as the control HWND.  Do not re-add without a parser path.
            "ARRAY_SELECT" => {
                // ARRAY SELECT arr(), start, end  (fork extension: mark a
                // selected range for subsequent array operations).  Batch 221:
                // the old arm passed (arr, start, end, 0) into the
                // (arr, count, start, end) runtime, shifting every argument.
                if let Some(Expr::FunctionCall(arr_name, _)) = call.args.first() {
                    let an = normalize_name(arr_name);
                    if let Some(arr_info) = self.symbols.lookup_array(&an).cloned() {
                        let base = Val::new(arr_info.ptr_name.clone(), IrType::Ptr);
                        let s = self.compile_expr(fb, &call.args[1])?;
                        let s32 = self.to_i32(fb, &s);
                        let e = self.compile_expr(fb, &call.args[2])?;
                        let e32 = self.to_i32(fb, &e);
                        fb.call_void(
                            "pb_array_select",
                            &[base, fb.const_i32(arr_info.total_elements as i32), s32, e32],
                        );
                    }
                }
            }
            "ARRAY_SELECT_OP" => {
                // ARRAY SELECT arr(), > 25, TO idx - fork extension that shares
                // the ARRAY SCAN scan runtime.  Batch 221: the old arm passed
                // only (base, elem_size, value, op) to the 7-argument
                // pb_array_scan_num, shifting total/index/count/value/op and
                // reading out of bounds (crash 0xC0000005 on batch106).
                if let Some(Expr::FunctionCall(arr_name, _)) = call.args.first() {
                    let an = normalize_name(arr_name);
                    if let Some(arr_info) = self.symbols.lookup_array(&an).cloned() {
                        let base = Val::new(arr_info.ptr_name.clone(), IrType::Ptr);
                        let elem_size = match &arr_info.elem_ir_type {
                            IrType::I8 | IrType::I1 => 1,
                            IrType::I16 => 2,
                            IrType::I32 | IrType::Float => 4,
                            IrType::I64 | IrType::Double | IrType::Ptr => 8,
                            _ => 4,
                        };
                        let op = match &call.args[1] {
                            Expr::IntegerLit(v) => *v as i32,
                            _ => 0,
                        };
                        let val = self.compile_expr(fb, &call.args[2])?;
                        let v64 = self.to_i64(fb, &val);
                        let total = fb.const_i64(arr_info.total_elements as i64);
                        let result = if arr_info.elem_ir_type == IrType::Ptr {
                            fb.call(
                                &IrType::I64,
                                "pb_array_scan_str",
                                &[base, total, fb.const_i64(1), fb.const_i64(0), v64],
                            )
                        } else {
                            fb.call(
                                &IrType::I64,
                                "pb_array_scan_num",
                                &[
                                    base,
                                    fb.const_i32(elem_size),
                                    total,
                                    fb.const_i64(1),
                                    fb.const_i64(0),
                                    v64,
                                    fb.const_i32(op),
                                ],
                            )
                        };
                        if let Some(tgt) = call.args.get(3) {
                            if let Some((ptr, ir_ty, pb_ty)) = self.lvalue_ptr(fb, tgt) {
                                let conv = self.convert_value(fb, &result, &ir_ty, &pb_ty);
                                fb.store(&conv, &ptr);
                            }
                        }
                    }
                }
            }
            "ARRAY_TAGARRAY" => {
                let a0 = self.compile_expr(fb, &call.args[0])?;
                let pa0 = self.convert_value(fb, &a0, &IrType::Ptr, &PbType::Long);
                let a1 = self.compile_expr(fb, &call.args[1])?;
                let pa1 = self.convert_value(fb, &a1, &IrType::Ptr, &PbType::Long);
                fb.call_void("pb_array_tagarray", &[pa0, fb.const_i32(0), pa1]);
            }
            "ARRAY_TAGARRAY_ERASE" => {
                let a0 = self.compile_expr(fb, &call.args[0])?;
                let pa0 = self.convert_value(fb, &a0, &IrType::Ptr, &PbType::Long);
                fb.call_void("pb_array_tagarray_erase", &[pa0, fb.const_i32(0)]);
            }
            "ACCEL_ATTACH" => {
                // batch 211: the first of the five parked statements gets a real
                // runtime.  `ACCEL ATTACH hDlg, keyTbl() [TO hAccel]`: the table is
                // built from the array's records, remembered for the message pumps
                // (otherwise nothing would ever consult it), and the handle can be
                // kept with the optional TO clause.  The element count is the
                // array's declared size - a compiler knows a constant array, and
                // the official form passes no count for exactly that reason.
                let a0 = self.compile_expr(fb, &call.args[0])?;
                let ph = self.convert_value(fb, &a0, &IrType::Ptr, &PbType::Long);
                let mut count: i32 = 0;
                let mut pk = fb.const_i64(0);
                if let Some(a1) = call.args.get(1) {
                    // `keys()` reaches codegen as ArrayAccess or as FunctionCall
                    // depending on how the parser resolved the name; matching only
                    // one of them made the count silently 0, so no table was built.
                    let arr_name = match a1 {
                        Expr::ArrayAccess(an, _) => Some(an.clone()),
                        Expr::FunctionCall(an, _) => Some(an.clone()),
                        _ => None,
                    };
                    if let Some(an) = arr_name {
                        if let Some(ai) = self.symbols.lookup_array(&normalize_name(&an)).cloned() {
                            count = ai.total_elements as i32;
                        }
                    }
                    // The pointer must be &arr[0].  The *value* path for an
                    // array yields the loaded element, so converting that integer
                    // to a pointer handed the runtime address 4259849 and the
                    // sample died with an access violation; the lvalue path with
                    // empty indices is exactly "address of element 0".
                    match self.compile_lvalue_ptr(fb, a1) {
                        Ok((p, _pb)) => pk = p,
                        Err(_) => {
                            let v = self.compile_expr(fb, a1)?;
                            pk = self.convert_value(fb, &v, &IrType::Ptr, &PbType::Long);
                        }
                    }
                }
                let hn = fb.call(
                    &IrType::I64,
                    "pb_accel_attach",
                    &[ph, pk, fb.const_i32(count)],
                );
                if matches!(call.args.get(2), Some(Expr::IntegerLit(1))) {
                    if let Some(t) = call.args.get(3) {
                        let target = self.compile_lvalue_ptr(fb, t)?;
                        let converted = self.convert_value(fb, &hn, &target.0.ty, &target.1);
                        fb.store(&converted, &target.0);
                    }
                }
            }
            // OOP event bus (batch 222).  EVENT SOURCE only exists inside a
            // CLASS block (parser collects it into the TypeDecl); reaching codegen
            // at statement position is a user error already reported by the
            // parser - keep it loud here too.  EVENTS FROM/END and RAISEEVENT are
            // real implementations against the simplified class-wide event bus:
            //   EVENTS FROM obj   -> build <Class>'s handler table (methods with
            //                        <= 1 real parameter: this + at most one arg)
            //                        and attach obj to the bus
            //   EVENTS END obj    -> detach obj
            //   RAISEEVENT I.M(a) -> validate I is an EVENT SOURCE of the current
            //                        class, then fire M(a) to every subscriber
            "EVENT_SOURCE" => {
                self.warnings.push(format!(
                    "line {}: `EVENT SOURCE` is only valid inside a CLASS/END CLASS block - no code generated",
                    call.line
                ));
            }
            "EVENTS_FROM" | "EVENTS_END" => {
                if let Some(obj) = call.args.first() {
                    if let Some(class) = self.struct_type_of_expr(obj) {
                        let (obj_ptr, _, _) = self.lvalue_ptr(fb, obj).ok_or_else(|| {
                            pb::error::PbError::parser(
                                format!(
                                    "`{}` object cannot be resolved on line {}",
                                    name, call.line
                                ),
                                None,
                                call.line,
                            )
                        })?;
                        if name == "EVENTS_FROM" {
                            let tbl = fb.call(&IrType::Ptr, "pb_events_new_table", &[]);
                            let prefix = format!("{}_", class);
                            let mut fns: Vec<(String, FuncInfo)> = self
                                .functions
                                .iter()
                                .chain(self.subs.iter())
                                .filter(|(k, _)| k.starts_with(&prefix))
                                .map(|(k, v)| (k.clone(), v.clone()))
                                .collect();
                            fns.sort_by(|a, b| a.0.cmp(&b.0));
                            for (fn_name, info) in &fns {
                                // a usable event handler takes this + at most one
                                // argument; anything wider cannot be called through
                                // the (void*, i32) bus signature
                                if info.params.len() <= 2 {
                                    let mname = &fn_name[prefix.len()..];
                                    let next_id = self.event_method_ids.len() as i32;
                                    let id = *self
                                        .event_method_ids
                                        .entry(mname.to_uppercase())
                                        .or_insert(next_id);
                                    fb.call_void(
                                        "pb_events_add",
                                        &[
                                            tbl.clone(),
                                            fb.const_i32(id),
                                            Val::new(format!("@{}", info.ir_name), IrType::Ptr),
                                        ],
                                    );
                                }
                            }
                            fb.call_void("pb_events_attach", &[obj_ptr, tbl]);
                        } else {
                            fb.call_void("pb_events_detach", &[obj_ptr]);
                        }
                    } else {
                        return Err(pb::error::PbError::parser(
                            format!(
                                "`{}` requires an object variable of a CLASS type on line {}",
                                name, call.line
                            ),
                            None,
                            call.line,
                        ));
                    }
                }
                return Ok(());
            }
            "RAISEEVENT" => {
                let (iface, method) = match (call.args.first(), call.args.get(1)) {
                    (Some(Expr::StringLit(a)), Some(Expr::StringLit(b))) => {
                        (a.to_uppercase(), b.to_uppercase())
                    }
                    _ => {
                        return Err(pb::error::PbError::parser(
                            format!(
                                "RAISEEVENT requires the Iface.Method form on line {}",
                                call.line
                            ),
                            None,
                            call.line,
                        ));
                    }
                };
                // validate: the current function must belong to a class that
                // declares EVENT SOURCE <iface>
                let cur = self.current_fn_name.clone().unwrap_or_default();
                let allowed = self.event_sources.iter().any(|(cls, ifaces)| {
                    cur.strip_prefix(cls.as_str())
                        .is_some_and(|rest| rest.starts_with('_'))
                        && ifaces.contains(&iface)
                });
                if !allowed {
                    self.warnings.push(format!(
                        "line {}: RAISEEVENT {}: no class in scope declares EVENT SOURCE {} - no code generated",
                        call.line, method, iface
                    ));
                    return Ok(());
                }
                let next_id = self.event_method_ids.len() as i32;
                let id = *self
                    .event_method_ids
                    .entry(method.clone())
                    .or_insert(next_id);
                let arg = if call.args.len() > 2 {
                    let v = self.compile_expr(fb, &call.args[2])?;
                    self.to_i32(fb, &v)
                } else {
                    fb.const_i32(0)
                };
                fb.call_void("pb_raise_event", &[fb.const_i32(id), arg]);
                return Ok(());
            }
            "XPRINT_GET_MARGIN" => {
                let mut ps = Vec::new();
                for i in 0..4 {
                    if let Some((p, _, _)) = self.lvalue_ptr(fb, &call.args[i]) {
                        ps.push(p);
                    }
                }
                if ps.len() == 4 {
                    fb.call_void(
                        "pb_xprint_get_margin",
                        &[ps[0].clone(), ps[1].clone(), ps[2].clone(), ps[3].clone()],
                    );
                }
            }
            "DISPLAY_OPENFILE" | "DISPLAY_SAVEFILE" => {
                // Result is always the LAST arg; filter/initial-dir are optional
                // and default to empty strings (supports: `DISPLAY OPENFILE title TO s`).
                let n = call.args.len();
                if n >= 2 {
                    let t0 = self.compile_expr(fb, &call.args[0])?;
                    let pt0 = self.convert_value(fb, &t0, &IrType::Ptr, &PbType::String);
                    let pt1 = if n >= 3 {
                        let v = self.compile_expr(fb, &call.args[1])?;
                        self.convert_value(fb, &v, &IrType::Ptr, &PbType::String)
                    } else {
                        Val::new(self.empty_string_name.clone(), IrType::Ptr)
                    };
                    let pt2 = if n >= 4 {
                        let v = self.compile_expr(fb, &call.args[2])?;
                        self.convert_value(fb, &v, &IrType::Ptr, &PbType::String)
                    } else {
                        Val::new(self.empty_string_name.clone(), IrType::Ptr)
                    };
                    if let Some((rp, _, _)) = self.lvalue_ptr(fb, &call.args[n - 1]) {
                        let sym = if name == "DISPLAY_OPENFILE" {
                            "pb_display_openfile"
                        } else {
                            "pb_display_savefile"
                        };
                        fb.call_void(sym, &[pt0, pt1, pt2, rp]);
                    }
                }
            }
            "DISPLAY_COLOR" => {
                if let Some(a) = call.args.first() {
                    if let Some((p, _, _)) = self.lvalue_ptr(fb, a) {
                        fb.call_void("pb_display_color", &[p]);
                    }
                }
            }
            "DISPLAY_FONT" => {
                if let Some(a) = call.args.first() {
                    if let Some((p, _, _)) = self.lvalue_ptr(fb, a) {
                        fb.call_void("pb_display_font", &[p]);
                    }
                }
            }
            "DISPLAY_BROWSE" => {
                // `DISPLAY BROWSE title TO s` or `DISPLAY BROWSE title, initialdir TO s`.
                let n = call.args.len();
                if n >= 2 {
                    let t0 = self.compile_expr(fb, &call.args[0])?;
                    let pt0 = self.convert_value(fb, &t0, &IrType::Ptr, &PbType::String);
                    let pt1 = if n >= 3 {
                        let v = self.compile_expr(fb, &call.args[1])?;
                        self.convert_value(fb, &v, &IrType::Ptr, &PbType::String)
                    } else {
                        Val::new(self.empty_string_name.clone(), IrType::Ptr)
                    };
                    if let Some((rp, _, _)) = self.lvalue_ptr(fb, &call.args[n - 1]) {
                        fb.call_void("pb_display_browse", &[pt0, pt1, rp]);
                    }
                }
            }

            "ARRAY_REDIM_INCR" => {
                let a0 = self.compile_expr(fb, &call.args[0])?;
                let pa0 = self.convert_value(fb, &a0, &IrType::Ptr, &PbType::Long);
                let a1 = self.compile_expr(fb, &call.args[1])?;
                let ia1 = self.convert_value(fb, &a1, &IrType::I32, &PbType::Long);
                fb.call_void(
                    "pb_array_redim_incr",
                    &[pa0, fb.const_i32(4), fb.const_i32(0), ia1],
                );
            }
            "ARRAY_REDIM_DECR" => {
                let a0 = self.compile_expr(fb, &call.args[0])?;
                let pa0 = self.convert_value(fb, &a0, &IrType::Ptr, &PbType::Long);
                let a1 = self.compile_expr(fb, &call.args[1])?;
                let ia1 = self.convert_value(fb, &a1, &IrType::I32, &PbType::Long);
                fb.call_void(
                    "pb_array_redim_decr",
                    &[pa0, fb.const_i32(4), fb.const_i32(0), ia1],
                );
            }
            "GRAPHIC_GET_BITS" => {
                // GRAPHIC GET BITS TO bitvar$ — whole bitmap as DIB string (batch 64)
                if let Some(dst) = call.args.first() {
                    let (ptr, _) = self.compile_lvalue_ptr(fb, dst)?;
                    fb.call_void("pb_graphic_get_bits", &[ptr]);
                }
                return Ok(());
            }
            "GRAPHIC_SET_BITS" => {
                // GRAPHIC SET BITS bitexpr$ — replace bitmap from DIB string (batch 64)
                if let Some(src) = call.args.first() {
                    let sv = self.compile_expr(fb, src)?;
                    fb.call_void("pb_graphic_set_bits", &[sv]);
                }
                return Ok(());
            }
            "GRAPHIC_GET_SCALE" => {
                // GRAPHIC GET SCALE TO x1!, y1!, x2!, y2!  (batch 64)
                if call.args.len() == 4 {
                    let mut ptrs = Vec::new();
                    for a in &call.args {
                        let (ptr, _) = self.compile_lvalue_ptr(fb, a)?;
                        ptrs.push(ptr);
                    }
                    fb.call_void("pb_graphic_get_scale", &ptrs);
                }
                return Ok(());
            }
            "GRAPHIC_SCALE" => {
                // GRAPHIC SCALE (x1!,y1!)-(x2!,y2!)  (batch 64)
                if call.args.len() == 4 {
                    let mut vals = Vec::new();
                    for a in &call.args {
                        let v0 = self.compile_expr(fb, a)?;
                        vals.push(self.convert_value(fb, &v0, &IrType::Float, &PbType::Single));
                    }
                    fb.call_void("pb_graphic_scale", &vals);
                }
                return Ok(());
            }
            "GRAPHIC_SCALE_PIXELS" => {
                fb.call_void("pb_graphic_scale_pixels", &[]);
                return Ok(());
            }
            "GRAPHIC_WINDOW_NEW" | "GRAPHIC_WINDOW_TEXT" => {
                // args: caption$, x&, y&, w&, h&, font, TO target, show
                let cap = self.compile_expr(fb, &call.args[0])?;
                let c = self.convert_value(fb, &cap, &IrType::Ptr, &PbType::String);
                let x0 = self.compile_expr(fb, &call.args[1])?;
                let x = self.to_i32(fb, &x0);
                let y0 = self.compile_expr(fb, &call.args[2])?;
                let y = self.to_i32(fb, &y0);
                let w0 = self.compile_expr(fb, &call.args[3])?;
                let w = self.to_i32(fb, &w0);
                let h0 = self.compile_expr(fb, &call.args[4])?;
                let h = self.to_i32(fb, &h0);
                let f0 = self.compile_expr(fb, &call.args[5])?;
                let fh = fb.inttoptr(&f0);
                let s0 = self.compile_expr(fb, &call.args[7])?;
                let show = self.to_i32(fb, &s0);
                let fname = match call.name.as_str() {
                    "GRAPHIC_WINDOW_NEW" => "pb_graphic_window_new",
                    _ => "pb_graphic_window_text",
                };
                let hc = fb.call(&IrType::Ptr, fname, &[c, x, y, w, h, fh, show]);
                // a window handle is a pointer: route it through the
                // destination type before storing (the batch 166/168 family)
                if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[6]) {
                    let cv = self.convert_value(fb, &hc, &ty, &pty);
                    fb.store(&cv, &ptr);
                }
            }
            "GRAPHIC_WINDOW_CLICK" => {
                // args: hWin&, TO click&, x!, y!
                if call.args.len() == 4 {
                    let h0 = self.compile_expr(fb, &call.args[0])?;
                    let hp = fb.inttoptr(&h0);
                    let p1 = self.lvalue_ptr(fb, &call.args[1]);
                    let p2 = self.lvalue_ptr(fb, &call.args[2]);
                    let p3 = self.lvalue_ptr(fb, &call.args[3]);
                    if let (Some((a, _, _)), Some((b, _, _)), Some((c, _, _))) = (p1, p2, p3) {
                        fb.call_void("pb_graphic_window_click", &[hp, a, b, c]);
                    }
                }
            }
            "GRAPHIC_WINDOW_END"
            | "GRAPHIC_WINDOW_HIDE"
            | "GRAPHIC_WINDOW_MINIMIZE"
            | "GRAPHIC_WINDOW_NORMALIZE"
            | "GRAPHIC_WINDOW_STABILIZE"
            | "GRAPHIC_WINDOW_NONSTABLE" => {
                // args: [hWin&]; a missing or zero handle means the selected window
                let fname = match call.name.as_str() {
                    "GRAPHIC_WINDOW_END" => "pb_graphic_window_end",
                    "GRAPHIC_WINDOW_HIDE" => "pb_graphic_window_hide",
                    "GRAPHIC_WINDOW_MINIMIZE" => "pb_graphic_window_minimize",
                    "GRAPHIC_WINDOW_NORMALIZE" => "pb_graphic_window_normalize",
                    /* STABILIZE and NONSTABLE are the two sides of one switch */
                    _ => "pb_graphic_window_stabilize",
                };
                let h0 = self.compile_expr(fb, &call.args[0])?;
                let hp = fb.inttoptr(&h0);
                if call.name.as_str() == "GRAPHIC_WINDOW_STABILIZE"
                    || call.name.as_str() == "GRAPHIC_WINDOW_NONSTABLE"
                {
                    let on = if call.name.as_str() == "GRAPHIC_WINDOW_STABILIZE" {
                        fb.const_i32(1)
                    } else {
                        fb.const_i32(0)
                    };
                    fb.call_void(fname, &[hp, on]);
                } else {
                    fb.call_void(fname, &[hp]);
                }
            }
            "GRAPHIC_SET_SIZE" => {
                // GRAPHIC SET SIZE nWide&, nHigh&  (batch 65)
                if call.args.len() == 2 {
                    let w0 = self.compile_expr(fb, &call.args[0])?;
                    let w = self.to_i32(fb, &w0);
                    let h0 = self.compile_expr(fb, &call.args[1])?;
                    let h = self.to_i32(fb, &h0);
                    fb.call_void("pb_graphic_set_size", &[w, h]);
                }
                return Ok(());
            }
            "GRAPHIC_SET_CLIP" => {
                // GRAPHIC SET CLIP l!, t!, r!, b!  (batch 65)
                if call.args.len() == 4 {
                    let mut vals = Vec::new();
                    for a in &call.args {
                        let v0 = self.compile_expr(fb, a)?;
                        vals.push(self.convert_value(fb, &v0, &IrType::Float, &PbType::Single));
                    }
                    fb.call_void("pb_graphic_set_clip", &vals);
                }
                return Ok(());
            }
            "GRAPHIC_SET_VIRTUAL" => {
                // GRAPHIC SET VIRTUAL nWide&, nHigh& [,USERSIZE]  (batch 65)
                // batch 219: the official syntax requires all 2 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 2 {
                    return Err(pb::error::PbError::parser(
                        "GRAPHIC_SET_VIRTUAL requires its arguments: GRAPHIC SET VIRTUAL nWidth& ,"
                            .to_string(),
                        None,
                        call.line,
                    ));
                }
                if call.args.len() >= 2 {
                    let w0 = self.compile_expr(fb, &call.args[0])?;
                    let w = self.to_i32(fb, &w0);
                    let h0 = self.compile_expr(fb, &call.args[1])?;
                    let h = self.to_i32(fb, &h0);
                    fb.call_void("pb_graphic_set_virtual", &[w, h]);
                }
                return Ok(());
            }
            "GRAPHIC_SET_FIXED" => {
                fb.call_void("pb_graphic_set_fixed", &[]);
            }
            "GRAPHIC_SET_FONT" => {
                let h0 = self.compile_expr(fb, &call.args[0])?;
                let h64 = self.convert_value(fb, &h0, &IrType::I64, &PbType::Quad);
                fb.call_void("pb_graphic_set_font", &[h64]);
            }
            "GRAPHIC_SET_WORDWRAP" => {
                // GRAPHIC SET WORDWRAP n&  (batch 65)
                if call.args.len() == 1 {
                    let w0 = self.compile_expr(fb, &call.args[0])?;
                    let w = self.to_i32(fb, &w0);
                    fb.call_void("pb_graphic_set_wordwrap", &[w]);
                }
                return Ok(());
            }
            "GRAPHIC_GET_WORDWRAP" => {
                // GRAPHIC GET WORDWRAP TO n&  (batch 65)
                if let Some(dst) = call.args.first() {
                    let (ptr, _) = self.compile_lvalue_ptr(fb, dst)?;
                    fb.call_void("pb_graphic_get_wordwrap", &[ptr]);
                }
                return Ok(());
            }
            "GRAPHIC_SET_AUTOSIZE" => {
                // GRAPHIC SET AUTOSIZE nWidth, nHeight [,USERSIZE]  (batch 64)
                // batch 219: the official syntax requires all 2 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 2 {
                    return Err(pb::error::PbError::parser(
                    "GRAPHIC_SET_AUTOSIZE requires its arguments: GRAPHIC SET AUTOSIZE nWidth ,"
                        .to_string(),
                    None,
                    call.line,
                ));
                }
                if call.args.len() >= 2 {
                    let w0 = self.compile_expr(fb, &call.args[0])?;
                    let w = self.to_i32(fb, &w0);
                    let h0 = self.compile_expr(fb, &call.args[1])?;
                    let h = self.to_i32(fb, &h0);
                    fb.call_void("pb_graphic_set_autosize", &[w, h]);
                }
                return Ok(());
            }
            "GRAPHIC_GET_POS" => {
                if let Some((xp, _, _)) = self.lvalue_ptr(fb, &call.args[0]) {
                    if let Some((yp, _, _)) = self.lvalue_ptr(fb, &call.args[1]) {
                        fb.call_void("pb_graphic_get_pos", &[xp, yp]);
                    }
                }
            }
            "GRAPHIC_GET_CLIP" => {
                if let Some((wp, _, _)) = self.lvalue_ptr(fb, &call.args[0]) {
                    if let Some((hp, _, _)) = self.lvalue_ptr(fb, &call.args[1]) {
                        fb.call_void("pb_graphic_get_clip", &[wp, hp]);
                    }
                }
            }
            "GRAPHIC_GET_VIEW" => {
                if let Some((xp, _, _)) = self.lvalue_ptr(fb, &call.args[0]) {
                    if let Some((yp, _, _)) = self.lvalue_ptr(fb, &call.args[1]) {
                        fb.call_void("pb_graphic_get_view", &[xp, yp]);
                    }
                }
            }
            "GRAPHIC_GET_LINES" => {
                if let Some((np, _, _)) = self.lvalue_ptr(fb, &call.args[0]) {
                    fb.call_void("pb_graphic_get_lines", &[np]);
                }
            }
            "GRAPHIC_GET_WRAP" => {
                if let Some((wp, _, _)) = self.lvalue_ptr(fb, &call.args[0]) {
                    fb.call_void("pb_graphic_get_wrap", &[wp]);
                }
            }
            "GRAPHIC_SET_VIEW" => {
                let x0 = self.compile_expr(fb, &call.args[0])?;
                let y0 = self.compile_expr(fb, &call.args[1])?;
                let xv = self.convert_value(fb, &x0, &IrType::Float, &PbType::Single);
                let yv = self.convert_value(fb, &y0, &IrType::Float, &PbType::Single);
                fb.call_void("pb_graphic_set_view", &[xv, yv]);
            }
            "GRAPHIC_SET_WRAP" => {
                let w0 = self.compile_expr(fb, &call.args[0])?;
                let wv = self.convert_value(fb, &w0, &IrType::I32, &PbType::Long);
                fb.call_void("pb_graphic_set_wrap", &[wv]);
            }
            "GRAPHIC_SET_POS" => {
                let x0 = self.compile_expr(fb, &call.args[0])?;
                let y0 = self.compile_expr(fb, &call.args[1])?;
                let xv = self.convert_value(fb, &x0, &IrType::Float, &PbType::Single);
                let yv = self.convert_value(fb, &y0, &IrType::Float, &PbType::Single);
                fb.call_void("pb_graphic_set_pos", &[xv, yv]);
            }
            "GRAPHIC_TEXT_SIZE" => {
                let sp = self.compile_str_payload(fb, &call.args[0])?;
                if let Some((wp, _, _)) = self.lvalue_ptr(fb, &call.args[1]) {
                    if let Some((hp, _, _)) = self.lvalue_ptr(fb, &call.args[2]) {
                        fb.call_void("pb_graphic_text_size", &[sp, wp, hp]);
                    }
                }
            }
            "GRAPHIC_GET_STRETCHMODE" => {
                if let Some((mp, _, _)) = self.lvalue_ptr(fb, &call.args[0]) {
                    fb.call_void("pb_graphic_get_stretchmode", &[mp]);
                }
            }
            "GRAPHIC_SET_STRETCHMODE" => {
                let m0 = self.compile_expr(fb, &call.args[0])?;
                let mv = self.convert_value(fb, &m0, &IrType::I32, &PbType::Long);
                fb.call_void("pb_graphic_set_stretchmode", &[mv]);
            }
            "GRAPHIC_GET_CAPTION" => {
                if let Some((sp, _, _)) = self.lvalue_ptr(fb, &call.args[0]) {
                    fb.call_void("pb_graphic_get_caption", &[sp]);
                }
            }
            "GRAPHIC_SET_CAPTION" => {
                let sp = self.compile_str_payload(fb, &call.args[0])?;
                fb.call_void("pb_graphic_set_caption", &[sp]);
            }
            "GRAPHIC_SET_TEXTALIGN" => {
                let a0 = self.compile_expr(fb, &call.args[0])?;
                let av = self.convert_value(fb, &a0, &IrType::I32, &PbType::Long);
                fb.call_void("pb_graphic_set_textalign", &[av]);
            }
            "GRAPHIC_GET_TEXTALIGN" => {
                if let Some((ptr, _, _)) = self.lvalue_ptr(fb, &call.args[0]) {
                    fb.call_void("pb_graphic_get_textalign", &[ptr]);
                }
            }
            "GRAPHIC_GET_CANVAS" | "GRAPHIC_GET_DC" => {
                let f = if call.name == "GRAPHIC_GET_CANVAS" {
                    "pb_graphic_get_canvas"
                } else {
                    "pb_graphic_get_dc"
                };
                let v = fb.call(&IrType::I64, f, &[]);
                if let Some((ptr, _, _)) = self.lvalue_ptr(fb, &call.args[0]) {
                    let rv = self.convert_value(fb, &v, &IrType::I64, &PbType::Long);
                    fb.store(&rv, &ptr);
                }
            }
            "GRAPHIC_GET_MIX" => {
                if let Some((ptr, _, _)) = self.lvalue_ptr(fb, &call.args[0]) {
                    fb.call_void("pb_graphic_get_mix", &[ptr]);
                }
            }
            "GRAPHIC_SET_MIX" => {
                let m = self.compile_expr(fb, &call.args[0])?;
                let mv = self.convert_value(fb, &m, &IrType::I32, &PbType::Long);
                fb.call_void("pb_graphic_set_mix", &[mv]);
            }
            "GRAPHIC_INSTAT" => {
                let r = fb.call(&IrType::I32, "pb_graphic_instat", &[]);
                if let Some(a0) = call.args.first().cloned() {
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &a0) {
                        let cv = self.convert_value(fb, &r, &ty, &pty);
                        fb.store(&cv, &ptr);
                    }
                }
                return Ok(());
            }
            "GRAPHIC_INPUT_FLUSH" => {
                fb.call_void("pb_graphic_input_flush", &[]);
                return Ok(());
            }
            "GRAPHIC_INKEY" => {
                // GRAPHIC INKEY$ TO var$ - a BSTR is stored straight into the string slot
                let v = fb.call(&IrType::Ptr, "pb_graphic_inkey", &[]);
                if let Some(a0) = call.args.first().cloned() {
                    if let Some((ptr, _ty, _pty)) = self.lvalue_ptr(fb, &a0) {
                        fb.store(&v, &ptr);
                    }
                }
                return Ok(());
            }
            "GRAPHIC_WAITKEY" => {
                // args: [KeyMask$, TimeOut&, dest?] - both slots always present
                let m = self.compile_expr(fb, &call.args[0])?;
                let t = {
                    let e = self.compile_expr(fb, &call.args[1])?;
                    self.convert_value(fb, &e, &IrType::I32, &PbType::Long)
                };
                let v = fb.call(&IrType::Ptr, "pb_graphic_waitkey", &[m, t]);
                if let Some(a2) = call.args.get(2).cloned() {
                    if let Some((ptr, _ty, _pty)) = self.lvalue_ptr(fb, &a2) {
                        fb.store(&v, &ptr);
                    }
                }
                return Ok(());
            }
            "GRAPHIC_SPLIT" | "GRAPHIC_SPLIT_WORD" => {
                // GRAPHIC SPLIT [WORD] MainStr, Part1Len TO Part1Var, Part2Var
                // Nothing was emitted for these two before: the parser built the call and
                // the codegen dropped it silently.
                if call.args.len() >= 4 {
                    let src = self.compile_expr(fb, &call.args[0])?;
                    let w = {
                        let e = self.compile_expr(fb, &call.args[1])?;
                        self.convert_value(fb, &e, &IrType::I32, &PbType::Long)
                    };
                    let p1 = self.lvalue_ptr(fb, &call.args[2]);
                    let p2 = self.lvalue_ptr(fb, &call.args[3]);
                    if let (Some((a, _, _)), Some((b, _, _))) = (p1, p2) {
                        let f = if call.name == "GRAPHIC_SPLIT_WORD" {
                            "pb_graphic_split_word"
                        } else {
                            "pb_graphic_split"
                        };
                        fb.call_void(f, &[src, w, a, b]);
                    }
                }
                return Ok(());
            }
            "GRAPHIC_LINE_INPUT" => {
                // args: [prompt, dest] or [dest]
                let (prompt, dest) = if call.args.len() >= 2 {
                    (
                        self.compile_expr(fb, &call.args[0])?,
                        call.args.get(1).cloned(),
                    )
                } else {
                    (fb.const_null_ptr(), call.args.first().cloned())
                };
                let v = fb.call(&IrType::Ptr, "pb_graphic_line_input", &[prompt]);
                if let Some(a) = dest {
                    if let Some((ptr, _ty, _pty)) = self.lvalue_ptr(fb, &a) {
                        fb.store(&v, &ptr);
                    }
                }
                return Ok(());
            }
            "GRAPHIC_INPUT" => {
                // args: [prompt?, var...] - the same leading-string rule the parser used
                let mut prompt = fb.const_null_ptr();
                let mut start = 0;
                let is_prompt = if call.args.len() >= 2 {
                    match call.args.first() {
                        Some(Expr::StringLit(_)) => true,
                        Some(Expr::Variable(v)) => v.ends_with('$'),
                        _ => false,
                    }
                } else {
                    false
                };
                if is_prompt {
                    prompt = self.compile_expr(fb, &call.args[0])?;
                    start = 1;
                }
                fb.call_void("pb_graphic_input_begin", &[prompt]);
                for (k, a) in call.args[start..].iter().enumerate() {
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, a) {
                        let is_str = matches!(pty, PbType::String | PbType::FixedString(_));
                        if is_str {
                            let v = fb.call(
                                &IrType::Ptr,
                                "pb_graphic_input_field",
                                &[fb.const_i32(k as i32)],
                            );
                            fb.store(&v, &ptr);
                        } else {
                            let v = fb.call(
                                &IrType::Double,
                                "pb_graphic_input_field_num",
                                &[fb.const_i32(k as i32)],
                            );
                            let cv = self.convert_value(fb, &v, &ty, &pty);
                            fb.store(&cv, &ptr);
                        }
                    }
                }
                return Ok(());
            }
            "GRAPHIC_BITMAP_CAPTURE" => {
                // args: hbmp (out, QUAD)   (batch 182, FORK EXTENSION)
                let h = fb.call(&IrType::I64, "pb_graphic_bitmap_capture", &[]);
                if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[0]) {
                    // route through convert_value: a LONG destination must get
                    // a 4-byte store, not the low half of this i64 (batch 183 fix)
                    let cv = self.convert_value(fb, &h, &ty, &pty);
                    fb.store(&cv, &ptr);
                }
            }
            "GRAPHIC_BITMAP_LOAD" => {
                // args: fname$, hbmp (out, QUAD)
                let fname = self.compile_expr(fb, &call.args[0])?;
                let h = fb.call(&IrType::I64, "pb_graphic_bitmap_load", &[fname]);
                if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[1]) {
                    // route through convert_value: a LONG destination must get
                    // a 4-byte store, not the low half of this i64 (batch 183 fix)
                    let cv = self.convert_value(fb, &h, &ty, &pty);
                    fb.store(&cv, &ptr);
                }
            }
            "GRAPHIC_CHR_SIZE" => {
                // args: text$, w&, h& (out)
                let txt = self.compile_expr(fb, &call.args[0])?;
                if let Some((wp, _, _)) = self.lvalue_ptr(fb, &call.args[1]) {
                    if let Some((hp, _, _)) = self.lvalue_ptr(fb, &call.args[2]) {
                        fb.call_void("pb_graphic_chr_size", &[txt, wp, hp]);
                    }
                }
            }
            "GRAPHIC_CELL_SIZE" | "GRAPHIC_CELL" => {
                let f = if call.name == "GRAPHIC_CELL_SIZE" {
                    "pb_graphic_cell_size"
                } else {
                    "pb_graphic_cell"
                };
                let a0 = self.compile_expr(fb, &call.args[0])?;
                let a0v = self.convert_value(fb, &a0, &IrType::I32, &PbType::Long);
                let a1 = self.compile_expr(fb, &call.args[1])?;
                let a1v = self.convert_value(fb, &a1, &IrType::I32, &PbType::Long);
                if let Some((o0p, _, _)) = self.lvalue_ptr(fb, &call.args[2]) {
                    if let Some((o1p, _, _)) = self.lvalue_ptr(fb, &call.args[3]) {
                        fb.call_void(f, &[a0v, a1v, o0p, o1p]);
                    }
                }
            }
            "GRAPHIC_CIRCLE" => {
                // args: x, y, r [, color&]
                let x = self.compile_expr(fb, &call.args[0])?;
                let xv = self.convert_value(fb, &x, &IrType::I32, &PbType::Long);
                let y = self.compile_expr(fb, &call.args[1])?;
                let yv = self.convert_value(fb, &y, &IrType::I32, &PbType::Long);
                let r = self.compile_expr(fb, &call.args[2])?;
                let rv = self.convert_value(fb, &r, &IrType::I32, &PbType::Long);
                let col = if let Some(a3) = call.args.get(3) {
                    let e = self.compile_expr(fb, a3)?;
                    self.convert_value(fb, &e, &IrType::I32, &PbType::Long)
                } else {
                    fb.const_i32(0)
                };
                fb.call_void("pb_graphic_circle", &[xv, yv, rv, col]);
            }
            "GRAPHIC_POLYGON" => {
                // args: x1,y1,x2,y2,... [, color&]  (even count = coords)
                let ncoords = call.args.len() - if call.args.len() % 2 == 1 { 1 } else { 0 };
                let npts = ncoords / 2;
                if npts >= 3 {
                    // build i32 array on stack via llvm alloca of [npts*2 x i32]
                    let arrty = IrType::Array(npts * 2, Box::new(IrType::I32));
                    let ptr = fb.alloca(&arrty);
                    for i in 0..ncoords {
                        let v = self.compile_expr(fb, &call.args[i])?;
                        let iv = self.convert_value(fb, &v, &IrType::I32, &PbType::Long);
                        let elem = fb.gep_byte(&ptr, &fb.const_i32((i as i32) * 4));
                        fb.store(&iv, &elem);
                    }
                    let col = if call.args.len() % 2 == 1 {
                        let e = self.compile_expr(fb, call.args.last().unwrap())?;
                        self.convert_value(fb, &e, &IrType::I32, &PbType::Long)
                    } else {
                        fb.const_i32(0)
                    };
                    fb.call_void("pb_graphic_polygon", &[ptr, fb.const_i32(npts as i32), col]);
                }
            }
            "GRAPHIC_POLYLINE" => {
                // args: x1,y1,x2,y2,... [, color&]
                let ncoords = call.args.len() - if call.args.len() % 2 == 1 { 1 } else { 0 };
                let npts = ncoords / 2;
                if npts >= 2 {
                    let arrty = IrType::Array(npts * 2, Box::new(IrType::I32));
                    let ptr = fb.alloca(&arrty);
                    for i in 0..ncoords {
                        let v = self.compile_expr(fb, &call.args[i])?;
                        let iv = self.convert_value(fb, &v, &IrType::I32, &PbType::Long);
                        let elem = fb.gep_byte(&ptr, &fb.const_i32((i as i32) * 4));
                        fb.store(&iv, &elem);
                    }
                    let col = if call.args.len() % 2 == 1 {
                        let e = self.compile_expr(fb, call.args.last().unwrap())?;
                        self.convert_value(fb, &e, &IrType::I32, &PbType::Long)
                    } else {
                        fb.const_i32(0)
                    };
                    fb.call_void(
                        "pb_graphic_polyline",
                        &[ptr, fb.const_i32(npts as i32), col],
                    );
                }
            }
            "GRAPHIC_ARC" => {
                // args: x1,y1,x2,y2,start,end[,color]
                let mut a = Vec::new();
                for i in 0..6.min(call.args.len()) {
                    let v = self.compile_expr(fb, &call.args[i])?;
                    a.push(self.convert_value(fb, &v, &IrType::I32, &PbType::Long));
                }
                let col = if call.args.len() > 6 {
                    let e = self.compile_expr(fb, &call.args[6])?;
                    self.convert_value(fb, &e, &IrType::I32, &PbType::Long)
                } else {
                    fb.const_i32(0)
                };
                a.push(col);
                fb.call_void("pb_graphic_arc", &a);
            }
            "GRAPHIC_PIE" => {
                // args: x1,y1,x2,y2,start,end[,color[,fillcolor[,fillstyle]]]
                let mut a = Vec::new();
                for i in 0..6.min(call.args.len()) {
                    let v = self.compile_expr(fb, &call.args[i])?;
                    a.push(self.convert_value(fb, &v, &IrType::I32, &PbType::Long));
                }
                for i in 6..9 {
                    if i < call.args.len() {
                        let e = self.compile_expr(fb, &call.args[i])?;
                        a.push(self.convert_value(fb, &e, &IrType::I32, &PbType::Long));
                    } else {
                        a.push(fb.const_i32(0));
                    }
                }
                fb.call_void("pb_graphic_pie", &a);
            }
            "GRAPHIC_PAINT" => {
                // args: x,y[,fill[,border[,style]]]
                let mut a = Vec::new();
                for i in 0..2.min(call.args.len()) {
                    let v = self.compile_expr(fb, &call.args[i])?;
                    a.push(self.convert_value(fb, &v, &IrType::I32, &PbType::Long));
                }
                for i in 2..5 {
                    if i < call.args.len() {
                        let e = self.compile_expr(fb, &call.args[i])?;
                        a.push(self.convert_value(fb, &e, &IrType::I32, &PbType::Long));
                    } else {
                        a.push(fb.const_i32(0));
                    }
                }
                fb.call_void("pb_graphic_paint", &a);
            }
            "GRAPHIC_REDRAW" => {
                // batch 181
                fb.call_void("pb_graphic_redraw", &[]);
            }
            "GRAPHIC_SET_FOCUS" => {
                fb.call_void("pb_graphic_set_focus", &[]);
            }
            "GRAPHIC_SET_LOC" | "GRAPHIC_SET_CLIENT" => {
                // both take a pair of coordinates / sizes
                let f = if call.name == "GRAPHIC_SET_LOC" {
                    "pb_graphic_set_loc"
                } else {
                    "pb_graphic_set_client"
                };
                if call.args.len() == 2 {
                    let a0 = self.compile_expr(fb, &call.args[0])?;
                    let a = self.convert_value(fb, &a0, &IrType::I64, &PbType::Quad);
                    let b0 = self.compile_expr(fb, &call.args[1])?;
                    let b = self.convert_value(fb, &b0, &IrType::I64, &PbType::Quad);
                    fb.call_void(f, &[a, b]);
                }
            }
            "GRAPHIC_SET_OVERLAP" => {
                // the operand is optional; a missing one means "enable"
                let v = if let Some(a0) = call.args.first() {
                    let e = self.compile_expr(fb, a0)?;
                    self.convert_value(fb, &e, &IrType::I32, &PbType::Long)
                } else {
                    fb.const_i32(1)
                };
                fb.call_void("pb_graphic_set_overlap", &[v]);
            }
            "GRAPHIC_GET_OVERLAP" => {
                if let Some(a0) = call.args.first() {
                    let r = fb.call(&IrType::I32, "pb_graphic_get_overlap", &[]);
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, a0) {
                        let cv = self.convert_value(fb, &r, &ty, &pty);
                        fb.store(&cv, &ptr);
                    }
                }
            }
            "GRAPHIC_SET_SCROLLTEXT" => {
                // the operand is optional; a missing one means "enable"
                let v = if let Some(a0) = call.args.first() {
                    let e = self.compile_expr(fb, a0)?;
                    self.convert_value(fb, &e, &IrType::I32, &PbType::Long)
                } else {
                    fb.const_i32(1)
                };
                fb.call_void("pb_graphic_set_scrolltext", &[v]);
            }
            "GRAPHIC_GET_SCROLLTEXT" => {
                if let Some(a0) = call.args.first() {
                    let r = fb.call(&IrType::I32, "pb_graphic_get_scrolltext", &[]);
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, a0) {
                        let cv = self.convert_value(fb, &r, &ty, &pty);
                        fb.store(&cv, &ptr);
                    }
                }
            }
            "GRAPHIC_IMAGELIST" => {
                // args: x, y, hLst(64-bit handle), index, overlay, style
                let mut vals = Vec::new();
                for (i, a) in call.args.iter().enumerate() {
                    let v = self.compile_expr(fb, a)?;
                    if i == 2 {
                        // a handle must keep its width; narrowing it to i32
                        // leaves the upper half of the register undefined
                        vals.push(v);
                    } else {
                        vals.push(self.convert_value(fb, &v, &IrType::I32, &PbType::Long));
                    }
                }
                while vals.len() < 6 {
                    vals.push(fb.const_i32(0));
                }
                fb.call_void(
                    "pb_graphic_imagelist",
                    &[
                        vals[0].clone(),
                        vals[1].clone(),
                        vals[2].clone(),
                        vals[3].clone(),
                        vals[4].clone(),
                        vals[5].clone(),
                    ],
                );
            }
            "GRAPHIC_RENDER" | "GRAPHIC_RENDER_ICON" => {
                let f = if call.name == "GRAPHIC_RENDER" {
                    "pb_graphic_render"
                } else {
                    "pb_graphic_render_icon"
                };
                if !call.args.is_empty() {
                    let nm = self.compile_expr(fb, &call.args[0])?;
                    let mut vals = Vec::new();
                    for a in call.args.iter().skip(1) {
                        let v = self.compile_expr(fb, a)?;
                        vals.push(self.convert_value(fb, &v, &IrType::I32, &PbType::Long));
                    }
                    while vals.len() < 4 {
                        vals.push(fb.const_i32(0));
                    }
                    fb.call_void(
                        f,
                        &[
                            nm,
                            vals[0].clone(),
                            vals[1].clone(),
                            vals[2].clone(),
                            vals[3].clone(),
                        ],
                    );
                }
            }
            "GRAPHIC_STRETCH" | "GRAPHIC_STRETCH_PAGE" => {
                let f = if call.name == "GRAPHIC_STRETCH" {
                    "pb_graphic_stretch"
                } else {
                    "pb_graphic_stretch_page"
                };
                // args: hBmp(64-bit handle), ID, then the coordinates
                let mut vals = Vec::new();
                for (i, a) in call.args.iter().enumerate() {
                    let v = self.compile_expr(fb, a)?;
                    if i == 0 {
                        vals.push(v);
                    } else {
                        vals.push(self.convert_value(fb, &v, &IrType::I32, &PbType::Long));
                    }
                }
                while vals.len() < 12 {
                    vals.push(fb.const_i32(0));
                }
                if call.name == "GRAPHIC_STRETCH" {
                    let mut all = Vec::new();
                    for v in vals.iter().take(12) {
                        all.push(v.clone());
                    }
                    fb.call_void(f, &all);
                } else {
                    // the PAGE form parses hBmp, ID [, Mix, Stretch], so the
                    // operands are indices 2 and 3 - 10/11 are only padding
                    fb.call_void(
                        f,
                        &[
                            vals[0].clone(),
                            vals[1].clone(),
                            vals[2].clone(),
                            vals[3].clone(),
                        ],
                    );
                }
            }
            "GRAPHIC_GET_CLIENT" | "GRAPHIC_GET_LOC" => {
                let f = if call.name == "GRAPHIC_GET_CLIENT" {
                    "pb_graphic_get_client"
                } else {
                    "pb_graphic_get_loc"
                };
                let mut ptrs = Vec::new();
                for a0 in call.args.iter() {
                    if let Some((ptr, _, _)) = self.lvalue_ptr(fb, a0) {
                        ptrs.push(ptr);
                    }
                }
                if ptrs.len() == 2 {
                    fb.call_void(f, &[ptrs[0].clone(), ptrs[1].clone()]);
                }
            }
            "GRAPHIC_PRINT" => {
                if call.args.is_empty() {
                    fb.call_void(
                        "pb_graphic_print_str",
                        &[Val::new(self.empty_string_name.clone(), IrType::Ptr)],
                    );
                } else {
                    for arg in &call.args {
                        let v = self.compile_expr(fb, arg)?;
                        let s = self.convert_value(fb, &v, &IrType::Ptr, &PbType::String);
                        fb.call_void("pb_graphic_print_str", &[s]);
                    }
                }
                return Ok(());
            }
            "GRAPHIC_COLOR" => {
                // args: fore& [, back&]
                let fore = self.compile_expr(fb, &call.args[0])?;
                let f = self.convert_value(fb, &fore, &IrType::I32, &PbType::Long);
                let back = if let Some(a1) = call.args.get(1) {
                    let e = self.compile_expr(fb, a1)?;
                    self.convert_value(fb, &e, &IrType::I32, &PbType::Long)
                } else {
                    fb.const_i32(0)
                };
                fb.call_void("pb_graphic_color", &[f, back]);
            }
            "GRAPHIC_GET_PIXEL" => {
                // args: x&, y&, dst& (out)
                let x = self.compile_expr(fb, &call.args[0])?;
                let xv = self.convert_value(fb, &x, &IrType::I32, &PbType::Long);
                let y = self.compile_expr(fb, &call.args[1])?;
                let yv = self.convert_value(fb, &y, &IrType::I32, &PbType::Long);
                if let Some((ptr, _, _)) = self.lvalue_ptr(fb, &call.args[2]) {
                    fb.call_void("pb_graphic_get_pixel", &[xv, yv, ptr]);
                }
            }
            "GRAPHIC_COPY" => {
                // args: x1,y1,x2,y2,x3,y3
                let mut ia = Vec::new();
                for i in 0..6 {
                    let v = self.compile_expr(fb, &call.args[i])?;
                    ia.push(self.convert_value(fb, &v, &IrType::I32, &PbType::Long));
                }
                fb.call_void("pb_graphic_copy", &ia);
            }
            "GRAPHIC_WIDTH" | "GRAPHIC_STYLE" => {
                // args: [n&]
                let f = if call.name == "GRAPHIC_STYLE" {
                    "pb_graphic_style"
                } else {
                    "pb_graphic_width"
                };
                let v = if let Some(a0) = call.args.first() {
                    let e = self.compile_expr(fb, a0)?;
                    self.convert_value(fb, &e, &IrType::I32, &PbType::Long)
                } else {
                    fb.const_i32(0)
                };
                fb.call_void(f, &[v]);
            }
            "GRAPHIC_SAVE" => {
                // args: BmpName$
                let fname = self.compile_expr(fb, &call.args[0])?;
                fb.call_void("pb_graphic_save", &[fname]);
            }
            "GRAPHIC_BOX" | "GRAPHIC_ELLIPSE" => {
                // args: x1,y1,x2,y2 [,corner[,color[,fillcolor[,fillstyle]]]]
                let f = if call.name == "GRAPHIC_BOX" {
                    "pb_graphic_box"
                } else {
                    "pb_graphic_ellipse"
                };
                let mut ia = Vec::new();
                for i in 0..4 {
                    let v = self.compile_expr(fb, &call.args[i])?;
                    ia.push(self.convert_value(fb, &v, &IrType::I32, &PbType::Long));
                }
                let color = if let Some(a4) = call.args.get(4) {
                    let v = self.compile_expr(fb, a4)?;
                    self.convert_value(fb, &v, &IrType::I32, &PbType::Long)
                } else {
                    fb.const_i32(0)
                };
                let fc = if let Some(a5) = call.args.get(5) {
                    let v = self.compile_expr(fb, a5)?;
                    self.convert_value(fb, &v, &IrType::I32, &PbType::Long)
                } else {
                    fb.const_i32(0)
                };
                let fs = if let Some(a6) = call.args.get(6) {
                    let v = self.compile_expr(fb, a6)?;
                    self.convert_value(fb, &v, &IrType::I32, &PbType::Long)
                } else {
                    fb.const_i32(0)
                };
                fb.call_void(
                    f,
                    &[
                        ia[0].clone(),
                        ia[1].clone(),
                        ia[2].clone(),
                        ia[3].clone(),
                        color,
                        fc,
                        fs,
                    ],
                );
            }

            "FILECOPY" => {
                // FILECOPY src$, dst$ — copy a file (sets ERR on failure)
                // batch 219: the official syntax requires all 2 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 2 {
                    return Err(pb::error::PbError::parser(
                        "FILECOPY requires its arguments: FILECOPY sourcefile , destfile"
                            .to_string(),
                        None,
                        call.line,
                    ));
                }
                if call.args.len() >= 2 {
                    let src = self.compile_expr(fb, &call.args[0])?;
                    let dst = self.compile_expr(fb, &call.args[1])?;
                    fb.call_void("pb_filecopy", &[src, dst]);
                }
                return Ok(());
            }
            "MENU_GET_STATE" => {
                let h = self.compile_expr(fb, &call.args[0])?;
                let bycmd = self.compile_expr(fb, &call.args[1])?;
                let pos = self.compile_expr(fb, &call.args[2])?;
                if let Some((dst, _, _)) = self.lvalue_ptr(fb, &call.args[3]) {
                    let hc = self.convert_value(fb, &h, &IrType::I64, &PbType::Quad);
                    let bc = self.convert_value(fb, &bycmd, &IrType::I32, &PbType::Long);
                    let pc = self.convert_value(fb, &pos, &IrType::I32, &PbType::Long);
                    fb.call_void("pb_menu_get_state", &[hc, bc, pc, dst]);
                }
                return Ok(());
            }
            "MENU_SET_STATE" => {
                let h = self.compile_expr(fb, &call.args[0])?;
                let bycmd = self.compile_expr(fb, &call.args[1])?;
                let pos = self.compile_expr(fb, &call.args[2])?;
                let val = self.compile_expr(fb, &call.args[3])?;
                let hc = self.convert_value(fb, &h, &IrType::I64, &PbType::Quad);
                let bc = self.convert_value(fb, &bycmd, &IrType::I32, &PbType::Long);
                let pc = self.convert_value(fb, &pos, &IrType::I32, &PbType::Long);
                let vc = self.convert_value(fb, &val, &IrType::I32, &PbType::Long);
                fb.call_void("pb_menu_set_state", &[hc, bc, pc, vc]);
                return Ok(());
            }
            "MENU_GET_TEXT" => {
                let h = self.compile_expr(fb, &call.args[0])?;
                let bycmd = self.compile_expr(fb, &call.args[1])?;
                let pos = self.compile_expr(fb, &call.args[2])?;
                if let Some((dst, _, _)) = self.lvalue_ptr(fb, &call.args[3]) {
                    let hc = self.convert_value(fb, &h, &IrType::I64, &PbType::Quad);
                    let bc = self.convert_value(fb, &bycmd, &IrType::I32, &PbType::Long);
                    let pc = self.convert_value(fb, &pos, &IrType::I32, &PbType::Long);
                    fb.call_void("pb_menu_get_text", &[hc, bc, pc, dst]);
                }
                return Ok(());
            }
            "MENU_SET_TEXT" => {
                let h = self.compile_expr(fb, &call.args[0])?;
                let bycmd = self.compile_expr(fb, &call.args[1])?;
                let pos = self.compile_expr(fb, &call.args[2])?;
                let txt = self.compile_str_payload(fb, &call.args[3])?;
                let hc = self.convert_value(fb, &h, &IrType::I64, &PbType::Quad);
                let bc = self.convert_value(fb, &bycmd, &IrType::I32, &PbType::Long);
                let pc = self.convert_value(fb, &pos, &IrType::I32, &PbType::Long);
                fb.call_void("pb_menu_set_text", &[hc, bc, pc, txt]);
                return Ok(());
            }
            "SETATTR" => {
                // SETATTR "path", attr& — set file attributes
                // batch 219: the official syntax requires all 2 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 2 {
                    return Err(pb::error::PbError::parser(
                        "SETATTR requires its arguments: SETATTR filespec$ , attribute".to_string(),
                        None,
                        call.line,
                    ));
                }
                if call.args.len() >= 2 {
                    let path = self.compile_expr(fb, &call.args[0])?;
                    let attr = self.compile_expr(fb, &call.args[1])?;
                    let a = self.to_i32(fb, &attr);
                    fb.call_void("pb_setattr", &[path, a]);
                }
                return Ok(());
            }
            "MKDIR" => {
                // MKDIR path$ — on failure set ERR: EEXIST -> 75, ENOENT -> 76 (PB semantics)
                if !call.args.is_empty() {
                    let path = self.compile_expr(fb, &call.args[0])?;
                    let ret = fb.call(&IrType::I32, "_mkdir", &[path]);
                    let neg1 = fb.const_i32(-1);
                    let is_err = fb.icmp("eq", &ret, &neg1);
                    let err_lbl = fb.next_label("mkdir.err");
                    let merge_lbl = fb.next_label("mkdir.merge");
                    fb.condbr(&is_err, &err_lbl, &merge_lbl);
                    fb.label(&err_lbl);
                    let errno_ptr = fb.call(&IrType::Ptr, "_errno", &[]);
                    let eno = fb.load(&IrType::I32, &errno_ptr);
                    let eexist = fb.const_i32(17);
                    let is_eexist = fb.icmp("eq", &eno, &eexist);
                    let eexist_lbl = fb.next_label("mkdir.eexist");
                    let other_lbl = fb.next_label("mkdir.other");
                    fb.condbr(&is_eexist, &eexist_lbl, &other_lbl);
                    fb.label(&eexist_lbl);
                    let g = Val::new("@pb_err".to_string(), IrType::Ptr);
                    fb.store(&fb.const_i32(75), &g);
                    fb.br(&merge_lbl);
                    fb.label(&other_lbl);
                    let g2 = Val::new("@pb_err".to_string(), IrType::Ptr);
                    fb.store(&fb.const_i32(76), &g2);
                    fb.br(&merge_lbl);
                    fb.label(&merge_lbl);
                }
                return Ok(());
            }
            "RMDIR" => {
                // RMDIR path$ — on failure set ERR 75 (PB semantics)
                if !call.args.is_empty() {
                    let path = self.compile_expr(fb, &call.args[0])?;
                    let ret = fb.call(&IrType::I32, "_rmdir", &[path]);
                    let neg1 = fb.const_i32(-1);
                    let is_err = fb.icmp("eq", &ret, &neg1);
                    let err_lbl = fb.next_label("rmdir.err");
                    let merge_lbl = fb.next_label("rmdir.merge");
                    fb.condbr(&is_err, &err_lbl, &merge_lbl);
                    fb.label(&err_lbl);
                    let g = Val::new("@pb_err".to_string(), IrType::Ptr);
                    fb.store(&fb.const_i32(75), &g);
                    fb.br(&merge_lbl);
                    fb.label(&merge_lbl);
                }
                return Ok(());
            }
            "CHDIR" => {
                // CHDIR path$ — on failure set ERR 76 (PB semantics)
                if !call.args.is_empty() {
                    let path = self.compile_expr(fb, &call.args[0])?;
                    let ret = fb.call(&IrType::I32, "_chdir", &[path]);
                    let neg1 = fb.const_i32(-1);
                    let is_err = fb.icmp("eq", &ret, &neg1);
                    let err_lbl = fb.next_label("chdir.err");
                    let merge_lbl = fb.next_label("chdir.merge");
                    fb.condbr(&is_err, &err_lbl, &merge_lbl);
                    fb.label(&err_lbl);
                    let g = Val::new("@pb_err".to_string(), IrType::Ptr);
                    fb.store(&fb.const_i32(76), &g);
                    fb.br(&merge_lbl);
                    fb.label(&merge_lbl);
                }
                return Ok(());
            }
            "MSGBOX" => {
                // MSGBOX text$ [, style& [, title$]] -> MessageBoxA(NULL, text, title, style)
                // CHW official: omitted style = %MB_OK (0); omitted title =
                // "PowerBASIC"; empty comma slots (`, ,`) are Expr::Omitted
                // (batch 250, PBWin-accepted).
                if !call.args.is_empty() {
                    let text = self.compile_expr(fb, &call.args[0])?;
                    let style = if call.args.len() > 1 {
                        match &call.args[1] {
                            Expr::Omitted => fb.const_i32(0),
                            _ => {
                                let sv = self.compile_expr(fb, &call.args[1])?;
                                self.to_i32(fb, &sv)
                            }
                        }
                    } else {
                        fb.const_i32(0)
                    };
                    let title = if call.args.len() > 2 {
                        match &call.args[2] {
                            Expr::Omitted => Val::new(self.msgbox_title_name.clone(), IrType::Ptr),
                            _ => self.compile_expr(fb, &call.args[2])?,
                        }
                    } else {
                        Val::new(self.msgbox_title_name.clone(), IrType::Ptr)
                    };
                    if self.module.is_32bit() {
                        fb.call_stdcall(
                            &IrType::I32,
                            "pb_msgbox_impl",
                            &[text.clone(), title.clone(), style.clone()],
                        );
                    } else {
                        fb.call(&IrType::I32, "pb_msgbox_impl", &[text, title, style]);
                    }
                }
                return Ok(());
            }
            "SHELL" => {
                // SHELL command$ [, mode&] -> ShellExecuteA(NULL, "open", command, NULL, NULL, mode)
                if !call.args.is_empty() {
                    let cmd = self.compile_expr(fb, &call.args[0])?;
                    let mode = if call.args.len() > 1 {
                        let sv = self.compile_expr(fb, &call.args[1])?;
                        self.to_i32(fb, &sv)
                    } else {
                        fb.const_i32(1) // SW_SHOWNORMAL
                    };
                    let null = Val::new("null".to_string(), IrType::Ptr);
                    let (open_name, _) = self.module.add_string_constant("open");
                    let open = Val::new(open_name, IrType::Ptr);
                    if self.module.is_32bit() {
                        fb.call_stdcall(
                            &IrType::Ptr,
                            "ShellExecuteA",
                            &[
                                null.clone(),
                                open.clone(),
                                cmd.clone(),
                                null.clone(),
                                null.clone(),
                                mode.clone(),
                            ],
                        );
                    } else {
                        fb.call(
                            &IrType::Ptr,
                            "ShellExecuteA",
                            &[
                                null.clone(),
                                open.clone(),
                                cmd.clone(),
                                null.clone(),
                                null.clone(),
                                mode,
                            ],
                        );
                    }
                }
                return Ok(());
            }
            "REPLACE" => {
                // REPLACE old$ WITH new$ IN target$  → pb_replace(&target, old, new)
                if call.args.len() >= 3 {
                    let old = self.compile_expr(fb, &call.args[1])?;
                    let new_s = self.compile_expr(fb, &call.args[2])?;
                    let (ptr, _pb_type) = self.compile_lvalue_ptr(fb, &call.args[0])?;
                    fb.call_void("pb_replace", &[ptr, old, new_s]);
                }
                return Ok(());
            }
            "ERRCLEAR" => {
                // ERRCLEAR — reset the ERR system variable to 0
                let g = Val::new("@pb_err".to_string(), IrType::Ptr);
                fb.store(&fb.const_i32(0), &g);
                return Ok(());
            }
            "BEEP" => {
                // BEEP -> Beep(800, 300) — default beep tone
                let freq = fb.const_i32(800);
                let dur = fb.const_i32(300);
                fb.call_void("Beep", &[freq, dur]);
                return Ok(());
            }
            "BIT SET" | "BIT RESET" | "BIT TOGGLE" | "BIT CALC" => {
                // BIT {SET|RESET|TOGGLE} intvar, bitnumber / BIT CALC intvar, bitnumber, expr
                if call.args.len() < 2 {
                    return Ok(());
                }
                if let Some((ptr, ir_ty, pb_ty)) = self.lvalue_ptr(fb, &call.args[0]) {
                    let cur = fb.load(&ir_ty, &ptr);
                    let vi = self.to_i64(fb, &cur);
                    let bitv = self.compile_expr(fb, &call.args[1])?;
                    let bi = self.to_i64(fb, &bitv);
                    let one = fb.const_i64(1);
                    let mask = fb.shl(&one, &bi);
                    let newv = match name.as_str() {
                        "BIT SET" => fb.or(&vi, &mask),
                        "BIT RESET" => {
                            let all = fb.const_i64(-1);
                            let not_mask = fb.xor(&mask, &all);
                            fb.and(&vi, &not_mask)
                        }
                        "BIT TOGGLE" => fb.xor(&vi, &mask),
                        _ => {
                            // BIT CALC: expr != 0 -> set, else reset
                            if let Some(expr) = call.args.get(2) {
                                let ev = self.compile_expr(fb, expr)?;
                                let ei = self.to_i64(fb, &ev);
                                let z = fb.const_i64(0);
                                let is_nonzero = fb.icmp("ne", &ei, &z);
                                let m = fb.select(&is_nonzero, &mask, &z);
                                fb.or(&vi, &m)
                            } else {
                                return Ok(());
                            }
                        }
                    };
                    let conv = self.convert_value(fb, &newv, &ir_ty, &pb_ty);
                    fb.store(&conv, &ptr);
                }
                return Ok(());
            }
            "PROCESS GET PRIORITY" | "PROCESS SET PRIORITY" => {
                if name == "PROCESS GET PRIORITY" {
                    if let Some(expr) = call.args.first() {
                        if let Some((ptr, ir_ty, pb_ty)) = self.lvalue_ptr(fb, expr) {
                            let v = fb.call(&IrType::I32, "pb_process_get_priority", &[]);
                            let conv = self.convert_value(fb, &v, &ir_ty, &pb_ty);
                            fb.store(&conv, &ptr);
                        }
                    }
                } else if let Some(expr) = call.args.first() {
                    let v = self.compile_expr(fb, expr)?;
                    let vi = self.to_i32(fb, &v);
                    fb.call_void("pb_process_set_priority", &[vi]);
                }
                return Ok(());
            }
            "POKE" => {
                // POKE [DataType,] Address, Value [, Value...] — default BYTE
                let (dt, rest) = match call.args.first() {
                    Some(Expr::Variable(n))
                        if matches!(
                            normalize_name(n).as_str(),
                            "BYTE"
                                | "WORD"
                                | "DWORD"
                                | "INTEGER"
                                | "LONG"
                                | "QUAD"
                                | "SINGLE"
                                | "DOUBLE"
                        ) =>
                    {
                        (normalize_name(n), &call.args[1..])
                    }
                    _ => ("BYTE".to_string(), &call.args[..]),
                };
                if let Some(addr_expr) = rest.first() {
                    let av = self.compile_expr(fb, addr_expr)?;
                    let av64 = if av.ty == IrType::I32 {
                        fb.sext(&av, &IrType::I64)
                    } else {
                        av
                    };
                    let base_addr = fb.inttoptr(&av64);
                    let size = match dt.as_str() {
                        "BYTE" | "INTEGER" | "WORD" => 1,
                        "DWORD" | "LONG" | "SINGLE" => 4,
                        "QUAD" | "DOUBLE" => 8,
                        _ => 1,
                    };
                    for (i, val_expr) in rest.iter().skip(1).enumerate() {
                        let v = self.compile_expr(fb, val_expr)?;
                        let addr = if i == 0 {
                            base_addr.clone()
                        } else {
                            let off = fb.const_i32((i * size) as i32);
                            fb.gep_byte(&base_addr, &off)
                        };
                        match dt.as_str() {
                            "BYTE" => {
                                let iv = self.to_i32(fb, &v);
                                fb.call_void("pb_poke8", &[addr, iv])
                            }
                            "INTEGER" | "WORD" => {
                                let iv = self.to_i32(fb, &v);
                                fb.call_void("pb_poke16", &[addr, iv])
                            }
                            "DWORD" | "LONG" => {
                                let iv = self.to_i32(fb, &v);
                                fb.call_void("pb_poke32", &[addr, iv])
                            }
                            "QUAD" => {
                                let qv = self.to_i64(fb, &v);
                                fb.call_void("pb_poke64", &[addr, qv])
                            }
                            "SINGLE" => {
                                let f = fb.sitofp(&v, &IrType::Float);
                                fb.call_void("pb_pokef", &[addr, f])
                            }
                            "DOUBLE" => {
                                let dv = self.to_f64(fb, &v);
                                fb.call_void("pb_poked", &[addr, dv])
                            }
                            _ => {
                                let iv = self.to_i32(fb, &v);
                                fb.call_void("pb_poke8", &[addr, iv])
                            }
                        };
                    }
                }
                return Ok(());
            }
            "READ" => {
                // READ var1, var2, ... — read next DATA items
                for arg in &call.args {
                    if let Some((ptr, ty, pb_ty)) = self.lvalue_ptr(fb, arg) {
                        let is_num = !matches!(
                            pb_ty,
                            PbType::String | PbType::FixedString(_) | PbType::Variant
                        );
                        if is_num {
                            let v = fb.call(&IrType::Double, "pb_read_data_num", &[]);
                            let conv = self.convert_value(fb, &v, &ty, &pb_ty);
                            fb.store(&conv, &ptr);
                        } else {
                            let v = fb.call(&IrType::Ptr, "pb_read_data_str", &[]);
                            fb.store(&v, &ptr);
                        }
                    }
                }
                return Ok(());
            }
            "ERASE" => {
                // ERASE array - zero / null all elements
                if let Some(Expr::Variable(vname)) = call.args.first() {
                    let arr_name = normalize_name(vname);
                    if let Some(arr_info) = self.symbols.lookup_array(&arr_name).cloned() {
                        let base = Val::new(arr_info.ptr_name.clone(), IrType::Ptr);
                        let elem_size = match &arr_info.elem_ir_type {
                            IrType::I8 | IrType::I1 => 1,
                            IrType::I16 => 2,
                            IrType::I32 | IrType::Float => 4,
                            IrType::I64 | IrType::Double | IrType::Ptr => 8,
                            _ => 4,
                        };
                        let is_string = if arr_info.elem_ir_type == IrType::Ptr {
                            1
                        } else {
                            0
                        };
                        fb.call_void(
                            "pb_erase_array",
                            &[
                                base,
                                fb.const_i32(elem_size),
                                fb.const_i32(arr_info.total_elements as i32),
                                fb.const_i32(is_string),
                            ],
                        );
                    }
                }
                return Ok(());
            }
            "GET" | "PUT" => {
                // RANDOM-mode record access:
                //   GET #f          → read current record into field buffer
                //   PUT #f          → write field buffer as current record
                //   GET #f, n / PUT #f, n  → same at record number n (1-based)
                if call.args.len() == 1 {
                    let f0 = self.compile_expr(fb, &call.args[0])?;
                    let filenum = self.to_i32(fb, &f0);
                    fb.call_void(
                        if call.name == "GET" {
                            "pb_get_record"
                        } else {
                            "pb_put_record"
                        },
                        &[filenum],
                    );
                    return Ok(());
                }
                if call.args.len() == 2 {
                    if let Expr::IntegerLit(n) = call.args[1] {
                        let f0 = self.compile_expr(fb, &call.args[0])?;
                        let filenum = self.to_i32(fb, &f0);
                        let rec = fb.const_i64(n);
                        fb.call_void("pb_seek_record", &[filenum.clone(), rec]);
                        fb.call_void(
                            if call.name == "GET" {
                                "pb_get_record"
                            } else {
                                "pb_put_record"
                            },
                            &[filenum],
                        );
                        return Ok(());
                    }
                }
                // GET #f [, pos], var  /  PUT #f [, pos], var
                // args = [filenum, (pos), var]
                if call.args.len() >= 2 {
                    let f0 = self.compile_expr(fb, &call.args[0])?;
                    let filenum = self.to_i32(fb, &f0);
                    // pos: 3 args -> args[1] is position; 2 args -> current position (-1)
                    let pos_val = if call.args.len() >= 3 {
                        let p = self.compile_expr(fb, &call.args[1])?;
                        self.to_i64(fb, &p)
                    } else {
                        fb.const_i64(-1)
                    };
                    let var_expr = &call.args[call.args.len() - 1];
                    let (ptr, _ir_ty, pb_type) = match self.lvalue_ptr(fb, var_expr) {
                        Some(lv) => lv,
                        None => {
                            return Err(PbError::parser(
                                format!(
                                    "{} # record variable must be a variable (use {} # for string literals/expressions)",
                                    call.name,
                                    if call.name == "GET" { "GET$" } else { "PUT$" }
                                ),
                                None,
                                call.line,
                            ))
                        }
                    };
                    let size = match &pb_type {
                        PbType::Byte => 1,
                        PbType::Word | PbType::Integer => 2,
                        PbType::Long | PbType::Dword => 4,
                        PbType::Single => 4,
                        PbType::Quad | PbType::Double | PbType::Ext | PbType::Cur => 8,
                        PbType::FixedString(n) => *n as i64,
                        _ => 0, // dynamic STRING / Variant / UDT: raw size not fixed
                    };
                    if size > 0 {
                        fb.call_void(
                            if call.name == "GET" {
                                "pb_get"
                            } else {
                                "pb_put"
                            },
                            &[filenum, pos_val, ptr, fb.const_i64(size)],
                        );
                    } else {
                        // batch 220: a dynamic STRING / Variant / UDT record target used to
                        // compile into a silent no-op (size 0 skipped the call). Report it.
                        return Err(PbError::parser(
                            format!(
                                "{} # with a variable-size variable (dynamic STRING/Variant/UDT) is not supported; use {} # filenum& , ... for string I/O",
                                call.name,
                                if call.name == "GET" { "GET$" } else { "PUT$" }
                            ),
                            None,
                            call.line,
                        ));
                    }
                }
                return Ok(());
            }
            "ARRAY COPY" | "ARRAY SWAP" | "ARRAY ADD" => {
                // ARRAY COPY src(), dest()  /  ARRAY SWAP a(), b()
                if let (Some(Expr::FunctionCall(sn, _)), Some(Expr::FunctionCall(dn, _))) =
                    (call.args.first(), call.args.get(1))
                {
                    let sname = normalize_name(sn);
                    let dname = normalize_name(dn);
                    let s_info = self.symbols.lookup_array(&sname).cloned();
                    let d_info = self.symbols.lookup_array(&dname).cloned();
                    if let (Some(si), Some(di)) = (s_info, d_info) {
                        let sbase = Val::new(si.ptr_name.clone(), IrType::Ptr);
                        let dbase = Val::new(di.ptr_name.clone(), IrType::Ptr);
                        let elem_size = match &si.elem_ir_type {
                            IrType::I8 | IrType::I1 => 1,
                            IrType::I16 => 2,
                            IrType::I32 | IrType::Float => 4,
                            IrType::I64 | IrType::Double | IrType::Ptr => 8,
                            _ => 4,
                        };
                        let total = si.total_elements.min(di.total_elements) as i64;
                        let fn_name = if call.name == "ARRAY SWAP" {
                            "pb_array_swap"
                        } else if call.name == "ARRAY ADD" {
                            "pb_array_add"
                        } else {
                            "pb_array_copy"
                        };
                        if call.name == "ARRAY ADD" {
                            let is_fl = matches!(&si.elem_ir_type, IrType::Float | IrType::Double);
                            fb.call_void(
                                fn_name,
                                &[
                                    sbase,
                                    dbase,
                                    fb.const_i32(elem_size),
                                    fb.const_i32(if is_fl { 1 } else { 0 }),
                                    fb.const_i64(total),
                                ],
                            );
                        } else {
                            fb.call_void(
                                fn_name,
                                &[dbase, sbase, fb.const_i32(elem_size), fb.const_i64(total)],
                            );
                        }
                    }
                }
                return Ok(());
            }
            "ARRAY ASSIGN" => {
                // ARRAY ASSIGN target() = source() 鈥?copy source elements into target
                if let (Some(Expr::FunctionCall(tn, _)), Some(Expr::FunctionCall(sn, _))) =
                    (call.args.first(), call.args.get(1))
                {
                    let tname = normalize_name(tn);
                    let sname = normalize_name(sn);
                    let t_info = self.symbols.lookup_array(&tname).cloned();
                    let s_info = self.symbols.lookup_array(&sname).cloned();
                    if let (Some(ti), Some(si)) = (t_info, s_info) {
                        let tbase = Val::new(ti.ptr_name.clone(), IrType::Ptr);
                        let sbase = Val::new(si.ptr_name.clone(), IrType::Ptr);
                        let elem_size = match &ti.elem_ir_type {
                            IrType::I8 | IrType::I1 => 1,
                            IrType::I16 => 2,
                            IrType::I32 | IrType::Float => 4,
                            IrType::I64 | IrType::Double | IrType::Ptr => 8,
                            _ => 4,
                        };
                        let total = ti.total_elements.min(si.total_elements) as i64;
                        fb.call_void(
                            "pb_array_copy",
                            &[tbase, sbase, fb.const_i32(elem_size), fb.const_i64(total)],
                        );
                    }
                }
                return Ok(());
            }
            "WINDOW SET TEXT" => {
                // WINDOW SET TEXT hwnd, text$ 鈥?console title (hwnd ignored)
                if let Some(text) = call.args.get(1) {
                    let tv = self.compile_expr(fb, text)?;
                    fb.call_void("pb_console_set_title", std::slice::from_ref(&tv));
                }
                return Ok(());
            }
            "WINDOW GET TEXT" => {
                // WINDOW GET TEXT hwnd TO s$ 鈥?console title (hwnd ignored)
                if let Some(target) = call.args.get(1) {
                    if let Some((ptr, _, _)) = self.lvalue_ptr(fb, target) {
                        let title = fb.call(&IrType::Ptr, "pb_console_get_title", &[]);
                        fb.store(&title, &ptr);
                    }
                }
                return Ok(());
            }
            "WINDOW_NEW" => {
                // WINDOW title$, x, y, w, h TO hWnd&
                if let Some(title) = call.args.first() {
                    let tv = self.compile_expr(fb, title)?;
                    let xv = self.compile_expr(fb, &call.args[1])?;
                    let yv = self.compile_expr(fb, &call.args[2])?;
                    let wv = self.compile_expr(fb, &call.args[3])?;
                    let hv = self.compile_expr(fb, &call.args[4])?;
                    let hwnd = fb.call(&IrType::Ptr, "pb_window_new", &[tv, xv, yv, wv, hv]);
                    if let Some(target) = call.args.get(5) {
                        if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, target) {
                            let cv = self.convert_value(fb, &hwnd, &ty, &pty);
                            fb.store(&cv, &ptr);
                        }
                    }
                }
                return Ok(());
            }
            "CONTROL_ADD_BUTTON" => {
                // batch 220: the official syntax requires all 8 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 8 {
                    return Err(pb::error::PbError::parser(
                        "CONTROL_ADD_BUTTON requires its arguments: CONTROL ADD BUTTON hDlg , id& , txt$ , x , y , xx , yy TO hCtrl&".to_string(),
                        None,
                        call.line,
                    ));
                }

                // CONTROL ADD BUTTON, hWnd, id, text$, x, y, w, h
                //   [, [style&] [, [exstyle&]]] [[,] CALL callback] [, TO hCtrl&]
                // batch 243: full official syntax - style/exstyle passthrough + CALL
                if call.args.len() >= 8 {
                    let mut hwnd = self.compile_expr(fb, &call.args[0])?;
                    if hwnd.ty != IrType::Ptr {
                        hwnd = fb.inttoptr(&hwnd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let text = self.compile_expr(fb, &call.args[2])?;
                    let x = self.compile_expr(fb, &call.args[3])?;
                    let y = self.compile_expr(fb, &call.args[4])?;
                    let w = self.compile_expr(fb, &call.args[5])?;
                    let h = self.compile_expr(fb, &call.args[6])?;
                    let target = if call.args.len() == 8 {
                        // legacy IR: args[7] was the TO target, no style/callback
                        Some(call.args[7].clone())
                    } else {
                        call.args.get(10).cloned()
                    };
                    let hctrl = if call.args.len() == 8 {
                        fb.call(
                            &IrType::Ptr,
                            "pb_control_add_button",
                            &[hwnd, id, text, x, y, w, h],
                        )
                    } else {
                        let style = self.compile_expr(fb, &call.args[7])?;
                        let exstyle = self.compile_expr(fb, &call.args[8])?;
                        let cb = self.compile_control_callback(fb, &call.args[9])?;
                        fb.call(
                            &IrType::Ptr,
                            "pb_control_add_button_ex",
                            &[hwnd, id, text, x, y, w, h, style, exstyle, cb],
                        )
                    };
                    if let Some(tgt) = target {
                        if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &tgt) {
                            let cv = self.convert_value(fb, &hctrl, &ty, &pty);
                            fb.store(&cv, &ptr);
                        }
                    }
                }
                return Ok(());
            }
            "PB_MESSAGE_LOOP" => {
                fb.call_void("pb_message_loop", &[]);
                return Ok(());
            }
            "CONTROL_ADD_EDITBOX" => {
                // batch 220: the official syntax requires all 8 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 8 {
                    return Err(pb::error::PbError::parser(
                        "CONTROL_ADD_EDITBOX requires its arguments: CONTROL ADD EDITBOX hDlg , id& , txt$ , x , y , xx , yy TO hCtrl&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 8 {
                    let mut hwnd = self.compile_expr(fb, &call.args[0])?;
                    if hwnd.ty != IrType::Ptr {
                        hwnd = fb.inttoptr(&hwnd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let text = self.compile_expr(fb, &call.args[2])?;
                    let x = self.compile_expr(fb, &call.args[3])?;
                    let y = self.compile_expr(fb, &call.args[4])?;
                    let w = self.compile_expr(fb, &call.args[5])?;
                    let h = self.compile_expr(fb, &call.args[6])?;
                    // batch 249: full official syntax (mirrors BUTTON)
                    let target = if call.args.len() == 8 {
                        Some(call.args[7].clone())
                    } else {
                        call.args.get(10).cloned()
                    };
                    let hctrl = if call.args.len() == 8 {
                        fb.call(
                            &IrType::Ptr,
                            "pb_control_add_editbox",
                            &[hwnd, id, text, x, y, w, h],
                        )
                    } else {
                        let style = self.compile_expr(fb, &call.args[7])?;
                        let exstyle = self.compile_expr(fb, &call.args[8])?;
                        let cb = self.compile_control_callback(fb, &call.args[9])?;
                        fb.call(
                            &IrType::Ptr,
                            "pb_control_add_editbox_ex",
                            &[hwnd, id, text, x, y, w, h, style, exstyle, cb],
                        )
                    };
                    if let Some(tgt) = target {
                        if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &tgt) {
                            let cv = self.convert_value(fb, &hctrl, &ty, &pty);
                            fb.store(&cv, &ptr);
                        }
                    }
                }
                return Ok(());
            }
            "CONTROL_ADD_COMBOBOX" => {
                // batch 248: full official syntax, fixed 11-arg layout
                // [hwnd, id, items, x, y, w, h, style, exstyle, callback, target]
                // (CONTROL ADD COMBOBOX, hDlg, id&, [items$()], x, y, xx, yy
                //   [, [style&] [, [exstyle&]]] [[,] CALL callback] [, TO hCtrl&])
                if call.args.len() < 11 {
                    return Err(pb::error::PbError::parser(
                        "CONTROL_ADD_COMBOBOX requires its arguments: CONTROL ADD COMBOBOX hDlg , id& , items$() , x , y , xx , yy [, [style&] [, [exstyle&]]] [[,] CALL callback] [, TO hCtrl&]".to_string(),
                        None,
                        call.line,
                    ));
                }

                let mut hwnd = self.compile_expr(fb, &call.args[0])?;
                if hwnd.ty != IrType::Ptr {
                    hwnd = fb.inttoptr(&hwnd);
                }
                let id = self.compile_expr(fb, &call.args[1])?;
                // items slot: full-array reference sArray() → pass base pointer +
                // element count so the runtime can CB_ADDSTRING each item.
                let (items, item_count) =
                    if let Some(Expr::FunctionCall(arr_name, _)) = call.args.get(2) {
                        let an = normalize_name(arr_name);
                        if let Some(arr_info) = self.symbols.lookup_array(&an).cloned() {
                            (
                                Val::new(arr_info.ptr_name.clone(), IrType::Ptr),
                                arr_info.total_elements as i64,
                            )
                        } else {
                            let v = self.compile_expr(fb, &call.args[2])?;
                            (v, 0)
                        }
                    } else {
                        let v = self.compile_expr(fb, &call.args[2])?;
                        (v, 0)
                    };
                let x = self.compile_expr(fb, &call.args[3])?;
                let y = self.compile_expr(fb, &call.args[4])?;
                let w = self.compile_expr(fb, &call.args[5])?;
                let h = self.compile_expr(fb, &call.args[6])?;
                let style = self.compile_expr(fb, &call.args[7])?;
                let exstyle = self.compile_expr(fb, &call.args[8])?;
                let cb = self.compile_control_callback(fb, &call.args[9])?;
                let hc = fb.call(
                    &IrType::Ptr,
                    "pb_control_add_combobox_ex",
                    &[
                        hwnd,
                        id,
                        items,
                        fb.const_i64(item_count),
                        x,
                        y,
                        w,
                        h,
                        style,
                        exstyle,
                        cb,
                    ],
                );
                if let Some(t) = call.args.get(10) {
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, t) {
                        let cv = self.convert_value(fb, &hc, &ty, &pty);
                        fb.store(&cv, &ptr);
                    }
                }
                return Ok(());
            }
            "CONTROL_GET_TEXT_ID" => {
                // batch 220: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                        "CONTROL_GET_TEXT_ID requires its arguments: CONTROL GET TEXT ID hDlg , id& TO txtv$".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 3 {
                    let hdlg = self.compile_expr(fb, &call.args[0])?;
                    let cid = self.compile_expr(fb, &call.args[1])?;
                    let hdlg64 = match hdlg.ty {
                        IrType::Ptr => hdlg,
                        _ => fb.inttoptr(&hdlg),
                    };
                    let buf = fb.alloca(&IrType::Array(256, Box::new(IrType::I8)));
                    let bp = fb.gep_byte(&buf, &fb.const_i32(0));
                    fb.call_void(
                        "pb_control_get_text_by_id",
                        &[hdlg64, cid, bp.clone(), fb.const_i32(256)],
                    );
                    if let Some((ptr, _, _)) = self.lvalue_ptr(fb, &call.args[2]) {
                        let len =
                            fb.call(&IrType::I32, "pb_str_cstr_len", std::slice::from_ref(&bp));
                        let bstr = fb.call(&IrType::Ptr, "pb_bstr_alloc", &[bp, len]);
                        fb.store(&bstr, &ptr);
                    }
                }
                return Ok(());
            }
            "CONTROL_GET_TEXT" => {
                if let (Some(hc), Some(tgt)) = (call.args.first(), call.args.get(1)) {
                    let hcv = self.compile_expr(fb, hc)?;
                    let buf = fb.alloca(&IrType::Array(256, Box::new(IrType::I8)));
                    let bp = fb.gep_byte(&buf, &fb.const_i32(0));
                    fb.call_void("pb_control_get_text", &[hcv, bp.clone(), fb.const_i32(256)]);
                    if let Some((ptr, _, _)) = self.lvalue_ptr(fb, tgt) {
                        let len =
                            fb.call(&IrType::I32, "pb_str_cstr_len", std::slice::from_ref(&bp));
                        let bstr = fb.call(&IrType::Ptr, "pb_bstr_alloc", &[bp, len]);
                        fb.store(&bstr, &ptr);
                    }
                }
                return Ok(());
            }
            "CONTROL_SET_TEXT" => {
                if let (Some(hc), Some(tx)) = (call.args.first(), call.args.get(1)) {
                    let hcv = self.compile_expr(fb, hc)?;
                    let cstr = self.compile_str_payload(fb, tx)?;
                    fb.call_void("pb_control_set_text", &[hcv, cstr]);
                }
                return Ok(());
            }
            "CONTROL_SHOW" => {
                if let Some(hc) = call.args.first() {
                    let v = self.compile_expr(fb, hc)?;
                    fb.call_void("pb_control_show", &[v]);
                }
                return Ok(());
            }
            "CONTROL_HIDE" => {
                if let Some(hc) = call.args.first() {
                    let v = self.compile_expr(fb, hc)?;
                    fb.call_void("pb_control_hide", &[v]);
                }
                return Ok(());
            }
            "CONTROL_ENABLE" => {
                if let Some(hc) = call.args.first() {
                    let v = self.compile_expr(fb, hc)?;
                    fb.call_void("pb_control_enable", &[v]);
                }
                return Ok(());
            }
            "CONTROL_DISABLE" => {
                if let Some(hc) = call.args.first() {
                    let v = self.compile_expr(fb, hc)?;
                    fb.call_void("pb_control_disable", &[v]);
                }
                return Ok(());
            }
            "CONTROL_FOCUS" => {
                if let Some(hc) = call.args.first() {
                    let v = self.compile_expr(fb, hc)?;
                    fb.call_void("pb_control_focus", &[v]);
                }
                return Ok(());
            }
            "CONTROL_CHECK" => {
                if let Some(hc) = call.args.first() {
                    let v = self.compile_expr(fb, hc)?;
                    fb.call_void("pb_control_check", &[v]);
                }
                return Ok(());
            }
            "CONTROL_UNCHECK" => {
                if let Some(hc) = call.args.first() {
                    let v = self.compile_expr(fb, hc)?;
                    fb.call_void("pb_control_uncheck", &[v]);
                }
                return Ok(());
            }
            // --- PROGRESSBAR / HEADER (batch 159, official hDlg+id syntax) ---
            "PROGRESSBAR_SET_RANGE" => {
                // batch 219: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                    "PROGRESSBAR_SET_RANGE requires its arguments: PROGRESSBAR SET RANGE hDlg , id& , lolimit& ,"
                        .to_string(),
                    None,
                    call.line,
                ));
                }
                if call.args.len() >= 4 {
                    let a = self.compile_expr(fb, &call.args[0])?;
                    let b = self.compile_expr(fb, &call.args[1])?;
                    let c = self.compile_expr(fb, &call.args[2])?;
                    let d = self.compile_expr(fb, &call.args[3])?;
                    fb.call(&IrType::I64, "pb_progressbar_set_range", &[a, b, c, d]);
                }
                return Ok(());
            }
            "PROGRESSBAR_SET_POS" => {
                // batch 219: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                    "PROGRESSBAR_SET_POS requires its arguments: PROGRESSBAR SET POS hDlg , id& , position&"
                        .to_string(),
                    None,
                    call.line,
                ));
                }
                if call.args.len() >= 3 {
                    let a = self.compile_expr(fb, &call.args[0])?;
                    let b = self.compile_expr(fb, &call.args[1])?;
                    let c = self.compile_expr(fb, &call.args[2])?;
                    fb.call(&IrType::I64, "pb_progressbar_set_pos", &[a, b, c]);
                }
                return Ok(());
            }
            "PROGRESSBAR_SET_STEP" => {
                // batch 219: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                    "PROGRESSBAR_SET_STEP requires its arguments: PROGRESSBAR SET STEP hDlg , id& , stepval&"
                        .to_string(),
                    None,
                    call.line,
                ));
                }
                if call.args.len() >= 3 {
                    let a = self.compile_expr(fb, &call.args[0])?;
                    let b = self.compile_expr(fb, &call.args[1])?;
                    let c = self.compile_expr(fb, &call.args[2])?;
                    fb.call(&IrType::I64, "pb_progressbar_set_step", &[a, b, c]);
                }
                return Ok(());
            }
            "PROGRESSBAR_STEP" => {
                // batch 220: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                        "PROGRESSBAR_STEP requires its arguments: PROGRESSBAR STEP hDlg , id& [, incramt& ]".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 3 {
                    let a = self.compile_expr(fb, &call.args[0])?;
                    let b = self.compile_expr(fb, &call.args[1])?;
                    let inc = self.compile_expr(fb, &call.args[2])?;
                    fb.call(&IrType::I64, "pb_progressbar_step", &[a, b, inc]);
                }
                return Ok(());
            }
            "PROGRESSBAR_GET_POS" => {
                // batch 220: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                        "PROGRESSBAR_GET_POS requires its arguments: PROGRESSBAR GET POS hDlg , id& TO datav&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 3 {
                    let a = self.compile_expr(fb, &call.args[0])?;
                    let b = self.compile_expr(fb, &call.args[1])?;
                    let val = fb.call(&IrType::I64, "pb_progressbar_get_pos", &[a, b]);
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[2]) {
                        let vv = self.convert_value(fb, &val, &ty, &pty);
                        fb.store(&vv, &ptr);
                    }
                }
                return Ok(());
            }
            "PROGRESSBAR_GET_RANGE" => {
                // batch 220: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                        "PROGRESSBAR_GET_RANGE requires its arguments: PROGRESSBAR GET RANGE hDlg , id& TO LoDatav& , HiDatav&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 4 {
                    let a = self.compile_expr(fb, &call.args[0])?;
                    let b = self.compile_expr(fb, &call.args[1])?;
                    let lo = fb.call(&IrType::I64, "pb_progressbar_get_lo", &[a, b]);
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[2]) {
                        let vv = self.convert_value(fb, &lo, &ty, &pty);
                        fb.store(&vv, &ptr);
                    }
                    // fb.call consumes its operands, so re-evaluate the two
                    // simple handle/id operands for the second query.
                    let a2 = self.compile_expr(fb, &call.args[0])?;
                    let b2 = self.compile_expr(fb, &call.args[1])?;
                    let hi = fb.call(&IrType::I64, "pb_progressbar_get_hi", &[a2, b2]);
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[3]) {
                        let vv = self.convert_value(fb, &hi, &ty, &pty);
                        fb.store(&vv, &ptr);
                    }
                }
                return Ok(());
            }
            "HEADER_GET_COUNT" => {
                // batch 220: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                        "HEADER_GET_COUNT requires its arguments: HEADER GET COUNT hDlg , id& TO datav&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 3 {
                    let a = self.compile_expr(fb, &call.args[0])?;
                    let b = self.compile_expr(fb, &call.args[1])?;
                    let val = fb.call(&IrType::I64, "pb_header_get_count", &[a, b]);
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[2]) {
                        let vv = self.convert_value(fb, &val, &ty, &pty);
                        fb.store(&vv, &ptr);
                    }
                }
                return Ok(());
            }
            "HEADER_SEND" => {
                // batch 220: the official syntax requires all 5 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 5 {
                    return Err(pb::error::PbError::parser(
                        "HEADER_SEND requires its arguments: HEADER SEND hWin , ID& , Msg& , wParam& , lParam& [ TO ResultVar& ]".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 5 {
                    let a = self.compile_expr(fb, &call.args[0])?;
                    let b = self.compile_expr(fb, &call.args[1])?;
                    let c = self.compile_expr(fb, &call.args[2])?;
                    let d = self.compile_expr(fb, &call.args[3])?;
                    let e = self.compile_expr(fb, &call.args[4])?;
                    let val = fb.call(&IrType::I64, "pb_header_send", &[a, b, c, d, e]);
                    if call.args.len() >= 6 {
                        if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[5]) {
                            let vv = self.convert_value(fb, &val, &ty, &pty);
                            fb.store(&vv, &ptr);
                        }
                    }
                }
                return Ok(());
            }
            "HEADER_GET_ITEM" => {
                // batch 220: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                        "HEADER_GET_ITEM requires its arguments: HEADER GET ITEM hDlg , id& , item& TO datav&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 4 {
                    let a = self.compile_expr(fb, &call.args[0])?;
                    let b = self.compile_expr(fb, &call.args[1])?;
                    let c = self.compile_expr(fb, &call.args[2])?;
                    let d = self.compile_expr(fb, &call.args[3])?;
                    let val = fb.call(&IrType::I64, "pb_header_get_item", &[a, b, c, d]);
                    if call.args.len() >= 5 {
                        if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[4]) {
                            let vv = self.convert_value(fb, &val, &ty, &pty);
                            fb.store(&vv, &ptr);
                        }
                    }
                }
                return Ok(());
            }
            "HEADER_SET_ITEM" => {
                // batch 220: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                        "HEADER_SET_ITEM requires its arguments: HEADER SET ITEM hDlg , id& , item& , value&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 4 {
                    let a = self.compile_expr(fb, &call.args[0])?;
                    let b = self.compile_expr(fb, &call.args[1])?;
                    let c = self.compile_expr(fb, &call.args[2])?;
                    let d = self.compile_expr(fb, &call.args[3])?;
                    let val = fb.call(&IrType::I64, "pb_header_set_item", &[a, b, c, d]);
                    if call.args.len() >= 5 {
                        if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[4]) {
                            let vv = self.convert_value(fb, &val, &ty, &pty);
                            fb.store(&vv, &ptr);
                        }
                    }
                }
                return Ok(());
            }
            // ---------------------------------------------------------
            // TAB family (batch 169) -- official TAB_statement.htm
            // ---------------------------------------------------------
            "TAB_DELETE" => {
                // batch 220: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                        "TAB_DELETE requires its arguments: TAB DELETE hDlg , ID& , PageNum&"
                            .to_string(),
                        None,
                        call.line,
                    ));
                }

                // TAB DELETE hDlg, ID&, PageNum&
                if call.args.len() >= 3 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let page = self.compile_expr(fb, &call.args[2])?;
                    let _ = fb.call(&IrType::I64, "pb_tab_delete", &[hd, id, page]);
                }
                return Ok(());
            }
            "TAB_GET_COUNT" => {
                // batch 220: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                        "TAB_GET_COUNT requires its arguments: TAB GET COUNT hDlg , ID& TO CountVar&".to_string(),
                        None,
                        call.line,
                    ));
                }

                // TAB GET COUNT hDlg, ID& TO CountVar&
                if call.args.len() >= 3 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let v = fb.call(&IrType::I64, "pb_tab_get_count", &[hd, id]);
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[2]) {
                        let vv = self.convert_value(fb, &v, &ty, &pty);
                        fb.store(&vv, &ptr);
                    }
                }
                return Ok(());
            }
            "TAB_GET_DIALOG" => {
                // batch 220: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                        "TAB_GET_DIALOG requires its arguments: TAB GET DIALOG hDlg , ID& , PageNum& TO PageDlgVar&".to_string(),
                        None,
                        call.line,
                    ));
                }

                // TAB GET DIALOG hDlg, ID&, PageNum& TO PageDlgVar&
                if call.args.len() >= 4 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let page = self.compile_expr(fb, &call.args[2])?;
                    let v = fb.call(&IrType::I64, "pb_tab_get_dialog", &[hd, id, page]);
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[3]) {
                        let vv = self.convert_value(fb, &v, &ty, &pty);
                        fb.store(&vv, &ptr);
                    }
                }
                return Ok(());
            }
            "TAB_GET_IMAGE" => {
                // batch 220: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                        "TAB_GET_IMAGE requires its arguments: TAB GET IMAGE hDlg , ID& , PageNum& TO ImageVar&".to_string(),
                        None,
                        call.line,
                    ));
                }

                // TAB GET IMAGE hDlg, ID&, PageNum& TO ImageVar&
                if call.args.len() >= 4 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let page = self.compile_expr(fb, &call.args[2])?;
                    let v = fb.call(&IrType::I64, "pb_tab_get_image", &[hd, id, page]);
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[3]) {
                        let vv = self.convert_value(fb, &v, &ty, &pty);
                        fb.store(&vv, &ptr);
                    }
                }
                return Ok(());
            }
            "TAB_GET_PAGE" => {
                // batch 220: the official syntax requires all 2 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 2 {
                    return Err(pb::error::PbError::parser(
                        "TAB_GET_PAGE requires its arguments: TAB GET PAGE PageDlg TO PageNumVar&"
                            .to_string(),
                        None,
                        call.line,
                    ));
                }

                // TAB GET PAGE PageDlg TO PageNumVar&
                // The operand is a page dialog handle, not an (hDlg, id) pair.
                if call.args.len() >= 2 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let v = fb.call(&IrType::I64, "pb_tab_get_page", &[hd]);
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[1]) {
                        let vv = self.convert_value(fb, &v, &ty, &pty);
                        fb.store(&vv, &ptr);
                    }
                }
                return Ok(());
            }
            "TAB_GET_SELECT" => {
                // batch 220: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                        "TAB_GET_SELECT requires its arguments: TAB GET SELECT hDlg , ID& TO PageNumVar&".to_string(),
                        None,
                        call.line,
                    ));
                }

                // TAB GET SELECT hDlg, ID& TO PageNumVar&
                if call.args.len() >= 3 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let v = fb.call(&IrType::I64, "pb_tab_get_select", &[hd, id]);
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[2]) {
                        let vv = self.convert_value(fb, &v, &ty, &pty);
                        fb.store(&vv, &ptr);
                    }
                }
                return Ok(());
            }
            "TAB_GET_TEXT" => {
                // batch 220: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                        "TAB_GET_TEXT requires its arguments: TAB GET TEXT hDlg , ID& , PageNum& TO TextVar$".to_string(),
                        None,
                        call.line,
                    ));
                }

                // TAB GET TEXT hDlg, ID&, PageNum& TO TextVar$
                if call.args.len() >= 4 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let page = self.compile_expr(fb, &call.args[2])?;
                    let buf = fb.alloca(&IrType::Array(256, Box::new(IrType::I8)));
                    let bp = fb.gep_byte(&buf, &fb.const_i32(0));
                    let _ = fb.call(
                        &IrType::I64,
                        "pb_tab_get_text",
                        &[hd, id, page, bp.clone(), fb.const_i32(256)],
                    );
                    if let Some((ptr, _, _)) = self.lvalue_ptr(fb, &call.args[3]) {
                        let len =
                            fb.call(&IrType::I32, "pb_str_cstr_len", std::slice::from_ref(&bp));
                        let bstr = fb.call(&IrType::Ptr, "pb_bstr_alloc", &[bp, len]);
                        fb.store(&bstr, &ptr);
                    }
                }
                return Ok(());
            }
            "TAB_INSERT_PAGE" => {
                // TAB INSERT PAGE hDlg, ID&, PageNum&, Image&, Text$ [CALL cb]
                //                 TO PageDlgVar&
                // The parser always emits seven operands; operand 5 is
                // Expr::IntegerLit(0) when no CALL was written.
                if call.args.len() >= 7 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let page = self.compile_expr(fb, &call.args[2])?;
                    let image = self.compile_expr(fb, &call.args[3])?;
                    let text = self.compile_expr(fb, &call.args[4])?;
                    let cbv = if let Expr::Variable(name) = &call.args[5] {
                        let upper = name.to_uppercase();
                        self.functions
                            .get(&upper)
                            .or_else(|| self.subs.get(&upper))
                            .map(|fi| Val::new(format!("@{}", fi.ir_name), IrType::Ptr))
                    } else {
                        None
                    };
                    let cbarg = match cbv {
                        Some(v) => v,
                        None => {
                            let z = fb.const_i64(0);
                            fb.inttoptr(&z)
                        }
                    };
                    let v = fb.call(
                        &IrType::I64,
                        "pb_tab_insert_page",
                        &[hd, id, page, image, text, cbarg],
                    );
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[6]) {
                        let vv = self.convert_value(fb, &v, &ty, &pty);
                        fb.store(&vv, &ptr);
                    }
                }
                return Ok(());
            }
            "TAB_RESET" => {
                // TAB RESET hDlg, ID&
                // batch 219: the official syntax requires all 2 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 2 {
                    return Err(pb::error::PbError::parser(
                        "TAB_RESET requires its arguments: TAB RESET hDlg ,".to_string(),
                        None,
                        call.line,
                    ));
                }
                if call.args.len() >= 2 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let _ = fb.call(&IrType::I64, "pb_tab_reset", &[hd, id]);
                }
                return Ok(());
            }
            "TAB_SELECT" => {
                // batch 220: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                        "TAB_SELECT requires its arguments: TAB SELECT hDlg , ID& , PageNum&"
                            .to_string(),
                        None,
                        call.line,
                    ));
                }

                // TAB SELECT hDlg, ID&, PageNum&
                if call.args.len() >= 3 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let page = self.compile_expr(fb, &call.args[2])?;
                    let _ = fb.call(&IrType::I64, "pb_tab_select", &[hd, id, page]);
                }
                return Ok(());
            }
            "TAB_SET_IMAGE" => {
                // batch 220: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                        "TAB_SET_IMAGE requires its arguments: TAB SET IMAGE hDlg , ID& , PageNum& , Image&".to_string(),
                        None,
                        call.line,
                    ));
                }

                // TAB SET IMAGE hDlg, ID&, PageNum&, Image&
                if call.args.len() >= 4 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let page = self.compile_expr(fb, &call.args[2])?;
                    let image = self.compile_expr(fb, &call.args[3])?;
                    let _ = fb.call(&IrType::I64, "pb_tab_set_image", &[hd, id, page, image]);
                }
                return Ok(());
            }
            "TAB_SET_IMAGELIST" => {
                // TAB SET IMAGELIST hDlg, ID&, hLst
                // batch 219: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                        "TAB_SET_IMAGELIST requires its arguments: TAB SET IMAGELIST hDlg , ID& ,"
                            .to_string(),
                        None,
                        call.line,
                    ));
                }
                if call.args.len() >= 3 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let mut hlst = self.compile_expr(fb, &call.args[2])?;
                    if hlst.ty != IrType::Ptr {
                        hlst = fb.inttoptr(&hlst);
                    }
                    let _ = fb.call(&IrType::I64, "pb_tab_set_imagelist", &[hd, id, hlst]);
                }
                return Ok(());
            }
            "TAB_SET_TEXT" => {
                // batch 220: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                        "TAB_SET_TEXT requires its arguments: TAB SET TEXT hDlg , ID& , PageNum& , Text$".to_string(),
                        None,
                        call.line,
                    ));
                }

                // TAB SET TEXT hDlg, ID&, PageNum&, Text$
                if call.args.len() >= 4 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let page = self.compile_expr(fb, &call.args[2])?;
                    let text = self.compile_expr(fb, &call.args[3])?;
                    let _ = fb.call(&IrType::I64, "pb_tab_set_text", &[hd, id, page, text]);
                }
                return Ok(());
            }
            "SCROLLBAR_GET_POS" => {
                // batch 220: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                        "SCROLLBAR_GET_POS requires its arguments: SCROLLBAR GET POS hDlg , id& TO datav&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 3 {
                    let mut h = self.compile_expr(fb, &call.args[0])?;
                    if h.ty != IrType::Ptr {
                        h = fb.inttoptr(&h);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let v = fb.call(&IrType::I64, "pb_scrollbar_get_pos", &[h, id]);
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[2]) {
                        let cv = self.convert_value(fb, &v, &ty, &pty);
                        fb.store(&cv, &ptr);
                    }
                }
                return Ok(());
            }
            "SCROLLBAR_GET_PAGESIZE" => {
                // batch 220: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                        "SCROLLBAR_GET_PAGESIZE requires its arguments: SCROLLBAR GET PAGESIZE hDlg , id& TO datav&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 3 {
                    let mut h = self.compile_expr(fb, &call.args[0])?;
                    if h.ty != IrType::Ptr {
                        h = fb.inttoptr(&h);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let v = fb.call(&IrType::I64, "pb_scrollbar_get_pagesize", &[h, id]);
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[2]) {
                        let cv = self.convert_value(fb, &v, &ty, &pty);
                        fb.store(&cv, &ptr);
                    }
                }
                return Ok(());
            }
            "SCROLLBAR_GET_TRACKPOS" => {
                // batch 220: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                        "SCROLLBAR_GET_TRACKPOS requires its arguments: SCROLLBAR GET TRACKPOS hDlg , id& TO datav&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 3 {
                    let mut h = self.compile_expr(fb, &call.args[0])?;
                    if h.ty != IrType::Ptr {
                        h = fb.inttoptr(&h);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let v = fb.call(&IrType::I64, "pb_scrollbar_get_trackpos", &[h, id]);
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[2]) {
                        let cv = self.convert_value(fb, &v, &ty, &pty);
                        fb.store(&cv, &ptr);
                    }
                }
                return Ok(());
            }
            "SCROLLBAR_GET_RANGE" => {
                // batch 220: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                        "SCROLLBAR_GET_RANGE requires its arguments: SCROLLBAR GET RANGE hDlg& , id& TO LoDatav& , HiDatav&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 4 {
                    let mut h = self.compile_expr(fb, &call.args[0])?;
                    if h.ty != IrType::Ptr {
                        h = fb.inttoptr(&h);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let lo = fb.call(&IrType::I64, "pb_scrollbar_get_lo", &[h, id]);
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[2]) {
                        let cv = self.convert_value(fb, &lo, &ty, &pty);
                        fb.store(&cv, &ptr);
                    }
                    // fb.call consumes its operands, so the two simple
                    // handle/id operands are re-evaluated for the second query.
                    let mut h2 = self.compile_expr(fb, &call.args[0])?;
                    if h2.ty != IrType::Ptr {
                        h2 = fb.inttoptr(&h2);
                    }
                    let id2 = self.compile_expr(fb, &call.args[1])?;
                    let hi = fb.call(&IrType::I64, "pb_scrollbar_get_hi", &[h2, id2]);
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[3]) {
                        let cv = self.convert_value(fb, &hi, &ty, &pty);
                        fb.store(&cv, &ptr);
                    }
                }
                return Ok(());
            }
            "SCROLLBAR_SET_POS" => {
                // batch 219: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                    "SCROLLBAR_SET_POS requires its arguments: SCROLLBAR SET POS hDlg , id& , position&"
                        .to_string(),
                    None,
                    call.line,
                ));
                }
                if call.args.len() >= 3 {
                    let mut h = self.compile_expr(fb, &call.args[0])?;
                    if h.ty != IrType::Ptr {
                        h = fb.inttoptr(&h);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let a2 = self.compile_expr(fb, &call.args[2])?;
                    fb.call(&IrType::I64, "pb_scrollbar_set_pos", &[h, id, a2]);
                }
                return Ok(());
            }
            "SCROLLBAR_SET_PAGESIZE" => {
                // batch 219: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                    "SCROLLBAR_SET_PAGESIZE requires its arguments: SCROLLBAR SET PAGESIZE hDlg , id& ,"
                        .to_string(),
                    None,
                    call.line,
                ));
                }
                if call.args.len() >= 3 {
                    let mut h = self.compile_expr(fb, &call.args[0])?;
                    if h.ty != IrType::Ptr {
                        h = fb.inttoptr(&h);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let a2 = self.compile_expr(fb, &call.args[2])?;
                    fb.call(&IrType::I64, "pb_scrollbar_set_pagesize", &[h, id, a2]);
                }
                return Ok(());
            }
            "SCROLLBAR_SET_RANGE" => {
                // batch 219: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                    "SCROLLBAR_SET_RANGE requires its arguments: SCROLLBAR SET RANGE hDlg , id& , lolimit& ,"
                        .to_string(),
                    None,
                    call.line,
                ));
                }
                if call.args.len() >= 4 {
                    let mut h = self.compile_expr(fb, &call.args[0])?;
                    if h.ty != IrType::Ptr {
                        h = fb.inttoptr(&h);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let a2 = self.compile_expr(fb, &call.args[2])?;
                    let a3 = self.compile_expr(fb, &call.args[3])?;
                    fb.call(&IrType::I64, "pb_scrollbar_set_range", &[h, id, a2, a3]);
                }
                return Ok(());
            }
            "COMBOBOX_ADD" => {
                if call.args.len() >= 3 {
                    let mut h = self.compile_expr(fb, &call.args[0])?;
                    if h.ty != IrType::Ptr {
                        h = fb.inttoptr(&h);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let text = self.compile_str_payload(fb, &call.args[2])?;
                    let v = fb.call(&IrType::I64, "pb_cblb_add", &[h, id, text, fb.const_i64(0)]);
                    if call.args.len() >= 4 {
                        if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[3]) {
                            let cv = self.convert_value(fb, &v, &ty, &pty);
                            fb.store(&cv, &ptr);
                        }
                    }
                } else if call.args.len() == 2 {
                    // Legacy fork form: COMBOBOX ADD hCtrl, "text"
                    let hc = self.compile_expr(fb, &call.args[0])?;
                    let txt = self.compile_str_payload(fb, &call.args[1])?;
                    fb.call_void("pb_combobox_add", &[hc, txt]);
                }
                return Ok(());
            }
            "COMBOBOX_DELETE" => {
                // batch 219: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                        "COMBOBOX_DELETE requires its arguments: COMBOBOX DELETE hDlg , id& ,"
                            .to_string(),
                        None,
                        call.line,
                    ));
                }
                if call.args.len() >= 3 {
                    let mut h = self.compile_expr(fb, &call.args[0])?;
                    if h.ty != IrType::Ptr {
                        h = fb.inttoptr(&h);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let item = self.compile_expr(fb, &call.args[2])?;
                    fb.call(
                        &IrType::I64,
                        "pb_cblb_delete",
                        &[h, id, item, fb.const_i64(0)],
                    );
                }
                return Ok(());
            }
            "COMBOBOX_FIND" => {
                // batch 220: the official syntax requires all 5 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 5 {
                    return Err(pb::error::PbError::parser(
                        "COMBOBOX_FIND requires its arguments: COMBOBOX FIND hDlg , id& , item& , StrExpr TO datav&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 5 {
                    let mut h = self.compile_expr(fb, &call.args[0])?;
                    if h.ty != IrType::Ptr {
                        h = fb.inttoptr(&h);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let item = self.compile_expr(fb, &call.args[2])?;
                    let text = self.compile_str_payload(fb, &call.args[3])?;
                    let v = fb.call(
                        &IrType::I64,
                        "pb_cblb_find",
                        &[h, id, item, text, fb.const_i64(0), fb.const_i64(0)],
                    );
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[4]) {
                        let cv = self.convert_value(fb, &v, &ty, &pty);
                        fb.store(&cv, &ptr);
                    }
                }
                return Ok(());
            }
            "COMBOBOX_FIND_EXACT" => {
                // batch 220: the official syntax requires all 5 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 5 {
                    return Err(pb::error::PbError::parser(
                        "COMBOBOX_FIND_EXACT requires its arguments: COMBOBOX FIND EXACT hDlg , id& , item& , StrExpr TO datav&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 5 {
                    let mut h = self.compile_expr(fb, &call.args[0])?;
                    if h.ty != IrType::Ptr {
                        h = fb.inttoptr(&h);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let item = self.compile_expr(fb, &call.args[2])?;
                    let text = self.compile_str_payload(fb, &call.args[3])?;
                    let v = fb.call(
                        &IrType::I64,
                        "pb_cblb_find",
                        &[h, id, item, text, fb.const_i64(1), fb.const_i64(0)],
                    );
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[4]) {
                        let cv = self.convert_value(fb, &v, &ty, &pty);
                        fb.store(&cv, &ptr);
                    }
                }
                return Ok(());
            }
            "COMBOBOX_GET_COUNT" => {
                // batch 220: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                        "COMBOBOX_GET_COUNT requires its arguments: COMBOBOX GET COUNT hDlg , id& TO datav&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 3 {
                    let mut h = self.compile_expr(fb, &call.args[0])?;
                    if h.ty != IrType::Ptr {
                        h = fb.inttoptr(&h);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let v = fb.call(&IrType::I64, "pb_cblb_get_count", &[h, id, fb.const_i64(0)]);
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[2]) {
                        let cv = self.convert_value(fb, &v, &ty, &pty);
                        fb.store(&cv, &ptr);
                    }
                }
                return Ok(());
            }
            "COMBOBOX_GET_SELCOUNT" => {
                // batch 220: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                        "COMBOBOX_GET_SELCOUNT requires its arguments: COMBOBOX GET SELCOUNT hDlg , id& TO datav&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 3 {
                    let mut h = self.compile_expr(fb, &call.args[0])?;
                    if h.ty != IrType::Ptr {
                        h = fb.inttoptr(&h);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let v = fb.call(
                        &IrType::I64,
                        "pb_cblb_get_selcount",
                        &[h, id, fb.const_i64(0)],
                    );
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[2]) {
                        let cv = self.convert_value(fb, &v, &ty, &pty);
                        fb.store(&cv, &ptr);
                    }
                }
                return Ok(());
            }
            "COMBOBOX_GET_SELECT" => {
                // batch 220: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                        "COMBOBOX_GET_SELECT requires its arguments: COMBOBOX GET SELECT hDlg , id& TO datav&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 3 {
                    let mut h = self.compile_expr(fb, &call.args[0])?;
                    if h.ty != IrType::Ptr {
                        h = fb.inttoptr(&h);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let v = fb.call(
                        &IrType::I64,
                        "pb_cblb_get_select",
                        &[h, id, fb.const_i32(1), fb.const_i64(0)],
                    );
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[2]) {
                        let cv = self.convert_value(fb, &v, &ty, &pty);
                        fb.store(&cv, &ptr);
                    }
                }
                return Ok(());
            }
            "COMBOBOX_GET_STATE" => {
                // batch 220: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                        "COMBOBOX_GET_STATE requires its arguments: COMBOBOX GET STATE hDlg , id& , item& TO datav&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 4 {
                    let mut h = self.compile_expr(fb, &call.args[0])?;
                    if h.ty != IrType::Ptr {
                        h = fb.inttoptr(&h);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let item = self.compile_expr(fb, &call.args[2])?;
                    let v = fb.call(
                        &IrType::I64,
                        "pb_cblb_get_state",
                        &[h, id, item, fb.const_i64(0)],
                    );
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[3]) {
                        let cv = self.convert_value(fb, &v, &ty, &pty);
                        fb.store(&cv, &ptr);
                    }
                }
                return Ok(());
            }
            "COMBOBOX_GET_TEXT" => {
                if call.args.len() >= 4 {
                    let mut h = self.compile_expr(fb, &call.args[0])?;
                    if h.ty != IrType::Ptr {
                        h = fb.inttoptr(&h);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let item = self.compile_expr(fb, &call.args[2])?;
                    let buf = fb.alloca(&IrType::Array(256, Box::new(IrType::I8)));
                    let bp = fb.gep_byte(&buf, &fb.const_i32(0));
                    fb.call_void(
                        "pb_cblb_get_text",
                        &[h, id, item, bp.clone(), fb.const_i32(256), fb.const_i64(0)],
                    );
                    if let Some((ptr, _, _)) = self.lvalue_ptr(fb, &call.args[3]) {
                        let len =
                            fb.call(&IrType::I32, "pb_str_cstr_len", std::slice::from_ref(&bp));
                        let bstr = fb.call(&IrType::Ptr, "pb_bstr_alloc", &[bp, len]);
                        fb.store(&bstr, &ptr);
                    }
                } else if call.args.len() == 3 {
                    let mut h = self.compile_expr(fb, &call.args[0])?;
                    if h.ty != IrType::Ptr {
                        h = fb.inttoptr(&h);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let buf = fb.alloca(&IrType::Array(256, Box::new(IrType::I8)));
                    let bp = fb.gep_byte(&buf, &fb.const_i32(0));
                    fb.call_void(
                        "pb_cblb_get_text",
                        &[
                            h,
                            id,
                            fb.const_i32(0),
                            bp.clone(),
                            fb.const_i32(256),
                            fb.const_i64(0),
                        ],
                    );
                    if let Some((ptr, _, _)) = self.lvalue_ptr(fb, &call.args[2]) {
                        let len =
                            fb.call(&IrType::I32, "pb_str_cstr_len", std::slice::from_ref(&bp));
                        let bstr = fb.call(&IrType::Ptr, "pb_bstr_alloc", &[bp, len]);
                        fb.store(&bstr, &ptr);
                    }
                }
                return Ok(());
            }
            "COMBOBOX_GET_USER" => {
                // batch 220: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                        "COMBOBOX_GET_USER requires its arguments: COMBOBOX GET USER hDlg , id& , item& TO datav&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 4 {
                    let mut h = self.compile_expr(fb, &call.args[0])?;
                    if h.ty != IrType::Ptr {
                        h = fb.inttoptr(&h);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let item = self.compile_expr(fb, &call.args[2])?;
                    let v = fb.call(
                        &IrType::I64,
                        "pb_cblb_get_user",
                        &[h, id, item, fb.const_i64(0)],
                    );
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[3]) {
                        let cv = self.convert_value(fb, &v, &ty, &pty);
                        fb.store(&cv, &ptr);
                    }
                }
                return Ok(());
            }
            "COMBOBOX_INSERT" => {
                // batch 220: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                        "COMBOBOX_INSERT requires its arguments: COMBOBOX INSERT hDlg , id& , item& , StrExpr [ TO datav& ]".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 4 {
                    let mut h = self.compile_expr(fb, &call.args[0])?;
                    if h.ty != IrType::Ptr {
                        h = fb.inttoptr(&h);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let item = self.compile_expr(fb, &call.args[2])?;
                    let text = self.compile_str_payload(fb, &call.args[3])?;
                    let v = fb.call(
                        &IrType::I64,
                        "pb_cblb_insert",
                        &[h, id, item, text, fb.const_i64(0)],
                    );
                    if call.args.len() >= 5 {
                        if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[4]) {
                            let cv = self.convert_value(fb, &v, &ty, &pty);
                            fb.store(&cv, &ptr);
                        }
                    }
                }
                return Ok(());
            }
            "COMBOBOX_RESET" => {
                // batch 219: the official syntax requires all 2 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 2 {
                    return Err(pb::error::PbError::parser(
                        "COMBOBOX_RESET requires its arguments: COMBOBOX RESET hDlg , id&"
                            .to_string(),
                        None,
                        call.line,
                    ));
                }
                if call.args.len() >= 2 {
                    let mut h = self.compile_expr(fb, &call.args[0])?;
                    if h.ty != IrType::Ptr {
                        h = fb.inttoptr(&h);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    fb.call(&IrType::I64, "pb_cblb_reset", &[h, id, fb.const_i64(0)]);
                }
                return Ok(());
            }
            "COMBOBOX_SELECT" => {
                // batch 219: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                        "COMBOBOX_SELECT requires its arguments: COMBOBOX SELECT hDlg , id& ,"
                            .to_string(),
                        None,
                        call.line,
                    ));
                }
                if call.args.len() >= 3 {
                    let mut h = self.compile_expr(fb, &call.args[0])?;
                    if h.ty != IrType::Ptr {
                        h = fb.inttoptr(&h);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let item = self.compile_expr(fb, &call.args[2])?;
                    fb.call(
                        &IrType::I64,
                        "pb_cblb_select",
                        &[h, id, item, fb.const_i64(0)],
                    );
                }
                return Ok(());
            }
            "COMBOBOX_SET_TEXT" => {
                // batch 219: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                    "COMBOBOX_SET_TEXT requires its arguments: COMBOBOX SET TEXT hDlg , id& , item& ,"
                        .to_string(),
                    None,
                    call.line,
                ));
                }
                if call.args.len() >= 4 {
                    let mut h = self.compile_expr(fb, &call.args[0])?;
                    if h.ty != IrType::Ptr {
                        h = fb.inttoptr(&h);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let item = self.compile_expr(fb, &call.args[2])?;
                    let text = self.compile_str_payload(fb, &call.args[3])?;
                    fb.call(
                        &IrType::I64,
                        "pb_cblb_set_text",
                        &[h, id, item, text, fb.const_i64(0)],
                    );
                }
                return Ok(());
            }
            "COMBOBOX_SET_USER" => {
                // batch 219: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                    "COMBOBOX_SET_USER requires its arguments: COMBOBOX SET USER hDlg , id& , item& ,"
                        .to_string(),
                    None,
                    call.line,
                ));
                }
                if call.args.len() >= 4 {
                    let mut h = self.compile_expr(fb, &call.args[0])?;
                    if h.ty != IrType::Ptr {
                        h = fb.inttoptr(&h);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let item = self.compile_expr(fb, &call.args[2])?;
                    let val = self.compile_expr(fb, &call.args[3])?;
                    let val64 = self.to_i64(fb, &val);
                    fb.call(
                        &IrType::I64,
                        "pb_cblb_set_user",
                        &[h, id, item, val64, fb.const_i64(0)],
                    );
                }
                return Ok(());
            }
            "COMBOBOX_UNSELECT" => {
                // batch 219: the official syntax requires all 2 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 2 {
                    return Err(pb::error::PbError::parser(
                        "COMBOBOX_UNSELECT requires its arguments: COMBOBOX UNSELECT hDlg , id&"
                            .to_string(),
                        None,
                        call.line,
                    ));
                }
                if call.args.len() >= 2 {
                    let mut h = self.compile_expr(fb, &call.args[0])?;
                    if h.ty != IrType::Ptr {
                        h = fb.inttoptr(&h);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    fb.call(
                        &IrType::I64,
                        "pb_cblb_unselect",
                        &[h, id, fb.const_i32(0), fb.const_i64(0)],
                    );
                }
                return Ok(());
            }
            "LISTBOX_ADD" => {
                if call.args.len() >= 3 {
                    let mut h = self.compile_expr(fb, &call.args[0])?;
                    if h.ty != IrType::Ptr {
                        h = fb.inttoptr(&h);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let text = self.compile_str_payload(fb, &call.args[2])?;
                    let v = fb.call(&IrType::I64, "pb_cblb_add", &[h, id, text, fb.const_i64(1)]);
                    if call.args.len() >= 4 {
                        if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[3]) {
                            let cv = self.convert_value(fb, &v, &ty, &pty);
                            fb.store(&cv, &ptr);
                        }
                    }
                } else if call.args.len() == 2 {
                    // Legacy fork form: COMBOBOX ADD hCtrl, "text"
                    let hc = self.compile_expr(fb, &call.args[0])?;
                    let txt = self.compile_str_payload(fb, &call.args[1])?;
                    fb.call_void("pb_listbox_add", &[hc, txt]);
                }
                return Ok(());
            }
            "LISTBOX_DELETE" => {
                // batch 219: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                        "LISTBOX_DELETE requires its arguments: LISTBOX DELETE hDlg , id& ,"
                            .to_string(),
                        None,
                        call.line,
                    ));
                }
                if call.args.len() >= 3 {
                    let mut h = self.compile_expr(fb, &call.args[0])?;
                    if h.ty != IrType::Ptr {
                        h = fb.inttoptr(&h);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let item = self.compile_expr(fb, &call.args[2])?;
                    fb.call(
                        &IrType::I64,
                        "pb_cblb_delete",
                        &[h, id, item, fb.const_i64(1)],
                    );
                }
                return Ok(());
            }
            "LISTBOX_FIND" => {
                // batch 220: the official syntax requires all 5 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 5 {
                    return Err(pb::error::PbError::parser(
                        "LISTBOX_FIND requires its arguments: LISTBOX FIND hDlg , id& , item& , StrExpr TO datav&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 5 {
                    let mut h = self.compile_expr(fb, &call.args[0])?;
                    if h.ty != IrType::Ptr {
                        h = fb.inttoptr(&h);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let item = self.compile_expr(fb, &call.args[2])?;
                    let text = self.compile_str_payload(fb, &call.args[3])?;
                    let v = fb.call(
                        &IrType::I64,
                        "pb_cblb_find",
                        &[h, id, item, text, fb.const_i64(0), fb.const_i64(1)],
                    );
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[4]) {
                        let cv = self.convert_value(fb, &v, &ty, &pty);
                        fb.store(&cv, &ptr);
                    }
                }
                return Ok(());
            }
            "LISTBOX_FIND_EXACT" => {
                // batch 220: the official syntax requires all 5 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 5 {
                    return Err(pb::error::PbError::parser(
                        "LISTBOX_FIND_EXACT requires its arguments: LISTBOX FIND EXACT hDlg , id& , item& , StrExpr TO datav&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 5 {
                    let mut h = self.compile_expr(fb, &call.args[0])?;
                    if h.ty != IrType::Ptr {
                        h = fb.inttoptr(&h);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let item = self.compile_expr(fb, &call.args[2])?;
                    let text = self.compile_str_payload(fb, &call.args[3])?;
                    let v = fb.call(
                        &IrType::I64,
                        "pb_cblb_find",
                        &[h, id, item, text, fb.const_i64(1), fb.const_i64(1)],
                    );
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[4]) {
                        let cv = self.convert_value(fb, &v, &ty, &pty);
                        fb.store(&cv, &ptr);
                    }
                }
                return Ok(());
            }
            "LISTBOX_GET_COUNT" => {
                // batch 220: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                        "LISTBOX_GET_COUNT requires its arguments: LISTBOX GET COUNT hDlg , id& TO datav&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 3 {
                    let mut h = self.compile_expr(fb, &call.args[0])?;
                    if h.ty != IrType::Ptr {
                        h = fb.inttoptr(&h);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let v = fb.call(&IrType::I64, "pb_cblb_get_count", &[h, id, fb.const_i64(1)]);
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[2]) {
                        let cv = self.convert_value(fb, &v, &ty, &pty);
                        fb.store(&cv, &ptr);
                    }
                }
                return Ok(());
            }
            "LISTBOX_GET_SELCOUNT" => {
                // batch 220: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                        "LISTBOX_GET_SELCOUNT requires its arguments: LISTBOX GET SELCOUNT hDlg , id& TO datav&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 3 {
                    let mut h = self.compile_expr(fb, &call.args[0])?;
                    if h.ty != IrType::Ptr {
                        h = fb.inttoptr(&h);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let v = fb.call(
                        &IrType::I64,
                        "pb_cblb_get_selcount",
                        &[h, id, fb.const_i64(1)],
                    );
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[2]) {
                        let cv = self.convert_value(fb, &v, &ty, &pty);
                        fb.store(&cv, &ptr);
                    }
                }
                return Ok(());
            }
            "LISTBOX_GET_SELECT" => {
                if call.args.len() >= 4 {
                    let mut h = self.compile_expr(fb, &call.args[0])?;
                    if h.ty != IrType::Ptr {
                        h = fb.inttoptr(&h);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let start = self.compile_expr(fb, &call.args[2])?;
                    let v = fb.call(
                        &IrType::I64,
                        "pb_cblb_get_select",
                        &[h, id, start, fb.const_i64(1)],
                    );
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[3]) {
                        let cv = self.convert_value(fb, &v, &ty, &pty);
                        fb.store(&cv, &ptr);
                    }
                } else if call.args.len() == 3 {
                    let mut h = self.compile_expr(fb, &call.args[0])?;
                    if h.ty != IrType::Ptr {
                        h = fb.inttoptr(&h);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let v = fb.call(
                        &IrType::I64,
                        "pb_cblb_get_select",
                        &[h, id, fb.const_i32(1), fb.const_i64(1)],
                    );
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[2]) {
                        let cv = self.convert_value(fb, &v, &ty, &pty);
                        fb.store(&cv, &ptr);
                    }
                }
                return Ok(());
            }
            "LISTBOX_GET_STATE" => {
                // batch 220: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                        "LISTBOX_GET_STATE requires its arguments: LISTBOX GET STATE hDlg , id& , item& TO datav&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 4 {
                    let mut h = self.compile_expr(fb, &call.args[0])?;
                    if h.ty != IrType::Ptr {
                        h = fb.inttoptr(&h);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let item = self.compile_expr(fb, &call.args[2])?;
                    let v = fb.call(
                        &IrType::I64,
                        "pb_cblb_get_state",
                        &[h, id, item, fb.const_i64(1)],
                    );
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[3]) {
                        let cv = self.convert_value(fb, &v, &ty, &pty);
                        fb.store(&cv, &ptr);
                    }
                }
                return Ok(());
            }
            "LISTBOX_GET_TEXT" => {
                if call.args.len() >= 4 {
                    let mut h = self.compile_expr(fb, &call.args[0])?;
                    if h.ty != IrType::Ptr {
                        h = fb.inttoptr(&h);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let item = self.compile_expr(fb, &call.args[2])?;
                    let buf = fb.alloca(&IrType::Array(256, Box::new(IrType::I8)));
                    let bp = fb.gep_byte(&buf, &fb.const_i32(0));
                    fb.call_void(
                        "pb_cblb_get_text",
                        &[h, id, item, bp.clone(), fb.const_i32(256), fb.const_i64(1)],
                    );
                    if let Some((ptr, _, _)) = self.lvalue_ptr(fb, &call.args[3]) {
                        let len =
                            fb.call(&IrType::I32, "pb_str_cstr_len", std::slice::from_ref(&bp));
                        let bstr = fb.call(&IrType::Ptr, "pb_bstr_alloc", &[bp, len]);
                        fb.store(&bstr, &ptr);
                    }
                } else if call.args.len() == 3 {
                    let mut h = self.compile_expr(fb, &call.args[0])?;
                    if h.ty != IrType::Ptr {
                        h = fb.inttoptr(&h);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let buf = fb.alloca(&IrType::Array(256, Box::new(IrType::I8)));
                    let bp = fb.gep_byte(&buf, &fb.const_i32(0));
                    fb.call_void(
                        "pb_cblb_get_text",
                        &[
                            h,
                            id,
                            fb.const_i32(0),
                            bp.clone(),
                            fb.const_i32(256),
                            fb.const_i64(1),
                        ],
                    );
                    if let Some((ptr, _, _)) = self.lvalue_ptr(fb, &call.args[2]) {
                        let len =
                            fb.call(&IrType::I32, "pb_str_cstr_len", std::slice::from_ref(&bp));
                        let bstr = fb.call(&IrType::Ptr, "pb_bstr_alloc", &[bp, len]);
                        fb.store(&bstr, &ptr);
                    }
                }
                return Ok(());
            }
            "LISTBOX_GET_USER" => {
                // batch 220: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                        "LISTBOX_GET_USER requires its arguments: LISTBOX GET USER hDlg , id& , item& TO datav&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 4 {
                    let mut h = self.compile_expr(fb, &call.args[0])?;
                    if h.ty != IrType::Ptr {
                        h = fb.inttoptr(&h);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let item = self.compile_expr(fb, &call.args[2])?;
                    let v = fb.call(
                        &IrType::I64,
                        "pb_cblb_get_user",
                        &[h, id, item, fb.const_i64(1)],
                    );
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[3]) {
                        let cv = self.convert_value(fb, &v, &ty, &pty);
                        fb.store(&cv, &ptr);
                    }
                }
                return Ok(());
            }
            "LISTBOX_INSERT" => {
                // batch 220: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                        "LISTBOX_INSERT requires its arguments: LISTBOX INSERT hDlg , id& , item& , StrExpr [ TO datav& ]".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 4 {
                    let mut h = self.compile_expr(fb, &call.args[0])?;
                    if h.ty != IrType::Ptr {
                        h = fb.inttoptr(&h);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let item = self.compile_expr(fb, &call.args[2])?;
                    let text = self.compile_str_payload(fb, &call.args[3])?;
                    let v = fb.call(
                        &IrType::I64,
                        "pb_cblb_insert",
                        &[h, id, item, text, fb.const_i64(1)],
                    );
                    if call.args.len() >= 5 {
                        if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[4]) {
                            let cv = self.convert_value(fb, &v, &ty, &pty);
                            fb.store(&cv, &ptr);
                        }
                    }
                }
                return Ok(());
            }
            "LISTBOX_RESET" => {
                // batch 219: the official syntax requires all 2 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 2 {
                    return Err(pb::error::PbError::parser(
                        "LISTBOX_RESET requires its arguments: LISTBOX RESET hDlg , id&"
                            .to_string(),
                        None,
                        call.line,
                    ));
                }
                if call.args.len() >= 2 {
                    let mut h = self.compile_expr(fb, &call.args[0])?;
                    if h.ty != IrType::Ptr {
                        h = fb.inttoptr(&h);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    fb.call(&IrType::I64, "pb_cblb_reset", &[h, id, fb.const_i64(1)]);
                }
                return Ok(());
            }
            "LISTBOX_SELECT" => {
                // batch 219: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                        "LISTBOX_SELECT requires its arguments: LISTBOX SELECT hDlg , id& ,"
                            .to_string(),
                        None,
                        call.line,
                    ));
                }
                if call.args.len() >= 3 {
                    let mut h = self.compile_expr(fb, &call.args[0])?;
                    if h.ty != IrType::Ptr {
                        h = fb.inttoptr(&h);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let item = self.compile_expr(fb, &call.args[2])?;
                    fb.call(
                        &IrType::I64,
                        "pb_cblb_select",
                        &[h, id, item, fb.const_i64(1)],
                    );
                }
                return Ok(());
            }
            "LISTBOX_SET_TEXT" => {
                // batch 220: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                        "LISTBOX_SET_TEXT requires its arguments: LISTBOX SET TEXT hDlg , id& , item& , StrExpr".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 4 {
                    let mut h = self.compile_expr(fb, &call.args[0])?;
                    if h.ty != IrType::Ptr {
                        h = fb.inttoptr(&h);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let item = self.compile_expr(fb, &call.args[2])?;
                    let text = self.compile_str_payload(fb, &call.args[3])?;
                    fb.call(
                        &IrType::I64,
                        "pb_cblb_set_text",
                        &[h, id, item, text, fb.const_i64(1)],
                    );
                }
                return Ok(());
            }
            "LISTBOX_SET_USER" => {
                // batch 220: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                        "LISTBOX_SET_USER requires its arguments: LISTBOX SET USER hDlg , id& , item& , NumExpr".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 4 {
                    let mut h = self.compile_expr(fb, &call.args[0])?;
                    if h.ty != IrType::Ptr {
                        h = fb.inttoptr(&h);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let item = self.compile_expr(fb, &call.args[2])?;
                    let val = self.compile_expr(fb, &call.args[3])?;
                    let val64 = self.to_i64(fb, &val);
                    fb.call(
                        &IrType::I64,
                        "pb_cblb_set_user",
                        &[h, id, item, val64, fb.const_i64(1)],
                    );
                }
                return Ok(());
            }
            "LISTBOX_UNSELECT" => {
                if call.args.len() >= 2 {
                    let mut h = self.compile_expr(fb, &call.args[0])?;
                    if h.ty != IrType::Ptr {
                        h = fb.inttoptr(&h);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let item = if call.args.len() >= 3 {
                        self.compile_expr(fb, &call.args[2])?
                    } else {
                        fb.const_i32(0)
                    };
                    fb.call(
                        &IrType::I64,
                        "pb_cblb_unselect",
                        &[h, id, item, fb.const_i64(1)],
                    );
                }
                return Ok(());
            }
            "CONTROL_GET_CHECK" => {
                // batch 219: the official syntax requires all 2 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 2 {
                    return Err(pb::error::PbError::parser(
                        "CONTROL_GET_CHECK requires its arguments: CONTROL GET CHECK hDlg ,"
                            .to_string(),
                        None,
                        call.line,
                    ));
                }
                if call.args.len() >= 2 {
                    let hc = self.compile_expr(fb, &call.args[0])?;
                    let val = fb.call(&IrType::I64, "pb_control_get_check", &[hc]);
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[1]) {
                        let vv = self.convert_value(fb, &val, &ty, &pty);
                        fb.store(&vv, &ptr);
                    }
                }
                return Ok(());
            }
            "CONTROL_ADD_HSCROLLBAR" => {
                // batch 220: the official syntax requires all 7 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 7 {
                    return Err(pb::error::PbError::parser(
                        "CONTROL_ADD_HSCROLLBAR requires its arguments: CONTROL ADD HSCROLLBAR hDlg , id& , x , y , xx , yy TO hCtrl&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 7 {
                    let parent = self.compile_expr(fb, &call.args[0])?;
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let x = self.compile_expr(fb, &call.args[2])?;
                    let y = self.compile_expr(fb, &call.args[3])?;
                    let w = self.compile_expr(fb, &call.args[4])?;
                    let h = self.compile_expr(fb, &call.args[5])?;
                    let hc = fb.call(
                        &IrType::Ptr,
                        "pb_control_add_hscrollbar",
                        &[parent, id, x, y, w, h],
                    );
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[6]) {
                        let hc_i = fb.ptrtoint64(&hc);
                        let hc_i_v = self.convert_value(fb, &hc_i, &ty, &pty);
                        fb.store(&hc_i_v, &ptr);
                    }
                }
                return Ok(());
            }
            "CONTROL_ADD_SCROLLBAR" => {
                // batch 220: the official syntax requires all 7 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 7 {
                    return Err(pb::error::PbError::parser(
                        "CONTROL_ADD_SCROLLBAR requires its arguments: CONTROL ADD SCROLLBAR hDlg , id& , x , y , xx , yy TO hCtrl&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 7 {
                    let parent = self.compile_expr(fb, &call.args[0])?;
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let x = self.compile_expr(fb, &call.args[2])?;
                    let y = self.compile_expr(fb, &call.args[3])?;
                    let w = self.compile_expr(fb, &call.args[4])?;
                    let h = self.compile_expr(fb, &call.args[5])?;
                    // batch 249: [, [style&] [, [exstyle&]]] [[,] CALL callback] [TO]
                    // (no text slot; layout = parent,id,x,y,w,h,style,exstyle,cb,target)
                    let target = if call.args.len() == 7 {
                        Some(call.args[6].clone())
                    } else {
                        call.args.get(9).cloned()
                    };
                    let hctrl = if call.args.len() == 7 {
                        fb.call(
                            &IrType::Ptr,
                            "pb_control_add_scrollbar",
                            &[parent, id, x, y, w, h],
                        )
                    } else {
                        let style = self.compile_expr(fb, &call.args[6])?;
                        let exstyle = self.compile_expr(fb, &call.args[7])?;
                        let cb = self.compile_control_callback(fb, &call.args[8])?;
                        fb.call(
                            &IrType::Ptr,
                            "pb_control_add_scrollbar_ex",
                            &[parent, id, x, y, w, h, style, exstyle, cb],
                        )
                    };
                    if let Some(tgt) = target {
                        if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &tgt) {
                            let hc_i = fb.ptrtoint64(&hctrl);
                            let hc_i_v = self.convert_value(fb, &hc_i, &ty, &pty);
                            fb.store(&hc_i_v, &ptr);
                        }
                    }
                }
                return Ok(());
            }
            "CONTROL_ADD_LABEL" => {
                // batch 220: the official syntax requires all 8 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 8 {
                    return Err(pb::error::PbError::parser(
                        "CONTROL_ADD_LABEL requires its arguments: CONTROL ADD LABEL hDlg , id& , txt$ , x , y , xx , yy TO hCtrl&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 8 {
                    let parent = self.compile_expr(fb, &call.args[0])?;
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let text = self.compile_expr(fb, &call.args[2])?;
                    let x = self.compile_expr(fb, &call.args[3])?;
                    let y = self.compile_expr(fb, &call.args[4])?;
                    let w = self.compile_expr(fb, &call.args[5])?;
                    let h = self.compile_expr(fb, &call.args[6])?;
                    // batch 249: full official syntax (mirrors BUTTON)
                    let target = if call.args.len() == 8 {
                        Some(call.args[7].clone())
                    } else {
                        call.args.get(10).cloned()
                    };
                    let hctrl = if call.args.len() == 8 {
                        fb.call(
                            &IrType::Ptr,
                            "pb_control_add_label",
                            &[parent, id, text, x, y, w, h],
                        )
                    } else {
                        let style = self.compile_expr(fb, &call.args[7])?;
                        let exstyle = self.compile_expr(fb, &call.args[8])?;
                        let cb = self.compile_control_callback(fb, &call.args[9])?;
                        fb.call(
                            &IrType::Ptr,
                            "pb_control_add_label_ex",
                            &[parent, id, text, x, y, w, h, style, exstyle, cb],
                        )
                    };
                    if let Some(tgt) = target {
                        if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &tgt) {
                            let hc_i = fb.ptrtoint64(&hctrl);
                            let hc_i_v = self.convert_value(fb, &hc_i, &ty, &pty);
                            fb.store(&hc_i_v, &ptr);
                        }
                    }
                }
                return Ok(());
            }
            "CONTROL_ADD_PROGRESSBAR" => {
                // batch 220: the official syntax requires all 7 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 7 {
                    return Err(pb::error::PbError::parser(
                        "CONTROL_ADD_PROGRESSBAR requires its arguments: CONTROL ADD PROGRESSBAR hDlg , id& , x , y , xx , yy TO hCtrl&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 7 {
                    let parent = self.compile_expr(fb, &call.args[0])?;
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let x = self.compile_expr(fb, &call.args[2])?;
                    let y = self.compile_expr(fb, &call.args[3])?;
                    let w = self.compile_expr(fb, &call.args[4])?;
                    let h = self.compile_expr(fb, &call.args[5])?;
                    let hc = fb.call(
                        &IrType::Ptr,
                        "pb_control_add_progressbar",
                        &[parent, id, x, y, w, h],
                    );
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[6]) {
                        let hc_i = fb.ptrtoint64(&hc);
                        let hc_i_v = self.convert_value(fb, &hc_i, &ty, &pty);
                        fb.store(&hc_i_v, &ptr);
                    }
                }
                return Ok(());
            }
            "CONTROL_ADD_MONTHCAL" => {
                // batch 220: the official syntax requires all 7 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 7 {
                    return Err(pb::error::PbError::parser(
                        "CONTROL_ADD_MONTHCAL requires its arguments: CONTROL ADD MONTHCAL hDlg , id& , x , y , xx , yy TO hCtrl&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 7 {
                    let mut parent = self.compile_expr(fb, &call.args[0])?;
                    if parent.ty != IrType::Ptr {
                        parent = fb.inttoptr(&parent);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let x = self.compile_expr(fb, &call.args[2])?;
                    let y = self.compile_expr(fb, &call.args[3])?;
                    let w = self.compile_expr(fb, &call.args[4])?;
                    let h = self.compile_expr(fb, &call.args[5])?;
                    let hc = fb.call(
                        &IrType::Ptr,
                        "pb_control_add_monthcal",
                        &[parent, id, x, y, w, h],
                    );
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[6]) {
                        let hc_i = fb.ptrtoint64(&hc);
                        let hc_i_v = self.convert_value(fb, &hc_i, &ty, &pty);
                        fb.store(&hc_i_v, &ptr);
                    }
                }
                return Ok(());
            }
            "CONTROL_ADD_ANIMATE" => {
                // batch 220: the official syntax requires all 7 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 7 {
                    return Err(pb::error::PbError::parser(
                        "CONTROL_ADD_ANIMATE requires its arguments: CONTROL ADD ANIMATE hDlg , id& , x , y , xx , yy TO hCtrl&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 7 {
                    let mut parent = self.compile_expr(fb, &call.args[0])?;
                    if parent.ty != IrType::Ptr {
                        parent = fb.inttoptr(&parent);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let x = self.compile_expr(fb, &call.args[2])?;
                    let y = self.compile_expr(fb, &call.args[3])?;
                    let w = self.compile_expr(fb, &call.args[4])?;
                    let h = self.compile_expr(fb, &call.args[5])?;
                    let hc = fb.call(
                        &IrType::Ptr,
                        "pb_control_add_animate",
                        &[parent, id, x, y, w, h],
                    );
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[6]) {
                        let hc_i = fb.ptrtoint64(&hc);
                        let hc_i_v = self.convert_value(fb, &hc_i, &ty, &pty);
                        fb.store(&hc_i_v, &ptr);
                    }
                }
                return Ok(());
            }
            "CONTROL_ADD_RICHEDIT" => {
                // batch 220: the official syntax requires all 7 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 7 {
                    return Err(pb::error::PbError::parser(
                        "CONTROL_ADD_RICHEDIT requires its arguments: CONTROL ADD RICHEDIT hDlg , id& , x , y , xx , yy TO hCtrl&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 7 {
                    let mut parent = self.compile_expr(fb, &call.args[0])?;
                    if parent.ty != IrType::Ptr {
                        parent = fb.inttoptr(&parent);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let x = self.compile_expr(fb, &call.args[2])?;
                    let y = self.compile_expr(fb, &call.args[3])?;
                    let w = self.compile_expr(fb, &call.args[4])?;
                    let h = self.compile_expr(fb, &call.args[5])?;
                    let hc = fb.call(
                        &IrType::Ptr,
                        "pb_control_add_richedit",
                        &[parent, id, x, y, w, h],
                    );
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[6]) {
                        let hc_i = fb.ptrtoint64(&hc);
                        let hc_i_v = self.convert_value(fb, &hc_i, &ty, &pty);
                        fb.store(&hc_i_v, &ptr);
                    }
                }
                return Ok(());
            }
            "CONTROL_ADD_TRACKBAR" => {
                // batch 220: the official syntax requires all 7 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 7 {
                    return Err(pb::error::PbError::parser(
                        "CONTROL_ADD_TRACKBAR requires its arguments: CONTROL ADD TRACKBAR hDlg , id& , x , y , xx , yy TO hCtrl&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 7 {
                    let mut parent = self.compile_expr(fb, &call.args[0])?;
                    if parent.ty != IrType::Ptr {
                        parent = fb.inttoptr(&parent);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let x = self.compile_expr(fb, &call.args[2])?;
                    let y = self.compile_expr(fb, &call.args[3])?;
                    let w = self.compile_expr(fb, &call.args[4])?;
                    let h = self.compile_expr(fb, &call.args[5])?;
                    let hc = fb.call(
                        &IrType::Ptr,
                        "pb_control_add_trackbar",
                        &[parent, id, x, y, w, h],
                    );
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[6]) {
                        let hc_i = fb.ptrtoint64(&hc);
                        let hc_i_v = self.convert_value(fb, &hc_i, &ty, &pty);
                        fb.store(&hc_i_v, &ptr);
                    }
                }
                return Ok(());
            }
            "CONTROL_ADD_UPDOWN" => {
                // batch 220: the official syntax requires all 7 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 7 {
                    return Err(pb::error::PbError::parser(
                        "CONTROL_ADD_UPDOWN requires its arguments: CONTROL ADD UPDOWN hDlg , id& , x , y , xx , yy TO hCtrl&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 7 {
                    let mut parent = self.compile_expr(fb, &call.args[0])?;
                    if parent.ty != IrType::Ptr {
                        parent = fb.inttoptr(&parent);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let x = self.compile_expr(fb, &call.args[2])?;
                    let y = self.compile_expr(fb, &call.args[3])?;
                    let w = self.compile_expr(fb, &call.args[4])?;
                    let h = self.compile_expr(fb, &call.args[5])?;
                    let hc = fb.call(
                        &IrType::Ptr,
                        "pb_control_add_updown",
                        &[parent, id, x, y, w, h],
                    );
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[6]) {
                        let hc_i = fb.ptrtoint64(&hc);
                        let hc_i_v = self.convert_value(fb, &hc_i, &ty, &pty);
                        fb.store(&hc_i_v, &ptr);
                    }
                }
                return Ok(());
            }
            "CONTROL_ADD_HOTKEY" => {
                // batch 220: the official syntax requires all 7 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 7 {
                    return Err(pb::error::PbError::parser(
                        "CONTROL_ADD_HOTKEY requires its arguments: CONTROL ADD HOTKEY hDlg , id& , x , y , xx , yy TO hCtrl&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 7 {
                    let mut parent = self.compile_expr(fb, &call.args[0])?;
                    if parent.ty != IrType::Ptr {
                        parent = fb.inttoptr(&parent);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let x = self.compile_expr(fb, &call.args[2])?;
                    let y = self.compile_expr(fb, &call.args[3])?;
                    let w = self.compile_expr(fb, &call.args[4])?;
                    let h = self.compile_expr(fb, &call.args[5])?;
                    let hc = fb.call(
                        &IrType::Ptr,
                        "pb_control_add_hotkey",
                        &[parent, id, x, y, w, h],
                    );
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[6]) {
                        let hc_i = fb.ptrtoint64(&hc);
                        let hc_i_v = self.convert_value(fb, &hc_i, &ty, &pty);
                        fb.store(&hc_i_v, &ptr);
                    }
                }
                return Ok(());
            }
            "CONTROL_ADD_IPADDRESS" => {
                // batch 220: the official syntax requires all 7 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 7 {
                    return Err(pb::error::PbError::parser(
                        "CONTROL_ADD_IPADDRESS requires its arguments: CONTROL ADD IPADDRESS hDlg , id& , x , y , xx , yy TO hCtrl&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 7 {
                    let mut parent = self.compile_expr(fb, &call.args[0])?;
                    if parent.ty != IrType::Ptr {
                        parent = fb.inttoptr(&parent);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let x = self.compile_expr(fb, &call.args[2])?;
                    let y = self.compile_expr(fb, &call.args[3])?;
                    let w = self.compile_expr(fb, &call.args[4])?;
                    let h = self.compile_expr(fb, &call.args[5])?;
                    let hc = fb.call(
                        &IrType::Ptr,
                        "pb_control_add_ipaddress",
                        &[parent, id, x, y, w, h],
                    );
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[6]) {
                        let hc_i = fb.ptrtoint64(&hc);
                        let hc_i_v = self.convert_value(fb, &hc_i, &ty, &pty);
                        fb.store(&hc_i_v, &ptr);
                    }
                }
                return Ok(());
            }
            "CONTROL_ADD_TAB" => {
                // batch 220: the official syntax requires all 7 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 7 {
                    return Err(pb::error::PbError::parser(
                        "CONTROL_ADD_TAB requires its arguments: CONTROL ADD TAB hDlg , id& , x , y , xx , yy TO hCtrl&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 7 {
                    let mut parent = self.compile_expr(fb, &call.args[0])?;
                    if parent.ty != IrType::Ptr {
                        parent = fb.inttoptr(&parent);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let x = self.compile_expr(fb, &call.args[2])?;
                    let y = self.compile_expr(fb, &call.args[3])?;
                    let w = self.compile_expr(fb, &call.args[4])?;
                    let h = self.compile_expr(fb, &call.args[5])?;
                    // batch 249: [, [style&] [, [exstyle&]]] [[,] CALL callback] [TO]
                    // (no text slot; layout = parent,id,x,y,w,h,style,exstyle,cb,target)
                    let target = if call.args.len() == 7 {
                        Some(call.args[6].clone())
                    } else {
                        call.args.get(9).cloned()
                    };
                    let hctrl = if call.args.len() == 7 {
                        fb.call(
                            &IrType::Ptr,
                            "pb_control_add_tab",
                            &[parent, id, x, y, w, h],
                        )
                    } else {
                        let style = self.compile_expr(fb, &call.args[6])?;
                        let exstyle = self.compile_expr(fb, &call.args[7])?;
                        let cb = self.compile_control_callback(fb, &call.args[8])?;
                        fb.call(
                            &IrType::Ptr,
                            "pb_control_add_tab_ex",
                            &[parent, id, x, y, w, h, style, exstyle, cb],
                        )
                    };
                    if let Some(tgt) = target {
                        if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &tgt) {
                            let hc_i = fb.ptrtoint64(&hctrl);
                            let hc_i_v = self.convert_value(fb, &hc_i, &ty, &pty);
                            fb.store(&hc_i_v, &ptr);
                        }
                    }
                }
                return Ok(());
            }
            "CONTROL_ADD_LISTVIEW" => {
                // batch 220: the official syntax requires all 7 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 7 {
                    return Err(pb::error::PbError::parser(
                        "CONTROL_ADD_LISTVIEW requires its arguments: CONTROL ADD LISTVIEW hDlg , id& , x , y , xx , yy TO hCtrl&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 7 {
                    let mut parent = self.compile_expr(fb, &call.args[0])?;
                    if parent.ty != IrType::Ptr {
                        parent = fb.inttoptr(&parent);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let x = self.compile_expr(fb, &call.args[2])?;
                    let y = self.compile_expr(fb, &call.args[3])?;
                    let w = self.compile_expr(fb, &call.args[4])?;
                    let h = self.compile_expr(fb, &call.args[5])?;
                    let hc = fb.call(
                        &IrType::Ptr,
                        "pb_control_add_listview",
                        &[parent, id, x, y, w, h],
                    );
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[6]) {
                        let hc_i = fb.ptrtoint64(&hc);
                        let hc_i_v = self.convert_value(fb, &hc_i, &ty, &pty);
                        fb.store(&hc_i_v, &ptr);
                    }
                }
                return Ok(());
            }
            "CONTROL_ADD_TREEVIEW" => {
                if call.args.len() >= 7 {
                    let mut parent = self.compile_expr(fb, &call.args[0])?;
                    if parent.ty != IrType::Ptr {
                        parent = fb.inttoptr(&parent);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let x = self.compile_expr(fb, &call.args[2])?;
                    let y = self.compile_expr(fb, &call.args[3])?;
                    let w = self.compile_expr(fb, &call.args[4])?;
                    let h = self.compile_expr(fb, &call.args[5])?;
                    // Optional style&/exstyle& occupy slots 7 and 8; the parser
                    // always emits them, but stay tolerant of a bare 7-operand
                    // form so hand-built ASTs keep working. 0 means "omitted",
                    // which is how the runtime selects the documented default.
                    let style = if call.args.len() >= 9 {
                        let v = self.compile_expr(fb, &call.args[7])?;
                        self.convert_value(fb, &v, &IrType::I32, &PbType::Long)
                    } else {
                        fb.const_i32(0)
                    };
                    let exstyle = if call.args.len() >= 9 {
                        let v = self.compile_expr(fb, &call.args[8])?;
                        self.convert_value(fb, &v, &IrType::I32, &PbType::Long)
                    } else {
                        fb.const_i32(0)
                    };
                    let hc = fb.call(
                        &IrType::Ptr,
                        "pb_control_add_treeview",
                        &[parent, id, x, y, w, h, style, exstyle],
                    );
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[6]) {
                        let hc_i = fb.ptrtoint64(&hc);
                        let hc_i_v = self.convert_value(fb, &hc_i, &ty, &pty);
                        fb.store(&hc_i_v, &ptr);
                    }
                }
                return Ok(());
            }
            // ---- TOOLBAR / STATUSBAR (batch 164) ----
            // The parser normalises every optional operand into a fixed slot, so
            // these arms read positions rather than guessing at the arity.
            "CONTROL_ADD_TOOLBAR"
            | "CONTROL_ADD_STATUSBAR"
            | "CONTROL_ADD_GRAPHIC"
            | "CONTROL_ADD_HEADER" => {
                if call.args.len() >= 10 {
                    let mut parent = self.compile_expr(fb, &call.args[0])?;
                    if parent.ty != IrType::Ptr {
                        parent = fb.inttoptr(&parent);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let text = self.compile_expr(fb, &call.args[2])?;
                    let x = self.compile_expr(fb, &call.args[3])?;
                    let y = self.compile_expr(fb, &call.args[4])?;
                    let w = self.compile_expr(fb, &call.args[5])?;
                    let h = self.compile_expr(fb, &call.args[6])?;
                    let style_v = self.compile_expr(fb, &call.args[8])?;
                    let exstyle_v = self.compile_expr(fb, &call.args[9])?;
                    // The declared signature takes i32 for both, so normalise
                    // here: a %CCS_* equate or a LONG variable both land in i32.
                    let style = self.convert_value(fb, &style_v, &IrType::I32, &PbType::Long);
                    let exstyle = self.convert_value(fb, &exstyle_v, &IrType::I32, &PbType::Long);
                    let fname = match call.name.as_str() {
                        "CONTROL_ADD_STATUSBAR" => "pb_control_add_statusbar",
                        "CONTROL_ADD_GRAPHIC" => "pb_control_add_graphic",
                        "CONTROL_ADD_HEADER" => "pb_control_add_header",
                        _ => "pb_control_add_toolbar",
                    };
                    let hc = fb.call(
                        &IrType::Ptr,
                        fname,
                        &[parent, id, text, x, y, w, h, style, exstyle],
                    );
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[7]) {
                        let hc_i = fb.ptrtoint64(&hc);
                        let hc_i_v = self.convert_value(fb, &hc_i, &ty, &pty);
                        fb.store(&hc_i_v, &ptr);
                    }
                    // Optional trailing CALL callback -> register it on the control
                    if call.args.len() >= 11 {
                        if let Expr::Variable(name) = &call.args[10] {
                            let upper = name.to_uppercase();
                            let fi = self.functions.get(&upper).or_else(|| self.subs.get(&upper));
                            if let Some(fi) = fi {
                                let fn_ptr = Val::new(format!("@{}", fi.ir_name), IrType::Ptr);
                                fb.call_void("pb_register_callback_hwnd", &[hc, fn_ptr]);
                            } else {
                                eprintln!(
                                    "warning: line {}: CONTROL ADD: unknown sub `{}`",
                                    call.line, upper
                                );
                            }
                        }
                    }
                }
                return Ok(());
            }
            "TOOLBAR_ADD_BUTTON" => {
                if call.args.len() >= 9 {
                    let a = self.compile_expr(fb, &call.args[0])?;
                    let b = self.compile_expr(fb, &call.args[1])?;
                    let image = self.compile_expr(fb, &call.args[2])?;
                    let cmd = self.compile_expr(fb, &call.args[3])?;
                    let style = self.compile_expr(fb, &call.args[4])?;
                    let text = self.compile_expr(fb, &call.args[5])?;
                    let at = self.compile_expr(fb, &call.args[6])?;
                    fb.call(
                        &IrType::I64,
                        "pb_toolbar_add_button",
                        &[a, b, image, cmd, style, text, at],
                    );
                    // Optional CALL callback -> register it on the toolbar
                    if let Expr::Variable(name) = &call.args[7] {
                        let upper = name.to_uppercase();
                        let fi = self.functions.get(&upper).or_else(|| self.subs.get(&upper));
                        if let Some(fi) = fi {
                            let fn_ptr = Val::new(format!("@{}", fi.ir_name), IrType::Ptr);
                            let a2 = self.compile_expr(fb, &call.args[0])?;
                            let b2 = self.compile_expr(fb, &call.args[1])?;
                            let ctrl = fb.call(&IrType::Ptr, "pb_pb_hwnd", &[a2, b2]);
                            fb.call_void("pb_register_callback_hwnd", &[ctrl, fn_ptr]);
                        } else {
                            eprintln!(
                                "warning: line {}: TOOLBAR ADD BUTTON: unknown sub `{}`",
                                call.line, upper
                            );
                        }
                    }
                }
                return Ok(());
            }
            "TOOLBAR_ADD_SEPARATOR" => {
                // batch 220: the official syntax requires all 6 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 6 {
                    return Err(pb::error::PbError::parser(
                        "TOOLBAR_ADD_SEPARATOR requires its arguments: TOOLBAR ADD SEPARATOR hDlg , ID , size& [, cmd& ] [ AT item& ]".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 6 {
                    let a = self.compile_expr(fb, &call.args[0])?;
                    let b = self.compile_expr(fb, &call.args[1])?;
                    let size = self.compile_expr(fb, &call.args[2])?;
                    let cmd = self.compile_expr(fb, &call.args[3])?;
                    let at = self.compile_expr(fb, &call.args[4])?;
                    fb.call(
                        &IrType::I64,
                        "pb_toolbar_add_separator",
                        &[a, b, size, cmd, at],
                    );
                }
                return Ok(());
            }
            "TOOLBAR_DELETE_BUTTON" => {
                // batch 220: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                        "TOOLBAR_DELETE_BUTTON requires its arguments: TOOLBAR DELETE BUTTON hDlg , id& , [ BYCMD ] item&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 4 {
                    let a = self.compile_expr(fb, &call.args[0])?;
                    let b = self.compile_expr(fb, &call.args[1])?;
                    let item = self.compile_expr(fb, &call.args[2])?;
                    let bycmd = self.compile_expr(fb, &call.args[3])?;
                    fb.call(
                        &IrType::I64,
                        "pb_toolbar_delete_button",
                        &[a, b, item, bycmd],
                    );
                }
                return Ok(());
            }
            "TOOLBAR_GET_STATE" => {
                // batch 220: the official syntax requires all 5 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 5 {
                    return Err(pb::error::PbError::parser(
                        "TOOLBAR_GET_STATE requires its arguments: TOOLBAR GET STATE hDlg , ID , [ BYCMD ] item& TO datav&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 5 {
                    let a = self.compile_expr(fb, &call.args[0])?;
                    let b = self.compile_expr(fb, &call.args[1])?;
                    let item = self.compile_expr(fb, &call.args[2])?;
                    let bycmd = self.compile_expr(fb, &call.args[4])?;
                    let val = fb.call(&IrType::I64, "pb_toolbar_get_state", &[a, b, item, bycmd]);
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[3]) {
                        let vv = self.convert_value(fb, &val, &ty, &pty);
                        fb.store(&vv, &ptr);
                    }
                }
                return Ok(());
            }
            "TOOLBAR_GET_COUNT" => {
                // batch 220: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                        "TOOLBAR_GET_COUNT requires its arguments: TOOLBAR GET COUNT hDlg , ID TO datav&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 4 {
                    let a = self.compile_expr(fb, &call.args[0])?;
                    let b = self.compile_expr(fb, &call.args[1])?;
                    let val = fb.call(&IrType::I64, "pb_toolbar_get_count", &[a, b]);
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[2]) {
                        let vv = self.convert_value(fb, &val, &ty, &pty);
                        fb.store(&vv, &ptr);
                    }
                }
                return Ok(());
            }
            "TOOLBAR_SET_IMAGELIST" => {
                // batch 220: the official syntax requires all 5 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 5 {
                    return Err(pb::error::PbError::parser(
                        "TOOLBAR_SET_IMAGELIST requires its arguments: TOOLBAR SET IMAGELIST hDlg , ID , hLst , ListType&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 5 {
                    let a = self.compile_expr(fb, &call.args[0])?;
                    let b = self.compile_expr(fb, &call.args[1])?;
                    let hlst = self.compile_expr(fb, &call.args[2])?;
                    let ltype = self.compile_expr(fb, &call.args[3])?;
                    fb.call(
                        &IrType::I64,
                        "pb_toolbar_set_imagelist",
                        &[a, b, hlst, ltype],
                    );
                }
                return Ok(());
            }
            "TOOLBAR_SET_STATE" => {
                // batch 220: the official syntax requires all 5 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 5 {
                    return Err(pb::error::PbError::parser(
                        "TOOLBAR_SET_STATE requires its arguments: TOOLBAR SET STATE hDlg , ID , [ BYCMD ] item& , state&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 5 {
                    let a = self.compile_expr(fb, &call.args[0])?;
                    let b = self.compile_expr(fb, &call.args[1])?;
                    let item = self.compile_expr(fb, &call.args[2])?;
                    let state = self.compile_expr(fb, &call.args[3])?;
                    let bycmd = self.compile_expr(fb, &call.args[4])?;
                    fb.call(
                        &IrType::I64,
                        "pb_toolbar_set_state",
                        &[a, b, item, state, bycmd],
                    );
                }
                return Ok(());
            }
            "STATUSBAR_SET_PARTS" => {
                // batch 220: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                        "STATUSBAR_SET_PARTS requires its arguments: STATUSBAR SET PARTS hDlg , id& , x& [ , x& ... ]".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 4 {
                    let a = self.compile_expr(fb, &call.args[0])?;
                    let b = self.compile_expr(fb, &call.args[1])?;
                    // Widths arrive as a variable-length tail; build a real
                    // [n x i32] stack array for SB_SETPARTS.
                    let n = call.args.len() - 3; // drop hDlg, id, trailing flag
                    let arr_ty = IrType::Array(n, Box::new(IrType::I32));
                    let buf = fb.alloca(&arr_ty);
                    for k in 0..n {
                        let v = self.compile_expr(fb, &call.args[2 + k])?;
                        let v32 = self.convert_value(fb, &v, &IrType::I32, &PbType::Long);
                        let idxv = fb.const_i64(k as i64);
                        let slot = fb.gep_array(&arr_ty, &buf, &idxv);
                        fb.store(&v32, &slot);
                    }
                    let cnt = fb.const_i64(n as i64);
                    fb.call_void("pb_statusbar_set_parts", &[a, b, buf, cnt]);
                }
                return Ok(());
            }
            "STATUSBAR_SET_TEXT" => {
                // batch 220: the official syntax requires all 6 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 6 {
                    return Err(pb::error::PbError::parser(
                        "STATUSBAR_SET_TEXT requires its arguments: STATUSBAR SET TEXT hDlg , id& , item& , style& , text$".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 6 {
                    let a = self.compile_expr(fb, &call.args[0])?;
                    let b = self.compile_expr(fb, &call.args[1])?;
                    let item = self.compile_expr(fb, &call.args[2])?;
                    let style = self.compile_expr(fb, &call.args[3])?;
                    let text = self.compile_expr(fb, &call.args[4])?;
                    fb.call(
                        &IrType::I64,
                        "pb_statusbar_set_text",
                        &[a, b, item, style, text],
                    );
                }
                return Ok(());
            }
            "LISTVIEW_INSERT_COLUMN" => {
                // batch 220: the official syntax requires all 6 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 6 {
                    return Err(pb::error::PbError::parser(
                        "LISTVIEW_INSERT_COLUMN requires its arguments: LISTVIEW INSERT COLUMN hDlg , id& , col& , StrExpr , ColWidth& , format&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 6 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let col = self.compile_expr(fb, &call.args[2])?;
                    let text = self.compile_str_payload(fb, &call.args[3])?;
                    let width = self.compile_expr(fb, &call.args[4])?;
                    let fmt = self.compile_expr(fb, &call.args[5])?;
                    fb.call_void(
                        "pb_listview_insert_column",
                        &[hd, id, col, text, width, fmt],
                    );
                }
                return Ok(());
            }
            "LISTVIEW_INSERT_ITEM" => {
                // batch 220: the official syntax requires all 5 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 5 {
                    return Err(pb::error::PbError::parser(
                        "LISTVIEW_INSERT_ITEM requires its arguments: LISTVIEW INSERT ITEM hDlg , id& , item& , image& , StrExpr".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 5 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let item = self.compile_expr(fb, &call.args[2])?;
                    let image = self.compile_expr(fb, &call.args[3])?;
                    let text = self.compile_str_payload(fb, &call.args[4])?;
                    fb.call_void("pb_listview_insert_item", &[hd, id, item, image, text]);
                }
                return Ok(());
            }
            "LISTVIEW_GET_COUNT" => {
                // batch 220: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                        "LISTVIEW_GET_COUNT requires its arguments: LISTVIEW GET COUNT hDlg , id& TO datav&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 3 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let n = fb.call(&IrType::I64, "pb_listview_get_count", &[hd, id]);
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[2]) {
                        let vv = self.convert_value(fb, &n, &ty, &pty);
                        fb.store(&vv, &ptr);
                    }
                }
                return Ok(());
            }
            "LISTVIEW_GET_TEXT" => {
                // batch 220: the official syntax requires all 5 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 5 {
                    return Err(pb::error::PbError::parser(
                        "LISTVIEW_GET_TEXT requires its arguments: LISTVIEW GET TEXT hDlg , id& , item& , col& TO txtv$".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 5 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let item = self.compile_expr(fb, &call.args[2])?;
                    let col = self.compile_expr(fb, &call.args[3])?;
                    let buf = fb.alloca(&IrType::Array(256, Box::new(IrType::I8)));
                    let bp = fb.gep_byte(&buf, &fb.const_i32(0));
                    fb.call_void(
                        "pb_listview_get_text",
                        &[hd, id, item, col, bp.clone(), fb.const_i32(256)],
                    );
                    if let Some((ptr, _, _)) = self.lvalue_ptr(fb, &call.args[4]) {
                        let len =
                            fb.call(&IrType::I32, "pb_str_cstr_len", std::slice::from_ref(&bp));
                        let bstr = fb.call(&IrType::Ptr, "pb_bstr_alloc", &[bp, len]);
                        fb.store(&bstr, &ptr);
                    }
                }
                return Ok(());
            }
            "LISTVIEW_SET_TEXT" => {
                // batch 220: the official syntax requires all 5 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 5 {
                    return Err(pb::error::PbError::parser(
                        "LISTVIEW_SET_TEXT requires its arguments: LISTVIEW SET TEXT hDlg , id& , item& , col& , StrExpr".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 5 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let item = self.compile_expr(fb, &call.args[2])?;
                    let col = self.compile_expr(fb, &call.args[3])?;
                    let text = self.compile_str_payload(fb, &call.args[4])?;
                    fb.call_void("pb_listview_set_text", &[hd, id, item, col, text]);
                }
                return Ok(());
            }
            "LISTVIEW_DELETE_ITEM" => {
                // batch 219: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                    "LISTVIEW_DELETE_ITEM requires its arguments: LISTVIEW DELETE ITEM hDlg , id& , item&"
                        .to_string(),
                    None,
                    call.line,
                ));
                }
                if call.args.len() >= 3 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let item = self.compile_expr(fb, &call.args[2])?;
                    fb.call_void("pb_listview_delete_item", &[hd, id, item]);
                }
                return Ok(());
            }
            "LISTVIEW_RESET" => {
                // batch 219: the official syntax requires all 2 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 2 {
                    return Err(pb::error::PbError::parser(
                        "LISTVIEW_RESET requires its arguments: LISTVIEW RESET hDlg , id&"
                            .to_string(),
                        None,
                        call.line,
                    ));
                }
                if call.args.len() >= 2 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    fb.call_void("pb_listview_reset", &[hd, id]);
                }
                return Ok(());
            }
            "LISTVIEW_DELETE_COLUMN" => {
                // batch 220: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                        "LISTVIEW_DELETE_COLUMN requires its arguments: LISTVIEW DELETE COLUMN hDlg , id& , col&".to_string(),
                        None,
                        call.line,
                    ));
                }

                // LISTVIEW DELETE COLUMN hDlg, ID&, Col&
                if call.args.len() >= 3 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let col = self.compile_expr(fb, &call.args[2])?;
                    fb.call_void("pb_listview_delete_column", &[hd, id, col]);
                }
                return Ok(());
            }
            "LISTVIEW_FIND" => {
                // batch 220: the official syntax requires all 5 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 5 {
                    return Err(pb::error::PbError::parser(
                        "LISTVIEW_FIND requires its arguments: LISTVIEW FIND hDlg , id& , item& , StrExpr TO datav&".to_string(),
                        None,
                        call.line,
                    ));
                }

                // LISTVIEW FIND hDlg, ID&, Item&, StrExpr TO datav&
                if call.args.len() >= 5 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let item = self.compile_expr(fb, &call.args[2])?;
                    let needle = self.compile_str_payload(fb, &call.args[3])?;
                    let v = fb.call(
                        &IrType::I64,
                        "pb_listview_find",
                        &[hd, id, item, needle, fb.const_i32(0)],
                    );
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[4]) {
                        let vv = self.convert_value(fb, &v, &ty, &pty);
                        fb.store(&vv, &ptr);
                    }
                }
                return Ok(());
            }
            "LISTVIEW_FIND_EXACT" => {
                // batch 220: the official syntax requires all 5 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 5 {
                    return Err(pb::error::PbError::parser(
                        "LISTVIEW_FIND_EXACT requires its arguments: LISTVIEW FIND EXACT hDlg , id& , item& , StrExpr TO datav&".to_string(),
                        None,
                        call.line,
                    ));
                }

                // LISTVIEW FIND EXACT hDlg, ID&, Item&, StrExpr TO datav&
                // Same runtime entry as LISTVIEW FIND; operand 5 selects
                // the exact-match comparison instead of the prefix match.
                if call.args.len() >= 5 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let item = self.compile_expr(fb, &call.args[2])?;
                    let needle = self.compile_str_payload(fb, &call.args[3])?;
                    let v = fb.call(
                        &IrType::I64,
                        "pb_listview_find",
                        &[hd, id, item, needle, fb.const_i32(1)],
                    );
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[4]) {
                        let vv = self.convert_value(fb, &v, &ty, &pty);
                        fb.store(&vv, &ptr);
                    }
                }
                return Ok(());
            }
            "LISTVIEW_FIT_CONTENT" => {
                // LISTVIEW FIT CONTENT hDlg, ID&, Col&
                // batch 219: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                    "LISTVIEW_FIT_CONTENT requires its arguments: LISTVIEW FIT CONTENT hDlg , id& , col&"
                        .to_string(),
                    None,
                    call.line,
                ));
                }
                if call.args.len() >= 3 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let col = self.compile_expr(fb, &call.args[2])?;
                    fb.call_void("pb_listview_fit_content", &[hd, id, col]);
                }
                return Ok(());
            }
            "LISTVIEW_FIT_HEADER" => {
                // LISTVIEW FIT HEADER hDlg, ID&, Col&
                // batch 219: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                    "LISTVIEW_FIT_HEADER requires its arguments: LISTVIEW FIT HEADER hDlg , id& , col&"
                        .to_string(),
                    None,
                    call.line,
                ));
                }
                if call.args.len() >= 3 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let col = self.compile_expr(fb, &call.args[2])?;
                    fb.call_void("pb_listview_fit_header", &[hd, id, col]);
                }
                return Ok(());
            }
            "LISTVIEW_GET_COLUMN" => {
                // batch 220: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                        "LISTVIEW_GET_COLUMN requires its arguments: LISTVIEW GET COLUMN hDlg , id& , col& TO datav&".to_string(),
                        None,
                        call.line,
                    ));
                }

                // LISTVIEW GET COLUMN hDlg, ID&, Col& TO datav&
                if call.args.len() >= 4 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let col = self.compile_expr(fb, &call.args[2])?;
                    let v = fb.call(&IrType::I64, "pb_listview_get_column", &[hd, id, col]);
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[3]) {
                        let vv = self.convert_value(fb, &v, &ty, &pty);
                        fb.store(&vv, &ptr);
                    }
                }
                return Ok(());
            }
            "LISTVIEW_SET_COLUMN" => {
                // LISTVIEW SET COLUMN hDlg, ID&, Col&, NumExpr
                // batch 219: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                    "LISTVIEW_SET_COLUMN requires its arguments: LISTVIEW SET COLUMN hDlg , id& , col& ,"
                        .to_string(),
                    None,
                    call.line,
                ));
                }
                if call.args.len() >= 4 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let col = self.compile_expr(fb, &call.args[2])?;
                    let width = self.compile_expr(fb, &call.args[3])?;
                    fb.call_void("pb_listview_set_column", &[hd, id, col, width]);
                }
                return Ok(());
            }
            "LISTVIEW_GET_HEADER" => {
                // batch 220: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                        "LISTVIEW_GET_HEADER requires its arguments: LISTVIEW GET HEADER hDlg , id& , col& TO txtv$".to_string(),
                        None,
                        call.line,
                    ));
                }

                // LISTVIEW GET HEADER hDlg, ID&, Col& TO txtv$
                if call.args.len() >= 4 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let col = self.compile_expr(fb, &call.args[2])?;
                    let buf = fb.alloca(&IrType::Array(256, Box::new(IrType::I8)));
                    let bp = fb.gep_byte(&buf, &fb.const_i32(0));
                    fb.call_void(
                        "pb_listview_get_header",
                        &[hd, id, col, bp.clone(), fb.const_i32(256)],
                    );
                    if let Some((ptr, _, _)) = self.lvalue_ptr(fb, &call.args[3]) {
                        let len =
                            fb.call(&IrType::I32, "pb_str_cstr_len", std::slice::from_ref(&bp));
                        let bstr = fb.call(&IrType::Ptr, "pb_bstr_alloc", &[bp, len]);
                        fb.store(&bstr, &ptr);
                    }
                }
                return Ok(());
            }
            "LISTVIEW_SET_HEADER" => {
                // LISTVIEW SET HEADER hDlg, ID&, Col&, StrExpr
                // batch 219: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                    "LISTVIEW_SET_HEADER requires its arguments: LISTVIEW SET HEADER hDlg , id& , col& ,"
                        .to_string(),
                    None,
                    call.line,
                ));
                }
                if call.args.len() >= 4 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let col = self.compile_expr(fb, &call.args[2])?;
                    let text = self.compile_str_payload(fb, &call.args[3])?;
                    fb.call_void("pb_listview_set_header", &[hd, id, col, text]);
                }
                return Ok(());
            }
            "LISTVIEW_GET_HEADERID" => {
                // batch 220: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                        "LISTVIEW_GET_HEADERID requires its arguments: LISTVIEW GET HEADERID hDlg , id& TO hLV , idv&".to_string(),
                        None,
                        call.line,
                    ));
                }

                // LISTVIEW GET HEADERID hDlg, ID& TO hLV, idv&
                // Two out-parameters, so both travel through I64 slots
                // that the runtime fills and we convert on the way back.
                if call.args.len() >= 4 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let hlv_tmp = fb.alloca(&IrType::I64);
                    // The runtime writes a `long` here, so the slot must be
                    // I32: an I64 slot would leave the high half uninitialised
                    // and the load below would read garbage.
                    let hid_tmp = fb.alloca(&IrType::I32);
                    fb.call_void(
                        "pb_listview_get_headerid",
                        &[hd, id, hlv_tmp.clone(), hid_tmp.clone()],
                    );
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[2]) {
                        let v = fb.load(&IrType::I64, &hlv_tmp);
                        let vv = self.convert_value(fb, &v, &ty, &pty);
                        fb.store(&vv, &ptr);
                    }
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[3]) {
                        let v = fb.load(&IrType::I32, &hid_tmp);
                        let vv = self.convert_value(fb, &v, &ty, &pty);
                        fb.store(&vv, &ptr);
                    }
                }
                return Ok(());
            }
            "LISTVIEW_GET_MODE" => {
                // batch 220: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                        "LISTVIEW_GET_MODE requires its arguments: LISTVIEW GET MODE hDlg , id& TO datav&".to_string(),
                        None,
                        call.line,
                    ));
                }

                // LISTVIEW GET MODE hDlg, ID& TO datav&
                if call.args.len() >= 3 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let v = fb.call(&IrType::I64, "pb_listview_get_mode", &[hd, id]);
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[2]) {
                        let vv = self.convert_value(fb, &v, &ty, &pty);
                        fb.store(&vv, &ptr);
                    }
                }
                return Ok(());
            }
            "LISTVIEW_SET_MODE" => {
                // LISTVIEW SET MODE hDlg, ID&, NumExpr
                // batch 219: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                    "LISTVIEW_SET_MODE requires its arguments: LISTVIEW SET MODE hDlg , id& , NumExpr"
                        .to_string(),
                    None,
                    call.line,
                ));
                }
                if call.args.len() >= 3 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let mode = self.compile_expr(fb, &call.args[2])?;
                    fb.call_void("pb_listview_set_mode", &[hd, id, mode]);
                }
                return Ok(());
            }
            "LISTVIEW_GET_SELCOUNT" => {
                // batch 220: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                        "LISTVIEW_GET_SELCOUNT requires its arguments: LISTVIEW GET SELCOUNT hDlg , id& TO datav&".to_string(),
                        None,
                        call.line,
                    ));
                }

                // LISTVIEW GET SELCOUNT hDlg, ID& TO datav&
                if call.args.len() >= 3 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let v = fb.call(&IrType::I64, "pb_listview_get_selcount", &[hd, id]);
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[2]) {
                        let vv = self.convert_value(fb, &v, &ty, &pty);
                        fb.store(&vv, &ptr);
                    }
                }
                return Ok(());
            }
            "LISTVIEW_GET_SELECT" => {
                // batch 220: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                        "LISTVIEW_GET_SELECT requires its arguments: LISTVIEW GET SELECT hDlg , id& [, item& ] TO datav&".to_string(),
                        None,
                        call.line,
                    ));
                }

                // LISTVIEW GET SELECT hDlg, ID& [, Item&] TO datav&
                // The parser always emits Item&, defaulting it to 1.
                if call.args.len() >= 4 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let start = self.compile_expr(fb, &call.args[2])?;
                    let v = fb.call(&IrType::I64, "pb_listview_get_select", &[hd, id, start]);
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[3]) {
                        let vv = self.convert_value(fb, &v, &ty, &pty);
                        fb.store(&vv, &ptr);
                    }
                }
                return Ok(());
            }
            "LISTVIEW_GET_STATE" => {
                // batch 220: the official syntax requires all 5 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 5 {
                    return Err(pb::error::PbError::parser(
                        "LISTVIEW_GET_STATE requires its arguments: LISTVIEW GET STATE hDlg , id& , item& , col& TO datav&".to_string(),
                        None,
                        call.line,
                    ));
                }

                // LISTVIEW GET STATE hDlg, ID&, Item&, Col& TO datav&
                if call.args.len() >= 5 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let item = self.compile_expr(fb, &call.args[2])?;
                    let col = self.compile_expr(fb, &call.args[3])?;
                    let v = fb.call(&IrType::I64, "pb_listview_get_state", &[hd, id, item, col]);
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[4]) {
                        let vv = self.convert_value(fb, &v, &ty, &pty);
                        fb.store(&vv, &ptr);
                    }
                }
                return Ok(());
            }
            "LISTVIEW_SELECT" => {
                // batch 220: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                        "LISTVIEW_SELECT requires its arguments: LISTVIEW SELECT hDlg , id& , item& [, col& ]".to_string(),
                        None,
                        call.line,
                    ));
                }

                // LISTVIEW SELECT hDlg, ID&, Item& [, Col&]
                if call.args.len() >= 4 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let item = self.compile_expr(fb, &call.args[2])?;
                    let col = self.compile_expr(fb, &call.args[3])?;
                    fb.call_void("pb_listview_select", &[hd, id, item, col]);
                }
                return Ok(());
            }
            "LISTVIEW_UNSELECT" => {
                // LISTVIEW UNSELECT hDlg, ID&, Item& [, Col&]
                // batch 219: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                    "LISTVIEW_UNSELECT requires its arguments: LISTVIEW UNSELECT hDlg , id& , item& ,"
                        .to_string(),
                    None,
                    call.line,
                ));
                }
                if call.args.len() >= 4 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let item = self.compile_expr(fb, &call.args[2])?;
                    let col = self.compile_expr(fb, &call.args[3])?;
                    fb.call_void("pb_listview_unselect", &[hd, id, item, col]);
                }
                return Ok(());
            }
            "LISTVIEW_GET_STYLEXX" => {
                // batch 220: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                        "LISTVIEW_GET_STYLEXX requires its arguments: LISTVIEW GET STYLEXX hDlg , id& TO datav&".to_string(),
                        None,
                        call.line,
                    ));
                }

                // LISTVIEW GET STYLEXX hDlg, ID& TO datav&
                if call.args.len() >= 3 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let v = fb.call(&IrType::I64, "pb_listview_get_stylexx", &[hd, id]);
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[2]) {
                        let vv = self.convert_value(fb, &v, &ty, &pty);
                        fb.store(&vv, &ptr);
                    }
                }
                return Ok(());
            }
            "LISTVIEW_SET_STYLEXX" => {
                // LISTVIEW SET STYLEXX hDlg, ID&, NumExpr
                // batch 219: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                    "LISTVIEW_SET_STYLEXX requires its arguments: LISTVIEW SET STYLEXX hDlg , id& , NumExpr"
                        .to_string(),
                    None,
                    call.line,
                ));
                }
                if call.args.len() >= 3 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let style = self.compile_expr(fb, &call.args[2])?;
                    fb.call_void("pb_listview_set_stylexx", &[hd, id, style]);
                }
                return Ok(());
            }
            "LISTVIEW_GET_USER" => {
                // batch 220: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                        "LISTVIEW_GET_USER requires its arguments: LISTVIEW GET USER hDlg , id& , item& TO datav&".to_string(),
                        None,
                        call.line,
                    ));
                }

                // LISTVIEW GET USER hDlg, ID&, Item& TO datav&
                if call.args.len() >= 4 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let item = self.compile_expr(fb, &call.args[2])?;
                    let v = fb.call(&IrType::I64, "pb_listview_get_user", &[hd, id, item]);
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[3]) {
                        let vv = self.convert_value(fb, &v, &ty, &pty);
                        fb.store(&vv, &ptr);
                    }
                }
                return Ok(());
            }
            "LISTVIEW_SET_USER" => {
                // LISTVIEW SET USER hDlg, ID&, Item&, NumExpr
                // batch 219: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                    "LISTVIEW_SET_USER requires its arguments: LISTVIEW SET USER hDlg , id& , item& ,"
                        .to_string(),
                    None,
                    call.line,
                ));
                }
                if call.args.len() >= 4 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let item = self.compile_expr(fb, &call.args[2])?;
                    let value = self.compile_expr(fb, &call.args[3])?;
                    fb.call_void("pb_listview_set_user", &[hd, id, item, value]);
                }
                return Ok(());
            }
            "LISTVIEW_SET_IMAGE" => {
                // LISTVIEW SET IMAGE hDlg, ID&, Item&, NumExpr
                // batch 219: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                    "LISTVIEW_SET_IMAGE requires its arguments: LISTVIEW SET IMAGE hDlg , id& , item& ,"
                        .to_string(),
                    None,
                    call.line,
                ));
                }
                if call.args.len() >= 4 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let item = self.compile_expr(fb, &call.args[2])?;
                    let image = self.compile_expr(fb, &call.args[3])?;
                    fb.call_void("pb_listview_set_image", &[hd, id, item, image]);
                }
                return Ok(());
            }
            "LISTVIEW_SET_IMAGE2" => {
                // LISTVIEW SET IMAGE2 hDlg, ID&, Item&, NumExpr
                // batch 219: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                    "LISTVIEW_SET_IMAGE2 requires its arguments: LISTVIEW SET IMAGE2 hDlg , id& , item& ,"
                        .to_string(),
                    None,
                    call.line,
                ));
                }
                if call.args.len() >= 4 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let item = self.compile_expr(fb, &call.args[2])?;
                    let image = self.compile_expr(fb, &call.args[3])?;
                    fb.call_void("pb_listview_set_image2", &[hd, id, item, image]);
                }
                return Ok(());
            }
            "LISTVIEW_SET_OVERLAY" => {
                // LISTVIEW SET OVERLAY hDlg, ID&, Item&, NumExpr
                // batch 219: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                    "LISTVIEW_SET_OVERLAY requires its arguments: LISTVIEW SET OVERLAY hDlg , id& , item& ,"
                        .to_string(),
                    None,
                    call.line,
                ));
                }
                if call.args.len() >= 4 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let item = self.compile_expr(fb, &call.args[2])?;
                    let overlay = self.compile_expr(fb, &call.args[3])?;
                    fb.call_void("pb_listview_set_overlay", &[hd, id, item, overlay]);
                }
                return Ok(());
            }
            "LISTVIEW_SET_IMAGELIST" => {
                // LISTVIEW SET IMAGELIST hDlg, ID&, hLst, NumExpr
                // batch 219: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                    "LISTVIEW_SET_IMAGELIST requires its arguments: LISTVIEW SET IMAGELIST hDlg , id& , hLst ,"
                        .to_string(),
                    None,
                    call.line,
                ));
                }
                if call.args.len() >= 4 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let mut hlst = self.compile_expr(fb, &call.args[2])?;
                    if hlst.ty != IrType::Ptr {
                        hlst = fb.inttoptr(&hlst);
                    }
                    let which = self.compile_expr(fb, &call.args[3])?;
                    fb.call_void("pb_listview_set_imagelist", &[hd, id, hlst, which]);
                }
                return Ok(());
            }
            "LISTVIEW_VISIBLE" => {
                // LISTVIEW VISIBLE hDlg, ID&, Item&
                // batch 219: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                        "LISTVIEW_VISIBLE requires its arguments: LISTVIEW VISIBLE hDlg , id& ,"
                            .to_string(),
                        None,
                        call.line,
                    ));
                }
                if call.args.len() >= 3 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let item = self.compile_expr(fb, &call.args[2])?;
                    fb.call_void("pb_listview_visible", &[hd, id, item]);
                }
                return Ok(());
            }
            "LISTVIEW_SORT" => {
                // LISTVIEW SORT hDlg, ID&, Col& [, options...]
                // The parser folds the option keywords into one bitmask
                // operand (see parse_listview_sort_options); the encoding
                // is shared with PB_LVSORT_* in pb_runtime.c.
                if call.args.len() >= 4 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let col = self.compile_expr(fb, &call.args[2])?;
                    let mode = self.compile_expr(fb, &call.args[3])?;
                    fb.call_void("pb_listview_sort", &[hd, id, col, mode]);
                }
                return Ok(());
            }
            "TREEVIEW_INSERT_ITEM" => {
                // batch 220: the official syntax requires all 8 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 8 {
                    return Err(pb::error::PbError::parser(
                        "TREEVIEW_INSERT_ITEM requires its arguments: TREEVIEW INSERT ITEM hDlg , id& , hPrnt , hIAftr , image& , simage& , txt$ TO hItem".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 8 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let mut hparent = self.compile_expr(fb, &call.args[2])?;
                    if hparent.ty != IrType::Ptr {
                        hparent = fb.inttoptr(&hparent);
                    }
                    let mut hafter = self.compile_expr(fb, &call.args[3])?;
                    if hafter.ty != IrType::Ptr {
                        hafter = fb.inttoptr(&hafter);
                    }
                    let image = self.compile_expr(fb, &call.args[4])?;
                    let simage = self.compile_expr(fb, &call.args[5])?;
                    let text = self.compile_str_payload(fb, &call.args[6])?;
                    let outp = fb.alloca(&IrType::Ptr);
                    fb.call_void(
                        "pb_treeview_insert_item",
                        &[hd, id, hparent, hafter, image, simage, text, outp.clone()],
                    );
                    if let Some((ptr, ty, _)) = self.lvalue_ptr(fb, &call.args[7]) {
                        let rv = fb.load(&IrType::Ptr, &outp);
                        let rv_i = fb.ptrtoint64(&rv);
                        let store_val = match &ty {
                            IrType::I32 => self.to_i32(fb, &rv_i),
                            _ => rv_i,
                        };
                        fb.store(&store_val, &ptr);
                    }
                }
                return Ok(());
            }
            "TREEVIEW_GET_COUNT" => {
                // batch 220: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                        "TREEVIEW_GET_COUNT requires its arguments: TREEVIEW GET COUNT hDlg , id& TO datav&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 3 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let n = fb.call(&IrType::I64, "pb_treeview_get_count", &[hd, id]);
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[2]) {
                        let vv = self.convert_value(fb, &n, &ty, &pty);
                        fb.store(&vv, &ptr);
                    }
                }
                return Ok(());
            }
            "TREEVIEW_GET_TEXT" => {
                // batch 220: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                        "TREEVIEW_GET_TEXT requires its arguments: TREEVIEW GET TEXT hDlg , id& , hItem TO txtv$".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 4 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let mut hitem = self.compile_expr(fb, &call.args[2])?;
                    if hitem.ty != IrType::Ptr {
                        hitem = fb.inttoptr(&hitem);
                    }
                    let buf = fb.alloca(&IrType::Array(256, Box::new(IrType::I8)));
                    let bp = fb.gep_byte(&buf, &fb.const_i32(0));
                    fb.call_void(
                        "pb_treeview_get_text",
                        &[hd, id, hitem, bp.clone(), fb.const_i32(256)],
                    );
                    if let Some((ptr, _, _)) = self.lvalue_ptr(fb, &call.args[3]) {
                        let len =
                            fb.call(&IrType::I32, "pb_str_cstr_len", std::slice::from_ref(&bp));
                        let bstr = fb.call(&IrType::Ptr, "pb_bstr_alloc", &[bp, len]);
                        fb.store(&bstr, &ptr);
                    }
                }
                return Ok(());
            }
            // ---- TREEVIEW attribute getters (batch 172) ----
            // All ten share one runtime entry point; the trailing `which`
            // selector (PB_TV_ATTR_* in pb_runtime.c; the numbers are repeated
            // here because codegen can only emit an integer literal - change
            // the two lists together) picks the field. The parser normalises
            // the operand list, so ROOT and SELECT arrive with three operands
            // and every other getter with four. PB promises 0 for "no such
            // item", which is exactly what the runtime returns.
            "TREEVIEW_GET_BOLD"
            | "TREEVIEW_GET_CHECK"
            | "TREEVIEW_GET_CHILD"
            | "TREEVIEW_GET_EXPANDED"
            | "TREEVIEW_GET_NEXT"
            | "TREEVIEW_GET_PARENT"
            | "TREEVIEW_GET_PREVIOUS"
            | "TREEVIEW_GET_ROOT"
            | "TREEVIEW_GET_SELECT"
            | "TREEVIEW_GET_USER" => {
                if call.args.len() >= 3 {
                    let which: i32 = match call.name.as_str() {
                        "TREEVIEW_GET_BOLD" => 1,
                        "TREEVIEW_GET_CHECK" => 2,
                        "TREEVIEW_GET_CHILD" => 3,
                        "TREEVIEW_GET_EXPANDED" => 4,
                        "TREEVIEW_GET_NEXT" => 5,
                        "TREEVIEW_GET_PARENT" => 6,
                        "TREEVIEW_GET_PREVIOUS" => 7,
                        "TREEVIEW_GET_ROOT" => 8,
                        "TREEVIEW_GET_SELECT" => 9,
                        _ => 10,
                    };
                    let has_item = call.args.len() >= 4;
                    let target_idx = if has_item { 3 } else { 2 };
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let hitem = if has_item {
                        let mut v = self.compile_expr(fb, &call.args[2])?;
                        if v.ty != IrType::Ptr {
                            v = fb.inttoptr(&v);
                        }
                        v
                    } else {
                        // ROOT / SELECT ignore the handle; pass a null pointer.
                        let zero = fb.const_i32(0);
                        fb.inttoptr(&zero)
                    };
                    let n = fb.call(
                        &IrType::I64,
                        "pb_treeview_get_attr",
                        &[hd, id, hitem, fb.const_i32(which)],
                    );
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[target_idx]) {
                        let vv = self.convert_value(fb, &n, &ty, &pty);
                        fb.store(&vv, &ptr);
                    }
                }
                return Ok(());
            }
            "TREEVIEW_SELECT" => {
                // batch 219: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                        "TREEVIEW_SELECT requires its arguments: TREEVIEW SELECT hDlg , id& ,"
                            .to_string(),
                        None,
                        call.line,
                    ));
                }
                if call.args.len() >= 3 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let mut hitem = self.compile_expr(fb, &call.args[2])?;
                    if hitem.ty != IrType::Ptr {
                        hitem = fb.inttoptr(&hitem);
                    }
                    fb.call_void("pb_treeview_select", &[hd, id, hitem]);
                }
                return Ok(());
            }
            "TREEVIEW_UNSELECT" => {
                // batch 219: the official syntax requires all 2 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 2 {
                    return Err(pb::error::PbError::parser(
                        "TREEVIEW_UNSELECT requires its arguments: TREEVIEW UNSELECT hDlg , id&"
                            .to_string(),
                        None,
                        call.line,
                    ));
                }
                if call.args.len() >= 2 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    fb.call_void("pb_treeview_unselect", &[hd, id]);
                }
                return Ok(());
            }
            "TREEVIEW_SET_BOLD"
            | "TREEVIEW_SET_CHECK"
            | "TREEVIEW_SET_EXPANDED"
            | "TREEVIEW_SET_USER" => {
                if call.args.len() >= 4 {
                    let which: i32 = match call.name.as_str() {
                        "TREEVIEW_SET_BOLD" => 1,
                        "TREEVIEW_SET_CHECK" => 2,
                        "TREEVIEW_SET_EXPANDED" => 4,
                        _ => 10,
                    };
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let mut hitem = self.compile_expr(fb, &call.args[2])?;
                    if hitem.ty != IrType::Ptr {
                        hitem = fb.inttoptr(&hitem);
                    }
                    let flag = self.compile_expr(fb, &call.args[3])?;
                    let flag64 = self.to_i64(fb, &flag);
                    fb.call_void(
                        "pb_treeview_set_attr",
                        &[hd, id, hitem, fb.const_i32(which), flag64],
                    );
                }
                return Ok(());
            }
            "TREEVIEW_SET_TEXT" => {
                // batch 219: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                    "TREEVIEW_SET_TEXT requires its arguments: TREEVIEW SET TEXT hDlg , id& , hItem ,"
                        .to_string(),
                    None,
                    call.line,
                ));
                }
                if call.args.len() >= 4 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let mut hitem = self.compile_expr(fb, &call.args[2])?;
                    if hitem.ty != IrType::Ptr {
                        hitem = fb.inttoptr(&hitem);
                    }
                    let text = self.compile_str_payload(fb, &call.args[3])?;
                    fb.call_void("pb_treeview_set_text", &[hd, id, hitem, text]);
                }
                return Ok(());
            }
            "TREEVIEW_SET_IMAGELIST" => {
                // batch 219: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                    "TREEVIEW_SET_IMAGELIST requires its arguments: TREEVIEW SET IMAGELIST hDlg , id& , hLst"
                        .to_string(),
                    None,
                    call.line,
                ));
                }
                if call.args.len() >= 3 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let mut hl = self.compile_expr(fb, &call.args[2])?;
                    if hl.ty != IrType::Ptr {
                        hl = fb.inttoptr(&hl);
                    }
                    fb.call_void("pb_treeview_set_imagelist", &[hd, id, hl]);
                }
                return Ok(());
            }
            "TREEVIEW_DELETE" => {
                // batch 220: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                        "TREEVIEW_DELETE requires its arguments: TREEVIEW DELETE hDlg , id& , hItem".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 3 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let mut hitem = self.compile_expr(fb, &call.args[2])?;
                    if hitem.ty != IrType::Ptr {
                        hitem = fb.inttoptr(&hitem);
                    }
                    fb.call_void("pb_treeview_delete", &[hd, id, hitem]);
                }
                return Ok(());
            }
            "TREEVIEW_RESET" => {
                // batch 219: the official syntax requires all 2 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 2 {
                    return Err(pb::error::PbError::parser(
                        "TREEVIEW_RESET requires its arguments: TREEVIEW RESET hDlg , id&"
                            .to_string(),
                        None,
                        call.line,
                    ));
                }
                if call.args.len() >= 2 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    fb.call_void("pb_treeview_reset", &[hd, id]);
                }
                return Ok(());
            }
            "DIALOG_NEW" => {
                // batch 220: the official syntax requires 9 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 9 {
                    return Err(pb::error::PbError::parser(
                        "DIALOG_NEW requires its arguments: DIALOG NEW hParent , title , x , y , nWide& , nHigh& [, style& ] [, exstyle& ] TO hDlg".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 9 {
                    let _parent = self.compile_expr(fb, &call.args[0])?;
                    let title = self.compile_expr(fb, &call.args[1])?;
                    let x = self.compile_expr(fb, &call.args[2])?;
                    let y = self.compile_expr(fb, &call.args[3])?;
                    let w = self.compile_expr(fb, &call.args[4])?;
                    let h = self.compile_expr(fb, &call.args[5])?;
                    let style = self.compile_expr(fb, &call.args[6])?;
                    let exstyle = self.compile_expr(fb, &call.args[7])?;
                    // batch 242: arg[9] = unit flag (0 UNITS/DLU default, 1 PIXELS),
                    // added by the parser for `[PIXELS, | UNITS,]`; older IR stays 9.
                    let units = if call.args.len() >= 10 {
                        self.compile_expr(fb, &call.args[9])?
                    } else {
                        fb.const_i64(0)
                    };
                    let hc = fb.call(
                        &IrType::Ptr,
                        "pb_window_new_dlg",
                        &[title, x, y, w, h, style, exstyle, units],
                    );
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[8]) {
                        let hc_i = fb.ptrtoint64(&hc);
                        let hc_i_v = self.convert_value(fb, &hc_i, &ty, &pty);
                        fb.store(&hc_i_v, &ptr);
                    }
                }
                return Ok(());
            }
            "DIALOG_SHOW_MODAL" => {
                if call.args.len() >= 2 {
                    let hd = self.compile_expr(fb, &call.args[0])?;
                    let hd_ptr = if hd.ty != IrType::Ptr {
                        fb.inttoptr(&hd)
                    } else {
                        hd
                    };
                    if let Expr::Variable(name) = &call.args[1] {
                        let upper = name.to_uppercase();
                        let fi = self.functions.get(&upper).or_else(|| self.subs.get(&upper));
                        if let Some(fi) = fi {
                            let fn_ptr = Val::new(format!("@{}", fi.ir_name), IrType::Ptr);
                            fb.call_void("pb_register_dialog_cb", &[fn_ptr]);
                            // batch 243: pass the dialog handle so the modal loop can
                            // route Enter/Tab via IsDialogMessage (BS_DEFPUSHBUTTON).
                            fb.call_void("pb_show_modal_to", &[hd_ptr]);
                        } else {
                            eprintln!("warning: DIALOG SHOW MODAL: unknown sub '{}'", upper);
                        }
                    }
                }
                return Ok(());
            }
            "DIALOG_SHOW_MODAL_TO" => {
                // DIALOG SHOW MODAL hDlg TO result   (official PB form)
                // batch 238: HelloDDT (C:\PBWin10\samples\DDT) - the modal
                // result set by DIALOG END is written back to the target
                // variable via pb_show_modal_to (returns pb_modal_result).
                if call.args.len() >= 2 {
                    let hd = self.compile_expr(fb, &call.args[0])?;
                    let hd_ptr = if hd.ty != IrType::Ptr {
                        fb.inttoptr(&hd)
                    } else {
                        hd
                    };
                    let res = fb.call(&IrType::I32, "pb_show_modal_to", &[hd_ptr]);
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[1]) {
                        let cv = self.convert_value(fb, &res, &ty, &pty);
                        fb.store(&cv, &ptr);
                    }
                }
                return Ok(());
            }
            "DIALOG_SET_TEXT" => {
                // batch 219: the official syntax requires all 2 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 2 {
                    return Err(pb::error::PbError::parser(
                        "DIALOG_SET_TEXT requires its arguments: DIALOG SET TEXT hDlg ,"
                            .to_string(),
                        None,
                        call.line,
                    ));
                }
                if call.args.len() >= 2 {
                    let hdlg = self.compile_expr(fb, &call.args[0])?;
                    let text = self.compile_expr(fb, &call.args[1])?;
                    let hdlg64 = fb.inttoptr(&hdlg);
                    fb.call_void("pb_dialog_set_text", &[hdlg64, text]);
                }
                return Ok(());
            }
            "DIALOG_DOEVENTS" => {
                fb.call_void("pb_dialog_doevents", &[]);
                return Ok(());
            }
            "DIALOG_GET_SIZE" => {
                // batch 220: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                        "DIALOG_GET_SIZE requires its arguments: DIALOG GET SIZE hDlg TO x& , y&"
                            .to_string(),
                        None,
                        call.line,
                    ));
                }

                // DIALOG GET SIZE hDlg TO x&, y&   (official statement)
                if call.args.len() >= 3 {
                    let hd = self.compile_expr(fb, &call.args[0])?;
                    let hdlg64 = fb.inttoptr(&hd);
                    let pw = fb.alloca(&IrType::I64);
                    let ph = fb.alloca(&IrType::I64);
                    fb.call_void("pb_dialog_get_size", &[hdlg64, pw.clone(), ph.clone()]);
                    for (argi, slot) in [(1usize, &pw), (2usize, &ph)] {
                        if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[argi]) {
                            let v = fb.load(&IrType::I64, slot);
                            let cv = self.convert_value(fb, &v, &ty, &pty);
                            fb.store(&cv, &ptr);
                        }
                    }
                }
                return Ok(());
            }
            "DIALOG_SET_SIZE" => {
                // DIALOG SET SIZE hDlg, nWide&, nHigh&   (official statement)
                // batch 219: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                    "DIALOG_SET_SIZE requires its arguments: DIALOG SET SIZE hDlg , nWide& , nHigh&"
                        .to_string(),
                    None,
                    call.line,
                ));
                }
                if call.args.len() >= 3 {
                    let hd = self.compile_expr(fb, &call.args[0])?;
                    let hdlg64 = fb.inttoptr(&hd);
                    let w = self.compile_expr(fb, &call.args[1])?;
                    let w64 = self.convert_value(fb, &w, &IrType::I64, &PbType::Quad);
                    let h = self.compile_expr(fb, &call.args[2])?;
                    let h64 = self.convert_value(fb, &h, &IrType::I64, &PbType::Quad);
                    fb.call_void("pb_dialog_set_size", &[hdlg64, w64, h64]);
                }
                return Ok(());
            }
            "DIALOG_ENABLE" => {
                // batch 220: the official syntax requires all 2 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 2 {
                    return Err(pb::error::PbError::parser(
                        "DIALOG_ENABLE requires its arguments: DIALOG ENABLE hDlg".to_string(),
                        None,
                        call.line,
                    ));
                }

                // DIALOG ENABLE hDlg / DIALOG DISABLE hDlg
                if call.args.len() >= 2 {
                    let hd = self.compile_expr(fb, &call.args[0])?;
                    let hdlg64 = fb.inttoptr(&hd);
                    let on = self.compile_expr(fb, &call.args[1])?;
                    let on32 = self.to_i32(fb, &on);
                    fb.call_void("pb_dialog_enable", &[hdlg64, on32]);
                }
                return Ok(());
            }
            "DIALOG_SHOW" => {
                // batch 220: the official syntax requires all 2 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 2 {
                    return Err(pb::error::PbError::parser(
                        "DIALOG_SHOW requires its arguments: DIALOG SHOW MODELESS hDlg".to_string(),
                        None,
                        call.line,
                    ));
                }

                // DIALOG HIDE / NORMALIZE / MINIMIZE / MAXIMIZE / SHOW MODELESS
                if call.args.len() >= 2 {
                    let hd = self.compile_expr(fb, &call.args[0])?;
                    let hdlg64 = fb.inttoptr(&hd);
                    let cmd = self.compile_expr(fb, &call.args[1])?;
                    let cmd32 = self.to_i32(fb, &cmd);
                    fb.call_void("pb_dialog_show", &[hdlg64, cmd32]);
                }
                return Ok(());
            }
            "DIALOG_STABILIZE" => {
                // batch 220: the official syntax requires all 2 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 2 {
                    return Err(pb::error::PbError::parser(
                        "DIALOG_STABILIZE requires its arguments: DIALOG STABILIZE hDlg"
                            .to_string(),
                        None,
                        call.line,
                    ));
                }

                // DIALOG STABILIZE hDlg / DIALOG NONSTABLE hDlg
                if call.args.len() >= 2 {
                    let hd = self.compile_expr(fb, &call.args[0])?;
                    let hdlg64 = fb.inttoptr(&hd);
                    let st = self.compile_expr(fb, &call.args[1])?;
                    let st32 = self.to_i32(fb, &st);
                    fb.call_void("pb_dialog_stabilize", &[hdlg64, st32]);
                }
                return Ok(());
            }
            "DIALOG_REDRAW" => {
                // DIALOG REDRAW hDlg
                if let Some(a0) = call.args.first() {
                    let hd = self.compile_expr(fb, a0)?;
                    let hdlg64 = fb.inttoptr(&hd);
                    fb.call_void("pb_dialog_redraw", &[hdlg64]);
                }
                return Ok(());
            }
            "DIALOG_SEND" => {
                // batch 220: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                        "DIALOG_SEND requires its arguments: DIALOG SEND hDlg , msg& , wParam& , lParam& [TO lResult& ]".to_string(),
                        None,
                        call.line,
                    ));
                }

                // DIALOG SEND hDlg, msg&, wParam&, lParam& [TO lResult&]
                if call.args.len() >= 4 {
                    let hd = self.compile_expr(fb, &call.args[0])?;
                    let hdlg64 = fb.inttoptr(&hd);
                    let msg = self.compile_expr(fb, &call.args[1])?;
                    let msg64 = self.convert_value(fb, &msg, &IrType::I64, &PbType::Quad);
                    let wp = self.compile_expr(fb, &call.args[2])?;
                    let wp64 = self.convert_value(fb, &wp, &IrType::I64, &PbType::Quad);
                    let lp = self.compile_expr(fb, &call.args[3])?;
                    let lp64 = self.convert_value(fb, &lp, &IrType::I64, &PbType::Quad);
                    let res = fb.call(&IrType::I64, "pb_dialog_send", &[hdlg64, msg64, wp64, lp64]);
                    if call.args.len() >= 5 {
                        if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[4]) {
                            let cv = self.convert_value(fb, &res, &ty, &pty);
                            fb.store(&cv, &ptr);
                        }
                    }
                }
                return Ok(());
            }
            "DIALOG_POST" => {
                // batch 220: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                        "DIALOG_POST requires its arguments: DIALOG POST hDlg , Msg& , wParam& , lParam&".to_string(),
                        None,
                        call.line,
                    ));
                }

                // DIALOG POST hDlg, msg&, wParam&, lParam&
                if call.args.len() >= 4 {
                    let hd = self.compile_expr(fb, &call.args[0])?;
                    let hdlg64 = fb.inttoptr(&hd);
                    let msg = self.compile_expr(fb, &call.args[1])?;
                    let msg64 = self.convert_value(fb, &msg, &IrType::I64, &PbType::Quad);
                    let wp = self.compile_expr(fb, &call.args[2])?;
                    let wp64 = self.convert_value(fb, &wp, &IrType::I64, &PbType::Quad);
                    let lp = self.compile_expr(fb, &call.args[3])?;
                    let lp64 = self.convert_value(fb, &lp, &IrType::I64, &PbType::Quad);
                    fb.call_void("pb_dialog_post", &[hdlg64, msg64, wp64, lp64]);
                }
                return Ok(());
            }
            "DIALOG_SET_USER" => {
                // batch 220: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                        "DIALOG_SET_USER requires its arguments: DIALOG SET USER hDlg , index& , usrval&".to_string(),
                        None,
                        call.line,
                    ));
                }

                // DIALOG SET USER hDlg, index&, usrval&
                if call.args.len() >= 3 {
                    let hd = self.compile_expr(fb, &call.args[0])?;
                    let hdlg64 = fb.inttoptr(&hd);
                    let idx = self.compile_expr(fb, &call.args[1])?;
                    let idx64 = self.convert_value(fb, &idx, &IrType::I64, &PbType::Quad);
                    let val = self.compile_expr(fb, &call.args[2])?;
                    let val64 = self.convert_value(fb, &val, &IrType::I64, &PbType::Quad);
                    fb.call_void("pb_dialog_set_user", &[hdlg64, idx64, val64]);
                }
                return Ok(());
            }
            "DIALOG_GET_USER" => {
                // batch 220: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                        "DIALOG_GET_USER requires its arguments: DIALOG GET USER hDlg , index& TO retvar&".to_string(),
                        None,
                        call.line,
                    ));
                }

                // DIALOG GET USER hDlg, index& TO retvar&
                if call.args.len() >= 3 {
                    let hd = self.compile_expr(fb, &call.args[0])?;
                    let hdlg64 = fb.inttoptr(&hd);
                    let idx = self.compile_expr(fb, &call.args[1])?;
                    let idx64 = self.convert_value(fb, &idx, &IrType::I64, &PbType::Quad);
                    let res = fb.call(&IrType::I64, "pb_dialog_get_user", &[hdlg64, idx64]);
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[2]) {
                        let cv = self.convert_value(fb, &res, &ty, &pty);
                        fb.store(&cv, &ptr);
                    }
                }
                return Ok(());
            }
            "DIALOG_SET_ICON" => {
                // DIALOG SET ICON hDlg, newicon$
                // batch 219: the official syntax requires all 2 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 2 {
                    return Err(pb::error::PbError::parser(
                        "DIALOG_SET_ICON requires its arguments: DIALOG SET ICON hDlg ,"
                            .to_string(),
                        None,
                        call.line,
                    ));
                }
                if call.args.len() >= 2 {
                    let hd = self.compile_expr(fb, &call.args[0])?;
                    let hdlg64 = fb.inttoptr(&hd);
                    let nm = self.compile_expr(fb, &call.args[1])?;
                    fb.call_void("pb_dialog_set_icon", &[hdlg64, nm]);
                }
                return Ok(());
            }
            "DIALOG_GET_CLIENT" => {
                // batch 220: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                        "DIALOG_GET_CLIENT requires its arguments: DIALOG GET CLIENT hDlg TO nWide& , nHigh&".to_string(),
                        None,
                        call.line,
                    ));
                }

                // DIALOG GET CLIENT hDlg TO nWide&, nHigh&
                if call.args.len() >= 3 {
                    let hd = self.compile_expr(fb, &call.args[0])?;
                    let hdlg64 = fb.inttoptr(&hd);
                    let pw = fb.alloca(&IrType::I64);
                    let ph = fb.alloca(&IrType::I64);
                    fb.call_void("pb_dialog_get_client", &[hdlg64, pw.clone(), ph.clone()]);
                    for (argi, slot) in [(1usize, &pw), (2usize, &ph)] {
                        if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[argi]) {
                            let v = fb.load(&IrType::I64, slot);
                            let cv = self.convert_value(fb, &v, &ty, &pty);
                            fb.store(&cv, &ptr);
                        }
                    }
                }
                return Ok(());
            }
            "DIALOG_SET_CLIENT" => {
                // batch 220: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                        "DIALOG_SET_CLIENT requires its arguments: DIALOG SET CLIENT hDlg , x& , y&".to_string(),
                        None,
                        call.line,
                    ));
                }

                // DIALOG SET CLIENT hDlg, x&, y&
                if call.args.len() >= 3 {
                    let hd = self.compile_expr(fb, &call.args[0])?;
                    let hdlg64 = fb.inttoptr(&hd);
                    let w = self.compile_expr(fb, &call.args[1])?;
                    let w64 = self.convert_value(fb, &w, &IrType::I64, &PbType::Quad);
                    let h = self.compile_expr(fb, &call.args[2])?;
                    let h64 = self.convert_value(fb, &h, &IrType::I64, &PbType::Quad);
                    fb.call_void("pb_dialog_set_client", &[hdlg64, w64, h64]);
                }
                return Ok(());
            }
            "DIALOG_GET_LOC" => {
                // batch 220: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                        "DIALOG_GET_LOC requires its arguments: DIALOG GET LOC hDlg TO x& , y&"
                            .to_string(),
                        None,
                        call.line,
                    ));
                }

                // DIALOG GET LOC hDlg TO x&, y&
                if call.args.len() >= 3 {
                    let hd = self.compile_expr(fb, &call.args[0])?;
                    let hdlg64 = fb.inttoptr(&hd);
                    let px = fb.alloca(&IrType::I64);
                    let py = fb.alloca(&IrType::I64);
                    fb.call_void("pb_dialog_get_loc", &[hdlg64, px.clone(), py.clone()]);
                    for (argi, slot) in [(1usize, &px), (2usize, &py)] {
                        if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[argi]) {
                            let v = fb.load(&IrType::I64, slot);
                            let cv = self.convert_value(fb, &v, &ty, &pty);
                            fb.store(&cv, &ptr);
                        }
                    }
                }
                return Ok(());
            }
            "DIALOG_SET_LOC" => {
                // batch 220: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                        "DIALOG_SET_LOC requires its arguments: DIALOG SET LOC hDlg , x& , y&"
                            .to_string(),
                        None,
                        call.line,
                    ));
                }

                // DIALOG SET LOC hDlg, x&, y&
                if call.args.len() >= 3 {
                    let hd = self.compile_expr(fb, &call.args[0])?;
                    let hdlg64 = fb.inttoptr(&hd);
                    let x = self.compile_expr(fb, &call.args[1])?;
                    let x64 = self.convert_value(fb, &x, &IrType::I64, &PbType::Quad);
                    let y = self.compile_expr(fb, &call.args[2])?;
                    let y64 = self.convert_value(fb, &y, &IrType::I64, &PbType::Quad);
                    fb.call_void("pb_dialog_set_loc", &[hdlg64, x64, y64]);
                }
                return Ok(());
            }
            "CONTROL_SET_COLOR" => {
                // batch 220: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                        "CONTROL_SET_COLOR requires its arguments: CONTROL SET COLOR hDlg , id& , foreclr& , backclr&".to_string(),
                        None,
                        call.line,
                    ));
                }

                // CONTROL SET COLOR hDlg, id&, foreclr&, backclr&
                //   The colour is recorded here and answered later, when Windows
                //   asks the dialog for a brush with a %WM_CTLCOLOR* message,
                //   which is why this arm only has to hand the four operands to
                //   the runtime.
                if call.args.len() >= 4 {
                    let hd = self.compile_expr(fb, &call.args[0])?;
                    let hdlg64 = fb.inttoptr(&hd);
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let id64 = self.convert_value(fb, &id, &IrType::I64, &PbType::Quad);
                    let fg = self.compile_expr(fb, &call.args[2])?;
                    let fg64 = self.convert_value(fb, &fg, &IrType::I64, &PbType::Quad);
                    let bg = self.compile_expr(fb, &call.args[3])?;
                    let bg64 = self.convert_value(fb, &bg, &IrType::I64, &PbType::Quad);
                    fb.call_void("pb_control_set_color", &[hdlg64, id64, fg64, bg64]);
                }
                return Ok(());
            }
            "DIALOG_SET_COLOR" => {
                // batch 220: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                        "DIALOG_SET_COLOR requires its arguments: DIALOG SET COLOR hDlg , foreclr& , backclr&".to_string(),
                        None,
                        call.line,
                    ));
                }

                // DIALOG SET COLOR hDlg, foreclr&, backclr&
                if call.args.len() >= 3 {
                    let hd = self.compile_expr(fb, &call.args[0])?;
                    let hdlg64 = fb.inttoptr(&hd);
                    let fg = self.compile_expr(fb, &call.args[1])?;
                    let fg64 = self.convert_value(fb, &fg, &IrType::I64, &PbType::Quad);
                    let bg = self.compile_expr(fb, &call.args[2])?;
                    let bg64 = self.convert_value(fb, &bg, &IrType::I64, &PbType::Quad);
                    fb.call_void("pb_dialog_set_color", &[hdlg64, fg64, bg64]);
                }
                return Ok(());
            }
            "DIALOG_DEFAULT_FONT" => {
                // DIALOG DEFAULT FONT fontname$ [, points& [, style& [, charset&]]]
                if !call.args.is_empty() {
                    let nm = self.compile_expr(fb, &call.args[0])?;
                    let mut vals = Vec::new();
                    for i in 1..4 {
                        if let Some(a) = call.args.get(i) {
                            let v = self.compile_expr(fb, a)?;
                            vals.push(self.to_i32(fb, &v));
                        } else {
                            vals.push(fb.const_i32(0));
                        }
                    }
                    fb.call_void(
                        "pb_dialog_default_font",
                        &[nm, vals[0].clone(), vals[1].clone(), vals[2].clone()],
                    );
                }
                return Ok(());
            }
            "DIALOG_PIXELS" | "DIALOG_UNITS" => {
                // DIALOG PIXELS  hDlg, x&, y& TO UNITS  xx&, yy&
                // DIALOG UNITS   hDlg, x&, y& TO PIXELS xx&, yy&
                if call.args.len() >= 5 {
                    let is_px = call.name.to_uppercase() == "DIALOG_PIXELS";
                    let fname = if is_px {
                        "pb_dialog_pixels"
                    } else {
                        "pb_dialog_units"
                    };
                    let hd = self.compile_expr(fb, &call.args[0])?;
                    let hdlg64 = fb.inttoptr(&hd);
                    let x = self.compile_expr(fb, &call.args[1])?;
                    let x64 = self.convert_value(fb, &x, &IrType::I64, &PbType::Quad);
                    let y = self.compile_expr(fb, &call.args[2])?;
                    let y64 = self.convert_value(fb, &y, &IrType::I64, &PbType::Quad);
                    let px = fb.alloca(&IrType::I64);
                    let py = fb.alloca(&IrType::I64);
                    fb.call_void(fname, &[hdlg64, x64, y64, px.clone(), py.clone()]);
                    for (argi, slot) in [(3usize, &px), (4usize, &py)] {
                        if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[argi]) {
                            let v = fb.load(&IrType::I64, slot);
                            let cv = self.convert_value(fb, &v, &ty, &pty);
                            fb.store(&cv, &ptr);
                        }
                    }
                }
                return Ok(());
            }
            "DIALOG_GET_TEXT" => {
                // batch 220: the official syntax requires all 2 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 2 {
                    return Err(pb::error::PbError::parser(
                        "DIALOG_GET_TEXT requires its arguments: DIALOG GET TEXT hDlg TO titletext$".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 2 {
                    let hd = self.compile_expr(fb, &call.args[0])?;
                    let hdlg64 = fb.inttoptr(&hd);
                    let buf = fb.alloca(&IrType::Array(256, Box::new(IrType::I8)));
                    let bp = fb.gep_byte(&buf, &fb.const_i32(0));
                    fb.call_void(
                        "pb_dialog_get_text",
                        &[hdlg64, bp.clone(), fb.const_i64(256)],
                    );
                    if let Some((ptr, _, _)) = self.lvalue_ptr(fb, &call.args[1]) {
                        let len =
                            fb.call(&IrType::I32, "pb_str_cstr_len", std::slice::from_ref(&bp));
                        let bstr = fb.call(&IrType::Ptr, "pb_bstr_alloc", &[bp, len]);
                        fb.store(&bstr, &ptr);
                    }
                }
                return Ok(());
            }
            "DIALOG_SHOW_STATE" => {
                // batch 219: the official syntax requires all 2 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 2 {
                    return Err(pb::error::PbError::parser(
                        "DIALOG_SHOW_STATE requires its arguments: DIALOG SHOW STATE hDlg ,"
                            .to_string(),
                        None,
                        call.line,
                    ));
                }
                if call.args.len() >= 2 {
                    let hd = self.compile_expr(fb, &call.args[0])?;
                    let st = self.compile_expr(fb, &call.args[1])?;
                    let hdlg64 = fb.inttoptr(&hd);
                    fb.call_void("pb_dialog_show_state", &[hdlg64, st]);
                }
                return Ok(());
            }
            "CONTROL_KILL" => {
                if !call.args.is_empty() {
                    let hc = self.compile_expr(fb, &call.args[0])?;
                    let hc64 = fb.inttoptr(&hc);
                    fb.call_void("pb_control_kill", &[hc64]);
                }
                return Ok(());
            }
            "CONTROL_SET_CHECK" => {
                // batch 219: the official syntax requires all 2 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 2 {
                    return Err(pb::error::PbError::parser(
                        "CONTROL_SET_CHECK requires its arguments: CONTROL SET CHECK hDlg ,"
                            .to_string(),
                        None,
                        call.line,
                    ));
                }
                if call.args.len() >= 2 {
                    let hc = self.compile_expr(fb, &call.args[0])?;
                    let st = self.compile_expr(fb, &call.args[1])?;
                    let hc64 = fb.inttoptr(&hc);
                    fb.call_void("pb_control_set_check", &[hc64, st]);
                }
                return Ok(());
            }
            "DIALOG_CENTER" => {
                if !call.args.is_empty() {
                    let hd = self.compile_expr(fb, &call.args[0])?;
                    let hd64 = fb.inttoptr(&hd);
                    fb.call_void("pb_dialog_center", &[hd64]);
                }
                return Ok(());
            }
            // CONTROL ADD classname$ - generic custom control (batch 166).
            // Layout normalised by the parser to
            //   [cls, hDlg, id, text, x, y, w, h, style, exstyle, target]
            "CONTROL_ADD_CUSTOM" => {
                // batch 220: the official syntax requires all 11 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 11 {
                    return Err(pb::error::PbError::parser(
                        "CONTROL_ADD_CUSTOM requires its arguments: CONTROL ADD CUSTOM classname$ , hDlg , id& , txt$ , x , y , xx , yy [, style& ] [, exstyle& ] TO hCtrl&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 11 {
                    let cls = self.compile_expr(fb, &call.args[0])?;
                    let mut parent = self.compile_expr(fb, &call.args[1])?;
                    if parent.ty != IrType::Ptr {
                        parent = fb.inttoptr(&parent);
                    }
                    let id = self.compile_expr(fb, &call.args[2])?;
                    let text = self.compile_expr(fb, &call.args[3])?;
                    let x = self.compile_expr(fb, &call.args[4])?;
                    let y = self.compile_expr(fb, &call.args[5])?;
                    let w = self.compile_expr(fb, &call.args[6])?;
                    let h = self.compile_expr(fb, &call.args[7])?;
                    let style_v = self.compile_expr(fb, &call.args[8])?;
                    let exstyle_v = self.compile_expr(fb, &call.args[9])?;
                    let style = self.convert_value(fb, &style_v, &IrType::I32, &PbType::Long);
                    let exstyle = self.convert_value(fb, &exstyle_v, &IrType::I32, &PbType::Long);
                    let hc = fb.call(
                        &IrType::Ptr,
                        "pb_control_add_custom",
                        &[cls, parent, id, text, x, y, w, h, style, exstyle],
                    );
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[10]) {
                        let hc_i = fb.ptrtoint64(&hc);
                        let hc_i_v = self.convert_value(fb, &hc_i, &ty, &pty);
                        fb.store(&hc_i_v, &ptr);
                    }
                }
                return Ok(());
            }
            "DIALOG_END" => {
                // batch 220: the official syntax requires all 2 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 2 {
                    return Err(pb::error::PbError::parser(
                        "DIALOG_END requires its arguments: DIALOG END hDlg [, lResult& ]"
                            .to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 2 {
                    let hd = self.compile_expr(fb, &call.args[0])?;
                    let result = self.compile_expr(fb, &call.args[1])?;
                    fb.call_void("pb_dialog_end", &[hd, result]);
                }
                return Ok(());
            }
            "DIALOG_MENU" => {
                // batch 220: the official syntax requires all 2 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 2 {
                    return Err(pb::error::PbError::parser(
                        "DIALOG_MENU requires its arguments: DIALOG MENU hDlg , hMenu".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 2 {
                    let hdlg = self.compile_expr(fb, &call.args[0])?;
                    let hmenu = self.compile_expr(fb, &call.args[1])?;
                    let hdlg64 = fb.inttoptr(&hdlg);
                    let hmenu64 = fb.inttoptr(&hmenu);
                    fb.call_void("pb_dialog_menu", &[hdlg64, hmenu64]);
                }
                return Ok(());
            }
            "CONTROL_SET_POS" => {
                // batch 219: the official syntax requires all 2 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 2 {
                    return Err(pb::error::PbError::parser(
                        "CONTROL_SET_POS requires its arguments: CONTROL SET FONT hDlg ,"
                            .to_string(),
                        None,
                        call.line,
                    ));
                }
                if call.args.len() >= 2 {
                    let hc = self.compile_expr(fb, &call.args[0])?;
                    let pos = self.compile_expr(fb, &call.args[1])?;
                    let pos64 = fb.sext(&pos, &IrType::I64);
                    fb.call_void("pb_control_set_pos", &[hc, pos64]);
                }
                return Ok(());
            }
            "CONTROL_GET_POS" => {
                // batch 219: the official syntax requires all 2 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 2 {
                    return Err(pb::error::PbError::parser(
                        "CONTROL_GET_POS requires its arguments: CONTROL GET LOC hDlg ,"
                            .to_string(),
                        None,
                        call.line,
                    ));
                }
                if call.args.len() >= 2 {
                    let hc = self.compile_expr(fb, &call.args[0])?;
                    let val = fb.call(&IrType::I64, "pb_control_get_pos", &[hc]);
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[1]) {
                        let vv = self.convert_value(fb, &val, &ty, &pty);
                        fb.store(&vv, &ptr);
                    }
                }
                return Ok(());
            }
            "CONTROL_GET_CLIENT" => {
                // batch 220: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                        "CONTROL_GET_CLIENT requires its arguments: CONTROL GET CLIENT hDlg , id& TO w& , h&".to_string(),
                        None,
                        call.line,
                    ));
                }

                // CONTROL GET CLIENT hDlg, id& TO nWide&, nHigh&
                if call.args.len() >= 4 {
                    let hd = self.compile_expr(fb, &call.args[0])?;
                    let hdlg64 = match hd.ty {
                        IrType::Ptr => hd,
                        _ => fb.inttoptr(&hd),
                    };
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let id64 = self.convert_value(fb, &id, &IrType::I64, &PbType::Quad);
                    let pa = fb.alloca(&IrType::I64);
                    let pb_ = fb.alloca(&IrType::I64);
                    fb.call_void(
                        "pb_control_get_client",
                        &[hdlg64, id64, pa.clone(), pb_.clone()],
                    );
                    for (argi, slot) in [(2usize, &pa), (3usize, &pb_)] {
                        if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[argi]) {
                            let v = fb.load(&IrType::I64, slot);
                            let cv = self.convert_value(fb, &v, &ty, &pty);
                            fb.store(&cv, &ptr);
                        }
                    }
                }
                return Ok(());
            }
            "CONTROL_GET_LOC" => {
                // batch 220: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                        "CONTROL_GET_LOC requires its arguments: CONTROL GET LOC hDlg , id& TO x& , y&".to_string(),
                        None,
                        call.line,
                    ));
                }

                // CONTROL GET LOC hDlg, id& TO x&, y&
                if call.args.len() >= 4 {
                    let hd = self.compile_expr(fb, &call.args[0])?;
                    let hdlg64 = match hd.ty {
                        IrType::Ptr => hd,
                        _ => fb.inttoptr(&hd),
                    };
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let id64 = self.convert_value(fb, &id, &IrType::I64, &PbType::Quad);
                    let pa = fb.alloca(&IrType::I64);
                    let pb_ = fb.alloca(&IrType::I64);
                    fb.call_void(
                        "pb_control_get_loc",
                        &[hdlg64, id64, pa.clone(), pb_.clone()],
                    );
                    for (argi, slot) in [(2usize, &pa), (3usize, &pb_)] {
                        if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[argi]) {
                            let v = fb.load(&IrType::I64, slot);
                            let cv = self.convert_value(fb, &v, &ty, &pty);
                            fb.store(&cv, &ptr);
                        }
                    }
                }
                return Ok(());
            }
            "CONTROL_GET_SIZE" => {
                // batch 220: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                        "CONTROL_GET_SIZE requires its arguments: CONTROL GET SIZE hDlg , id& TO nWide& , nHigh&".to_string(),
                        None,
                        call.line,
                    ));
                }

                // CONTROL GET SIZE hDlg, id& TO nWide&, nHigh&
                if call.args.len() >= 4 {
                    let hd = self.compile_expr(fb, &call.args[0])?;
                    let hdlg64 = match hd.ty {
                        IrType::Ptr => hd,
                        _ => fb.inttoptr(&hd),
                    };
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let id64 = self.convert_value(fb, &id, &IrType::I64, &PbType::Quad);
                    let pa = fb.alloca(&IrType::I64);
                    let pb_ = fb.alloca(&IrType::I64);
                    fb.call_void(
                        "pb_control_get_size",
                        &[hdlg64, id64, pa.clone(), pb_.clone()],
                    );
                    for (argi, slot) in [(2usize, &pa), (3usize, &pb_)] {
                        if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[argi]) {
                            let v = fb.load(&IrType::I64, slot);
                            let cv = self.convert_value(fb, &v, &ty, &pty);
                            fb.store(&cv, &ptr);
                        }
                    }
                }
                return Ok(());
            }
            "CONTROL_SET_CLIENT" => {
                // batch 220: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                        "CONTROL_SET_CLIENT requires its arguments: CONTROL SET CLIENT hDlg , id& , nWide& , nHigh&".to_string(),
                        None,
                        call.line,
                    ));
                }

                // CONTROL SET CLIENT hDlg, id&, nWide&, nHigh&
                if call.args.len() >= 4 {
                    let hd = self.compile_expr(fb, &call.args[0])?;
                    let hdlg64 = match hd.ty {
                        IrType::Ptr => hd,
                        _ => fb.inttoptr(&hd),
                    };
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let id64 = self.convert_value(fb, &id, &IrType::I64, &PbType::Quad);
                    let a = self.compile_expr(fb, &call.args[2])?;
                    let a64 = self.convert_value(fb, &a, &IrType::I64, &PbType::Quad);
                    let b = self.compile_expr(fb, &call.args[3])?;
                    let b64 = self.convert_value(fb, &b, &IrType::I64, &PbType::Quad);
                    fb.call_void("pb_control_set_client", &[hdlg64, id64, a64, b64]);
                }
                return Ok(());
            }
            "CONTROL_SET_LOC" => {
                // batch 220: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                        "CONTROL_SET_LOC requires its arguments: CONTROL SET LOC hDlg , id& , x& , y&".to_string(),
                        None,
                        call.line,
                    ));
                }

                // CONTROL SET LOC hDlg, id&, x&, y&
                if call.args.len() >= 4 {
                    let hd = self.compile_expr(fb, &call.args[0])?;
                    let hdlg64 = match hd.ty {
                        IrType::Ptr => hd,
                        _ => fb.inttoptr(&hd),
                    };
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let id64 = self.convert_value(fb, &id, &IrType::I64, &PbType::Quad);
                    let a = self.compile_expr(fb, &call.args[2])?;
                    let a64 = self.convert_value(fb, &a, &IrType::I64, &PbType::Quad);
                    let b = self.compile_expr(fb, &call.args[3])?;
                    let b64 = self.convert_value(fb, &b, &IrType::I64, &PbType::Quad);
                    fb.call_void("pb_control_set_loc", &[hdlg64, id64, a64, b64]);
                }
                return Ok(());
            }
            "CONTROL_SET_SIZE" => {
                // batch 220: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                        "CONTROL_SET_SIZE requires its arguments: CONTROL SET SIZE hDlg , id& , nWide& , nHigh&".to_string(),
                        None,
                        call.line,
                    ));
                }

                // CONTROL SET SIZE hDlg, id&, nWide&, nHigh&
                if call.args.len() >= 4 {
                    let hd = self.compile_expr(fb, &call.args[0])?;
                    let hdlg64 = match hd.ty {
                        IrType::Ptr => hd,
                        _ => fb.inttoptr(&hd),
                    };
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let id64 = self.convert_value(fb, &id, &IrType::I64, &PbType::Quad);
                    let a = self.compile_expr(fb, &call.args[2])?;
                    let a64 = self.convert_value(fb, &a, &IrType::I64, &PbType::Quad);
                    let b = self.compile_expr(fb, &call.args[3])?;
                    let b64 = self.convert_value(fb, &b, &IrType::I64, &PbType::Quad);
                    fb.call_void("pb_control_set_size", &[hdlg64, id64, a64, b64]);
                }
                return Ok(());
            }
            // --- Batch 176: CONTROL messages / state (official hDlg + id syntax) ---
            "CONTROL_HANDLE" => {
                // batch 220: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                        "CONTROL_HANDLE requires its arguments: CONTROL HANDLE hDlg , id& TO hCtl&"
                            .to_string(),
                        None,
                        call.line,
                    ));
                }

                // CONTROL HANDLE hDlg, id& TO hCtl&
                if call.args.len() >= 3 {
                    let hd = self.compile_expr(fb, &call.args[0])?;
                    let hdlg64 = match hd.ty {
                        IrType::Ptr => hd,
                        _ => fb.inttoptr(&hd),
                    };
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let id64 = self.convert_value(fb, &id, &IrType::I64, &PbType::Quad);
                    let hctl = fb.call(&IrType::Ptr, "pb_control_handle", &[hdlg64, id64]);
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[2]) {
                        let cv = self.convert_value(fb, &hctl, &ty, &pty);
                        fb.store(&cv, &ptr);
                    }
                }
                return Ok(());
            }
            "CONTROL_SEND" => {
                // batch 220: the official syntax requires all 5 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 5 {
                    return Err(pb::error::PbError::parser(
                        "CONTROL_SEND requires its arguments: CONTROL SEND hDlg , id& , Msg& , wParam& , lParam& [TO lResult& ]".to_string(),
                        None,
                        call.line,
                    ));
                }

                // CONTROL SEND hDlg, id&, Msg&, wParam&, lParam& [TO lResult&]
                if call.args.len() >= 5 {
                    let hd = self.compile_expr(fb, &call.args[0])?;
                    let hdlg64 = match hd.ty {
                        IrType::Ptr => hd,
                        _ => fb.inttoptr(&hd),
                    };
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let id64 = self.convert_value(fb, &id, &IrType::I64, &PbType::Quad);
                    let msg = self.compile_expr(fb, &call.args[2])?;
                    let msg64 = self.convert_value(fb, &msg, &IrType::I64, &PbType::Quad);
                    let wp = self.compile_expr(fb, &call.args[3])?;
                    let wp64 = self.convert_value(fb, &wp, &IrType::I64, &PbType::Quad);
                    let lp = self.compile_expr(fb, &call.args[4])?;
                    let lp64 = self.convert_value(fb, &lp, &IrType::I64, &PbType::Quad);
                    let res = fb.call(
                        &IrType::I64,
                        "pb_control_send",
                        &[hdlg64, id64, msg64, wp64, lp64],
                    );
                    if let Some(tgt) = call.args.get(5) {
                        if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, tgt) {
                            let cv = self.convert_value(fb, &res, &ty, &pty);
                            fb.store(&cv, &ptr);
                        }
                    }
                }
                return Ok(());
            }
            "CONTROL_POST" => {
                // batch 220: the official syntax requires all 5 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 5 {
                    return Err(pb::error::PbError::parser(
                        "CONTROL_POST requires its arguments: CONTROL POST hDlg , id& , Msg& , wParam& , lParam&".to_string(),
                        None,
                        call.line,
                    ));
                }

                // CONTROL POST hDlg, id&, Msg&, wParam&, lParam&
                if call.args.len() >= 5 {
                    let hd = self.compile_expr(fb, &call.args[0])?;
                    let hdlg64 = match hd.ty {
                        IrType::Ptr => hd,
                        _ => fb.inttoptr(&hd),
                    };
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let id64 = self.convert_value(fb, &id, &IrType::I64, &PbType::Quad);
                    let msg = self.compile_expr(fb, &call.args[2])?;
                    let msg64 = self.convert_value(fb, &msg, &IrType::I64, &PbType::Quad);
                    let wp = self.compile_expr(fb, &call.args[3])?;
                    let wp64 = self.convert_value(fb, &wp, &IrType::I64, &PbType::Quad);
                    let lp = self.compile_expr(fb, &call.args[4])?;
                    let lp64 = self.convert_value(fb, &lp, &IrType::I64, &PbType::Quad);
                    fb.call_void("pb_control_post", &[hdlg64, id64, msg64, wp64, lp64]);
                }
                return Ok(());
            }
            "CONTROL_REDRAW" | "CONTROL_NORMALIZE" | "CONTROL_SET_FOCUS" => {
                // CONTROL REDRAW / NORMALIZE / SET FOCUS  hDlg, id&
                if call.args.len() >= 2 {
                    let hd = self.compile_expr(fb, &call.args[0])?;
                    let hdlg64 = match hd.ty {
                        IrType::Ptr => hd,
                        _ => fb.inttoptr(&hd),
                    };
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let id64 = self.convert_value(fb, &id, &IrType::I64, &PbType::Quad);
                    let fname = match call.name.as_str() {
                        "CONTROL_REDRAW" => "pb_control_redraw",
                        "CONTROL_NORMALIZE" => "pb_control_normalize",
                        _ => "pb_control_set_focus",
                    };
                    fb.call_void(fname, &[hdlg64, id64]);
                }
                return Ok(());
            }
            "CONTROL_SET_FONT" => {
                // batch 220: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                        "CONTROL_SET_FONT requires its arguments: CONTROL SET FONT hDlg , id& , FontHndl&".to_string(),
                        None,
                        call.line,
                    ));
                }

                // CONTROL SET FONT hDlg, id&, FontHndl&
                if call.args.len() >= 3 {
                    let hd = self.compile_expr(fb, &call.args[0])?;
                    let hdlg64 = match hd.ty {
                        IrType::Ptr => hd,
                        _ => fb.inttoptr(&hd),
                    };
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let id64 = self.convert_value(fb, &id, &IrType::I64, &PbType::Quad);
                    let fnt = self.compile_expr(fb, &call.args[2])?;
                    let fnt64 = self.convert_value(fb, &fnt, &IrType::I64, &PbType::Quad);
                    fb.call_void("pb_control_set_font", &[hdlg64, id64, fnt64]);
                }
                return Ok(());
            }
            "CONTROL_SHOW_STATE" => {
                // batch 220: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                        "CONTROL_SHOW_STATE requires its arguments: CONTROL SHOW STATE hDlg , id& , showstate& [TO lResult& ]".to_string(),
                        None,
                        call.line,
                    ));
                }

                // CONTROL SHOW STATE hDlg, id&, showstate& [TO lResult&]
                if call.args.len() >= 3 {
                    let hd = self.compile_expr(fb, &call.args[0])?;
                    let hdlg64 = match hd.ty {
                        IrType::Ptr => hd,
                        _ => fb.inttoptr(&hd),
                    };
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let id64 = self.convert_value(fb, &id, &IrType::I64, &PbType::Quad);
                    let st = self.compile_expr(fb, &call.args[2])?;
                    let st64 = self.convert_value(fb, &st, &IrType::I64, &PbType::Quad);
                    let res = fb.call(&IrType::I64, "pb_control_show_state", &[hdlg64, id64, st64]);
                    if let Some(tgt) = call.args.get(3) {
                        if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, tgt) {
                            let cv = self.convert_value(fb, &res, &ty, &pty);
                            fb.store(&cv, &ptr);
                        }
                    }
                }
                return Ok(());
            }
            "CONTROL_SET_USER" => {
                // batch 220: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                        "CONTROL_SET_USER requires its arguments: CONTROL SET USER hDlg , id& , index& , usrval&".to_string(),
                        None,
                        call.line,
                    ));
                }

                // CONTROL SET USER hDlg, id&, index&, usrval&
                if call.args.len() >= 4 {
                    let hd = self.compile_expr(fb, &call.args[0])?;
                    let hdlg64 = match hd.ty {
                        IrType::Ptr => hd,
                        _ => fb.inttoptr(&hd),
                    };
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let id64 = self.convert_value(fb, &id, &IrType::I64, &PbType::Quad);
                    let idx = self.compile_expr(fb, &call.args[2])?;
                    let idx64 = self.convert_value(fb, &idx, &IrType::I64, &PbType::Quad);
                    let val = self.compile_expr(fb, &call.args[3])?;
                    let val64 = self.convert_value(fb, &val, &IrType::I64, &PbType::Quad);
                    fb.call_void("pb_control_set_user", &[hdlg64, id64, idx64, val64]);
                }
                return Ok(());
            }
            "CONTROL_GET_USER" => {
                // batch 220: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                        "CONTROL_GET_USER requires its arguments: CONTROL GET USER hDlg , id& , index& TO retvar&".to_string(),
                        None,
                        call.line,
                    ));
                }

                // CONTROL GET USER hDlg, id&, index& TO retvar&
                if call.args.len() >= 4 {
                    let hd = self.compile_expr(fb, &call.args[0])?;
                    let hdlg64 = match hd.ty {
                        IrType::Ptr => hd,
                        _ => fb.inttoptr(&hd),
                    };
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let id64 = self.convert_value(fb, &id, &IrType::I64, &PbType::Quad);
                    let idx = self.compile_expr(fb, &call.args[2])?;
                    let idx64 = self.convert_value(fb, &idx, &IrType::I64, &PbType::Quad);
                    let res = fb.call(&IrType::I64, "pb_control_get_user", &[hdlg64, id64, idx64]);
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &call.args[3]) {
                        let cv = self.convert_value(fb, &res, &ty, &pty);
                        fb.store(&cv, &ptr);
                    }
                }
                return Ok(());
            }
            "CONTROL_CMD" => {
                if call.args.len() >= 2 {
                    let hctrl = self.compile_expr(fb, &call.args[0])?;
                    if let Expr::Variable(name) = &call.args[1] {
                        let upper = name.to_uppercase();
                        let fi = self.functions.get(&upper).or_else(|| self.subs.get(&upper));
                        if let Some(fi) = fi {
                            let fn_ptr = Val::new(format!("@{}", fi.ir_name), IrType::Ptr);
                            fb.call_void("pb_register_callback_hwnd", &[hctrl, fn_ptr]);
                        } else {
                            eprintln!("warning: CONTROL CMD: unknown sub '{}'", upper);
                        }
                    }
                }
                return Ok(());
            }
            "CONTROL_ADD_LISTBOX" => {
                // batch 245: full official syntax, fixed 11-arg layout
                // [hwnd, id, items, x, y, w, h, style, exstyle, callback, target]
                // (CONTROL ADD LISTBOX, hDlg, id&, [items$()|txt$|], x, y, xx, yy
                //   [, [style&] [, [exstyle&]]] [[,] CALL callback] [, TO hCtrl&])
                if call.args.len() < 11 {
                    return Err(pb::error::PbError::parser(
                        "CONTROL_ADD_LISTBOX requires its arguments: CONTROL ADD LISTBOX hDlg , id& , items$() , x , y , xx , yy [, [style&] [, [exstyle&]]] [[,] CALL callback] [, TO hCtrl&]".to_string(),
                        None,
                        call.line,
                    ));
                }

                let mut hwnd = self.compile_expr(fb, &call.args[0])?;
                if hwnd.ty != IrType::Ptr {
                    hwnd = fb.inttoptr(&hwnd);
                }
                let id = self.compile_expr(fb, &call.args[1])?;
                // items slot: full-array reference sArray() → pass base pointer +
                // element count so the runtime can LB_ADDSTRING each item.
                // (compile_expr would load only the first element, which is wrong.)
                let (items, item_count) =
                    if let Some(Expr::FunctionCall(arr_name, _)) = call.args.get(2) {
                        let an = normalize_name(arr_name);
                        if let Some(arr_info) = self.symbols.lookup_array(&an).cloned() {
                            (
                                Val::new(arr_info.ptr_name.clone(), IrType::Ptr),
                                arr_info.total_elements as i64,
                            )
                        } else {
                            let v = self.compile_expr(fb, &call.args[2])?;
                            (v, 0)
                        }
                    } else {
                        let v = self.compile_expr(fb, &call.args[2])?;
                        (v, 0)
                    };
                let x = self.compile_expr(fb, &call.args[3])?;
                let y = self.compile_expr(fb, &call.args[4])?;
                let w = self.compile_expr(fb, &call.args[5])?;
                let h = self.compile_expr(fb, &call.args[6])?;
                let style = self.compile_expr(fb, &call.args[7])?;
                let exstyle = self.compile_expr(fb, &call.args[8])?;
                let cb = self.compile_control_callback(fb, &call.args[9])?;
                let hc = fb.call(
                    &IrType::Ptr,
                    "pb_control_add_listbox_ex",
                    &[
                        hwnd,
                        id,
                        items,
                        fb.const_i64(item_count),
                        x,
                        y,
                        w,
                        h,
                        style,
                        exstyle,
                        cb,
                    ],
                );
                if let Some(t) = call.args.get(10) {
                    if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, t) {
                        let cv = self.convert_value(fb, &hc, &ty, &pty);
                        fb.store(&cv, &ptr);
                    }
                }
                return Ok(());
            }
            "CONTROL_ADD_CHECKBOX" => {
                // batch 220: the official syntax requires all 8 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 8 {
                    return Err(pb::error::PbError::parser(
                        "CONTROL_ADD_CHECKBOX requires its arguments: CONTROL ADD CHECKBOX hDlg , id& , txt$ , x , y , xx , yy TO hCtrl&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 8 {
                    let mut hwnd = self.compile_expr(fb, &call.args[0])?;
                    if hwnd.ty != IrType::Ptr {
                        hwnd = fb.inttoptr(&hwnd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let text = self.compile_expr(fb, &call.args[2])?;
                    let x = self.compile_expr(fb, &call.args[3])?;
                    let y = self.compile_expr(fb, &call.args[4])?;
                    let w = self.compile_expr(fb, &call.args[5])?;
                    let h = self.compile_expr(fb, &call.args[6])?;
                    // batch 249: full official syntax (mirrors BUTTON)
                    let target = if call.args.len() == 8 {
                        Some(call.args[7].clone())
                    } else {
                        call.args.get(10).cloned()
                    };
                    let hctrl = if call.args.len() == 8 {
                        fb.call(
                            &IrType::Ptr,
                            "pb_control_add_checkbox",
                            &[hwnd, id, text, x, y, w, h],
                        )
                    } else {
                        let style = self.compile_expr(fb, &call.args[7])?;
                        let exstyle = self.compile_expr(fb, &call.args[8])?;
                        let cb = self.compile_control_callback(fb, &call.args[9])?;
                        fb.call(
                            &IrType::Ptr,
                            "pb_control_add_checkbox_ex",
                            &[hwnd, id, text, x, y, w, h, style, exstyle, cb],
                        )
                    };
                    if let Some(tgt) = target {
                        if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &tgt) {
                            let cv = self.convert_value(fb, &hctrl, &ty, &pty);
                            fb.store(&cv, &ptr);
                        }
                    }
                }
                return Ok(());
            }
            "CONTROL_ADD_RADIOBUTTON" => {
                // batch 220: the official syntax requires all 8 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 8 {
                    return Err(pb::error::PbError::parser(
                        "CONTROL_ADD_RADIOBUTTON requires its arguments: CONTROL ADD RADIOBUTTON hDlg , id& , txt$ , x , y , xx , yy TO hCtrl&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 8 {
                    let mut hwnd = self.compile_expr(fb, &call.args[0])?;
                    if hwnd.ty != IrType::Ptr {
                        hwnd = fb.inttoptr(&hwnd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let text = self.compile_expr(fb, &call.args[2])?;
                    let x = self.compile_expr(fb, &call.args[3])?;
                    let y = self.compile_expr(fb, &call.args[4])?;
                    let w = self.compile_expr(fb, &call.args[5])?;
                    let h = self.compile_expr(fb, &call.args[6])?;
                    // batch 249: full official syntax (mirrors BUTTON)
                    let target = if call.args.len() == 8 {
                        Some(call.args[7].clone())
                    } else {
                        call.args.get(10).cloned()
                    };
                    let hctrl = if call.args.len() == 8 {
                        fb.call(
                            &IrType::Ptr,
                            "pb_control_add_radiobutton",
                            &[hwnd, id, text, x, y, w, h],
                        )
                    } else {
                        let style = self.compile_expr(fb, &call.args[7])?;
                        let exstyle = self.compile_expr(fb, &call.args[8])?;
                        let cb = self.compile_control_callback(fb, &call.args[9])?;
                        fb.call(
                            &IrType::Ptr,
                            "pb_control_add_radiobutton_ex",
                            &[hwnd, id, text, x, y, w, h, style, exstyle, cb],
                        )
                    };
                    if let Some(tgt) = target {
                        if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &tgt) {
                            let cv = self.convert_value(fb, &hctrl, &ty, &pty);
                            fb.store(&cv, &ptr);
                        }
                    }
                }
                return Ok(());
            }
            "CONTROL_ADD_OPTION"
            | "CONTROL_ADD_CHECK3STATE"
            | "CONTROL_ADD_FRAME"
            | "CONTROL_ADD_TEXTBOX"
            | "CONTROL_ADD_LINE" => {
                // batch 177: the plain and static CONTROL family.  Same
                // argument shape as every other typed CONTROL ADD arm:
                // (hDlg, id, txt$, x, y, w, h, TO-target).
                // batch 249: full official syntax - style/exstyle passthrough
                // + CALL callback (mirrors CONTROL ADD BUTTON).
                if call.args.len() >= 8 {
                    let mut hwnd = self.compile_expr(fb, &call.args[0])?;
                    if hwnd.ty != IrType::Ptr {
                        hwnd = fb.inttoptr(&hwnd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let text = self.compile_expr(fb, &call.args[2])?;
                    let x = self.compile_expr(fb, &call.args[3])?;
                    let y = self.compile_expr(fb, &call.args[4])?;
                    let w = self.compile_expr(fb, &call.args[5])?;
                    let h = self.compile_expr(fb, &call.args[6])?;
                    let (fname, fname_ex) = if call.name == "CONTROL_ADD_OPTION" {
                        ("pb_control_add_option", "pb_control_add_option_ex")
                    } else if call.name == "CONTROL_ADD_CHECK3STATE" {
                        (
                            "pb_control_add_check3state",
                            "pb_control_add_check3state_ex",
                        )
                    } else if call.name == "CONTROL_ADD_FRAME" {
                        ("pb_control_add_frame", "pb_control_add_frame_ex")
                    } else if call.name == "CONTROL_ADD_TEXTBOX" {
                        ("pb_control_add_textbox", "pb_control_add_textbox_ex")
                    } else {
                        ("pb_control_add_line", "pb_control_add_line_ex")
                    };
                    let target = if call.args.len() == 8 {
                        Some(call.args[7].clone())
                    } else {
                        call.args.get(10).cloned()
                    };
                    let hctrl = if call.args.len() == 8 {
                        fb.call(&IrType::Ptr, fname, &[hwnd, id, text, x, y, w, h])
                    } else {
                        let style = self.compile_expr(fb, &call.args[7])?;
                        let exstyle = self.compile_expr(fb, &call.args[8])?;
                        let cb = self.compile_control_callback(fb, &call.args[9])?;
                        fb.call(
                            &IrType::Ptr,
                            fname_ex,
                            &[hwnd, id, text, x, y, w, h, style, exstyle, cb],
                        )
                    };
                    if let Some(tgt) = target {
                        if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &tgt) {
                            let cv = self.convert_value(fb, &hctrl, &ty, &pty);
                            fb.store(&cv, &ptr);
                        }
                    }
                }
                return Ok(());
            }
            "CONTROL_SET_OPTION" => {
                // batch 220: the official syntax requires all 4 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 4 {
                    return Err(pb::error::PbError::parser(
                        "CONTROL_SET_OPTION requires its arguments: CONTROL SET OPTION hDlg , id& , minid& , maxid&".to_string(),
                        None,
                        call.line,
                    ));
                }

                // CONTROL SET OPTION hDlg, id&, minid&, maxid&
                if call.args.len() >= 4 {
                    let hd = self.compile_expr(fb, &call.args[0])?;
                    let hdlg64 = match hd.ty {
                        IrType::Ptr => hd,
                        _ => fb.inttoptr(&hd),
                    };
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let id64 = self.convert_value(fb, &id, &IrType::I64, &PbType::Quad);
                    let lo = self.compile_expr(fb, &call.args[2])?;
                    let lo64 = self.convert_value(fb, &lo, &IrType::I64, &PbType::Quad);
                    let hi = self.compile_expr(fb, &call.args[3])?;
                    let hi64 = self.convert_value(fb, &hi, &IrType::I64, &PbType::Quad);
                    fb.call_void("pb_control_set_option", &[hdlg64, id64, lo64, hi64]);
                }
                return Ok(());
            }
            "CONTROL_ADD_IMAGE"
            | "CONTROL_ADD_IMAGEX"
            | "CONTROL_ADD_IMGBUTTON"
            | "CONTROL_ADD_IMGBUTTONX" => {
                // batch 178: the resource-image family.  All four statements
                // share one runtime helper; the two compile-time flags say
                // whether the control is a BUTTON and whether it is the X
                // (resize-to-fit) form.  The style and exstyle clauses are not
                // reachable from the parser, so -1 is passed for both and the
                // documented defaults apply.
                if call.args.len() >= 8 {
                    let mut hwnd = self.compile_expr(fb, &call.args[0])?;
                    if hwnd.ty != IrType::Ptr {
                        hwnd = fb.inttoptr(&hwnd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let image = self.compile_expr(fb, &call.args[2])?;
                    let x = self.compile_expr(fb, &call.args[3])?;
                    let y = self.compile_expr(fb, &call.args[4])?;
                    let w = self.compile_expr(fb, &call.args[5])?;
                    let h = self.compile_expr(fb, &call.args[6])?;
                    let is_button = if call.name.contains("IMGBUTTON") {
                        1
                    } else {
                        0
                    };
                    let is_x = if call.name.ends_with('X') { 1 } else { 0 };
                    let style = fb.const_i32(-1);
                    let exstyle = fb.const_i32(-1);
                    let flag_button = fb.const_i32(is_button);
                    let flag_x = fb.const_i32(is_x);
                    let hc = fb.call(
                        &IrType::Ptr,
                        "pb_control_add_image",
                        &[
                            hwnd,
                            id,
                            image,
                            x,
                            y,
                            w,
                            h,
                            style,
                            exstyle,
                            flag_button,
                            flag_x,
                        ],
                    );
                    if let Some(t) = call.args.get(7) {
                        if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, t) {
                            let cv = self.convert_value(fb, &hc, &ty, &pty);
                            fb.store(&cv, &ptr);
                        }
                    }
                }
                return Ok(());
            }
            "CONTROL_SET_IMAGE"
            | "CONTROL_SET_IMAGEX"
            | "CONTROL_SET_IMGBUTTON"
            | "CONTROL_SET_IMGBUTTONX" => {
                // CONTROL SET <IMAGE|IMAGEX|IMGBUTTON|IMGBUTTONX> hDlg, id&, newimage$
                // (batch 178).  The helper reads the format already displayed
                // off the control, releases the image it replaces, and scales
                // the new one for the X forms.
                if call.args.len() >= 3 {
                    let mut hd = self.compile_expr(fb, &call.args[0])?;
                    if hd.ty != IrType::Ptr {
                        hd = fb.inttoptr(&hd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let image = self.compile_expr(fb, &call.args[2])?;
                    let is_button = if call.name.contains("IMGBUTTON") {
                        1
                    } else {
                        0
                    };
                    let flag_button = fb.const_i32(is_button);
                    let flag_x = fb.const_i32(if call.name.ends_with('X') { 1 } else { 0 });
                    fb.call_void(
                        "pb_control_set_image",
                        &[hd, id, image, flag_button, flag_x],
                    );
                }
                return Ok(());
            }
            "CONTROL_ADD_GROUPBOX" => {
                // batch 220: the official syntax requires all 8 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 8 {
                    return Err(pb::error::PbError::parser(
                        "CONTROL_ADD_GROUPBOX requires its arguments: CONTROL ADD GROUPBOX hDlg , id& , txt$ , x , y , xx , yy TO hCtrl&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 8 {
                    let mut hwnd = self.compile_expr(fb, &call.args[0])?;
                    if hwnd.ty != IrType::Ptr {
                        hwnd = fb.inttoptr(&hwnd);
                    }
                    let id = self.compile_expr(fb, &call.args[1])?;
                    let text = self.compile_expr(fb, &call.args[2])?;
                    let x = self.compile_expr(fb, &call.args[3])?;
                    let y = self.compile_expr(fb, &call.args[4])?;
                    let w = self.compile_expr(fb, &call.args[5])?;
                    let h = self.compile_expr(fb, &call.args[6])?;
                    // batch 249: full official syntax (mirrors BUTTON)
                    let target = if call.args.len() == 8 {
                        Some(call.args[7].clone())
                    } else {
                        call.args.get(10).cloned()
                    };
                    let hctrl = if call.args.len() == 8 {
                        fb.call(
                            &IrType::Ptr,
                            "pb_control_add_groupbox",
                            &[hwnd, id, text, x, y, w, h],
                        )
                    } else {
                        let style = self.compile_expr(fb, &call.args[7])?;
                        let exstyle = self.compile_expr(fb, &call.args[8])?;
                        let cb = self.compile_control_callback(fb, &call.args[9])?;
                        fb.call(
                            &IrType::Ptr,
                            "pb_control_add_groupbox_ex",
                            &[hwnd, id, text, x, y, w, h, style, exstyle, cb],
                        )
                    };
                    if let Some(tgt) = target {
                        if let Some((ptr, ty, pty)) = self.lvalue_ptr(fb, &tgt) {
                            let cv = self.convert_value(fb, &hctrl, &ty, &pty);
                            fb.store(&cv, &ptr);
                        }
                    }
                }
                return Ok(());
            }
            "TYPE SET" => {
                // TYPE SET dest = src : copy bytes into a TYPE variable.
                // src may be another TYPE variable or a STRING.
                if let (Some(dest_expr), Some(src_expr)) = (call.args.first(), call.args.get(1)) {
                    if let Some((dest_ptr, _, PbType::UserDefined(type_name))) =
                        self.lvalue_ptr(fb, dest_expr)
                    {
                        let norm = normalize_name(&type_name);
                        let size = if let Some(layout) = self.type_layouts.get(&norm) {
                            layout
                                .fields
                                .iter()
                                .map(|f| Self::ir_size_bytes(&f.ir_type))
                                .sum::<u32>() as i32
                        } else {
                            0
                        };
                        if size > 0 {
                            // src is another TYPE variable -> raw memcpy
                            if let Some((src_ptr, _, src_pb)) = self.lvalue_ptr(fb, src_expr) {
                                if matches!(src_pb, PbType::UserDefined(_)) {
                                    fb.call_void(
                                        "pb_type_set",
                                        &[dest_ptr.clone(), src_ptr, fb.const_i32(size)],
                                    );
                                    return Ok(());
                                }
                            }
                            // otherwise treat src as a STRING (payload pointer)
                            let sv = self.compile_expr(fb, src_expr)?;
                            fb.call_void("pb_type_set_str", &[dest_ptr, sv, fb.const_i32(size)]);
                        }
                    }
                }
                return Ok(());
            }
            "ARRAY UNIQUE" => {
                // ARRAY UNIQUE arr() [FOR count] — in-place dedup, returns new count
                if let Some(Expr::FunctionCall(arr_name, _)) = call.args.first() {
                    let an = normalize_name(arr_name);
                    if let Some(arr_info) = self.symbols.lookup_array(&an).cloned() {
                        let base = Val::new(arr_info.ptr_name.clone(), IrType::Ptr);
                        let elem_size = match &arr_info.elem_ir_type {
                            IrType::I8 | IrType::I1 => 1,
                            IrType::I16 => 2,
                            IrType::I32 | IrType::Float => 4,
                            IrType::I64 | IrType::Double | IrType::Ptr => 8,
                            _ => 4,
                        };
                        let is_string = if arr_info.elem_ir_type == IrType::Ptr {
                            1
                        } else {
                            0
                        };
                        let start = if let Some(cx) = call.args.get(1) {
                            let cv = self.compile_expr(fb, cx)?;
                            self.to_i64(fb, &cv)
                        } else {
                            fb.const_i64(0)
                        };
                        // start is 1-based in PB; ignore range for now, scan whole array
                        let _ = start;
                        fb.call_void(
                            "pb_array_unique",
                            &[
                                base,
                                fb.const_i32(elem_size),
                                fb.const_i64(arr_info.total_elements as i64),
                                fb.const_i32(is_string),
                            ],
                        );
                    }
                }
                return Ok(());
            }
            "ARRAY SORT" => {
                // ARRAY SORT arr() [FOR n] [, DESCEND|ASCEND]
                if let Some(Expr::FunctionCall(arr_name, _)) = call.args.first() {
                    let name = normalize_name(arr_name);
                    if let Some(arr_info) = self.symbols.lookup_array(&name).cloned() {
                        let base = Val::new(arr_info.ptr_name.clone(), IrType::Ptr);
                        let (elem_size, sort_type) = match &arr_info.elem_ir_type {
                            IrType::I8 | IrType::I1 => (1, 0),
                            IrType::I16 => (2, 0),
                            IrType::I32 => (4, 0),
                            IrType::Float => (4, 4),
                            IrType::I64 => (8, 1),
                            IrType::Double => (8, 2),
                            IrType::Ptr => (8, 3),
                            _ => (4, 0),
                        };
                        fb.call_void(
                            "pb_array_sort",
                            &[
                                base,
                                fb.const_i32(elem_size),
                                fb.const_i32(arr_info.total_elements as i32),
                                fb.const_i32(sort_type),
                            ],
                        );
                    }
                }
                return Ok(());
            }
            "LSET" | "RSET" => {
                // LSET target$ = value  → pb_lset(&target, value, len)
                if call.args.len() >= 2 {
                    let value = self.compile_expr(fb, &call.args[1])?;
                    let (ptr, pb_type) = self.compile_lvalue_ptr(fb, &call.args[0])?;
                    let len = match pb_type {
                        PbType::FixedString(n) => n as i32,
                        _ => 256,
                    };
                    let is_fixed = matches!(pb_type, PbType::FixedString(_));
                    let len_const = fb.const_i32(len);
                    if is_fixed {
                        if name == "LSET" {
                            fb.call_void("pb_lset_buf", &[ptr, value, len_const]);
                        } else {
                            fb.call_void("pb_rset_buf", &[ptr, value, len_const]);
                        }
                    } else if name == "LSET" {
                        fb.call_void("pb_lset", &[ptr, value, len_const]);
                    } else {
                        fb.call_void("pb_rset", &[ptr, value, len_const]);
                    }
                }
                return Ok(());
            }
            "CSET" => {
                // CSET target$ = value → pb_cset(&target, value, len)
                if call.args.len() >= 2 {
                    let value = self.compile_expr(fb, &call.args[1])?;
                    let (ptr, pb_type) = self.compile_lvalue_ptr(fb, &call.args[0])?;
                    let len = match pb_type {
                        PbType::FixedString(n) => n as i32,
                        _ => 256,
                    };
                    let len_const = fb.const_i32(len);
                    if matches!(pb_type, PbType::FixedString(_)) {
                        fb.call_void("pb_cset_buf", &[ptr, value, len_const]);
                    } else {
                        fb.call_void("pb_cset", &[ptr, value, len_const]);
                    }
                }
                return Ok(());
            }
            "RESET" => {
                fb.call_void("pb_reset", &[]);
                return Ok(());
            }
            "PLAY WAVE" => {
                // PLAY WAVE "file.wav" — play synchronously via PlaySoundA
                if let Some(arg) = call.args.first() {
                    let path = self.compile_expr(fb, arg)?;
                    fb.call_void("pb_play_wave", &[path]);
                }
                return Ok(());
            }
            "CHDRIVE" => {
                // CHDRIVE drv$ — change current drive
                if let Some(arg) = call.args.first() {
                    let drv = self.compile_expr(fb, arg)?;
                    fb.call_void("pb_chdrive", &[drv]);
                }
                return Ok(());
            }
            "SETEOF" => {
                // SETEOF #f — truncate file at current position
                if let Some(arg) = call.args.first() {
                    let sv = self.compile_expr(fb, arg)?;
                    let f = self.to_i32(fb, &sv);
                    fb.call_void("pb_seteof", &[f]);
                }
                return Ok(());
            }
            "SWAP" => {
                // SWAP var1, var2 — exchange two variables of the same type
                if call.args.len() == 2 {
                    if let (Some((ptr1, ty1, _)), Some((ptr2, ty2, _))) = (
                        self.lvalue_ptr(fb, &call.args[0]),
                        self.lvalue_ptr(fb, &call.args[1]),
                    ) {
                        let v1 = fb.load(&ty1, &ptr1);
                        let v2 = fb.load(&ty2, &ptr2);
                        fb.store(&v2, &ptr1);
                        fb.store(&v1, &ptr2);
                    }
                }
                return Ok(());
            }
            "SHIFT LEFT" | "SHIFT RIGHT" | "SHIFT SIGNED LEFT" | "SHIFT SIGNED RIGHT"
            | "ROTATE LEFT" | "ROTATE RIGHT" => {
                // SHIFT [SIGNED] {LEFT|RIGHT} ivar, count / ROTATE {LEFT|RIGHT} ivar, count
                if call.args.len() == 2 {
                    if let Some((ptr, ty, pb_ty)) = self.lvalue_ptr(fb, &call.args[0]) {
                        let val = fb.load(&ty, &ptr);
                        let v = self.to_i64(fb, &val);
                        let cnt = self.compile_expr(fb, &call.args[1])?;
                        let n = self.to_i32(fb, &cnt);
                        let res = match name.as_str() {
                            "SHIFT LEFT" | "SHIFT SIGNED LEFT" => {
                                fb.call(&IrType::I64, "pb_shift_left", &[v, n])
                            }
                            "SHIFT RIGHT" => {
                                let keep = fb.const_i32(0);
                                fb.call(&IrType::I64, "pb_shift_right", &[v, n, keep])
                            }
                            "SHIFT SIGNED RIGHT" => {
                                let keep = fb.const_i32(1);
                                fb.call(&IrType::I64, "pb_shift_right", &[v, n, keep])
                            }
                            "ROTATE LEFT" => fb.call(&IrType::I64, "pb_rotate_left", &[v, n]),
                            _ => fb.call(&IrType::I64, "pb_rotate_right", &[v, n]),
                        };
                        let back = self.convert_value(fb, &res, &ty, &pb_ty);
                        fb.store(&back, &ptr);
                    }
                }
                return Ok(());
            }
            "ARRAY REVERSE" => {
                // ARRAY REVERSE arr() — reverse all elements in place
                if let Some(arr_expr) = call.args.first() {
                    let arr_name = match arr_expr {
                        Expr::FunctionCall(n, _) => Some(n.as_str()),
                        Expr::Variable(n) => Some(n.as_str()),
                        _ => None,
                    };
                    if let Some(an) = arr_name {
                        let name = normalize_name(an);
                        if let Some(arr_info) = self.symbols.lookup_array(&name).cloned() {
                            let base = Val::new(arr_info.ptr_name.clone(), IrType::Ptr);
                            let elem_size = match &arr_info.elem_ir_type {
                                IrType::I8 | IrType::I1 => 1,
                                IrType::I16 => 2,
                                IrType::I32 | IrType::Float => 4,
                                IrType::I64 | IrType::Double | IrType::Ptr => 8,
                                _ => 4,
                            };
                            fb.call_void(
                                "pb_array_reverse",
                                &[
                                    base,
                                    fb.const_i32(elem_size),
                                    fb.const_i32(arr_info.total_elements as i32),
                                ],
                            );
                        }
                    }
                }
                return Ok(());
            }
            "PUT_STR" => {
                // batch 220: the official syntax requires all 2 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 2 {
                    return Err(pb::error::PbError::parser(
                        "PUT_STR requires its arguments: PUT # filenum& , StrgExpr".to_string(),
                        None,
                        call.line,
                    ));
                }

                // PUT$ [#] filenum&, StrgExpr — write ANSI string at file position
                if call.args.len() >= 2 {
                    let sv = self.compile_expr(fb, &call.args[0])?;
                    let f = self.to_i32(fb, &sv);
                    let str_val = self.compile_expr(fb, &call.args[1])?;
                    fb.call_void("pb_put_string", &[f, str_val]);
                }
                return Ok(());
            }
            "GET_STR" => {
                // batch 220: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                        "GET_STR requires its arguments: GET # filenum& , Count& , StrgVar"
                            .to_string(),
                        None,
                        call.line,
                    ));
                }

                // GET$ [#] filenum&, Count&, StrgVar — read Count bytes into a string var
                if call.args.len() >= 3 {
                    let fv = self.compile_expr(fb, &call.args[0])?;
                    let f = self.to_i32(fb, &fv);
                    let cv = self.compile_expr(fb, &call.args[1])?;
                    let count = self.to_i64(fb, &cv);
                    let (ptr, _) = self.compile_lvalue_ptr(fb, &call.args[2])?;
                    fb.call_void("pb_get_string", &[f, count, ptr]);
                }
                return Ok(());
            }
            "PUT_WSTR" => {
                // batch 220: the official syntax requires all 2 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 2 {
                    return Err(pb::error::PbError::parser(
                        "PUT_WSTR requires its arguments: PUT$$ # filenum& , StrgExpr".to_string(),
                        None,
                        call.line,
                    ));
                }

                // PUT$$ [#] filenum&, StrgExpr — write WIDE (UTF-16LE) string
                if call.args.len() >= 2 {
                    let sv = self.compile_expr(fb, &call.args[0])?;
                    let f = self.to_i32(fb, &sv);
                    let str_val = self.compile_expr(fb, &call.args[1])?;
                    fb.call_void("pb_put_wstring", &[f, str_val]);
                }
                return Ok(());
            }
            "GET_WSTR" => {
                // batch 220: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                        "GET_WSTR requires its arguments: GET$$ # filenum& , Count& , StrgVar"
                            .to_string(),
                        None,
                        call.line,
                    ));
                }

                // GET$$ [#] filenum&, Count&, StrgVar — read Count WIDE chars
                if call.args.len() >= 3 {
                    let fv = self.compile_expr(fb, &call.args[0])?;
                    let f = self.to_i32(fb, &fv);
                    let cv = self.compile_expr(fb, &call.args[1])?;
                    let count = self.to_i64(fb, &cv);
                    let (ptr, _) = self.compile_lvalue_ptr(fb, &call.args[2])?;
                    fb.call_void("pb_get_wstring", &[f, count, ptr]);
                }
                return Ok(());
            }
            "DIR CLOSE" => {
                fb.call_void("pb_dir_close", &[]);
                return Ok(());
            }
            "DIR NEXT" => {
                // DIR NEXT TO s$ - next matching filename
                if let Some(target) = call.args.first() {
                    let (ptr, _) = self.compile_lvalue_ptr(fb, target)?;
                    let r = fb.call(&IrType::Ptr, "pb_dir_next", &[]);
                    fb.store(&r, &ptr);
                }
                return Ok(());
            }
            "DIR" => {
                // batch 220: the official syntax requires all 2 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 2 {
                    return Err(pb::error::PbError::parser(
                        "DIR requires its arguments: DIR mask [, [ONLY] attr& ] TO s$".to_string(),
                        None,
                        call.line,
                    ));
                }

                // DIR mask [, [ONLY] attr] TO s$
                // args: [mask, target] | [mask, attr, target] | [mask, only, attr, target]
                if call.args.len() >= 2 {
                    let mask = self.compile_expr(fb, &call.args[0])?;
                    let (only, attr) = match call.args.len() {
                        4 => {
                            let av = self.compile_expr(fb, &call.args[2])?;
                            (fb.const_i32(1), self.to_i32(fb, &av))
                        }
                        3 => {
                            let av = self.compile_expr(fb, &call.args[1])?;
                            (fb.const_i32(0), self.to_i32(fb, &av))
                        }
                        _ => (fb.const_i32(0), fb.const_i32(0)),
                    };
                    let target = call.args.last().unwrap();
                    let (ptr, _) = self.compile_lvalue_ptr(fb, target)?;
                    let r = fb.call(&IrType::Ptr, "pb_dir_first", &[mask, only, attr]);
                    fb.store(&r, &ptr);
                }
                return Ok(());
            }
            "DESKTOP GET SIZE" => {
                // batch 220: the official syntax requires all 2 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 2 {
                    return Err(pb::error::PbError::parser(
                        "DESKTOP GET SIZE requires its arguments: DESKTOP GET SIZE TO ncWidth& , ncHeight&".to_string(),
                        None,
                        call.line,
                    ));
                }

                // DESKTOP GET SIZE TO ncWidth&, ncHeight&
                if call.args.len() >= 2 {
                    let (wp, _) = self.compile_lvalue_ptr(fb, &call.args[0])?;
                    let (hp, _) = self.compile_lvalue_ptr(fb, &call.args[1])?;
                    let w = fb.call(&IrType::I32, "GetSystemMetrics", &[fb.const_i32(0)]);
                    let h = fb.call(&IrType::I32, "GetSystemMetrics", &[fb.const_i32(1)]);
                    fb.store(&w, &wp);
                    fb.store(&h, &hp);
                }
                return Ok(());
            }
            "DESKTOP GET CLIENT" => {
                // batch 220: the official syntax requires all 2 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 2 {
                    return Err(pb::error::PbError::parser(
                        "DESKTOP GET CLIENT requires its arguments: DESKTOP GET CLIENT TO ncWidth& , ncHeight&".to_string(),
                        None,
                        call.line,
                    ));
                }

                // DESKTOP GET CLIENT TO ncWidth&, ncHeight& (work area)
                if call.args.len() >= 2 {
                    let (wp, _) = self.compile_lvalue_ptr(fb, &call.args[0])?;
                    let (hp, _) = self.compile_lvalue_ptr(fb, &call.args[1])?;
                    fb.call_void("pb_desktop_get_client", &[wp, hp]);
                }
                return Ok(());
            }
            "DESKTOP GET LOC" => {
                // batch 220: the official syntax requires all 2 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 2 {
                    return Err(pb::error::PbError::parser(
                        "DESKTOP GET LOC requires its arguments: DESKTOP GET LOC TO x& , y&"
                            .to_string(),
                        None,
                        call.line,
                    ));
                }

                // DESKTOP GET LOC TO x&, y& (work area origin)
                if call.args.len() >= 2 {
                    let (xp, _) = self.compile_lvalue_ptr(fb, &call.args[0])?;
                    let (yp, _) = self.compile_lvalue_ptr(fb, &call.args[1])?;
                    fb.call_void("pb_desktop_get_loc", &[xp, yp]);
                }
                return Ok(());
            }
            "DESKTOP GET PPI" => {
                // batch 220: the official syntax requires all 2 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 2 {
                    return Err(pb::error::PbError::parser(
                        "DESKTOP GET PPI requires its arguments: DESKTOP GET PPI TO x& , y&"
                            .to_string(),
                        None,
                        call.line,
                    ));
                }

                // DESKTOP GET PPI TO x&, y& (pixels per inch)
                if call.args.len() >= 2 {
                    let (xp, _) = self.compile_lvalue_ptr(fb, &call.args[0])?;
                    let (yp, _) = self.compile_lvalue_ptr(fb, &call.args[1])?;
                    fb.call_void("pb_desktop_get_ppi", &[xp, yp]);
                }
                return Ok(());
            }
            "GLOBALMEM ALLOC" => {
                if call.args.len() < 2 {
                    // batch 218: the official syntax always carries a `TO <var>` target.
                    // Without it the old code fell straight through and the statement
                    // became a silent no-op; malformed input must be loud instead.
                    return Err(pb::error::PbError::parser(
                        "GLOBALMEM ALLOC requires its destination: GLOBALMEM ALLOC count TO vHndl"
                            .to_string(),
                        None,
                        call.line,
                    ));
                }
                // GLOBALMEM ALLOC count TO vHndl
                if call.args.len() >= 2 {
                    let cv = self.compile_expr(fb, &call.args[0])?;
                    let n = self.to_i32(fb, &cv);
                    let h = fb.call(&IrType::I32, "pb_globalmem_alloc", &[n]);
                    let (ptr, _) = self.compile_lvalue_ptr(fb, &call.args[1])?;
                    fb.store(&h, &ptr);
                }
                return Ok(());
            }
            "GLOBALMEM FREE" => {
                if call.args.len() < 2 {
                    // batch 218: the official syntax always carries a `TO <var>` target.
                    // Without it the old code fell straight through and the statement
                    // became a silent no-op; malformed input must be loud instead.
                    return Err(pb::error::PbError::parser(
                        "GLOBALMEM FREE requires its destination: GLOBALMEM FREE mHndl TO vHndl"
                            .to_string(),
                        None,
                        call.line,
                    ));
                }
                // GLOBALMEM FREE mHndl TO vHndl
                if call.args.len() >= 2 {
                    let cv = self.compile_expr(fb, &call.args[0])?;
                    let m = self.to_i32(fb, &cv);
                    let h = fb.call(&IrType::I32, "pb_globalmem_free", &[m]);
                    let (ptr, _) = self.compile_lvalue_ptr(fb, &call.args[1])?;
                    fb.store(&h, &ptr);
                }
                return Ok(());
            }
            "GLOBALMEM LOCK" => {
                if call.args.len() < 2 {
                    // batch 218: the official syntax always carries a `TO <var>` target.
                    // Without it the old code fell straight through and the statement
                    // became a silent no-op; malformed input must be loud instead.
                    return Err(pb::error::PbError::parser(
                        "GLOBALMEM LOCK requires its destination: GLOBALMEM LOCK mHndl TO vPtr"
                            .to_string(),
                        None,
                        call.line,
                    ));
                }
                // GLOBALMEM LOCK mHndl TO vPtr
                if call.args.len() >= 2 {
                    let cv = self.compile_expr(fb, &call.args[0])?;
                    let m = self.to_i32(fb, &cv);
                    let p = fb.call(&IrType::Ptr, "pb_globalmem_lock", &[m]);
                    let (ptr, ty) = self.compile_lvalue_ptr(fb, &call.args[1])?;
                    // PBWin10 stores a truncated pointer into a LONG; the
                    // full 64-bit address is only usable from a QUAD target
                    // (needed for POKE in x64 builds). Store at the target's
                    // own width so neither spills nor truncates wrongly.
                    match ty {
                        PbType::Quad => {
                            let p64 = fb.ptrtoint64(&p);
                            fb.store(&p64, &ptr);
                        }
                        _ => {
                            let p32 = fb.ptrtoint(&p);
                            fb.store(&p32, &ptr);
                        }
                    }
                }
                return Ok(());
            }
            "GLOBALMEM SIZE" => {
                if call.args.len() < 2 {
                    // batch 218: the official syntax always carries a `TO <var>` target.
                    // Without it the old code fell straight through and the statement
                    // became a silent no-op; malformed input must be loud instead.
                    return Err(pb::error::PbError::parser(
                        "GLOBALMEM SIZE requires its destination: GLOBALMEM SIZE mHndl TO vSize"
                            .to_string(),
                        None,
                        call.line,
                    ));
                }
                // GLOBALMEM SIZE mHndl TO vSize
                if call.args.len() >= 2 {
                    let cv = self.compile_expr(fb, &call.args[0])?;
                    let m = self.to_i32(fb, &cv);
                    let sz = fb.call(&IrType::I32, "pb_globalmem_size", &[m]);
                    let (ptr, _) = self.compile_lvalue_ptr(fb, &call.args[1])?;
                    fb.store(&sz, &ptr);
                }
                return Ok(());
            }
            "GLOBALMEM UNLOCK" => {
                if call.args.len() < 2 {
                    // batch 218: the official syntax always carries a `TO <var>` target.
                    // Without it the old code fell straight through and the statement
                    // became a silent no-op; malformed input must be loud instead.
                    return Err(pb::error::PbError::parser(
                        "GLOBALMEM UNLOCK requires its destination: GLOBALMEM UNLOCK mHndl TO vLocked"
                            .to_string(),
                        None,
                        call.line,
                    ));
                }
                // GLOBALMEM UNLOCK mHndl TO vLocked
                if call.args.len() >= 2 {
                    let cv = self.compile_expr(fb, &call.args[0])?;
                    let m = self.to_i32(fb, &cv);
                    let lk = fb.call(&IrType::I32, "pb_globalmem_unlock", &[m]);
                    let (ptr, _) = self.compile_lvalue_ptr(fb, &call.args[1])?;
                    fb.store(&lk, &ptr);
                }
                return Ok(());
            }
            "MOUSEPTR" => {
                // MOUSEPTR style [TO var&]
                if !call.args.is_empty() {
                    let cv = self.compile_expr(fb, &call.args[0])?;
                    let st = self.to_i32(fb, &cv);
                    let r = fb.call(&IrType::I32, "pb_mouseptr", &[st]);
                    if call.args.len() >= 2 {
                        let (ptr, _) = self.compile_lvalue_ptr(fb, &call.args[1])?;
                        fb.store(&r, &ptr);
                    }
                }
                return Ok(());
            }
            "UCODEPAGE" => {
                // UCODEPAGE ANSI|OEM|NumExpr [TO PrevPage&]
                if !call.args.is_empty() {
                    let cv = self.compile_expr(fb, &call.args[0])?;
                    let cp = self.to_i32(fb, &cv);
                    let old = fb.call(&IrType::I32, "pb_ucodepage", &[cp]);
                    if call.args.len() >= 2 {
                        let (ptr, _) = self.compile_lvalue_ptr(fb, &call.args[1])?;
                        fb.store(&old, &ptr);
                    }
                }
                return Ok(());
            }

            "PLAY SOUND" => {
                // batch 220: the official syntax requires all 2 arguments, so a shorter

                // spelling is malformed. Report it instead of silently doing nothing.

                if call.args.len() < 2 {
                    return Err(pb::error::PbError::parser(
                        "PLAY SOUND requires its arguments: PLAY SOUND freq& , duration&"
                            .to_string(),
                        None,
                        call.line,
                    ));
                }

                // PLAY SOUND freq&, duration& — speaker beep
                if call.args.len() >= 2 {
                    let fv = self.compile_expr(fb, &call.args[0])?;
                    let freq = self.to_i32(fb, &fv);
                    let dv = self.compile_expr(fb, &call.args[1])?;
                    let dur = self.to_i32(fb, &dv);
                    fb.call_void("pb_play_sound", &[freq, dur]);
                }
                return Ok(());
            }
            "SPLIT" => {
                // SPLIT [WORD] MainStr, Part1Len TO Part1Var, Part2Var
                if call.args.len() == 4 {
                    let src = self.compile_expr(fb, &call.args[0])?;
                    let len_v = self.compile_expr(fb, &call.args[1])?;
                    let n = self.to_i32(fb, &len_v);
                    if let (Some((p1, _, _)), Some((p2, _, _))) = (
                        self.lvalue_ptr(fb, &call.args[2]),
                        self.lvalue_ptr(fb, &call.args[3]),
                    ) {
                        fb.call_void("pb_split", &[src, n, p1, p2]);
                    }
                }
                return Ok(());
            }
            "ARRAY SCAN =" | "ARRAY SCAN <>" | "ARRAY SCAN <" | "ARRAY SCAN >"
            | "ARRAY SCAN <=" | "ARRAY SCAN >=" => {
                // ARRAY SCAN arr(), OP expr, TO var&
                let op = match name.as_str() {
                    "ARRAY SCAN <>" => 1,
                    "ARRAY SCAN <" => 2,
                    "ARRAY SCAN >" => 3,
                    "ARRAY SCAN <=" => 4,
                    "ARRAY SCAN >=" => 5,
                    _ => 0,
                };
                if call.args.len() >= 3 {
                    if let Some(Expr::FunctionCall(arr_name, _)) = call.args.first() {
                        let an = normalize_name(arr_name);
                        if let Some(arr_info) = self.symbols.lookup_array(&an).cloned() {
                            let base = Val::new(arr_info.ptr_name.clone(), IrType::Ptr);
                            let elem_size = match &arr_info.elem_ir_type {
                                IrType::I8 | IrType::I1 => 1,
                                IrType::I16 => 2,
                                IrType::I32 | IrType::Float => 4,
                                IrType::I64 | IrType::Double | IrType::Ptr => 8,
                                _ => 4,
                            };
                            let value = self.compile_expr(fb, &call.args[1])?;
                            let total = fb.const_i64(arr_info.total_elements as i64);
                            let result = if arr_info.elem_ir_type == IrType::Ptr {
                                fb.call(
                                    &IrType::I64,
                                    "pb_array_scan_str",
                                    &[base, total, fb.const_i64(1), fb.const_i64(0), value],
                                )
                            } else {
                                let v64 = self.to_i64(fb, &value);
                                fb.call(
                                    &IrType::I64,
                                    "pb_array_scan_num",
                                    &[
                                        base,
                                        fb.const_i32(elem_size),
                                        total,
                                        fb.const_i64(1),
                                        fb.const_i64(0),
                                        v64,
                                        fb.const_i32(op),
                                    ],
                                )
                            };
                            if let Some(tgt) = call.args.get(2) {
                                if let Some((ptr, ir_ty, pb_ty)) = self.lvalue_ptr(fb, tgt) {
                                    let conv = self.convert_value(fb, &result, &ir_ty, &pb_ty);
                                    fb.store(&conv, &ptr);
                                }
                            }
                        }
                    }
                }
                return Ok(());
            }
            "ARRAY INSERT" => {
                // ARRAY INSERT arr(index), value
                if let Some(Expr::FunctionCall(arr_name, indices)) = call.args.first() {
                    let an = normalize_name(arr_name);
                    if let Some(arr_info) = self.symbols.lookup_array(&an).cloned() {
                        let base = Val::new(arr_info.ptr_name.clone(), IrType::Ptr);
                        let elem_size = match &arr_info.elem_ir_type {
                            IrType::I8 | IrType::I1 => 1,
                            IrType::I16 => 2,
                            IrType::I32 | IrType::Float => 4,
                            IrType::I64 | IrType::Double | IrType::Ptr => 8,
                            _ => 4,
                        };
                        let idx = if let Some(ix) = indices.first() {
                            self.compile_expr(fb, ix)?
                        } else {
                            fb.const_i64(1)
                        };
                        let idx64 = if idx.ty == IrType::I32 {
                            self.to_i64(fb, &idx)
                        } else {
                            idx
                        };
                        if arr_info.elem_ir_type == IrType::Ptr {
                            let val = self.compile_expr(fb, &call.args[1])?;
                            fb.call_void(
                                "pb_array_insert_str",
                                &[
                                    base,
                                    fb.const_i64(arr_info.total_elements as i64),
                                    idx64,
                                    val,
                                ],
                            );
                        } else {
                            let val = self.compile_expr(fb, &call.args[1])?;
                            let v64 = self.to_i64(fb, &val);
                            fb.call_void(
                                "pb_array_insert_num",
                                &[
                                    base,
                                    fb.const_i32(elem_size),
                                    fb.const_i64(arr_info.total_elements as i64),
                                    idx64,
                                    v64,
                                ],
                            );
                        }
                    }
                }
                return Ok(());
            }
            "ARRAY ARRAYIX" => {
                // ARRAY ARRAYIX arr() — set each element to its element index
                if let Some(Expr::FunctionCall(arr_name, _)) = call.args.first() {
                    let an = normalize_name(arr_name);
                    if let Some(arr_info) = self.symbols.lookup_array(&an).cloned() {
                        let base = Val::new(arr_info.ptr_name.clone(), IrType::Ptr);
                        let elem_size = match &arr_info.elem_ir_type {
                            IrType::I8 | IrType::I1 => 1,
                            IrType::I16 => 2,
                            IrType::I32 | IrType::Float => 4,
                            IrType::I64 | IrType::Double | IrType::Ptr => 8,
                            _ => 4,
                        };
                        let is_string = if arr_info.elem_ir_type == IrType::Ptr {
                            1
                        } else {
                            0
                        };
                        fb.call_void(
                            "pb_array_arrayix",
                            &[
                                base,
                                fb.const_i32(elem_size),
                                fb.const_i64(arr_info.total_elements as i64),
                                fb.const_i32(is_string),
                            ],
                        );
                    }
                }
                return Ok(());
            }
            "HOST ADDR" => {
                // HOST ADDR [hostname$] TO ip&
                let host_ptr = if let Some(h) = call.args.first() {
                    self.compile_expr(fb, h)?
                } else {
                    let (n, _) = self.module.add_string_constant("");
                    Val::new(n, IrType::Ptr)
                };
                if let Some(t) = call.args.get(1) {
                    if let Some((ptr, _, _)) = self.lvalue_ptr(fb, t) {
                        fb.call_void("pb_host_addr", &[host_ptr, ptr]);
                    }
                }
                return Ok(());
            }
            "HOST NAME" => {
                // HOST NAME [ip&] TO hostname$
                let ip = if let Some(h) = call.args.first() {
                    let hv = self.compile_expr(fb, h)?;
                    self.to_i32(fb, &hv)
                } else {
                    fb.const_i32(0)
                };
                let val = fb.call(&IrType::Ptr, "pb_host_name", &[ip]);
                if let (Some(Expr::Variable(_orig)), Some(info)) = (
                    call.args.get(1),
                    call.args.get(1).and_then(|t| {
                        if let Expr::Variable(orig) = t {
                            let name = normalize_name(orig);
                            self.symbols.lookup(&name)
                        } else {
                            None
                        }
                    }),
                ) {
                    let ptr = Val::new(info.ptr_name.clone(), IrType::Ptr);
                    let converted =
                        self.convert_value(fb, &val, &info.ir_type.clone(), &info.pb_type.clone());
                    fb.store(&converted, &ptr);
                }
                return Ok(());
            }
            "TCP OPEN" => {
                // TCP OPEN [SERVER] {PORT p | srvc} [AT addr$] AS #f [TIMEOUT t]
                let _e0 = self.compile_expr(fb, &call.args[0])?;
                let mode = self.to_i32(fb, &_e0);
                let _e1 = self.compile_expr(fb, &call.args[1])?;
                let port = self.to_i32(fb, &_e1);
                let addr = self.compile_expr(fb, &call.args[2])?;
                let _e3 = self.compile_expr(fb, &call.args[3])?;
                let fnum = self.to_i32(fb, &_e3);
                let _e4 = self.compile_expr(fb, &call.args[4])?;
                let timeout = self.to_i32(fb, &_e4);
                fb.call_void("pb_tcp_open", &[mode, port, addr, fnum, timeout]);
                return Ok(());
            }
            "TCP ACCEPT" => {
                // TCP ACCEPT #srv AS newf  (newf is an OUTPUT variable: the
                // runtime allocates the accepted socket's file number into it)
                let _e0 = self.compile_expr(fb, &call.args[0])?;
                let srv = self.to_i32(fb, &_e0);
                if let Some((ptr, _, _)) = self.lvalue_ptr(fb, &call.args[1]) {
                    fb.call_void("pb_tcp_accept", &[srv, ptr]);
                }
                return Ok(());
            }
            "TCP SEND" => {
                let _e0 = self.compile_expr(fb, &call.args[0])?;
                let fnum = self.to_i32(fb, &_e0);
                let data = self.compile_expr(fb, &call.args[1])?;
                fb.call_void("pb_tcp_send", &[fnum, data]);
                return Ok(());
            }
            "TCP RECV" => {
                // TCP RECV #f, count&, var$
                let _e0 = self.compile_expr(fb, &call.args[0])?;
                let fnum = self.to_i32(fb, &_e0);
                let _e1 = self.compile_expr(fb, &call.args[1])?;
                let count = self.to_i32(fb, &_e1);
                if let Some((ptr, _, _)) = self.lvalue_ptr(fb, &call.args[2]) {
                    fb.call_void("pb_tcp_recv", &[fnum, count, ptr]);
                }
                return Ok(());
            }
            "TCP LINE INPUT" => {
                // TCP LINE INPUT #f, var$
                let _e0 = self.compile_expr(fb, &call.args[0])?;
                let fnum = self.to_i32(fb, &_e0);
                if let Some((ptr, _, _)) = self.lvalue_ptr(fb, &call.args[1]) {
                    fb.call_void("pb_tcp_line_input", &[fnum, ptr]);
                }
                return Ok(());
            }
            "TCP PRINT" => {
                // TCP PRINT #f, expr [;]
                let _e0 = self.compile_expr(fb, &call.args[0])?;
                let fnum = self.to_i32(fb, &_e0);
                let data = self.compile_expr(fb, &call.args[1])?;
                let _e2 = self.compile_expr(fb, &call.args[2])?;
                let nl = self.to_i32(fb, &_e2);
                fb.call_void("pb_tcp_print", &[fnum, data, nl]);
                return Ok(());
            }
            "TCP CLOSE" => {
                let _e0 = self.compile_expr(fb, &call.args[0])?;
                let fnum = self.to_i32(fb, &_e0);
                fb.call_void("pb_tcp_close", &[fnum]);
                return Ok(());
            }
            "UDP OPEN" => {
                // UDP OPEN [PORT p] AS #f [TIMEOUT t]
                let _e0 = self.compile_expr(fb, &call.args[0])?;
                let port = self.to_i32(fb, &_e0);
                let _e1 = self.compile_expr(fb, &call.args[1])?;
                let fnum = self.to_i32(fb, &_e1);
                let _e2 = self.compile_expr(fb, &call.args[2])?;
                let timeout = self.to_i32(fb, &_e2);
                fb.call_void("pb_udp_open", &[port, fnum, timeout]);
                return Ok(());
            }
            "UDP SEND" => {
                // UDP SEND #f, AT ip&, pNum&, data$  (ip may be LONG or string)
                let _e0 = self.compile_expr(fb, &call.args[0])?;
                let fnum = self.to_i32(fb, &_e0);
                let ip = self.compile_expr(fb, &call.args[1])?;
                let _e2 = self.compile_expr(fb, &call.args[2])?;
                let port = self.to_i32(fb, &_e2);
                let data = self.compile_expr(fb, &call.args[3])?;
                if ip.ty == IrType::Ptr {
                    fb.call_void("pb_udp_send_str", &[fnum, ip, port, data]);
                } else {
                    let ip32 = self.to_i32(fb, &ip);
                    fb.call_void("pb_udp_send", &[fnum, ip32, port, data]);
                }
                return Ok(());
            }
            "UDP RECV" => {
                // UDP RECV #f, FROM ip&, pNum&, buf$
                let _e0 = self.compile_expr(fb, &call.args[0])?;
                let fnum = self.to_i32(fb, &_e0);
                if let Some((ip_ptr, _, _)) = self.lvalue_ptr(fb, &call.args[1]) {
                    if let Some((port_ptr, _, _)) = self.lvalue_ptr(fb, &call.args[2]) {
                        if let Some((buf_ptr, _, _)) = self.lvalue_ptr(fb, &call.args[3]) {
                            fb.call_void("pb_udp_recv", &[fnum, ip_ptr, port_ptr, buf_ptr]);
                        }
                    }
                }
                return Ok(());
            }
            "UDP CLOSE" => {
                let _e0 = self.compile_expr(fb, &call.args[0])?;
                let fnum = self.to_i32(fb, &_e0);
                fb.call_void("pb_udp_close", &[fnum]);
                return Ok(());
            }
            "ARRAY DELETE" => {
                // ARRAY DELETE arr(index) [FOR count]
                if let Some(Expr::FunctionCall(arr_name, indices)) = call.args.first() {
                    let an = normalize_name(arr_name);
                    if let Some(arr_info) = self.symbols.lookup_array(&an).cloned() {
                        let base = Val::new(arr_info.ptr_name.clone(), IrType::Ptr);
                        let elem_size = match &arr_info.elem_ir_type {
                            IrType::I8 | IrType::I1 => 1,
                            IrType::I16 => 2,
                            IrType::I32 | IrType::Float => 4,
                            IrType::I64 | IrType::Double | IrType::Ptr => 8,
                            _ => 4,
                        };
                        let is_string = if arr_info.elem_ir_type == IrType::Ptr {
                            1
                        } else {
                            0
                        };
                        let idx = if let Some(ix) = indices.first() {
                            self.compile_expr(fb, ix)?
                        } else {
                            fb.const_i64(1)
                        };
                        let idx64 = if idx.ty == IrType::I32 {
                            self.to_i64(fb, &idx)
                        } else {
                            idx
                        };
                        let count = if let Some(cexpr) = call.args.get(1) {
                            let cv = self.compile_expr(fb, cexpr)?;
                            self.to_i64(fb, &cv)
                        } else {
                            fb.const_i64(1)
                        };
                        fb.call_void(
                            "pb_array_delete",
                            &[
                                base,
                                fb.const_i32(elem_size),
                                fb.const_i64(arr_info.total_elements as i64),
                                idx64,
                                count,
                                fb.const_i32(is_string),
                            ],
                        );
                    }
                }
                return Ok(());
            }
            "ARRAY SHUFFLE" => {
                // ARRAY SHUFFLE arr() — Fisher-Yates shuffle in place
                if let Some(arr_expr) = call.args.first() {
                    let arr_name = match arr_expr {
                        Expr::FunctionCall(n, _) => Some(n.as_str()),
                        Expr::Variable(n) => Some(n.as_str()),
                        _ => None,
                    };
                    if let Some(an) = arr_name {
                        let name = normalize_name(an);
                        if let Some(arr_info) = self.symbols.lookup_array(&name).cloned() {
                            let base = Val::new(arr_info.ptr_name.clone(), IrType::Ptr);
                            let elem_size = match &arr_info.elem_ir_type {
                                IrType::I8 | IrType::I1 => 1,
                                IrType::I16 => 2,
                                IrType::I32 | IrType::Float => 4,
                                IrType::I64 | IrType::Double | IrType::Ptr => 8,
                                _ => 4,
                            };
                            fb.call_void(
                                "pb_array_shuffle",
                                &[
                                    base,
                                    fb.const_i32(elem_size),
                                    fb.const_i32(arr_info.total_elements as i32),
                                ],
                            );
                        }
                    }
                }
                return Ok(());
            }
            "FLUSH" => {
                if let Some(arg) = call.args.first() {
                    let sv = self.compile_expr(fb, arg)?;
                    let f = self.to_i32(fb, &sv);
                    fb.call_void("pb_flush", &[f]);
                }
                return Ok(());
            }
            "NAME" => {
                // batch 220: the official syntax requires all 2 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 2 {
                    return Err(pb::error::PbError::parser(
                        "NAME requires its arguments: NAME filespec1$ AS filespec2$".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 2 {
                    let old = self.compile_expr(fb, &call.args[0])?;
                    let new_s = self.compile_expr(fb, &call.args[1])?;
                    fb.call_void("pb_name", &[old, new_s]);
                }
                return Ok(());
            }
            "WAITKEY" => {
                fb.call(&IrType::Ptr, "pb_waitkey", &[]);
                return Ok(());
            }
            "COMM OPEN" => {
                if call.args.len() >= 2 {
                    let port = self.compile_expr(fb, &call.args[0])?;
                    let sv0 = self.compile_expr(fb, &call.args[1])?;
                    let ch = self.to_i32(fb, &sv0);
                    let baud = if call.args.len() >= 3 {
                        let v = self.compile_expr(fb, &call.args[2])?;
                        self.to_i32(fb, &v)
                    } else {
                        fb.const_i32(0)
                    };
                    let parity = if call.args.len() >= 4 {
                        self.compile_expr(fb, &call.args[3])?
                    } else {
                        let (n, _) = self.module.add_string_constant("");
                        Val::new(n, IrType::Ptr)
                    };
                    let data = if call.args.len() >= 5 {
                        let v = self.compile_expr(fb, &call.args[4])?;
                        self.to_i32(fb, &v)
                    } else {
                        fb.const_i32(0)
                    };
                    let stop = if call.args.len() >= 6 {
                        let v = self.compile_expr(fb, &call.args[5])?;
                        self.to_i32(fb, &v)
                    } else {
                        fb.const_i32(0)
                    };
                    fb.call_void("pb_comm_open", &[port, ch, baud, parity, data, stop]);
                }
                return Ok(());
            }
            "COMM CLOSE" => {
                if let Some(arg) = call.args.first() {
                    let v = self.compile_expr(fb, arg)?;
                    let ch = self.to_i32(fb, &v);
                    fb.call_void("pb_comm_close", std::slice::from_ref(&ch));
                }
                return Ok(());
            }
            "COMM RESET" => {
                fb.call_void("pb_comm_reset", &[]);
                return Ok(());
            }
            "COMM LINE INPUT" => {
                // batch 220: the official syntax requires all 2 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 2 {
                    return Err(pb::error::PbError::parser(
                        "COMM LINE INPUT requires its arguments: COMM LINE [INPUT] [#] hComm , string_var".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 2 {
                    let v0 = self.compile_expr(fb, &call.args[0])?;
                    let ch = self.to_i32(fb, &v0);
                    if let Some((ptr, _, _)) = self.lvalue_ptr(fb, &call.args[1]) {
                        fb.call_void("pb_comm_line_input", &[ch, ptr]);
                    }
                }
                return Ok(());
            }
            "COMM RECV" => {
                // batch 220: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                        "COMM RECV requires its arguments: COMM RECV [#] hComm , count& , string_var".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 3 {
                    let v0 = self.compile_expr(fb, &call.args[0])?;
                    let ch = self.to_i32(fb, &v0);
                    let v1 = self.compile_expr(fb, &call.args[1])?;
                    let bytes = self.to_i64(fb, &v1);
                    if let Some((ptr, _, _)) = self.lvalue_ptr(fb, &call.args[2]) {
                        fb.call_void("pb_comm_recv", &[ch, bytes, ptr]);
                    }
                }
                return Ok(());
            }
            "COMM SEND" => {
                // batch 220: the official syntax requires all 2 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 2 {
                    return Err(pb::error::PbError::parser(
                        "COMM SEND requires its arguments: COMM SEND [#] hComm , string_expression TO [ CharCountVar ]".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 2 {
                    let v0 = self.compile_expr(fb, &call.args[0])?;
                    let ch = self.to_i32(fb, &v0);
                    let s = self.compile_expr(fb, &call.args[1])?;
                    fb.call_void("pb_comm_send", &[ch, s]);
                }
                return Ok(());
            }
            "COMM PRINT" => {
                if !call.args.is_empty() {
                    let v0 = self.compile_expr(fb, &call.args[0])?;
                    let ch = self.to_i32(fb, &v0);
                    for arg in call.args.iter().skip(1) {
                        let v = self.compile_expr(fb, arg)?;
                        match &v.ty {
                            IrType::I32 | IrType::I64 => {
                                let vi = self.to_i64(fb, &v);
                                fb.call_void("pb_comm_print_int", &[ch.clone(), vi]);
                            }
                            IrType::Double => fb.call_void("pb_comm_print_dbl", &[ch.clone(), v]),
                            _ => fb.call_void("pb_comm_print_str", &[ch.clone(), v]),
                        }
                    }
                }
                return Ok(());
            }
            "COMM SET" => {
                // batch 220: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                        "COMM SET requires its arguments: COMM SET [#] hComm , Comfunc = value"
                            .to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 3 {
                    let v0 = self.compile_expr(fb, &call.args[0])?;
                    let ch = self.to_i32(fb, &v0);
                    let opt = self.compile_expr(fb, &call.args[1])?;
                    let v2 = self.compile_expr(fb, &call.args[2])?;
                    let on = self.to_i32(fb, &v2);
                    fb.call_void("pb_comm_set", &[ch, opt, on]);
                }
                return Ok(());
            }
            "COMM TIMEOUT" => {
                // batch 220: the official syntax requires all 2 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 2 {
                    return Err(pb::error::PbError::parser(
                        "COMM TIMEOUT requires its arguments: COMM TIMEOUT [#] hComm , TimeOutMS&"
                            .to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 2 {
                    let v0 = self.compile_expr(fb, &call.args[0])?;
                    let ch = self.to_i32(fb, &v0);
                    let v1 = self.compile_expr(fb, &call.args[1])?;
                    let ms = self.to_i64(fb, &v1);
                    fb.call_void("pb_comm_timeout", &[ch, ms]);
                }
                return Ok(());
            }
            "THREAD CREATE" => {
                if !call.args.is_empty() {
                    let func_name = match &call.args[0] {
                        Expr::Variable(n) => normalize_name(n),
                        Expr::FunctionCall(n, _) => normalize_name(n),
                        _ => String::new(),
                    };
                    let ir_name = self
                        .functions
                        .get(&func_name)
                        .map(|i| i.ir_name.clone())
                        .or_else(|| self.subs.get(&func_name).map(|i| i.ir_name.clone()))
                        .unwrap_or_default();
                    if ir_name.is_empty() {
                        self.warnings.push(format!(
                            "line {}: THREAD CREATE: unknown thread function `{}`",
                            call.line, func_name
                        ));
                        return Ok(());
                    }
                    let fp = Val::new(format!("@{}", ir_name), IrType::Ptr);
                    if call.args.len() >= 2 {
                        if let Some((id_ptr, _, _)) = self.lvalue_ptr(fb, &call.args[1]) {
                            fb.call_void("pb_thread_create", &[fp, id_ptr]);
                        }
                    } else {
                        let nul = Val::new("null".to_string(), IrType::Ptr);
                        fb.call_void("pb_thread_create", &[fp, nul]);
                    }
                }
                return Ok(());
            }
            "THREAD CLOSE" | "THREAD SUSPEND" | "THREAD RESUME" => {
                if let Some(arg) = call.args.first() {
                    let v = self.compile_expr(fb, arg)?;
                    let id = self.to_i32(fb, &v);
                    let fn_name = match name.as_str() {
                        "THREAD CLOSE" => "pb_thread_close",
                        "THREAD SUSPEND" => "pb_thread_suspend",
                        _ => "pb_thread_resume",
                    };
                    fb.call_void(fn_name, std::slice::from_ref(&id));
                }
                return Ok(());
            }
            "THREAD STATUS" => {
                // batch 220: the official syntax requires all 2 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 2 {
                    return Err(pb::error::PbError::parser(
                        "THREAD STATUS requires its arguments: THREAD STATUS hThread TO lResult&"
                            .to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 2 {
                    let v0 = self.compile_expr(fb, &call.args[0])?;
                    let id = self.to_i32(fb, &v0);
                    if let Some((ptr, ir, pb)) = self.lvalue_ptr(fb, &call.args[1]) {
                        let rc = fb.call(&IrType::I32, "pb_thread_status", &[id]);
                        let converted = self.convert_value(fb, &rc, &ir, &pb);
                        fb.store(&converted, &ptr);
                    }
                }
                return Ok(());
            }
            "THREAD GET PRIORITY" => {
                // batch 220: the official syntax requires all 2 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 2 {
                    return Err(pb::error::PbError::parser(
                        "THREAD GET PRIORITY requires its arguments: THREAD GET PRIORITY hThread TO lResult&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 2 {
                    let v0 = self.compile_expr(fb, &call.args[0])?;
                    let id = self.to_i32(fb, &v0);
                    if let Some((ptr, ir, pb)) = self.lvalue_ptr(fb, &call.args[1]) {
                        let rc = fb.call(&IrType::I32, "pb_thread_get_priority", &[id]);
                        let converted = self.convert_value(fb, &rc, &ir, &pb);
                        fb.store(&converted, &ptr);
                    }
                }
                return Ok(());
            }
            "THREAD SET PRIORITY" => {
                // batch 220: the official syntax requires all 2 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 2 {
                    return Err(pb::error::PbError::parser(
                        "THREAD SET PRIORITY requires its arguments: THREAD SET PRIORITY hThread , Priority&".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 2 {
                    let v0 = self.compile_expr(fb, &call.args[0])?;
                    let id = self.to_i32(fb, &v0);
                    let v1 = self.compile_expr(fb, &call.args[1])?;
                    let p = self.to_i32(fb, &v1);
                    fb.call_void("pb_thread_set_priority", &[id, p]);
                }
                return Ok(());
            }
            "LPRINT" => {
                // LPRINT expr[,|;]... — send each value to the attached device
                for arg in &call.args {
                    let val = self.compile_expr(fb, arg)?;
                    if val.ty == IrType::Ptr {
                        fb.call_void("pb_lprint_bstr", std::slice::from_ref(&val));
                    } else if val.ty == IrType::I64 {
                        fb.call_void("pb_lprint_i64", std::slice::from_ref(&val));
                    } else if val.ty.is_int() {
                        let v = self.to_i32(fb, &val);
                        fb.call_void("pb_lprint_int", std::slice::from_ref(&v));
                    } else if val.ty.is_float() {
                        let v = self.to_f64(fb, &val);
                        fb.call_void("pb_lprint_dbl", std::slice::from_ref(&v));
                    }
                }
                fb.call_void("pb_lprint_crlf", &[]);
                return Ok(());
            }
            "LPRINT ATTACH" => {
                if let Some(arg) = call.args.first() {
                    let v = self.compile_expr(fb, arg)?;
                    fb.call_void("pb_lprint_attach", std::slice::from_ref(&v));
                }
                return Ok(());
            }
            "LPRINT CLOSE" => {
                fb.call_void("pb_lprint_close", &[]);
                return Ok(());
            }
            "LPRINT FLUSH" => {
                fb.call_void("pb_lprint_flush", &[]);
                return Ok(());
            }
            "LPRINT FORMFEED" => {
                fb.call_void("pb_lprint_formfeed", &[]);
                return Ok(());
            }
            "TRACE NEW" => {
                if let Some(arg) = call.args.first() {
                    let v = self.compile_expr(fb, arg)?;
                    fb.call_void("pb_trace_new", std::slice::from_ref(&v));
                }
                return Ok(());
            }
            "TRACE ON" => {
                fb.call_void("pb_trace_on", &[]);
                return Ok(());
            }
            "TRACE OFF" => {
                fb.call_void("pb_trace_off", &[]);
                return Ok(());
            }
            "TRACE PRINT" => {
                if let Some(arg) = call.args.first() {
                    let v = self.compile_expr(fb, arg)?;
                    if v.ty == IrType::Ptr {
                        fb.call_void("pb_trace_print", std::slice::from_ref(&v));
                    } else {
                        // Non-string: format with num_to_string, which returns a BSTR -
                        // a pointer that ALREADY addresses the character data (the 4-byte
                        // length sits at bstr[-4], SysAllocStringByteLen semantics).  The
                        // old code skipped a further 4 bytes, walked past the terminator
                        // and printed uninitialised heap: `TRACE PRINT 42` wrote "s".
                        let s = self.num_to_string(fb, &v);
                        fb.call_void("pb_trace_print", std::slice::from_ref(&s));
                    }
                }
                return Ok(());
            }
            "TRACE CLOSE" => {
                fb.call_void("pb_trace_close", &[]);
                return Ok(());
            }
            "IMPORT ADDR" => {
                // batch 220: the official syntax requires all 3 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 3 {
                    return Err(pb::error::PbError::parser(
                        "IMPORT ADDR requires its arguments: IMPORT ADDR ProcName$ , LibName$ TO AddrVar& [, HndlVar& ]".to_string(),
                        None,
                        call.line,
                    ));
                }

                if call.args.len() >= 2 {
                    let pv = self.compile_expr(fb, &call.args[0])?;
                    let lv = self.compile_expr(fb, &call.args[1])?;
                    let mut out_addr = Val::new("null".to_string(), IrType::Ptr);
                    let mut out_hndl = Val::new("null".to_string(), IrType::Ptr);
                    if call.args.len() >= 3 {
                        if let Some((ptr, _, _)) = self.lvalue_ptr(fb, &call.args[2]) {
                            out_addr = ptr;
                        }
                    }
                    if call.args.len() >= 4 {
                        if let Some((ptr, _, _)) = self.lvalue_ptr(fb, &call.args[3]) {
                            out_hndl = ptr;
                        }
                    }
                    fb.call_void(
                        "pb_import_addr",
                        &[pv, lv, out_addr.clone(), out_hndl.clone()],
                    );
                }
                return Ok(());
            }
            "IMPORT CLOSE" => {
                if let Some(arg) = call.args.first() {
                    let v = self.compile_expr(fb, arg)?;
                    let p = self.to_i64(fb, &v);
                    let ptr = fb.inttoptr(&p);
                    fb.call_void("pb_import_close", std::slice::from_ref(&ptr));
                }
                return Ok(());
            }
            "CALL DWORD" | "CALL DWORD TO" => {
                if let Some(arg) = call.args.first() {
                    let target = self.compile_expr(fb, arg)?;
                    // IMPORT ADDR stores a truncated 32-bit address in a PB DWORD;
                    // zero-extend so the upper half is clean before inttoptr.
                    let t64 = if target.ty == IrType::I32
                        || target.ty == IrType::I16
                        || target.ty == IrType::I8
                    {
                        fb.zext(&target, &IrType::I64)
                    } else {
                        self.to_i64(fb, &target)
                    };
                    let tptr = fb.inttoptr(&t64);
                    // args[1..] are the callee arguments; with "TO", the last one is the
                    // result target (a lvalue)
                    let has_result = name == "CALL DWORD TO";
                    let call_count = if has_result {
                        call.args.len().saturating_sub(2)
                    } else {
                        call.args.len().saturating_sub(1)
                    };
                    let mut callee_args: Vec<Val> = Vec::new();
                    for i in 1..=call_count {
                        let v = self.compile_expr(fb, &call.args[i])?;
                        callee_args.push(v);
                    }
                    if has_result && call.args.len() >= 2 {
                        if let Some((ptr, ir, pb)) =
                            self.lvalue_ptr(fb, &call.args[call.args.len() - 1])
                        {
                            if ir.is_int() || ir.is_float() {
                                let rc = fb.call_indirect(&ir, &tptr, &callee_args);
                                let conv = self.convert_value(fb, &rc, &ir, &pb);
                                fb.store(&conv, &ptr);
                            } else {
                                fb.call_indirect_void(&tptr, &callee_args);
                            }
                        }
                    } else {
                        fb.call_indirect_void(&tptr, &callee_args);
                    }
                }
                return Ok(());
            }
            "WRITE" => {
                if !call.args.is_empty() {
                    let sv0 = self.compile_expr(fb, &call.args[0])?;
                    let f = self.to_i32(fb, &sv0);
                    fb.call_void("pb_write_file_begin", std::slice::from_ref(&f));
                    for arg in call.args.iter().skip(1) {
                        let v = self.compile_expr(fb, arg)?;
                        match &v.ty {
                            IrType::I32 | IrType::I64 => {
                                let vi = self.to_i64(fb, &v);
                                fb.call_void("pb_write_file_int", &[f.clone(), vi]);
                            }
                            IrType::Double => fb.call_void("pb_write_file_dbl", &[f.clone(), v]),
                            _ => fb.call_void("pb_write_file_str", &[f.clone(), v]),
                        }
                    }
                    fb.call_void("pb_write_file_newline", &[f]);
                }
                return Ok(());
            }
            "SEEK" => {
                // batch 219: the official syntax requires all 2 arguments, so a shorter
                // spelling is malformed. Report it instead of silently doing nothing.
                if call.args.len() < 2 {
                    return Err(pb::error::PbError::parser(
                        "SEEK requires its arguments: SEEK [#] filenum& , position&&".to_string(),
                        None,
                        call.line,
                    ));
                }
                if call.args.len() >= 2 {
                    let sv0 = self.compile_expr(fb, &call.args[0])?;
                    let f = self.to_i32(fb, &sv0);
                    let sv1 = self.compile_expr(fb, &call.args[1])?;
                    let pos = self.to_i64(fb, &sv1);
                    fb.call_void("pb_seek", &[f, pos]);
                }
                return Ok(());
            }
            "FILESCAN" => {
                // FILESCAN [#] fnum&, RECORDS TO y& [, WIDTH TO x&]
                if !call.args.is_empty() {
                    let sv0 = self.compile_expr(fb, &call.args[0])?;
                    let f = self.to_i32(fb, &sv0);
                    let rec_tmp = fb.alloca(&IrType::I64);
                    let wid_tmp = fb.alloca(&IrType::I64);
                    fb.call_void("pb_filescan", &[f, rec_tmp.clone(), wid_tmp.clone()]);
                    if let Some(rec_arg) = call.args.get(1) {
                        if let Some((ptr, ir, pb)) = self.lvalue_ptr(fb, rec_arg) {
                            let v = fb.load(&IrType::I64, &rec_tmp);
                            let converted = self.convert_value(fb, &v, &ir, &pb);
                            fb.store(&converted, &ptr);
                        }
                    }
                    if let Some(wid_arg) = call.args.get(2) {
                        if let Some((ptr, ir, pb)) = self.lvalue_ptr(fb, wid_arg) {
                            let v = fb.load(&IrType::I64, &wid_tmp);
                            let converted = self.convert_value(fb, &v, &ir, &pb);
                            fb.store(&converted, &ptr);
                        }
                    }
                }
                return Ok(());
            }
            "LOCK" | "UNLOCK" => {
                if !call.args.is_empty() {
                    let sv0 = self.compile_expr(fb, &call.args[0])?;
                    let f = self.to_i32(fb, &sv0);
                    let rec = if call.args.len() >= 2 {
                        let sv1 = self.compile_expr(fb, &call.args[1])?;
                        self.to_i64(fb, &sv1)
                    } else {
                        fb.const_i64(0)
                    };
                    let len = if call.args.len() >= 3 {
                        let sv2 = self.compile_expr(fb, &call.args[2])?;
                        self.to_i64(fb, &sv2)
                    } else {
                        fb.const_i64(0)
                    };
                    if name == "LOCK" {
                        fb.call_void("pb_lock", &[f, rec, len]);
                    } else {
                        fb.call_void("pb_unlock", &[f, rec, len]);
                    }
                }
                return Ok(());
            }
            _ => {}
        }

        // GRAPHIC_* and MENU_* statements are fully handled by the match above.
        // Without this guard, arms that emit IR without a bare `return Ok(())`
        // fall through to the unimplemented report below (false positives,
        // batches 51-63).
        // Parser sometimes emits family names with spaces ("FONT NEW",
        // "GRAPHIC SET FIXED"); normalize before the handled-family guard.
        let fam = name.replace(' ', "_");
        if fam.starts_with("GRAPHIC_")
            || fam.starts_with("MENU_")
            || fam.starts_with("XPRINT_")
            || fam.starts_with("DISPLAY_")
            || fam.starts_with("FONT_")
            || fam.starts_with("IMAGELIST_")
            || fam.starts_with("ARRAY_")
            || fam.starts_with("TCP_")
            || fam.starts_with("UDP_")
            || fam == "ACCEL_ATTACH"
            || fam == "LET_PTR"
            || fam == "DEF_FN"
        {
            return Ok(());
        }

        // `o.M(args)` written as a statement, result discarded (batch 217). The
        // parser keeps the receiver as argument 0 and marks the name with a
        // leading dot, because only this layer knows the receiver's TYPE. Lower
        // it to the qualified procedure `<Class>_<Method>(o, args)`; every
        // unresolvable case stops the build instead of doing nothing.
        if let Some(method) = name.strip_prefix('.') {
            if let Some(receiver) = call.args.first() {
                match self.struct_type_of_expr(receiver) {
                    Some(class) => {
                        let qualified = format!("{}_{}", class, method);
                        let info = self
                            .functions
                            .get(&qualified)
                            .cloned()
                            .or_else(|| self.subs.get(&qualified).cloned());
                        if let Some(info) = info {
                            let margs = self.compile_call_args(fb, &call.args, &info)?;
                            if info.ret_type == IrType::Void {
                                if info.is_stdcall {
                                    fb.call_void_stdcall(&info.ir_name, &margs);
                                } else {
                                    fb.call_void(&info.ir_name, &margs);
                                }
                            } else if info.is_stdcall {
                                fb.call_stdcall(&info.ret_type, &info.ir_name, &margs);
                            } else {
                                fb.call(&info.ret_type, &info.ir_name, &margs);
                            }
                            return Ok(());
                        }
                        return Err(pb::error::PbError::parser(
                            format!(
                                "Unknown method `{}.{}` - no procedure `{}` was compiled",
                                class, method, qualified
                            ),
                            None,
                            call.line,
                        ));
                    }
                    None => {
                        return Err(pb::error::PbError::parser(
                            format!(
                                "Method call `.{}(...)`: receiver is not a TYPE instance",
                                method
                            ),
                            None,
                            call.line,
                        ));
                    }
                }
            }
        }

        if let Some(info) = self.functions.get(&name).cloned() {
            let args = self.compile_call_args(fb, &call.args, &info)?;
            if info.is_stdcall {
                fb.call_stdcall(&info.ret_type, &info.ir_name, &args);
            } else {
                fb.call(&info.ret_type, &info.ir_name, &args);
            }
            return Ok(());
        }
        if let Some(info) = self.subs.get(&name).cloned() {
            let args = self.compile_call_args(fb, &call.args, &info)?;
            if info.is_stdcall {
                fb.call_void_stdcall(&info.ir_name, &args);
            } else {
                fb.call_void(&info.ir_name, &args);
            }
            return Ok(());
        }

        eprintln!(
            "Error: unknown statement/subroutine `{}` on line {} - not implemented",
            call.name, call.line
        );
        Err(pb::error::PbError::parser(
            format!("Unknown statement/subroutine: `{}`", call.name),
            None,
            call.line,
        ))
    }

    fn compile_call_args(
        &mut self,
        fb: &mut FunctionBuilder,
        args: &[Expr],
        func_info: &FuncInfo,
    ) -> PbResult<Vec<Val>> {
        let mut compiled = Vec::new();
        for (i, arg) in args.iter().enumerate() {
            if i < func_info.params.len() {
                let param = &func_info.params[i];
                // BYVAL override at call site: pass value directly even if param is BYREF
                let (is_byval_override, inner_arg) = match arg {
                    Expr::ByvalOverride(inner) => (true, inner.as_ref()),
                    _ => (false, arg),
                };
                if param.is_byval || is_byval_override {
                    let val = self.compile_expr(fb, inner_arg)?;
                    // x64 ABI: every 4-byte BYVAL arg must occupy the full 8-byte
                    // register/stack slot.  A bare i32 leaves garbage in the upper
                    // 32 bits of a stack slot (WriteFile's lpOverlapped was the
                    // 0xC0000005 crash in batch 224).  inttoptr zero-extends the
                    // value to pointer width, preserving the low 32-bit value.
                    if val.ty == IrType::I32 {
                        let ptr_val = fb.inttoptr(&val);
                        compiled.push(ptr_val);
                    } else {
                        compiled.push(val);
                    }
                } else {
                    // BYREF: pass address
                    match inner_arg {
                        Expr::Variable(vname) => {
                            let norm = normalize_name(vname);
                            let ptr_name = self.ensure_variable_ptr(fb, &norm, vname)?;
                            compiled.push(Val::new(ptr_name, IrType::Ptr));
                        }
                        _ => {
                            // Expression to BYREF: allocate temp
                            let val = self.compile_expr(fb, inner_arg)?;
                            let tmp = fb.alloca(&val.ty);
                            fb.store(&val, &tmp);
                            compiled.push(tmp);
                        }
                    }
                }
            } else {
                let val = self.compile_expr(fb, arg)?;
                compiled.push(val);
            }
        }

        // Fill in default values for missing OPTIONAL parameters
        for i in args.len()..func_info.params.len() {
            let param = &func_info.params[i];
            if !param.is_optional {
                break; // Non-optional param missing → stop (caller error, handled at runtime)
            }
            let default_val = if Self::is_string_pb(&param.pb_type) {
                Val::new(self.empty_string_name.clone(), IrType::Ptr)
            } else {
                let ir_type = Self::ir_type_for(&param.pb_type);
                Val::new(ir_type.zero_literal(), ir_type)
            };
            if param.is_byval {
                compiled.push(default_val);
            } else {
                // BYREF: allocate temp with default value and pass its address
                let tmp = fb.alloca(&default_val.ty);
                fb.store(&default_val, &tmp);
                compiled.push(tmp);
            }
        }

        Ok(compiled)
    }

    // ========== IF ==========

    fn compile_if(&mut self, fb: &mut FunctionBuilder, if_stmt: &IfStmt) -> PbResult<()> {
        let cond = self.compile_expr_as_bool(fb, &if_stmt.condition)?;
        let merge = fb.next_label("if.merge");

        let has_else = !if_stmt.elseif_clauses.is_empty() || !if_stmt.else_body.is_empty();

        if has_else {
            let then_label = fb.next_label("if.then");
            let else_label = fb.next_label("if.else");

            fb.condbr(&cond, &then_label, &else_label);

            // Then
            fb.label(&then_label);
            self.compile_body(fb, &if_stmt.then_body)?;
            if !fb.is_terminated() {
                fb.br(&merge);
            }

            // Elseif chain
            let mut current_else = else_label;
            for (i, clause) in if_stmt.elseif_clauses.iter().enumerate() {
                fb.label(&current_else);
                let elif_cond = self.compile_expr_as_bool(fb, &clause.condition)?;
                let elif_then = fb.next_label(&format!("elif.then.{}", i));
                let elif_else = fb.next_label(&format!("elif.else.{}", i));
                fb.condbr(&elif_cond, &elif_then, &elif_else);

                fb.label(&elif_then);
                self.compile_body(fb, &clause.body)?;
                if !fb.is_terminated() {
                    fb.br(&merge);
                }

                current_else = elif_else;
            }

            // Else
            fb.label(&current_else);
            if !if_stmt.else_body.is_empty() {
                self.compile_body(fb, &if_stmt.else_body)?;
            }
            if !fb.is_terminated() {
                fb.br(&merge);
            }
        } else {
            let then_label = fb.next_label("if.then");
            fb.condbr(&cond, &then_label, &merge);
            fb.label(&then_label);
            self.compile_body(fb, &if_stmt.then_body)?;
            if !fb.is_terminated() {
                fb.br(&merge);
            }
        }

        fb.label(&merge);
        Ok(())
    }

    // ========== FOR ==========

    fn compile_for(&mut self, fb: &mut FunctionBuilder, for_stmt: &ForStmt) -> PbResult<()> {
        let var_name = normalize_name(&for_stmt.var);
        let var_ptr_name = self.ensure_variable_ptr(fb, &var_name, &for_stmt.var)?;

        // Compute and store start
        let start = self.compile_expr(fb, &for_stmt.start)?;
        let start_i32 = self.to_i32(fb, &start);
        let var_ptr = Val::new(var_ptr_name.clone(), IrType::Ptr);
        fb.store(&start_i32, &var_ptr);

        let end_val = self.compile_expr(fb, &for_stmt.end)?;
        let end_i32 = self.to_i32(fb, &end_val);

        let step_val = if let Some(ref step) = for_stmt.step {
            let s = self.compile_expr(fb, step)?;
            self.to_i32(fb, &s)
        } else {
            fb.const_i32(1)
        };

        // Store end and step in locals so they survive across iterations
        let end_ptr = fb.alloca(&IrType::I32);
        fb.store(&end_i32, &end_ptr);
        let step_ptr = fb.alloca(&IrType::I32);
        fb.store(&step_val, &step_ptr);

        let cond_label = fb.next_label("for.cond");
        let body_label = fb.next_label("for.body");
        let incr_label = fb.next_label("for.incr");
        let exit_label = fb.next_label("for.exit");

        fb.br(&cond_label);

        // Condition
        fb.label(&cond_label);
        let cur = fb.load(&IrType::I32, &var_ptr);
        let end = fb.load(&IrType::I32, &end_ptr);
        let step = fb.load(&IrType::I32, &step_ptr);
        let zero = fb.const_i32(0);
        let step_pos = fb.icmp("sgt", &step, &zero);
        let cond_le = fb.icmp("sle", &cur, &end);
        let cond_ge = fb.icmp("sge", &cur, &end);
        let cond = fb.select(&step_pos, &cond_le, &cond_ge);
        // select returns the type of the selected values (i1), convert to i1 for branch
        let cond_bool = if cond.ty != IrType::I1 {
            fb.icmp("ne", &cond, &fb.const_i32(0))
        } else {
            cond
        };
        fb.condbr(&cond_bool, &body_label, &exit_label);

        // Body
        fb.label(&body_label);
        self.loop_stack.push(LoopInfo {
            continue_label: incr_label.clone(),
            exit_label: exit_label.clone(),
            is_for: true,
        });
        self.compile_body(fb, &for_stmt.body)?;
        self.loop_stack.pop();
        if !fb.is_terminated() {
            fb.br(&incr_label);
        }

        // Increment
        fb.label(&incr_label);
        let cur2 = fb.load(&IrType::I32, &var_ptr);
        let step2 = fb.load(&IrType::I32, &step_ptr);
        let next = fb.add(&cur2, &step2);
        fb.store(&next, &var_ptr);
        fb.br(&cond_label);

        fb.label(&exit_label);
        Ok(())
    }

    // ========== DO/LOOP ==========

    fn compile_do_loop(&mut self, fb: &mut FunctionBuilder, do_stmt: &DoLoopStmt) -> PbResult<()> {
        let cond_label = fb.next_label("do.cond");
        let body_label = fb.next_label("do.body");
        let exit_label = fb.next_label("do.exit");

        if let (true, Some(condition)) = (do_stmt.is_pre_test, do_stmt.condition.as_ref()) {
            fb.br(&cond_label);
            fb.label(&cond_label);
            let cond = self.compile_expr_as_bool(fb, condition)?;
            let cond = if !do_stmt.is_while {
                // UNTIL: negate
                let one = fb.const_i1(true);
                fb.xor(&cond, &one)
            } else {
                cond
            };
            fb.condbr(&cond, &body_label, &exit_label);
        } else {
            fb.br(&body_label);
        }

        // Body
        fb.label(&body_label);
        self.loop_stack.push(LoopInfo {
            continue_label: cond_label.clone(),
            exit_label: exit_label.clone(),
            is_for: false,
        });
        self.compile_body(fb, &do_stmt.body)?;
        self.loop_stack.pop();

        if !fb.is_terminated() {
            if do_stmt.is_pre_test {
                fb.br(&cond_label);
            } else if let Some(ref cond_expr) = do_stmt.condition {
                fb.br(&cond_label);
                fb.label(&cond_label);
                let cond = self.compile_expr_as_bool(fb, cond_expr)?;
                let cond = if !do_stmt.is_while {
                    let one = fb.const_i1(true);
                    fb.xor(&cond, &one)
                } else {
                    cond
                };
                fb.condbr(&cond, &body_label, &exit_label);
            } else {
                fb.br(&body_label); // Infinite loop
            }
        }

        fb.label(&exit_label);
        Ok(())
    }

    // ========== WHILE/WEND ==========

    fn compile_while_wend(
        &mut self,
        fb: &mut FunctionBuilder,
        while_stmt: &WhileWendStmt,
    ) -> PbResult<()> {
        let cond_label = fb.next_label("while.cond");
        let body_label = fb.next_label("while.body");
        let exit_label = fb.next_label("while.exit");

        fb.br(&cond_label);

        fb.label(&cond_label);
        let cond = self.compile_expr_as_bool(fb, &while_stmt.condition)?;
        fb.condbr(&cond, &body_label, &exit_label);

        fb.label(&body_label);
        self.loop_stack.push(LoopInfo {
            continue_label: cond_label.clone(),
            exit_label: exit_label.clone(),
            is_for: false,
        });
        self.compile_body(fb, &while_stmt.body)?;
        self.loop_stack.pop();
        if !fb.is_terminated() {
            fb.br(&cond_label);
        }

        fb.label(&exit_label);
        Ok(())
    }

    // ========== SELECT CASE ==========

    fn compile_select_case(
        &mut self,
        fb: &mut FunctionBuilder,
        select: &SelectCaseStmt,
    ) -> PbResult<()> {
        let test_val = self.compile_expr(fb, &select.expr)?;
        let merge_label = fb.next_label("select.merge");

        let else_label = if !select.else_body.is_empty() {
            fb.next_label("select.else")
        } else {
            merge_label.clone()
        };

        // Generate test+body labels for each case
        let case_labels: Vec<(String, String)> = (0..select.cases.len())
            .map(|i| {
                (
                    fb.next_label(&format!("case.test.{}", i)),
                    fb.next_label(&format!("case.body.{}", i)),
                )
            })
            .collect();

        // Branch to first case
        if !case_labels.is_empty() {
            fb.br(&case_labels[0].0);
        } else {
            fb.br(&else_label);
        }

        for (i, case) in select.cases.iter().enumerate() {
            let (ref test_l, ref body_l) = case_labels[i];
            let fallthrough = if i + 1 < case_labels.len() {
                case_labels[i + 1].0.clone()
            } else {
                else_label.clone()
            };

            // Test
            fb.label(test_l);
            let matched = self.compile_case_patterns(fb, &case.patterns, &test_val)?;
            fb.condbr(&matched, body_l, &fallthrough);

            // Body
            fb.label(body_l);

            // Debug: log which CASE branch matched
            if self.debug_mode {
                let case_desc = Self::describe_case_patterns(&case.patterns);
                let (cat, _) = self.module.add_string_constant("SELECT");
                let (msg, _) = self.module.add_string_constant(&case_desc);
                fb.call_void(
                    "pb_debug_log_msg",
                    &[Val::new(cat, IrType::Ptr), Val::new(msg, IrType::Ptr)],
                );
            }

            self.compile_body(fb, &case.body)?;
            if !fb.is_terminated() {
                fb.br(&merge_label);
            }
        }

        // Else
        if !select.else_body.is_empty() {
            fb.label(&else_label);
            self.compile_body(fb, &select.else_body)?;
            if !fb.is_terminated() {
                fb.br(&merge_label);
            }
        }

        fb.label(&merge_label);
        Ok(())
    }

    /// Build a human-readable description of CASE patterns for debug logging.
    /// e.g. "CASE 20" or "CASE 60, 70" or "CASE 1 TO 10" or "CASE IS > 5"
    fn describe_case_patterns(patterns: &[CasePattern]) -> String {
        let parts: Vec<String> = patterns
            .iter()
            .map(|p| match p {
                CasePattern::Value(Expr::IntegerLit(n)) => format!("{}", n),
                CasePattern::Value(Expr::FloatLit(f)) => format!("{}", f),
                CasePattern::Value(Expr::StringLit(s)) => format!("\"{}\"", s),
                CasePattern::Value(_) => "<expr>".to_string(),
                CasePattern::Range(Expr::IntegerLit(lo), Expr::IntegerLit(hi)) => {
                    format!("{} TO {}", lo, hi)
                }
                CasePattern::Range(_, _) => "<expr> TO <expr>".to_string(),
                CasePattern::Is(op, Expr::IntegerLit(n)) => {
                    let op_str = match op {
                        CaseOp::Lt => "<",
                        CaseOp::Lte => "<=",
                        CaseOp::Gt => ">",
                        CaseOp::Gte => ">=",
                        CaseOp::Eq => "=",
                        CaseOp::Neq => "<>",
                    };
                    format!("IS {} {}", op_str, n)
                }
                CasePattern::Is(_, _) => "IS <op> <expr>".to_string(),
            })
            .collect();
        format!("CASE {}", parts.join(", "))
    }

    fn compile_case_patterns(
        &mut self,
        fb: &mut FunctionBuilder,
        patterns: &[CasePattern],
        test_val: &Val,
    ) -> PbResult<Val> {
        let mut result = fb.const_i1(false);

        for pattern in patterns {
            let matched = match pattern {
                CasePattern::Value(expr) => {
                    let val = self.compile_expr(fb, expr)?;
                    self.compare_eq(fb, test_val, &val)
                }
                CasePattern::Range(lo_expr, hi_expr) => {
                    let lo = self.compile_expr(fb, lo_expr)?;
                    let hi = self.compile_expr(fb, hi_expr)?;
                    let ge = self.compare_ge(fb, test_val, &lo);
                    let le = self.compare_le(fb, test_val, &hi);
                    fb.and(&ge, &le)
                }
                CasePattern::Is(op, expr) => {
                    let val = self.compile_expr(fb, expr)?;
                    match op {
                        CaseOp::Lt => self.compare_lt(fb, test_val, &val),
                        CaseOp::Gt => self.compare_gt(fb, test_val, &val),
                        CaseOp::Lte => self.compare_le(fb, test_val, &val),
                        CaseOp::Gte => self.compare_ge(fb, test_val, &val),
                        CaseOp::Eq => self.compare_eq(fb, test_val, &val),
                        CaseOp::Neq => self.compare_ne(fb, test_val, &val),
                    }
                }
            };
            result = fb.or(&result, &matched);
        }

        Ok(result)
    }

    // ========== PRINT ==========

    fn compile_print(&mut self, fb: &mut FunctionBuilder, print_stmt: &PrintStmt) -> PbResult<()> {
        for arg in &print_stmt.args {
            let val = self.compile_expr(fb, arg)?;
            if val.ty == IrType::Ptr {
                let (str_name, _len) = self.module.add_string_constant("%s");
                let str_ptr = Val::new(str_name, IrType::Ptr);
                fb.call_variadic(&IrType::I32, "printf", &[str_ptr, val]);
            } else if val.ty.is_int() {
                // PB PRINT semantics: numeric fields carry a leading space for
                // non-negative values (PRINT 42 prints " 42").  % lld / % d.
                if val.ty == IrType::I64 {
                    let (str_name, _len) = self.module.add_string_constant("% lld");
                    let str_ptr = Val::new(str_name, IrType::Ptr);
                    fb.call_variadic(&IrType::I32, "printf", &[str_ptr, val]);
                } else {
                    let (str_name, _len) = self.module.add_string_constant("% d");
                    let str_ptr = Val::new(str_name, IrType::Ptr);
                    // Extend to i32 if needed
                    let val_i32 = self.to_i32(fb, &val);
                    fb.call_variadic(&IrType::I32, "printf", &[str_ptr, val_i32]);
                }
            } else if val.ty.is_float() {
                let (str_name, _len) = self.module.add_string_constant("% .6g");
                let str_ptr = Val::new(str_name, IrType::Ptr);
                // Promote to f64 if needed
                let val_f64 = self.to_f64(fb, &val);
                fb.call_variadic(&IrType::I32, "printf", &[str_ptr, val_f64]);
            }
        }
        // A trailing `;` (or `,`) suppresses the newline, so the next PRINT
        // continues on the same line - see PrintStmt::trailing.
        if print_stmt.trailing.is_none() {
            let (nl_name, _) = self.module.add_string_constant("\n");
            let nl_ptr = Val::new(nl_name, IrType::Ptr);
            fb.call_variadic(&IrType::I32, "printf", &[nl_ptr]);
        }
        // Flush so console output is visible immediately (and survives
        // abnormal termination / redirection, not just process exit)
        fb.call_void("fflush", &[Val::new("null", IrType::Ptr)]);
        Ok(())
    }

    // ========== File I/O ==========

    fn compile_open(&mut self, fb: &mut FunctionBuilder, open: &OpenStmt) -> PbResult<()> {
        let filename = self.compile_expr(fb, &open.filename)?;
        // RANDOM mode: OPEN "f" FOR RANDOM AS #n LEN=reclen → pb_open_random
        if matches!(open.mode, OpenMode::Random) {
            let filenum = self.compile_expr(fb, &open.file_num)?;
            let filenum_i32 = self.to_i32(fb, &filenum);
            let reclen = match &open.reclen {
                Some(e) => {
                    let r = self.compile_expr(fb, e)?;
                    self.to_i32(fb, &r)
                }
                None => fb.const_i32(128),
            };
            fb.call_void("pb_open_random", &[filename, filenum_i32, reclen]);
            return Ok(());
        }
        let mode = match open.mode {
            OpenMode::Input => fb.const_i32(0),
            OpenMode::Output => fb.const_i32(1),
            OpenMode::Append => fb.const_i32(2),
            OpenMode::Binary => fb.const_i32(3),
            OpenMode::Random => unreachable!(),
        };
        let filenum = self.compile_expr(fb, &open.file_num)?;
        let filenum_i32 = self.to_i32(fb, &filenum);
        fb.call(&IrType::I32, "pb_open", &[filename, mode, filenum_i32]);
        Ok(())
    }

    fn compile_close(&mut self, fb: &mut FunctionBuilder, close: &CloseStmt) -> PbResult<()> {
        let filenum = self.compile_expr(fb, &close.file_num)?;
        let filenum_i32 = self.to_i32(fb, &filenum);
        fb.call_void("pb_close", &[filenum_i32]);
        Ok(())
    }

    /// MAT a() = RHS — matrix algebra (batch 32).
    fn compile_mat(&mut self, fb: &mut FunctionBuilder, m: &MatStmt) -> PbResult<()> {
        let dst = match self.symbols.lookup_array(&m.dst) {
            Some(info) => info.clone(),
            None => {
                self.warnings
                    .push(format!("MAT: array '{}' not declared", m.dst));
                return Ok(());
            }
        };
        let dst_base = Val::new(dst.ptr_name.clone(), IrType::Ptr);
        let es = Self::array_elem_size(&dst.elem_ir_type);
        let es_i32 = fb.const_i32(es);
        let is_float = fb.const_i32(
            if matches!(dst.elem_ir_type, IrType::Float | IrType::Double) {
                1
            } else {
                0
            },
        );

        let dims = dst.dims.len();
        let (rows_i32, cols_i32): (Option<Val>, Option<Val>) = if dims >= 2 {
            (
                Some(fb.const_i32(dst.dims[0].1 as i32)),
                Some(fb.const_i32(dst.dims[1].1 as i32)),
            )
        } else {
            (None, None)
        };
        let total = dst.total_elements as i64;
        let total_i64 = fb.const_i64(total);

        let src1_base = match &m.src1 {
            Some(n) => match self.symbols.lookup_array(&normalize_name(n)) {
                Some(info) => Some(Val::new(info.ptr_name.clone(), IrType::Ptr)),
                None => {
                    self.warnings
                        .push(format!("MAT: source array '{}' not declared", n));
                    return Ok(());
                }
            },
            None => None,
        };
        let src2_base = match &m.src2 {
            Some(n) => match self.symbols.lookup_array(&normalize_name(n)) {
                Some(info) => Some(Val::new(info.ptr_name.clone(), IrType::Ptr)),
                None => {
                    self.warnings
                        .push(format!("MAT: source array '{}' not declared", n));
                    return Ok(());
                }
            },
            None => None,
        };

        match m.op {
            MatOp::Con => {
                fb.call_void(
                    "pb_mat_fill",
                    &[
                        dst_base.clone(),
                        es_i32.clone(),
                        is_float.clone(),
                        total_i64.clone(),
                        fb.const_f64(1.0),
                    ],
                );
            }
            MatOp::Zer => {
                fb.call_void(
                    "pb_mat_fill",
                    &[
                        dst_base.clone(),
                        es_i32.clone(),
                        is_float.clone(),
                        total_i64.clone(),
                        fb.const_f64(0.0),
                    ],
                );
            }
            MatOp::ConScalar => {
                let v = self.compile_expr(fb, m.scalar.as_ref().unwrap())?;
                let vf = self.to_f64(fb, &v);
                fb.call_void(
                    "pb_mat_fill",
                    &[
                        dst_base.clone(),
                        es_i32.clone(),
                        is_float.clone(),
                        total_i64.clone(),
                        vf,
                    ],
                );
            }
            MatOp::Assign => {
                fb.call_void(
                    "pb_mat_copy",
                    &[
                        dst_base.clone(),
                        src1_base.clone().unwrap(),
                        es_i32.clone(),
                        total_i64.clone(),
                    ],
                );
            }
            MatOp::Add | MatOp::Sub => {
                let sub = if m.op == MatOp::Sub {
                    fb.const_i32(1)
                } else {
                    fb.const_i32(0)
                };
                fb.call_void(
                    "pb_mat_add",
                    &[
                        dst_base.clone(),
                        src1_base.clone().unwrap(),
                        src2_base.clone().unwrap(),
                        es_i32.clone(),
                        is_float.clone(),
                        total_i64.clone(),
                        sub,
                    ],
                );
            }
            MatOp::Scale => {
                let v = self.compile_expr(fb, m.scalar.as_ref().unwrap())?;
                let vf = self.to_f64(fb, &v);
                fb.call_void(
                    "pb_mat_scale",
                    &[
                        dst_base.clone(),
                        es_i32.clone(),
                        is_float.clone(),
                        total_i64.clone(),
                        vf,
                        src1_base.clone().unwrap(),
                    ],
                );
            }
            MatOp::Idn => {
                if dims >= 2 {
                    fb.call_void(
                        "pb_mat_identity",
                        &[
                            dst_base.clone(),
                            es_i32.clone(),
                            is_float.clone(),
                            rows_i32.clone().unwrap(),
                            cols_i32.clone().unwrap(),
                        ],
                    );
                } else {
                    self.warnings
                        .push(format!("MAT IDN requires a 2-D array ({})", m.dst));
                }
            }
            MatOp::Trn => {
                if dims >= 2 {
                    // rows/cols must be the SOURCE array's dimensions (dst dims are swapped)
                    let src_info = self
                        .symbols
                        .lookup_array(&normalize_name(m.src1.as_ref().unwrap()))
                        .unwrap()
                        .clone();
                    let src_rows = fb.const_i32(src_info.dims[0].1 as i32);
                    let src_cols = fb.const_i32(src_info.dims[1].1 as i32);
                    fb.call_void(
                        "pb_mat_trn",
                        &[
                            dst_base.clone(),
                            src1_base.clone().unwrap(),
                            es_i32.clone(),
                            is_float.clone(),
                            src_rows,
                            src_cols,
                        ],
                    );
                } else {
                    self.warnings
                        .push(format!("MAT TRN requires 2-D arrays ({})", m.dst));
                }
            }
            MatOp::Mul => {
                if dims >= 2 {
                    let l = rows_i32.clone().unwrap();
                    let n = cols_i32.clone().unwrap();
                    let src1_info = self
                        .symbols
                        .lookup_array(&normalize_name(m.src1.as_ref().unwrap()))
                        .unwrap()
                        .clone();
                    let m2 = if src1_info.dims.len() >= 2 {
                        fb.const_i32(src1_info.dims[1].1 as i32)
                    } else {
                        fb.const_i32(0)
                    };
                    fb.call_void(
                        "pb_mat_mul",
                        &[
                            dst_base.clone(),
                            src1_base.clone().unwrap(),
                            src2_base.clone().unwrap(),
                            es_i32.clone(),
                            is_float.clone(),
                            l,
                            m2,
                            n,
                        ],
                    );
                } else {
                    self.warnings
                        .push(format!("MAT * requires 2-D arrays ({})", m.dst));
                }
            }
            MatOp::Inv => {
                if dims >= 2 {
                    let n = rows_i32.clone().unwrap();
                    fb.call(
                        &IrType::I32,
                        "pb_mat_inv",
                        &[
                            dst_base.clone(),
                            src1_base.clone().unwrap(),
                            es_i32.clone(),
                            is_float.clone(),
                            n,
                        ],
                    );
                } else {
                    self.warnings
                        .push(format!("MAT INV requires a 2-D square array ({})", m.dst));
                }
            }
        }
        Ok(())
    }

    /// Element size in bytes for a MAT array element type.
    fn array_elem_size(ir: &IrType) -> i32 {
        match ir {
            IrType::I8 | IrType::I1 => 1,
            IrType::I16 => 2,
            IrType::I32 | IrType::Float => 4,
            IrType::I64 | IrType::Double | IrType::Ptr => 8,
            _ => 4,
        }
    }

    /// FIELD #n, size AS var[, ...] / FIELD dyn$, size AS var[, ...]
    /// FIELD RESET var[, ...] / FIELD STRING var[, ...]
    fn compile_field(&mut self, fb: &mut FunctionBuilder, f: &FieldStmt) -> PbResult<()> {
        match f.kind {
            FieldKind::File => {
                let filenum = self.compile_expr(fb, f.filenum.as_ref().unwrap())?;
                let filenum_i32 = self.to_i32(fb, &filenum);
                let mut offset: i64 = 0;
                for spec in &f.specs {
                    if let Some(info) = self.symbols.lookup(&normalize_name(&spec.name)) {
                        if spec.offset >= 0 {
                            offset = spec.offset;
                        }
                        let fv_ptr = Val::new(info.ptr_name.clone(), IrType::Ptr);
                        let off = fb.const_i64(offset);
                        let len = fb.const_i32(spec.size as i32);
                        fb.call_void(
                            "pb_field_bind_file",
                            &[filenum_i32.clone(), off, len, fv_ptr],
                        );
                    }
                    offset += spec.size;
                }
            }
            FieldKind::Str => {
                // Bind BY REFERENCE to the string variable's payload slot so
                // reassigning the string follows automatically. Mark the
                // variable: its later assignments must copy to a fresh
                // mutable buffer instead of pointing at a read-only constant.
                if let Expr::Variable(nm) = f.dyn_expr.as_ref().unwrap() {
                    self.field_bound_strings.insert(normalize_name(nm));
                }
                let (slot, _) = self.compile_lvalue_ptr(fb, f.dyn_expr.as_ref().unwrap())?;
                let mut offset: i64 = 0;
                for spec in &f.specs {
                    if let Some(info) = self.symbols.lookup(&normalize_name(&spec.name)) {
                        if spec.offset >= 0 {
                            offset = spec.offset;
                        }
                        let fv_ptr = Val::new(info.ptr_name.clone(), IrType::Ptr);
                        let off = fb.const_i64(offset);
                        let len = fb.const_i32(spec.size as i32);
                        fb.call_void("pb_field_bind_str", &[slot.clone(), off, len, fv_ptr]);
                    }
                    offset += spec.size;
                }
            }
            FieldKind::Reset | FieldKind::ToStr => {
                let is_reset = f.kind == FieldKind::Reset;
                for spec in &f.specs {
                    if let Some(info) = self.symbols.lookup(&normalize_name(&spec.name)) {
                        let fv_ptr = Val::new(info.ptr_name.clone(), IrType::Ptr);
                        fb.call_void(
                            if is_reset {
                                "pb_field_reset"
                            } else {
                                "pb_field_tostr"
                            },
                            &[fv_ptr],
                        );
                    }
                }
            }
        }
        Ok(())
    }

    fn compile_print_file(&mut self, fb: &mut FunctionBuilder, pf: &PrintFileStmt) -> PbResult<()> {
        let filenum = self.compile_expr(fb, &pf.file_num)?;
        let filenum_i32 = self.to_i32(fb, &filenum);
        for arg in &pf.args {
            let val = self.compile_expr(fb, arg)?;
            let s = self.val_to_string(fb, &val);
            fb.call_void("pb_print_file", &[filenum_i32.clone(), s]);
        }
        // Print newline after args
        fb.call_void("pb_print_file_newline", &[filenum_i32]);
        Ok(())
    }

    fn compile_input_file(
        &mut self,
        fb: &mut FunctionBuilder,
        inp: &InputFileStmt,
    ) -> PbResult<()> {
        let filenum = self.compile_expr(fb, &inp.file_num)?;
        let filenum_i32 = self.to_i32(fb, &filenum);
        for var_expr in &inp.vars {
            let (ptr, pb_type) = self.compile_lvalue_ptr(fb, var_expr)?;
            match pb_type {
                PbType::String | PbType::FixedString(_) => {
                    fb.call_void("pb_input_file_str", &[filenum_i32.clone(), ptr]);
                }
                PbType::Double | PbType::Single | PbType::Ext | PbType::Cur => {
                    fb.call_void("pb_input_file_dbl", &[filenum_i32.clone(), ptr]);
                }
                _ => {
                    // Integer types (Long, Integer, Dword, etc.)
                    fb.call_void("pb_input_file_int", &[filenum_i32.clone(), ptr]);
                }
            }
        }
        Ok(())
    }

    fn compile_line_input_file(
        &mut self,
        fb: &mut FunctionBuilder,
        li: &LineInputFileStmt,
    ) -> PbResult<()> {
        let filenum = self.compile_expr(fb, &li.file_num)?;
        let filenum_i32 = self.to_i32(fb, &filenum);
        let result = fb.call(&IrType::Ptr, "pb_line_input", &[filenum_i32]);
        let (ptr, _pb_type) = self.compile_lvalue_ptr(fb, &li.var)?;
        fb.store(&result, &ptr);
        Ok(())
    }

    fn compile_input_console(
        &mut self,
        fb: &mut FunctionBuilder,
        inp: &InputConsoleStmt,
    ) -> PbResult<()> {
        // prompt (optional)
        let prompt_val = if let Some(ref p) = inp.prompt {
            self.compile_expr(fb, p)?
        } else {
            fb.const_null_ptr()
        };
        let has_prompt = fb.const_i32(if inp.prompt.is_some() { 1 } else { 0 });
        let no_newline = fb.const_i32(if inp.no_newline { 1 } else { 0 });
        let result = fb.call(
            &IrType::Ptr,
            "pb_input_console",
            &[prompt_val, has_prompt, no_newline],
        );
        // assign to first variable (string only for now)
        if let Some(var_expr) = inp.vars.first() {
            let (ptr, _pb_type) = self.compile_lvalue_ptr(fb, var_expr)?;
            fb.store(&result, &ptr);
        }
        Ok(())
    }

    fn compile_line_input_console(
        &mut self,
        fb: &mut FunctionBuilder,
        li: &LineInputConsoleStmt,
    ) -> PbResult<()> {
        let prompt_val = if let Some(ref p) = li.prompt {
            self.compile_expr(fb, p)?
        } else {
            fb.const_null_ptr()
        };
        let has_prompt = fb.const_i32(if li.prompt.is_some() { 1 } else { 0 });
        let result = fb.call(
            &IrType::Ptr,
            "pb_line_input_console",
            &[prompt_val, has_prompt],
        );
        let (ptr, _pb_type) = self.compile_lvalue_ptr(fb, &li.var)?;
        fb.store(&result, &ptr);
        Ok(())
    }

    // ========== INCR / DECR ==========

    fn compile_incr_decr(
        &mut self,
        fb: &mut FunctionBuilder,
        stmt: &IncrDecrStmt,
        is_incr: bool,
    ) -> PbResult<()> {
        match &stmt.target {
            Expr::Variable(ref orig_name) => {
                let name = normalize_name(orig_name);
                let ptr_name = self.ensure_variable_ptr(fb, &name, orig_name)?;
                let ptr = Val::new(ptr_name, IrType::Ptr);
                let current = fb.load(&IrType::I32, &ptr);
                let amount = if let Some(ref amt) = stmt.amount {
                    let v = self.compile_expr(fb, amt)?;
                    self.to_i32(fb, &v)
                } else {
                    fb.const_i32(1)
                };
                let result = if is_incr {
                    fb.add(&current, &amount)
                } else {
                    fb.sub(&current, &amount)
                };
                fb.store(&result, &ptr);
            }
            Expr::ArrayAccess(ref arr_name, ref indices)
            | Expr::FunctionCall(ref arr_name, ref indices) => {
                let name = normalize_name(arr_name);
                if let Some(arr_info) = self.symbols.lookup_array(&name).cloned() {
                    let mut compiled_indices = Vec::new();
                    for idx in indices {
                        compiled_indices.push(self.compile_expr(fb, idx)?);
                    }
                    let flat_idx = self.compute_flat_index(fb, &compiled_indices, &arr_info);
                    let base = Val::new(arr_info.ptr_name.clone(), IrType::Ptr);
                    let elem_ptr = fb.gep_array(&arr_info.array_ir_type, &base, &flat_idx);
                    let current = fb.load(&arr_info.elem_ir_type, &elem_ptr);
                    let cur_i32 = self.to_i32(fb, &current);
                    let amount = if let Some(ref amt) = stmt.amount {
                        let v = self.compile_expr(fb, amt)?;
                        self.to_i32(fb, &v)
                    } else {
                        fb.const_i32(1)
                    };
                    let result = if is_incr {
                        fb.add(&cur_i32, &amount)
                    } else {
                        fb.sub(&cur_i32, &amount)
                    };
                    let converted =
                        self.convert_value(fb, &result, &arr_info.elem_ir_type, &arr_info.pb_type);
                    fb.store(&converted, &elem_ptr);
                }
            }
            _ => {}
        }
        Ok(())
    }

    // ========== GOSUB / RETURN / GOTO / LABEL codegen ==========

    /// After a call statement, if error trapping is enabled, test the runtime
    /// error flag and branch to the ON ERROR handler when an error occurred.
    fn emit_error_check(&mut self, fb: &mut FunctionBuilder) -> PbResult<()> {
        let handler = match &self.onerror_handler {
            Some(h) => h.clone(),
            None => return Ok(()),
        };
        if fb.is_terminated() {
            return Ok(());
        }
        let stmt_start = fb.current_label.clone();
        let cont = fb.next_label("err.cont");
        let id = self.onerror_checkpoints.len() as i32;

        // err != 0 && !active  ->  jump handler (record statement id, set active)
        // The stores happen only on the error path, so handler-internal calls
        // can never clobber the recorded statement id.
        let err_ptr = Val::new("@pb_err".to_string(), IrType::Ptr);
        let act_ptr = Val::new("@pb_err_active".to_string(), IrType::Ptr);
        let id_ptr = Val::new("@pb_err_stmt_id".to_string(), IrType::Ptr);
        let err = fb.load(&IrType::I32, &err_ptr);
        let errnz = fb.icmp("ne", &err, &fb.const_i32(0));
        let act = fb.load(&IrType::I32, &act_ptr);
        let actz = fb.icmp("eq", &act, &fb.const_i32(0));
        let fire = fb.and(&errnz, &actz);
        let err_setup = fb.next_label("err.setup");
        fb.condbr(&fire, &err_setup, &cont);
        fb.label(&err_setup);
        fb.store(&fb.const_i32(id), &id_ptr);
        fb.store(&fb.const_i32(1), &act_ptr);
        fb.br(&handler);
        fb.label(&cont);
        self.onerror_checkpoints.push((stmt_start, cont));
        Ok(())
    }

    /// Compile a RESUME statement: clear the "in handler" flag and jump back
    /// to the statement that errored (Retry) or the one after it (Next).
    fn compile_resume(&mut self, fb: &mut FunctionBuilder, mode: ResumeMode) -> PbResult<()> {
        if fb.is_terminated() {
            return Ok(());
        }
        // Clear the runtime error flag so the resumed statements don't re-trigger
        fb.store(
            &fb.const_i32(0),
            &Val::new("@pb_err".to_string(), IrType::Ptr),
        );
        fb.store(
            &fb.const_i32(0),
            &Val::new("@pb_err_active".to_string(), IrType::Ptr),
        );
        let id = fb.load(
            &IrType::I32,
            &Val::new("@pb_err_stmt_id".to_string(), IrType::Ptr),
        );
        let default = fb.next_label("resume.default");
        let cases: Vec<(i32, String)> = self
            .onerror_checkpoints
            .iter()
            .enumerate()
            .map(|(i, (start, next))| match mode {
                ResumeMode::Retry => (i as i32, start.clone()),
                ResumeMode::Next => (i as i32, next.clone()),
            })
            .collect();
        fb.switch(&id, &default, &cases);
        fb.label(&default);
        Ok(())
    }

    // ========== TRY / CATCH / FINALLY (batch 26) ==========

    /// Compile TRY ... CATCH ... [FINALLY ...] END TRY.
    /// Reuses the ON ERROR trap machinery: inside the TRY body every checked
    /// call emits an error check that branches to the CATCH block. CATCH and
    /// FINALLY run with trapping disabled. ERR/active are cleared on entry
    /// and on exit (documented approximation of official local-ERR semantics).
    fn compile_try(&mut self, fb: &mut FunctionBuilder, try_stmt: &TryStmt) -> PbResult<()> {
        if fb.is_terminated() {
            return Ok(());
        }
        let saved_handler = self.onerror_handler.clone();
        let saved_checkpoints = std::mem::take(&mut self.onerror_checkpoints);

        let catch_label = fb.next_label("try.catch");
        let finally_label = fb.next_label("try.finally");
        let end_label = fb.next_label("try.end");

        let err_ptr = Val::new("@pb_err".to_string(), IrType::Ptr);
        let act_ptr = Val::new("@pb_err_active".to_string(), IrType::Ptr);

        // ERR is local to the TRY structure: clear error + active flags so
        // the first failing statement inside the body can be trapped.
        fb.store(&fb.const_i32(0), &err_ptr);
        fb.store(&fb.const_i32(0), &act_ptr);

        // Route runtime errors inside the TRY body to the CATCH block.
        self.onerror_handler = Some(catch_label.clone());
        self.onerror_checkpoints = Vec::new();
        // Track END TRY block so EXIT TRY can jump out of the structure.
        self.try_exit_stack.push(end_label.clone());

        // TRY body
        self.compile_body(fb, &try_stmt.body)?;
        if !fb.is_terminated() {
            fb.br(&finally_label);
        }

        // CATCH block - trapping disabled, CATCH body calls can't re-trigger.
        fb.label(&catch_label);
        self.onerror_handler = None;
        if !try_stmt.catch.is_empty() {
            self.compile_body(fb, &try_stmt.catch)?;
        }
        if !fb.is_terminated() {
            fb.br(&finally_label);
        }

        // FINALLY block - unconditional, also trapping disabled.
        fb.label(&finally_label);
        if !try_stmt.finally.is_empty() {
            self.compile_body(fb, &try_stmt.finally)?;
        }
        if !fb.is_terminated() {
            fb.br(&end_label);
        }

        // END TRY - restore previous trap state and clear error flags.
        fb.label(&end_label);
        self.try_exit_stack.pop();
        self.onerror_handler = saved_handler;
        self.onerror_checkpoints = saved_checkpoints;
        fb.store(&fb.const_i32(0), &err_ptr);
        fb.store(&fb.const_i32(0), &act_ptr);
        Ok(())
    }

    fn compile_label(&mut self, fb: &mut FunctionBuilder, name: &str) -> PbResult<()> {
        // If name matches a known SUB/FUNCTION, it was misparsed → emit a call instead
        let norm = normalize_name(name);
        if self.functions.contains_key(&norm) {
            let info = self.functions.get(&norm).unwrap().clone();
            if info.ret_type == IrType::Void {
                fb.call_void(&info.ir_name, &[]);
            } else {
                fb.call(&info.ret_type, &info.ir_name, &[]);
            }
            return Ok(());
        }

        if let Some(ref mut ctx) = self.gosub_context {
            if let Some(block_name) = ctx.label_blocks.get(name) {
                // Skip duplicate label emissions
                if ctx.emitted_labels.contains(name) {
                    return Ok(());
                }
                ctx.emitted_labels.insert(name.to_string());
                let block_name = block_name.clone();
                // Fallthrough: unconditional branch to the label block
                fb.br(&block_name);
                fb.label(&block_name);
            }
        }
        Ok(())
    }

    fn compile_gosub(&mut self, fb: &mut FunctionBuilder, label: &str) -> PbResult<()> {
        if let Some(ref mut ctx) = self.gosub_context {
            let return_id = (ctx.return_points.len() + 1) as i32;
            let return_label = fb.next_label(&format!("gosub.ret.{}", return_id));
            ctx.return_points.push(return_label.clone());

            // Store unique return ID
            let ret_addr_ptr = ctx.ret_addr_ptr.clone();
            let id_val = fb.const_i32(return_id);
            fb.store(&id_val, &ret_addr_ptr);

            // Branch to the label's block
            if let Some(target_block) = ctx.label_blocks.get(label) {
                let target_block = target_block.clone();
                fb.br(&target_block);
            }

            // Emit the return-point label (where RETURN comes back to)
            fb.label(&return_label);
        }
        Ok(())
    }

    fn compile_return(&mut self, fb: &mut FunctionBuilder) -> PbResult<()> {
        if let Some(ref ctx) = self.gosub_context {
            if !ctx.return_points.is_empty() {
                // Branch to the return dispatch block
                let dispatch_label = "gosub.dispatch".to_string();
                fb.br(&dispatch_label);
            }
        }
        Ok(())
    }

    fn compile_goto(&mut self, fb: &mut FunctionBuilder, label: &str) -> PbResult<()> {
        if let Some(ref ctx) = self.gosub_context {
            if let Some(block_name) = ctx.label_blocks.get(label) {
                let block_name = block_name.clone();
                fb.br(&block_name);
            }
        }
        Ok(())
    }

    fn compile_on_goto(
        &mut self,
        fb: &mut FunctionBuilder,
        expr: &Expr,
        labels: &[String],
    ) -> PbResult<()> {
        // ON n GOTO label1, label2, ... — n is 1-based; out-of-range falls through
        let n = self.compile_expr(fb, expr)?;
        let ni = self.to_i32(fb, &n);
        if let Some(ctx) = self.gosub_context.as_ref() {
            let after = fb.next_label("on.after");
            if !labels.is_empty() {
                for (i, label) in labels.iter().enumerate() {
                    let idx = fb.const_i32((i + 1) as i32);
                    let cond = fb.icmp("eq", &ni, &idx);
                    let then_b = fb.next_label(&format!("on.goto.{}", i));
                    if i + 1 < labels.len() {
                        let next_test = fb.next_label(&format!("on.test.{}", i + 1));
                        fb.condbr(&cond, &then_b, &next_test);
                        fb.label(&then_b);
                        match ctx.label_blocks.get(label) {
                            Some(block) => fb.br(block),
                            None => fb.br(&after),
                        }
                        fb.label(&next_test);
                    } else {
                        fb.condbr(&cond, &then_b, &after);
                        fb.label(&then_b);
                        match ctx.label_blocks.get(label) {
                            Some(block) => fb.br(block),
                            None => fb.br(&after),
                        }
                        fb.label(&after);
                    }
                }
            } else {
                fb.label(&after);
            }
        }
        Ok(())
    }

    fn compile_on_gosub(
        &mut self,
        fb: &mut FunctionBuilder,
        expr: &Expr,
        labels: &[String],
    ) -> PbResult<()> {
        // ON n GOSUB label1, label2, ... — like GOSUB but selected by n (1-based)
        let n = self.compile_expr(fb, expr)?;
        let ni = self.to_i32(fb, &n);
        if let Some(ref mut ctx) = self.gosub_context {
            let after = fb.next_label("on.after");
            if !labels.is_empty() {
                for (i, label) in labels.iter().enumerate() {
                    let idx = fb.const_i32((i + 1) as i32);
                    let cond = fb.icmp("eq", &ni, &idx);
                    let then_b = fb.next_label(&format!("on.gosub.{}", i));
                    let next_test = if i + 1 < labels.len() {
                        Some(fb.next_label(&format!("on.gtest.{}", i + 1)))
                    } else {
                        None
                    };
                    match &next_test {
                        Some(nt) => fb.condbr(&cond, &then_b, nt),
                        None => fb.condbr(&cond, &then_b, &after),
                    }
                    fb.label(&then_b);
                    // gosub action: store unique return id, branch to target
                    let return_id = (ctx.return_points.len() + 1) as i32;
                    let return_label = fb.next_label(&format!("gosub.ret.{}", return_id));
                    ctx.return_points.push(return_label.clone());
                    let id_val = fb.const_i32(return_id);
                    let ret_addr_ptr = ctx.ret_addr_ptr.clone();
                    fb.store(&id_val, &ret_addr_ptr);
                    match ctx.label_blocks.get(label) {
                        Some(block) => fb.br(block),
                        None => fb.br(&after),
                    }
                    fb.label(&return_label);
                    fb.br(&after);
                    if let Some(nt) = next_test {
                        fb.label(&nt);
                    }
                }
                fb.label(&after);
                // Clear the return address so a later fallthrough to the
                // dispatch block cannot loop back to a stale return point.
                let zero = fb.const_i32(0);
                fb.store(&zero, &ctx.ret_addr_ptr.clone());
            } else {
                fb.label(&after);
            }
        }
        Ok(())
    }

    fn compile_on_call(
        &mut self,
        fb: &mut FunctionBuilder,
        expr: &Expr,
        targets: &[OnCallTarget],
    ) -> PbResult<()> {
        // ON n CALL proc1(args), func2(args) TO var, ... — n is 1-based;
        // out-of-range falls through to the next statement.
        let n = self.compile_expr(fb, expr)?;
        let ni = self.to_i32(fb, &n);
        let after = fb.next_label("oncall.after");
        for (i, t) in targets.iter().enumerate() {
            let idx = fb.const_i32((i + 1) as i32);
            let cond = fb.icmp("eq", &ni, &idx);
            let then_b = fb.next_label(&format!("oncall.t{}", i));
            let next_test = if i + 1 < targets.len() {
                Some(fb.next_label(&format!("oncall.x{}", i + 1)))
            } else {
                None
            };
            match &next_test {
                Some(nt) => fb.condbr(&cond, &then_b, nt),
                None => fb.condbr(&cond, &then_b, &after),
            }
            fb.label(&then_b);
            let cname = normalize_name(&t.name);
            if let Some(info) = self.functions.get(&cname).cloned() {
                let args = self.compile_call_args(fb, &t.args, &info)?;
                let ret = if info.is_stdcall {
                    fb.call_stdcall(&info.ret_type, &info.ir_name, &args)
                } else {
                    fb.call(&info.ret_type, &info.ir_name, &args)
                };
                if let Some(rv) = &t.ret_var {
                    if let Some((ptr, ty, pbty)) = self.lvalue_ptr(fb, &Expr::Variable(rv.clone()))
                    {
                        let cv = self.convert_value(fb, &ret, &ty, &pbty);
                        fb.store(&cv, &ptr);
                    }
                }
            } else if let Some(info) = self.subs.get(&cname).cloned() {
                let args = self.compile_call_args(fb, &t.args, &info)?;
                if info.is_stdcall {
                    fb.call_void_stdcall(&info.ir_name, &args);
                } else {
                    fb.call_void(&info.ir_name, &args);
                }
            } else {
                self.warnings.push(format!(
                    "ON CALL target `{}` is not a declared SUB/FUNCTION - skipped",
                    t.name
                ));
            }
            fb.br(&after);
            if let Some(nt) = next_test {
                fb.label(&nt);
            }
        }
        fb.label(&after);
        Ok(())
    }

    fn compile_clipboard_set(
        &mut self,
        fb: &mut FunctionBuilder,
        text: &Expr,
        result: Option<&Expr>,
    ) -> PbResult<()> {
        let v = self.compile_expr(fb, text)?;
        let rc = fb.call(&IrType::I32, "pb_clipboard_set_text", &[v]);
        if let Some(res) = result {
            if let Some((ptr, _, _)) = self.lvalue_ptr(fb, res) {
                fb.store(&rc, &ptr);
            }
        }
        Ok(())
    }

    fn compile_clipboard_get(
        &mut self,
        fb: &mut FunctionBuilder,
        target: &Expr,
        result: Option<&Expr>,
    ) -> PbResult<()> {
        let val = fb.call(&IrType::Ptr, "pb_clipboard_get_text", &[]);
        if let Expr::Variable(orig) = target {
            let name = normalize_name(orig);
            if let Some(info) = self.symbols.lookup(&name) {
                let ptr = Val::new(info.ptr_name.clone(), IrType::Ptr);
                let converted =
                    self.convert_value(fb, &val, &info.ir_type.clone(), &info.pb_type.clone());
                fb.store(&converted, &ptr);
            }
        }
        if let Some(res) = result {
            if let Some((ptr, _, _)) = self.lvalue_ptr(fb, res) {
                let one = fb.const_i32(1);
                fb.store(&one, &ptr);
            }
        }
        Ok(())
    }

    fn emit_return_dispatch(&mut self, fb: &mut FunctionBuilder) {
        let ctx = match self.gosub_context.take() {
            Some(ctx) if !ctx.return_points.is_empty() => ctx,
            other => {
                self.gosub_context = other;
                return;
            }
        };

        // Build switch cases: return_id -> return-point label
        let cases: Vec<(i32, String)> = ctx
            .return_points
            .iter()
            .enumerate()
            .map(|(i, label)| ((i + 1) as i32, label.clone()))
            .collect();

        // Create a "bad return" label (should never be reached)
        let bad_label = fb.next_label("gosub.bad");

        // Emit the dispatch block
        fb.br("gosub.dispatch");
        fb.label("gosub.dispatch");
        let ret_addr = fb.load(&IrType::I32, &ctx.ret_addr_ptr);
        fb.switch(&ret_addr, &bad_label, &cases);

        // Bad label → unreachable but needed by LLVM
        fb.label(&bad_label);
        // Just fall through (will hit the function's normal return)

        self.gosub_context = None;
    }

    // ========== Expression compilation ==========

    fn compile_expr(&mut self, fb: &mut FunctionBuilder, expr: &Expr) -> PbResult<Val> {
        match expr {
            Expr::IntegerLit(n) => {
                if *n >= i32::MIN as i64 && *n <= i32::MAX as i64 {
                    Ok(fb.const_i32(*n as i32))
                } else {
                    Ok(fb.const_i64(*n))
                }
            }
            Expr::FloatLit(f) => Ok(fb.const_f64(*f)),
            Expr::StringLit(s) => {
                let (str_name, _) = self.module.add_string_constant(s);
                Ok(Val::new(str_name, IrType::Ptr))
            }
            Expr::Variable(orig_name) => {
                // Built-in string equates ($CRLF, $TAB, ...) — check before normalize
                // (normalize would strip the leading/trailing $ and hide the equate)
                if orig_name.starts_with('$') && orig_name.len() > 1 {
                    if let Some(s) = string_equate(orig_name) {
                        let (str_name, _) = self.module.add_string_constant(&s);
                        return Ok(Val::new(str_name, IrType::Ptr));
                    }
                    if let Some(w) = wide_equate(orig_name) {
                        return Ok(fb.const_i32(w));
                    }
                    if wide_equate_unsupported(orig_name) {
                        self.warnings.push(format!(
                            "wide multi-char equate `{}` is not supported yet \
                             (no wide-string type in this compiler) - treated as empty string",
                            orig_name
                        ));
                        let (str_name, _) = self.module.add_string_constant("");
                        return Ok(Val::new(str_name, IrType::Ptr));
                    }
                }
                let name = normalize_name(orig_name);
                // System variables that are actually function calls
                match name.as_str() {
                    "DATE" if orig_name.ends_with('$') => {
                        return Ok(fb.call(&IrType::Ptr, "pb_date", &[]));
                    }
                    "TIME" if orig_name.ends_with('$') => {
                        return Ok(fb.call(&IrType::Ptr, "pb_time", &[]));
                    }
                    "WAITKEY" if orig_name.ends_with('$') => {
                        return Ok(fb.call(&IrType::Ptr, "pb_waitkey", &[]));
                    }
                    "CALLSTKCOUNT" => {
                        return Ok(fb.call(&IrType::I64, "pb_callstk_count", &[]));
                    }
                    "CURDIR" if orig_name.ends_with('$') => {
                        let buf = fb.alloca(&IrType::Array(1024, Box::new(IrType::I8)));
                        let len_const = fb.const_i32(1024);
                        if self.module.is_32bit() {
                            fb.call_stdcall(
                                &IrType::I32,
                                "GetCurrentDirectoryA",
                                &[len_const.clone(), buf.clone()],
                            );
                        } else {
                            fb.call(
                                &IrType::I32,
                                "GetCurrentDirectoryA",
                                &[len_const.clone(), buf.clone()],
                            );
                        }
                        let str_len = fb.call(&IrType::I32, "strlen", std::slice::from_ref(&buf));
                        return Ok(fb.call(&IrType::Ptr, "pb_bstr_alloc", &[buf, str_len]));
                    }
                    "ERR" => {
                        // PB-compatible ERR system variable: load @pb_err (i32)
                        let g = Val::new("@pb_err".to_string(), IrType::Ptr);
                        return Ok(fb.load(&IrType::I32, &g));
                    }
                    "RND" => {
                        // Bare RND (no parens) is the PB random function
                        return self.builtin_rnd(fb, &[]);
                    }
                    "TIX" => {
                        // TIX — high-resolution performance counter (QUAD)
                        return Ok(fb.call(&IrType::I64, "pb_tix", &[]));
                    }
                    _ => {}
                }
                // Implicit member access inside a METHOD (batch 216): a bare name that
                // is one of the receiver's INSTANCE fields reads this.<name>.
                if let Some(member_expr) = self.implicit_this_member(&name) {
                    return self.compile_expr(fb, &member_expr);
                }
                let ptr_name = self.ensure_variable_ptr(fb, &name, orig_name)?;
                let info = self.symbols.lookup(&name).unwrap();
                // FixedString (ASCIIZ*N): alloca IS the buffer → create heap copy
                // (stack buffer would be dangling after function return)
                if matches!(info.pb_type, PbType::FixedString(_)) {
                    let buf_ptr = Val::new(ptr_name, IrType::Ptr);
                    let len = fb.call(&IrType::I32, "strlen", std::slice::from_ref(&buf_ptr));
                    return Ok(fb.call(&IrType::Ptr, "pb_bstr_alloc", &[buf_ptr, len]));
                }
                // FIELD variable: fresh BSTR copy of the bound sub-section
                if matches!(info.pb_type, PbType::Field) {
                    let fv_ptr = Val::new(ptr_name, IrType::Ptr);
                    return Ok(fb.call(&IrType::Ptr, "pb_field_get", &[fv_ptr]));
                }
                let ptr = Val::new(ptr_name, IrType::Ptr);
                let ir_type = info.ir_type.clone();
                let val = fb.load(&ir_type, &ptr);
                // For string variables, null-guard: replace null with empty string
                if ir_type == IrType::Ptr && Self::is_string_name(orig_name) {
                    let is_null = fb.icmp("eq", &val, &Val::new("null".to_string(), IrType::Ptr));
                    let empty = Val::new(self.empty_string_name.clone(), IrType::Ptr);
                    Ok(fb.select(&is_null, &empty, &val))
                } else {
                    Ok(val)
                }
            }
            Expr::PercentConst(name) => {
                // batch 243: PBWin ships equates.inc with all Win32 equates
                // pre-defined; the fork used to silently resolve any unknown
                // %CONSTANT to 0, which corrupted styles like
                // "%BS_DEFAULT OR %WS_TABSTOP" (OK button lost its tab stop).
                // Look up the program's own #CONST/%X= first, then fall back
                // to the built-in Win32 equate table below.
                let upper = name.to_uppercase();
                let val = self.pp_constants.get(&upper).copied().or_else(|| {
                    PB_BUILTIN_EQUATES
                        .iter()
                        .find(|(k, _)| *k == upper.as_str())
                        .map(|(_, v)| *v)
                });
                let v = val.unwrap_or(0);
                Ok(fb.const_i32(v as i32))
            }
            Expr::FunctionCall(name, args) => {
                let name = normalize_name(name);
                // Check if it's actually an array access (parser can't distinguish)
                if let Some(arr_info) = self.symbols.lookup_array(&name).cloned() {
                    let mut compiled_indices = Vec::new();
                    for idx in args {
                        compiled_indices.push(self.compile_expr(fb, idx)?);
                    }
                    let flat_idx = self.compute_flat_index(fb, &compiled_indices, &arr_info);
                    let base = Val::new(arr_info.ptr_name.clone(), IrType::Ptr);
                    let elem_ptr = fb.gep_array(&arr_info.array_ir_type, &base, &flat_idx);
                    let val = fb.load(&arr_info.elem_ir_type, &elem_ptr);
                    // Null-guard string array elements (zeroinitializer → null pointers)
                    if arr_info.elem_ir_type == IrType::Ptr {
                        Ok(self.null_guard_string(fb, &val))
                    } else {
                        Ok(val)
                    }
                } else if let Some(result) = self.compile_builtin(fb, &name, args) {
                    result
                } else if let Some(info) = self.functions.get(&name).cloned() {
                    let compiled_args = self.compile_call_args(fb, args, &info)?;
                    if info.is_stdcall {
                        Ok(fb.call_stdcall(&info.ret_type, &info.ir_name, &compiled_args))
                    } else {
                        Ok(fb.call(&info.ret_type, &info.ir_name, &compiled_args))
                    }
                } else {
                    self.warnings.push(format!(
                        "expression function `{}` not implemented - replaced with 0",
                        name
                    ));
                    Ok(fb.const_i32(0)) // Unknown function
                }
            }
            Expr::ArrayAccess(arr_name, indices) => {
                let name = normalize_name(arr_name);
                if let Some(arr_info) = self.symbols.lookup_array(&name).cloned() {
                    let mut compiled_indices = Vec::new();
                    for idx in indices {
                        compiled_indices.push(self.compile_expr(fb, idx)?);
                    }
                    let flat_idx = self.compute_flat_index(fb, &compiled_indices, &arr_info);
                    let base = Val::new(arr_info.ptr_name.clone(), IrType::Ptr);
                    let elem_ptr = fb.gep_array(&arr_info.array_ir_type, &base, &flat_idx);
                    let val = fb.load(&arr_info.elem_ir_type, &elem_ptr);
                    // Null-guard string array elements (zeroinitializer → null pointers)
                    if arr_info.elem_ir_type == IrType::Ptr {
                        Ok(self.null_guard_string(fb, &val))
                    } else {
                        Ok(val)
                    }
                } else {
                    Ok(fb.const_i32(0))
                }
            }
            Expr::MethodCall { base, name, args } => {
                // `o.M(a, b)` — the dotted method-call sugar. Lower it to the
                // qualified free function the METHOD was compiled as, prepending
                // the receiver as argument 0 so the implicit BYREF `this` receives
                // the object itself. Everything else (argument modes, byref
                // addresses) is the existing call machinery's job.
                //
                // Loud on every failure path on purpose: an unresolvable receiver
                // or method must stop the build, never produce a silent value.
                let class = self.struct_type_of_expr(base).ok_or_else(|| {
                    PbError::runtime(format!(
                        "method call `.{}(...)`: receiver is not a TYPE instance",
                        name
                    ))
                })?;
                let qualified = format!("{}_{}", class, normalize_name(name));
                if !self.functions.contains_key(&qualified) {
                    return Err(PbError::runtime(format!(
                        "unknown method `{}.{}` - no procedure `{}` was compiled",
                        class,
                        normalize_name(name),
                        qualified
                    )));
                }
                let mut call_args = Vec::with_capacity(args.len() + 1);
                call_args.push((**base).clone());
                call_args.extend(args.iter().cloned());
                let lowered = Expr::FunctionCall(qualified, call_args);
                self.compile_expr(fb, &lowered)
            }
            Expr::TypeMember(base_expr, member) => {
                // Intercept EXE.PATH$ and EXE.NAME$
                if let Expr::Variable(ref base_name) = **base_expr {
                    let bn = normalize_name(base_name);
                    let mn = normalize_name(member);
                    if bn == "EXE" && mn == "PATH" {
                        return Ok(fb.call(&IrType::Ptr, "pb_exe_path", &[]));
                    }
                    if bn == "EXE" && (mn == "NAME" || mn == "FULL") {
                        return Ok(fb.call(&IrType::Ptr, "pb_exe_name", &[]));
                    }
                }
                let (field_ptr, field_pb) = self.compile_lvalue_ptr(fb, expr)?;
                // FixedString TYPE fields: the buffer address IS the string →
                // don't load from the buffer (that would read buffer content as a pointer)
                if matches!(field_pb, PbType::FixedString(_)) {
                    return Ok(field_ptr);
                }
                let field_ir = Self::ir_type_for(&field_pb);
                Ok(fb.load(&field_ir, &field_ptr))
            }
            Expr::Negate(inner) => {
                let val = self.compile_expr(fb, inner)?;
                if val.ty.is_float() {
                    Ok(fb.fneg(&val))
                } else {
                    Ok(fb.neg(&val))
                }
            }
            Expr::UnaryOp(UnaryOp::Not, inner) => {
                let val = self.compile_expr(fb, inner)?;
                Ok(fb.not(&val))
            }
            Expr::UnaryOp(UnaryOp::Neg, inner) => {
                let val = self.compile_expr(fb, inner)?;
                if val.ty.is_float() {
                    Ok(fb.fneg(&val))
                } else {
                    Ok(fb.neg(&val))
                }
            }
            Expr::Varptr(inner) => {
                // VARPTR(var) or VARPTR(arr(idx)) → return address as i32
                let (ptr, _pb_type) = self.compile_lvalue_ptr(fb, inner)?;
                Ok(fb.ptrtoint64(&ptr))
            }
            Expr::ByvalOverride(inner) => {
                // BYVAL override outside of call args context → just compile inner expr
                self.compile_expr(fb, inner)
            }
            Expr::Omitted => Err(PbError::runtime(
                "Omitted argument slot is not valid inside an expression",
            )),
            Expr::BinaryOp(op, left, right) => {
                let lhs = self.compile_expr(fb, left)?;
                let rhs = self.compile_expr(fb, right)?;
                self.compile_binary_op(fb, op, &lhs, &rhs)
            }
        }
    }

    fn compile_binary_op(
        &mut self,
        fb: &mut FunctionBuilder,
        op: &BinaryOp,
        lhs: &Val,
        rhs: &Val,
    ) -> PbResult<Val> {
        // String operations
        if lhs.ty == IrType::Ptr || rhs.ty == IrType::Ptr {
            return self.compile_string_binop(fb, op, lhs, rhs);
        }

        // Float promotion if either side is float
        if lhs.ty.is_float() || rhs.ty.is_float() {
            let l = self.to_f64(fb, lhs);
            let r = self.to_f64(fb, rhs);
            return self.compile_float_binop(fb, op, &l, &r);
        }

        // Integer: promote to same width
        let (l, r) = self.promote_ints(fb, lhs, rhs);

        match op {
            BinaryOp::Add => Ok(fb.add(&l, &r)),
            BinaryOp::Sub => Ok(fb.sub(&l, &r)),
            BinaryOp::Mul => Ok(fb.mul(&l, &r)),
            BinaryOp::Div => {
                // PB `/` produces float result
                let lf = fb.sitofp(&l, &IrType::Double);
                let rf = fb.sitofp(&r, &IrType::Double);
                Ok(fb.fdiv(&lf, &rf))
            }
            BinaryOp::IntDiv => Ok(fb.sdiv(&l, &r)),
            BinaryOp::Mod => Ok(fb.srem(&l, &r)),
            BinaryOp::Pow => {
                let lf = fb.sitofp(&l, &IrType::Double);
                let rf = fb.sitofp(&r, &IrType::Double);
                Ok(fb.call(&IrType::Double, "llvm.pow.f64", &[lf, rf]))
            }
            BinaryOp::And => Ok(fb.and(&l, &r)),
            BinaryOp::Or => Ok(fb.or(&l, &r)),
            BinaryOp::Xor => Ok(fb.xor(&l, &r)),
            BinaryOp::Eqv => {
                // a EQV b = NOT (a XOR b) — bitwise equivalence
                let x = fb.xor(&l, &r);
                let all_ones = fb.const_i64(-1);
                Ok(fb.xor(&x, &all_ones))
            }
            BinaryOp::Imp => {
                // a IMP b = (NOT a) OR b — bitwise implication
                let all_ones = fb.const_i64(-1);
                let not_l = fb.xor(&l, &all_ones);
                Ok(fb.or(&not_l, &r))
            }
            BinaryOp::Eq => Ok(fb.icmp("eq", &l, &r)),
            BinaryOp::Neq => Ok(fb.icmp("ne", &l, &r)),
            BinaryOp::Lt => Ok(fb.icmp("slt", &l, &r)),
            BinaryOp::Gt => Ok(fb.icmp("sgt", &l, &r)),
            BinaryOp::Lte => Ok(fb.icmp("sle", &l, &r)),
            BinaryOp::Gte => Ok(fb.icmp("sge", &l, &r)),
            BinaryOp::StrConcat => {
                // & on non-string operands: convert both to strings and concat
                let ls = self.num_to_string(fb, &l);
                let rs = self.num_to_string(fb, &r);
                Ok(self.compile_str_concat(fb, &ls, &rs))
            }
        }
    }

    fn compile_float_binop(
        &mut self,
        fb: &mut FunctionBuilder,
        op: &BinaryOp,
        l: &Val,
        r: &Val,
    ) -> PbResult<Val> {
        match op {
            BinaryOp::Add => Ok(fb.fadd(l, r)),
            BinaryOp::Sub => Ok(fb.fsub(l, r)),
            BinaryOp::Mul => Ok(fb.fmul(l, r)),
            BinaryOp::Div => Ok(fb.fdiv(l, r)),
            BinaryOp::IntDiv => {
                let div = fb.fdiv(l, r);
                Ok(fb.fptosi(&div, &IrType::I32))
            }
            BinaryOp::Mod => Ok(fb.frem(l, r)),
            BinaryOp::Pow => Ok(fb.call(&IrType::Double, "llvm.pow.f64", &[l.clone(), r.clone()])),
            BinaryOp::Eq => Ok(fb.fcmp("oeq", l, r)),
            BinaryOp::Neq => Ok(fb.fcmp("one", l, r)),
            BinaryOp::Lt => Ok(fb.fcmp("olt", l, r)),
            BinaryOp::Gt => Ok(fb.fcmp("ogt", l, r)),
            BinaryOp::Lte => Ok(fb.fcmp("ole", l, r)),
            BinaryOp::Gte => Ok(fb.fcmp("oge", l, r)),
            BinaryOp::StrConcat => {
                let ls = self.num_to_string(fb, l);
                let rs = self.num_to_string(fb, r);
                Ok(self.compile_str_concat(fb, &ls, &rs))
            }
            _ => Ok(fb.const_f64(0.0)),
        }
    }

    // ========== String binary ops ==========

    fn compile_str_payload(&mut self, fb: &mut FunctionBuilder, expr: &Expr) -> PbResult<Val> {
        // runtime string params expect a payload (C-string) pointer.
        // add_string_constant already returns a getelementptr to the payload
        // (4-byte BSTR length prefix skipped), and variables / concat results
        // are payload pointers too — so plain compile_expr is correct.
        self.compile_expr(fb, expr)
    }

    fn compile_string_binop(
        &mut self,
        fb: &mut FunctionBuilder,
        op: &BinaryOp,
        lhs: &Val,
        rhs: &Val,
    ) -> PbResult<Val> {
        // Null-guard both operands for all string operations.
        // A numeric operand in a string context must first be converted to
        // text: pb_str_concat/strcmp expect ptr, and passing an i32/f64 Val
        // as a ptr is a type mismatch (batch 226: "P3" & LEN("abc") dropped
        // the whole output; "ISFILE = " & ISFILE(...) lost the value).
        let safe_l = self.stringify_operand(fb, lhs);
        let safe_r = self.stringify_operand(fb, rhs);
        match op {
            BinaryOp::StrConcat => Ok(self.compile_str_concat(fb, &safe_l, &safe_r)),
            BinaryOp::Add => {
                // "+" on strings acts as concat in some contexts
                Ok(self.compile_str_concat(fb, &safe_l, &safe_r))
            }
            BinaryOp::Eq => {
                let cmp = fb.call(&IrType::I32, "strcmp", &[safe_l, safe_r]);
                let zero = fb.const_i32(0);
                Ok(fb.icmp("eq", &cmp, &zero))
            }
            BinaryOp::Neq => {
                let cmp = fb.call(&IrType::I32, "strcmp", &[safe_l, safe_r]);
                let zero = fb.const_i32(0);
                Ok(fb.icmp("ne", &cmp, &zero))
            }
            BinaryOp::Lt => {
                let cmp = fb.call(&IrType::I32, "strcmp", &[safe_l, safe_r]);
                let zero = fb.const_i32(0);
                Ok(fb.icmp("slt", &cmp, &zero))
            }
            BinaryOp::Gt => {
                let cmp = fb.call(&IrType::I32, "strcmp", &[safe_l, safe_r]);
                let zero = fb.const_i32(0);
                Ok(fb.icmp("sgt", &cmp, &zero))
            }
            BinaryOp::Lte => {
                let cmp = fb.call(&IrType::I32, "strcmp", &[safe_l, safe_r]);
                let zero = fb.const_i32(0);
                Ok(fb.icmp("sle", &cmp, &zero))
            }
            BinaryOp::Gte => {
                let cmp = fb.call(&IrType::I32, "strcmp", &[safe_l, safe_r]);
                let zero = fb.const_i32(0);
                Ok(fb.icmp("sge", &cmp, &zero))
            }
            _ => Ok(fb.const_i32(0)), // Other ops not meaningful on strings
        }
    }

    fn compile_str_concat(&mut self, fb: &mut FunctionBuilder, a: &Val, b: &Val) -> Val {
        // Use null-safe runtime concat (handles null pointers from uninitialized strings)
        fb.call(&IrType::Ptr, "pb_str_concat", &[a.clone(), b.clone()])
    }

    // ========== Boolean conversion ==========

    fn compile_expr_as_bool(&mut self, fb: &mut FunctionBuilder, expr: &Expr) -> PbResult<Val> {
        let val = self.compile_expr(fb, expr)?;
        if val.ty == IrType::I1 {
            return Ok(val);
        }
        if val.ty == IrType::Ptr {
            // String: non-empty = true (check first byte != 0)
            let safe = self.null_guard_string(fb, &val);
            let first_byte = fb.load(&IrType::I8, &safe);
            let zero = fb.const_i8(0);
            Ok(fb.icmp("ne", &first_byte, &zero))
        } else if val.ty.is_int() {
            let zero = Val::new(val.ty.zero_literal(), val.ty.clone());
            Ok(fb.icmp("ne", &val, &zero))
        } else if val.ty.is_float() {
            let zero = Val::new(val.ty.zero_literal(), val.ty.clone());
            Ok(fb.fcmp("one", &val, &zero))
        } else {
            Ok(fb.const_i1(false))
        }
    }

    // ========== Comparison helpers ==========

    fn string_compare(&self, fb: &mut FunctionBuilder, pred: &str, a: &Val, b: &Val) -> Val {
        // Null-guard both operands: null → empty string (prevents strcmp crash)
        let safe_a = self.null_guard_string(fb, a);
        let safe_b = self.null_guard_string(fb, b);
        let cmp = fb.call(&IrType::I32, "strcmp", &[safe_a, safe_b]);
        let zero = fb.const_i32(0);
        fb.icmp(pred, &cmp, &zero)
    }

    /// Replace null string pointer with empty string constant. No-op for non-pointer types.
    fn null_guard_string(&self, fb: &mut FunctionBuilder, val: &Val) -> Val {
        if val.ty != IrType::Ptr {
            return val.clone();
        }
        let is_null = fb.icmp("eq", val, &Val::new("null".to_string(), IrType::Ptr));
        let empty = Val::new(self.empty_string_name.clone(), IrType::Ptr);
        fb.select(&is_null, &empty, val)
    }

    /// Convert a Val to a string operand: pass pointer operands through the
    /// null guard, turn numeric operands into text (STR$ semantics) so mixed
    /// `"x" & 42` / `"x" & LEN(...)` concatenations and comparisons receive
    /// well-formed ptr arguments.
    fn stringify_operand(&mut self, fb: &mut FunctionBuilder, val: &Val) -> Val {
        if val.ty == IrType::Ptr {
            self.null_guard_string(fb, val)
        } else {
            self.num_to_string(fb, val)
        }
    }

    fn compare_eq(&self, fb: &mut FunctionBuilder, a: &Val, b: &Val) -> Val {
        if a.ty == IrType::Ptr || b.ty == IrType::Ptr {
            self.string_compare(fb, "eq", a, b)
        } else if a.ty.is_int() && b.ty.is_int() {
            let (l, r) = self.promote_ints(fb, a, b);
            fb.icmp("eq", &l, &r)
        } else {
            let l = self.to_f64(fb, a);
            let r = self.to_f64(fb, b);
            fb.fcmp("oeq", &l, &r)
        }
    }

    fn compare_ne(&self, fb: &mut FunctionBuilder, a: &Val, b: &Val) -> Val {
        if a.ty == IrType::Ptr || b.ty == IrType::Ptr {
            self.string_compare(fb, "ne", a, b)
        } else if a.ty.is_int() && b.ty.is_int() {
            let (l, r) = self.promote_ints(fb, a, b);
            fb.icmp("ne", &l, &r)
        } else {
            let l = self.to_f64(fb, a);
            let r = self.to_f64(fb, b);
            fb.fcmp("one", &l, &r)
        }
    }

    fn compare_lt(&self, fb: &mut FunctionBuilder, a: &Val, b: &Val) -> Val {
        if a.ty == IrType::Ptr || b.ty == IrType::Ptr {
            self.string_compare(fb, "slt", a, b)
        } else if a.ty.is_int() && b.ty.is_int() {
            let (l, r) = self.promote_ints(fb, a, b);
            fb.icmp("slt", &l, &r)
        } else {
            let l = self.to_f64(fb, a);
            let r = self.to_f64(fb, b);
            fb.fcmp("olt", &l, &r)
        }
    }

    fn compare_gt(&self, fb: &mut FunctionBuilder, a: &Val, b: &Val) -> Val {
        if a.ty == IrType::Ptr || b.ty == IrType::Ptr {
            self.string_compare(fb, "sgt", a, b)
        } else if a.ty.is_int() && b.ty.is_int() {
            let (l, r) = self.promote_ints(fb, a, b);
            fb.icmp("sgt", &l, &r)
        } else {
            let l = self.to_f64(fb, a);
            let r = self.to_f64(fb, b);
            fb.fcmp("ogt", &l, &r)
        }
    }

    fn compare_le(&self, fb: &mut FunctionBuilder, a: &Val, b: &Val) -> Val {
        if a.ty == IrType::Ptr || b.ty == IrType::Ptr {
            self.string_compare(fb, "sle", a, b)
        } else if a.ty.is_int() && b.ty.is_int() {
            let (l, r) = self.promote_ints(fb, a, b);
            fb.icmp("sle", &l, &r)
        } else {
            let l = self.to_f64(fb, a);
            let r = self.to_f64(fb, b);
            fb.fcmp("ole", &l, &r)
        }
    }

    fn compare_ge(&self, fb: &mut FunctionBuilder, a: &Val, b: &Val) -> Val {
        if a.ty == IrType::Ptr || b.ty == IrType::Ptr {
            self.string_compare(fb, "sge", a, b)
        } else if a.ty.is_int() && b.ty.is_int() {
            let (l, r) = self.promote_ints(fb, a, b);
            fb.icmp("sge", &l, &r)
        } else {
            let l = self.to_f64(fb, a);
            let r = self.to_f64(fb, b);
            fb.fcmp("oge", &l, &r)
        }
    }

    /// Wrap a malloc'd C string buffer into a BSTR and free the temp buffer.
    fn bstr_from_buf(&mut self, fb: &mut FunctionBuilder, buf: &Val) -> Val {
        let len = fb.call(&IrType::I32, "strlen", std::slice::from_ref(buf));
        let bstr = fb.call(&IrType::Ptr, "pb_bstr_alloc", &[buf.clone(), len]);
        fb.call_void("free", std::slice::from_ref(buf));
        bstr
    }

    // ========== Type conversion helpers ==========

    fn to_i32(&self, fb: &mut FunctionBuilder, val: &Val) -> Val {
        match &val.ty {
            IrType::I32 => val.clone(),
            IrType::I1 => fb.zext(val, &IrType::I32),
            // BYTE is unsigned; without this arm the catch-all below
            // returns the value unchanged and callers that truncate to
            // i8 emit an illegal `trunc i8 to i8` (batch 171).
            IrType::I8 => fb.zext(val, &IrType::I32),
            IrType::I16 => fb.sext(val, &IrType::I32),
            IrType::I64 => fb.trunc(val, &IrType::I32),
            IrType::Double | IrType::Float => fb.fptosi(val, &IrType::I32),
            _ => val.clone(),
        }
    }

    fn to_f64(&self, fb: &mut FunctionBuilder, val: &Val) -> Val {
        match &val.ty {
            IrType::Double => val.clone(),
            IrType::Float => fb.fpext(val, &IrType::Double),
            IrType::I1 | IrType::I16 | IrType::I32 | IrType::I64 => fb.sitofp(val, &IrType::Double),
            _ => val.clone(),
        }
    }

    fn val_to_string(&mut self, fb: &mut FunctionBuilder, val: &Val) -> Val {
        if val.ty == IrType::Ptr {
            val.clone()
        } else {
            self.num_to_string(fb, val)
        }
    }

    fn promote_ints(&self, fb: &mut FunctionBuilder, a: &Val, b: &Val) -> (Val, Val) {
        let aw = a.ty.bit_width();
        let bw = b.ty.bit_width();
        let ext = |fb: &mut FunctionBuilder, v: &Val, ty: &IrType| -> Val {
            // PB BYTE (I8) is unsigned — widen with zext, everything else sext
            if v.ty == IrType::I8 {
                fb.zext(v, ty)
            } else {
                fb.sext(v, ty)
            }
        };
        if aw == bw {
            (a.clone(), b.clone())
        } else if aw > bw {
            let b_ext = ext(fb, b, &a.ty);
            (a.clone(), b_ext)
        } else {
            let a_ext = ext(fb, a, &b.ty);
            (a_ext, b.clone())
        }
    }

    fn convert_value(
        &self,
        fb: &mut FunctionBuilder,
        val: &Val,
        target_ir: &IrType,
        target_pb: &PbType,
    ) -> Val {
        // Callers that store into an lvalue pass the *pointer's* IR type here
        // (IrType::Ptr) together with the variable's PB type.  For a numeric
        // destination the real width comes from the PB type; without resolving it
        // the conversion is a no-op and an i64 call result is stored into a 32-bit
        // slot - an 8-byte out-of-bounds store that LLVM then drops, so the
        // assignment silently never happens (batch 200e: IMAGELIST NEW ... TO h).
        let resolved: IrType;
        let target_ir: &IrType = {
            let from_pb = Self::ir_type_for(target_pb);
            if *target_ir == IrType::Ptr && (from_pb.is_int() || from_pb.is_float()) {
                resolved = from_pb;
                &resolved
            } else {
                target_ir
            }
        };

        if val.ty == *target_ir {
            return val.clone();
        }
        // Ptr→Ptr: no conversion needed (string to string)
        if val.ty == IrType::Ptr && *target_ir == IrType::Ptr {
            return val.clone();
        }
        if Self::is_float_pb(target_pb) {
            if val.ty.is_float() {
                if val.ty.bit_width() < target_ir.bit_width() {
                    fb.fpext(val, target_ir)
                } else {
                    fb.fptrunc(val, target_ir)
                }
            } else {
                fb.sitofp(val, target_ir)
            }
        } else if target_ir.is_int() {
            if val.ty == IrType::Ptr {
                // A window/control handle is a pointer at the IR level.  Converting
                // one to an integer destination has to go through ptrtoint; without
                // this the fallback below emitted `fptosi ptr ... to i32`, which is
                // invalid IR and broke every `CONTROL ADD ... TO hVar` store.
                let as_int = fb.ptrtoint(val);
                if as_int.ty == *target_ir {
                    return as_int;
                }
                if as_int.ty.bit_width() > target_ir.bit_width() {
                    return fb.trunc(&as_int, target_ir);
                }
                return as_int;
            }
            if val.ty.is_int() {
                if val.ty.bit_width() < target_ir.bit_width() {
                    // PB BYTE (I8) is unsigned; widen with zext, everything else sext
                    if val.ty == IrType::I8 {
                        fb.zext(val, target_ir)
                    } else {
                        fb.sext(val, target_ir)
                    }
                } else {
                    fb.trunc(val, target_ir)
                }
            } else {
                fb.fptosi(val, target_ir)
            }
        } else {
            val.clone()
        }
    }

    // ========== Builtin functions ==========

    /// Try to compile a builtin function call. Returns None if not a builtin.
    fn compile_builtin(
        &mut self,
        fb: &mut FunctionBuilder,
        name: &str,
        args: &[Expr],
    ) -> Option<PbResult<Val>> {
        match name {
            "ABS" => Some(self.builtin_abs(fb, args)),
            "SGN" => Some(self.builtin_sgn(fb, args)),
            "MIN" => Some(self.builtin_min_max(fb, args, false)),
            "MAX" => Some(self.builtin_min_max(fb, args, true)),
            "INT" => Some(self.builtin_int(fb, args)),
            "FIX" => Some(self.builtin_fix(fb, args)),
            "CEIL" => Some(self.builtin_ceil(fb, args)),
            "FLOOR" => Some(self.builtin_floor(fb, args)),
            "TRUNC" => Some(self.builtin_trunc(fb, args)),
            "SQR" => Some(self.builtin_unary_math(fb, args, "llvm.sqrt.f64")),
            "LOG" => Some(self.builtin_unary_math(fb, args, "llvm.log.f64")),
            "EXP" => Some(self.builtin_unary_math(fb, args, "llvm.exp.f64")),
            "SIN" => Some(self.builtin_unary_math(fb, args, "llvm.sin.f64")),
            "COS" => Some(self.builtin_unary_math(fb, args, "llvm.cos.f64")),
            "TAN" => Some(self.builtin_unary_math(fb, args, "tan")),
            "ATN" => Some(self.builtin_unary_math(fb, args, "atan")),
            "ASIN" => Some(self.builtin_unary_math(fb, args, "asin")),
            "ACOS" => Some(self.builtin_unary_math(fb, args, "acos")),
            "SINH" => Some(self.builtin_unary_math(fb, args, "sinh")),
            "COSH" => Some(self.builtin_unary_math(fb, args, "cosh")),
            "TANH" => Some(self.builtin_unary_math(fb, args, "tanh")),
            "ATN2" => Some(self.builtin_binary_math(fb, args, "atan2")),
            "ASINH" => Some(self.builtin_unary_math(fb, args, "asinh")),
            "ACOSH" => Some(self.builtin_unary_math(fb, args, "acosh")),
            "ATANH" => Some(self.builtin_unary_math(fb, args, "atanh")),
            "COTH" => Some(self.builtin_coth(fb, args)),
            "SEC" => Some(self.builtin_reciprocal(fb, args, "llvm.cos.f64")),
            "CSC" => Some(self.builtin_reciprocal(fb, args, "llvm.sin.f64")),
            "COT" => Some(self.builtin_reciprocal(fb, args, "tan")),
            "SECH" => Some(self.builtin_reciprocal(fb, args, "cosh")),
            "CSCH" => Some(self.builtin_reciprocal(fb, args, "sinh")),
            "HYPOT" => Some(self.builtin_binary_math(fb, args, "hypot")),
            "CBRT" => Some(self.builtin_unary_math(fb, args, "cbrt")),
            "EXPM1" => Some(self.builtin_unary_math(fb, args, "expm1")),
            "LOG1P" => Some(self.builtin_unary_math(fb, args, "log1p")),
            "ERF" => Some(self.builtin_unary_math(fb, args, "erf")),
            "CINT" | "CLNG" | "CDWD" | "CLNGINT" | "CUINT" | "CULNG" => {
                Some(self.builtin_cint(fb, args))
            }
            "CDBL" => Some(self.builtin_to_f64(fb, args)),
            "CVBYT" | "CVW" | "CVL" | "CVDWD" | "CVQ" => Some(self.builtin_cv_int(fb, args, name)),
            "CFLT" => Some(self.builtin_to_f32(fb, args)),
            "CSNG" => Some(self.builtin_to_f32(fb, args)),
            "CSTR" => Some(self.builtin_str(fb, args)),
            "CQUAD" => Some(self.builtin_to_i64(fb, args)),
            "CBYTE" => Some(self.builtin_to_u8(fb, args)),
            "CWORD" => Some(self.builtin_to_u16(fb, args)),
            "CDWORD" => Some(self.builtin_to_i32(fb, args)),
            "ISTRUE" => Some(self.builtin_istrue(fb, args, false)),
            "ISFALSE" => Some(self.builtin_istrue(fb, args, true)),
            "ISEVEN" => Some(self.builtin_parity(fb, args, false)),
            "ISODD" => Some(self.builtin_parity(fb, args, true)),
            "CVS" | "CVD" | "CVE" | "CVCUR" | "CVCUX" => Some(self.builtin_cv_dbl(fb, args, name)),
            "TALLY" => Some(self.builtin_tally(fb, args)),
            "STRREVERSE" => Some(self.builtin_strreverse(fb, args)),
            "STRINSERT" => Some(self.builtin_strinsert(fb, args)),
            "STRDELETE" => Some(self.builtin_strdelete(fb, args)),
            "REPEAT" => Some(self.builtin_repeat(fb, args)),
            "FRAC" => Some(self.builtin_frac(fb, args)),
            "ISFOLDER" => Some(self.builtin_isfolder(fb, args)),
            "EXP2" | "EXP10" | "LOG2" | "LOG10" => Some(self.builtin_math1(fb, args, name)),
            "IIF" => Some(self.builtin_iif(fb, args)),
            "CHOOSE" => Some(self.builtin_choose(fb, args)),
            "BIN" => Some(self.builtin_radix(fb, args, "BIN")),
            "OCT" => Some(self.builtin_radix(fb, args, "OCT")),
            "DEC" => Some(self.builtin_radix(fb, args, "DEC")),
            "VERIFY" => Some(self.builtin_verify(fb, args)),
            "MOD" => Some(self.builtin_mod(fb, args)),
            "GETATTR" => Some(self.builtin_getattr(fb, args)),
            "DISKFREE" => Some(self.builtin_disk(fb, args, "pb_diskfree")),
            "DISKSIZE" => Some(self.builtin_disk(fb, args, "pb_disksize")),
            "CHRTOOEM" => Some(self.builtin_conv1(fb, args, "pb_chr_to_oem")),
            "OEMTOCHR" => Some(self.builtin_conv1(fb, args, "pb_oem_to_chr")),
            "CHRTOUTF8" => Some(self.builtin_conv1(fb, args, "pb_chr_to_utf8")),
            "UTF8TOCHR" => Some(self.builtin_conv1(fb, args, "pb_utf8_to_chr")),
            "CLIP" => Some(self.builtin_clip(fb, args)),
            "WRAP" => Some(self.builtin_conv3(fb, args, "pb_wrap")),
            "UNWRAP" => Some(self.builtin_conv3(fb, args, "pb_unwrap")),
            "SHRINK" => Some(self.builtin_shrink(fb, args)),
            "BUILD" => Some(self.builtin_build(fb, args)),
            "DAYNAME" => Some(self.builtin_name1(fb, args, "pb_dayname")),
            "BITS" => Some(self.builtin_str2(fb, args, "pb_bits_str")),
            "PATHNAME" => Some(self.builtin_str2(fb, args, "pb_pathname")),
            "PRINTERCOUNT" => Some(self.builtin_count0(fb, "pb_printer_count")),
            "THREADID" => Some(self.builtin_threadid(fb)),
            "CHRBYTES" => {
                // CHRBYTES(var) — returns 1 for ANSI strings, 2 for wide strings
                // We only support ANSI strings, so always return 1
                Some(Ok(fb.const_i32(1)))
            }
            "ACODE" => {
                Some(self.builtin_code_page(fb, args, "pb_acode", "ACODE$ requires 1 argument"))
            }
            "UCODE" => {
                Some(self.builtin_code_page(fb, args, "pb_ucode", "UCODE$ requires 1 argument"))
            }
            "METRICS" => Some(self.builtin_metrics(fb, args)),
            "FUNCNAME" => {
                // FUNCNAME$ — returns the name of the current Sub/Function
                let name = self.current_fn_name.clone().unwrap_or_default();
                let (sname, _) = self.module.add_string_constant(&name);
                let sv = Val::new(sname, IrType::Ptr);
                let len = fb.const_i32(name.len() as i32);
                Some(Ok(fb.call(&IrType::Ptr, "pb_bstr_alloc", &[sv, len])))
            }
            "SWITCH" | "SWITCH$" => Some(self.builtin_switch(fb, args, name)),
            "HI" | "LO" => Some(self.builtin_hilo(fb, args, name)),
            "FILEATTR" => Some(self.builtin_fileattr(fb, args)),
            "FILENAME" => Some(self.builtin_filename(fb, args)),
            "PATHSCAN" => Some(self.builtin_strn(fb, args, "pb_pathscan", 2)),
            "ERL" => Some(self.builtin_str0(fb, "pb_erl_str")),
            "CBOOL" => Some(self.builtin_cbool(fb, args)),
            "ERROR" => Some(self.builtin_error_str(fb, args)),
            "EXTRACT" => Some(self.builtin_extract(fb, args)),
            "REMOVE" => Some(self.builtin_remove(fb, args)),
            "RETAIN" => Some(self.builtin_retain(fb, args)),
            "REMAIN" => Some(self.builtin_remain(fb, args)),
            "MCASE" => {
                // MCASE$(s$) — capitalize first letter of each word, lowercase rest
                let s = self.compile_expr(fb, &args[0]);
                Some(s.map(|v| fb.call(&IrType::Ptr, "pb_mcase_string", &[v])))
            }
            "RGB" | "BGR" => Some(self.builtin_rgb(fb, args, name)),
            "MONTHNAME" => Some(self.builtin_name1(fb, args, "pb_monthname")),
            "DATACOUNT" => Some(self.builtin_count0(fb, "pb_data_count")),
            "THREADCOUNT" => Some(self.builtin_count0(fb, "pb_thread_count")),
            "LBOUND" => Some(Ok(fb.const_i32(1))), /* PB default lower bound = 1 */
            "UBOUND" => Some(Ok(fb.const_i32(0))), /* TODO: track array dimensions */
            "JOIN$" => Some(self.builtin_join(fb, args)),
            "INPUTBOX$" => Some(self.builtin_inputbox(fb, args)),
            "MSGBOX" => Some(self.builtin_msgbox(fb, args)),
            "RND" => Some(self.builtin_rnd(fb, args)),
            "ROUND" => Some(self.builtin_round(fb, args)),
            // String builtins
            "LEN" => Some(self.builtin_len(fb, args)),
            "CHR" => Some(self.builtin_chr(fb, args)),
            "ASC" => Some(self.builtin_asc(fb, args)),
            "STR" => Some(self.builtin_str(fb, args)),
            "HEX" => Some(self.builtin_hex(fb, args)),
            "VAL" => Some(self.builtin_val(fb, args)),
            "LEFT" => Some(self.builtin_left(fb, args)),
            "RIGHT" => Some(self.builtin_right(fb, args)),
            "MID" => Some(self.builtin_mid(fb, args)),
            "INSTR" => Some(self.builtin_instr(fb, args)),
            "UCASE" => Some(self.builtin_ucase(fb, args)),
            "LCASE" => Some(self.builtin_lcase(fb, args)),
            "TRIM" => Some(self.builtin_trim(fb, args)),
            "LTRIM" => Some(self.builtin_ltrim(fb, args)),
            "RTRIM" => Some(self.builtin_rtrim(fb, args)),
            "SPACE" => Some(self.builtin_space(fb, args)),
            "STRING" => Some(self.builtin_string_fn(fb, args)),
            // PB runtime builtins
            "FORMAT" => Some(self.builtin_format(fb, args)),
            "PARSE" => Some(self.builtin_parse(fb, args)),
            "PARSECOUNT" => Some(self.builtin_parsecount(fb, args)),
            "USING" => Some(self.builtin_using(fb, args)),
            // File I/O builtins
            "FREEFILE" => Some(Ok(fb.call(&IrType::I32, "pb_freefile", &[]))),
            "PB_GET_CB_MSG" => Some(Ok(fb.call(&IrType::I32, "pb_get_cb_msg", &[]))),
            "PB_GET_CB_HNDL" => Some(Ok(fb.call(&IrType::Ptr, "pb_get_cb_hwnd", &[]))),
            "PB_GET_CB_CTL" => Some(Ok(fb.call(&IrType::I32, "pb_get_cb_ctl", &[]))),
            "PB_GET_CB_CTLMSG" => Some(Ok(fb.call(&IrType::I32, "pb_get_cb_ctlmsg", &[]))),
            "PB_GET_CB_WPARAM" => Some(Ok(fb.call(&IrType::I64, "pb_get_cb_wparam", &[]))),
            "PB_GET_CB_LPARAM" => Some(Ok(fb.call(&IrType::I64, "pb_get_cb_lparam", &[]))),
            "FRE" => Some(Ok(fb.call(&IrType::I64, "pb_fre", &[]))),
            "EOF" => {
                let filenum = self.compile_expr(fb, &args[0]);
                Some(filenum.map(|v| {
                    let v32 = self.to_i32(fb, &v);
                    fb.call(&IrType::I32, "pb_eof", &[v32])
                }))
            }
            // System builtins
            "TIMER" => {
                let null = Val::new("null".to_string(), IrType::Ptr);
                Some(Ok(fb.call(&IrType::I32, "time", &[null])))
            }
            "COMMAND" => {
                let cmd = if self.module.is_32bit() {
                    fb.call_stdcall(&IrType::Ptr, "GetCommandLineA", &[])
                } else {
                    fb.call(&IrType::Ptr, "GetCommandLineA", &[])
                };
                let len = fb.call(&IrType::I32, "strlen", std::slice::from_ref(&cmd));
                Some(Ok(fb.call(&IrType::Ptr, "pb_bstr_alloc", &[cmd, len])))
            }
            "EXIST" => {
                let path = self.compile_expr(fb, &args[0]);
                Some(path.map(|p| {
                    let zero = fb.const_i32(0);
                    let result = fb.call(&IrType::I32, "_access", &[p, zero.clone()]);
                    // _access returns 0 if file exists, -1 if not
                    // PB EXIST returns -1 (true) if exists, 0 if not
                    let exists = fb.icmp("eq", &result, &zero);
                    let neg_one = fb.const_i32(-1);
                    let zero2 = fb.const_i32(0);
                    fb.select(&exists, &neg_one, &zero2)
                }))
            }
            "ISFILE" => {
                // ISFILE(path$) → returns -1 (true) if the file exists, 0 if not (PB9+ alias of EXIST)
                let path = self.compile_expr(fb, &args[0]);
                Some(path.map(|p| {
                    let zero = fb.const_i32(0);
                    let result = fb.call(&IrType::I32, "_access", &[p, zero.clone()]);
                    let exists = fb.icmp("eq", &result, &zero);
                    let neg_one = fb.const_i32(-1);
                    let zero2 = fb.const_i32(0);
                    fb.select(&exists, &neg_one, &zero2)
                }))
            }
            "GETDLGITEM" => {
                // GETDLGITEM(hParent&, id&) -> HWND of the child control, or 0.
                // HWND is a numeric DWORD in PB: return it as an integer so
                // `IF CB.LPARAM = GetDlgItem(...)` compares numerically.
                // (Returning IrType::Ptr made codegen string-compare the two.)
                if args.len() >= 2 {
                    let parent = self.compile_expr(fb, &args[0]);
                    let id = self.compile_expr(fb, &args[1]);
                    Some(parent.and_then(|p| {
                        id.map(|i| {
                            let hwnd = fb.call(&IrType::Ptr, "GetDlgItem", &[p, i]);
                            fb.ptrtoint(&hwnd)
                        })
                    }))
                } else {
                    Some(Ok(fb.const_i64(0)))
                }
            }
            "ISWIN" => {
                // ISWIN(hWnd&) → -1 if window exists, 0 if not
                // ISWIN(hParent&, id&) → -1 if control exists, 0 if not
                if args.len() >= 2 {
                    let parent = self.compile_expr(fb, &args[0]);
                    let id = self.compile_expr(fb, &args[1]);
                    Some(parent.and_then(|p| {
                        id.map(|i| {
                            let hwnd = fb.call(&IrType::Ptr, "GetDlgItem", &[p, i]);
                            let result = fb.call(&IrType::I32, "IsWindow", &[hwnd]);
                            let exists = fb.icmp("ne", &result, &fb.const_i32(0));
                            fb.select(&exists, &fb.const_i32(-1), &fb.const_i32(0))
                        })
                    }))
                } else {
                    let hwnd = self.compile_expr(fb, &args[0]);
                    Some(hwnd.map(|h| {
                        let result = fb.call(&IrType::I32, "IsWindow", &[h]);
                        let exists = fb.icmp("ne", &result, &fb.const_i32(0));
                        fb.select(&exists, &fb.const_i32(-1), &fb.const_i32(0))
                    }))
                }
            }
            "DIR" => {
                // DIR$(mask [, ONLY attr]) or DIR$(NEXT) — returns BSTR filename
                // (parser emits FunctionCall "DIR$"; normalize_name strips the $)
                if args.len() == 1 {
                    if let Expr::StringLit(s) = &args[0] {
                        if s == "\u{0}DIRNEXT" {
                            return Some(Ok(fb.call(&IrType::Ptr, "pb_dir_next", &[])));
                        }
                    }
                    let mask = self.compile_expr(fb, &args[0]);
                    return Some(mask.map(|m| {
                        fb.call(
                            &IrType::Ptr,
                            "pb_dir_first",
                            &[m, fb.const_i32(0), fb.const_i32(0)],
                        )
                    }));
                }
                if args.len() >= 2 {
                    let m = match self.compile_expr(fb, &args[0]) {
                        Ok(v) => v,
                        Err(e) => return Some(Err(e)),
                    };
                    let av = match self.compile_expr(fb, &args[1]) {
                        Ok(v) => v,
                        Err(e) => return Some(Err(e)),
                    };
                    let av32 = self.to_i32(fb, &av);
                    return Some(Ok(fb.call(
                        &IrType::Ptr,
                        "pb_dir_first",
                        &[m, fb.const_i32(1), av32],
                    )));
                }
                Some(Ok(fb.call(&IrType::Ptr, "pb_dir_next", &[])))
            }
            "CURDIR" => {
                // CURDIR$ → current working directory, returned as a PB string
                let buf = fb.alloca(&IrType::Array(1024, Box::new(IrType::I8)));
                let len_const = fb.const_i32(1024);
                if self.module.is_32bit() {
                    fb.call_stdcall(
                        &IrType::I32,
                        "GetCurrentDirectoryA",
                        &[len_const.clone(), buf.clone()],
                    );
                } else {
                    fb.call(
                        &IrType::I32,
                        "GetCurrentDirectoryA",
                        &[len_const.clone(), buf.clone()],
                    );
                }
                let str_len = fb.call(&IrType::I32, "strlen", std::slice::from_ref(&buf));
                Some(Ok(fb.call(&IrType::Ptr, "pb_bstr_alloc", &[buf, str_len])))
            }
            "CALLSTKCOUNT" => Some(Ok(fb.call(&IrType::I64, "pb_callstk_count", &[]))),
            "CALLSTK" => {
                let n = if args.is_empty() {
                    Val::new("1", IrType::I32)
                } else {
                    match self.compile_expr(fb, &args[0]) {
                        Ok(v) => v,
                        Err(e) => return Some(Err(e)),
                    }
                };
                let n_i64 = self.to_i64(fb, &n);
                Some(Ok(fb.call(&IrType::Ptr, "pb_callstk_get", &[n_i64])))
            }
            "SIZEOF" => {
                // SIZEOF(var) → return size in bytes
                // For ASCIIZ*N / FixedString(N): returns N
                // For TYPE variables: returns struct size
                // For scalar types: returns type size
                if let Some(arg) = args.first() {
                    let size = match arg {
                        Expr::Variable(vname) => {
                            let norm = normalize_name(vname);
                            if let Some(info) = self.symbols.lookup(&norm) {
                                match &info.pb_type {
                                    PbType::FixedString(n) => *n as i32,
                                    PbType::Long | PbType::Dword | PbType::Single => 4,
                                    PbType::Integer | PbType::Word => 2,
                                    PbType::Byte => 1,
                                    PbType::Double | PbType::Ext | PbType::Cur | PbType::Quad => 8,
                                    PbType::String => 4, // pointer size (32-bit)
                                    PbType::UserDefined(tn) => {
                                        let tn = normalize_name(tn);
                                        self.type_layouts
                                            .get(&tn)
                                            .map(|l| {
                                                l.fields
                                                    .iter()
                                                    .map(|f| match &f.pb_type {
                                                        PbType::Long
                                                        | PbType::Dword
                                                        | PbType::Single
                                                        | PbType::String => 4,
                                                        PbType::Integer | PbType::Word => 2,
                                                        PbType::Byte => 1,
                                                        PbType::Double
                                                        | PbType::Ext
                                                        | PbType::Cur
                                                        | PbType::Quad => 8,
                                                        PbType::FixedString(n) => *n as i32,
                                                        _ => 4,
                                                    })
                                                    .sum::<i32>()
                                            })
                                            .unwrap_or(0)
                                    }
                                    _ => 4,
                                }
                            } else {
                                0
                            }
                        }
                        _ => 0,
                    };
                    Some(Ok(fb.const_i32(size)))
                } else {
                    Some(Ok(fb.const_i32(0)))
                }
            }
            "ENVIRON" => {
                let var_name = self.compile_expr(fb, &args[0]);
                Some(var_name.map(|v| fb.call(&IrType::Ptr, "pb_environ", &[v])))
            }
            "EXE" => {
                // EXE.PATH$ and EXE.NAME$ → handled via the TypeMember path
                // This shouldn't normally be reached but as fallback return empty
                Some(Ok(fb.call(&IrType::Ptr, "pb_exe_path", &[])))
            }
            "STRPTR" => {
                // STRPTR(s$) → returns pointer to string data as integer
                let s = self.compile_expr(fb, &args[0]);
                Some(s.map(|s| {
                    // The string value IS the pointer to char data
                    fb.ptrtoint64(&s)
                }))
            }
            "VARPTR" => {
                // VARPTR(v) → returns address of variable as integer
                match &args[0] {
                    Expr::Variable(orig_name) => {
                        let vname = normalize_name(orig_name);
                        let ptr_name = match self.ensure_variable_ptr(fb, &vname, orig_name) {
                            Ok(p) => p,
                            Err(e) => return Some(Err(e)),
                        };
                        let ptr = Val::new(ptr_name, IrType::Ptr);
                        Some(Ok(fb.ptrtoint64(&ptr)))
                    }
                    Expr::ArrayAccess(arr_name, indices) => {
                        let name = normalize_name(arr_name);
                        if let Some(arr_info) = self.symbols.lookup_array(&name).cloned() {
                            let mut compiled_indices = Vec::new();
                            for idx in indices {
                                match self.compile_expr(fb, idx) {
                                    Ok(v) => compiled_indices.push(v),
                                    Err(e) => return Some(Err(e)),
                                }
                            }
                            let flat_idx =
                                self.compute_flat_index(fb, &compiled_indices, &arr_info);
                            let base = Val::new(arr_info.ptr_name.clone(), IrType::Ptr);
                            let elem_ptr = fb.gep_array(&arr_info.array_ir_type, &base, &flat_idx);
                            Some(Ok(fb.ptrtoint64(&elem_ptr)))
                        } else {
                            Some(Ok(fb.const_i32(0)))
                        }
                    }
                    _ => {
                        // Expression: allocate temp and return its address
                        match self.compile_expr(fb, &args[0]) {
                            Ok(val) => {
                                let tmp = fb.alloca(&val.ty);
                                fb.store(&val, &tmp);
                                Some(Ok(fb.ptrtoint64(&tmp)))
                            }
                            Err(e) => Some(Err(e)),
                        }
                    }
                }
            }
            "LOBYT" | "LOWRD" => {
                let v = self.compile_expr(fb, &args[0]);
                Some(v.map(|v| {
                    let v32 = self.to_i32(fb, &v);
                    let mask = fb.const_i32(0xFF);
                    fb.and(&v32, &mask)
                }))
            }
            "HIBYT" | "HIWRD" => {
                let v = self.compile_expr(fb, &args[0]);
                Some(v.map(|v| {
                    let v32 = self.to_i32(fb, &v);
                    let eight = fb.const_i32(8);
                    let shifted = fb.lshr(&v32, &eight);
                    let mask = fb.const_i32(0xFF);
                    fb.and(&shifted, &mask)
                }))
            }
            "MAKDWD" | "MAK" => {
                if args.len() >= 2 {
                    let lo = self.compile_expr(fb, &args[0]);
                    Some(lo.and_then(|lo| {
                        let hi = self.compile_expr(fb, &args[1])?;
                        let lo32 = self.to_i32(fb, &lo);
                        let hi32 = self.to_i32(fb, &hi);
                        let sixteen = fb.const_i32(16);
                        let hi_shifted = fb.shl(&hi32, &sixteen);
                        Ok(fb.or(&lo32, &hi_shifted))
                    }))
                } else {
                    Some(Ok(fb.const_i32(0)))
                }
            }
            "MKI" | "MKWRD" => {
                // MKI$ / MKWRD$ — 2-byte binary string
                if args.is_empty() {
                    return Some(Err(PbError::runtime("MKI$/MKWRD$ requires 1 argument")));
                }
                let v = self.compile_expr(fb, &args[0]);
                Some(v.map(|val| {
                    let b = self.to_i32(fb, &val);
                    fb.call(&IrType::Ptr, "pb_mkint", &[b])
                }))
            }
            "MKL" | "MKDWD" => {
                // MKL$ / MKDWD$ — 4-byte binary string
                if args.is_empty() {
                    return Some(Err(PbError::runtime("MKL$/MKDWD$ requires 1 argument")));
                }
                let v = self.compile_expr(fb, &args[0]);
                Some(v.map(|val| {
                    let b = self.to_i32(fb, &val);
                    fb.call(&IrType::Ptr, "pb_mklong", &[b])
                }))
            }
            "MKQ" | "MKCUR" | "MKCUX" => {
                // MKQ$ / MKCUR$ / MKCUX$ — 8-byte binary string
                if args.is_empty() {
                    return Some(Err(PbError::runtime(
                        "MKQ$/MKCUR$/MKCUX$ requires 1 argument",
                    )));
                }
                let v = self.compile_expr(fb, &args[0]);
                Some(v.map(|val| {
                    let b = self.to_i64(fb, &val);
                    fb.call(&IrType::Ptr, "pb_mkquad", &[b])
                }))
            }
            "MKS" => {
                // MKS$ — 4-byte binary string (single precision)
                if args.is_empty() {
                    return Some(Err(PbError::runtime("MKS$ requires 1 argument")));
                }
                let v = self.compile_expr(fb, &args[0]);
                Some(v.map(|val| {
                    let d = self.to_f64(fb, &val);
                    let f = fb.fptrunc(&d, &IrType::Float);
                    fb.call(&IrType::Ptr, "pb_mksingle", &[f])
                }))
            }
            "MKD" | "MKE" => {
                // MKD$ — 8-byte binary string (double precision)
                // MKE$ — EXT in this compiler is an 8-byte IEEE-754 double (the official
                // PB 10-byte 80-bit extended format is not modelled), so MKE$ yields the
                // same 8 bytes as MKD$. Documented in the coverage matrix.
                if args.is_empty() {
                    return Some(Err(PbError::runtime("MKD$ requires 1 argument")));
                }
                let v = self.compile_expr(fb, &args[0]);
                Some(v.map(|val| {
                    let d = self.to_f64(fb, &val);
                    fb.call(&IrType::Ptr, "pb_mkdouble", &[d])
                }))
            }
            "MKBYT" => {
                // MKBYT$ (n) — one byte as a single-character string
                if args.is_empty() {
                    return Some(Err(PbError::runtime("MKBYT$ requires 1 argument")));
                }
                let v = self.compile_expr(fb, &args[0]);
                Some(v.map(|val| {
                    let b = self.to_i32(fb, &val);
                    fb.call(&IrType::Ptr, "pb_mkbyt", &[b])
                }))
            }
            "LOF" | "LOC" | "SEEK" => {
                // LOF(f) / LOC(f) / SEEK(f) — file size / position (optional #)
                if args.is_empty() {
                    return Some(Err(PbError::runtime(format!(
                        "{} requires a file number",
                        name
                    ))));
                }
                let v = self.compile_expr(fb, &args[0]);
                Some(v.map(|val| {
                    let f = self.to_i32(fb, &val);
                    let fn_name = if name == "LOF" { "pb_lof" } else { "pb_loc" };
                    fb.call(&IrType::I64, fn_name, &[f])
                }))
            }
            "BIT" => {
                // BIT(intvar, bitnumber) — 0 or 1
                if args.len() < 2 {
                    return Some(Err(PbError::runtime("BIT requires 2 arguments")));
                }
                let v = self.compile_expr(fb, &args[0]);
                let bit = self.compile_expr(fb, &args[1]);
                Some(match (v, bit) {
                    (Ok(vv), Ok(bb)) => {
                        let vi = self.to_i64(fb, &vv);
                        let bi = self.to_i64(fb, &bb);
                        let shifted = fb.lshr(&vi, &bi);
                        let one = fb.const_i64(1);
                        let r = fb.and(&shifted, &one);
                        Ok(self.to_i32(fb, &r))
                    }
                    (Err(e), _) | (_, Err(e)) => Err(e),
                })
            }
            "BITSE" => {
                // BITSE(intvar, bitnumber) — test bit and set it, return original bit value (0 or 1)
                if args.len() < 2 {
                    return Some(Err(PbError::runtime("BITSE requires 2 arguments")));
                }
                if let Some((ptr, ir_ty, pb_ty)) = self.lvalue_ptr(fb, &args[0]) {
                    let cur = fb.load(&ir_ty, &ptr);
                    let vi = self.to_i64(fb, &cur);
                    let bitv = match self.compile_expr(fb, &args[1]) {
                        Ok(b) => b,
                        Err(e) => return Some(Err(e)),
                    };
                    let bi = self.to_i64(fb, &bitv);
                    let one = fb.const_i64(1);
                    // Test original bit
                    let shifted = fb.lshr(&vi, &bi);
                    let orig = fb.and(&shifted, &one);
                    // Set the bit
                    let mask = fb.shl(&one, &bi);
                    let newv = fb.or(&vi, &mask);
                    let conv = self.convert_value(fb, &newv, &ir_ty, &pb_ty);
                    fb.store(&conv, &ptr);
                    // Return original bit value
                    Some(Ok(self.to_i32(fb, &orig)))
                } else {
                    Some(Err(PbError::runtime(
                        "BITSE first argument must be a variable",
                    )))
                }
            }
            "PEEK" => {
                // PEEK([datatype,] address) — default BYTE
                let (dt, addr_expr) = match args.first() {
                    Some(Expr::Variable(n))
                        if matches!(
                            normalize_name(n).as_str(),
                            "BYTE"
                                | "WORD"
                                | "DWORD"
                                | "INTEGER"
                                | "LONG"
                                | "QUAD"
                                | "SINGLE"
                                | "DOUBLE"
                        ) =>
                    {
                        (normalize_name(n), args.get(1))
                    }
                    _ => ("BYTE".to_string(), args.first()),
                };
                let ae = match addr_expr {
                    Some(e) => e,
                    None => return Some(Ok(fb.const_i32(0))),
                };
                let av = self.compile_expr(fb, ae);
                Some(av.map(|av| {
                    let av64 = if av.ty == IrType::I32 {
                        fb.sext(&av, &IrType::I64)
                    } else {
                        av
                    };
                    let addr = fb.inttoptr(&av64);
                    match dt.as_str() {
                        "BYTE" | "INTEGER" => fb.call(&IrType::I32, "pb_peek8", &[addr]),
                        "WORD" => fb.call(&IrType::I32, "pb_peek16", &[addr]),
                        "DWORD" | "LONG" => fb.call(&IrType::I32, "pb_peek32", &[addr]),
                        "QUAD" => fb.call(&IrType::I64, "pb_peek64", &[addr]),
                        "SINGLE" => fb.call(&IrType::Float, "pb_peekf", &[addr]),
                        "DOUBLE" => fb.call(&IrType::Double, "pb_peekd", &[addr]),
                        _ => fb.call(&IrType::I32, "pb_peek8", &[addr]),
                    }
                }))
            }
            "CODEPTR" => {
                // CODEPTR(BlockName) — address of an ASMDATA read-only block
                if let Some(Expr::Variable(n)) = args.first() {
                    let up = normalize_name(n);
                    if let Some(sym) = self.asmdata_blocks.get(&up) {
                        let gv = Val::new(sym.clone(), IrType::Ptr);
                        return Some(Ok(fb.ptrtoint64(&gv)));
                    }
                }
                // Fall through: CODEPTR of anything else is unsupported
                None
            }
            "ISINFINITE" | "ISNORMAL" => {
                // ISINFINITE (x) / ISNORMAL (x) — IEEE-754 checks, -1/0
                if args.is_empty() {
                    return Some(Err(PbError::runtime(format!(
                        "{} requires 1 argument",
                        name
                    ))));
                }
                let v = self.compile_expr(fb, &args[0]);
                Some(v.map(|val| {
                    let d = self.to_f64(fb, &val);
                    let fn_name = if name == "ISINFINITE" {
                        "pb_isinfinite"
                    } else {
                        "pb_isnormal"
                    };
                    fb.call(&IrType::I32, fn_name, &[d])
                }))
            }
            _ => None,
        }
    }

    /// Compile a CONTROL ADD ... CALL callback argument into a function pointer.
    /// "_no_cb" placeholder (parser default) -> null; a known function/sub name ->
    /// its IR entry pointer; anything else -> compiled expression (must be a ptr).
    fn compile_control_callback(
        &mut self,
        fb: &mut FunctionBuilder,
        arg: &Expr,
    ) -> Result<Val, pb::error::PbError> {
        if let Expr::Variable(name) = arg {
            if name == "_no_cb" {
                return Ok(Val::new("null".to_string(), IrType::Ptr));
            }
            let upper = name.to_uppercase();
            let fi = self.functions.get(&upper).or_else(|| self.subs.get(&upper));
            if let Some(fi) = fi {
                return Ok(Val::new(format!("@{}", fi.ir_name), IrType::Ptr));
            }
            eprintln!(
                "warning: CONTROL ADD ... CALL: unknown callback '{}'",
                upper
            );
            return Ok(Val::new("null".to_string(), IrType::Ptr));
        }
        // batch 249: official samples write "CALL ButtonCallback()" with parens,
        // which parses as a FunctionCall, not a Variable.
        if let Expr::FunctionCall(name, _) = arg {
            let upper = name.to_uppercase();
            let fi = self.functions.get(&upper).or_else(|| self.subs.get(&upper));
            if let Some(fi) = fi {
                return Ok(Val::new(format!("@{}", fi.ir_name), IrType::Ptr));
            }
            eprintln!(
                "warning: CONTROL ADD ... CALL: unknown callback '{}'",
                upper
            );
            return Ok(Val::new("null".to_string(), IrType::Ptr));
        }
        let v = self.compile_expr(fb, arg)?;
        if v.ty != IrType::Ptr {
            return Ok(fb.inttoptr(&v));
        }
        Ok(v)
    }

    /// Returns (writable ptr, IR type, PB type) for a variable or array element.
    fn lvalue_ptr(
        &mut self,
        fb: &mut FunctionBuilder,
        expr: &Expr,
    ) -> Option<(Val, IrType, PbType)> {
        match expr {
            Expr::Variable(orig_name) => {
                let name = normalize_name(orig_name);
                let info = self.symbols.lookup(&name)?.clone();
                Some((
                    Val::new(info.ptr_name.clone(), IrType::Ptr),
                    info.ir_type.clone(),
                    info.pb_type.clone(),
                ))
            }
            Expr::ArrayAccess(arr_name, indices) | Expr::FunctionCall(arr_name, indices) => {
                let name = normalize_name(arr_name);
                let arr_info = self.symbols.lookup_array(&name)?.clone();
                let mut compiled_indices = Vec::new();
                for idx in indices {
                    compiled_indices.push(self.compile_expr(fb, idx).ok()?);
                }
                let flat_idx = self.compute_flat_index(fb, &compiled_indices, &arr_info);
                let base = Val::new(arr_info.ptr_name.clone(), IrType::Ptr);
                let elem_ptr = fb.gep_array(&arr_info.array_ir_type, &base, &flat_idx);
                Some((
                    elem_ptr,
                    arr_info.elem_ir_type.clone(),
                    arr_info.pb_type.clone(),
                ))
            }
            _ => None,
        }
    }

    fn builtin_abs(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        let val = self.compile_expr(fb, &args[0])?;
        if val.ty.is_float() {
            let f64_val = self.to_f64(fb, &val);
            Ok(fb.call(&IrType::Double, "llvm.fabs.f64", &[f64_val]))
        } else {
            let i32_val = self.to_i32(fb, &val);
            let zero = fb.const_i32(0);
            let is_neg = fb.icmp("slt", &i32_val, &zero);
            let neg_val = fb.neg(&i32_val);
            Ok(fb.select(&is_neg, &neg_val, &i32_val))
        }
    }

    fn builtin_sgn(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        let val = self.compile_expr(fb, &args[0])?;
        let pos_one = fb.const_i32(1);
        let neg_one = fb.const_i32(-1);
        let zero_i32 = fb.const_i32(0);
        if val.ty.is_float() {
            let f64_val = self.to_f64(fb, &val);
            let zero_f = fb.const_f64(0.0);
            let is_pos = fb.fcmp("ogt", &f64_val, &zero_f);
            let is_neg = fb.fcmp("olt", &f64_val, &zero_f);
            let tmp = fb.select(&is_neg, &neg_one, &zero_i32);
            Ok(fb.select(&is_pos, &pos_one, &tmp))
        } else {
            let i32_val = self.to_i32(fb, &val);
            let is_pos = fb.icmp("sgt", &i32_val, &zero_i32);
            let is_neg = fb.icmp("slt", &i32_val, &zero_i32);
            let tmp = fb.select(&is_neg, &neg_one, &zero_i32);
            Ok(fb.select(&is_pos, &pos_one, &tmp))
        }
    }

    fn builtin_min_max(
        &mut self,
        fb: &mut FunctionBuilder,
        args: &[Expr],
        is_max: bool,
    ) -> PbResult<Val> {
        let a = self.compile_expr(fb, &args[0])?;
        let b = self.compile_expr(fb, &args[1])?;
        if a.ty.is_float() || b.ty.is_float() {
            let fa = self.to_f64(fb, &a);
            let fb_val = self.to_f64(fb, &b);
            let cmp = if is_max {
                fb.fcmp("ogt", &fa, &fb_val)
            } else {
                fb.fcmp("olt", &fa, &fb_val)
            };
            Ok(fb.select(&cmp, &fa, &fb_val))
        } else {
            let (ia, ib) = self.promote_ints(fb, &a, &b);
            let cmp = if is_max {
                fb.icmp("sgt", &ia, &ib)
            } else {
                fb.icmp("slt", &ia, &ib)
            };
            Ok(fb.select(&cmp, &ia, &ib))
        }
    }

    fn builtin_int(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        let val = self.compile_expr(fb, &args[0])?;
        if val.ty.is_float() {
            let f64_val = self.to_f64(fb, &val);
            let floored = fb.call(&IrType::Double, "llvm.floor.f64", &[f64_val]);
            Ok(fb.fptosi(&floored, &IrType::I32))
        } else {
            Ok(self.to_i32(fb, &val))
        }
    }

    fn builtin_fix(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        let val = self.compile_expr(fb, &args[0])?;
        if val.ty.is_float() {
            // fptosi truncates toward zero (same as FIX)
            Ok(fb.fptosi(&val, &IrType::I32))
        } else {
            Ok(self.to_i32(fb, &val))
        }
    }

    fn builtin_ceil(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        let val = self.compile_expr(fb, &args[0])?;
        let f64_val = self.to_f64(fb, &val);
        let ceiled = fb.call(&IrType::Double, "llvm.ceil.f64", &[f64_val]);
        Ok(fb.fptosi(&ceiled, &IrType::I32))
    }

    fn builtin_floor(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        let val = self.compile_expr(fb, &args[0])?;
        let f64_val = self.to_f64(fb, &val);
        let floored = fb.call(&IrType::Double, "llvm.floor.f64", &[f64_val]);
        Ok(fb.fptosi(&floored, &IrType::I32))
    }

    fn builtin_trunc(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        let val = self.compile_expr(fb, &args[0])?;
        let f64_val = self.to_f64(fb, &val);
        let truncated = fb.call(&IrType::Double, "llvm.trunc.f64", &[f64_val]);
        Ok(fb.fptosi(&truncated, &IrType::I32))
    }

    fn builtin_unary_math(
        &mut self,
        fb: &mut FunctionBuilder,
        args: &[Expr],
        func_name: &str,
    ) -> PbResult<Val> {
        let val = self.compile_expr(fb, &args[0])?;
        let f64_val = self.to_f64(fb, &val);
        Ok(fb.call(&IrType::Double, func_name, &[f64_val]))
    }

    // Batch 94: two-argument math (ATN2(y, x) -> atan2)
    fn builtin_binary_math(
        &mut self,
        fb: &mut FunctionBuilder,
        args: &[Expr],
        func_name: &str,
    ) -> PbResult<Val> {
        let a = self.compile_expr(fb, &args[0])?;
        let b = self.compile_expr(fb, &args[1])?;
        let fa = self.to_f64(fb, &a);
        let fb_val = self.to_f64(fb, &b);
        Ok(fb.call(&IrType::Double, func_name, &[fa, fb_val]))
    }

    // Batch 94: COTH(x) = 1/tanh(x)
    fn builtin_coth(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        let val = self.compile_expr(fb, &args[0])?;
        let f64_val = self.to_f64(fb, &val);
        let t = fb.call(&IrType::Double, "tanh", &[f64_val]);
        let one = fb.const_f64(1.0);
        Ok(fb.fdiv(&one, &t))
    }

    // Batch 95: generic reciprocal 1/f(x) — SEC=1/cos, CSC=1/sin, COT=1/tan, SECH=1/cosh, CSCH=1/sinh
    fn builtin_reciprocal(
        &mut self,
        fb: &mut FunctionBuilder,
        args: &[Expr],
        func_name: &str,
    ) -> PbResult<Val> {
        let val = self.compile_expr(fb, &args[0])?;
        let f64_val = self.to_f64(fb, &val);
        let t = fb.call(&IrType::Double, func_name, &[f64_val]);
        let one = fb.const_f64(1.0);
        Ok(fb.fdiv(&one, &t))
    }

    // Batch 91 fix: CINT/CLNG/CLNGINT/CUINT/CULNG — round to nearest (ties away from zero), not truncate.
    fn builtin_cint(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        let val = self.compile_expr(fb, &args[0])?;
        match &val.ty {
            IrType::Double | IrType::Float => {
                let f64_val = self.to_f64(fb, &val);
                // PB CINT/CLNG uses banker's rounding (round half to even).
                // llvm.nearbyint.f64 = ties to even (NOT llvm.round which is ties away from zero).
                let rounded = fb.call(&IrType::Double, "llvm.nearbyint.f64", &[f64_val]);
                Ok(fb.fptosi(&rounded, &IrType::I32))
            }
            _ => Ok(self.to_i32(fb, &val)),
        }
    }

    fn builtin_to_i32(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        let val = self.compile_expr(fb, &args[0])?;
        Ok(self.to_i32(fb, &val))
    }

    // Batch 87: CQUAD — convert to 64-bit signed integer
    fn builtin_to_i64(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        let val = self.compile_expr(fb, &args[0])?;
        Ok(self.to_i64(fb, &val))
    }

    // Batch 87: CBYTE — convert to unsigned 8-bit (0..255). Truncate low byte.
    fn builtin_to_u8(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        let val = self.compile_expr(fb, &args[0])?;
        let i32v = self.to_i32(fb, &val);
        Ok(fb.trunc(&i32v, &IrType::I8))
    }

    // Batch 87: CWORD — convert to unsigned 16-bit (0..65535). Truncate low word.
    fn builtin_to_u16(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        let val = self.compile_expr(fb, &args[0])?;
        let i32v = self.to_i32(fb, &val);
        Ok(fb.trunc(&i32v, &IrType::I16))
    }

    // Batch 88: ISTRUE/ISFALSE — PB boolean: TRUE=-1 (all bits 1), FALSE=0.
    // invert=false: ISTRUE (non-zero -> -1, zero -> 0)
    // invert=true:  ISFALSE (zero -> -1, non-zero -> 0)
    fn builtin_istrue(
        &mut self,
        fb: &mut FunctionBuilder,
        args: &[Expr],
        invert: bool,
    ) -> PbResult<Val> {
        let val = self.compile_expr(fb, &args[0])?;
        let i32v = self.to_i32(fb, &val);
        let zero = fb.const_i32(0);
        let cmp = if invert {
            fb.icmp("eq", &i32v, &zero)
        } else {
            fb.icmp("ne", &i32v, &zero)
        };
        let zext = fb.zext(&cmp, &IrType::I32); // 0 or 1
        Ok(fb.neg(&zext)) // 0 or -1 (PB TRUE=-1)
    }

    // Batch 90: CBOOL — convert expression to boolean (non-zero -> -1, zero -> 0).
    fn builtin_cbool(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        if args.is_empty() {
            return Err(PbError::runtime("CBOOL requires an argument"));
        }
        let val = self.compile_expr(fb, &args[0])?;
        let i32v = self.to_i32(fb, &val);
        let zero = fb.const_i32(0);
        let cmp = fb.icmp("ne", &i32v, &zero);
        let zext = fb.zext(&cmp, &IrType::I32); // 0 or 1
        Ok(fb.neg(&zext)) // 0 or -1 (PB TRUE=-1)
    }

    // Batch 88: ISEVEN/ISODD — test parity of integer.
    // odd=false: ISEVEN (even -> -1, odd -> 0)
    // odd=true:  ISODD (odd -> -1, even -> 0)
    fn builtin_parity(
        &mut self,
        fb: &mut FunctionBuilder,
        args: &[Expr],
        odd: bool,
    ) -> PbResult<Val> {
        let val = self.compile_expr(fb, &args[0])?;
        let i32v = self.to_i32(fb, &val);
        let one = fb.const_i32(1);
        let lsb = fb.and(&i32v, &one); // 0 if even, 1 if odd
        let zero = fb.const_i32(0);
        let cmp = if odd {
            fb.icmp("eq", &lsb, &one) // odd: lsb==1
        } else {
            fb.icmp("eq", &lsb, &zero) // even: lsb==0
        };
        let zext = fb.zext(&cmp, &IrType::I32);
        Ok(fb.neg(&zext)) // 0 or -1
    }

    // Batch 37: CVx — read little-endian binary strings. off is 1-based, default 1.
    fn builtin_cv_int(
        &mut self,
        fb: &mut FunctionBuilder,
        args: &[Expr],
        name: &str,
    ) -> PbResult<Val> {
        if args.is_empty() {
            return Err(PbError::runtime("CVx$ requires 1 argument"));
        }
        let (n, mode) = match name {
            "CVBYT" => (1, 0),
            "CVW" => (2, 0),
            "CVDWD" => (4, 0),
            "CVL" => (4, 1),
            _ => (8, 1), // CVQ
        };
        let sval = self.compile_expr(fb, &args[0])?;
        let off = if args.len() >= 2 {
            let o = self.compile_expr(fb, &args[1])?;
            self.to_i64(fb, &o)
        } else {
            fb.const_i32(1)
        };
        Ok(fb.call(
            &IrType::I64,
            "pb_cv_int",
            &[sval, off, fb.const_i32(n), fb.const_i32(mode)],
        ))
    }
    fn builtin_cv_dbl(
        &mut self,
        fb: &mut FunctionBuilder,
        args: &[Expr],
        name: &str,
    ) -> PbResult<Val> {
        if args.is_empty() {
            return Err(PbError::runtime("CVx$ requires 1 argument"));
        }
        let n = if name == "CVS" { 4 } else { 8 };
        let sval = self.compile_expr(fb, &args[0])?;
        let off = if args.len() >= 2 {
            let o = self.compile_expr(fb, &args[1])?;
            self.to_i64(fb, &o)
        } else {
            fb.const_i32(1)
        };
        Ok(fb.call(&IrType::Double, "pb_cv_dbl", &[sval, off, fb.const_i32(n)]))
    }

    // Batch 38: string / math builtins
    fn builtin_tally(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        if args.len() < 2 {
            return Err(PbError::runtime("TALLY requires 2 arguments"));
        }
        let s = self.compile_expr(fb, &args[0])?;
        let m = self.compile_expr(fb, &args[1])?;
        Ok(fb.call(&IrType::I64, "pb_tally", &[s, m]))
    }
    fn builtin_strreverse(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        if args.is_empty() {
            return Err(PbError::runtime("STRREVERSE$ requires 1 argument"));
        }
        let s = self.compile_expr(fb, &args[0])?;
        Ok(fb.call(&IrType::Ptr, "pb_strreverse", std::slice::from_ref(&s)))
    }
    fn builtin_strinsert(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        if args.len() < 3 {
            return Err(PbError::runtime("STRINSERT$ requires 3 arguments"));
        }
        let s = self.compile_expr(fb, &args[0])?;
        let n = self.compile_expr(fb, &args[1])?;
        let pos = self.compile_expr(fb, &args[2])?;
        let pos_i = self.to_i64(fb, &pos);
        Ok(fb.call(&IrType::Ptr, "pb_strinsert", &[s, n, pos_i]))
    }
    fn builtin_strdelete(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        if args.len() < 3 {
            return Err(PbError::runtime("STRDELETE$ requires 3 arguments"));
        }
        let s = self.compile_expr(fb, &args[0])?;
        let st = self.compile_expr(fb, &args[1])?;
        let ct = self.compile_expr(fb, &args[2])?;
        let st_i = self.to_i64(fb, &st);
        let ct_i = self.to_i64(fb, &ct);
        Ok(fb.call(&IrType::Ptr, "pb_strdelete", &[s, st_i, ct_i]))
    }
    fn builtin_repeat(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        if args.len() < 2 {
            return Err(PbError::runtime("REPEAT$ requires 2 arguments"));
        }
        let n = self.compile_expr(fb, &args[0])?;
        let s = self.compile_expr(fb, &args[1])?;
        let n_i = self.to_i64(fb, &n);
        Ok(fb.call(&IrType::Ptr, "pb_repeat", &[n_i, s]))
    }
    fn builtin_frac(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        if args.is_empty() {
            return Err(PbError::runtime("FRAC requires 1 argument"));
        }
        let v = self.compile_expr(fb, &args[0])?;
        let d = self.to_f64(fb, &v);
        Ok(fb.call(&IrType::Double, "pb_frac", std::slice::from_ref(&d)))
    }
    fn builtin_isfolder(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        if args.is_empty() {
            return Err(PbError::runtime("ISFOLDER requires 1 argument"));
        }
        let s = self.compile_expr(fb, &args[0])?;
        Ok(fb.call(&IrType::I64, "pb_isfolder", std::slice::from_ref(&s)))
    }
    fn builtin_math1(
        &mut self,
        fb: &mut FunctionBuilder,
        args: &[Expr],
        name: &str,
    ) -> PbResult<Val> {
        if args.is_empty() {
            return Err(PbError::runtime("math function requires 1 argument"));
        }
        let v = self.compile_expr(fb, &args[0])?;
        let d = self.to_f64(fb, &v);
        let fname = match name {
            "EXP2" => "pb_exp2",
            "EXP10" => "pb_exp10",
            "LOG2" => "pb_log2",
            _ => "pb_log10",
        };
        Ok(fb.call(&IrType::Double, fname, std::slice::from_ref(&d)))
    }
    fn builtin_iif(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        if args.len() < 3 {
            return Err(PbError::runtime("IIF requires 3 arguments"));
        }
        let cond = self.compile_expr(fb, &args[0])?;
        let cond_i = self.to_i64(fb, &cond);
        let zero = fb.const_i64(0);
        let is_true = fb.icmp("ne", &cond_i, &zero);
        let t = self.compile_expr(fb, &args[1])?;
        let f = self.compile_expr(fb, &args[2])?;
        match t.ty {
            IrType::Ptr => Ok(fb.select(&is_true, &t, &f)),
            IrType::Double | IrType::Float => {
                let td = self.to_f64(fb, &t);
                let fd = self.to_f64(fb, &f);
                Ok(fb.select(&is_true, &td, &fd))
            }
            _ => {
                let ti = self.to_i64(fb, &t);
                let fi = self.to_i64(fb, &f);
                Ok(fb.select(&is_true, &ti, &fi))
            }
        }
    }
    fn builtin_choose(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        if args.len() < 2 {
            return Err(PbError::runtime(
                "CHOOSE requires an index and at least 1 choice",
            ));
        }
        let idx = self.compile_expr(fb, &args[0])?;
        let idx_i = self.to_i64(fb, &idx);
        // PBWin10: out-of-range index returns the type's default value
        // (0 for numerics, "" for strings) - probe: CHOOSE(9,10,20,30)=0.
        let first = self.compile_expr(fb, &args[1])?;
        let mut r = match first.ty {
            IrType::Ptr => {
                let (s, _) = self.module.add_string_constant("");
                Val::new(s, IrType::Ptr)
            }
            IrType::Double | IrType::Float => fb.const_f64(0.0),
            _ => fb.const_i64(0),
        };
        for (i, c) in args[1..].iter().enumerate() {
            let cv = self.compile_expr(fb, c)?;
            let n_i = fb.const_i64(i as i64 + 1);
            let eq = fb.icmp("eq", &idx_i, &n_i);
            r = match r.ty {
                IrType::Ptr => fb.select(&eq, &cv, &r),
                IrType::Double | IrType::Float => {
                    let rd = self.to_f64(fb, &r);
                    let cd = self.to_f64(fb, &cv);
                    fb.select(&eq, &cd, &rd)
                }
                _ => {
                    let ri = self.to_i64(fb, &r);
                    let ci = self.to_i64(fb, &cv);
                    fb.select(&eq, &ci, &ri)
                }
            };
        }
        Ok(r)
    }

    // Batch 39: radix strings, VERIFY, MOD, file-system
    fn builtin_radix(
        &mut self,
        fb: &mut FunctionBuilder,
        args: &[Expr],
        which: &str,
    ) -> PbResult<Val> {
        if args.is_empty() {
            return Err(PbError::runtime("radix function requires 1 argument"));
        }
        let v = self.compile_expr(fb, &args[0])?;
        let vi = self.to_i64(fb, &v);
        let fname = match which {
            "BIN" => "pb_bin",
            "OCT" => "pb_oct",
            _ => "pb_dec",
        };
        Ok(fb.call(&IrType::Ptr, fname, std::slice::from_ref(&vi)))
    }
    fn builtin_verify(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        // VERIFY([start&,] MainString, MatchString)
        let (start, s, m) = if args.len() >= 3 {
            let st = self.compile_expr(fb, &args[0])?;
            let s1 = self.compile_expr(fb, &args[1])?;
            let m1 = self.compile_expr(fb, &args[2])?;
            (Some(st), s1, m1)
        } else if args.len() == 2 {
            (
                None,
                self.compile_expr(fb, &args[0])?,
                self.compile_expr(fb, &args[1])?,
            )
        } else {
            return Err(PbError::runtime("VERIFY requires 2 or 3 arguments"));
        };
        let one = fb.const_i64(1);
        let start_v = match start {
            Some(st) => self.to_i64(fb, &st),
            None => one.clone(),
        };
        Ok(fb.call(&IrType::I64, "pb_verify", &[s, m, start_v]))
    }
    fn builtin_mod(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        if args.len() < 2 {
            return Err(PbError::runtime("MOD requires 2 arguments"));
        }
        let a = self.compile_expr(fb, &args[0])?;
        let b = self.compile_expr(fb, &args[1])?;
        let ai = self.to_i64(fb, &a);
        let bi = self.to_i64(fb, &b);
        Ok(fb.srem(&ai, &bi))
    }
    fn builtin_getattr(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        if args.is_empty() {
            return Err(PbError::runtime("GETATTR requires 1 argument"));
        }
        let s = self.compile_expr(fb, &args[0])?;
        Ok(fb.call(&IrType::I64, "pb_getattr", std::slice::from_ref(&s)))
    }
    fn builtin_disk(
        &mut self,
        fb: &mut FunctionBuilder,
        args: &[Expr],
        fname: &str,
    ) -> PbResult<Val> {
        let s = if args.is_empty() {
            let empty = self.empty_string_name.clone();
            Val::new(empty, IrType::Ptr)
        } else {
            self.compile_expr(fb, &args[0])?
        };
        Ok(fb.call(&IrType::I64, fname, std::slice::from_ref(&s)))
    }

    fn builtin_conv3(
        &mut self,
        fb: &mut FunctionBuilder,
        args: &[Expr],
        fname: &str,
    ) -> PbResult<Val> {
        if args.len() != 3 {
            return Err(PbError::runtime(format!(
                "{fname} requires 3 arguments (got {})",
                args.len()
            )));
        }
        let a0 = self.compile_expr(fb, &args[0])?;
        let a1 = self.compile_expr(fb, &args[1])?;
        let a2 = self.compile_expr(fb, &args[2])?;
        Ok(fb.call(&IrType::Ptr, fname, &[a0, a1, a2]))
    }

    fn builtin_clip(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        if args.len() != 3 && args.len() != 4 {
            return Err(PbError::runtime(format!(
                "CLIP$ requires 3 or 4 arguments (got {}: {:?})",
                args.len(),
                args
            )));
        }
        let mode = self.compile_expr(fb, &args[0])?;
        let s = self.compile_expr(fb, &args[1])?;
        let z = fb.const_i64(0);
        let (start, count) = if args.len() == 4 {
            (
                self.compile_expr(fb, &args[2])?,
                self.compile_expr(fb, &args[3])?,
            )
        } else {
            (z.clone(), self.compile_expr(fb, &args[2])?)
        };
        Ok(fb.call(&IrType::Ptr, "pb_clip", &[mode, s, start, count]))
    }

    fn builtin_shrink(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        if args.is_empty() {
            return Err(PbError::runtime("SHRINK$ requires 1 argument"));
        }
        let s = self.compile_expr(fb, &args[0])?;
        let mask = if args.len() > 1 {
            self.compile_expr(fb, &args[1])?
        } else {
            fb.const_i64(0)
        };
        Ok(fb.call(&IrType::Ptr, "pb_shrink", &[s, mask]))
    }

    fn builtin_build(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        if args.is_empty() {
            return Err(PbError::runtime("BUILD$ requires at least 1 argument"));
        }
        let n = args.len();
        let arr_ty = IrType::Array(n, Box::new(IrType::Ptr));
        let arr = fb.alloca(&arr_ty);
        for (i, e) in args.iter().enumerate() {
            let v = self.compile_expr(fb, e)?;
            let slot = fb.gep_array(&arr_ty, &arr, &fb.const_i32(i as i32));
            fb.store(&v, &slot);
        }
        let nv = fb.const_i64(n as i64);
        Ok(fb.call(&IrType::Ptr, "pb_build", &[arr, nv]))
    }

    fn builtin_name1(
        &mut self,
        fb: &mut FunctionBuilder,
        args: &[Expr],
        fname: &str,
    ) -> PbResult<Val> {
        if args.len() != 1 {
            return Err(PbError::runtime(format!("{fname} requires 1 argument")));
        }
        let n = self.compile_expr(fb, &args[0])?;
        Ok(fb.call(&IrType::Ptr, fname, std::slice::from_ref(&n)))
    }

    fn builtin_str2(
        &mut self,
        fb: &mut FunctionBuilder,
        args: &[Expr],
        fname: &str,
    ) -> PbResult<Val> {
        if args.len() != 2 {
            return Err(PbError::runtime(format!("{fname} requires 2 arguments")));
        }
        let a0 = self.compile_expr(fb, &args[0])?;
        let a1 = self.compile_expr(fb, &args[1])?;
        let v = fb.call(&IrType::Ptr, fname, &[a0, a1]);
        Ok(self.null_guard_string(fb, &v))
    }

    fn builtin_count0(&mut self, fb: &mut FunctionBuilder, fname: &str) -> PbResult<Val> {
        Ok(fb.call(&IrType::I64, fname, &[]))
    }

    fn builtin_join(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        /* JOIN$(array(), delimiter$) */
        if args.len() != 2 {
            return Err(PbError::runtime("JOIN$: expected (array(), delimiter$)"));
        }
        let arr_ptr = self.compile_expr(fb, &args[0])?;
        let delim = self.compile_expr(fb, &args[1])?;
        let delim_ptr = self.convert_value(fb, &delim, &IrType::Ptr, &PbType::String);
        let arr_ptr = self.convert_value(fb, &arr_ptr, &IrType::Ptr, &PbType::String);
        Ok(fb.call(&IrType::Ptr, "pb_join", &[arr_ptr, delim_ptr]))
    }

    fn builtin_inputbox(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        /* INPUTBOX$(prompt$, [title$], [default$]) - console fallback */
        if args.is_empty() {
            return Err(PbError::runtime("INPUTBOX$: expected prompt$"));
        }
        let prompt = self.compile_expr(fb, &args[0])?;
        let prompt_ptr = self.convert_value(fb, &prompt, &IrType::Ptr, &PbType::String);
        /* Use console input: prompt, has_prompt=1, no_newline=0 */
        Ok(fb.call(
            &IrType::Ptr,
            "pb_input_console",
            &[prompt_ptr, fb.const_i32(1), fb.const_i32(0)],
        ))
    }

    fn builtin_msgbox(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        /* MSGBOX(txt$, [style&], [title$]) - function form, returns the
        button id (%IDOK/%IDYES/%IDNO/...) that the user pressed.  The
        statement form lives in compile_stmt; this is the expression
        form used by e.g. `n = MSGBOX("Go?", %MB_YESNO, "Ask")`. */
        if args.is_empty() {
            return Err(PbError::runtime("MSGBOX: expected txt$"));
        }
        let text = self.compile_expr(fb, &args[0])?;
        let style = if args.len() > 1 {
            match &args[1] {
                Expr::Omitted => fb.const_i32(0),
                _ => {
                    let sv = self.compile_expr(fb, &args[1])?;
                    self.to_i32(fb, &sv)
                }
            }
        } else {
            fb.const_i32(0)
        };
        let title = if args.len() > 2 {
            match &args[2] {
                Expr::Omitted => Val::new(self.msgbox_title_name.clone(), IrType::Ptr),
                _ => self.compile_expr(fb, &args[2])?,
            }
        } else {
            Val::new(self.msgbox_title_name.clone(), IrType::Ptr)
        };
        if self.module.is_32bit() {
            Ok(fb.call_stdcall(
                &IrType::I32,
                "pb_msgbox_impl",
                &[text.clone(), title.clone(), style.clone()],
            ))
        } else {
            Ok(fb.call(&IrType::I32, "pb_msgbox_impl", &[text, title, style]))
        }
    }

    fn builtin_threadid(&mut self, fb: &mut FunctionBuilder) -> PbResult<Val> {
        Ok(fb.call(&IrType::I32, "GetCurrentThreadId", &[]))
    }

    fn builtin_switch(
        &mut self,
        fb: &mut FunctionBuilder,
        args: &[Expr],
        _name: &str,
    ) -> PbResult<Val> {
        if args.is_empty() || !args.len().is_multiple_of(2) {
            return Err(PbError::runtime(
                "SWITCH: expected pairs of (condition, value)",
            ));
        }
        // String vs numeric variant is decided by the first value's IR type
        // (the lexer may or may not keep the trailing $ in the builtin name).
        let is_str = self.compile_expr(fb, &args[1])?.ty == IrType::Ptr;
        let mut conds = Vec::new();
        let mut vals = Vec::new();
        for pair in args.chunks(2) {
            let c = self.compile_expr(fb, &pair[0])?;
            let c1 = self.convert_value(fb, &c, &IrType::I64, &PbType::Long);
            let cond = fb.icmp("ne", &c1, &fb.const_i64(0));
            let v = self.compile_expr(fb, &pair[1])?;
            let vv = if is_str {
                self.convert_value(fb, &v, &IrType::Ptr, &PbType::String)
            } else {
                self.convert_value(fb, &v, &IrType::I64, &PbType::Long)
            };
            conds.push(cond);
            vals.push(vv);
        }
        let default = if is_str {
            fb.const_null_ptr()
        } else {
            fb.const_i64(0)
        };
        let mut acc = default;
        for (cond, val) in conds.iter().zip(vals.iter()).rev() {
            acc = fb.select(cond, val, &acc);
        }
        Ok(acc)
    }

    fn builtin_hilo(
        &mut self,
        fb: &mut FunctionBuilder,
        args: &[Expr],
        name: &str,
    ) -> PbResult<Val> {
        let dtype = match &args[0] {
            Expr::StringLit(d) => d.clone(),
            _ => return Err(PbError::runtime("HI/LO: expected a data type")),
        };
        let v = self.compile_expr(fb, &args[1])?;
        let vi = self.convert_value(fb, &v, &IrType::I64, &PbType::Quad);
        let bits: i64 = match dtype.as_str() {
            "BYTE" => 8,
            "WORD" | "INTEGER" => 16,
            _ => 32,
        };
        let mask = (1i64 << bits) - 1;
        let r = if name == "HI" {
            let shifted = fb.lshr(&vi, &fb.const_i64(bits));
            fb.and(&shifted, &fb.const_i64(mask))
        } else {
            fb.and(&vi, &fb.const_i64(mask))
        };
        Ok(r)
    }

    fn builtin_fileattr(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        if args.len() != 2 {
            return Err(PbError::runtime("FILEATTR requires 2 arguments"));
        }
        let a0 = self.compile_expr(fb, &args[0])?;
        let a1 = self.compile_expr(fb, &args[1])?;
        let f = self.convert_value(fb, &a0, &IrType::I32, &PbType::Long);
        let at = self.convert_value(fb, &a1, &IrType::I32, &PbType::Long);
        Ok(fb.call(&IrType::I64, "pb_fileattr", &[f, at]))
    }

    fn builtin_filename(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        if args.len() != 1 {
            return Err(PbError::runtime("FILENAME$ requires 1 argument"));
        }
        let a0 = self.compile_expr(fb, &args[0])?;
        let f = self.convert_value(fb, &a0, &IrType::I32, &PbType::Long);
        Ok(fb.call(&IrType::Ptr, "pb_filename", &[f]))
    }

    fn builtin_strn(
        &mut self,
        fb: &mut FunctionBuilder,
        args: &[Expr],
        fname: &str,
        min: usize,
    ) -> PbResult<Val> {
        if args.len() < min {
            return Err(PbError::runtime(format!(
                "{fname} requires at least {min} arguments"
            )));
        }
        let mut a = Vec::new();
        for e in args {
            let v = self.compile_expr(fb, e)?;
            let p = self.convert_value(fb, &v, &IrType::Ptr, &PbType::String);
            a.push(p);
        }
        Ok(fb.call(&IrType::Ptr, fname, &a))
    }

    fn builtin_str0(&mut self, fb: &mut FunctionBuilder, fname: &str) -> PbResult<Val> {
        Ok(fb.call(&IrType::Ptr, fname, &[]))
    }

    // Batch 89: ERROR$ — returns error message string.
    // ERROR$ (no args) -> message for current pb_err (pass -1 to runtime)
    // ERROR$(n) -> message for error code n
    fn builtin_error_str(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        let code = if args.is_empty() {
            fb.const_i32(-1) // -1 means "use current pb_err"
        } else {
            let v = self.compile_expr(fb, &args[0])?;
            self.to_i32(fb, &v)
        };
        Ok(fb.call(&IrType::Ptr, "pb_error_message", &[code]))
    }

    fn builtin_extract(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        if args.len() < 2 || args.len() > 4 {
            return Err(PbError::runtime(
                "EXTRACT$: expected ([start,] MainStr, [ANY] MatchStr)",
            ));
        }
        let mut any = false;
        let mut start_expr: Option<&Expr> = None;
        let mut str_args: Vec<&Expr> = Vec::new();
        for e in args {
            if let Expr::StringLit(s) = e {
                if s == "__ANY__" {
                    any = true;
                } else {
                    str_args.push(e);
                }
            } else if str_args.is_empty() && start_expr.is_none() {
                start_expr = Some(e);
            } else {
                str_args.push(e);
            }
        }
        if str_args.len() < 2 {
            return Err(PbError::runtime("EXTRACT$: expected MainStr and MatchStr"));
        }
        let start = if let Some(sx) = start_expr {
            let v = self.compile_expr(fb, sx)?;
            self.convert_value(fb, &v, &IrType::I64, &PbType::Long)
        } else {
            fb.const_i64(1)
        };
        let m = self.compile_expr(fb, str_args[0])?;
        let m2 = self.convert_value(fb, &m, &IrType::Ptr, &PbType::String);
        let mc = self.compile_expr(fb, str_args[1])?;
        let mc2 = self.convert_value(fb, &mc, &IrType::Ptr, &PbType::String);
        let anyv = if any {
            fb.const_i32(1)
        } else {
            fb.const_i32(0)
        };
        Ok(fb.call(&IrType::Ptr, "pb_extract", &[start, m2, mc2, anyv]))
    }

    fn builtin_rgb(
        &mut self,
        fb: &mut FunctionBuilder,
        args: &[Expr],
        name: &str,
    ) -> PbResult<Val> {
        if args.len() == 1 {
            let v = self.compile_expr(fb, &args[0])?;
            let vi = self.convert_value(fb, &v, &IrType::I64, &PbType::Long);
            Ok(fb.call(&IrType::I64, "pb_rgb_swap", &[vi]))
        } else if args.len() == 3 {
            let mut a = Vec::new();
            for e in args {
                let v = self.compile_expr(fb, e)?;
                let vi = self.convert_value(fb, &v, &IrType::I64, &PbType::Long);
                a.push(vi);
            }
            let fname = if name == "BGR" { "pb_bgr3" } else { "pb_rgb3" };
            Ok(fb.call(&IrType::I64, fname, &a))
        } else {
            Err(PbError::runtime("RGB/BGR: expected 1 or 3 arguments"))
        }
    }

    fn builtin_conv1(
        &mut self,
        fb: &mut FunctionBuilder,
        args: &[Expr],
        fname: &str,
    ) -> PbResult<Val> {
        if args.is_empty() {
            return Err(PbError::runtime("conversion function requires 1 argument"));
        }
        let s = self.compile_expr(fb, &args[0])?;
        Ok(fb.call(&IrType::Ptr, fname, std::slice::from_ref(&s)))
    }

    fn builtin_to_f64(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        let val = self.compile_expr(fb, &args[0])?;
        Ok(self.to_f64(fb, &val))
    }

    fn builtin_to_f32(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        let val = self.compile_expr(fb, &args[0])?;
        let f64_val = self.to_f64(fb, &val);
        Ok(fb.fptrunc(&f64_val, &IrType::Float))
    }

    fn builtin_rnd(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        if args.is_empty() {
            // RND() → random double 0.0 to <1.0
            let raw = fb.call(&IrType::I32, "rand", &[]);
            let raw_f64 = fb.sitofp(&raw, &IrType::Double);
            let max_val = fb.const_f64(32768.0);
            Ok(fb.fdiv(&raw_f64, &max_val))
        } else if args.len() == 1 {
            // RND(n) → random int 0 to n-1
            let n = self.compile_expr(fb, &args[0])?;
            let n_i32 = self.to_i32(fb, &n);
            let raw = fb.call(&IrType::I32, "rand", &[]);
            Ok(fb.srem(&raw, &n_i32))
        } else {
            // RND(lo, hi) → random int lo to hi inclusive
            let lo = self.compile_expr(fb, &args[0])?;
            let hi = self.compile_expr(fb, &args[1])?;
            let lo_i32 = self.to_i32(fb, &lo);
            let hi_i32 = self.to_i32(fb, &hi);
            let raw = fb.call(&IrType::I32, "rand", &[]);
            let range = fb.sub(&hi_i32, &lo_i32);
            let one = fb.const_i32(1);
            let range_plus = fb.add(&range, &one);
            let modded = fb.srem(&raw, &range_plus);
            Ok(fb.add(&lo_i32, &modded))
        }
    }

    fn builtin_round(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        let val = self.compile_expr(fb, &args[0])?;
        let f64_val = self.to_f64(fb, &val);
        if args.len() >= 2 {
            // ROUND(x, n) → round to n decimal places
            let n = self.compile_expr(fb, &args[1])?;
            let n_f64 = self.to_f64(fb, &n);
            let ten = fb.const_f64(10.0);
            let factor = fb.call(&IrType::Double, "llvm.pow.f64", &[ten, n_f64]);
            let scaled = fb.fmul(&f64_val, &factor);
            let rounded = fb.call(&IrType::Double, "llvm.round.f64", &[scaled]);
            Ok(fb.fdiv(&rounded, &factor))
        } else {
            // ROUND(x) → round to nearest integer
            Ok(fb.call(&IrType::Double, "llvm.round.f64", &[f64_val]))
        }
    }

    // ========== String builtins ==========

    fn builtin_len(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        let s = self.compile_expr(fb, &args[0])?;
        // FixedString variables are raw buffers (no BSTR prefix): keep strlen
        if let Expr::Variable(name) = &args[0] {
            let nn = normalize_name(name);
            if let Some(info) = self.symbols.lookup(&nn) {
                if matches!(info.pb_type, PbType::FixedString(_)) {
                    return Ok(fb.call(&IrType::I32, "strlen", &[s]));
                }
            }
        }
        // Constants carry a BSTR prefix; runtime strings are BSTRs: byte-length prefix
        Ok(fb.call(&IrType::I32, "pb_str_len", &[s]))
    }

    fn builtin_chr(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        if args.is_empty() {
            let tmp = fb.alloca(&IrType::I8);
            let zero = fb.const_i32(0);
            return Ok(fb.call(&IrType::Ptr, "pb_bstr_alloc", &[tmp, zero]));
        }
        // Multi-arg CHR$(a, b, c) concatenates one byte per argument
        let len = args.len() as i32;
        let len_val = fb.const_i32(len);
        let buf = fb.call(&IrType::Ptr, "malloc", std::slice::from_ref(&len_val));
        for (i, arg) in args.iter().enumerate() {
            let code = self.compile_expr(fb, arg)?;
            let code_i32 = self.to_i32(fb, &code);
            let byte = fb.trunc(&code_i32, &IrType::I8);
            let off = fb.const_i32(i as i32);
            let slot = fb.gep_byte(&buf, &off);
            fb.store(&byte, &slot);
        }
        Ok(fb.call(&IrType::Ptr, "pb_bstr_alloc", &[buf, len_val]))
    }

    fn builtin_asc(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        let s = self.compile_expr(fb, &args[0])?;
        let byte = fb.load(&IrType::I8, &s);
        Ok(fb.zext(&byte, &IrType::I32))
    }

    fn builtin_str(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        let val = self.compile_expr(fb, &args[0])?;
        let buf_size = fb.const_i32(32);
        let buf = fb.call(&IrType::Ptr, "malloc", std::slice::from_ref(&buf_size));
        if val.ty.is_int() {
            // PB STR$ semantics: full-precision decimal, leading space for
            // positive values (STR$(353) = " 353").  % d / % lld give exactly
            // that: space flag = leading blank for non-negative, '-' for negative.
            if val.ty == IrType::I64 {
                let (fmt_name, _) = self.module.add_string_constant("% lld");
                let fmt_ptr = Val::new(fmt_name, IrType::Ptr);
                fb.call_variadic_with_sig(
                    &IrType::I32,
                    "snprintf",
                    &[buf.clone(), buf_size, fmt_ptr, val.clone()],
                    &[IrType::Ptr, IrType::I32, IrType::Ptr, IrType::I64],
                );
            } else {
                let (fmt_name, _) = self.module.add_string_constant("% d");
                let fmt_ptr = Val::new(fmt_name, IrType::Ptr);
                let val_i32 = self.to_i32(fb, &val);
                fb.call_variadic_with_sig(
                    &IrType::I32,
                    "snprintf",
                    &[buf.clone(), buf_size, fmt_ptr, val_i32],
                    &[IrType::Ptr, IrType::I32, IrType::Ptr, IrType::I32],
                );
            }
        } else {
            let f64_val = self.to_f64(fb, &val);
            let (fmt_name, _) = self.module.add_string_constant("% g");
            let fmt_ptr = Val::new(fmt_name, IrType::Ptr);
            fb.call_variadic_with_sig(
                &IrType::I32,
                "snprintf",
                &[buf.clone(), buf_size, fmt_ptr, f64_val],
                &[IrType::Ptr, IrType::I32, IrType::Ptr],
            );
        }
        let len = fb.call(&IrType::I32, "strlen", std::slice::from_ref(&buf));
        let bstr = fb.call(&IrType::Ptr, "pb_bstr_alloc", &[buf.clone(), len]);
        fb.call_void("free", &[buf]);
        Ok(bstr)
    }

    /// HEX$(n [, digits]) — hexadecimal string, PB semantics (measured on PBWin10):
    /// negative LONG/Integer values are sign-extended and rendered as 64-bit hex
    /// (HEX$(-1) = "FFFFFFFFFFFFFFFF"), positive values as their natural width.
    /// QUAD values as unsigned 64-bit. Optional digits sets a minimum zero-padded width.
    fn builtin_hex(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        let val = self.compile_expr(fb, &args[0])?;
        // 32-bit (and smaller) types: sign-extend so negative LONGs format as
        // u64 hex (PBWin10 renders HEX$(-1) as 16 F's, not 8).
        let v64 = if val.ty == IrType::I32 || val.ty == IrType::I16 || val.ty == IrType::I8 {
            fb.sext(&val, &IrType::I64)
        } else {
            self.to_i64(fb, &val)
        };
        let buf_size = fb.const_i32(32);
        let buf = fb.call(&IrType::Ptr, "malloc", std::slice::from_ref(&buf_size));
        let (fmt_name, _) = self.module.add_string_constant("%0*llX");
        let fmt_ptr = Val::new(fmt_name, IrType::Ptr);
        let width = if args.len() >= 2 {
            self.compile_expr(fb, &args[1])?
        } else {
            fb.const_i32(0)
        };
        fb.call_variadic_with_sig(
            &IrType::I32,
            "snprintf",
            &[buf.clone(), buf_size, fmt_ptr, width, v64],
            &[
                IrType::Ptr,
                IrType::I32,
                IrType::Ptr,
                IrType::I32,
                IrType::I64,
            ],
        );
        let len = fb.call(&IrType::I32, "strlen", std::slice::from_ref(&buf));
        let bstr = fb.call(&IrType::Ptr, "pb_bstr_alloc", &[buf.clone(), len]);
        fb.call_void("free", &[buf]);
        Ok(bstr)
    }

    /// Convert a numeric value to a string (like STR$), for use in & concatenation
    fn num_to_string(&mut self, fb: &mut FunctionBuilder, val: &Val) -> Val {
        let buf_size = fb.const_i32(32);
        let buf = fb.call(&IrType::Ptr, "malloc", std::slice::from_ref(&buf_size));
        if val.ty.is_int() {
            // Integers keep exact formatting: % lld for QUAD, % d otherwise.
            // The space flag gives PB STR$ semantics (leading blank for >=0).
            if val.ty == IrType::I64 {
                let (fmt_name, _) = self.module.add_string_constant("% lld");
                let fmt_ptr = Val::new(fmt_name, IrType::Ptr);
                fb.call_variadic_with_sig(
                    &IrType::I32,
                    "snprintf",
                    &[buf.clone(), buf_size, fmt_ptr, val.clone()],
                    &[IrType::Ptr, IrType::I32, IrType::Ptr, IrType::I64],
                );
            } else {
                let (fmt_name, _) = self.module.add_string_constant("% d");
                let fmt_ptr = Val::new(fmt_name, IrType::Ptr);
                let val_i32 = self.to_i32(fb, val);
                fb.call_variadic_with_sig(
                    &IrType::I32,
                    "snprintf",
                    &[buf.clone(), buf_size, fmt_ptr, val_i32],
                    &[IrType::Ptr, IrType::I32, IrType::Ptr, IrType::I32],
                );
            }
        } else {
            let f64_val = self.to_f64(fb, val);
            let (fmt_name, _) = self.module.add_string_constant("% g");
            let fmt_ptr = Val::new(fmt_name, IrType::Ptr);
            fb.call_variadic_with_sig(
                &IrType::I32,
                "snprintf",
                &[buf.clone(), buf_size, fmt_ptr, f64_val],
                &[IrType::Ptr, IrType::I32, IrType::Ptr],
            );
        }
        let len = fb.call(&IrType::I32, "strlen", std::slice::from_ref(&buf));
        let bstr = fb.call(&IrType::Ptr, "pb_bstr_alloc", &[buf.clone(), len]);
        fb.call_void("free", &[buf]);
        bstr
    }

    fn builtin_val(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        let s = self.compile_expr(fb, &args[0])?;
        Ok(fb.call(&IrType::Double, "atof", &[s]))
    }

    fn builtin_left(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        let s = self.compile_expr(fb, &args[0])?;
        let n = self.compile_expr(fb, &args[1])?;
        let n_i32 = self.to_i32(fb, &n);
        // SysAllocStringByteLen copies n bytes from src and null-terminates
        Ok(fb.call(&IrType::Ptr, "pb_bstr_alloc", &[s, n_i32]))
    }

    fn builtin_right(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        let s = self.compile_expr(fb, &args[0])?;
        let n = self.compile_expr(fb, &args[1])?;
        let n_i32 = self.to_i32(fb, &n);
        let len = fb.call(&IrType::I32, "strlen", std::slice::from_ref(&s));
        let offset = fb.sub(&len, &n_i32);
        let src = fb.gep_byte(&s, &offset);
        Ok(fb.call(&IrType::Ptr, "pb_bstr_alloc", &[src, n_i32]))
    }

    /// UCODE$ / ACODE$ - the legacy ANSI <-> UNICODE byte-string pair.
    /// Both take (string [, codepage]); an omitted codepage is passed as -1 so the
    /// runtime falls back to the value recorded by UCODEPAGE (system ANSI by default).
    fn builtin_code_page(
        &mut self,
        fb: &mut FunctionBuilder,
        args: &[Expr],
        rt: &str,
        missing: &str,
    ) -> PbResult<Val> {
        if args.is_empty() {
            return Err(PbError::runtime(missing));
        }
        let s = self.compile_expr(fb, &args[0])?;
        let cp = if args.len() >= 2 {
            let cv = self.compile_expr(fb, &args[1])?;
            self.to_i32(fb, &cv)
        } else {
            fb.const_i32(-1)
        };
        Ok(fb.call(&IrType::Ptr, rt, &[s, cp]))
    }

    /// METRICS(metric) - Win32 GetSystemMetrics.  All dimensions are in pixels.
    /// Dotted metric names were already folded into integer literals by the parser.
    fn builtin_metrics(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        if args.is_empty() {
            return Err(PbError::runtime("METRICS requires 1 argument"));
        }
        let v = self.compile_expr(fb, &args[0])?;
        let idx = self.to_i32(fb, &v);
        Ok(fb.call(&IrType::I32, "GetSystemMetrics", &[idx]))
    }

    fn builtin_mid(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        // MID$(s, start) or MID$(s, start, length)
        // PB MID$ is 1-based
        let s = self.compile_expr(fb, &args[0])?;
        let start = self.compile_expr(fb, &args[1])?;
        let start_i32 = self.to_i32(fb, &start);
        let one = fb.const_i32(1);
        let offset = fb.sub(&start_i32, &one); // convert to 0-based

        let src = fb.gep_byte(&s, &offset);

        if args.len() >= 3 {
            // MID$(s, start, length)
            let length = self.compile_expr(fb, &args[2])?;
            let len_i32 = self.to_i32(fb, &length);
            Ok(fb.call(&IrType::Ptr, "pb_bstr_alloc", &[src, len_i32]))
        } else {
            // MID$(s, start) → rest of string
            let src_len = fb.call(&IrType::I32, "strlen", std::slice::from_ref(&src));
            Ok(fb.call(&IrType::Ptr, "pb_bstr_alloc", &[src, src_len]))
        }
    }

    fn builtin_instr(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        // INSTR(haystack, needle) or INSTR(start, haystack, needle)
        // Returns 1-based position, or 0 if not found
        let (base, search_start, needle) = if args.len() >= 3 {
            let start = self.compile_expr(fb, &args[0])?;
            let start_i32 = self.to_i32(fb, &start);
            let one = fb.const_i32(1);
            let offset = fb.sub(&start_i32, &one); // 1-based to 0-based
            let h = self.compile_expr(fb, &args[1])?;
            let n = self.compile_expr(fb, &args[2])?;
            let h_offset = fb.gep_byte(&h, &offset);
            (h.clone(), h_offset, n)
        } else {
            let h = self.compile_expr(fb, &args[0])?;
            let n = self.compile_expr(fb, &args[1])?;
            (h.clone(), h, n)
        };

        let result_ptr = fb.alloca(&IrType::I32);
        let zero_i32 = fb.const_i32(0);
        fb.store(&zero_i32, &result_ptr);

        let found = fb.call(&IrType::Ptr, "strstr", &[search_start, needle]);
        let null_ptr = fb.const_null_ptr();
        let is_null = fb.icmp("eq", &found, &null_ptr);

        let then_label = fb.next_label("instr.found");
        let merge_label = fb.next_label("instr.merge");

        fb.condbr(&is_null, &merge_label, &then_label);

        fb.label(&then_label);
        let found_int = fb.ptrtoint(&found);
        let base_int = fb.ptrtoint(&base);
        let diff = fb.sub(&found_int, &base_int);
        let one_i32 = fb.const_i32(1);
        let pos = fb.add(&diff, &one_i32); // 1-based relative to original string
        fb.store(&pos, &result_ptr);
        fb.br(&merge_label);

        fb.label(&merge_label);
        Ok(fb.load(&IrType::I32, &result_ptr))
    }

    fn builtin_ucase(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        let s = self.compile_expr(fb, &args[0])?;
        let len = fb.call(&IrType::I32, "strlen", std::slice::from_ref(&s));
        let one = fb.const_i32(1);
        let buf_size = fb.add(&len, &one);
        let buf = fb.call(&IrType::Ptr, "malloc", &[buf_size]);

        // Loop: for i = 0 to len-1, buf[i] = toupper(s[i])
        let cond_label = fb.next_label("ucase.cond");
        let body_label = fb.next_label("ucase.body");
        let exit_label = fb.next_label("ucase.exit");

        let i_ptr = fb.alloca(&IrType::I32);
        let zero_i32_val = fb.const_i32(0);
        fb.store(&zero_i32_val, &i_ptr);
        fb.br(&cond_label);

        fb.label(&cond_label);
        let i = fb.load(&IrType::I32, &i_ptr);
        let cmp = fb.icmp("slt", &i, &len);
        fb.condbr(&cmp, &body_label, &exit_label);

        fb.label(&body_label);
        let src_ptr = fb.gep_byte(&s, &i);
        let byte = fb.load(&IrType::I8, &src_ptr);
        let byte_i32 = fb.zext(&byte, &IrType::I32);
        let upper = fb.call(&IrType::I32, "toupper", &[byte_i32]);
        let upper_i8 = fb.trunc(&upper, &IrType::I8);
        let dst_ptr = fb.gep_byte(&buf, &i);
        fb.store(&upper_i8, &dst_ptr);
        let next_i = fb.add(&i, &one);
        fb.store(&next_i, &i_ptr);
        fb.br(&cond_label);

        fb.label(&exit_label);
        // Null-terminate
        let i_final = fb.load(&IrType::I32, &i_ptr);
        let null_ptr = fb.gep_byte(&buf, &i_final);
        let zero_byte = fb.const_i8(0);
        fb.store(&zero_byte, &null_ptr);
        Ok(self.bstr_from_buf(fb, &buf))
    }

    fn builtin_lcase(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        let s = self.compile_expr(fb, &args[0])?;
        let len = fb.call(&IrType::I32, "strlen", std::slice::from_ref(&s));
        let one = fb.const_i32(1);
        let buf_size = fb.add(&len, &one);
        let buf = fb.call(&IrType::Ptr, "malloc", &[buf_size]);

        let cond_label = fb.next_label("lcase.cond");
        let body_label = fb.next_label("lcase.body");
        let exit_label = fb.next_label("lcase.exit");

        let i_ptr = fb.alloca(&IrType::I32);
        let zero_i32_val = fb.const_i32(0);
        fb.store(&zero_i32_val, &i_ptr);
        fb.br(&cond_label);

        fb.label(&cond_label);
        let i = fb.load(&IrType::I32, &i_ptr);
        let cmp = fb.icmp("slt", &i, &len);
        fb.condbr(&cmp, &body_label, &exit_label);

        fb.label(&body_label);
        let src_ptr = fb.gep_byte(&s, &i);
        let byte = fb.load(&IrType::I8, &src_ptr);
        let byte_i32 = fb.zext(&byte, &IrType::I32);
        let lower = fb.call(&IrType::I32, "tolower", &[byte_i32]);
        let lower_i8 = fb.trunc(&lower, &IrType::I8);
        let dst_ptr = fb.gep_byte(&buf, &i);
        fb.store(&lower_i8, &dst_ptr);
        let next_i = fb.add(&i, &one);
        fb.store(&next_i, &i_ptr);
        fb.br(&cond_label);

        fb.label(&exit_label);
        let i_final = fb.load(&IrType::I32, &i_ptr);
        let null_ptr = fb.gep_byte(&buf, &i_final);
        let zero_byte = fb.const_i8(0);
        fb.store(&zero_byte, &null_ptr);
        Ok(self.bstr_from_buf(fb, &buf))
    }

    fn builtin_ltrim(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        // Skip leading spaces, return new string
        let s = self.compile_expr(fb, &args[0])?;

        let cond_label = fb.next_label("ltrim.cond");
        let body_label = fb.next_label("ltrim.body");
        let exit_label = fb.next_label("ltrim.exit");

        let i_ptr = fb.alloca(&IrType::I32);
        let zero_i32_val = fb.const_i32(0);
        fb.store(&zero_i32_val, &i_ptr);
        fb.br(&cond_label);

        fb.label(&cond_label);
        let i = fb.load(&IrType::I32, &i_ptr);
        let cur_ptr = fb.gep_byte(&s, &i);
        let byte = fb.load(&IrType::I8, &cur_ptr);
        let space = fb.const_i8(32); // space character
        let is_space = fb.icmp("eq", &byte, &space);
        let not_null = fb.icmp("ne", &byte, &fb.const_i8(0));
        let cont = fb.and(&is_space, &not_null);
        fb.condbr(&cont, &body_label, &exit_label);

        fb.label(&body_label);
        let one = fb.const_i32(1);
        let next = fb.add(&i, &one);
        fb.store(&next, &i_ptr);
        fb.br(&cond_label);

        fb.label(&exit_label);
        let i_final = fb.load(&IrType::I32, &i_ptr);
        let start = fb.gep_byte(&s, &i_final);
        let rest_len = fb.call(&IrType::I32, "strlen", std::slice::from_ref(&start));
        let one = fb.const_i32(1);
        let buf_size = fb.add(&rest_len, &one);
        let buf = fb.call(&IrType::Ptr, "malloc", &[buf_size]);
        fb.call(&IrType::Ptr, "strcpy", &[buf.clone(), start]);
        Ok(self.bstr_from_buf(fb, &buf))
    }

    fn builtin_rtrim(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        // Copy string, then walk backwards removing trailing spaces
        let s = self.compile_expr(fb, &args[0])?;
        let len = fb.call(&IrType::I32, "strlen", std::slice::from_ref(&s));
        let one = fb.const_i32(1);
        let buf_size = fb.add(&len, &one);
        let buf = fb.call(&IrType::Ptr, "malloc", &[buf_size]);
        fb.call(&IrType::Ptr, "strcpy", &[buf.clone(), s]);

        // Walk backwards: i = len - 1, while i >= 0 and buf[i] == ' ', i--
        let cond_label = fb.next_label("rtrim.cond");
        let body_label = fb.next_label("rtrim.body");
        let exit_label = fb.next_label("rtrim.exit");

        let i_ptr = fb.alloca(&IrType::I32);
        let len_minus1 = fb.sub(&len, &one);
        fb.store(&len_minus1, &i_ptr);
        fb.br(&cond_label);

        fb.label(&cond_label);
        let i = fb.load(&IrType::I32, &i_ptr);
        let zero_i32_val = fb.const_i32(0);
        let ge_zero = fb.icmp("sge", &i, &zero_i32_val);
        let cur_ptr = fb.gep_byte(&buf, &i);
        let byte = fb.load(&IrType::I8, &cur_ptr);
        let space = fb.const_i8(32);
        let is_space = fb.icmp("eq", &byte, &space);
        let cont = fb.and(&ge_zero, &is_space);
        fb.condbr(&cont, &body_label, &exit_label);

        fb.label(&body_label);
        let next = fb.sub(&i, &one);
        fb.store(&next, &i_ptr);
        fb.br(&cond_label);

        fb.label(&exit_label);
        // Null-terminate at i+1
        let i_final = fb.load(&IrType::I32, &i_ptr);
        let term_pos = fb.add(&i_final, &one);
        let term_ptr = fb.gep_byte(&buf, &term_pos);
        let zero_byte = fb.const_i8(0);
        fb.store(&zero_byte, &term_ptr);
        Ok(self.bstr_from_buf(fb, &buf))
    }

    fn builtin_trim(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        // TRIM$ = LTRIM$(RTRIM$(s))
        // First RTRIM, then LTRIM
        let s = self.compile_expr(fb, &args[0])?;

        // RTRIM part: copy and truncate trailing spaces
        let len = fb.call(&IrType::I32, "strlen", std::slice::from_ref(&s));
        let one = fb.const_i32(1);
        let buf_size = fb.add(&len, &one);
        let buf = fb.call(&IrType::Ptr, "malloc", &[buf_size]);
        fb.call(&IrType::Ptr, "strcpy", &[buf.clone(), s]);

        let cond_label = fb.next_label("trim.rcond");
        let body_label = fb.next_label("trim.rbody");
        let exit_label = fb.next_label("trim.rexit");

        let i_ptr = fb.alloca(&IrType::I32);
        let len_minus1 = fb.sub(&len, &one);
        fb.store(&len_minus1, &i_ptr);
        fb.br(&cond_label);

        fb.label(&cond_label);
        let i = fb.load(&IrType::I32, &i_ptr);
        let zero_i32_val = fb.const_i32(0);
        let ge_zero = fb.icmp("sge", &i, &zero_i32_val);
        let cur_ptr = fb.gep_byte(&buf, &i);
        let byte = fb.load(&IrType::I8, &cur_ptr);
        let space = fb.const_i8(32);
        let is_space = fb.icmp("eq", &byte, &space);
        let cont = fb.and(&ge_zero, &is_space);
        fb.condbr(&cont, &body_label, &exit_label);

        fb.label(&body_label);
        let next = fb.sub(&i, &one);
        fb.store(&next, &i_ptr);
        fb.br(&cond_label);

        fb.label(&exit_label);
        let i_final = fb.load(&IrType::I32, &i_ptr);
        let term_pos = fb.add(&i_final, &one);
        let term_ptr = fb.gep_byte(&buf, &term_pos);
        let zero_byte = fb.const_i8(0);
        fb.store(&zero_byte, &term_ptr);

        // LTRIM part: skip leading spaces of buf
        let cond2 = fb.next_label("trim.lcond");
        let body2 = fb.next_label("trim.lbody");
        let exit2 = fb.next_label("trim.lexit");

        let j_ptr = fb.alloca(&IrType::I32);
        fb.store(&zero_i32_val, &j_ptr);
        fb.br(&cond2);

        fb.label(&cond2);
        let j = fb.load(&IrType::I32, &j_ptr);
        let cur_ptr2 = fb.gep_byte(&buf, &j);
        let byte2 = fb.load(&IrType::I8, &cur_ptr2);
        let is_space2 = fb.icmp("eq", &byte2, &space);
        let not_null = fb.icmp("ne", &byte2, &fb.const_i8(0));
        let cont2 = fb.and(&is_space2, &not_null);
        fb.condbr(&cont2, &body2, &exit2);

        fb.label(&body2);
        let next_j = fb.add(&j, &one);
        fb.store(&next_j, &j_ptr);
        fb.br(&cond2);

        fb.label(&exit2);
        let j_final = fb.load(&IrType::I32, &j_ptr);
        let start = fb.gep_byte(&buf, &j_final);
        let rest_len = fb.call(&IrType::I32, "strlen", std::slice::from_ref(&start));
        let result_size = fb.add(&rest_len, &one);
        let result = fb.call(&IrType::Ptr, "malloc", &[result_size]);
        fb.call(&IrType::Ptr, "strcpy", &[result.clone(), start]);
        fb.call_void("free", &[buf]);
        Ok(self.bstr_from_buf(fb, &result))
    }

    fn builtin_space(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        let n = self.compile_expr(fb, &args[0])?;
        let n_i32 = self.to_i32(fb, &n);
        let one = fb.const_i32(1);
        let buf_size = fb.add(&n_i32, &one);
        let buf = fb.call(&IrType::Ptr, "malloc", &[buf_size]);

        // Loop: fill with spaces
        let cond_label = fb.next_label("space.cond");
        let body_label = fb.next_label("space.body");
        let exit_label = fb.next_label("space.exit");

        let i_ptr = fb.alloca(&IrType::I32);
        let zero_i32_val = fb.const_i32(0);
        fb.store(&zero_i32_val, &i_ptr);
        fb.br(&cond_label);

        fb.label(&cond_label);
        let i = fb.load(&IrType::I32, &i_ptr);
        let cmp = fb.icmp("slt", &i, &n_i32);
        fb.condbr(&cmp, &body_label, &exit_label);

        fb.label(&body_label);
        let dst = fb.gep_byte(&buf, &i);
        let space = fb.const_i8(32);
        fb.store(&space, &dst);
        let next = fb.add(&i, &one);
        fb.store(&next, &i_ptr);
        fb.br(&cond_label);

        fb.label(&exit_label);
        // Null-terminate
        let null_ptr = fb.gep_byte(&buf, &n_i32);
        let zero_byte = fb.const_i8(0);
        fb.store(&zero_byte, &null_ptr);
        Ok(self.bstr_from_buf(fb, &buf))
    }

    fn builtin_string_fn(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        // STRING$(count, char_code) or STRING$(count, string$)
        let n = self.compile_expr(fb, &args[0])?;
        let n_i32 = self.to_i32(fb, &n);
        let char_val = self.compile_expr(fb, &args[1])?;

        // Get the character byte
        let char_byte = if char_val.ty == IrType::Ptr {
            // STRING$(n, "X") → use first char
            fb.load(&IrType::I8, &char_val)
        } else {
            // STRING$(n, code) → use as char code
            let code_i32 = self.to_i32(fb, &char_val);
            fb.trunc(&code_i32, &IrType::I8)
        };

        let one = fb.const_i32(1);
        let buf_size = fb.add(&n_i32, &one);
        let buf = fb.call(&IrType::Ptr, "malloc", &[buf_size]);

        // Loop: fill with character
        let cond_label = fb.next_label("stringfn.cond");
        let body_label = fb.next_label("stringfn.body");
        let exit_label = fb.next_label("stringfn.exit");

        let i_ptr = fb.alloca(&IrType::I32);
        let zero_i32_val = fb.const_i32(0);
        fb.store(&zero_i32_val, &i_ptr);
        fb.br(&cond_label);

        fb.label(&cond_label);
        let i = fb.load(&IrType::I32, &i_ptr);
        let cmp = fb.icmp("slt", &i, &n_i32);
        fb.condbr(&cmp, &body_label, &exit_label);

        fb.label(&body_label);
        let dst = fb.gep_byte(&buf, &i);
        fb.store(&char_byte, &dst);
        let next = fb.add(&i, &one);
        fb.store(&next, &i_ptr);
        fb.br(&cond_label);

        fb.label(&exit_label);
        let null_ptr = fb.gep_byte(&buf, &n_i32);
        let zero_byte = fb.const_i8(0);
        fb.store(&zero_byte, &null_ptr);
        Ok(self.bstr_from_buf(fb, &buf))
    }

    // ========== PB Runtime builtins ==========

    fn builtin_format(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        // FORMAT$(number, format_string)
        let val = self.compile_expr(fb, &args[0])?;
        let f64_val = self.to_f64(fb, &val);
        let fmt = if args.len() >= 2 {
            self.compile_expr(fb, &args[1])?
        } else {
            Val::new(self.empty_string_name.clone(), IrType::Ptr)
        };
        Ok(fb.call(&IrType::Ptr, "pb_format", &[f64_val, fmt]))
    }

    fn builtin_parse(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        // PARSE$(string, delimiter, index) — 3-arg form (official)
        // PARSE$(string, index)          — 2-arg form: default delimiter = comma
        // PARSE$(string)                 — 1-arg form: default delimiter = comma
        // (PBWin10 measured: PARSECOUNT("one two three four") = 1, so the
        //  omitted delimiter is "," not " ".)
        let s = self.compile_expr(fb, &args[0])?;
        let (delim, index) = if args.len() >= 3 {
            (
                self.compile_expr(fb, &args[1])?,
                self.compile_expr(fb, &args[2])?,
            )
        } else if args.len() == 2 {
            let (n, _) = self.module.add_string_constant(",");
            (Val::new(n, IrType::Ptr), self.compile_expr(fb, &args[1])?)
        } else {
            let (n, _) = self.module.add_string_constant(",");
            (Val::new(n, IrType::Ptr), fb.const_i32(0))
        };
        let index_i32 = self.to_i32(fb, &index);
        Ok(fb.call(&IrType::Ptr, "pb_parse", &[s, delim, index_i32]))
    }

    fn builtin_parsecount(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        // PARSECOUNT(string)         — default delimiter = comma
        // PARSECOUNT(string, delim)  — explicit delimiter
        if args.is_empty() {
            return Ok(fb.const_i32(0));
        }
        let s = self.compile_expr(fb, &args[0])?;
        let delim = if args.len() >= 2 {
            self.compile_expr(fb, &args[1])?
        } else {
            let (n, _) = self.module.add_string_constant(",");
            Val::new(n, IrType::Ptr)
        };
        Ok(fb.call(&IrType::I32, "pb_parsecount", &[s, delim]))
    }

    fn builtin_remove(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        if args.len() < 2 {
            return Err(PbError::runtime("REMOVE$ requires at least 2 arguments"));
        }
        let s = self.compile_expr(fb, &args[0])?;
        // Check for ANY keyword: REMOVE$(main, ANY, matchchars)
        let any_flag = match &args[1] {
            Expr::Variable(ref name) => name.to_uppercase() == "ANY",
            Expr::StringLit(s) => s == "__ANY__",
            _ => false,
        };
        let match_idx = if any_flag { 2 } else { 1 };
        if args.len() <= match_idx {
            return Err(PbError::runtime("REMOVE$ requires a match string"));
        }
        let m = self.compile_expr(fb, &args[match_idx])?;
        let flag = fb.const_i32(if any_flag { 1 } else { 0 });
        Ok(fb.call(&IrType::Ptr, "pb_remove_string", &[s, m, flag]))
    }

    fn builtin_retain(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        if args.len() < 2 {
            return Err(PbError::runtime("RETAIN$ requires at least 2 arguments"));
        }
        let s = self.compile_expr(fb, &args[0])?;
        let any_flag = match &args[1] {
            Expr::Variable(ref name) => name.to_uppercase() == "ANY",
            Expr::StringLit(s) => s == "__ANY__",
            _ => false,
        };
        let match_idx = if any_flag { 2 } else { 1 };
        if args.len() <= match_idx {
            return Err(PbError::runtime("RETAIN$ requires a match string"));
        }
        let m = self.compile_expr(fb, &args[match_idx])?;
        let flag = fb.const_i32(if any_flag { 1 } else { 0 });
        Ok(fb.call(&IrType::Ptr, "pb_retain_string", &[s, m, flag]))
    }

    fn builtin_remain(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        if args.len() < 2 {
            return Err(PbError::runtime("REMAIN$ requires at least 2 arguments"));
        }
        // Determine if first arg is Start (integer literal) or MainStr
        let (start_idx, main_idx): (Option<usize>, usize) = if let Expr::IntegerLit(_) = args[0] {
            (Some(0), 1)
        } else {
            (None, 0)
        };
        let main = self.compile_expr(fb, &args[main_idx])?;
        let start_val = if let Some(si) = start_idx {
            self.compile_expr(fb, &args[si])?
        } else {
            fb.const_i64(1)
        };
        // Check for ANY after main
        let any_pos = main_idx + 1;
        let any_flag = if any_pos < args.len() {
            match &args[any_pos] {
                Expr::Variable(ref name) => name.to_uppercase() == "ANY",
                Expr::StringLit(s) => s == "__ANY__",
                _ => false,
            }
        } else {
            false
        };
        let match_idx = if any_flag { any_pos + 1 } else { any_pos };
        if args.len() <= match_idx {
            return Err(PbError::runtime("REMAIN$ requires a match string"));
        }
        let m = self.compile_expr(fb, &args[match_idx])?;
        let flag = fb.const_i32(if any_flag { 1 } else { 0 });
        Ok(fb.call(
            &IrType::Ptr,
            "pb_remain_string",
            &[main, m, start_val, flag],
        ))
    }

    fn builtin_using(&mut self, fb: &mut FunctionBuilder, args: &[Expr]) -> PbResult<Val> {
        let fmt = self.compile_expr(fb, &args[0])?;
        let val = self.compile_expr(fb, &args[1])?;
        let f64_val = self.to_f64(fb, &val);
        Ok(fb.call(&IrType::Ptr, "pb_using", &[fmt, f64_val]))
    }

    // ========== Array helpers ==========

    /// Evaluate a constant expression at compile time (for array bounds).
    fn eval_const_expr(expr: &Expr) -> Result<i64, &'static str> {
        match expr {
            Expr::IntegerLit(n) => Ok(*n),
            Expr::FloatLit(f) => Ok(*f as i64),
            Expr::Negate(inner) | Expr::UnaryOp(UnaryOp::Neg, inner) => {
                Ok(-Self::eval_const_expr(inner)?)
            }
            Expr::BinaryOp(op, left, right) => {
                let l = Self::eval_const_expr(left)?;
                let r = Self::eval_const_expr(right)?;
                match op {
                    BinaryOp::Add => Ok(l + r),
                    BinaryOp::Sub => Ok(l - r),
                    BinaryOp::Mul => Ok(l * r),
                    _ => Err("unsupported op in const expr"),
                }
            }
            _ => Err("non-constant expression in array bounds"),
        }
    }

    /// Convert any value to i64 (for GEP indices).
    #[allow(dead_code)]
    fn to_i64(&self, fb: &mut FunctionBuilder, val: &Val) -> Val {
        match &val.ty {
            IrType::I64 => val.clone(),
            IrType::I1 => fb.zext(val, &IrType::I64),
            IrType::I16 | IrType::I32 => fb.sext(val, &IrType::I64),
            IrType::Double | IrType::Float => fb.fptosi(val, &IrType::I64),
            _ => val.clone(),
        }
    }

    /// Compute flat index from multi-dimensional indices.
    /// Strides are computed at compile time from dimension counts.
    fn compute_flat_index(
        &self,
        fb: &mut FunctionBuilder,
        indices: &[Val],
        arr_info: &ArrayInfo,
    ) -> Val {
        let ndims = arr_info.dims.len();

        // Compute strides: stride[last] = 1, stride[i] = stride[i+1] * count[i+1]
        let mut strides = vec![1i64; ndims];
        for i in (0..ndims.saturating_sub(1)).rev() {
            strides[i] = strides[i + 1] * arr_info.dims[i + 1].1;
        }

        let mut result = fb.const_i32(0);
        for (i, idx) in indices.iter().enumerate() {
            if i >= ndims {
                break;
            }
            let idx_i32 = self.to_i32(fb, idx);
            let lower = arr_info.dims[i].0;
            let adjusted = if lower != 0 {
                let lower_val = fb.const_i32(lower as i32);
                fb.sub(&idx_i32, &lower_val)
            } else {
                idx_i32
            };
            if strides[i] != 1 {
                let stride_val = fb.const_i32(strides[i] as i32);
                let contribution = fb.mul(&adjusted, &stride_val);
                result = fb.add(&result, &contribution);
            } else {
                result = fb.add(&result, &adjusted);
            }
        }
        result
    }

    /// Declare a global array and register it in the symbol table.
    fn declare_global_array(&mut self, name: &str, pb_type: &PbType, bounds: &[DimBound]) {
        let norm_name = normalize_name(name);
        // Already declared → skip
        if self.symbols.lookup_array(&norm_name).is_some() {
            return;
        }
        let elem_ir = Self::ir_type_for(pb_type);
        let mut dims = Vec::new();
        let mut total = 1usize;
        for bound in bounds {
            let lower = Self::eval_const_expr(&bound.lower).unwrap_or(0);
            let upper = Self::eval_const_expr(&bound.upper).unwrap_or(0);
            let count = (upper - lower + 1) as usize;
            dims.push((lower, count as i64));
            total *= count;
        }
        let array_ir = IrType::Array(total, Box::new(elem_ir.clone()));

        if self.session_mode {
            // Defer: collect into session_fields for struct emission later
            if !self.session_fields.iter().any(|f| f.name == norm_name) {
                self.session_fields.push(SessionFieldInfo {
                    name: norm_name.clone(),
                    ir_type: array_ir,
                    pb_type: pb_type.clone(),
                    is_array: true,
                    elem_ir_type: Some(elem_ir),
                    array_dims: Some(dims),
                    total_elements: Some(total),
                });
            }
            self.pending_global_arrays.remove(&norm_name);
        } else {
            self.module
                .add_global(&norm_name, &array_ir, "zeroinitializer");
            self.symbols.insert_global_array(
                norm_name.clone(),
                ArrayInfo {
                    ptr_name: format!("@{}", norm_name),
                    array_ir_type: array_ir,
                    elem_ir_type: elem_ir,
                    pb_type: pb_type.clone(),
                    dims,
                    total_elements: total,
                },
            );
            self.pending_global_arrays.remove(&norm_name);
        }
    }

    // ========== Session Struct ==========

    /// Scan a function/sub body for DIM statements that resolve pending global arrays,
    /// and for GLOBAL scalar declarations (parsed as DimScope::Global).
    /// This handles the common PB pattern where GLOBAL arr() is in globals.inc but DIM arr(N)
    /// is inside a SetupDims() function body, as well as GLOBAL scalars declared inside SUBs.
    fn scan_body_for_global_dims(&mut self, stmts: &[Statement]) {
        for stmt in stmts {
            match stmt {
                Statement::Dim(dim) => {
                    let name = normalize_name(&dim.name);
                    if !dim.bounds.is_empty() && self.pending_global_arrays.contains_key(&name) {
                        let pb_type = self.pending_global_arrays.get(&name).unwrap().clone();
                        // Compute array layout and push to session_fields
                        self.declare_global_array(&name, &pb_type, &dim.bounds);
                    } else if dim.scope == DimScope::Global
                        && dim.bounds.is_empty()
                        && self.symbols.lookup(&name).is_none()
                    {
                        // GLOBAL scalar declared inside a SUB body (e.g., GLOBAL ForceSync AS LONG in SetupDims)
                        let vd = VarDecl {
                            name: dim.name.clone(),
                            pb_type: dim.pb_type.clone(),
                            is_array: false,
                            line: dim.line,
                        };
                        self.declare_global(&vd);
                    }
                }
                Statement::If(if_stmt) => {
                    self.scan_body_for_global_dims(&if_stmt.then_body);
                    for clause in &if_stmt.elseif_clauses {
                        self.scan_body_for_global_dims(&clause.body);
                    }
                    if !if_stmt.else_body.is_empty() {
                        self.scan_body_for_global_dims(&if_stmt.else_body);
                    }
                }
                Statement::For(for_stmt) => self.scan_body_for_global_dims(&for_stmt.body),
                Statement::DoLoop(do_stmt) => self.scan_body_for_global_dims(&do_stmt.body),
                Statement::WhileWend(w) => self.scan_body_for_global_dims(&w.body),
                Statement::SelectCase(sel) => {
                    for case in &sel.cases {
                        self.scan_body_for_global_dims(&case.body);
                    }
                    if !sel.else_body.is_empty() {
                        self.scan_body_for_global_dims(&sel.else_body);
                    }
                }
                _ => {}
            }
        }
    }

    /// Emit the PBSessionData struct type and global instance from collected session_fields.
    /// Registers each field in the symbol table with a constant GEP expression as its pointer.
    fn emit_session_struct(&mut self) {
        let struct_name = "PBSESSIONDATA";

        // Collect field types
        let field_types: Vec<IrType> = self
            .session_fields
            .iter()
            .map(|f| f.ir_type.clone())
            .collect();

        // Define the struct type: %PBSESSIONDATA = type { ... }
        self.module.define_struct_type(struct_name, &field_types);

        // Emit the global instance with dllexport
        self.module.add_global_exported(
            "pb.session",
            &IrType::Struct(struct_name.to_string()),
            "zeroinitializer",
        );

        // Register each field in the symbol table
        for (idx, field) in self.session_fields.iter().enumerate() {
            let gep_expr = format!(
                "getelementptr inbounds (%{}, ptr @pb.session, i32 0, i32 {})",
                struct_name, idx
            );

            if field.is_array {
                let dims = field.array_dims.clone().unwrap();
                let total = field.total_elements.unwrap();
                let elem_ir = field.elem_ir_type.clone().unwrap();
                self.symbols.insert_global_array(
                    field.name.clone(),
                    ArrayInfo {
                        ptr_name: gep_expr,
                        array_ir_type: field.ir_type.clone(),
                        elem_ir_type: elem_ir,
                        pb_type: field.pb_type.clone(),
                        dims,
                        total_elements: total,
                    },
                );
            } else {
                self.symbols.insert_global_with_ptr(
                    field.name.clone(),
                    gep_expr,
                    field.ir_type.clone(),
                    field.pb_type.clone(),
                );
            }
        }

        eprintln!(
            "[PB64Basic] Session struct: {} fields in %{}",
            self.session_fields.len(),
            struct_name
        );
    }

    /// Emit the GetSession() export function returning a pointer to the session struct.
    fn emit_get_session(&mut self) {
        let mut fb = self
            .module
            .create_function_builder_ex("GetSession", &IrType::Ptr, &[], true);
        fb.ret(&Val::new("@pb.session", IrType::Ptr));
        self.module.add_function_body(fb.finish());
    }

    // ========== Utility ==========

    /// Scan a function/sub body for DIM statements with bounds (array declarations).
    fn scan_body_for_array_names(body: &[Statement], out: &mut HashSet<String>) {
        for stmt in body {
            match stmt {
                Statement::Dim(dim) | Statement::Redim(dim) => {
                    if !dim.bounds.is_empty() {
                        out.insert(normalize_name(&dim.name));
                    }
                }
                Statement::Block(stmts) => Self::scan_body_for_array_names(stmts, out),
                _ => {}
            }
        }
    }

    fn ensure_variable_ptr(
        &mut self,
        fb: &mut FunctionBuilder,
        name: &str,
        original_name: &str,
    ) -> PbResult<String> {
        if let Some(info) = self.symbols.lookup(name) {
            return Ok(info.ptr_name.clone());
        }
        // #DIM ALL / OPTION EXPLICIT: every identifier must be declared first
        // -> PBWin10 Error 519 "Missing declaration".
        if self.option_explicit {
            return Err(PbError::parser(
                format!(
                    "Missing declaration: variable `{}` must be declared before use",
                    original_name
                ),
                None,
                self.current_line,
            ));
        }
        // Official PB: an undeclared variable needs a type source (name suffix
        // or DEFxxx). This fork has no DEFxxx support yet, so a bare name is
        // Error 516, never silently auto-declared.
        if !name_has_type_suffix(original_name) {
            return Err(PbError::parser(
                format!(
                    "DefType, Type id (?%&!#$), or AS... required: {}",
                    original_name.to_uppercase()
                ),
                None,
                self.current_line,
            ));
        }
        // Auto-allocate → use original_name (with type suffix) for type inference
        let pb_type = infer_type_from_name(original_name);
        let ir_type = Self::ir_type_for(&pb_type);
        let ptr = fb.alloca(&ir_type);
        if Self::is_string_pb(&pb_type) {
            fb.store(&Val::new(self.empty_string_name.clone(), IrType::Ptr), &ptr);
        } else {
            fb.store(&Val::new(ir_type.zero_literal(), ir_type.clone()), &ptr);
        }
        let ptr_name = ptr.name.clone();
        self.symbols
            .insert_local(name.to_string(), ptr_name.clone(), ir_type, pb_type);
        Ok(ptr_name)
    }
}

/// Normalize a PB name: uppercase, strip type suffix.
/// PB built-in string equates — the 18 ANSI forms from the official docs.
/// Returns the expanded string, or None if the name is not a built-in equate.
fn string_equate(name: &str) -> Option<String> {
    let upper = name.to_uppercase();
    let v = match upper.as_str() {
        "$NUL" => "\u{0}".to_string(),
        "$BEL" => "\u{7}".to_string(),
        "$BS" => "\u{8}".to_string(),
        "$TAB" => "\u{9}".to_string(),
        "$LF" => "\n".to_string(),
        "$VT" => "\x0B".to_string(),
        "$FF" => "\x0C".to_string(),
        "$CR" => "\r".to_string(),
        "$CRLF" => "\r\n".to_string(),
        "$EOF" => "\u{1A}".to_string(),
        "$ESC" => "\u{1B}".to_string(),
        "$SPC" => " ".to_string(),
        "$DQ" => "\"".to_string(),
        "$DQ2" => "\"\"".to_string(),
        "$SQ" => "'".to_string(),
        "$SQ2" => "''".to_string(),
        "$QCQ" => "\",\"".to_string(),
        "$WHITESPACE" => " \t\r\n".to_string(),
        _ => return None,
    };
    Some(v)
}

/// Wide ($$) equates that return a single Word value — usable as a numeric
/// constant in this compiler (i32). Multi-char wide forms need a wide-string
/// type and are rejected explicitly (see wide_equate_unsupported).
fn wide_equate(name: &str) -> Option<i32> {
    let upper = name.to_uppercase();
    let v = match upper.as_str() {
        "$$NUL" => 0,
        "$$BEL" => 7,
        "$$BS" => 8,
        "$$TAB" => 9,
        "$$LF" => 10,
        "$$VT" => 11,
        "$$FF" => 12,
        "$$CR" => 13,
        "$$EOF" => 26,
        "$$ESC" => 27,
        "$$SPC" => 32,
        "$$DQ" => 34,
        "$$SQ" => 39,
        _ => return None,
    };
    Some(v)
}

/// Wide multi-char equates that this compiler cannot represent yet
/// (they are wide strings). Recognised so we can fail explicitly instead of
/// silently treating them as ordinary variables.
fn wide_equate_unsupported(name: &str) -> bool {
    let upper = name.to_uppercase();
    matches!(
        upper.as_str(),
        "$$CRLF" | "$$DQ2" | "$$SQ2" | "$$QCQ" | "$$WHITESPACE"
    )
}

fn normalize_name(name: &str) -> String {
    let upper = name.to_uppercase();
    let bytes = upper.as_bytes();
    let len = bytes.len();
    if len == 0 {
        return upper;
    }
    if len >= 2 {
        let last2 = &upper[len - 2..];
        if matches!(last2, "##" | "&&" | "%%" | "@@") {
            return upper[..len - 2].to_string();
        }
    }
    let last = bytes[len - 1];
    if matches!(last, b'#' | b'&' | b'%' | b'!' | b'@' | b'$') {
        return upper[..len - 1].to_string();
    }
    upper
}

/// Strip an inline-ASM comment: everything after ';' or a single quote
/// (outside double quotes) is dropped.
fn strip_asm_comment(text: &str) -> String {
    let mut out = String::new();
    let mut in_str = false;
    for ch in text.chars() {
        if ch == '"' {
            in_str = !in_str;
            out.push(ch);
            continue;
        }
        if !in_str && (ch == ';' || ch == '\'') {
            break;
        }
        out.push(ch);
    }
    out
}

/// Split an inline-ASM line into tokens. Whitespace and commas separate
/// tokens; a bracketed memory operand `[...]` stays one token; double-quoted
/// strings stay one token.
fn split_asm_tokens(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut in_bracket = false;
    let mut in_str = false;
    for ch in text.chars() {
        match ch {
            '"' => {
                in_str = !in_str;
                cur.push(ch);
            }
            '[' if !in_str => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
                in_bracket = true;
                cur.push(ch);
            }
            ']' if in_bracket => {
                in_bracket = false;
                cur.push(ch);
            }
            ',' if !in_bracket && !in_str => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
                out.push(",".to_string());
            }
            c if c.is_whitespace() && !in_bracket && !in_str => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
            }
            c => {
                cur.push(c);
            }
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// True if the name carries an explicit PB type suffix ($ % & ! # @ ## && %%).
///
/// Official PowerBASIC accepts an undeclared variable ONLY when a type source
/// exists: a suffix or a DEFxxx statement. This fork has no DEFxxx support yet,
/// so a bare name is an error (Error 516), never silently auto-declared.
fn name_has_type_suffix(name: &str) -> bool {
    name.ends_with("##")
        || name.ends_with("&&")
        || name.ends_with("%%")
        || name.ends_with('#')
        || name.ends_with('&')
        || name.ends_with('%')
        || name.ends_with('!')
        || name.ends_with('@')
        || name.ends_with('$')
}

/// Infer PB type from variable name suffix.
fn infer_type_from_name(name: &str) -> PbType {
    if name.ends_with("##") {
        PbType::Ext
    } else if name.ends_with('#') {
        PbType::Double
    } else if name.ends_with("&&") {
        PbType::Quad
    } else if name.ends_with('&') {
        PbType::Long
    } else if name.ends_with("%%") {
        PbType::Word
    } else if name.ends_with('%') {
        PbType::Integer
    } else if name.ends_with('!') {
        PbType::Single
    } else if name.ends_with('@') {
        PbType::Cur
    } else if name.ends_with('$') {
        PbType::String
    } else {
        PbType::Long
    }
}
