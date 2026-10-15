#!/bin/sh
# Local mirror of .github/workflows/ci.yml.
#
# Sourced by pre-commit and pre-push; running it directly performs the whole
# mirror. It is a separate file because both hooks must run exactly the same
# commands, and because git invokes hooks with a minimal PATH - see the
# toolchain discovery below.
#
#   pb_fmt_gate     formatting only (about a second)
#   pb_full_mirror  fmt, clippy, release build, both runtimes, both test suites

# --- toolchain discovery -------------------------------------------------
# git runs hooks with a PATH that normally excludes ~/.cargo/bin and the LLVM
# bin directory, so `cargo` is "command not found" inside a hook even though it
# works in the user's shell. Find the tools explicitly.
pb_find() {
  _name="$1"
  shift
  if command -v "$_name" >/dev/null 2>&1; then
    command -v "$_name"
    return 0
  fi
  for _d in "$@"; do
    for _e in "$_d/$_name" "$_d/$_name.exe"; do
      if [ -x "$_e" ]; then
        echo "$_e"
        return 0
      fi
    done
  done
  return 1
}

PB_CARGO=$(pb_find cargo "$HOME/.cargo/bin" "$USERPROFILE/.cargo/bin" \
           "/c/Users/$USERNAME/.cargo/bin" "/c/Program Files/Rust/bin") || true
PB_CLANG=$(pb_find clang "/c/Program Files/LLVM/bin" "/c/Program Files (x86)/LLVM/bin") || true

# --- the fast gate -------------------------------------------------------
pb_fmt_gate() {
  if [ -z "$PB_CARGO" ]; then
    echo "ci-mirror: cargo not found - skipping the formatting gate"
    echo "ci-mirror: (looked in ~/.cargo/bin, %USERPROFILE%/.cargo/bin, PATH)"
    return 0
  fi
  echo "ci-mirror: cargo fmt --all -- --check"
  "$PB_CARGO" fmt --all -- --check || {
    echo ""
    echo "Formatting check failed. Run 'cargo fmt --all' to fix, then re-commit."
    return 1
  }
  return 0
}

# --- the full mirror -----------------------------------------------------
pb_full_mirror() {
  if [ -z "$PB_CARGO" ]; then
    echo "ci-mirror: cargo not found - cannot run the mirror. Install Rust first."
    return 1
  fi
  if [ -z "$PB_CLANG" ]; then
    echo "ci-mirror: clang not found - cannot build the runtime."
    echo "ci-mirror: (looked in PATH and C:/Program Files/LLVM/bin)"
    return 1
  fi

  pb_fmt_gate || return 1

  echo "ci-mirror: cargo clippy --all-targets -- -D warnings"
  "$PB_CARGO" clippy --all-targets -- -D warnings || {
    echo "Clippy check failed. Fix the warnings above."
    return 1
  }

  echo "ci-mirror: cargo build --release"
  "$PB_CARGO" build --release || {
    echo "Build failed."
    return 1
  }

  COMPILER="target/release/PB64Basic"
  if [ -f "$COMPILER.exe" ]; then
    COMPILER="$COMPILER.exe"
  fi

  echo "ci-mirror: build runtime (32-bit)"
  "$PB_CLANG" -c --target=i686-pc-windows-msvc pbcompiler/runtime/pb_runtime.c \
    -o pbcompiler/runtime/pb_runtime.obj || {
    echo "32-bit runtime build failed."
    return 1
  }

  echo "ci-mirror: build runtime (64-bit)"
  "$PB_CLANG" -c --target=x86_64-pc-windows-msvc pbcompiler/runtime/pb_runtime.c \
    -o pbcompiler/runtime/pb_runtime_x64.obj || {
    echo "64-bit runtime build failed."
    return 1
  }

  pb_run_suite "" "pbcompiler/runtime/pb_runtime.obj" "" "32-bit" || return 1
  pb_run_suite "x86_64-pc-windows-msvc" "pbcompiler/runtime/pb_runtime_x64.obj" "_x64" "64-bit" || return 1

  echo "ci-mirror: all checks passed."
  return 0
}

pb_run_suite() {
  _target="$1"
  _lib="$2"
  _suffix="$3"
  _label="$4"
  _failed=0
  echo "ci-mirror: compiler tests ($_label)"
  for _f in pbcompiler/tests/l*.bas; do
    _base="${_f%.bas}$_suffix"
    echo "  $_f ($_label)"
    if [ -n "$_target" ]; then
      "$COMPILER" build "$_f" -o "$_base" --exe --target "$_target" --runtime-lib "$_lib" || {
        echo "  COMPILE FAILED: $_f"
        _failed=$((_failed + 1))
        continue
      }
    else
      "$COMPILER" build "$_f" -o "$_base" --exe --runtime-lib "$_lib" || {
        echo "  COMPILE FAILED: $_f"
        _failed=$((_failed + 1))
        continue
      }
    fi
    "$_base.exe" || {
      echo "  TEST FAILED: $_f ($_label)"
      _failed=$((_failed + 1))
    }
  done
  if [ "$_failed" -ne 0 ]; then
    echo "$_failed $_label test(s) failed."
    return 1
  fi
  echo "  All $_label tests passed."
  return 0
}

# --- run the mirror only when executed, not when sourced -----------------
if [ "$(basename "$0")" = "ci-mirror.sh" ]; then
  set -e
  pb_full_mirror
fi
