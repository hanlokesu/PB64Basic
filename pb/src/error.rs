// =====================================================================
// Modified by the hanlokesu/PB64Basic fork (https://github.com/hanlokesu/PB64Basic).
// Derived from benstopics/powerbasilisk (https://github.com/benstopics/powerbasilisk), Apache-2.0.
// See LICENSE for the full license terms.
// =====================================================================
use std::fmt;

#[derive(Debug, Clone)]
pub struct PbError {
    pub kind: ErrorKind,
    pub message: String,
    pub file: Option<String>,
    pub line: Option<usize>,
    pub col: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind {
    Lexer,
    Preprocessor,
    Parser,
    Runtime,
    Io,
}

impl PbError {
    pub fn lexer(msg: impl Into<String>, file: Option<&str>, line: usize, col: usize) -> Self {
        PbError {
            kind: ErrorKind::Lexer,
            message: msg.into(),
            file: file.map(|s| s.to_string()),
            line: Some(line),
            col: Some(col),
        }
    }

    pub fn preprocessor(msg: impl Into<String>, file: Option<&str>, line: usize) -> Self {
        PbError {
            kind: ErrorKind::Preprocessor,
            message: msg.into(),
            file: file.map(|s| s.to_string()),
            line: Some(line),
            col: None,
        }
    }

    pub fn parser(msg: impl Into<String>, file: Option<&str>, line: usize) -> Self {
        PbError {
            kind: ErrorKind::Parser,
            message: msg.into(),
            file: file.map(|s| s.to_string()),
            line: Some(line),
            col: None,
        }
    }

    pub fn parser_at(msg: impl Into<String>, file: Option<&str>, line: usize, col: usize) -> Self {
        PbError {
            kind: ErrorKind::Parser,
            message: msg.into(),
            file: file.map(|s| s.to_string()),
            line: Some(line),
            col: Some(col),
        }
    }

    pub fn runtime(msg: impl Into<String>) -> Self {
        PbError {
            kind: ErrorKind::Runtime,
            message: msg.into(),
            file: None,
            line: None,
            col: None,
        }
    }

    pub fn io(msg: impl Into<String>) -> Self {
        PbError {
            kind: ErrorKind::Io,
            message: msg.into(),
            file: None,
            line: None,
            col: None,
        }
    }
}

impl PbError {
    /// Map this error to the PBWin10 compile-time error code (the same number
    /// PBWin.exe returns as its process exit code, e.g. 477 for a syntax error).
    /// Keyword-first (high-confidence matches on the English message), then a
    /// per-kind default.  Only the PBWin error-number table (extracted from the
    /// official help, error_NNN_-_... pages) is used; codes that do not exist
    /// in PBWin are never invented here.
    pub fn pbwin_code(&self) -> i32 {
        let m = self.message.to_ascii_lowercase();
        // I/O family first: link/write/read failures are their own PBWin codes,
        // and must not be hijacked by keyword rules below (e.g. a linker message
        // containing "undefined symbol" is a LINK failure, not an equate error).
        if self.kind == ErrorKind::Io {
            if m.contains("link") {
                return 630; // Link File Error
            }
            if m.contains("write") {
                return 496; // Destination file write error
            }
            if m.contains("read") || m.contains("not found") || m.contains("accessible") {
                return 493; // Compiler file not found / accessible
            }
            return 51; // Internal error
        }
        // Unknown statement/subroutine reference -> Error 462 "Undefined procedure
        // reference" (matches PBWin10 for `CALL NoSuchProc`).  Must precede the
        // "not implemented" rule, since fork words these messages with that suffix.
        if m.contains("unknown statement") || m.contains("unknown subroutine") {
            return 462;
        }
        // Unsupported-feature family -> Error 551 "Not supported in this version"
        if m.contains("not supported")
            || m.contains("not implemented")
            || m.contains("unimplemented")
        {
            return 551;
        }
        // Unknown/undefined symbols -> PBWin family
        if m.contains("unknown array") {
            return 423; // Array variable expected
        }
        if m.contains("unknown type") || m.contains("undefined type") {
            return 524; // Undefined type
        }
        if m.contains("unknown field") || m.contains("invalid member") {
            return 530; // Invalid member name definition
        }
        if m.contains("undefined") {
            if m.contains("procedure") || m.contains("function") || m.contains("method") {
                return 462; // Undefined procedure reference
            }
            if m.contains("label") || m.contains("line") {
                return 463; // Undefined label/line reference
            }
            return 460; // Undefined equate
        }
        // Name conflicts -> Error 466 "This name is already in use"
        if m.contains("duplicate") || m.contains("already in use") || m.contains("already defined")
        {
            return 466;
        }
        // Type/shape mismatches -> Error 482 "Data type mismatch"
        if m.contains("type mismatch") || m.contains("mismatch") {
            return 482;
        }
        // Length/complexity limits
        if m.contains("too long") {
            if m.contains("line") {
                return 407; // Source line too long
            }
            if m.contains("expression") || m.contains("complex") {
                return 401; // Expression too long / complex
            }
            return 402; // Statement too long / complex
        }
        // lvalue assignment to a constant/expression -> Error 542 "May not be altered"
        if m.contains("not an lvalue") || m.contains("may not be altered") {
            return 542;
        }
        // Missing declaration (undeclared identifier under #DIM ALL /
        // OPTION EXPLICIT / ...)
        if m.contains("missing declaration")
            || m.contains("not declared")
            || m.contains("undeclared")
            || m.contains("must be declared")
        {
            return 519;
        }
        // DefType / type-id required -> Error 516 (this message wording is
        // exactly PBWin's own Error 516 text: "DefType, Type id (?%&!#$), or
        // AS... required").
        if m.contains("deftype") || m.contains("type id") || m.contains("type-id") {
            return 516;
        }
        // String operand expected -> Error 421 (PBWin: assigning a numeric
        // value to a string-typed variable, e.g. `x AS STRING : x = 5`).
        if m.contains("string operand") {
            return 421;
        }
        // Token expectations -> generic syntax error 477 (a more specific 4xx
        // "expected" code is left as future work; 477 is what PBWin returns for
        // a bare syntax failure such as "PRINT 1 +").
        if m.contains("expected") || m.contains("unexpected token") {
            return 477;
        }
        match self.kind {
            ErrorKind::Lexer => 477,        // Syntax error
            ErrorKind::Preprocessor => 475, // Metastatements not allowed here
            ErrorKind::Parser => 477,       // Syntax error
            ErrorKind::Runtime => 51,       // Internal error
            ErrorKind::Io => 51,            // Already handled above; unreachable
        }
    }
}

impl fmt::Display for PbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind_str = match self.kind {
            ErrorKind::Lexer => "Lexer error",
            ErrorKind::Preprocessor => "Preprocessor error",
            ErrorKind::Parser => "Parse error",
            ErrorKind::Runtime => "Runtime error",
            ErrorKind::Io => "I/O error",
        };
        write!(f, "{}", kind_str)?;
        if let Some(ref file) = self.file {
            write!(f, " in {}", file)?;
        }
        if let Some(line) = self.line {
            write!(f, " at line {}", line)?;
            if let Some(col) = self.col {
                write!(f, ":{}", col)?;
            }
        }
        write!(f, ": {}", self.message)
    }
}

impl std::error::Error for PbError {}

pub type PbResult<T> = Result<T, PbError>;
