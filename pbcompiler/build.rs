// =====================================================================
// Modified by the hanlokesu/PB64Basic fork (https://github.com/hanlokesu/PB64Basic).
// Derived from benstopics/powerbasilisk (https://github.com/benstopics/powerbasilisk), Apache-2.0.
// See LICENSE for the full license terms.
// =====================================================================
// Stamp the release version into the compiler binary.
//
// `VERSION` at the repository root is the single source of truth: every release
// bumps it (release.py refuses to publish when it does not match --version), and
// build.rs bakes the value into the binary so the banner can print it without
// reading any file at run time.  Reading a file at run time would break as soon
// as the EXE is copied away from the repository, which is exactly how the
// compiler is normally used.
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let version_file = manifest_dir
        .parent()
        .map(|p| p.join("VERSION"))
        .unwrap_or_else(|| manifest_dir.join("VERSION"));

    println!("cargo:rerun-if-changed={}", version_file.display());

    let version = std::fs::read_to_string(&version_file)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| format!("v{}", env!("CARGO_PKG_VERSION")));

    println!("cargo:rustc-env=PB64BASIC_VERSION={}", version);
}
