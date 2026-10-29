// =====================================================================
// Modified by the hanlokesu/PB64Basic fork (https://github.com/hanlokesu/PB64Basic).
// Derived from benstopics/powerbasilisk (https://github.com/benstopics/powerbasilisk), Apache-2.0.
// See LICENSE for the full license terms.
// =====================================================================
use crate::ast::*;
use crate::error::{PbError, PbResult};
use crate::token::{Located, Token};
use std::collections::HashMap;

/// The 12 official DEFtype statement names (CHW DEFBYT_DEFCUR_DEFCUX_DEFDBL_
/// DEFDWD_DEFEXT_DEFINT_DEFLNG.htm).  Note DEFtype only affects implicitly
/// declared variables; under #DIM ALL / OPTION EXPLICIT it has no effect.
const DEFTYPE_NAMES: [&str; 12] = [
    "DEFBYT", "DEFWRD", "DEFINT", "DEFLNG", "DEFQUD", "DEFSNG", "DEFDBL", "DEFEXT", "DEFCUR",
    "DEFCUX", "DEFDWD", "DEFSTR",
];

fn is_deftype_name(name: &str) -> bool {
    DEFTYPE_NAMES.contains(&name.to_uppercase().as_str())
}

fn deftype_for_name(name: &str) -> Option<PbType> {
    match name.to_uppercase().as_str() {
        "DEFBYT" => Some(PbType::Byte),
        "DEFWRD" => Some(PbType::Word),
        "DEFINT" => Some(PbType::Integer),
        "DEFLNG" => Some(PbType::Long),
        "DEFQUD" => Some(PbType::Quad),
        "DEFSNG" => Some(PbType::Single),
        "DEFDBL" => Some(PbType::Double),
        "DEFEXT" => Some(PbType::Ext),
        "DEFCUR" => Some(PbType::Cur),
        // No ExtCurrency in PbType; Cur maps to Double storage (fork's
        // PbType::Cur -> IrType::Double).  DEFCUX keeps that mapping.
        "DEFCUX" => Some(PbType::Cur),
        "DEFDWD" => Some(PbType::Dword),
        "DEFSTR" => Some(PbType::String),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum BodyEnd {
    EndMethod,
    EndSub,
    EndFunction,
    EndFastProc,
    EndProperty,
    EndIf,
    Else,
    ElseIf,
    Next,
    Loop,
    Wend,
    EndSelect,
    Case,
    CaseElse,
}

pub struct Parser {
    tokens: Vec<Located>,
    pending: std::collections::VecDeque<TopLevel>,
    pos: usize,
    def_funcs: HashMap<String, (Vec<String>, Expr)>,
    pub error_count: usize,
    /// Set when the source contains `OPTION EXPLICIT` (same effect as
    /// `#DIM ALL`): every variable must be declared before it is used.
    pub option_explicit: bool,
    /// Interface names declared by INTERFACE blocks inside CLASS blocks
    /// (batch 264).  `AS <IfaceName>` then parses to PbType::Interface, whose
    /// variable storage is a pointer (an object reference), never a struct.
    interface_names: std::collections::HashSet<String>,
    /// Name of the FUNCTION whose body is being parsed (batch 276).  Inside a
    /// function body, `LIBMAIN = expr` is the official return-value assignment
    /// (CHW LIBMAIN_DLLMAIN_function.htm) and must parse as FunctionReturn,
    /// not as an assignment to an undeclared variable.
    current_function_name: Option<String>,
    /// Names of already-defined TYPE/UNION structures (batch 278).  A bare
    /// structure name on its own line inside another TYPE/UNION embeds that
    /// structure (CHW "Structures within structures"); without this registry
    /// the parser would demand `AS` and reject the embed as Error 477.
    defined_types: std::collections::HashSet<String>,
}

impl Parser {
    pub fn new(tokens: Vec<Located>) -> Self {
        let mut parser = Parser {
            tokens,
            pending: std::collections::VecDeque::new(),
            pos: 0,
            def_funcs: HashMap::new(),
            error_count: 0,
            option_explicit: false,
            interface_names: std::collections::HashSet::new(),
            current_function_name: None,
            defined_types: std::collections::HashSet::new(),
        };
        // batch271: pre-register the built-in COM interface names (CHW
        // Object_Data_Types.htm: IDISPATCH = generic IDispatch object variable;
        // IUNKNOWN = generic IUnknown).  `LOCAL o AS IDISPATCH` must be a
        // pointer object reference, not an unsized user type.
        parser.interface_names.insert("IDISPATCH".to_string());
        parser.interface_names.insert("IUNKNOWN".to_string());
        parser
    }

    fn peek(&self) -> &Token {
        self.tokens
            .get(self.pos)
            .map(|t| &t.token)
            .unwrap_or(&Token::Eof)
    }

    /// Reserved words allowed as TYPE/UNION member names (batch 278; the
    /// official MyDLL.bas uses BYREF / DWORD as members). Returns the
    /// canonical uppercase spelling for the given keyword token.
    fn keyword_field_spelling(tok: &Token) -> Option<String> {
        Some(
            match tok {
                Token::Dword => "DWORD",
                Token::Long => "LONG",
                Token::Integer => "INTEGER",
                Token::Single => "SINGLE",
                Token::Double => "DOUBLE",
                Token::Ext => "EXT",
                Token::Quad => "QUAD",
                Token::Cur => "CUR",
                Token::String_ => "STRING",
                Token::Byref => "BYREF",
                Token::Byval => "BYVAL",
                Token::Input => "INPUT",
                Token::Output => "OUTPUT",
                Token::Append => "APPEND",
                Token::Optional => "OPTIONAL",
                Token::Static => "STATIC",
                Token::Register => "REGISTER",
                Token::Threaded => "THREADED",
                _ => return None,
            }
            .to_string(),
        )
    }

    /// Consume a member/field name: an identifier, or a reserved word that PB
    /// allows in that position (e.g. `BYREF AS LONG` inside a TYPE block).
    fn consume_field_name(&mut self) -> PbResult<String> {
        if let Token::Identifier(w) = self.peek() {
            let w = w.clone();
            self.advance();
            return Ok(w);
        }
        if let Some(spelling) = Self::keyword_field_spelling(self.peek()) {
            self.advance();
            return Ok(spelling);
        }
        Err(PbError::generic(format!(
            "Expected identifier (member name) at line {}",
            self.current_line()
        )))
    }

    fn peek_located(&self) -> &Located {
        static EOF: Located = Located {
            token: Token::Eof,
            line: 0,
            col: 0,
            file: None,
        };
        self.tokens.get(self.pos).unwrap_or(&EOF)
    }

    fn current_line(&self) -> usize {
        self.peek_located().line
    }

    fn current_file(&self) -> Option<&str> {
        self.peek_located().file.as_deref()
    }

    fn advance(&mut self) -> &Token {
        let tok = self
            .tokens
            .get(self.pos)
            .map(|t| &t.token)
            .unwrap_or(&Token::Eof);
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        tok
    }

    fn expect(&mut self, expected: &Token) -> PbResult<()> {
        if self.peek() == expected {
            self.advance();
            Ok(())
        } else {
            Err(PbError::parser(
                format!("Expected {:?}, got {:?}", expected, self.peek()),
                self.current_file(),
                self.current_line(),
            ))
        }
    }

    /// Skip a balanced parenthesised group (TYPE/UNION member array
    /// subscripts, e.g. `a1(1 TO 1024)`). Nested parens are handled.
    fn skip_balanced_parens(&mut self) {
        if self.peek() != &Token::LParen {
            return;
        }
        self.advance(); // LParen
        let mut depth = 1usize;
        while depth > 0 && !self.at_end() {
            match self.peek() {
                Token::LParen => {
                    depth += 1;
                    self.advance();
                }
                Token::RParen => {
                    depth -= 1;
                    self.advance();
                }
                _ => {
                    self.advance();
                }
            }
        }
    }

    fn skip_eol(&mut self) {
        while matches!(self.peek(), Token::Eol | Token::Colon) {
            self.advance();
        }
    }

    fn at_end(&self) -> bool {
        matches!(self.peek(), Token::Eof)
    }

    fn at_eol_or_eof(&self) -> bool {
        matches!(self.peek(), Token::Eol | Token::Eof | Token::Colon)
    }

    fn consume_to_eol(&mut self) {
        while !matches!(
            self.peek(),
            Token::Eol | Token::Eof | Token::Colon | Token::Else | Token::ElseIf
        ) {
            self.advance();
        }
        // Consume Eol but NOT Colon/Else/ElseIf — these are statement-level
        // boundaries that callers need to detect
        // (e.g., single-line IF: IF cond THEN stmt ELSE stmt)
        if self.peek() == &Token::Eol {
            self.advance();
        }
    }

    #[allow(dead_code)]
    fn is_identifier(&self, name: &str) -> bool {
        matches!(self.peek(), Token::Identifier(ref s) if s == name)
    }

    #[allow(dead_code)]
    fn peek_identifier(&self) -> Option<&str> {
        match self.peek() {
            Token::Identifier(s) => Some(s.as_str()),
            _ => None,
        }
    }

    fn consume_identifier(&mut self) -> PbResult<String> {
        match self.peek().clone() {
            Token::Identifier(s) => {
                self.advance();
                Ok(s)
            }
            _ => Err(PbError::parser(
                format!("Expected identifier, got {:?}", self.peek()),
                self.current_file(),
                self.current_line(),
            )),
        }
    }

    pub fn parse(&mut self) -> PbResult<Program> {
        let mut items = Vec::new();
        self.skip_eol();

        while !self.at_end() {
            let saved_pos = self.pos;
            // An item may come from a queue (METHOD blocks inside a CLASS): that hands back
            // an item WITHOUT consuming a token, so `produced` - not `pos` alone - decides
            // whether this step really made no progress.
            let mut produced = false;
            match self.parse_top_level() {
                Ok(Some(item)) => {
                    items.push(item);
                    produced = true;
                }
                Ok(None) => {} // consumed but no item (e.g., comment)
                Err(e) => {
                    eprintln!("Error: Parse error: {}", e);
                    self.error_count += 1;
                    self.consume_to_eol();
                }
            }
            self.skip_eol();
            // Safety: prevent infinite loop at top level
            if !produced && self.pos == saved_pos && !self.at_end() {
                self.advance();
            }
        }

        Ok(Program { items })
    }

    fn parse_top_level(&mut self) -> PbResult<Option<TopLevel>> {
        // Methods parsed inside a CLASS block are handed back one per call, because
        // parse_top_level returns a single item.
        if let Some(item) = self.pending.pop_front() {
            return Ok(Some(item));
        }
        self.skip_eol();
        if self.at_end() {
            return Ok(None);
        }
        let line = self.current_line();

        match self.peek().clone() {
            Token::Global => self.parse_global_decl(),
            Token::Dim => {
                let stmts = self.parse_dim_statement(DimScope::Dim)?;
                Ok(Some(TopLevel::DimDeclList(stmts)))
            }
            Token::Threaded => {
                let stmts = self.parse_dim_statement(DimScope::Threaded)?;
                Ok(Some(TopLevel::DimDeclList(stmts)))
            }
            Token::Redim => {
                let stmt = self.parse_redim_statement()?;
                Ok(Some(TopLevel::DimDecl(stmt)))
            }
            Token::PercentConstant(_) => {
                let c = self.parse_percent_const_decl(line)?;
                Ok(Some(TopLevel::ConstDecl(c)))
            }
            Token::Type => {
                let td = self.parse_type_decl(line)?;
                Ok(Some(TopLevel::TypeDecl(td)))
            }
            Token::Union => {
                let td = self.parse_union_decl(line)?;
                Ok(Some(TopLevel::TypeDecl(td)))
            }
            Token::Enum => {
                let ed = self.parse_enum_decl(line)?;
                Ok(Some(TopLevel::EnumDecl(ed)))
            }
            Token::Sub => {
                let sd = self.parse_sub_decl()?;
                Ok(Some(TopLevel::SubDecl(sd)))
            }
            Token::Function => {
                let fd = self.parse_function_decl()?;
                Ok(Some(TopLevel::FunctionDecl(fd)))
            }
            Token::FastProc => {
                // CHW FASTPROC_END_FASTPROC_statements.htm: `FASTPROC
                // ProcName([args]) [THREADSAFE] [AS LONG] ... END FASTPROC
                // [= RetVal]`.  Parsed exactly like a FUNCTION (argument /
                // return-type syntax is the same); the body terminator is END
                // FASTPROC (END FUNCTION also accepted).  The FastProc
                // register/no-stack-frame execution model is not reproduced on
                // LLVM -- a normal function is emitted, so behaviour matches
                // and the official syntax is accepted.  (Added in the A/B/C
                // re-audit.)
                self.advance(); // FASTPROC
                let fd = self.parse_fastproc_decl()?;
                Ok(Some(TopLevel::FunctionDecl(fd)))
            }
            Token::Callback => {
                // batch 261: CALLBACK FUNCTION / CALLBACK SUB top-level declaration
                // (CHW callbacks.htm — DDT callback functions are declared with
                // the CALLBACK keyword, e.g. `CALLBACK FUNCTION DlgProc() AS LONG`).
                match self.peek_at(1) {
                    Some(Token::Function) => {
                        self.advance(); // CALLBACK
                        let fd = self.parse_function_decl()?;
                        Ok(Some(TopLevel::FunctionDecl(fd)))
                    }
                    Some(Token::Sub) => {
                        self.advance(); // CALLBACK
                        let sd = self.parse_sub_decl()?;
                        Ok(Some(TopLevel::SubDecl(sd)))
                    }
                    _ => {
                        self.consume_to_eol();
                        Ok(Some(TopLevel::Statement(Statement::Noop(
                            "CALLBACK".to_string(),
                            line,
                        ))))
                    }
                }
            }
            Token::Identifier(w)
                if w.eq_ignore_ascii_case("THREAD")
                    && self.peek_at(1) == Some(&Token::Function) =>
            {
                // batch 234: official THREAD FUNCTION Fn(byval x AS LONG) AS LONG
                // declaration form -- THREAD is an Identifier token here, so it
                // must be peeled off before the normal FUNCTION decl parser runs.
                self.advance(); // THREAD
                let fd = self.parse_function_decl()?;
                Ok(Some(TopLevel::FunctionDecl(fd)))
            }
            Token::Declare => match self.parse_declare_stmt(line)? {
                Some(ds) => Ok(Some(TopLevel::DeclareStmt(ds))),
                None => Ok(None),
            },
            Token::HashInclude
            | Token::HashCompile
            | Token::HashDim
            | Token::HashRegister
            | Token::HashDebug => {
                self.consume_to_eol();
                Ok(None)
            }
            Token::HashLink => {
                // #LINK "filespec" — extra object/library handed to the linker
                // (CHW _LINK metastatement; batch 260, Erol request).  The fork
                // accepts .obj/.lib (clang/lld inputs) in addition to the
                // official .SLL/.PBLIB spellings.
                self.advance(); // consume #LINK
                let path = match self.peek() {
                    Token::StringLiteral(s) => s.clone(),
                    _ => String::new(),
                };
                self.consume_to_eol();
                if path.is_empty() {
                    eprintln!("Error: #LINK requires a quoted file spec on line {}", line);
                    self.error_count += 1;
                }
                Ok(Some(TopLevel::LinkFile(path)))
            }
            Token::Macro => {
                // MACRO name = body : END MACRO — skip to END MACRO
                self.advance();
                loop {
                    match self.peek() {
                        Token::EndMacro | Token::Eof => {
                            self.advance();
                            break;
                        }
                        _ => {
                            self.advance();
                        }
                    }
                }
                self.consume_to_eol();
                Ok(None)
            }
            Token::Identifier(w) if w.eq_ignore_ascii_case("ASMDATA") => {
                let ad = self.parse_asmdata_decl(line)?;
                Ok(Some(TopLevel::AsmData(ad)))
            }
            Token::Identifier(w) if w.eq_ignore_ascii_case("CLASS") => {
                // CLASS Name ... END CLASS — the body's INSTANCE lines are the object's per-object
                // storage, so a CLASS produces a real TypeDecl (the object's struct type).  Until
                // batch 212 the whole block was skipped and INSTANCE had no storage at all.
                let line = self.current_line();
                self.advance(); // consume CLASS
                let name = if let Token::Identifier(cn) = self.peek() {
                    let n = cn.clone();
                    self.advance();
                    n
                } else {
                    String::new()
                };
                self.consume_to_eol();
                self.skip_eol();

                let mut fields: Vec<TypeField> = Vec::new();
                let mut event_sources: Vec<String> = Vec::new();
                // Interfaces implemented by this CLASS (batch 264).  An INTERFACE
                // block inside a CLASS records the interface name -> class name
                // mapping so `LOCAL o AS Iface` + `o.M(a)` can be lowered to the
                // compiled <Class>_<Method> function.
                let mut interfaces: Vec<String> = Vec::new();
                loop {
                    if self.at_end() || self.peek() == &Token::Eof {
                        break;
                    }
                    if self.peek() == &Token::End
                        && matches!(self.peek_at(1), Some(Token::Identifier(e))
                                    if e.eq_ignore_ascii_case("CLASS"))
                    {
                        self.advance(); // END
                        self.advance(); // CLASS
                        self.consume_to_eol();
                        break;
                    }
                    let is_instance = matches!(self.peek(), Token::Identifier(k)
                                               if k.eq_ignore_ascii_case("INSTANCE"));
                    if is_instance {
                        self.advance(); // INSTANCE
                        loop {
                            if self.at_end() {
                                break;
                            }
                            let fname = match self.peek() {
                                Token::Identifier(f) => {
                                    let n = f.clone();
                                    self.advance();
                                    n
                                }
                                _ => break,
                            };
                            // INSTANCE a(10) AS LONG needs an array-typed struct slot, which is not
                            // supported yet.  Reject it loudly instead of skipping the line silently.
                            if self.peek() == &Token::LParen {
                                self.error_count += 1;
                                self.consume_to_eol();
                                break;
                            }
                            let ftype = if self.peek() == &Token::As {
                                self.advance();
                                self.parse_type()?
                            } else {
                                PbType::Long
                            };
                            fields.push(TypeField {
                                name: fname,
                                pb_type: ftype,
                                nested: None,
                            });
                            if self.peek() == &Token::Comma {
                                self.advance();
                                continue;
                            }
                            break;
                        }
                        self.consume_to_eol();
                        self.skip_eol();
                        continue;
                    }
                    // METHOD name ... END METHOD: a real method body.  It leaves this
                    // branch as an ordinary SUB/FUNCTION (see parse_method_decl), queued in
                    // self.pending because parse_top_level hands back one item per call.
                    let is_method = matches!(self.peek(), Token::Identifier(k)
                                               if k.eq_ignore_ascii_case("METHOD"));
                    if is_method {
                        let item = self.parse_method_decl(Some(&name))?;
                        self.pending.push_back(item);
                        self.skip_eol();
                        continue;
                    }
                    // PROPERTY GET|SET name ... END PROPERTY — property procedures
                    // inside a CLASS (batch 298, CHW PROPERTY/END PROPERTY).  Compiled
                    // as <Class>_<Name>_Get / <Class>_<Name>_Set free functions, the
                    // same lowering METHOD uses, so codegen needs no new machinery.
                    let is_property = matches!(self.peek(), Token::Identifier(k)
                                                if k.eq_ignore_ascii_case("PROPERTY"))
                        && matches!(self.peek_at(1), Some(Token::Identifier(k2))
                                    if k2.eq_ignore_ascii_case("GET") || k2.eq_ignore_ascii_case("SET"));
                    if is_property {
                        let item = self.parse_property_decl(Some(&name))?;
                        self.pending.push_back(item);
                        self.skip_eol();
                        continue;
                    }
                    // INTERFACE IfaceName — implementation block inside a CLASS
                    // (CHW CLASS_END_CLASS_Block: a CLASS implements an interface
                    // by declaring an INTERFACE block that names it).  The block's
                    // INHERIT line and END INTERFACE line are consumed by the
                    // generic fall-through below; every METHOD body inside it is
                    // compiled as a <Class>_<Method> function by the branch above.
                    let is_iface = matches!(self.peek(), Token::Identifier(k)
                                              if k.eq_ignore_ascii_case("INTERFACE"));
                    if is_iface {
                        self.advance(); // INTERFACE
                        if let Token::Identifier(iname) = self.peek() {
                            let iface = iname.to_uppercase();
                            interfaces.push(iface.clone());
                            self.interface_names.insert(iface);
                            self.advance();
                        }
                        self.consume_to_eol();
                        self.skip_eol();
                        continue;
                    }
                    // EVENT SOURCE IfaceName — advertise an event interface this
                    // class can raise (batch 222).  Collected into the TypeDecl so
                    // codegen can validate RAISEEVENT uses.
                    let is_event_source = matches!(self.peek(), Token::Identifier(k)
                                                    if k.eq_ignore_ascii_case("EVENT"))
                        && matches!(self.peek_at(1), Some(Token::Identifier(k2))
                                    if k2.eq_ignore_ascii_case("SOURCE"));
                    if is_event_source {
                        self.advance(); // EVENT
                        self.advance(); // SOURCE
                        if let Token::Identifier(iface) = self.peek() {
                            event_sources.push(iface.clone());
                            self.advance();
                        }
                        self.consume_to_eol();
                        self.skip_eol();
                        continue;
                    }
                    self.consume_to_eol();
                    self.skip_eol();
                }

                if name.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(TopLevel::TypeDecl(TypeDecl {
                        name,
                        fields,
                        line,
                        is_union: false,
                        event_sources,
                        interfaces,
                    })))
                }
            }
            Token::Identifier(w) if w.eq_ignore_ascii_case("OPTION") => {
                // OPTION EXPLICIT — official PB statement with the same effect
                // as #DIM ALL: all variables must be declared before use.
                // Nothing is emitted here; codegen enforces it.
                self.advance(); // consume OPTION
                if self.peek_plain_upper() == "EXPLICIT" {
                    self.advance();
                    self.option_explicit = true;
                }
                self.consume_to_eol();
                Ok(None)
            }
            Token::Identifier(w) if w.eq_ignore_ascii_case("METHOD") => {
                // parse_method_decl() consumes the leading keyword itself and honours an
                // `AS type` return type.  Going through parse_sub_decl() here (batch 159)
                // silently dropped that type, so `FUNCTION = x` inside a top-level METHOD
                // returned 0 to its caller (fixed in batch 214).
                Ok(Some(self.parse_method_decl(None)?))
            }
            Token::Identifier(w) if w.eq_ignore_ascii_case("INTERFACE") => {
                // INTERFACE Name [DIRECT|IDBIND] ... END INTERFACE — OOP interface block (skip)
                self.advance(); // consume INTERFACE
                if let Token::Identifier(_) = self.peek() {
                    self.advance();
                }
                // optional DIRECT or IDBIND
                if let Token::Identifier(w2) = self.peek() {
                    let up = w2.to_uppercase();
                    if up == "DIRECT" || up == "IDBIND" {
                        self.advance();
                    }
                }
                self.consume_to_eol();
                let mut depth = 1;
                while depth > 0 && self.peek() != &Token::Eof {
                    let is_iface_start = matches!(self.peek(), Token::Identifier(w2) if w2.eq_ignore_ascii_case("INTERFACE"));
                    let is_end = matches!(self.peek(), Token::End);
                    let next_is_iface = matches!(self.peek_at(1), Some(Token::Identifier(w3)) if w3.eq_ignore_ascii_case("INTERFACE"));
                    if is_end && next_is_iface {
                        depth -= 1;
                        self.advance();
                        self.advance();
                        self.consume_to_eol();
                        continue;
                    }
                    if is_iface_start {
                        depth += 1;
                    }
                    self.advance();
                }
                Ok(None)
            }
            Token::Identifier(w) if w.eq_ignore_ascii_case("INSTANCE") => {
                // INSTANCE declares per-object storage and is only valid at the
                // start of a CLASS/END CLASS block.  A top-level occurrence was
                // silently swallowed before batch 222; report it loudly.
                self.error_count += 1;
                eprintln!(
                    "Error: Parse error at line {}: INSTANCE is only valid inside a CLASS/END CLASS block",
                    line
                );
                self.consume_to_eol();
                Ok(None)
            }
            Token::Identifier(w) if w.eq_ignore_ascii_case("RAISEEVENT") => {
                // RAISEEVENT Iface.Method(args) fires event handlers and is only
                // valid inside a CLASS that declares the EVENT SOURCE interface.
                self.error_count += 1;
                eprintln!(
                    "Error: Parse error at line {}: RAISEEVENT is only valid inside a CLASS that declares an EVENT SOURCE interface",
                    line
                );
                self.consume_to_eol();
                Ok(None)
            }
            Token::Identifier(w) if w.eq_ignore_ascii_case("EVENT") => {
                // EVENT SOURCE declares an event interface and is only valid
                // inside a CLASS/END CLASS block.  EVENTS FROM/END is a statement
                // and is parsed by the statement branch, not here.
                self.error_count += 1;
                eprintln!(
                    "Error: Parse error at line {}: EVENT SOURCE is only valid inside a CLASS/END CLASS block (EVENTS FROM/END is a statement)",
                    line
                );
                self.consume_to_eol();
                Ok(None)
            }
            Token::Identifier(w) if w.eq_ignore_ascii_case("DEF") => {
                // DEF fnName(params) = expr — single-line function, stored for inline
                self.advance(); // consume DEF
                let fname = self.consume_identifier()?;
                self.expect(&Token::LParen)?;
                let mut params: Vec<String> = Vec::new();
                if self.peek() != &Token::RParen {
                    loop {
                        params.push(self.consume_identifier()?);
                        if self.peek() != &Token::Comma {
                            break;
                        }
                        self.advance();
                    }
                }
                self.expect(&Token::RParen)?;
                self.expect(&Token::Eq)?;
                let body = self.parse_expression()?;
                self.def_funcs.insert(fname.to_uppercase(), (params, body));
                Ok(None)
            }
            Token::HashIf => {
                // Should have been handled by preprocessor, but skip if present
                self.skip_conditional_block();
                Ok(None)
            }
            Token::Identifier(name) => {
                eprintln!(
                    "Error: Parse error at line {}: unrecognized statement/keyword: `{}`",
                    line, name
                );
                self.error_count += 1;
                self.consume_to_eol();
                Ok(None)
            }
            Token::IntegerLiteral(n) => {
                eprintln!(
                    "Error: Parse error at line {}: unexpected number at top level: {}",
                    line, n
                );
                self.error_count += 1;
                self.consume_to_eol();
                Ok(None)
            }
            Token::FloatLiteral(f) => {
                eprintln!(
                    "Error: Parse error at line {}: unexpected number at top level: {}",
                    line, f
                );
                self.error_count += 1;
                self.consume_to_eol();
                Ok(None)
            }
            Token::StringLiteral(s) => {
                eprintln!(
                    "Error: Parse error at line {}: unexpected string at top level: `{}`",
                    line, s
                );
                self.error_count += 1;
                self.consume_to_eol();
                Ok(None)
            }
            _ => {
                // Other unknown tokens, skip silently
                self.consume_to_eol();
                Ok(None)
            }
        }
    }

    /// ASMDATA BlockName / DB|DW|DD|DQ values... / END ASMDATA
    fn parse_asmdata_decl(&mut self, line: usize) -> PbResult<AsmDataDecl> {
        self.advance(); // consume ASMDATA
        let name = match self.peek().clone() {
            Token::Identifier(w) => w.to_uppercase(),
            other => {
                return Err(PbError::parser(
                    format!(
                        "ASMDATA: expected block name after ASMDATA, found {:?}",
                        other
                    ),
                    None,
                    line,
                ))
            }
        };
        self.advance();
        let mut items: Vec<AsmDataItem> = Vec::new();
        loop {
            self.skip_eol();
            if self.at_end() {
                break;
            }
            let tok = self.peek().clone();
            match tok {
                Token::Identifier(w) if w.eq_ignore_ascii_case("DB") => {
                    self.advance();
                    items.push(AsmDataItem::Db(self.parse_asmdata_values(line)?));
                }
                Token::Identifier(w) if w.eq_ignore_ascii_case("DW") => {
                    self.advance();
                    items.push(AsmDataItem::Dw(self.parse_asmdata_values(line)?));
                }
                Token::Identifier(w) if w.eq_ignore_ascii_case("DD") => {
                    self.advance();
                    items.push(AsmDataItem::Dd(self.parse_asmdata_values(line)?));
                }
                Token::Identifier(w) if w.eq_ignore_ascii_case("DQ") => {
                    self.advance();
                    items.push(AsmDataItem::Dq(self.parse_asmdata_values(line)?));
                }
                Token::End => {
                    self.advance();
                    // expect ASMDATA after END
                    if let Token::Identifier(w2) = self.peek().clone() {
                        if w2.eq_ignore_ascii_case("ASMDATA") {
                            self.advance();
                            self.consume_to_eol();
                            break;
                        }
                    }
                    break;
                }
                _ => {
                    // tolerate unknown lines inside the block (skip)
                    self.consume_to_eol();
                }
            }
        }
        Ok(AsmDataDecl { name, items, line })
    }

    fn parse_asmdata_values(&mut self, line: usize) -> PbResult<Vec<AsmDataValue>> {
        let mut values: Vec<AsmDataValue> = Vec::new();
        loop {
            let tok = self.peek().clone();
            match tok {
                Token::IntegerLiteral(n) => {
                    self.advance();
                    values.push(AsmDataValue::Num(n));
                }
                Token::StringLiteral(s) => {
                    self.advance();
                    values.push(AsmDataValue::Str(s));
                }
                Token::Minus => {
                    self.advance();
                    match self.peek().clone() {
                        Token::IntegerLiteral(n) => {
                            self.advance();
                            values.push(AsmDataValue::Num(-n));
                        }
                        _ => break,
                    }
                }
                Token::Comma => {
                    self.advance();
                }
                _ => break,
            }
        }
        if values.is_empty() {
            return Err(PbError::parser(
                "ASMDATA: DB/DW/DD/DQ line needs at least one value",
                None,
                line,
            ));
        }
        Ok(values)
    }

    fn skip_conditional_block(&mut self) {
        let mut depth = 0;
        loop {
            match self.peek() {
                Token::HashIf => {
                    depth += 1;
                    self.advance();
                }
                Token::HashEndIf => {
                    self.advance();
                    depth -= 1;
                    if depth <= 0 {
                        self.consume_to_eol();
                        break;
                    }
                }
                Token::Eof => break,
                _ => {
                    self.advance();
                }
            }
        }
    }

    fn parse_global_decl(&mut self) -> PbResult<Option<TopLevel>> {
        let line = self.current_line();
        self.advance(); // consume GLOBAL

        // Parse first (and possibly only) variable
        let decls = self.parse_global_var_list(line)?;
        self.consume_to_eol();

        if decls.len() == 1 {
            return Ok(Some(TopLevel::GlobalDecl(
                decls.into_iter().next().unwrap(),
            )));
        }
        // Multiple globals on one line: emit as GlobalDeclList
        Ok(Some(TopLevel::GlobalDeclList(decls)))
    }

    fn parse_global_var_list(&mut self, line: usize) -> PbResult<Vec<VarDecl>> {
        let mut decls: Vec<VarDecl> = Vec::new();
        // PB semantics: a trailing AS type-fills the whole list, both
        // back-filling previously untyped names and propagating forward.
        // `GLOBAL a, b AS QUAD` declares both a and b as QUAD.
        let mut pending_type: Option<PbType> = None;
        let mut untyped: Vec<usize> = Vec::new();
        loop {
            let name = self.consume_identifier()?;

            // Check for array declaration: Name() [AS Type]
            if self.peek() == &Token::LParen {
                self.advance(); // (
                self.expect(&Token::RParen)?; // )
                let idx = decls.len();
                let pb_type = if self.peek() == &Token::As {
                    self.advance();
                    let t = self.parse_type()?;
                    for i in &untyped {
                        decls[*i].pb_type = t.clone();
                    }
                    untyped.clear();
                    pending_type = Some(t.clone());
                    t
                } else if let Some(t) = &pending_type {
                    t.clone()
                } else {
                    let t = type_from_suffix(&name);
                    untyped.push(idx);
                    t
                };
                decls.push(VarDecl {
                    name,
                    pb_type,
                    is_array: true,
                    line,
                });
            } else if self.peek() == &Token::As {
                self.advance();
                let t = self.parse_type()?;
                for i in &untyped {
                    decls[*i].pb_type = t.clone();
                }
                untyped.clear();
                pending_type = Some(t.clone());
                decls.push(VarDecl {
                    name,
                    pb_type: t,
                    is_array: false,
                    line,
                });
            } else if let Some(t) = &pending_type {
                decls.push(VarDecl {
                    name,
                    pb_type: t.clone(),
                    is_array: false,
                    line,
                });
            } else {
                // GLOBAL name (no type, infer from suffix)
                let idx = decls.len();
                let pb_type = type_from_suffix(&name);
                untyped.push(idx);
                decls.push(VarDecl {
                    name,
                    pb_type,
                    is_array: false,
                    line,
                });
            }

            // Check for comma-separated additional variables
            if self.peek() == &Token::Comma {
                self.advance();
            } else {
                break;
            }
        }
        Ok(decls)
    }

    fn parse_dim_statement(&mut self, scope: DimScope) -> PbResult<Vec<DimStatement>> {
        let line = self.current_line();
        self.advance(); // consume DIM/LOCAL/STATIC

        // DIM SHARED -> treat as global
        let actual_scope = if self.peek() == &Token::Shared {
            self.advance();
            DimScope::Global
        } else {
            scope
        };

        let mut stmts: Vec<DimStatement> = Vec::new();
        // PB semantics: a trailing AS type-fills the whole list, both
        // back-filling previously untyped names and propagating forward
        // until the next AS. E.g. `DIM a, b AS LONG` → a and b are LONG.
        let mut pending_type: Option<PbType> = None;
        let mut untyped: Vec<usize> = Vec::new();

        loop {
            let name = self.consume_identifier()?;

            // Array bounds
            let mut bounds = Vec::new();
            let mut is_array = false;
            if self.peek() == &Token::LParen {
                is_array = true;
                self.advance();
                if self.peek() != &Token::RParen {
                    bounds = self.parse_dim_bounds()?;
                }
                self.expect(&Token::RParen)?;
            }

            let idx = stmts.len();
            let pb_type = if self.peek() == &Token::As {
                self.advance();
                let t = self.parse_type()?;
                for i in &untyped {
                    stmts[*i].pb_type = t.clone();
                }
                untyped.clear();
                pending_type = Some(t.clone());
                t
            } else if let Some(t) = &pending_type {
                t.clone()
            } else {
                let t = type_from_suffix(&name);
                untyped.push(idx);
                t
            };

            stmts.push(DimStatement {
                scope: actual_scope.clone(),
                name,
                pb_type,
                bounds,
                line,
                is_redim: false,
                is_array,
            });

            if self.peek() == &Token::Comma {
                self.advance();
            } else {
                break;
            }
        }

        self.consume_to_eol();
        Ok(stmts)
    }

    fn parse_redim_statement(&mut self) -> PbResult<DimStatement> {
        let line = self.current_line();
        self.advance(); // consume REDIM

        let name = self.consume_identifier()?;

        let mut bounds = Vec::new();
        let mut is_array = false;
        if self.peek() == &Token::LParen {
            is_array = true;
            self.advance();
            if self.peek() != &Token::RParen {
                bounds = self.parse_dim_bounds()?;
            }
            self.expect(&Token::RParen)?;
        }

        let pb_type = if self.peek() == &Token::As {
            self.advance();
            self.parse_type()?
        } else {
            let t = type_from_suffix(&name);
            // REDIM without an explicit type: if the name has no type suffix,
            // mark it Variant so codegen can inherit the type declared earlier
            // (e.g. `LOCAL sarr() AS STRING` + `REDIM sarr(1 TO 4)`).
            if has_type_suffix(&name) {
                t
            } else {
                PbType::Variant
            }
        };

        self.consume_to_eol();
        Ok(DimStatement {
            scope: DimScope::Dim,
            name,
            pb_type,
            bounds,
            line,
            is_redim: true,
            is_array,
        })
    }

    fn parse_dim_bounds(&mut self) -> PbResult<Vec<DimBound>> {
        let mut bounds = Vec::new();
        loop {
            let first = self.parse_expression()?;
            // PB supports both TO and : as range separator: DIM arr(1 TO 10) or DIM arr(1:10)
            if self.peek() == &Token::To || self.peek() == &Token::Colon {
                self.advance();
                let upper = self.parse_expression()?;
                bounds.push(DimBound {
                    lower: first,
                    upper,
                });
            } else {
                // Just upper bound, lower is 0
                bounds.push(DimBound {
                    lower: Expr::IntegerLit(0),
                    upper: first,
                });
            }
            if self.peek() == &Token::Comma {
                self.advance();
            } else {
                break;
            }
        }
        Ok(bounds)
    }

    fn parse_type(&mut self) -> PbResult<PbType> {
        let t = self.parse_type_inner()?;
        // PTR suffix: DWORD PTR / ASCIIZ PTR / LONG PTR — x64 pointers are 64-bit
        if let Token::Identifier(s) = self.peek() {
            if s.eq_ignore_ascii_case("PTR") {
                self.advance();
                return Ok(PbType::Quad);
            }
        }
        // Bare PTR (already consumed as UserDefined("PTR"))
        if let PbType::UserDefined(n) = &t {
            if n.eq_ignore_ascii_case("PTR") {
                return Ok(PbType::Quad);
            }
        }
        Ok(t)
    }

    fn parse_type_inner(&mut self) -> PbResult<PbType> {
        // Skip LOCAL keyword if present (PB allows DIM arr() AS LOCAL STRING)
        if self.peek() == &Token::Local {
            self.advance();
        }
        match self.peek().clone() {
            Token::Long => {
                self.advance();
                Ok(PbType::Long)
            }
            Token::Identifier(w) if w.eq_ignore_ascii_case("OBJECT") => {
                self.advance();
                Ok(PbType::Long) // OBJECT = COM object pointer (simplified as LONG)
            }
            Token::Identifier(w) if w.eq_ignore_ascii_case("WSTRING") => {
                self.advance();
                Ok(PbType::WString) // UTF-16 wide string (batch280)
            }
            // batch294: WSTRINGZ is a pointer to a NUL-terminated wide string
            // (WINMAIN's lpszCmdLine parameter); the fork stores wide strings
            // as UTF-16 BSTR pointers, so the same representation applies.
            Token::Identifier(w) if w.eq_ignore_ascii_case("WSTRINGZ") => {
                self.advance();
                Ok(PbType::WString)
            }
            Token::Integer => {
                self.advance();
                Ok(PbType::Integer)
            }
            Token::Double => {
                self.advance();
                Ok(PbType::Double)
            }
            Token::Single => {
                self.advance();
                Ok(PbType::Single)
            }
            Token::String_ => {
                self.advance();
                // Check for STRING * N (fixed string)
                if self.peek() == &Token::Star {
                    self.advance();
                    if let Token::IntegerLiteral(n) = self.peek().clone() {
                        self.advance();
                        return Ok(PbType::FixedString(n as usize));
                    }
                }
                Ok(PbType::String)
            }
            Token::Quad => {
                self.advance();
                Ok(PbType::Quad)
            }
            Token::Dword => {
                self.advance();
                Ok(PbType::Dword)
            }
            Token::Ext => {
                self.advance();
                Ok(PbType::Ext)
            }
            Token::Cur => {
                self.advance();
                Ok(PbType::Cur)
            }
            Token::Identifier(ref s) => {
                let upper = s.to_uppercase();
                match upper.as_str() {
                    "BYTE" => {
                        self.advance();
                        Ok(PbType::Byte)
                    }
                    "WORD" => {
                        self.advance();
                        Ok(PbType::Word)
                    }
                    "CURRENCY" | "CUR" => {
                        self.advance();
                        Ok(PbType::Cur)
                    }
                    "ASCIIZ" | "ASCIZ" => {
                        self.advance();
                        // ASCIIZ * N — fixed-size null-terminated string
                        if self.peek() == &Token::Star {
                            self.advance();
                            if let Token::IntegerLiteral(n) = self.peek().clone() {
                                self.advance();
                                return Ok(PbType::FixedString(n as usize));
                            }
                        }
                        Ok(PbType::FixedString(260)) // default ASCIIZ size
                    }
                    "FIELD" => {
                        self.advance();
                        Ok(PbType::Field)
                    }
                    // batch293: VARIANT is a real PB type (16-byte container),
                    // not a user-defined type name.
                    "VARIANT" => {
                        self.advance();
                        Ok(PbType::Variant)
                    }
                    _ => {
                        let name = s.clone();
                        self.advance();
                        // An interface name used as a variable type is an
                        // object reference (batch 264): `LOCAL o AS MyIface`.
                        if self.interface_names.contains(&name.to_uppercase()) {
                            Ok(PbType::Interface(name.to_uppercase()))
                        } else {
                            Ok(PbType::UserDefined(name))
                        }
                    }
                }
            }
            _ => {
                // Default
                Ok(PbType::Variant)
            }
        }
    }

    fn parse_percent_const_decl(&mut self, line: usize) -> PbResult<ConstDecl> {
        let name = match self.peek().clone() {
            Token::PercentConstant(s) => {
                self.advance();
                if s.eq_ignore_ascii_case("%CONSTANT") {
                    // Official form `%CONSTANT %name = value` (CHW
                    // constant_directive.htm): the keyword token is a plain
                    // PercentConstant like the name itself.  Consume it, then
                    // take the next PercentConstant as the name (batch 276 —
                    // previously the keyword was taken as the name and the
                    // real name caused "Expected Eq, got PercentConstant").
                    match self.peek().clone() {
                        Token::PercentConstant(s2) => {
                            self.advance();
                            s2
                        }
                        _ => {
                            return Err(PbError::parser(
                                "Expected %CONSTANT name",
                                self.current_file(),
                                line,
                            ))
                        }
                    }
                } else {
                    s
                }
            }
            _ => {
                return Err(PbError::parser(
                    "Expected %CONSTANT",
                    self.current_file(),
                    line,
                ))
            }
        };
        self.expect(&Token::Eq)?;
        let value = self.parse_expression()?;
        self.consume_to_eol();
        Ok(ConstDecl { name, value, line })
    }

    fn parse_type_decl(&mut self, line: usize) -> PbResult<TypeDecl> {
        self.advance(); // TYPE
        let name = self.consume_identifier()?;
        self.consume_to_eol();
        self.skip_eol();

        let mut fields = Vec::new();
        loop {
            if self.peek() == &Token::End {
                self.advance();
                // END TYPE
                if self.peek() == &Token::Type {
                    self.advance();
                }
                self.consume_to_eol();
                break;
            }
            if self.at_end() {
                break;
            }

            // Parse field: name AS type (array subscripts `(lo TO hi)` are
            // skipped — CHW TYPE members may be arrays).
            let fname = self.consume_field_name()?;
            if self.peek() == &Token::LParen {
                self.skip_balanced_parens();
            }
            // CHW "Structures within structures": a bare previously-defined
            // structure name on its own line embeds that TYPE/UNION.
            if self.peek() != &Token::As && self.defined_types.contains(&fname.to_uppercase()) {
                let nested_name = fname.to_uppercase();
                fields.push(TypeField {
                    name: String::new(),
                    pb_type: PbType::Long,
                    nested: Some(nested_name),
                });
                self.consume_to_eol();
                self.skip_eol();
                continue;
            }
            self.expect(&Token::As)?;
            let ftype = self.parse_type()?;
            fields.push(TypeField {
                name: fname,
                pb_type: ftype,
                nested: None,
            });
            self.consume_to_eol();
            self.skip_eol();
        }

        let key = name.to_uppercase();
        self.defined_types.insert(key);
        Ok(TypeDecl {
            name,
            fields,
            line,
            is_union: false,
            event_sources: Vec::new(),
            interfaces: Vec::new(),
        })
    }

    /// `UNION name ... END UNION` (CHW UNION/END UNION): a user-defined type
    /// whose members all overlap at offset 0. Same member syntax as TYPE
    /// (`name [(subscripts)] AS type`), including embedded arrays. Bit fields
    /// (`AS BIT * n`) are not supported yet — they surface as parse errors.
    fn parse_union_decl(&mut self, line: usize) -> PbResult<TypeDecl> {
        self.advance(); // UNION
        let name = self.consume_identifier()?;
        self.consume_to_eol();
        self.skip_eol();

        let mut fields = Vec::new();
        loop {
            if self.peek() == &Token::End {
                self.advance();
                if self.peek() == &Token::Union {
                    self.advance();
                }
                self.consume_to_eol();
                break;
            }
            if self.at_end() {
                break;
            }
            // Optional `name[(lo TO hi)[, ...]]` array subscripts (TYPE member
            // arrays follow the same rules — CHW TYPE/END TYPE "TYPE Members").
            let fname = self.consume_field_name()?;
            if self.peek() == &Token::LParen {
                self.skip_balanced_parens();
            }
            // CHW: structures may be embedded inside a UNION too (same member
            // syntax as TYPE).
            if self.peek() != &Token::As && self.defined_types.contains(&fname.to_uppercase()) {
                let nested_name = fname.to_uppercase();
                fields.push(TypeField {
                    name: String::new(),
                    pb_type: PbType::Long,
                    nested: Some(nested_name),
                });
                self.consume_to_eol();
                self.skip_eol();
                continue;
            }
            self.expect(&Token::As)?;
            let ftype = self.parse_type()?;
            fields.push(TypeField {
                name: fname,
                pb_type: ftype,
                nested: None,
            });
            self.consume_to_eol();
            self.skip_eol();
        }

        let key = name.to_uppercase();
        self.defined_types.insert(key);
        Ok(TypeDecl {
            name,
            fields,
            line,
            is_union: true,
            event_sources: Vec::new(),
            interfaces: Vec::new(),
        })
    }

    /// `ENUM name [BITS] [SINGULAR] ... END ENUM` (CHW ENUM/END ENUM):
    /// each member registers as an equate. Auto-increment from 0 (or from the
    /// previous explicit value + 1); BITS mode auto-assigns 1, 2, 4, 8 ...
    fn parse_enum_decl(&mut self, line: usize) -> PbResult<EnumDecl> {
        self.advance(); // ENUM
        let name = self.consume_identifier()?;
        let mut bits = false;
        let mut singular = false;
        loop {
            match self.peek() {
                Token::Identifier(ref s) if s.eq_ignore_ascii_case("BITS") => {
                    self.advance();
                    bits = true;
                }
                Token::Identifier(ref s) if s.eq_ignore_ascii_case("SINGULAR") => {
                    self.advance();
                    singular = true;
                }
                _ => break,
            }
        }
        self.consume_to_eol();
        self.skip_eol();

        let mut items = Vec::new();
        loop {
            if self.peek() == &Token::End {
                self.advance();
                if self.peek() == &Token::Enum {
                    self.advance();
                }
                self.consume_to_eol();
                break;
            }
            if self.at_end() {
                break;
            }
            let item_name = match self.peek().clone() {
                Token::PercentConstant(s) => {
                    self.advance();
                    s
                }
                Token::Identifier(s) => {
                    self.advance();
                    s
                }
                _ => {
                    self.consume_to_eol();
                    self.skip_eol();
                    continue;
                }
            };
            let value = if self.peek() == &Token::Eq {
                self.advance();
                let v = self.parse_expression()?;
                Some(v)
            } else {
                None
            };
            items.push(EnumItem {
                name: item_name,
                value,
            });
            self.consume_to_eol();
            self.skip_eol();
        }

        Ok(EnumDecl {
            name,
            items,
            line,
            bits,
            singular,
        })
    }

    /// A METHOD block inside a CLASS: `METHOD Name [(params)] [AS type] ... END METHOD`.
    /// It leaves this branch as an ordinary SUB/FUNCTION named `<Class>_<Method>`, so
    /// codegen needs no new machinery at all (register_sub / register_function exist).
    fn parse_method_decl(&mut self, prefix: Option<&str>) -> PbResult<TopLevel> {
        let line = self.current_line();
        self.advance(); // METHOD
        let mname = self.consume_identifier()?;

        // Optional CDECL/STDCALL/BDECL decoration (ignored, same as parse_sub_decl).
        while matches!(self.peek(), Token::Identifier(ref s) if {
            let u = s.to_uppercase();
            u == "CDECL" || u == "STDCALL" || u == "BDECL"
        }) {
            self.advance();
        }

        let params = if self.peek() == &Token::LParen {
            self.advance();
            let p = self.parse_params()?;
            self.expect(&Token::RParen)?;
            p
        } else {
            Vec::new()
        };

        // A METHOD declared inside a CLASS takes an implicit BYREF receiver as its
        // first parameter (batch 216).  Two things depend on it:
        //   * a bare INSTANCE field name in the body means `this.<field>` -- before
        //     this, such a name missed the symbol table, fell through to the
        //     auto-declare path and silently became a fresh zero-initialised local,
        //     so a method could never change the object it was called on;
        //   * the caller hands over the object, so two objects keep separate state.
        // PowerBASIC's parameter default is BYREF, and that is what makes the receiver
        // an address instead of a copy: with `is_byval: false` codegen types the
        // parameter as a pointer while keeping the declared CLASS type, which is exactly
        // what `this.<field>` needs.  A top-level METHOD (no CLASS) gets no receiver.
        let mut params = params;
        if let Some(cls) = prefix {
            params.insert(
                0,
                Param {
                    name: "this".to_string(),
                    pb_type: PbType::UserDefined(cls.to_string()),
                    is_byval: false,
                    is_optional: false,
                    is_array: false,
                },
            );
        }

        let return_type = if self.peek() == &Token::As {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        let export = if self.peek() == &Token::Export {
            self.advance();
            true
        } else {
            false
        };

        self.consume_to_eol();
        let (body, method_end) =
            self.parse_body_with_terminator(&[BodyEnd::EndMethod, BodyEnd::EndSub])?;
        // A body that never matched its terminator was cut short by parse_body()'s
        // safety net: the END METHOD is missing.  Say so instead of letting the rest of
        // the file be swallowed and reported later as a missing entry point (batch 214).
        if method_end.is_none() {
            eprintln!(
                "Error: METHOD block starting on line {} has no END METHOD terminator",
                line
            );
            self.error_count += 1;
        }
        let name = match prefix {
            Some(p) => format!("{}_{}", p, mname),
            None => mname,
        };

        Ok(match return_type {
            Some(rt) => TopLevel::FunctionDecl(FunctionDecl {
                name,
                params,
                return_type: rt,
                body,
                alias: None,
                export,
                line,
            }),
            None => TopLevel::SubDecl(SubDecl {
                name,
                params,
                body,
                alias: None,
                export,
                line,
            }),
        })
    }

    /// A PROPERTY block inside a CLASS (CHW PROPERTY/END PROPERTY): both forms
    /// are lowered like METHOD — an ordinary SUB/FUNCTION with the class-name
    /// prefix and the implicit BYREF receiver — so codegen needs no new
    /// machinery.  The GET form keeps a return type and is compiled as
    /// `<Class>_<Name>_Get`; the SET form (no return type) is compiled as
    /// `<Class>_<Name>_Set`.  The caller's syntax picks which one runs:
    /// `x = o.Prop(...)` resolves to the Get, `o.Prop(...) = expr` to the Set.
    fn parse_property_decl(&mut self, prefix: Option<&str>) -> PbResult<TopLevel> {
        let line = self.current_line();
        self.advance(); // PROPERTY
        let is_get = matches!(self.peek(), Token::Identifier(k) if k.eq_ignore_ascii_case("GET"));
        self.advance(); // GET / SET
        let pname = self.consume_identifier()?;

        let params = if self.peek() == &Token::LParen {
            self.advance();
            let p = self.parse_params()?;
            self.expect(&Token::RParen)?;
            p
        } else {
            Vec::new()
        };

        // THREADSAFE decoration (ignored, same as the METHOD form).
        while matches!(self.peek(), Token::Identifier(ref s) if {
            let u = s.to_uppercase();
            u == "THREADSAFE"
        }) {
            self.advance();
        }

        let mut params = params;
        if let Some(cls) = prefix {
            params.insert(
                0,
                Param {
                    name: "this".to_string(),
                    pb_type: PbType::UserDefined(cls.to_string()),
                    is_byval: false,
                    is_optional: false,
                    is_array: false,
                },
            );
        }

        let return_type = if self.peek() == &Token::As {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        self.consume_to_eol();
        let (body, prop_end) =
            self.parse_body_with_terminator(&[BodyEnd::EndProperty, BodyEnd::EndSub])?;
        if prop_end.is_none() {
            eprintln!(
                "Error: PROPERTY block starting on line {} has no END PROPERTY terminator",
                line
            );
            self.error_count += 1;
        }
        let name = match prefix {
            Some(p) => format!("{}_{}_{}", p, pname, if is_get { "Get" } else { "Set" }),
            None => format!("{}_{}", pname, if is_get { "Get" } else { "Set" }),
        };

        Ok(match return_type {
            Some(rt) => TopLevel::FunctionDecl(FunctionDecl {
                name,
                params,
                return_type: rt,
                body,
                alias: None,
                export: false,
                line,
            }),
            None => TopLevel::SubDecl(SubDecl {
                name,
                params,
                body,
                alias: None,
                export: false,
                line,
            }),
        })
    }

    /// batch 234: parse an official PB coordinate group used by XPRINT drawing
    /// statements:  `(x1, y1)` -> [x1, y1] ; `(x1, y1)-(x2, y2)` -> [x1, y1, x2, y2] ;
    /// a bare expression -> [expr] (back-compat with this fork's older examples).
    fn parse_xprint_pt(&mut self) -> PbResult<Vec<Expr>> {
        if self.peek() == &Token::LParen {
            self.advance();
            let x = self.parse_expression()?;
            self.expect(&Token::Comma)?;
            let y = self.parse_expression()?;
            self.expect(&Token::RParen)?;
            let mut v = vec![x, y];
            if self.peek() == &Token::Minus && self.peek_at(1) == Some(&Token::LParen) {
                self.advance(); // -
                self.advance(); // (
                let x2 = self.parse_expression()?;
                self.expect(&Token::Comma)?;
                let y2 = self.parse_expression()?;
                self.expect(&Token::RParen)?;
                v.push(x2);
                v.push(y2);
            }
            Ok(v)
        } else {
            Ok(vec![self.parse_expression()?])
        }
    }

    fn parse_sub_decl(&mut self) -> PbResult<SubDecl> {
        let line = self.current_line();
        self.advance(); // SUB
        let name = self.consume_identifier()?;

        // Optional: CALLBACK keyword after name (treat as decoration)
        if self.peek() == &Token::Callback {
            self.advance();
        }

        // Optional: CDECL/STDCALL calling convention (skip)
        while matches!(self.peek(), Token::Identifier(ref s) if {
            let u = s.to_uppercase();
            u == "CDECL" || u == "STDCALL" || u == "BDECL"
        }) {
            self.advance();
        }

        // Optional ALIAS "ExternalName"
        let alias = if self.peek() == &Token::Alias {
            self.advance();
            match self.peek().clone() {
                Token::StringLiteral(s) => {
                    self.advance();
                    Some(s)
                }
                _ => None,
            }
        } else {
            None
        };

        let params = if self.peek() == &Token::LParen {
            self.advance();
            let p = self.parse_params()?;
            self.expect(&Token::RParen)?;
            p
        } else {
            Vec::new()
        };

        // Optional COMMON / EXPORT descriptors (SLL sharing, CHW
        // "Creating a Static Link Library"): both may appear, in any order,
        // e.g. `SUB xxx() COMMON EXPORT`. fork links statically, so the
        // descriptors are accepted and have no behavioural difference.
        let mut export = false;
        while matches!(self.peek(), Token::Common | Token::Export) {
            if self.peek() == &Token::Export {
                export = true;
            }
            self.advance();
        }

        self.consume_to_eol();
        let body = self.parse_body(&[BodyEnd::EndSub, BodyEnd::EndMethod])?;

        Ok(SubDecl {
            name,
            params,
            body,
            alias,
            export,
            line,
        })
    }

    fn parse_fastproc_decl(&mut self) -> PbResult<FunctionDecl> {
        let line = self.current_line();
        // FASTPROC already consumed by the caller.  CHW
        // FASTPROC_END_FASTPROC_statements.htm: the header syntax (name,
        // optional THREADSAFE, optional AS LONG return) matches FUNCTION, so
        // reuse the FUNCTION header parser after peeling the FASTPROC keyword
        // (the CALLBACK/THREAD peel pattern).  Body terminator: END FASTPROC
        // (END FUNCTION accepted too).
        let name = self.consume_identifier()?;

        while matches!(self.peek(), Token::Identifier(ref s) if {
            let u = s.to_uppercase();
            u == "CDECL" || u == "STDCALL" || u == "BDECL"
        }) {
            self.advance();
        }

        // Optional THREADSAFE descriptor (CHW FASTPROC header)
        while matches!(self.peek(), Token::Identifier(ref s) if s.eq_ignore_ascii_case("THREADSAFE"))
        {
            self.advance();
        }

        let alias = if self.peek() == &Token::Alias {
            self.advance();
            match self.peek().clone() {
                Token::StringLiteral(s) => {
                    self.advance();
                    Some(s)
                }
                _ => None,
            }
        } else {
            None
        };

        let params = if self.peek() == &Token::LParen {
            self.advance();
            let p = if self.peek() == &Token::RParen {
                Vec::new()
            } else {
                self.parse_params()?
            };
            self.expect(&Token::RParen)?;
            p
        } else {
            Vec::new()
        };

        let mut export = false;
        while matches!(self.peek(), Token::Common | Token::Export) {
            if self.peek() == &Token::Export {
                export = true;
            }
            self.advance();
        }

        let return_type = if self.peek() == &Token::As {
            self.advance();
            self.parse_type()?
        } else {
            type_from_suffix(&name)
        };

        self.consume_to_eol();
        self.current_function_name = Some(name.to_uppercase());
        let body = self.parse_body(&[BodyEnd::EndFastProc, BodyEnd::EndFunction])?;
        self.current_function_name = None;

        Ok(FunctionDecl {
            name,
            params,
            return_type,
            body,
            alias,
            export,
            line,
        })
    }

    fn parse_function_decl(&mut self) -> PbResult<FunctionDecl> {
        let line = self.current_line();
        self.advance(); // FUNCTION
        let name = self.consume_identifier()?;

        // Optional: CDECL/STDCALL calling convention (skip)
        while matches!(self.peek(), Token::Identifier(ref s) if {
            let u = s.to_uppercase();
            u == "CDECL" || u == "STDCALL" || u == "BDECL"
        }) {
            self.advance();
        }

        // Optional ALIAS "ExternalName"
        let alias = if self.peek() == &Token::Alias {
            self.advance();
            match self.peek().clone() {
                Token::StringLiteral(s) => {
                    self.advance();
                    Some(s)
                }
                _ => None,
            }
        } else {
            None
        };

        let params = if self.peek() == &Token::LParen {
            self.advance();
            let p = if self.peek() == &Token::RParen {
                Vec::new()
            } else {
                self.parse_params()?
            };
            self.expect(&Token::RParen)?;
            p
        } else {
            Vec::new()
        };

        // Optional COMMON / EXPORT descriptors — come BEFORE the AS return
        // type in PB: `FUNCTION Foo (x AS LONG) COMMON EXPORT AS STRING`
        let mut export = false;
        while matches!(self.peek(), Token::Common | Token::Export) {
            if self.peek() == &Token::Export {
                export = true;
            }
            self.advance();
        }

        let return_type = if self.peek() == &Token::As {
            self.advance();
            self.parse_type()?
        } else {
            type_from_suffix(&name)
        };

        self.consume_to_eol();
        self.current_function_name = Some(name.to_uppercase());
        let body = self.parse_body(&[BodyEnd::EndFunction])?;
        self.current_function_name = None;

        Ok(FunctionDecl {
            name,
            params,
            return_type,
            body,
            alias,
            export,
            line,
        })
    }

    fn parse_params(&mut self) -> PbResult<Vec<Param>> {
        let mut params = Vec::new();
        loop {
            if self.peek() == &Token::RParen || self.at_end() {
                break;
            }

            let mut is_byval = false;
            let mut is_optional = false;

            // OPTIONAL, BYVAL, BYREF modifiers
            loop {
                match self.peek() {
                    Token::Optional => {
                        is_optional = true;
                        self.advance();
                    }
                    Token::Byval => {
                        is_byval = true;
                        self.advance();
                    }
                    Token::Byref => {
                        self.advance(); // BYREF is default
                    }
                    _ => break,
                }
            }

            let name = self.consume_identifier()?;
            let is_array = if self.peek() == &Token::LParen {
                self.advance();
                self.expect(&Token::RParen)?;
                true
            } else {
                false
            };

            let pb_type = if self.peek() == &Token::As {
                self.advance();
                self.parse_type()?
            } else {
                type_from_suffix(&name)
            };

            params.push(Param {
                name,
                pb_type,
                is_byval,
                is_optional,
                is_array,
            });

            if self.peek() == &Token::Comma {
                self.advance();
            } else {
                break;
            }
        }
        Ok(params)
    }

    fn parse_declare_stmt(&mut self, line: usize) -> PbResult<Option<DeclareStmt>> {
        self.advance(); // consume DECLARE

        // Optional CALLBACK keyword (treat as decoration)
        if self.peek() == &Token::Callback {
            self.advance();
        }

        // SUB or FUNCTION
        let is_function = match self.peek() {
            Token::Function => {
                self.advance();
                true
            }
            Token::Sub => {
                self.advance();
                false
            }
            _ => {
                self.consume_to_eol();
                return Ok(None);
            }
        };

        let name = self.consume_identifier()?;

        // Optional: CDECL, STDCALL, etc. — skip any identifier that's a calling convention
        while matches!(self.peek(), Token::Identifier(ref s) if {
            let u = s.to_uppercase();
            u == "CDECL" || u == "STDCALL" || u == "BDECL"
        }) {
            self.advance();
        }

        // Optional LIB "dll.dll"
        let lib = if self.peek() == &Token::Lib {
            self.advance();
            match self.peek().clone() {
                Token::StringLiteral(s) => {
                    self.advance();
                    Some(s)
                }
                _ => None,
            }
        } else {
            None
        };

        // Optional ALIAS "ExternalName"
        let alias = if self.peek() == &Token::Alias {
            self.advance();
            match self.peek().clone() {
                Token::StringLiteral(s) => {
                    self.advance();
                    Some(s)
                }
                _ => None,
            }
        } else {
            None
        };

        // Optional parameter list
        let params = if self.peek() == &Token::LParen {
            self.advance();
            let p = if self.peek() == &Token::RParen {
                Vec::new()
            } else {
                self.parse_params()?
            };
            self.expect(&Token::RParen)?;
            p
        } else {
            Vec::new()
        };

        // Optional COMMON / EXPORT descriptors (SLL sharing) — come BEFORE
        // the AS return type: `DECLARE FUNCTION MyFunc(Parm AS LONG) COMMON
        // AS DOUBLE`. Accepted and ignored (fork links statically).
        while matches!(self.peek(), Token::Common | Token::Export) {
            self.advance();
        }

        // Optional AS type (for FUNCTION)
        let return_type = if is_function && self.peek() == &Token::As {
            self.advance();
            self.parse_type()?
        } else if is_function {
            type_from_suffix(&name)
        } else {
            PbType::Variant // SUB has no return type
        };

        self.consume_to_eol();

        Ok(Some(DeclareStmt {
            name,
            is_function,
            params,
            return_type,
            lib,
            alias,
            line,
        }))
    }

    // ===== Body parsing (inside SUB/FUNCTION) =====

    fn parse_body(&mut self, terminators: &[BodyEnd]) -> PbResult<Vec<Statement>> {
        self.parse_body_with_terminator(terminators)
            .map(|(stmts, _)| stmts)
    }

    /// Like parse_body but also returns which terminator was matched (if any).
    fn parse_body_with_terminator(
        &mut self,
        terminators: &[BodyEnd],
    ) -> PbResult<(Vec<Statement>, Option<BodyEnd>)> {
        let mut stmts = Vec::new();
        self.skip_eol();

        let mut matched_terminator = None;

        loop {
            if self.at_end() {
                break;
            }

            // Check for terminators
            if let Some(t) = self.which_terminator(terminators) {
                if let Some(ret) = self.consume_terminator(terminators) {
                    stmts.push(ret);
                }
                matched_terminator = Some(t);
                break;
            }

            self.skip_eol();
            if self.at_end() {
                break;
            }
            if let Some(t) = self.which_terminator(terminators) {
                if let Some(ret) = self.consume_terminator(terminators) {
                    stmts.push(ret);
                }
                matched_terminator = Some(t);
                break;
            }

            // Safety: break on END SUB/END FUNCTION even if not in our terminator
            // list — this indicates we've overrun into a parent scope (e.g., a CASE
            // body encountering the enclosing function's END SUB).
            if self.peek() == &Token::End {
                match self.peek_at(1) {
                    Some(&Token::Sub) | Some(&Token::Function) | Some(&Token::FastProc)
                        if !terminators.iter().any(|t| {
                            matches!(
                                t,
                                BodyEnd::EndSub | BodyEnd::EndFunction | BodyEnd::EndFastProc
                            )
                        }) =>
                    {
                        break; // don't consume, let parent handle it
                    }
                    _ => {}
                }
            }

            // Safety: if we see a top-level declaration keyword that shouldn't
            // be inside a body, break out to avoid consuming the rest of the file.
            if self.is_top_level_keyword() {
                break;
            }

            let saved_pos = self.pos;
            match self.parse_statement() {
                Ok(stmt) => stmts.push(stmt),
                Err(e) => {
                    eprintln!("Error: Parse error: {}", e);
                    self.error_count += 1;
                    self.consume_to_eol();
                }
            }

            self.skip_eol();

            // Safety: if no progress was made, force advance to prevent infinite loops.
            if self.pos == saved_pos {
                self.advance();
            }
        }

        Ok((stmts, matched_terminator))
    }

    /// Check if current position looks like a top-level declaration that shouldn't be inside a body
    fn is_top_level_keyword(&self) -> bool {
        match self.peek() {
            Token::Sub => {
                // SUB followed by identifier = new sub declaration
                matches!(self.peek_at(1), Some(Token::Identifier(_)))
            }
            Token::Function => {
                // FUNCTION followed by identifier (not =) = new function declaration
                match self.peek_at(1) {
                    Some(Token::Eq) => false,           // FUNCTION = expr (return value)
                    Some(Token::Identifier(_)) => true, // New function declaration
                    _ => false,
                }
            }
            Token::FastProc => {
                // FASTPROC followed by identifier = new fastproc declaration
                matches!(self.peek_at(1), Some(Token::Identifier(_)))
            }
            Token::Declare => true,
            Token::Type => {
                // TYPE at start of line (new type decl, not inside body).
                // TYPE SET (statement) is NOT a top-level keyword.
                match self.peek_at(1) {
                    Some(Token::Identifier(w)) if w.to_uppercase() == "SET" => false,
                    Some(Token::Identifier(_)) => true,
                    _ => false,
                }
            }
            _ => false,
        }
    }

    fn which_terminator(&self, terminators: &[BodyEnd]) -> Option<BodyEnd> {
        for t in terminators {
            if self.matches_terminator(t) {
                return Some(*t);
            }
        }
        None
    }

    fn matches_terminator(&self, term: &BodyEnd) -> bool {
        match term {
            BodyEnd::EndMethod => {
                self.peek() == &Token::End
                    && matches!(self.peek_at(1), Some(Token::Identifier(w))
                                if w.eq_ignore_ascii_case("METHOD"))
            }
            BodyEnd::EndSub => self.peek() == &Token::End && self.peek_at(1) == Some(&Token::Sub),
            BodyEnd::EndFunction => {
                self.peek() == &Token::End && self.peek_at(1) == Some(&Token::Function)
            }
            BodyEnd::EndFastProc => {
                self.peek() == &Token::End && self.peek_at(1) == Some(&Token::FastProc)
            }
            BodyEnd::EndProperty => {
                self.peek() == &Token::End
                    && matches!(self.peek_at(1), Some(Token::Identifier(w))
                                if w.eq_ignore_ascii_case("PROPERTY"))
            }
            BodyEnd::EndIf => self.peek() == &Token::End && self.peek_at(1) == Some(&Token::If),
            BodyEnd::Else => self.peek() == &Token::Else,
            BodyEnd::ElseIf => self.peek() == &Token::ElseIf,
            BodyEnd::Next => self.peek() == &Token::Next,
            BodyEnd::Loop => self.peek() == &Token::Loop,
            BodyEnd::Wend => self.peek() == &Token::Wend,
            BodyEnd::EndSelect => {
                self.peek() == &Token::End && self.peek_at(1) == Some(&Token::Select)
            }
            BodyEnd::Case => self.peek() == &Token::Case,
            BodyEnd::CaseElse => {
                self.peek() == &Token::Case && self.peek_at(1) == Some(&Token::Else)
            }
        }
    }

    fn consume_terminator(&mut self, terminators: &[BodyEnd]) -> Option<Statement> {
        for t in terminators {
            if self.matches_terminator(t) {
                match t {
                    BodyEnd::EndSub
                    | BodyEnd::EndFunction
                    | BodyEnd::EndIf
                    | BodyEnd::EndMethod
                    | BodyEnd::EndProperty => {
                        self.advance(); // END
                        self.advance(); // SUB/FUNCTION/IF/METHOD/PROPERTY
                        self.consume_to_eol();
                        return None;
                    }
                    BodyEnd::EndFastProc => {
                        self.advance(); // END
                        self.advance(); // FASTPROC
                                        // CHW FASTPROC_END_FASTPROC_statements.htm: the return
                                        // value of a FastProc is given by
                                        // `END FASTPROC [= ReturnValue]` (NOT by assigning the
                                        // procedure name in the body, which PBWin rejects with
                                        // error 466).  Parse `= expr` into a FunctionReturn.
                        let line = self.current_line();
                        if self.peek() == &Token::Eq {
                            self.advance();
                            let value = self.parse_expression().ok()?;
                            self.consume_to_eol();
                            return Some(Statement::FunctionReturn(FunctionReturnStmt {
                                value,
                                line,
                            }));
                        }
                        self.consume_to_eol();
                        return None;
                    }
                    BodyEnd::EndSelect
                    | BodyEnd::Else
                    | BodyEnd::ElseIf
                    | BodyEnd::Next
                    | BodyEnd::Loop
                    | BodyEnd::Wend
                    | BodyEnd::Case
                    | BodyEnd::CaseElse => {
                        // Don't consume — caller needs to see it
                    }
                }
                return None;
            }
        }
        None
    }

    fn peek_at(&self, offset: usize) -> Option<&Token> {
        self.tokens.get(self.pos + offset).map(|t| &t.token)
    }

    fn parse_data_items(&mut self) -> Vec<DataItem> {
        let mut items = Vec::new();
        loop {
            match self.peek().clone() {
                Token::StringLiteral(s) => {
                    items.push(DataItem {
                        text: s,
                        quoted: true,
                    });
                    self.advance();
                }
                Token::IntegerLiteral(n) => {
                    items.push(DataItem {
                        text: n.to_string(),
                        quoted: false,
                    });
                    self.advance();
                }
                Token::FloatLiteral(f) => {
                    items.push(DataItem {
                        text: f.to_string(),
                        quoted: false,
                    });
                    self.advance();
                }
                Token::Identifier(s) => {
                    items.push(DataItem {
                        text: s,
                        quoted: false,
                    });
                    self.advance();
                }
                Token::Minus => {
                    self.advance();
                    match self.peek().clone() {
                        Token::IntegerLiteral(n) => {
                            items.push(DataItem {
                                text: format!("-{}", n),
                                quoted: false,
                            });
                            self.advance();
                        }
                        Token::FloatLiteral(f) => {
                            items.push(DataItem {
                                text: format!("-{}", f),
                                quoted: false,
                            });
                            self.advance();
                        }
                        _ => items.push(DataItem {
                            text: "-".to_string(),
                            quoted: false,
                        }),
                    }
                }
                _ => break,
            }
            if self.at_eol_or_eof() {
                break;
            }
            if self.peek() == &Token::Comma {
                self.advance();
            } else {
                break;
            }
        }
        items
    }

    /// LISTVIEW <sub-command> ...  -- batch 158, completed in batch 170
    ///
    /// Syntax per the official PowerBASIC documentation
    /// (LISTVIEW_statement.htm). Every documented sub-command is handled
    /// here. Item and column numbers are 1-based (First = 1), exactly as the
    /// official help specifies; the runtime converts to the 0-based indices
    /// Win32 expects.
    ///
    ///   LISTVIEW DELETE COLUMN hDlg, id&, col&
    ///   LISTVIEW DELETE ITEM   hDlg, id&, item&
    ///   LISTVIEW FIND          hDlg, id&, item&, StrExpr TO datav&
    ///   LISTVIEW FIND EXACT    hDlg, id&, item&, StrExpr TO datav&
    ///   LISTVIEW FIT CONTENT   hDlg, id&, col&
    ///   LISTVIEW FIT HEADER    hDlg, id&, col&
    ///   LISTVIEW GET COLUMN    hDlg, id&, col& TO datav&
    ///   LISTVIEW GET COUNT     hDlg, id& TO datav&
    ///   LISTVIEW GET HEADER    hDlg, id&, col& TO txtv$
    ///   LISTVIEW GET HEADERID  hDlg, id& TO hLV, idv&
    ///   LISTVIEW GET MODE      hDlg, id& TO datav&
    ///   LISTVIEW GET SELCOUNT  hDlg, id& TO datav&
    ///   LISTVIEW GET SELECT    hDlg, id& [, item&] TO datav&
    ///   LISTVIEW GET STATE     hDlg, id&, item&, col& TO datav&
    ///   LISTVIEW GET STYLEXX   hDlg, id& TO datav&
    ///   LISTVIEW GET TEXT      hDlg, id&, item&, col& TO txtv$
    ///   LISTVIEW GET USER      hDlg, id&, item& TO datav&
    ///   LISTVIEW INSERT COLUMN hDlg, id&, col&, StrExpr, ColWidth&, format&
    ///   LISTVIEW INSERT ITEM   hDlg, id&, item&, image&, StrExpr
    ///   LISTVIEW RESET         hDlg, id&
    ///   LISTVIEW SELECT        hDlg, id&, item& [, col&]
    ///   LISTVIEW SET COLUMN    hDlg, id&, col&, NumExpr
    ///   LISTVIEW SET HEADER    hDlg, id&, col&, StrExpr
    ///   LISTVIEW SET IMAGE     hDlg, id&, item&, NumExpr
    ///   LISTVIEW SET IMAGE2    hDlg, id&, item&, NumExpr
    ///   LISTVIEW SET IMAGELIST hDlg, id&, hLst, NumExpr
    ///   LISTVIEW SET MODE      hDlg, id&, NumExpr
    ///   LISTVIEW SET OVERLAY   hDlg, id&, item&, NumExpr
    ///   LISTVIEW SET STYLEXX   hDlg, id&, NumExpr
    ///   LISTVIEW SET TEXT      hDlg, id&, item&, col&, StrExpr
    ///   LISTVIEW SET USER      hDlg, id&, item&, NumExpr
    ///   LISTVIEW SORT          hDlg, id&, col& [, options...]
    ///   LISTVIEW UNSELECT      hDlg, id&, item& [, col&]
    ///   LISTVIEW VISIBLE       hDlg, id&, item&
    ///
    /// SELECT is lexed as the reserved token Token::Select, not an
    /// identifier, so both the verb and the noun lookups go through
    /// peek_word_upper() instead of peek_plain_upper() - the same trap that
    /// silently broke COMBOBOX SELECT in batch 168 and TAB SELECT in 169.
    fn parse_listview_statement(&mut self, line: usize) -> PbResult<Statement> {
        self.advance(); // consume LISTVIEW
        let sub = self.peek_word_upper();
        match sub.as_str() {
            "RESET" => {
                self.advance();
                let h = self.parse_expression()?;
                self.expect(&Token::Comma)?;
                let id = self.parse_expression()?;
                self.consume_to_eol();
                Ok(Statement::Call(CallStmt {
                    name: "LISTVIEW_RESET".to_string(),
                    args: vec![h, id],
                    line,
                }))
            }
            "SELECT" => {
                self.advance();
                let h = self.parse_expression()?;
                self.expect(&Token::Comma)?;
                let id = self.parse_expression()?;
                self.expect(&Token::Comma)?;
                let item = self.parse_expression()?;
                // The optional col& defaults to the primary data item.
                let col = if self.peek() == &Token::Comma {
                    self.advance();
                    self.parse_expression()?
                } else {
                    Expr::IntegerLit(1)
                };
                self.consume_to_eol();
                Ok(Statement::Call(CallStmt {
                    name: "LISTVIEW_SELECT".to_string(),
                    args: vec![h, id, item, col],
                    line,
                }))
            }
            "UNSELECT" => {
                self.advance();
                let h = self.parse_expression()?;
                self.expect(&Token::Comma)?;
                let id = self.parse_expression()?;
                self.expect(&Token::Comma)?;
                let item = self.parse_expression()?;
                let col = if self.peek() == &Token::Comma {
                    self.advance();
                    self.parse_expression()?
                } else {
                    Expr::IntegerLit(1)
                };
                self.consume_to_eol();
                Ok(Statement::Call(CallStmt {
                    name: "LISTVIEW_UNSELECT".to_string(),
                    args: vec![h, id, item, col],
                    line,
                }))
            }
            "VISIBLE" => {
                self.advance();
                let h = self.parse_expression()?;
                self.expect(&Token::Comma)?;
                let id = self.parse_expression()?;
                self.expect(&Token::Comma)?;
                let item = self.parse_expression()?;
                self.consume_to_eol();
                Ok(Statement::Call(CallStmt {
                    name: "LISTVIEW_VISIBLE".to_string(),
                    args: vec![h, id, item],
                    line,
                }))
            }
            "SORT" => {
                self.advance();
                let h = self.parse_expression()?;
                self.expect(&Token::Comma)?;
                let id = self.parse_expression()?;
                self.expect(&Token::Comma)?;
                let col = self.parse_expression()?;
                // The options are comma-delimited bare keywords describing
                // the sequence and the nature of the sort-key column.  They
                // are folded into one bitmask here; the encoding is shared
                // with PB_LVSORT_* in pb_runtime.c and the two must change
                // together.  An unrecognised option is a hard error: it
                // would otherwise silently change the sort semantics.
                let mut mode: i64 = 0;
                while self.peek() == &Token::Comma {
                    self.advance();
                    let opt = self.peek_word_upper();
                    let bit: i64 = match opt.as_str() {
                        "ASCEND" => 0x0001,
                        "DESCEND" => 0x0002,
                        "ALPHANUM" => 0x0004,
                        "UCASE" => 0x0008,
                        "NUMERIC" => 0x0010,
                        "MMDDYYYY" => 0x0020,
                        "DDMMYYYY" => 0x0040,
                        "YYYYMMDD" => 0x0080,
                        "YYYYDDMM" => 0x0100,
                        _ => {
                            return Err(PbError::parser(
                                format!(
                                    "LISTVIEW SORT: unknown option '{}' (expected one of \
                                     ASCEND, DESCEND, ALPHANUM, UCASE, NUMERIC, MMDDYYYY, \
                                     DDMMYYYY, YYYYMMDD, YYYYDDMM)",
                                    opt
                                ),
                                None,
                                line,
                            ))
                        }
                    };
                    self.advance();
                    mode |= bit;
                }
                // Ascending order is the documented default.
                if mode == 0 {
                    mode = 0x0001;
                }
                self.consume_to_eol();
                Ok(Statement::Call(CallStmt {
                    name: "LISTVIEW_SORT".to_string(),
                    args: vec![h, id, col, Expr::IntegerLit(mode)],
                    line,
                }))
            }
            "FIND" => {
                self.advance();
                // Only `FIND EXACT` carries a noun; in the plain form the very
                // next token is already the hDlg operand, so the noun must not
                // be consumed unless it really is EXACT.
                let exact = self.peek_word_upper() == "EXACT";
                if exact {
                    self.advance();
                }
                let h = self.parse_expression()?;
                self.expect(&Token::Comma)?;
                let id = self.parse_expression()?;
                self.expect(&Token::Comma)?;
                let item = self.parse_expression()?;
                self.expect(&Token::Comma)?;
                let needle = self.parse_expression()?;
                if self.peek() == &Token::To {
                    self.advance();
                }
                let target = self.parse_expression()?;
                self.consume_to_eol();
                let name = if exact {
                    "LISTVIEW_FIND_EXACT"
                } else {
                    "LISTVIEW_FIND"
                };
                Ok(Statement::Call(CallStmt {
                    name: name.to_string(),
                    args: vec![h, id, item, needle, target],
                    line,
                }))
            }
            "FIT" => {
                self.advance();
                let what = self.peek_word_upper();
                self.advance();
                if what == "CONTENT" || what == "HEADER" {
                    let h = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let col = self.parse_expression()?;
                    self.consume_to_eol();
                    let name = if what == "CONTENT" {
                        "LISTVIEW_FIT_CONTENT"
                    } else {
                        "LISTVIEW_FIT_HEADER"
                    };
                    return Ok(Statement::Call(CallStmt {
                        name: name.to_string(),
                        args: vec![h, id, col],
                        line,
                    }));
                }
                self.consume_to_eol();
                eprintln!("Error: LISTVIEW FIT: unsupported form on line {}", line);
                self.error_count += 1;
                Ok(Statement::Noop("LISTVIEW FIT".to_string(), line))
            }
            "GET" => {
                self.advance();
                let what = self.peek_word_upper();
                self.advance();
                match what.as_str() {
                    "COUNT" | "MODE" | "SELCOUNT" | "STYLEXX" => {
                        let h = self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                        let id = self.parse_expression()?;
                        if self.peek() == &Token::To {
                            self.advance();
                        }
                        let target = self.parse_expression()?;
                        self.consume_to_eol();
                        let name = format!("LISTVIEW_GET_{}", what);
                        return Ok(Statement::Call(CallStmt {
                            name,
                            args: vec![h, id, target],
                            line,
                        }));
                    }
                    "COLUMN" => {
                        let h = self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                        let id = self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                        let col = self.parse_expression()?;
                        if self.peek() == &Token::To {
                            self.advance();
                        }
                        let target = self.parse_expression()?;
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "LISTVIEW_GET_COLUMN".to_string(),
                            args: vec![h, id, col, target],
                            line,
                        }));
                    }
                    "HEADER" => {
                        let h = self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                        let id = self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                        let col = self.parse_expression()?;
                        if self.peek() == &Token::To {
                            self.advance();
                        }
                        let target = self.parse_expression()?;
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "LISTVIEW_GET_HEADER".to_string(),
                            args: vec![h, id, col, target],
                            line,
                        }));
                    }
                    "HEADERID" => {
                        // Two result variables: the embedded header control
                        // handle and its control id.
                        let h = self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                        let id = self.parse_expression()?;
                        if self.peek() == &Token::To {
                            self.advance();
                        }
                        let t1 = self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                        let t2 = self.parse_expression()?;
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "LISTVIEW_GET_HEADERID".to_string(),
                            args: vec![h, id, t1, t2],
                            line,
                        }));
                    }
                    "SELECT" => {
                        // The item is optional; the documented default is to
                        // search from the first item.
                        let h = self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                        let id = self.parse_expression()?;
                        let start = if self.peek() == &Token::Comma {
                            self.advance();
                            self.parse_expression()?
                        } else {
                            Expr::IntegerLit(1)
                        };
                        if self.peek() == &Token::To {
                            self.advance();
                        }
                        let target = self.parse_expression()?;
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "LISTVIEW_GET_SELECT".to_string(),
                            args: vec![h, id, start, target],
                            line,
                        }));
                    }
                    "STATE" => {
                        let h = self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                        let id = self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                        let item = self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                        let col = self.parse_expression()?;
                        if self.peek() == &Token::To {
                            self.advance();
                        }
                        let target = self.parse_expression()?;
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "LISTVIEW_GET_STATE".to_string(),
                            args: vec![h, id, item, col, target],
                            line,
                        }));
                    }
                    "TEXT" => {
                        let h = self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                        let id = self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                        let item = self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                        let col = self.parse_expression()?;
                        if self.peek() == &Token::To {
                            self.advance();
                        }
                        let target = self.parse_expression()?;
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "LISTVIEW_GET_TEXT".to_string(),
                            args: vec![h, id, item, col, target],
                            line,
                        }));
                    }
                    "USER" => {
                        let h = self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                        let id = self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                        let item = self.parse_expression()?;
                        if self.peek() == &Token::To {
                            self.advance();
                        }
                        let target = self.parse_expression()?;
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "LISTVIEW_GET_USER".to_string(),
                            args: vec![h, id, item, target],
                            line,
                        }));
                    }
                    _ => {}
                }
                self.consume_to_eol();
                eprintln!("Error: LISTVIEW GET: unsupported form on line {}", line);
                self.error_count += 1;
                Ok(Statement::Noop("LISTVIEW GET".to_string(), line))
            }
            "SET" => {
                self.advance();
                let what = self.peek_word_upper();
                self.advance();
                match what.as_str() {
                    "MODE" | "STYLEXX" => {
                        let h = self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                        let id = self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                        let value = self.parse_expression()?;
                        self.consume_to_eol();
                        let name = format!("LISTVIEW_SET_{}", what);
                        return Ok(Statement::Call(CallStmt {
                            name,
                            args: vec![h, id, value],
                            line,
                        }));
                    }
                    "COLUMN" => {
                        let h = self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                        let id = self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                        let col = self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                        let width = self.parse_expression()?;
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "LISTVIEW_SET_COLUMN".to_string(),
                            args: vec![h, id, col, width],
                            line,
                        }));
                    }
                    "HEADER" => {
                        let h = self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                        let id = self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                        let col = self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                        let text = self.parse_expression()?;
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "LISTVIEW_SET_HEADER".to_string(),
                            args: vec![h, id, col, text],
                            line,
                        }));
                    }
                    "TEXT" => {
                        let h = self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                        let id = self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                        let item = self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                        let col = self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                        let text = self.parse_expression()?;
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "LISTVIEW_SET_TEXT".to_string(),
                            args: vec![h, id, item, col, text],
                            line,
                        }));
                    }
                    "IMAGE" | "IMAGE2" | "OVERLAY" | "USER" => {
                        let h = self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                        let id = self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                        let item = self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                        let value = self.parse_expression()?;
                        self.consume_to_eol();
                        let name = format!("LISTVIEW_SET_{}", what);
                        return Ok(Statement::Call(CallStmt {
                            name,
                            args: vec![h, id, item, value],
                            line,
                        }));
                    }
                    "IMAGELIST" => {
                        let h = self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                        let id = self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                        let hlst = self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                        let which = self.parse_expression()?;
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "LISTVIEW_SET_IMAGELIST".to_string(),
                            args: vec![h, id, hlst, which],
                            line,
                        }));
                    }
                    _ => {}
                }
                self.consume_to_eol();
                eprintln!("Error: LISTVIEW SET: unsupported form on line {}", line);
                self.error_count += 1;
                Ok(Statement::Noop("LISTVIEW SET".to_string(), line))
            }
            "INSERT" => {
                self.advance();
                let what = self.peek_word_upper();
                self.advance();
                if what == "COLUMN" {
                    let h = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let col = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let text = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let width = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let fmt = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "LISTVIEW_INSERT_COLUMN".to_string(),
                        args: vec![h, id, col, text, width, fmt],
                        line,
                    }));
                }
                if what == "ITEM" {
                    let h = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let item = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let image = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let text = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "LISTVIEW_INSERT_ITEM".to_string(),
                        args: vec![h, id, item, image, text],
                        line,
                    }));
                }
                self.consume_to_eol();
                eprintln!("Error: LISTVIEW INSERT: unsupported form on line {}", line);
                self.error_count += 1;
                Ok(Statement::Noop("LISTVIEW INSERT".to_string(), line))
            }
            "DELETE" => {
                self.advance();
                let what = self.peek_word_upper();
                self.advance();
                if what == "ITEM" {
                    let h = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let item = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "LISTVIEW_DELETE_ITEM".to_string(),
                        args: vec![h, id, item],
                        line,
                    }));
                }
                if what == "COLUMN" {
                    let h = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let col = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "LISTVIEW_DELETE_COLUMN".to_string(),
                        args: vec![h, id, col],
                        line,
                    }));
                }
                self.consume_to_eol();
                eprintln!("Error: LISTVIEW DELETE: unsupported form on line {}", line);
                self.error_count += 1;
                Ok(Statement::Noop("LISTVIEW DELETE".to_string(), line))
            }
            _ => {
                let bad = self.peek_word_upper();
                eprintln!(
                    "Error: LISTVIEW: unknown sub-command \"{}\" on line {}",
                    bad, line
                );
                self.error_count += 1;
                self.consume_to_eol();
                Ok(Statement::Noop("LISTVIEW".to_string(), line))
            }
        }
    }
    /// TREEVIEW <sub-command> ...  -- batch 158, completed in batch 172
    ///
    /// Syntax and semantics per the official PowerBASIC `TREEVIEW statement`
    /// help page. Everything the page documents is implemented here:
    ///   TREEVIEW DELETE        hDlg, id&, hItem
    ///   TREEVIEW GET BOLD      hDlg, id&, hItem TO datav&
    ///   TREEVIEW GET CHECK     hDlg, id&, hItem TO datav&
    ///   TREEVIEW GET CHILD     hDlg, id&, hItem TO datav&
    ///   TREEVIEW GET COUNT     hDlg, id& TO datav&
    ///   TREEVIEW GET EXPANDED  hDlg, id&, hItem TO datav&
    ///   TREEVIEW GET NEXT      hDlg, id&, hItem TO datav&
    ///   TREEVIEW GET PARENT    hDlg, id&, hItem TO datav&
    ///   TREEVIEW GET PREVIOUS  hDlg, id&, hItem TO datav&
    ///   TREEVIEW GET ROOT      hDlg, id& TO datav&
    ///   TREEVIEW GET SELECT    hDlg, id& TO datav&
    ///   TREEVIEW GET TEXT      hDlg, id&, hItem TO txtv$
    ///   TREEVIEW GET USER      hDlg, id&, hItem TO datav&
    ///   TREEVIEW INSERT ITEM   hDlg, id&, hPrnt, hIAftr, image&, simage&, txt$ TO hItem
    ///   TREEVIEW RESET         hDlg, id&
    ///   TREEVIEW SELECT        hDlg, id&, hItem
    ///   TREEVIEW SET BOLD      hDlg, id&, hItem, flag&
    ///   TREEVIEW SET CHECK     hDlg, id&, hItem, flag&
    ///   TREEVIEW SET EXPANDED  hDlg, id&, hItem, flag&
    ///   TREEVIEW SET IMAGELIST hDlg, id&, hLst
    ///   TREEVIEW SET TEXT      hDlg, id&, hItem, txt$
    ///   TREEVIEW SET USER      hDlg, id&, hItem, NumExpr
    ///   TREEVIEW UNSELECT      hDlg, id&
    ///
    /// `SELECT` is lexed as `Token::Select`, so the verb and noun lookups go
    /// through `peek_word_upper()`: `peek_plain_upper()` does not report that
    /// token, and both `TREEVIEW SELECT` and `TREEVIEW GET SELECT` would fall
    /// through to statement-level expression parsing ("Unexpected token in
    /// expression: Select"). That is the batch 168 COMBOBOX defect, same shape.
    fn parse_treeview_statement(&mut self, line: usize) -> PbResult<Statement> {
        self.advance(); // consume TREEVIEW
        let sub = self.peek_treeview_word_upper();
        match sub.as_str() {
            "RESET" | "UNSELECT" => {
                self.advance();
                let name = format!("TREEVIEW_{}", sub);
                let h = self.parse_expression()?;
                self.expect(&Token::Comma)?;
                let id = self.parse_expression()?;
                self.consume_to_eol();
                Ok(Statement::Call(CallStmt {
                    name,
                    args: vec![h, id],
                    line,
                }))
            }
            "SELECT" => {
                self.advance();
                let h = self.parse_expression()?;
                self.expect(&Token::Comma)?;
                let id = self.parse_expression()?;
                self.expect(&Token::Comma)?;
                let item = self.parse_expression()?;
                self.consume_to_eol();
                Ok(Statement::Call(CallStmt {
                    name: "TREEVIEW_SELECT".to_string(),
                    args: vec![h, id, item],
                    line,
                }))
            }
            "GET" => {
                self.advance();
                let what = self.peek_treeview_word_upper();
                self.advance();
                // COUNT / ROOT / SELECT take no item handle; the other nine
                // selectors all name one specific item.
                let itemless = matches!(what.as_str(), "COUNT" | "ROOT" | "SELECT");
                let known = itemless
                    || matches!(
                        what.as_str(),
                        "BOLD"
                            | "CHECK"
                            | "CHILD"
                            | "EXPANDED"
                            | "NEXT"
                            | "PARENT"
                            | "PREVIOUS"
                            | "TEXT"
                            | "USER"
                    );
                if !known {
                    self.consume_to_eol();
                    eprintln!("Error: TREEVIEW GET: unsupported form on line {}", line);
                    self.error_count += 1;
                    return Ok(Statement::Noop("TREEVIEW GET".to_string(), line));
                }
                let h = self.parse_expression()?;
                self.expect(&Token::Comma)?;
                let id = self.parse_expression()?;
                let mut args = vec![h, id];
                if !itemless {
                    self.expect(&Token::Comma)?;
                    args.push(self.parse_expression()?);
                }
                if self.peek() == &Token::To {
                    self.advance();
                }
                args.push(self.parse_expression()?);
                self.consume_to_eol();
                Ok(Statement::Call(CallStmt {
                    name: format!("TREEVIEW_GET_{}", what),
                    args,
                    line,
                }))
            }
            "SET" => {
                self.advance();
                let what = self.peek_treeview_word_upper();
                self.advance();
                if !matches!(
                    what.as_str(),
                    "BOLD" | "CHECK" | "EXPANDED" | "IMAGELIST" | "TEXT" | "USER"
                ) {
                    self.consume_to_eol();
                    eprintln!("Error: TREEVIEW SET: unsupported form on line {}", line);
                    self.error_count += 1;
                    return Ok(Statement::Noop("TREEVIEW SET".to_string(), line));
                }
                let h = self.parse_expression()?;
                self.expect(&Token::Comma)?;
                let id = self.parse_expression()?;
                let mut args = vec![h, id];
                // SET IMAGELIST is the only form without an item handle.
                if what != "IMAGELIST" {
                    self.expect(&Token::Comma)?;
                    args.push(self.parse_expression()?);
                }
                self.expect(&Token::Comma)?;
                args.push(self.parse_expression()?);
                self.consume_to_eol();
                Ok(Statement::Call(CallStmt {
                    name: format!("TREEVIEW_SET_{}", what),
                    args,
                    line,
                }))
            }
            "INSERT" => {
                self.advance();
                let what = self.peek_treeview_word_upper();
                self.advance();
                if what == "ITEM" {
                    let h = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let hparent = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let hafter = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let image = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let simage = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let text = self.parse_expression()?;
                    if self.peek() == &Token::To {
                        self.advance();
                    }
                    let target = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "TREEVIEW_INSERT_ITEM".to_string(),
                        args: vec![h, id, hparent, hafter, image, simage, text, target],
                        line,
                    }));
                }
                self.consume_to_eol();
                eprintln!("Error: TREEVIEW INSERT: unsupported form on line {}", line);
                self.error_count += 1;
                Ok(Statement::Noop("TREEVIEW INSERT".to_string(), line))
            }
            "DELETE" => {
                self.advance();
                let h = self.parse_expression()?;
                self.expect(&Token::Comma)?;
                let id = self.parse_expression()?;
                self.expect(&Token::Comma)?;
                let item = self.parse_expression()?;
                self.consume_to_eol();
                Ok(Statement::Call(CallStmt {
                    name: "TREEVIEW_DELETE".to_string(),
                    args: vec![h, id, item],
                    line,
                }))
            }
            _ => {
                let bad = self.peek_word_upper();
                eprintln!(
                    "Error: TREEVIEW: unknown sub-command \"{}\" on line {}",
                    bad, line
                );
                self.error_count += 1;
                self.consume_to_eol();
                Ok(Statement::Noop("TREEVIEW".to_string(), line))
            }
        }
    }

    /// SCROLLBAR <GET|SET> <POS|RANGE|PAGESIZE|TRACKPOS> hDlg, id& [, ...] (batch 168)
    ///
    /// Official syntax (`SCROLLBAR_statement.htm`):
    ///   SCROLLBAR GET PAGESIZE hDlg, id& TO datav&
    ///   SCROLLBAR GET POS      hDlg, id& TO datav&
    ///   SCROLLBAR GET RANGE    hDlg, id& TO lo&, hi&
    ///   SCROLLBAR GET TRACKPOS hDlg, id& TO datav&
    ///   SCROLLBAR SET PAGESIZE hDlg, id&, page&
    ///   SCROLLBAR SET POS      hDlg, id&, pos&
    ///   SCROLLBAR SET RANGE    hDlg, id&, lo&, hi&
    ///
    /// TRACKPOS is GET-only - there is no official `SCROLLBAR SET TRACKPOS`,
    /// so the SET side is rejected below with a message that says so (batch 192).
    fn parse_scrollbar_statement(&mut self, line: usize) -> PbResult<Statement> {
        self.advance(); // SCROLLBAR
        let verb = self.peek_word_upper();
        if verb != "GET" && verb != "SET" {
            eprintln!(
                "Error: SCROLLBAR: expected GET or SET, found \"{}\" on line {}",
                verb, line
            );
            self.error_count += 1;
            self.consume_to_eol();
            return Ok(Statement::Noop("SCROLLBAR".to_string(), line));
        }
        self.advance(); // verb
        let noun = self.peek_word_upper();
        // TRACKPOS exists on the GET side only: the parser's own syntax note above
        // lists six official forms and `SCROLLBAR SET TRACKPOS` is not one of them,
        // there is no `SCROLLBAR SET TRACKPOS` row in docs/statement-coverage.csv,
        // and Win32 SIF_TRACKPOS is a read-only member.  Rejecting it here - instead
        // of letting it fall through to a codegen "unknown statement" - keeps the
        // diagnosis at the layer where the mistake was made.
        let noun_ok = match noun.as_str() {
            "POS" | "RANGE" | "PAGESIZE" => true,
            "TRACKPOS" => verb == "GET",
            _ => false,
        };
        if !noun_ok {
            if noun == "TRACKPOS" {
                eprintln!(
                    "Error: SCROLLBAR SET: TRACKPOS is a GET-only sub-command on line {}",
                    line
                );
            } else {
                eprintln!(
                    "Error: SCROLLBAR: unknown sub-command \"{}\" on line {}",
                    noun, line
                );
            }
            self.error_count += 1;
            self.consume_to_eol();
            return Ok(Statement::Noop("SCROLLBAR".to_string(), line));
        }
        self.advance(); // noun
        let mut args = Vec::new();
        args.push(self.parse_expression()?);
        self.expect(&Token::Comma)?;
        args.push(self.parse_expression()?);
        loop {
            // Both separators are accepted at every step: `TO` introduces the
            // result variable(s) and a comma can follow it for a two-target
            // form such as `HEADER GET ITEM ... TO lo, hi`.
            if self.peek() == &Token::Comma || self.peek() == &Token::To {
                self.advance();
                args.push(self.parse_expression()?);
            } else {
                break;
            }
        }
        self.consume_to_eol();
        Ok(Statement::Call(CallStmt {
            name: format!("SCROLLBAR_{}_{}", verb, noun),
            args,
            line,
        }))
    }

    /// COMBOBOX / LISTBOX statements, official PB syntax (batch 168).
    ///
    /// Both families expose the same operation set and are driven by the same
    /// Win32 messages; `kind` is 0 for COMBOBOX and 1 for LISTBOX.  The official
    /// help indexes every `item&` from ONE ("1 for the first item, 2 for the
    /// second"), which the runtime converts to the zero-based Win32 index.
    ///
    /// ADD keeps accepting the older two-operand form `COMBOBOX ADD hCtrl, "s"`
    /// alongside the official `COMBOBOX ADD hDlg, id&, StrExpr [TO datav&]`;
    /// codegen picks the path from the operand count.
    fn parse_cblb_statement(&mut self, line: usize, head: &str, kind: i64) -> PbResult<Statement> {
        self.advance(); // COMBOBOX / LISTBOX
        let verb = self.peek_word_upper();
        let known = matches!(
            verb.as_str(),
            "ADD" | "DELETE" | "FIND" | "GET" | "INSERT" | "RESET" | "SELECT" | "SET" | "UNSELECT"
        );
        if !known {
            self.consume_to_eol();
            eprintln!(
                "Error: COMBOBOX/LISTBOX: unknown sub-command on line {}",
                line
            );
            self.error_count += 1;
            return Ok(Statement::Noop(head.to_string(), line));
        }
        self.advance(); // verb
        let mut noun = String::new();
        if verb == "FIND" {
            if self.peek_plain_upper() == "EXACT" {
                noun = "EXACT".to_string();
                self.advance();
            }
        } else if verb == "GET" || verb == "SET" {
            let n = self.peek_word_upper();
            if matches!(
                n.as_str(),
                "COUNT" | "SELCOUNT" | "SELECT" | "STATE" | "TEXT" | "USER"
            ) {
                noun = n;
                self.advance();
            }
        }
        let mut args = Vec::new();
        if verb == "RESET" {
            args.push(self.parse_expression()?);
            self.expect(&Token::Comma)?;
            args.push(self.parse_expression()?);
        } else {
            args.push(self.parse_expression()?);
            self.expect(&Token::Comma)?;
            args.push(self.parse_expression()?);
            loop {
                // Both separators are accepted at every step: `TO` introduces the
                // result variable(s) and a comma can follow it for a two-target
                // form such as `HEADER GET ITEM ... TO lo, hi`.
                if self.peek() == &Token::Comma || self.peek() == &Token::To {
                    self.advance();
                    args.push(self.parse_expression()?);
                } else {
                    break;
                }
            }
        }
        self.consume_to_eol();
        let name = if noun.is_empty() {
            format!("{}_{}", head, verb)
        } else {
            format!("{}_{}_{}", head, verb, noun)
        };
        let _ = kind; // the family is carried by the statement name
        Ok(Statement::Call(CallStmt { name, args, line }))
    }

    /// TAB <verb> [<noun>] ... -- official TAB_statement.htm (batch 169).
    ///
    /// Verbs: DELETE, GET, INSERT, RESET, SELECT, SET.
    /// Nouns: COUNT, DIALOG, IMAGE, PAGE, SELECT, TEXT, IMAGELIST.
    /// `SELECT` is a reserved token, so the verb/noun are read with
    /// `peek_word_upper()` (which maps Token::Select to "SELECT").
    fn parse_tab_statement(&mut self, line: usize) -> PbResult<Statement> {
        self.advance(); // TAB
        let verb = self.peek_word_upper();
        let known = matches!(
            verb.as_str(),
            "DELETE" | "GET" | "INSERT" | "RESET" | "SELECT" | "SET"
        );
        if !known {
            self.consume_to_eol();
            eprintln!("Error: TAB: unsupported form on line {}", line);
            self.error_count += 1;
            return Ok(Statement::Noop("TAB".to_string(), line));
        }
        self.advance(); // verb
        let mut noun = String::new();
        if verb == "GET" || verb == "SET" {
            let n = self.peek_word_upper();
            if matches!(
                n.as_str(),
                "COUNT" | "DIALOG" | "IMAGE" | "PAGE" | "SELECT" | "TEXT" | "IMAGELIST"
            ) {
                noun = n;
                self.advance();
            }
        } else if verb == "INSERT" && self.peek_word_upper() == "PAGE" {
            noun = "PAGE".to_string();
            self.advance();
        }
        let mut args = Vec::new();
        if verb == "INSERT" && noun == "PAGE" {
            // TAB INSERT PAGE hDlg, ID&, PageNum&, Image&, Text$ [CALL cb]
            //                 TO PageDlgVar&
            args.push(self.parse_expression()?);
            self.expect(&Token::Comma)?;
            args.push(self.parse_expression()?);
            self.expect(&Token::Comma)?;
            args.push(self.parse_expression()?);
            self.expect(&Token::Comma)?;
            args.push(self.parse_expression()?);
            self.expect(&Token::Comma)?;
            args.push(self.parse_expression()?);
            // The optional CALL operand is always emitted, as IntegerLit(0)
            // when absent, so codegen sees a fixed seven-operand layout.
            if self.peek() == &Token::Call {
                self.advance();
                args.push(self.parse_expression()?);
            } else {
                args.push(Expr::IntegerLit(0));
            }
            self.expect(&Token::To)?;
            args.push(self.parse_expression()?);
        } else if verb == "RESET" {
            args.push(self.parse_expression()?);
            self.expect(&Token::Comma)?;
            args.push(self.parse_expression()?);
        } else {
            args.push(self.parse_expression()?);
            loop {
                // Both separators are accepted at every step: `TO` introduces
                // the result variable and a comma separates further operands.
                if self.peek() == &Token::Comma || self.peek() == &Token::To {
                    self.advance();
                    args.push(self.parse_expression()?);
                } else {
                    break;
                }
            }
        }
        self.consume_to_eol();
        let name = if noun.is_empty() {
            format!("TAB_{}", verb)
        } else {
            format!("TAB_{}_{}", verb, noun)
        };
        Ok(Statement::Call(CallStmt { name, args, line }))
    }

    fn parse_statement(&mut self) -> PbResult<Statement> {
        self.skip_eol();
        let line = self.current_line();

        match self.peek().clone() {
            Token::Identifier(name) if is_deftype_name(&name) => self.parse_deftype_statement(),
            Token::If => self.parse_if_statement(),
            Token::For => self.parse_for_statement(),
            Token::Do => self.parse_do_loop(),
            Token::While => self.parse_while_wend(),
            Token::Select => self.parse_select_case(),
            Token::Print => self.parse_print_statement(),
            Token::Input => {
                self.advance(); // consume INPUT
                                // INPUT FLUSH — discard buffered console input
                if let Token::Identifier(s) = self.peek() {
                    if s.eq_ignore_ascii_case("FLUSH") {
                        self.advance();
                        self.consume_to_eol();
                        return Ok(Statement::InputFlush);
                    }
                }
                // INPUT #filenum, var1, var2, ...
                if self.peek() == &Token::Hash {
                    self.advance(); // consume #
                    let file_num = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let mut vars = Vec::new();
                    vars.push(self.parse_expression()?);
                    while self.peek() == &Token::Comma {
                        self.advance();
                        vars.push(self.parse_expression()?);
                    }
                    self.consume_to_eol();
                    return Ok(Statement::InputFile(InputFileStmt {
                        file_num,
                        vars,
                        line,
                    }));
                }
                // Plain INPUT (console) — read from stdin
                let mut prompt = None;
                let mut no_newline = false;
                // optional leading ; means no newline
                if self.peek() == &Token::Semicolon {
                    no_newline = true;
                    self.advance();
                }
                // optional prompt string
                if let Token::StringLiteral(_) = self.peek() {
                    prompt = Some(self.parse_expression()?);
                    if self.peek() == &Token::Semicolon || self.peek() == &Token::Comma {
                        self.advance();
                    }
                }
                // variable list
                let mut vars = Vec::new();
                vars.push(self.parse_expression()?);
                while self.peek() == &Token::Comma {
                    self.advance();
                    vars.push(self.parse_expression()?);
                }
                self.consume_to_eol();
                Ok(Statement::InputConsole(InputConsoleStmt {
                    prompt,
                    no_newline,
                    vars,
                    line,
                }))
            }
            Token::Open => self.parse_open_statement(),
            Token::Close => self.parse_close_statement(),
            Token::Callback => {
                // batch 249: CALLBACK FUNCTION [= expr] / CALLBACK SUB
                // PBWin callback pseudo-statement (sets/clears the callback return
                // value).  Compiled to an assignment on synthetic variables so the
                // AST stays unchanged; codegen stores the value and the callback
                // dispatcher treats the slot as "return 0".
                self.advance(); // CALLBACK
                let is_sub = self.peek() == &Token::Sub;
                if self.peek() == &Token::Function || self.peek() == &Token::Sub {
                    self.advance();
                }
                let value = if self.peek() == &Token::Eq {
                    self.advance();
                    self.parse_expression()?
                } else {
                    Expr::IntegerLit(0)
                };
                self.consume_to_eol();
                let cname = if is_sub {
                    "__CALLBACK_SUB".to_string()
                } else {
                    "__CALLBACK_FUNCTION".to_string()
                };
                Ok(Statement::Assign(AssignStmt {
                    target: Expr::Variable(cname),
                    value,
                    line,
                }))
            }
            Token::Call => {
                self.advance();
                // CALL DWORD TargetPtr [USING proto(args)] [TO result]
                if let Token::Identifier(w) = self.peek() {
                    if w.to_uppercase() == "DWORD" {
                        return self.parse_call_dword(line);
                    }
                }
                if matches!(self.peek(), Token::Dword) {
                    return self.parse_call_dword(line);
                }
                let name = self.consume_identifier()?;
                let args = if self.peek() == &Token::LParen {
                    self.advance();
                    let a = self.parse_arg_list()?;
                    self.expect(&Token::RParen)?;
                    a
                } else {
                    Vec::new()
                };
                self.consume_to_eol();
                Ok(Statement::Call(CallStmt { name, args, line }))
            }
            Token::Incr => {
                self.advance();
                let target = self.parse_expression()?;
                let amount = if self.peek() == &Token::Comma {
                    self.advance();
                    Some(self.parse_expression()?)
                } else {
                    None
                };
                self.consume_to_eol();
                Ok(Statement::Incr(IncrDecrStmt {
                    target,
                    amount,
                    line,
                }))
            }
            Token::Decr => {
                self.advance();
                let target = self.parse_expression()?;
                let amount = if self.peek() == &Token::Comma {
                    self.advance();
                    Some(self.parse_expression()?)
                } else {
                    None
                };
                self.consume_to_eol();
                Ok(Statement::Decr(IncrDecrStmt {
                    target,
                    amount,
                    line,
                }))
            }
            Token::Exit => {
                self.advance();
                let stmt = match self.peek() {
                    Token::Sub => {
                        self.advance();
                        Statement::ExitSub
                    }
                    Token::Function => {
                        self.advance();
                        Statement::ExitFunction
                    }
                    Token::For => {
                        self.advance();
                        Statement::ExitFor
                    }
                    Token::Do => {
                        self.advance();
                        Statement::ExitDo
                    }
                    Token::Select => {
                        self.advance();
                        Statement::ExitSelect
                    }
                    // EXIT TRY (batch 26)
                    Token::Identifier(ref kw) if kw.eq_ignore_ascii_case("TRY") => {
                        self.advance();
                        Statement::ExitTry
                    }
                    // CHW: EXIT LOOP is the generic loop-exit for both WHILE/WEND
                    // and DO/LOOP (PowerBASIC Help "EXIT Statement"). Previously the
                    // fallback treated EXIT LOOP as EXIT SUB (silent early return).
                    Token::Loop => {
                        self.advance();
                        Statement::ExitLoop
                    }
                    _ => Statement::ExitSub,
                };
                self.consume_to_eol();
                Ok(stmt)
            }
            Token::Function => {
                // FUNCTION = expr (return value)
                self.advance();
                self.expect(&Token::Eq)?;
                let value = self.parse_expression()?;
                self.consume_to_eol();
                Ok(Statement::FunctionReturn(FunctionReturnStmt {
                    value,
                    line,
                }))
            }
            Token::Local => {
                let dims = self.parse_local_decl()?;
                if dims.len() == 1 {
                    Ok(Statement::Dim(dims.into_iter().next().unwrap()))
                } else {
                    Ok(Statement::Block(
                        dims.into_iter().map(Statement::Dim).collect(),
                    ))
                }
            }
            // REGISTER var [AS type] [, ...] 鈥?optimization hint; accepted as LOCAL
            Token::Register => {
                let dims = self.parse_local_decl()?;
                if dims.len() == 1 {
                    Ok(Statement::Dim(dims.into_iter().next().unwrap()))
                } else {
                    Ok(Statement::Block(
                        dims.into_iter().map(Statement::Dim).collect(),
                    ))
                }
            }
            // Inline assembly shortcut: ! opcode ... (raw text captured by lexer)
            Token::AsmText(text) => {
                let t = text.clone();
                self.advance();
                Ok(Statement::Asm(t))
            }
            Token::Dim => {
                let dims = self.parse_dim_statement(DimScope::Dim)?;
                if dims.len() == 1 {
                    Ok(Statement::Dim(dims.into_iter().next().unwrap()))
                } else {
                    Ok(Statement::Block(
                        dims.into_iter().map(Statement::Dim).collect(),
                    ))
                }
            }
            Token::Redim => {
                let dim = self.parse_redim_statement()?;
                Ok(Statement::Redim(dim))
            }
            Token::Static => {
                let dims = self.parse_dim_statement(DimScope::Static)?;
                if dims.len() == 1 {
                    Ok(Statement::Dim(dims.into_iter().next().unwrap()))
                } else {
                    Ok(Statement::Block(
                        dims.into_iter().map(Statement::Dim).collect(),
                    ))
                }
            }
            Token::Threaded => {
                let dims = self.parse_dim_statement(DimScope::Threaded)?;
                if dims.len() == 1 {
                    Ok(Statement::Dim(dims.into_iter().next().unwrap()))
                } else {
                    Ok(Statement::Block(
                        dims.into_iter().map(Statement::Dim).collect(),
                    ))
                }
            }
            Token::Type => {
                // TYPE SET dest = src : copy bytes into a TYPE variable
                if matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase() == "SET")
                {
                    self.advance(); // consume TYPE
                    self.advance(); // consume SET
                                    // parse_primary (not parse_expression): `dest = src` must not be
                                    // swallowed as an Eq comparison expression
                    let dest = self.parse_primary()?;
                    self.expect(&Token::Eq)?;
                    let src = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "TYPE SET".to_string(),
                        args: vec![dest, src],
                        line,
                    }));
                }
                // Bare TYPE (no SET) inside a body.  PB declares UDTs at module level,
                // so this is not a declaration site - and silently skipping the line
                // made a misplaced declaration vanish without a word (batch 208).
                self.consume_to_eol();
                eprintln!(
                    "Error: bare TYPE is not a statement - a UDT declaration belongs at module level (line {})",
                    line
                );
                self.error_count += 1;
                Ok(Statement::Noop("TYPE".to_string(), line))
            }
            Token::Global => {
                // GLOBAL inside a sub (shouldn't happen but handle gracefully)
                let dims = self.parse_dim_statement(DimScope::Global)?;
                if dims.len() == 1 {
                    Ok(Statement::Dim(dims.into_iter().next().unwrap()))
                } else {
                    Ok(Statement::Block(
                        dims.into_iter().map(Statement::Dim).collect(),
                    ))
                }
            }
            Token::On => {
                self.advance();
                // ON ERROR GOTO label
                if self.peek() == &Token::Error {
                    self.advance();
                    if self.peek() == &Token::GoTo {
                        self.advance();
                        if let Token::IntegerLiteral(0) = self.peek() {
                            self.advance();
                            self.consume_to_eol();
                            return Ok(Statement::OnErrorGotoZero);
                        }
                        let label = match self.peek().clone() {
                            Token::Identifier(s) => {
                                self.advance();
                                s
                            }
                            Token::IntegerLiteral(n) => {
                                self.advance();
                                n.to_string()
                            }
                            _ => {
                                self.consume_to_eol();
                                eprintln!(
                                    "Error: ON ERROR GOTO: unsupported form on line {}",
                                    line
                                );
                                self.error_count += 1;
                                return Ok(Statement::Noop("ON ERROR GOTO".to_string(), line));
                            }
                        };
                        self.consume_to_eol();
                        return Ok(Statement::OnErrorGoto(label));
                    }
                    // ON ERROR RESUME NEXT 鈥?disable error trapping
                    if matches!(self.peek(), Token::Resume) || self.peek_plain_upper() == "RESUME" {
                        self.advance();
                        self.consume_to_eol();
                        return Ok(Statement::OnErrorGotoZero);
                    }
                }
                // ON expr GOTO label1, label2, ... / ON expr GOSUB ... / ON expr CALL proc() ...
                let expr = self.parse_expression()?;
                if self.peek() == &Token::Call {
                    // ON expr CALL Procedure() [, Function() TO RetVar] ...
                    self.advance(); // consume CALL
                    let mut targets: Vec<OnCallTarget> = Vec::new();
                    loop {
                        let name = self.consume_identifier()?;
                        let args = if self.peek() == &Token::LParen {
                            self.advance();
                            let a = self.parse_arg_list()?;
                            self.expect(&Token::RParen)?;
                            a
                        } else {
                            Vec::new()
                        };
                        let mut ret_var = None;
                        if self.peek() == &Token::To
                            || matches!(self.peek(), Token::Identifier(id) if id.eq_ignore_ascii_case("TO"))
                        {
                            self.advance();
                            ret_var = Some(self.consume_identifier()?);
                        }
                        targets.push(OnCallTarget {
                            name,
                            args,
                            ret_var,
                        });
                        if self.peek() == &Token::Comma {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    self.consume_to_eol();
                    if targets.is_empty() {
                        eprintln!("Error: ON CALL: unsupported form on line {}", line);
                        self.error_count += 1;
                        return Ok(Statement::Noop("ON CALL".to_string(), line));
                    }
                    return Ok(Statement::OnCall {
                        expr: Box::new(expr),
                        targets,
                    });
                }
                let is_gosub = match self.peek() {
                    Token::GoTo => {
                        self.advance();
                        false
                    }
                    Token::GoSub => {
                        self.advance();
                        true
                    }
                    _ => {
                        self.consume_to_eol();
                        eprintln!("Error: ON: unsupported form on line {}", line);
                        self.error_count += 1;
                        return Ok(Statement::Noop("ON".to_string(), line));
                    }
                };
                let mut labels: Vec<String> = Vec::new();
                loop {
                    match self.peek().clone() {
                        Token::Identifier(s) => {
                            self.advance();
                            labels.push(s);
                        }
                        Token::IntegerLiteral(n) => {
                            self.advance();
                            labels.push(n.to_string());
                        }
                        _ => break,
                    }
                    if self.peek() == &Token::Comma {
                        self.advance();
                    } else {
                        break;
                    }
                }
                self.consume_to_eol();
                if labels.is_empty() {
                    eprintln!("Error: ON GOTO/GOSUB: unsupported form on line {}", line);
                    self.error_count += 1;
                    return Ok(Statement::Noop("ON GOTO/GOSUB".to_string(), line));
                }
                if is_gosub {
                    Ok(Statement::OnGoSub {
                        expr: Box::new(expr),
                        labels,
                    })
                } else {
                    Ok(Statement::OnGoTo {
                        expr: Box::new(expr),
                        labels,
                    })
                }
            }
            Token::Error => {
                // ERROR n — set the PB error code (readable via ERR)
                self.advance(); // consume ERROR
                let args = if !self.at_eol_or_eof() {
                    vec![self.parse_expression()?]
                } else {
                    Vec::new()
                };
                self.consume_to_eol();
                Ok(Statement::Call(CallStmt {
                    name: "ERROR".to_string(),
                    args,
                    line,
                }))
            }
            Token::Replace => {
                // REPLACE old$ WITH new$ IN target$
                self.advance();
                let old = self.parse_expression()?;
                if matches!(self.peek(), Token::Identifier(id) if id.eq_ignore_ascii_case("WITH")) {
                    self.advance();
                }
                let new_s = self.parse_expression()?;
                if matches!(self.peek(), Token::Identifier(id) if id.eq_ignore_ascii_case("IN")) {
                    self.advance();
                }
                let target = self.parse_expression()?;
                self.consume_to_eol();
                Ok(Statement::Call(CallStmt {
                    name: "REPLACE".to_string(),
                    args: vec![target, old, new_s],
                    line,
                }))
            }
            Token::Remove => {
                // REMOVE$ is usually a function but REMOVE can be a statement
                self.advance();
                self.consume_to_eol();
                eprintln!("Error: REMOVE: unsupported form on line {}", line);
                self.error_count += 1;
                Ok(Statement::Noop("REMOVE".to_string(), line))
            }
            Token::Resume => {
                self.advance();
                // RESUME NEXT
                if matches!(self.peek(), Token::Next) || self.peek_plain_upper() == "NEXT" {
                    self.advance();
                    self.consume_to_eol();
                    return Ok(Statement::ResumeNext);
                }
                // RESUME FLUSH
                if self.peek_plain_upper() == "FLUSH" {
                    self.advance();
                    self.consume_to_eol();
                    return Ok(Statement::ResumeFlush);
                }
                // RESUME label / line_number
                match self.peek().clone() {
                    Token::Identifier(s) => {
                        self.advance();
                        self.consume_to_eol();
                        return Ok(Statement::ResumeLabel(s));
                    }
                    Token::IntegerLiteral(n) => {
                        self.advance();
                        self.consume_to_eol();
                        return Ok(Statement::ResumeLabel(n.to_string()));
                    }
                    _ => {}
                }
                // bare RESUME: re-execute the statement that errored
                self.consume_to_eol();
                Ok(Statement::Resume)
            }
            Token::GoSub => {
                self.advance();
                let label = match self.peek().clone() {
                    Token::Identifier(s) => {
                        self.advance();
                        s
                    }
                    Token::IntegerLiteral(n) => {
                        self.advance();
                        n.to_string()
                    }
                    _ => {
                        return Err(PbError::parser(
                            format!("Expected label after GOSUB, got {:?}", self.peek()),
                            self.current_file(),
                            line,
                        ))
                    }
                };
                self.consume_to_eol();
                Ok(Statement::GoSub(label))
            }
            Token::GoTo => {
                self.advance();
                let label = match self.peek().clone() {
                    Token::Identifier(s) => {
                        self.advance();
                        s
                    }
                    Token::IntegerLiteral(n) => {
                        self.advance();
                        n.to_string()
                    }
                    _ => {
                        return Err(PbError::parser(
                            format!("Expected label after GOTO, got {:?}", self.peek()),
                            self.current_file(),
                            line,
                        ))
                    }
                };
                self.consume_to_eol();
                Ok(Statement::GoTo(label))
            }
            Token::Return => {
                self.advance();
                self.consume_to_eol();
                Ok(Statement::Return)
            }
            Token::Iterate => {
                self.advance();
                let target = match self.peek() {
                    Token::For => {
                        self.advance();
                        IterateTarget::For
                    }
                    Token::Do => {
                        self.advance();
                        IterateTarget::Do
                    }
                    _ => IterateTarget::For,
                };
                self.consume_to_eol();
                Ok(Statement::Iterate(target))
            }
            Token::HashInclude => {
                // #INCLUDE inside a SUB (e.g., SetupDims including DIMFILE.INC)
                // We need to treat the included content as statements
                // For now, skip — the preprocessor should have resolved this
                self.consume_to_eol();
                Ok(Statement::Noop("#INCLUDE".to_string(), line))
            }
            Token::PercentConstant(_) => {
                // %CONSTANT = value inside a sub
                self.consume_to_eol();
                eprintln!("Error: %CONSTANT: defining a constant inside a SUB is not supported on line {}", line);
                self.error_count += 1;
                Ok(Statement::Noop("%CONSTANT".to_string(), line))
            }
            Token::End => {
                // Check what follows END
                match self.peek_at(1) {
                    Some(Token::Sub) | Some(Token::Function) => {
                        // END SUB/FUNCTION should be handled as body terminators,
                        // not consumed here. Return error to let body parser deal with it.
                        Err(PbError::parser(
                            "Unexpected END SUB/FUNCTION inside body",
                            None,
                            self.current_line(),
                        ))
                    }
                    Some(Token::If) | Some(Token::Select) | Some(Token::Type) => {
                        // Mismatched block-end — consume and skip
                        self.consume_to_eol();
                        eprintln!(
                            "Error: END (mismatched block): unsupported form on line {}",
                            line
                        );
                        self.error_count += 1;
                        Ok(Statement::Noop("END (mismatched block)".to_string(), line))
                    }
                    _ => {
                        // Standalone END (program termination)
                        self.advance(); // consume END
                        self.consume_to_eol();
                        Ok(Statement::Noop("END".to_string(), line))
                    }
                }
            }
            Token::IntegerLiteral(n) => {
                // Numeric label: 30300 followed by anything (used with GOSUB/GOTO)
                let label = n.to_string();
                self.advance(); // consume the number
                Ok(Statement::Label(label))
            }
            Token::Question => {
                // PB official: `? txt$ [, [style&], title$]` is the MSGBOX
                // statement abbreviation (CHW msgbox_s.htm; PBWin compiles it
                // to a real message box — batch 250 probe p4/p5).  Lower it to
                // the same CallStmt the MSGBOX keyword produces so codegen's
                // existing "MSGBOX" arm handles text/style/title.
                self.advance(); // consume ?
                let mut args = Vec::new();
                if !self.at_eol_or_eof() {
                    args.push(self.parse_expression()?);
                    while self.peek() == &Token::Comma {
                        self.advance();
                        // PB official optional-arg empty slot: `? "t", , "cap"`
                        if matches!(
                            self.peek(),
                            Token::Comma | Token::Eol | Token::Eof | Token::Colon
                        ) {
                            args.push(Expr::Omitted);
                            continue;
                        }
                        args.push(self.parse_expression()?);
                    }
                }
                self.consume_to_eol();
                Ok(Statement::Call(CallStmt {
                    name: "MSGBOX".to_string(),
                    args,
                    line,
                }))
            }
            Token::Identifier(ref name) => {
                let name_upper = name.to_uppercase();

                // PROPERTY = expr — return value inside a PROPERTY GET body
                // (CHW PROPERTY/END PROPERTY).  "PROPERTY" is otherwise not a
                // declared name, so this must be intercepted before the generic
                // assignment path rejects it.
                if name_upper == "PROPERTY" && self.peek_at(1) == Some(&Token::Eq) {
                    self.advance(); // PROPERTY
                    self.advance(); // =
                    let value = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::FunctionReturn(FunctionReturnStmt {
                        value,
                        line,
                    }));
                }

                // OBJECT CALL/GET/LET/SET obj.member [(args)] [TO result] —
                // DISPATCH access with VARIANT marshalling (batch278-74).
                if name_upper == "OBJECT" {
                    // CALL is the reserved Token::Call (not an Identifier);
                    // GET/LET/SET are plain identifiers.
                    let obj_kind: Option<String> = match self.peek_at(1) {
                        Some(Token::Call) => Some("CALL".to_string()),
                        Some(Token::Identifier(kw)) => {
                            let k = kw.to_uppercase();
                            if k == "GET" || k == "LET" || k == "SET" {
                                Some(k)
                            } else {
                                None
                            }
                        }
                        _ => None,
                    };
                    if let Some(k) = obj_kind {
                        self.advance(); // OBJECT
                        self.advance(); // CALL/GET/LET/SET
                        return self.parse_object_stmt(&k, line);
                    }
                }
                // DATA item1, item2, ... — string constants for READ
                if name_upper == "DATA" {
                    self.advance(); // consume DATA
                    let items = self.parse_data_items();
                    self.consume_to_eol();
                    return Ok(Statement::Data(items));
                }
                // RESTORE [label] — reset the DATA cursor
                if name_upper == "RESTORE" {
                    self.advance();
                    self.consume_to_eol();
                    return Ok(Statement::Restore);
                }
                // READ var1, var2, ... — read next DATA items
                if name_upper == "READ" {
                    self.advance(); // consume READ
                    let mut args = Vec::new();
                    while !self.at_eol_or_eof() {
                        args.push(self.parse_expression()?);
                        if self.peek() == &Token::Comma {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "READ".to_string(),
                        args,
                        line,
                    }));
                }
                // LINE INPUT #filenum, var$
                if name_upper == "LINE" {
                    // Peek ahead: LINE INPUT #...
                    if let Some(Token::Input) = self.peek_at(1) {
                        self.advance(); // consume LINE
                        self.advance(); // consume INPUT
                        if self.peek() == &Token::Hash {
                            self.advance(); // consume #
                            let file_num = self.parse_expression()?;
                            self.expect(&Token::Comma)?;
                            let var = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::LineInputFile(LineInputFileStmt {
                                file_num,
                                var,
                                line,
                            }));
                        }
                        // LINE INPUT without # — console, read whole line
                        let mut prompt = None;
                        if self.peek() == &Token::Semicolon {
                            self.advance();
                        }
                        if let Token::StringLiteral(_) = self.peek() {
                            prompt = Some(self.parse_expression()?);
                            if self.peek() == &Token::Semicolon {
                                self.advance();
                            }
                        }
                        let var = self.parse_expression()?;
                        self.consume_to_eol();
                        return Ok(Statement::LineInputConsole(LineInputConsoleStmt {
                            prompt,
                            var,
                            line,
                        }));
                    }
                    // LINE (not INPUT) — DDT drawing or other, consume
                    self.advance();
                    self.consume_to_eol();
                    eprintln!("Error: LINE: unsupported form on line {}", line);
                    self.error_count += 1;
                    return Ok(Statement::Noop("LINE".to_string(), line));
                }

                // KILL filename$
                if name_upper == "KILL" {
                    self.advance(); // consume KILL
                    let filename = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Kill(filename));
                }

                // MENU NEW BAR TO hMenu | MENU NEW POPUP TO hPop
                // MENU ADD STRING, hMenu, txt$, id&, state& | MENU ADD POPUP, hMenu, hSub, id&
                // MENU DELETE hMenu, pos& (batch 50)
                // GRAPHIC BITMAP NEW w&, h& TO hBmp | GRAPHIC BITMAP END [hBmp]
                // Memory DIB bitmap — not visible, console-testable (batch 51)
                if name_upper == "GRAPHIC" {
                    self.advance(); // consume GRAPHIC
                    let gop = self.peek_plain_upper();
                    let gop_kw = self.peek_graphic_word_upper();
                    if gop == "ATTACH" {
                        self.advance();
                        let mut args = vec![self.parse_expression()?];
                        while self.peek() == &Token::Comma {
                            self.advance();
                            args.push(self.parse_expression()?);
                        }
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "GRAPHIC_ATTACH".to_string(),
                            args,
                            line,
                        }));
                    }
                    if gop == "WINDOW" {
                        // GRAPHIC WINDOW NEW|TEXT caption$, x&, y&, w&, h& [, hFont]
                        //     TO hWin& [, HIDE|NORMALIZE]
                        // GRAPHIC WINDOW CLICK [hWin&] TO click&, x!, y!
                        // GRAPHIC WINDOW END|HIDE|MINIMIZE|NORMALIZE|STABILIZE|NONSTABLE [hWin&]
                        // (batch 180; the standalone graphic window of the CC
                        //  graphics model, not the DDT dialog)
                        self.advance(); // consume WINDOW
                        let sub = self.peek_plain_upper();
                        if sub == "NEW" || sub == "TEXT" {
                            self.advance(); // consume NEW / TEXT
                            let cap = self.parse_expression()?;
                            self.expect(&Token::Comma)?;
                            let x = self.parse_expression()?;
                            self.expect(&Token::Comma)?;
                            let y = self.parse_expression()?;
                            self.expect(&Token::Comma)?;
                            let w = self.parse_expression()?;
                            self.expect(&Token::Comma)?;
                            let h = self.parse_expression()?;
                            // the font is optional and only a font, never the TO target
                            let font = if self.peek() == &Token::Comma
                                && self.peek_at(1) != Some(&Token::To)
                            {
                                self.advance();
                                self.parse_expression()?
                            } else {
                                Expr::IntegerLit(0)
                            };
                            self.expect(&Token::To)?;
                            let target = self.parse_expression()?;
                            // [, HIDE|NORMALIZE] - the default is a visible window
                            let mut show = Expr::IntegerLit(1);
                            if self.peek() == &Token::Comma {
                                self.advance();
                            }
                            if let Token::Identifier(kw) = self.peek() {
                                let u = kw.to_uppercase();
                                if u == "HIDE" {
                                    show = Expr::IntegerLit(0);
                                    self.advance();
                                } else if u == "NORMALIZE" {
                                    show = Expr::IntegerLit(1);
                                    self.advance();
                                }
                            }
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: if sub == "NEW" {
                                    "GRAPHIC_WINDOW_NEW".to_string()
                                } else {
                                    "GRAPHIC_WINDOW_TEXT".to_string()
                                },
                                args: vec![cap, x, y, w, h, font, target, show],
                                line,
                            }));
                        }
                        if sub == "CLICK" {
                            self.advance(); // consume CLICK
                            let hw = if self.peek() == &Token::To {
                                Expr::IntegerLit(0)
                            } else {
                                self.parse_expression()?
                            };
                            self.expect(&Token::To)?;
                            let ck = self.parse_expression()?;
                            self.expect(&Token::Comma)?;
                            let px = self.parse_expression()?;
                            self.expect(&Token::Comma)?;
                            let py = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "GRAPHIC_WINDOW_CLICK".to_string(),
                                args: vec![hw, ck, px, py],
                                line,
                            }));
                        }
                        if sub == "END"
                            || sub == "HIDE"
                            || sub == "MINIMIZE"
                            || sub == "NORMALIZE"
                            || sub == "STABILIZE"
                            || sub == "NONSTABLE"
                        {
                            self.advance(); // consume the verb
                            let hw = if self.at_eol_or_eof() {
                                Expr::IntegerLit(0)
                            } else {
                                self.parse_expression()?
                            };
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: format!("GRAPHIC_WINDOW_{}", sub),
                                args: vec![hw],
                                line,
                            }));
                        }
                    }
                    if gop == "GET" {
                        // GRAPHIC GET CANVAS TO hbmp | GRAPHIC GET DC TO hdc | GRAPHIC GET MIX TO mix&
                        // GRAPHIC GET SIZE TO w,h | GRAPHIC GET TEXTALIGN TO align&
                        self.advance();
                        let sub = self.peek_plain_upper();
                        if sub == "SIZE" {
                            self.advance();
                            self.expect(&Token::To)?;
                            let w = self.parse_expression()?;
                            self.expect(&Token::Comma)?;
                            let h = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "GRAPHIC_GET_SIZE".to_string(),
                                args: vec![w, h],
                                line,
                            }));
                        }
                        if sub == "TEXTALIGN" {
                            self.advance();
                            self.expect(&Token::To)?;
                            let dst = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "GRAPHIC_GET_TEXTALIGN".to_string(),
                                args: vec![dst],
                                line,
                            }));
                        }
                        if sub == "CLIENT" {
                            // GRAPHIC GET CLIENT TO w&, h&
                            self.advance();
                            self.expect(&Token::To)?;
                            let w = self.parse_expression()?;
                            self.expect(&Token::Comma)?;
                            let h = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "GRAPHIC_GET_CLIENT".to_string(),
                                args: vec![w, h],
                                line,
                            }));
                        }
                        if sub == "LOC" {
                            // GRAPHIC GET LOC TO x&, y&
                            self.advance();
                            self.expect(&Token::To)?;
                            let x = self.parse_expression()?;
                            self.expect(&Token::Comma)?;
                            let y = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "GRAPHIC_GET_LOC".to_string(),
                                args: vec![x, y],
                                line,
                            }));
                        }
                        if sub == "OVERLAP" {
                            // GRAPHIC GET OVERLAP TO var&  (batch 181)
                            self.advance();
                            self.expect(&Token::To)?;
                            let dst = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "GRAPHIC_GET_OVERLAP".to_string(),
                                args: vec![dst],
                                line,
                            }));
                        }
                        if sub == "CLIP" || sub == "VIEW" || sub == "LINES" || sub == "WRAP" {
                            // GRAPHIC GET CLIP TO w!, h! | GRAPHIC GET VIEW TO x!, y!
                            // GRAPHIC GET LINES TO n& | GRAPHIC GET WRAP TO w&  (batch 63)
                            self.advance();
                            self.expect(&Token::To)?;
                            let a = self.parse_expression()?;
                            let mut args = vec![a];
                            if sub == "CLIP" || sub == "VIEW" {
                                self.expect(&Token::Comma)?;
                                let b = self.parse_expression()?;
                                args.push(b);
                            }
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: format!("GRAPHIC_GET_{}", sub),
                                args,
                                line,
                            }));
                        }
                        if sub == "BITS" {
                            // GRAPHIC GET BITS TO bitvar$ — whole bitmap as DIB string (batch 64)
                            self.advance();
                            self.expect(&Token::To)?;
                            let dst = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "GRAPHIC_GET_BITS".to_string(),
                                args: vec![dst],
                                line,
                            }));
                        }
                        if sub == "SCALE" {
                            // GRAPHIC GET SCALE TO x1!, y1!, x2!, y2!  (batch 64)
                            self.advance();
                            self.expect(&Token::To)?;
                            let a = self.parse_expression()?;
                            let mut args = vec![a];
                            for _ in 0..3 {
                                self.expect(&Token::Comma)?;
                                args.push(self.parse_expression()?);
                            }
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "GRAPHIC_GET_SCALE".to_string(),
                                args,
                                line,
                            }));
                        }
                        if sub == "WORDWRAP" {
                            // GRAPHIC GET WORDWRAP TO n&  (batch 65)
                            self.advance();
                            self.expect(&Token::To)?;
                            let dst = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "GRAPHIC_GET_WORDWRAP".to_string(),
                                args: vec![dst],
                                line,
                            }));
                        }
                        if sub == "PIXEL" {
                            // GRAPHIC GET PIXEL (x,y) TO dst&
                            self.advance();
                            self.expect(&Token::LParen)?;
                            let x = self.parse_expression()?;
                            self.expect(&Token::Comma)?;
                            let y = self.parse_expression()?;
                            self.expect(&Token::RParen)?;
                            self.expect(&Token::To)?;
                            let dst = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "GRAPHIC_GET_PIXEL".to_string(),
                                args: vec![x, y, dst],
                                line,
                            }));
                        }
                        if sub == "PPI" {
                            self.advance();
                            self.expect(&Token::To)?;
                            let x = self.parse_expression()?;
                            self.expect(&Token::Comma)?;
                            let y = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "GRAPHIC_GET_PPI".to_string(),
                                args: vec![x, y],
                                line,
                            }));
                        }
                        if sub == "POS" {
                            self.advance();
                            self.expect(&Token::To)?;
                            let x = self.parse_expression()?;
                            self.expect(&Token::Comma)?;
                            let y = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "GRAPHIC_GET_POS".to_string(),
                                args: vec![x, y],
                                line,
                            }));
                        }
                        if sub == "STRETCHMODE" {
                            self.advance();
                            self.expect(&Token::To)?;
                            let m = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "GRAPHIC_GET_STRETCHMODE".to_string(),
                                args: vec![m],
                                line,
                            }));
                        }
                        if sub == "CAPTION" {
                            self.advance();
                            self.expect(&Token::To)?;
                            let c = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "GRAPHIC_GET_CAPTION".to_string(),
                                args: vec![c],
                                line,
                            }));
                        }
                        if sub == "CANVAS" || sub == "DC" || sub == "MIX" {
                            self.advance();
                            self.expect(&Token::To)?;
                            let dst = self.parse_expression()?;
                            self.consume_to_eol();
                            let name = match sub.as_str() {
                                "CANVAS" => "GRAPHIC_GET_CANVAS",
                                "DC" => "GRAPHIC_GET_DC",
                                _ => "GRAPHIC_GET_MIX",
                            };
                            return Ok(Statement::Call(CallStmt {
                                name: name.to_string(),
                                args: vec![dst],
                                line,
                            }));
                        }
                        if sub == "SCROLLTEXT" {
                            // GRAPHIC GET SCROLLTEXT TO ScrollVar&   (batch 182)
                            self.advance();
                            self.expect(&Token::To)?;
                            let dst = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "GRAPHIC_GET_SCROLLTEXT".to_string(),
                                args: vec![dst],
                                line,
                            }));
                        }
                    }
                    if gop == "REDRAW" {
                        // GRAPHIC REDRAW  (batch 181)
                        self.advance();
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "GRAPHIC_REDRAW".to_string(),
                            args: vec![],
                            line,
                        }));
                    }
                    if gop == "SET" {
                        // GRAPHIC SET MIX (mix&) | GRAPHIC SET PIXEL (x,y),color | GRAPHIC SET TEXTALIGN (align&)
                        self.advance();
                        let sub = self.peek_plain_upper();
                        if sub == "SIZE" {
                            // GRAPHIC SET SIZE nWide&, nHigh&  (batch 65)
                            self.advance();
                            let w = self.parse_expression()?;
                            self.expect(&Token::Comma)?;
                            let h = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "GRAPHIC_SET_SIZE".to_string(),
                                args: vec![w, h],
                                line,
                            }));
                        }
                        if sub == "CLIP" {
                            // GRAPHIC SET CLIP l!, t!, r!, b!  (batch 65)
                            self.advance();
                            let mut args = Vec::new();
                            for k in 0..4 {
                                if k > 0 {
                                    self.expect(&Token::Comma)?;
                                }
                                args.push(self.parse_expression()?);
                            }
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "GRAPHIC_SET_CLIP".to_string(),
                                args,
                                line,
                            }));
                        }
                        if sub == "VIRTUAL" {
                            // GRAPHIC SET VIRTUAL nWide&, nHigh& [,USERSIZE]  (batch 65)
                            self.advance();
                            let mut args = vec![self.parse_expression()?];
                            while matches!(self.peek(), Token::Comma) {
                                self.advance();
                                if self.peek_plain_upper() == "USERSIZE" {
                                    self.advance();
                                    break;
                                }
                                args.push(self.parse_expression()?);
                            }
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "GRAPHIC_SET_VIRTUAL".to_string(),
                                args,
                                line,
                            }));
                        }
                        if sub == "FIXED" {
                            // GRAPHIC SET FIXED  (batch 66)
                            self.advance();
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "GRAPHIC_SET_FIXED".to_string(),
                                args: vec![],
                                line,
                            }));
                        }
                        if sub == "FONT" {
                            // GRAPHIC SET FONT fonthndl&  (batch 66)
                            self.advance();
                            let h = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "GRAPHIC_SET_FONT".to_string(),
                                args: vec![h],
                                line,
                            }));
                        }
                        if sub == "WORDWRAP" {
                            // GRAPHIC SET WORDWRAP n&  (batch 65)
                            self.advance();
                            let n = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "GRAPHIC_SET_WORDWRAP".to_string(),
                                args: vec![n],
                                line,
                            }));
                        }
                        if sub == "PIXEL" {
                            self.advance();
                            self.expect(&Token::LParen)?;
                            let x = self.parse_expression()?;
                            self.expect(&Token::Comma)?;
                            let y = self.parse_expression()?;
                            self.expect(&Token::RParen)?;
                            self.expect(&Token::Comma)?;
                            let color = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "GRAPHIC_SET_PIXEL".to_string(),
                                args: vec![x, y, color],
                                line,
                            }));
                        }
                        if sub == "TEXTALIGN" {
                            self.advance();
                            if matches!(self.peek(), Token::LParen) {
                                self.advance();
                            }
                            let align = self.parse_expression()?;
                            if matches!(self.peek(), Token::RParen) {
                                self.advance();
                            }
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "GRAPHIC_SET_TEXTALIGN".to_string(),
                                args: vec![align],
                                line,
                            }));
                        }
                        if sub == "BITS" {
                            // GRAPHIC SET BITS bitexpr$ — replace bitmap from DIB string (batch 64)
                            self.advance();
                            let src = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "GRAPHIC_SET_BITS".to_string(),
                                args: vec![src],
                                line,
                            }));
                        }
                        if sub == "AUTOSIZE" {
                            // GRAPHIC SET AUTOSIZE nWidth, nHeight [,USERSIZE]  (batch 64)
                            self.advance();
                            let mut args = vec![self.parse_expression()?];
                            while matches!(self.peek(), Token::Comma) {
                                self.advance();
                                if self.peek_plain_upper() == "USERSIZE" {
                                    self.advance();
                                    break;
                                }
                                args.push(self.parse_expression()?);
                            }
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "GRAPHIC_SET_AUTOSIZE".to_string(),
                                args,
                                line,
                            }));
                        }
                        if sub == "VIEW" || sub == "WRAP" {
                            // GRAPHIC SET VIEW x!, y! | GRAPHIC SET WRAP [n&]  (batch 63)
                            self.advance();
                            let a = self.parse_expression()?;
                            let mut args = vec![a];
                            if sub == "VIEW" {
                                self.expect(&Token::Comma)?;
                                let b = self.parse_expression()?;
                                args.push(b);
                            }
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: format!("GRAPHIC_SET_{}", sub),
                                args,
                                line,
                            }));
                        }
                        if sub == "POS" {
                            self.advance();
                            if matches!(self.peek(), Token::Step) {
                                self.advance();
                            }
                            self.expect(&Token::LParen)?;
                            let x = self.parse_expression()?;
                            self.expect(&Token::Comma)?;
                            let y = self.parse_expression()?;
                            self.expect(&Token::RParen)?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "GRAPHIC_SET_POS".to_string(),
                                args: vec![x, y],
                                line,
                            }));
                        }
                        if sub == "STRETCHMODE" {
                            self.advance();
                            let m = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "GRAPHIC_SET_STRETCHMODE".to_string(),
                                args: vec![m],
                                line,
                            }));
                        }
                        if sub == "CAPTION" {
                            self.advance();
                            let c = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "GRAPHIC_SET_CAPTION".to_string(),
                                args: vec![c],
                                line,
                            }));
                        }
                        if sub == "MIX" {
                            self.advance();
                            if matches!(self.peek(), Token::LParen) {
                                self.advance();
                            }
                            let mix = self.parse_expression()?;
                            if matches!(self.peek(), Token::RParen) {
                                self.advance();
                            }
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "GRAPHIC_SET_MIX".to_string(),
                                args: vec![mix],
                                line,
                            }));
                        }
                        if sub == "FOCUS" {
                            // GRAPHIC SET FOCUS  (batch 181)
                            self.advance();
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "GRAPHIC_SET_FOCUS".to_string(),
                                args: vec![],
                                line,
                            }));
                        }
                        if sub == "LOC" {
                            // GRAPHIC SET LOC x&, y&  (batch 181)
                            self.advance();
                            let x = self.parse_expression()?;
                            self.expect(&Token::Comma)?;
                            let y = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "GRAPHIC_SET_LOC".to_string(),
                                args: vec![x, y],
                                line,
                            }));
                        }
                        if sub == "CLIENT" {
                            // GRAPHIC SET CLIENT nWide&, nHigh&  (batch 181)
                            self.advance();
                            let w = self.parse_expression()?;
                            self.expect(&Token::Comma)?;
                            let h = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "GRAPHIC_SET_CLIENT".to_string(),
                                args: vec![w, h],
                                line,
                            }));
                        }
                        if sub == "OVERLAP" {
                            // GRAPHIC SET OVERLAP [NumrExpr&]  (batch 181)
                            self.advance();
                            let mut args = Vec::new();
                            if self.peek() != &Token::Eol && self.peek() != &Token::Eof {
                                args.push(self.parse_expression()?);
                            }
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "GRAPHIC_SET_OVERLAP".to_string(),
                                args,
                                line,
                            }));
                        }
                        if sub == "SCROLLTEXT" {
                            // GRAPHIC SET SCROLLTEXT [NumrExpr&]   (batch 182)
                            self.advance();
                            let mut args = Vec::new();
                            if self.peek() != &Token::Eol && self.peek() != &Token::Eof {
                                args.push(self.parse_expression()?);
                            }
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "GRAPHIC_SET_SCROLLTEXT".to_string(),
                                args,
                                line,
                            }));
                        }
                    }
                    if gop == "TEXT" {
                        // GRAPHIC TEXT SIZE txt$ TO w!, h!
                        self.advance();
                        self.expect(&Token::Identifier("SIZE".to_string()))?;
                        let txt = self.parse_expression()?;
                        self.expect(&Token::To)?;
                        let w = self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                        let h = self.parse_expression()?;
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "GRAPHIC_TEXT_SIZE".to_string(),
                            args: vec![txt, w, h],
                            line,
                        }));
                    }
                    if gop == "CHR" {
                        // GRAPHIC CHR SIZE (text$) TO w&, h&
                        self.advance();
                        self.expect(&Token::Identifier("SIZE".to_string()))?;
                        self.expect(&Token::LParen)?;
                        let txt = self.parse_expression()?;
                        self.expect(&Token::RParen)?;
                        self.expect(&Token::To)?;
                        let w = self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                        let h = self.parse_expression()?;
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "GRAPHIC_CHR_SIZE".to_string(),
                            args: vec![txt, w, h],
                            line,
                        }));
                    }
                    if gop == "CELL" {
                        // GRAPHIC CELL SIZE (rows,cols) TO w,h  |  GRAPHIC CELL (row,col) TO x,y
                        self.advance();
                        let sub = self.peek_plain_upper();
                        if sub == "SIZE" {
                            self.advance();
                        }
                        self.expect(&Token::LParen)?;
                        let a0 = self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                        let a1 = self.parse_expression()?;
                        self.expect(&Token::RParen)?;
                        self.expect(&Token::To)?;
                        let o0 = self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                        let o1 = self.parse_expression()?;
                        self.consume_to_eol();
                        let name = if sub == "SIZE" {
                            "GRAPHIC_CELL_SIZE".to_string()
                        } else {
                            "GRAPHIC_CELL".to_string()
                        };
                        return Ok(Statement::Call(CallStmt {
                            name,
                            args: vec![a0, a1, o0, o1],
                            line,
                        }));
                    }
                    if gop == "CIRCLE" {
                        // GRAPHIC CIRCLE (x,y), r [, color&]
                        self.advance();
                        let mut args = Vec::new();
                        self.expect(&Token::LParen)?;
                        args.push(self.parse_expression()?);
                        self.expect(&Token::Comma)?;
                        args.push(self.parse_expression()?);
                        self.expect(&Token::RParen)?;
                        self.expect(&Token::Comma)?;
                        args.push(self.parse_expression()?);
                        if self.peek() == &Token::Comma {
                            self.advance();
                            args.push(self.parse_expression()?);
                        }
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "GRAPHIC_CIRCLE".to_string(),
                            args,
                            line,
                        }));
                    }
                    if gop == "POLYGON" {
                        // GRAPHIC POLYGON (x1,y1)-(x2,y2)-... [, color&]
                        self.advance();
                        let mut args = Vec::new();
                        loop {
                            self.expect(&Token::LParen)?;
                            args.push(self.parse_expression()?);
                            self.expect(&Token::Comma)?;
                            args.push(self.parse_expression()?);
                            self.expect(&Token::RParen)?;
                            if self.peek() == &Token::Minus {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                        if self.peek() == &Token::Comma {
                            self.advance();
                            args.push(self.parse_expression()?);
                        }
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "GRAPHIC_POLYGON".to_string(),
                            args,
                            line,
                        }));
                    }
                    if gop == "SCALE" {
                        // GRAPHIC SCALE (x1!,y1!)-(x2!,y2!) | GRAPHIC SCALE PIXELS  (batch 64)
                        self.advance();
                        if matches!(self.peek(), Token::LParen) {
                            self.advance();
                            let x1 = self.parse_expression()?;
                            self.expect(&Token::Comma)?;
                            let y1 = self.parse_expression()?;
                            self.expect(&Token::RParen)?;
                            if matches!(self.peek(), Token::Minus) {
                                self.advance();
                            }
                            self.expect(&Token::LParen)?;
                            let x2 = self.parse_expression()?;
                            self.expect(&Token::Comma)?;
                            let y2 = self.parse_expression()?;
                            self.expect(&Token::RParen)?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "GRAPHIC_SCALE".to_string(),
                                args: vec![x1, y1, x2, y2],
                                line,
                            }));
                        }
                        // SCALE PIXELS
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "GRAPHIC_SCALE_PIXELS".to_string(),
                            args: vec![],
                            line,
                        }));
                    }
                    if gop == "GET" {
                        // GRAPHIC GET CLIENT TO w&, h& / GET LOC TO x&, y&
                        self.advance();
                        let sub = self.peek_plain_upper();
                        if sub == "CLIENT" {
                            self.advance();
                            self.expect(&Token::To)?;
                            let w = self.parse_expression()?;
                            self.expect(&Token::Comma)?;
                            let h = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "GRAPHIC_GET_CLIENT".to_string(),
                                args: vec![w, h],
                                line,
                            }));
                        } else if sub == "LOC" {
                            self.advance();
                            self.expect(&Token::To)?;
                            let x = self.parse_expression()?;
                            self.expect(&Token::Comma)?;
                            let y = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "GRAPHIC_GET_LOC".to_string(),
                                args: vec![x, y],
                                line,
                            }));
                        } else {
                            // GRAPHIC GET PIXEL (x,y) TO var&
                            // sub already matched; consume PIXEL only if present
                            if self.peek_plain_upper() == "PIXEL" {
                                self.advance();
                            }
                            if self.peek() != &Token::LParen {
                                self.consume_to_eol();
                                let gname = if sub.is_empty() {
                                    "GRAPHIC GET".to_string()
                                } else {
                                    format!("GRAPHIC GET {}", sub)
                                };
                                eprintln!(
                                    "Error: GRAPHIC GET: expected an argument list on line {}",
                                    line
                                );
                                self.error_count += 1;
                                return Ok(Statement::Noop(gname, line));
                            }
                            self.advance(); // (
                            let x = self.parse_expression()?;
                            self.expect(&Token::Comma)?;
                            let y = self.parse_expression()?;
                            self.expect(&Token::RParen)?;
                            self.expect(&Token::To)?;
                            let dst = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "GRAPHIC_GET_PIXEL".to_string(),
                                args: vec![x, y, dst],
                                line,
                            }));
                        }
                    }
                    if gop == "COLOR" {
                        self.advance();
                        let mut args = vec![self.parse_expression()?];
                        if self.peek() == &Token::Comma {
                            self.advance();
                            args.push(self.parse_expression()?);
                        }
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "GRAPHIC_COLOR".to_string(),
                            args,
                            line,
                        }));
                    }
                    if gop == "PRINT" {
                        // GRAPHIC PRINT [expr][;|, expr]... (batch 118)
                        self.advance(); // consume PRINT
                        let mut pargs = Vec::new();
                        while self.peek() != &Token::Eol
                            && self.peek() != &Token::Eof
                            && self.peek() != &Token::Colon
                        {
                            if self.peek() == &Token::Comma || self.peek() == &Token::Semicolon {
                                self.advance();
                                continue;
                            }
                            pargs.push(self.parse_expression()?);
                        }
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "GRAPHIC_PRINT".to_string(),
                            args: pargs,
                            line,
                        }));
                    }
                    if gop == "COPY" {
                        // GRAPHIC COPY (x1,y1)-(x2,y2), (x3,y3)
                        self.advance();
                        let mut args = Vec::new();
                        self.expect(&Token::LParen)?;
                        args.push(self.parse_expression()?);
                        self.expect(&Token::Comma)?;
                        args.push(self.parse_expression()?);
                        self.expect(&Token::RParen)?;
                        if self.peek() == &Token::Minus {
                            self.advance();
                        }
                        self.expect(&Token::LParen)?;
                        args.push(self.parse_expression()?);
                        self.expect(&Token::Comma)?;
                        args.push(self.parse_expression()?);
                        self.expect(&Token::RParen)?;
                        if self.peek() == &Token::Comma {
                            self.advance();
                        }
                        self.expect(&Token::LParen)?;
                        args.push(self.parse_expression()?);
                        self.expect(&Token::Comma)?;
                        args.push(self.parse_expression()?);
                        self.expect(&Token::RParen)?;
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "GRAPHIC_COPY".to_string(),
                            args,
                            line,
                        }));
                    }
                    if gop == "WIDTH" || gop == "STYLE" {
                        self.advance();
                        let args = if self.peek() != &Token::Eol && self.peek() != &Token::Eof {
                            vec![self.parse_expression()?]
                        } else {
                            Vec::new()
                        };
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: if gop == "STYLE" {
                                "GRAPHIC_STYLE".to_string()
                            } else {
                                "GRAPHIC_WIDTH".to_string()
                            },
                            args,
                            line,
                        }));
                    }
                    if gop == "SAVE" {
                        self.advance();
                        let args = vec![self.parse_expression()?];
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "GRAPHIC_SAVE".to_string(),
                            args,
                            line,
                        }));
                    }
                    if gop == "ARC" || gop == "PIE" {
                        // GRAPHIC ARC (x1,y1)-(x2,y2), start, end [, color& [, ...]]
                        self.advance();
                        let mut args = Vec::new();
                        self.expect(&Token::LParen)?;
                        args.push(self.parse_expression()?);
                        self.expect(&Token::Comma)?;
                        args.push(self.parse_expression()?);
                        self.expect(&Token::RParen)?;
                        self.expect(&Token::Minus)?;
                        self.expect(&Token::LParen)?;
                        args.push(self.parse_expression()?);
                        self.expect(&Token::Comma)?;
                        args.push(self.parse_expression()?);
                        self.expect(&Token::RParen)?;
                        self.expect(&Token::Comma)?;
                        args.push(self.parse_expression()?);
                        self.expect(&Token::Comma)?;
                        args.push(self.parse_expression()?);
                        while self.peek() == &Token::Comma {
                            self.advance();
                            args.push(self.parse_expression()?);
                        }
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: if gop == "ARC" {
                                "GRAPHIC_ARC".to_string()
                            } else {
                                "GRAPHIC_PIE".to_string()
                            },
                            args,
                            line,
                        }));
                    }
                    if gop == "POLYLINE" {
                        // GRAPHIC POLYLINE (x1,y1)-(x2,y2)-... [, color&]
                        self.advance();
                        let mut args = Vec::new();
                        while matches!(self.peek(), Token::LParen) {
                            self.advance();
                            args.push(self.parse_expression()?);
                            if self.peek() == &Token::Comma {
                                self.advance();
                                args.push(self.parse_expression()?);
                            }
                            if matches!(self.peek(), Token::RParen) {
                                self.advance();
                            }
                            if matches!(self.peek(), Token::Minus) {
                                self.advance();
                            }
                        }
                        while self.peek() == &Token::Comma {
                            self.advance();
                            args.push(self.parse_expression()?);
                        }
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "GRAPHIC_POLYLINE".to_string(),
                            args,
                            line,
                        }));
                    }
                    if gop == "PAINT" {
                        // GRAPHIC PAINT [BORDER|REPLACE] [STEP] (x,y) [, fill& [, border& [, style&]]]
                        self.advance();
                        let mut args = Vec::new();
                        loop {
                            let u = self.peek_plain_upper();
                            if u == "BORDER" || u == "REPLACE" || u == "STEP" {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                        self.expect(&Token::LParen)?;
                        args.push(self.parse_expression()?);
                        self.expect(&Token::Comma)?;
                        args.push(self.parse_expression()?);
                        self.expect(&Token::RParen)?;
                        while self.peek() == &Token::Comma {
                            self.advance();
                            args.push(self.parse_expression()?);
                        }
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "GRAPHIC_PAINT".to_string(),
                            args,
                            line,
                        }));
                    }
                    if gop == "LINE" || gop == "BOX" || gop == "ELLIPSE" {
                        self.advance();
                        let mut args = Vec::new();
                        // [STEP] (x1,y1) - [STEP] (x2,y2) [, color& [, ...]]
                        while matches!(self.peek(), Token::LParen) {
                            self.advance();
                            args.push(self.parse_expression()?);
                            if self.peek() == &Token::Comma {
                                self.advance();
                                args.push(self.parse_expression()?);
                            }
                            if matches!(self.peek(), Token::RParen) {
                                self.advance();
                            }
                            if matches!(self.peek(), Token::Minus) {
                                self.advance();
                            }
                        }
                        while self.peek() == &Token::Comma {
                            self.advance();
                            args.push(self.parse_expression()?);
                        }
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: match gop.as_str() {
                                "BOX" => "GRAPHIC_BOX".to_string(),
                                "ELLIPSE" => "GRAPHIC_ELLIPSE".to_string(),
                                _ => "GRAPHIC_LINE".to_string(),
                            },
                            args,
                            line,
                        }));
                    }
                    if gop == "DETACH" {
                        self.advance();
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "GRAPHIC_DETACH".to_string(),
                            args: Vec::new(),
                            line,
                        }));
                    }
                    if gop == "IMAGELIST" {
                        self.advance();
                        let mut args = Vec::new();
                        // GRAPHIC IMAGELIST (x!,y!), hLst, index&, overlay&, style&
                        if matches!(self.peek(), Token::LParen) {
                            self.advance();
                            args.push(self.parse_expression()?);
                            if self.peek() == &Token::Comma {
                                self.advance();
                                args.push(self.parse_expression()?);
                            }
                            if matches!(self.peek(), Token::RParen) {
                                self.advance();
                            }
                        }
                        while self.peek() == &Token::Comma {
                            self.advance();
                            args.push(self.parse_expression()?);
                        }
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "GRAPHIC_IMAGELIST".to_string(),
                            args,
                            line,
                        }));
                    }
                    if gop == "RENDER" {
                        self.advance();
                        let mut args = Vec::new();
                        // GRAPHIC RENDER [BITMAP | ICON] ImgName, (x1!,y1!)-(x2!,y2!)
                        let mut icon = false;
                        let director = self.peek_plain_upper();
                        if director == "ICON" {
                            icon = true;
                            self.advance();
                        } else if director == "BITMAP" {
                            self.advance();
                        }
                        args.push(self.parse_expression()?);
                        if self.peek() == &Token::Comma {
                            self.advance();
                        }
                        if matches!(self.peek(), Token::LParen) {
                            self.advance();
                            args.push(self.parse_expression()?);
                            if self.peek() == &Token::Comma {
                                self.advance();
                                args.push(self.parse_expression()?);
                            }
                            if matches!(self.peek(), Token::RParen) {
                                self.advance();
                            }
                        }
                        if matches!(self.peek(), Token::Minus) {
                            self.advance();
                        }
                        if matches!(self.peek(), Token::LParen) {
                            self.advance();
                            args.push(self.parse_expression()?);
                            if self.peek() == &Token::Comma {
                                self.advance();
                                args.push(self.parse_expression()?);
                            }
                            if matches!(self.peek(), Token::RParen) {
                                self.advance();
                            }
                        }
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: if icon {
                                "GRAPHIC_RENDER_ICON".to_string()
                            } else {
                                "GRAPHIC_RENDER".to_string()
                            },
                            args,
                            line,
                        }));
                    }
                    if gop == "SPLIT" {
                        self.advance();
                        let mut args = Vec::new();
                        // GRAPHIC SPLIT [WORD] MainStr, Part1Len To Part1Var, Part2Var
                        let mut word = false;
                        if self.peek_plain_upper() == "WORD" {
                            word = true;
                            self.advance();
                        }
                        args.push(self.parse_expression()?);
                        if self.peek() == &Token::Comma {
                            self.advance();
                        }
                        args.push(self.parse_expression()?);
                        if matches!(self.peek(), Token::To) {
                            self.advance();
                        }
                        args.push(self.parse_expression()?);
                        if self.peek() == &Token::Comma {
                            self.advance();
                        }
                        args.push(self.parse_expression()?);
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: if word {
                                "GRAPHIC_SPLIT_WORD".to_string()
                            } else {
                                "GRAPHIC_SPLIT".to_string()
                            },
                            args,
                            line,
                        }));
                    }
                    if gop == "STRETCH" {
                        self.advance();
                        let mut args = Vec::new();
                        // GRAPHIC STRETCH [PAGE] hBmp, ID
                        //     , (x1,y1)-(x2,y2) TO (x3,y3)-(x4,y4) [, Mix, Stretch]
                        let mut page = false;
                        if self.peek_plain_upper() == "PAGE" {
                            page = true;
                            self.advance();
                        }
                        args.push(self.parse_expression()?);
                        if self.peek() == &Token::Comma {
                            self.advance();
                        }
                        args.push(self.parse_expression()?);
                        if !page {
                            if self.peek() == &Token::Comma {
                                self.advance();
                            }
                            // up to FOUR pairs:
                            //   (x1,y1)-(x2,y2) TO (x3,y3)-(x4,y4)
                            // The first cut stopped after two, so the TO tail was
                            // discarded and the runtime got no destination rect.
                            for _pair in 0..4 {
                                if matches!(self.peek(), Token::LParen) {
                                    self.advance();
                                    args.push(self.parse_expression()?);
                                    if self.peek() == &Token::Comma {
                                        self.advance();
                                        args.push(self.parse_expression()?);
                                    }
                                    if matches!(self.peek(), Token::RParen) {
                                        self.advance();
                                    }
                                }
                                if matches!(self.peek(), Token::Minus) {
                                    self.advance();
                                }
                                if matches!(self.peek(), Token::To) {
                                    self.advance();
                                }
                                if !matches!(self.peek(), Token::LParen) {
                                    break;
                                }
                            }
                        }
                        while self.peek() == &Token::Comma {
                            self.advance();
                            args.push(self.parse_expression()?);
                        }
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: if page {
                                "GRAPHIC_STRETCH_PAGE".to_string()
                            } else {
                                "GRAPHIC_STRETCH".to_string()
                            },
                            args,
                            line,
                        }));
                    }
                    if gop == "INSTAT" {
                        // GRAPHIC INSTAT TO NumericVar   (batch 185)
                        self.advance();
                        let mut args = Vec::new();
                        if matches!(self.peek(), Token::To) {
                            self.advance();
                            args.push(self.parse_expression()?);
                        }
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "GRAPHIC_INSTAT".to_string(),
                            args,
                            line,
                        }));
                    }
                    if gop == "INKEY$" || gop == "INKEY" {
                        // GRAPHIC INKEY$ TO InkeyVar$   (batch 185)
                        self.advance();
                        let mut args = Vec::new();
                        if matches!(self.peek(), Token::To) {
                            self.advance();
                            args.push(self.parse_expression()?);
                        }
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "GRAPHIC_INKEY".to_string(),
                            args,
                            line,
                        }));
                    }
                    if gop == "WAITKEY$" || gop == "WAITKEY" {
                        // GRAPHIC WAITKEY$ [(KeyMask$ [, TimeOut&])] [TO WaitVar$]
                        // Always two fixed slots plus the optional destination, so the codegen
                        // arm can tell "no mask" (empty string = any key) from "no timeout"
                        // (-1 = wait indefinitely).
                        self.advance();
                        let mut mask = Expr::StringLit(String::new());
                        let mut timeout = Expr::IntegerLit(-1);
                        if self.peek() == &Token::LParen {
                            self.advance();
                            mask = self.parse_expression()?;
                            if self.peek() == &Token::Comma {
                                self.advance();
                                timeout = self.parse_expression()?;
                            }
                            if self.peek() == &Token::RParen {
                                self.advance();
                            }
                        }
                        let mut args = vec![mask, timeout];
                        if matches!(self.peek(), Token::To) {
                            self.advance();
                            args.push(self.parse_expression()?);
                        }
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "GRAPHIC_WAITKEY".to_string(),
                            args,
                            line,
                        }));
                    }
                    if gop == "LINE" {
                        // GRAPHIC LINE INPUT ["prompt"] string_variable   (batch 185)
                        self.advance();
                        if self.peek_graphic_word_upper() == "INPUT" {
                            self.advance();
                            let mut args = Vec::new();
                            args.push(self.parse_expression()?);
                            if !matches!(self.peek(), Token::Eol | Token::Eof) {
                                args.push(self.parse_expression()?);
                            }
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "GRAPHIC_LINE_INPUT".to_string(),
                                args,
                                line,
                            }));
                        }
                    }
                    if gop_kw == "INPUT" {
                        // GRAPHIC INPUT FLUSH            (no operands)
                        // GRAPHIC INPUT [prompt,] varlist
                        // (batch 185)
                        self.advance();
                        if self.peek_plain_upper() == "FLUSH" {
                            self.advance();
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "GRAPHIC_INPUT_FLUSH".to_string(),
                                args: Vec::new(),
                                line,
                            }));
                        }
                        // The docs define prompt as a quoted string or a string equate, and
                        // varlist as comma delimited, so a leading string operand followed by a
                        // comma is the prompt.  `GRAPHIC INPUT name$, age` is the one ambiguous
                        // spelling in the official syntax; it is read as a prompt.
                        let mut string_lead = matches!(self.peek(), Token::StringLiteral(_));
                        if let Token::Identifier(w) = self.peek() {
                            string_lead = w.ends_with('$');
                        }
                        let mut args = Vec::new();
                        if string_lead {
                            let first = self.parse_expression()?;
                            args.push(first);
                            if self.peek() == &Token::Comma {
                                self.advance();
                            }
                        }
                        loop {
                            args.push(self.parse_expression()?);
                            if self.peek() == &Token::Comma {
                                self.advance();
                                continue;
                            }
                            break;
                        }
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "GRAPHIC_INPUT".to_string(),
                            args,
                            line,
                        }));
                    }
                    if gop == "CLEAR" {
                        self.advance();
                        let mut args = Vec::new();
                        if self.peek() != &Token::Eol && self.peek() != &Token::Eof {
                            args.push(self.parse_expression()?);
                            while self.peek() == &Token::Comma {
                                self.advance();
                                args.push(self.parse_expression()?);
                            }
                        }
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "GRAPHIC_CLEAR".to_string(),
                            args,
                            line,
                        }));
                    }
                    if self.peek_plain_upper() == "BITMAP" {
                        self.advance();
                        let op = self.peek_plain_upper();
                        if op == "LOAD" {
                            // GRAPHIC BITMAP LOAD "file.bmp" TO hbmp
                            self.advance();
                            let fname = self.parse_expression()?;
                            if matches!(self.peek(), Token::To) {
                                self.advance();
                            }
                            let dst = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "GRAPHIC_BITMAP_LOAD".to_string(),
                                args: vec![fname, dst],
                                line,
                            }));
                        }
                        if op == "NEW" {
                            self.advance();
                            let mut args = vec![self.parse_expression()?];
                            while self.peek() == &Token::Comma {
                                self.advance();
                                args.push(self.parse_expression()?);
                            }
                            if matches!(self.peek(), Token::To) {
                                self.advance();
                                args.push(self.parse_expression()?);
                            }
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "GRAPHIC_BITMAP_NEW".to_string(),
                                args,
                                line,
                            }));
                        }
                        if matches!(self.peek(), Token::End) {
                            self.advance();
                            let mut args = Vec::new();
                            if self.peek() != &Token::Eol && self.peek() != &Token::Eof {
                                args.push(self.parse_expression()?);
                            }
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "GRAPHIC_BITMAP_END".to_string(),
                                args,
                                line,
                            }));
                        }
                        if op == "CAPTURE" {
                            // GRAPHIC BITMAP CAPTURE TO hBmp&   (batch 182, FORK EXTENSION)
                            self.advance();
                            self.expect(&Token::To)?;
                            let dst = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "GRAPHIC_BITMAP_CAPTURE".to_string(),
                                args: vec![dst],
                                line,
                            }));
                        }
                    }
                }

                // XPRINT (batch 68): host-based printer GDI operations
                if name_upper == "XPRINT" {
                    self.advance(); // consume XPRINT
                    let xop = self.peek_plain_upper();
                    if xop == "ATTACH" {
                        // XPRINT ATTACH {DEFAULT | PrinterName$} [, JobName$]
                        self.advance();
                        let first = self.peek_plain_upper();
                        let printer = if first == "DEFAULT" {
                            self.advance();
                            Expr::StringLit(String::new())
                        } else {
                            self.parse_expression()?
                        };
                        let mut args = vec![printer];
                        if self.peek() == &Token::Comma {
                            self.advance();
                            args.push(self.parse_expression()?);
                        }
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "XPRINT_ATTACH".to_string(),
                            args,
                            line,
                        }));
                    }
                    if xop == "CLOSE" {
                        self.advance();
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "XPRINT_CLOSE".to_string(),
                            args: vec![],
                            line,
                        }));
                    }
                    if xop == "SCALE" {
                        self.advance();
                        // batch 234: official XPRINT SCALE (x1,y1)-(x2,y2)
                        let mut args = self.parse_xprint_pt()?;
                        while self.peek() == &Token::Comma {
                            self.advance();
                            args.push(self.parse_expression()?);
                        }
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "XPRINT_SCALE".to_string(),
                            args,
                            line,
                        }));
                    }
                    if xop == "CELL" {
                        self.advance();
                        // XPRINT CELL SIZE TO w, h is the official form and must be tested
                        // before the two-argument XPRINT CELL, otherwise SIZE is parsed as a
                        // variable and the following TO raises "Expected Comma, got To".
                        if self.peek_plain_upper() == "SIZE" {
                            self.advance();
                            self.expect(&Token::To)?;
                            let w = self.parse_expression()?;
                            self.expect(&Token::Comma)?;
                            let h = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_CELL_SIZE".to_string(),
                                args: vec![w, h],
                                line,
                            }));
                        }
                        let x = self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                        let y = self.parse_expression()?;
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "XPRINT_CELL".to_string(),
                            args: vec![x, y],
                            line,
                        }));
                    }
                    if xop == "PREVIEW" {
                        self.advance();
                        // batch 234: official XPRINT PREVIEW CLOSE (no arguments)
                        if self.peek_plain_upper() == "CLOSE" {
                            self.advance();
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_PREVIEW_CLOSE".to_string(),
                                args: vec![],
                                line,
                            }));
                        }
                        let mode = if self.peek() != &Token::Eol && self.peek() != &Token::Eof {
                            self.parse_expression()?
                        } else {
                            Expr::IntegerLit(0)
                        };
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "XPRINT_PREVIEW".to_string(),
                            args: vec![mode],
                            line,
                        }));
                    }
                    if xop == "RENDER" {
                        self.advance();
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "XPRINT_RENDER".to_string(),
                            args: vec![],
                            line,
                        }));
                    }
                    if xop == "SPLIT" {
                        self.advance();
                        // batch 234: official XPRINT SPLIT "str", len TO s1$, s2$
                        let mut args = vec![self.parse_expression()?];
                        if self.peek() == &Token::Comma {
                            self.advance();
                            args.push(self.parse_expression()?);
                        }
                        self.expect(&Token::To)?;
                        args.push(self.parse_expression()?);
                        self.expect(&Token::Comma)?;
                        args.push(self.parse_expression()?);
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "XPRINT_SPLIT".to_string(),
                            args,
                            line,
                        }));
                    }
                    if xop == "STRETCH" {
                        self.advance();
                        let mut args = vec![];
                        for i in 0..8 {
                            args.push(self.parse_expression()?);
                            if i < 7 {
                                self.expect(&Token::Comma)?;
                            }
                        }
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "XPRINT_STRETCH".to_string(),
                            args,
                            line,
                        }));
                    }
                    if xop == "IMAGELIST" {
                        self.advance();
                        let mut args = vec![];
                        for i in 0..3 {
                            args.push(self.parse_expression()?);
                            if i < 2 {
                                self.expect(&Token::Comma)?;
                            }
                        }
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "XPRINT_IMAGELIST".to_string(),
                            args,
                            line,
                        }));
                    }
                    if xop == "POLYGON" {
                        self.advance();
                        let mut args = vec![];
                        while self.peek() != &Token::Eol && self.peek() != &Token::Eof {
                            args.push(self.parse_expression()?);
                            if self.peek() == &Token::Comma {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "XPRINT_POLYGON".to_string(),
                            args,
                            line,
                        }));
                    }
                    if xop == "POLYLINE" {
                        self.advance();
                        let mut args = vec![];
                        while self.peek() != &Token::Eol && self.peek() != &Token::Eof {
                            args.push(self.parse_expression()?);
                            if self.peek() == &Token::Comma {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "XPRINT_POLYLINE".to_string(),
                            args,
                            line,
                        }));
                    }
                    if xop == "COPY" {
                        self.advance();
                        let mut args = vec![];
                        for i in 0..6 {
                            args.push(self.parse_expression()?);
                            if i < 5 {
                                self.expect(&Token::Comma)?;
                            }
                        }
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "XPRINT_COPY".to_string(),
                            args,
                            line,
                        }));
                    }
                    if xop == "CHR" {
                        self.advance();
                        if self.peek_plain_upper() == "SIZE" {
                            self.advance();
                            self.expect(&Token::To)?;
                            let w = self.parse_expression()?;
                            self.expect(&Token::Comma)?;
                            let h = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_CHR_SIZE".to_string(),
                                args: vec![w, h],
                                line,
                            }));
                        }
                    }
                    if xop == "TEXT" {
                        self.advance();
                        if self.peek_plain_upper() == "SIZE" {
                            self.advance();
                            let txt = self.parse_expression()?;
                            self.expect(&Token::To)?;
                            let w = self.parse_expression()?;
                            self.expect(&Token::Comma)?;
                            let h = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_TEXT_SIZE".to_string(),
                                args: vec![txt, w, h],
                                line,
                            }));
                        }
                    }
                    if xop == "ARC" {
                        self.advance();
                        // batch 234: official (ARC) coordinate-pair form, e.g.
                        // XPRINT ARC (x1,y1)-(x2,y2)[, start, end]
                        let mut args = self.parse_xprint_pt()?;
                        while self.peek() == &Token::Comma {
                            self.advance();
                            args.push(self.parse_expression()?);
                        }
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "XPRINT_ARC".to_string(),
                            args,
                            line,
                        }));
                    }
                    if xop == "ELLIPSE" {
                        self.advance();
                        // batch 234: official (ELLIPSE) coordinate-pair form, e.g.
                        // XPRINT ELLIPSE (x1,y1)-(x2,y2)[, start, end]
                        let mut args = self.parse_xprint_pt()?;
                        while self.peek() == &Token::Comma {
                            self.advance();
                            args.push(self.parse_expression()?);
                        }
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "XPRINT_ELLIPSE".to_string(),
                            args,
                            line,
                        }));
                    }
                    if xop == "PIE" {
                        self.advance();
                        // batch 234: official (PIE) coordinate-pair form, e.g.
                        // XPRINT PIE (x1,y1)-(x2,y2)[, start, end]
                        let mut args = self.parse_xprint_pt()?;
                        while self.peek() == &Token::Comma {
                            self.advance();
                            args.push(self.parse_expression()?);
                        }
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "XPRINT_PIE".to_string(),
                            args,
                            line,
                        }));
                    }
                    if xop == "CANCEL" {
                        self.advance();
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "XPRINT_CANCEL".to_string(),
                            args: vec![],
                            line,
                        }));
                    }
                    if xop == "FORMFEED" {
                        self.advance();
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "XPRINT_FORMFEED".to_string(),
                            args: vec![],
                            line,
                        }));
                    }
                    if xop == "LINE" {
                        self.advance();
                        // batch 234: official XPRINT LINE (x1,y1)-(x2,y2)
                        let mut args = self.parse_xprint_pt()?;
                        while self.peek() == &Token::Comma {
                            self.advance();
                            args.push(self.parse_expression()?);
                        }
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "XPRINT_LINE".to_string(),
                            args,
                            line,
                        }));
                    }
                    if xop == "BOX" {
                        self.advance();
                        // batch 234: official XPRINT BOX (x1,y1)-(x2,y2)
                        let mut args = self.parse_xprint_pt()?;
                        while self.peek() == &Token::Comma {
                            self.advance();
                            args.push(self.parse_expression()?);
                        }
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "XPRINT_BOX".to_string(),
                            args,
                            line,
                        }));
                    }
                    if xop == "COLOR" {
                        // XPRINT COLOR r[, g[, b]] RGB components (batch 118)
                        self.advance(); // consume COLOR
                        let mut cargs = vec![self.parse_expression()?];
                        while self.peek() == &Token::Comma {
                            self.advance();
                            cargs.push(self.parse_expression()?);
                        }
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "XPRINT_COLOR".to_string(),
                            args: cargs,
                            line,
                        }));
                    }
                    if xop == "WIDTH" {
                        self.advance();
                        let w = self.parse_expression()?;
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "XPRINT_WIDTH".to_string(),
                            args: vec![w],
                            line,
                        }));
                    }
                    if xop == "STYLE" {
                        self.advance();
                        let st = self.parse_expression()?;
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "XPRINT_STYLE".to_string(),
                            args: vec![st],
                            line,
                        }));
                    }
                    if xop == "PRINT" {
                        self.advance();
                        let mut args = Vec::new();
                        while !matches!(self.peek(), Token::Eol | Token::Colon | Token::Eof) {
                            if matches!(self.peek(), Token::Comma | Token::Semicolon) {
                                self.advance();
                                continue;
                            }
                            args.push(self.parse_expression()?);
                        }
                        return Ok(Statement::Call(CallStmt {
                            name: "XPRINT_PRINT".to_string(),
                            args,
                            line,
                        }));
                    }
                    if xop == "GET" {
                        self.advance();
                        let sub = self.peek_plain_upper();
                        if sub == "PPI" || sub == "SIZE" {
                            // XPRINT GET PPI TO x&, y& | XPRINT GET SIZE TO w&, h&
                            self.advance();
                            self.expect(&Token::To)?;
                            let a = self.parse_expression()?;
                            self.expect(&Token::Comma)?;
                            let b = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: format!("XPRINT_GET_{}", sub),
                                args: vec![a, b],
                                line,
                            }));
                        }
                        if sub == "DC" {
                            // XPRINT GET DC TO hdc&
                            self.advance();
                            self.expect(&Token::To)?;
                            let dst = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_GET_DC".to_string(),
                                args: vec![dst],
                                line,
                            }));
                        }
                        if sub == "ATTACH" {
                            self.advance();
                            self.expect(&Token::To)?;
                            let v = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_GET_ATTACH".to_string(),
                                args: vec![v],
                                line,
                            }));
                        }
                        if sub == "POS" {
                            self.advance();
                            self.expect(&Token::To)?;
                            let x = self.parse_expression()?;
                            self.expect(&Token::Comma)?;
                            let y = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_GET_POS".to_string(),
                                args: vec![x, y],
                                line,
                            }));
                        }
                        if sub == "MIX" {
                            self.advance();
                            self.expect(&Token::To)?;
                            let v = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_GET_MIX".to_string(),
                                args: vec![v],
                                line,
                            }));
                        }
                        if sub == "STRETCHMODE" {
                            self.advance();
                            self.expect(&Token::To)?;
                            let v = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_GET_STRETCHMODE".to_string(),
                                args: vec![v],
                                line,
                            }));
                        }
                        if sub == "CLIENT" {
                            self.advance();
                            self.expect(&Token::To)?;
                            let w = self.parse_expression()?;
                            self.expect(&Token::Comma)?;
                            let h = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_GET_CLIENT".to_string(),
                                args: vec![w, h],
                                line,
                            }));
                        }
                        if sub == "CANVAS" {
                            self.advance();
                            self.expect(&Token::To)?;
                            let w = self.parse_expression()?;
                            self.expect(&Token::Comma)?;
                            let h = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_GET_CANVAS".to_string(),
                                args: vec![w, h],
                                line,
                            }));
                        }
                        if sub == "WRAP" {
                            self.advance();
                            self.expect(&Token::To)?;
                            let v = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_GET_WRAP".to_string(),
                                args: vec![v],
                                line,
                            }));
                        }
                        if sub == "WORDWRAP" {
                            self.advance();
                            self.expect(&Token::To)?;
                            let v = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_GET_WORDWRAP".to_string(),
                                args: vec![v],
                                line,
                            }));
                        }
                        if sub == "OVERLAP" {
                            self.advance();
                            self.expect(&Token::To)?;
                            let v = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_GET_OVERLAP".to_string(),
                                args: vec![v],
                                line,
                            }));
                        }
                        if sub == "CLIP" {
                            self.advance();
                            self.expect(&Token::To)?;
                            // batch 234: official XPRINT GET CLIP TO WidthVar!, HeightVar!
                            let mut args = vec![self.parse_expression()?];
                            while self.peek() == &Token::Comma {
                                self.advance();
                                args.push(self.parse_expression()?);
                            }
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_GET_CLIP".to_string(),
                                args,
                                line,
                            }));
                        }
                        if sub == "SCALE" {
                            self.advance();
                            self.expect(&Token::To)?;
                            // batch 234: official XPRINT GET SCALE TO x1!,y1!,x2!,y2!
                            let mut args = vec![self.parse_expression()?];
                            while self.peek() == &Token::Comma {
                                self.advance();
                                args.push(self.parse_expression()?);
                            }
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_GET_SCALE".to_string(),
                                args,
                                line,
                            }));
                        }
                        if sub == "LINES" {
                            self.advance();
                            self.expect(&Token::To)?;
                            let n = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_GET_LINES".to_string(),
                                args: vec![n],
                                line,
                            }));
                        }
                        if sub == "COPIES" {
                            self.advance();
                            self.expect(&Token::To)?;
                            let v = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_GET_COPIES".to_string(),
                                args: vec![v],
                                line,
                            }));
                        }
                        if sub == "ORIENTATION" {
                            self.advance();
                            self.expect(&Token::To)?;
                            let v = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_GET_ORIENTATION".to_string(),
                                args: vec![v],
                                line,
                            }));
                        }
                        if sub == "QUALITY" {
                            self.advance();
                            self.expect(&Token::To)?;
                            let v = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_GET_QUALITY".to_string(),
                                args: vec![v],
                                line,
                            }));
                        }
                        if sub == "DUPLEX" {
                            self.advance();
                            self.expect(&Token::To)?;
                            let v = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_GET_DUPLEX".to_string(),
                                args: vec![v],
                                line,
                            }));
                        }
                        if sub == "COLLATE" {
                            self.advance();
                            self.expect(&Token::To)?;
                            let v = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_GET_COLLATE".to_string(),
                                args: vec![v],
                                line,
                            }));
                        }
                        if sub == "COLORMODE" {
                            self.advance();
                            self.expect(&Token::To)?;
                            let v = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_GET_COLORMODE".to_string(),
                                args: vec![v],
                                line,
                            }));
                        }
                        if sub == "PAGES" {
                            self.advance();
                            self.expect(&Token::To)?;
                            let v = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_GET_PAGES".to_string(),
                                args: vec![v],
                                line,
                            }));
                        }
                        if sub == "SELECTION" {
                            self.advance();
                            self.expect(&Token::To)?;
                            let v = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_GET_SELECTION".to_string(),
                                args: vec![v],
                                line,
                            }));
                        }
                        if sub == "PAPER" {
                            self.advance();
                            self.expect(&Token::To)?;
                            let v = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_GET_PAPER".to_string(),
                                args: vec![v],
                                line,
                            }));
                        }
                        if sub == "TRAY" {
                            self.advance();
                            self.expect(&Token::To)?;
                            let v = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_GET_TRAY".to_string(),
                                args: vec![v],
                                line,
                            }));
                        }
                        if sub == "PAPERS" {
                            self.advance();
                            self.expect(&Token::To)?;
                            let v = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_GET_PAPERS".to_string(),
                                args: vec![v],
                                line,
                            }));
                        }
                        if sub == "TRAYS" {
                            self.advance();
                            self.expect(&Token::To)?;
                            let v = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_GET_TRAYS".to_string(),
                                args: vec![v],
                                line,
                            }));
                        }
                        if sub == "MARGIN" {
                            self.advance();
                            self.expect(&Token::To)?;
                            let l = self.parse_expression()?;
                            self.expect(&Token::Comma)?;
                            let t = self.parse_expression()?;
                            self.expect(&Token::Comma)?;
                            let r = self.parse_expression()?;
                            self.expect(&Token::Comma)?;
                            let b = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_GET_MARGIN".to_string(),
                                args: vec![l, t, r, b],
                                line,
                            }));
                        }
                        if sub == "COLOR" {
                            self.advance();
                            self.expect(&Token::To)?;
                            let c = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_GET_COLOR".to_string(),
                                args: vec![c],
                                line,
                            }));
                        }
                        if sub == "PIXEL" {
                            self.advance();
                            // batch 234: official XPRINT GET PIXEL (x,y) TO c&
                            let mut args = self.parse_xprint_pt()?;
                            self.expect(&Token::To)?;
                            args.push(self.parse_expression()?);
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_GET_PIXEL".to_string(),
                                args,
                                line,
                            }));
                        }
                        if sub == "TEXTALIGN" {
                            self.advance();
                            self.expect(&Token::To)?;
                            let a = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_GET_TEXTALIGN".to_string(),
                                args: vec![a],
                                line,
                            }));
                        }
                    }
                    if xop == "SET" {
                        self.advance();
                        let sub = self.peek_plain_upper();
                        if sub == "POS" {
                            self.advance();
                            // batch 234: official XPRINT SET POS (x!, y!)
                            let args = self.parse_xprint_pt()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_SET_POS".to_string(),
                                args,
                                line,
                            }));
                        }
                        if sub == "FONT" {
                            self.advance();
                            // batch 234: official XPRINT SET FONT fonthndl& -- a
                            // single handle from FONT NEW (legacy 4-arg kept).
                            let mut args = vec![self.parse_expression()?];
                            while self.peek() == &Token::Comma {
                                self.advance();
                                args.push(self.parse_expression()?);
                            }
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_SET_FONT".to_string(),
                                args,
                                line,
                            }));
                        }
                        if sub == "MIX" {
                            self.advance();
                            let v = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_SET_MIX".to_string(),
                                args: vec![v],
                                line,
                            }));
                        }
                        if sub == "STRETCHMODE" {
                            self.advance();
                            let v = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_SET_STRETCHMODE".to_string(),
                                args: vec![v],
                                line,
                            }));
                        }
                        if sub == "WRAP" {
                            self.advance();
                            let v = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_SET_WRAP".to_string(),
                                args: vec![v],
                                line,
                            }));
                        }
                        if sub == "WORDWRAP" {
                            self.advance();
                            let v = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_SET_WORDWRAP".to_string(),
                                args: vec![v],
                                line,
                            }));
                        }
                        if sub == "OVERLAP" {
                            self.advance();
                            let v = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_SET_OVERLAP".to_string(),
                                args: vec![v],
                                line,
                            }));
                        }
                        if sub == "CLIP" {
                            self.advance();
                            let x1 = self.parse_expression()?;
                            self.expect(&Token::Comma)?;
                            let y1 = self.parse_expression()?;
                            self.expect(&Token::Comma)?;
                            let x2 = self.parse_expression()?;
                            self.expect(&Token::Comma)?;
                            let y2 = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_SET_CLIP".to_string(),
                                args: vec![x1, y1, x2, y2],
                                line,
                            }));
                        }
                        if sub == "COPIES" {
                            self.advance();
                            let v = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_SET_COPIES".to_string(),
                                args: vec![v],
                                line,
                            }));
                        }
                        if sub == "ORIENTATION" {
                            self.advance();
                            let v = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_SET_ORIENTATION".to_string(),
                                args: vec![v],
                                line,
                            }));
                        }
                        if sub == "QUALITY" {
                            self.advance();
                            let v = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_SET_QUALITY".to_string(),
                                args: vec![v],
                                line,
                            }));
                        }
                        if sub == "DUPLEX" {
                            self.advance();
                            let v = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_SET_DUPLEX".to_string(),
                                args: vec![v],
                                line,
                            }));
                        }
                        if sub == "COLLATE" {
                            self.advance();
                            let v = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_SET_COLLATE".to_string(),
                                args: vec![v],
                                line,
                            }));
                        }
                        if sub == "COLORMODE" {
                            self.advance();
                            let v = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_SET_COLORMODE".to_string(),
                                args: vec![v],
                                line,
                            }));
                        }
                        if sub == "PAGES" {
                            self.advance();
                            let v = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_SET_PAGES".to_string(),
                                args: vec![v],
                                line,
                            }));
                        }
                        if sub == "PAPER" {
                            self.advance();
                            let v = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_SET_PAPER".to_string(),
                                args: vec![v],
                                line,
                            }));
                        }
                        if sub == "TRAY" {
                            self.advance();
                            let v = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_SET_TRAY".to_string(),
                                args: vec![v],
                                line,
                            }));
                        }
                        if sub == "COLOR" {
                            self.advance();
                            let c = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_SET_COLOR".to_string(),
                                args: vec![c],
                                line,
                            }));
                        }
                        if sub == "PIXEL" {
                            self.advance();
                            // batch 234: official XPRINT SET PIXEL (x,y), rgb&
                            let mut args = self.parse_xprint_pt()?;
                            while self.peek() == &Token::Comma {
                                self.advance();
                                args.push(self.parse_expression()?);
                            }
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_SET_PIXEL".to_string(),
                                args,
                                line,
                            }));
                        }
                        if sub == "TEXTALIGN" {
                            self.advance();
                            let a = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "XPRINT_SET_TEXTALIGN".to_string(),
                                args: vec![a],
                                line,
                            }));
                        }
                    }
                    // fallback: consume rest as noop for unimplemented XPRINT verbs
                    self.consume_to_eol();
                    eprintln!("Error: XPRINT: unsupported verb on line {}", line);
                    self.error_count += 1;
                    return Ok(Statement::Noop("XPRINT_".to_string() + &xop, line));
                }

                if name_upper == "MENU" {
                    self.advance(); // consume MENU
                    let op = self.peek_plain_upper();
                    if op == "NEW" {
                        self.advance();
                        let kind = self.peek_plain_upper();
                        if kind == "BAR" || kind == "POPUP" {
                            self.advance();
                        }
                        let mut args = Vec::new();
                        if matches!(self.peek(), Token::To) {
                            self.advance();
                            args.push(self.parse_expression()?);
                        }
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: if kind == "POPUP" {
                                "MENU_NEW_POPUP".to_string()
                            } else {
                                "MENU_NEW_BAR".to_string()
                            },
                            args,
                            line,
                        }));
                    }
                    if op == "ADD" {
                        self.advance();
                        let mut kind = String::new();
                        if matches!(self.peek(), Token::String_) {
                            self.advance();
                            kind = "STRING".to_string();
                        } else {
                            let k2 = self.peek_plain_upper();
                            if k2 == "POPUP" || k2 == "STRING" {
                                self.advance();
                                kind = k2;
                            }
                        }
                        // Official PB syntax separates the sub-keyword from the first argument
                        // with a comma:  MENU ADD STRING, hMenu, txt$, id&, state&  and
                        // MENU ADD POPUP, hMenu, txt$, hPopup, state&.
                        // Accept it (and the comma-less form) alike.
                        if matches!(self.peek(), Token::Comma) {
                            self.advance();
                        }
                        let mut args = Vec::new();
                        // First arg: hMenu (before first comma)
                        if !self.at_eol_or_eof() {
                            args.push(self.parse_expression()?);
                        }
                        while self.peek() == &Token::Comma {
                            self.advance();
                            args.push(self.parse_expression()?);
                        }
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: if kind == "POPUP" {
                                "MENU_ADD_POPUP".to_string()
                            } else {
                                "MENU_ADD_STRING".to_string()
                            },
                            args,
                            line,
                        }));
                    }
                    if op == "DELETE" {
                        self.advance();
                        let mut args = vec![self.parse_expression()?];
                        while self.peek() == &Token::Comma {
                            self.advance();
                            args.push(self.parse_expression()?);
                        }
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "MENU_DELETE".to_string(),
                            args,
                            line,
                        }));
                    }
                    if op == "GET" || op == "SET" {
                        self.advance();
                        let sub = self.peek_plain_upper();
                        let is_text = sub == "TEXT";
                        if is_text || sub == "STATE" {
                            self.advance();
                            let h = self.parse_expression()?;
                            let mut args = vec![h];
                            // optional BYCMD flag: MENU GET/SET ... h [, BYCMD] item TO dst
                            let bycmd;
                            if matches!(self.peek(), Token::Comma) {
                                self.advance();
                                if self.peek_plain_upper() == "BYCMD" {
                                    self.advance();
                                    bycmd = 1;
                                    // batch 234: official MENU GET/SET ... BYCMD id&
                                    // has NO comma after BYCMD (also accept legacy
                                    // `BYCMD, id` form).
                                    if matches!(self.peek(), Token::Comma) {
                                        self.advance();
                                    }
                                } else {
                                    bycmd = 0;
                                }
                            } else if self.peek_plain_upper() == "BYCMD" {
                                self.advance();
                                bycmd = 1;
                                if matches!(self.peek(), Token::Comma) {
                                    self.advance();
                                }
                            } else {
                                bycmd = 0;
                            }
                            args.push(Expr::IntegerLit(bycmd));
                            let pos = self.parse_expression()?;
                            args.push(pos);
                            if op == "GET" {
                                self.expect(&Token::To)?;
                                let dst = self.parse_expression()?;
                                args.push(dst);
                            } else {
                                self.expect(&Token::Comma)?;
                                let val = self.parse_expression()?;
                                args.push(val);
                            }
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: if is_text {
                                    if op == "GET" {
                                        "MENU_GET_TEXT".to_string()
                                    } else {
                                        "MENU_SET_TEXT".to_string()
                                    }
                                } else if op == "GET" {
                                    "MENU_GET_STATE".to_string()
                                } else {
                                    "MENU_SET_STATE".to_string()
                                },
                                args,
                                line,
                            }));
                        }
                    }
                    if op == "ATTACH" {
                        // MENU ATTACH hMenu, hDlg - attach a menu to a dialog (replacing
                        // any existing one; the dialog is redrawn to make room for it).
                        self.advance();
                        if matches!(self.peek(), Token::Comma) {
                            self.advance();
                        }
                        let hmenu = self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                        let hdlg = self.parse_expression()?;
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "MENU_ATTACH".to_string(),
                            args: vec![hmenu, hdlg],
                            line,
                        }));
                    }
                    if op == "CONTEXT" {
                        // MENU CONTEXT hContext, x&, y&, flags& TO CmdVar& - floating popup
                        // menu; the id the user picked comes back in CmdVar&.
                        self.advance();
                        if matches!(self.peek(), Token::Comma) {
                            self.advance();
                        }
                        let hmenu = self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                        let x = self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                        let y = self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                        let flags = self.parse_expression()?;
                        self.expect(&Token::To)?;
                        let dst = self.parse_expression()?;
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "MENU_CONTEXT".to_string(),
                            args: vec![hmenu, x, y, flags, dst],
                            line,
                        }));
                    }
                    if op == "DRAW" {
                        // MENU DRAW BAR hDlg - redraw a dialog's menu bar after it changed.
                        self.advance();
                        if self.peek_plain_upper() == "BAR" {
                            self.advance();
                        }
                        if matches!(self.peek(), Token::Comma) {
                            self.advance();
                        }
                        let hdlg = self.parse_expression()?;
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "MENU_DRAW_BAR".to_string(),
                            args: vec![hdlg],
                            line,
                        }));
                    }
                }

                // COLOR fore& [, back&] — console text attribute (PB/CC, batch 49)
                if name_upper == "COLOR" {
                    self.advance(); // consume COLOR
                    let mut args = Vec::new();
                    if self.peek() != &Token::Eol {
                        args.push(self.parse_expression()?);
                        if self.peek() == &Token::Comma {
                            self.advance();
                            args.push(self.parse_expression()?);
                        }
                    }
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "COLOR".to_string(),
                        args,
                        line,
                    }));
                }

                // MEMORY COPY src&, dst&, count& | MEMORY SWAP src&, dst&, count&
                // MEMORY FILL dst&, count&, BYTE|WORD|DWORD expr | MEMORY FILL dst&, count&, str$
                if name_upper == "MEMORY" {
                    self.advance(); // consume MEMORY
                    let op = self.peek_plain_upper();
                    if op == "COPY" || op == "SWAP" {
                        self.advance(); // consume COPY / SWAP
                        let src = self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                        let dst = self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                        let cnt = self.parse_expression()?;
                        return Ok(Statement::Call(CallStmt {
                            name: format!("MEMORY_{}", op),
                            args: vec![src, dst, cnt],
                            line,
                        }));
                    } else if op == "FILL" {
                        self.advance(); // consume FILL
                        let dst = self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                        let cnt = self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                        // optional BYTE|WORD|DWORD width prefix before the value
                        let w = self.peek_plain_upper();
                        if w == "BYTE" || w == "WORD" || w == "DWORD" {
                            self.advance(); // consume width keyword
                            let v = self.parse_expression()?;
                            let width = if w == "BYTE" {
                                1
                            } else if w == "WORD" {
                                2
                            } else {
                                4
                            };
                            return Ok(Statement::Call(CallStmt {
                                name: "MEMORY_FILL".to_string(),
                                args: vec![dst, cnt, v, Expr::IntegerLit(width as i64)],
                                line,
                            }));
                        }
                        let v = self.parse_expression()?;
                        return Ok(Statement::Call(CallStmt {
                            name: "MEMORY_FILLS".to_string(),
                            args: vec![dst, cnt, v],
                            line,
                        }));
                    }
                }

                // FONT NEW fontname$ [, points! , style& , charset& , pitch& , escapement& ] TO fhndl
                // FONT END fhndl
                if name_upper == "FONT" {
                    self.advance(); // consume FONT
                    let op = self.peek_plain_upper();
                    if op == "NEW" {
                        self.advance(); // consume NEW
                        let mut args = vec![self.parse_expression()?]; // fontname$
                                                                       // optional comma-separated args (points, style, charset, pitch, escapement)
                        while self.peek() == &Token::Comma {
                            self.advance();
                            args.push(self.parse_expression()?);
                        }
                        // TO fhndl (TO is a reserved-word token, not an Identifier)
                        if matches!(self.peek(), Token::To) {
                            self.advance();
                            args.push(self.parse_expression()?);
                        }
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "FONT_NEW".to_string(),
                            args,
                            line,
                        }));
                    }
                    if op == "END" {
                        self.advance(); // consume END
                        let args = vec![self.parse_expression()?];
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "FONT_END".to_string(),
                            args,
                            line,
                        }));
                    }
                }

                // IMAGELIST NEW BITMAP|ICON w&, h&, depth&, initial& TO hLst
                // IMAGELIST GET COUNT hLst TO dataValue&
                // IMAGELIST KILL hLst
                if name_upper == "IMAGELIST" {
                    self.advance(); // consume IMAGELIST
                    let op = self.peek_plain_upper();
                    if op == "NEW" {
                        self.advance(); // consume NEW
                        let kind = self.peek_plain_upper();
                        // The BITMAP / ICON word selects the statement name, so
                        // the two documented forms stay distinguishable in the
                        // IR.  An ICON list is not a different comctl32 object
                        // (see pb_imagelist_new in the runtime), but the word
                        // must not be thrown away on the way through either.
                        let is_icon = kind == "ICON";
                        if kind == "BITMAP" || kind == "ICON" {
                            self.advance();
                        }
                        let mut args = vec![self.parse_expression()?]; // nWidth
                        while self.peek() == &Token::Comma {
                            self.advance();
                            args.push(self.parse_expression()?);
                        }
                        // has_to travels as an explicit argument.  Deriving the
                        // TO target from the last argument broke the legal
                        // `IMAGELIST NEW BITMAP 16, 16, 32, n` form (no TO): the
                        // handle would have been stored into n.
                        let has_to = matches!(self.peek(), Token::To);
                        if has_to {
                            self.advance();
                        }
                        let to_arg = if has_to {
                            Some(self.parse_expression()?)
                        } else {
                            None
                        };
                        args.push(Expr::IntegerLit(if has_to { 1 } else { 0 }));
                        if let Some(t) = to_arg {
                            args.push(t);
                        }
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: if is_icon {
                                "IMAGELIST_NEW_ICON".to_string()
                            } else {
                                "IMAGELIST_NEW".to_string()
                            },
                            args,
                            line,
                        }));
                    }
                    if op == "GET" {
                        self.advance(); // consume GET
                        if self.peek_plain_upper() == "COUNT" {
                            self.advance(); // consume COUNT
                        }
                        let mut args = vec![self.parse_expression()?]; // hLst
                        if matches!(self.peek(), Token::To) {
                            self.advance();
                            args.push(self.parse_expression()?);
                        }
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "IMAGELIST_COUNT".to_string(),
                            args,
                            line,
                        }));
                    }
                    if op == "KILL" {
                        self.advance(); // consume KILL
                        let args = vec![self.parse_expression()?];
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "IMAGELIST_KILL".to_string(),
                            args,
                            line,
                        }));
                    }
                    // IMAGELIST ADD BITMAP hLst, hBmp [, hMsk] [TO dataValue&]
                    // IMAGELIST ADD BITMAP hLst, Bmp$ [, Msk$] [TO dataValue&]
                    // IMAGELIST ADD ICON   hLst, hIcn [TO dataValue&]
                    // IMAGELIST ADD ICON   hLst, Icn$  [TO dataValue&]
                    // IMAGELIST ADD MASKED hLst, hBmp, rgbColor& [TO dataValue&]
                    // IMAGELIST ADD MASKED hLst, Bmp$, rgbColor& [TO dataValue&]
                    // IMAGELIST SET OVERLAY hLst, image&, overlay&
                    if op == "ADD" || op == "SET" {
                        // The verb after ADD / SET is an identifier in every
                        // documented form, so the narrow peek is safe here.
                        let sub = match self.peek_at(1) {
                            Some(Token::Identifier(w)) => w.to_uppercase(),
                            _ => String::new(),
                        };
                        let form = if op == "ADD" {
                            match sub.as_str() {
                                "BITMAP" => 1,
                                "ICON" => 2,
                                "MASKED" => 3,
                                _ => 0,
                            }
                        } else if sub == "OVERLAY" {
                            4
                        } else {
                            0
                        };
                        if form == 4 {
                            self.advance(); // consume SET
                            self.advance(); // consume OVERLAY
                            let mut ops = vec![self.parse_expression()?]; // hLst
                            while self.peek() == &Token::Comma {
                                self.advance();
                                ops.push(self.parse_expression()?);
                            }
                            self.consume_to_eol();
                            if ops.len() != 3 {
                                return Err(PbError::parser(
                                    "IMAGELIST SET OVERLAY: expected hLst, image&, overlay&"
                                        .to_string(),
                                    None,
                                    line,
                                ));
                            }
                            return Ok(Statement::Call(CallStmt {
                                name: "IMAGELIST_SET_OVERLAY".to_string(),
                                args: ops,
                                line,
                            }));
                        }
                        if form != 0 {
                            self.advance(); // consume ADD
                            self.advance(); // consume BITMAP / ICON / MASKED
                            let mut ops = vec![self.parse_expression()?]; // hLst
                            while self.peek() == &Token::Comma {
                                self.advance();
                                ops.push(self.parse_expression()?);
                            }
                            if ops.len() < 2 {
                                return Err(PbError::parser(
                                    "IMAGELIST ADD: expected a handle and an image".to_string(),
                                    None,
                                    line,
                                ));
                            }
                            let has_to = matches!(self.peek(), Token::To);
                            let to_arg = if has_to {
                                self.advance();
                                Some(self.parse_expression()?)
                            } else {
                                None
                            };
                            self.consume_to_eol();
                            // A literal string operand selects the resource /
                            // disk-file form; anything else is a handle.
                            let is_name = matches!(ops.get(1), Some(Expr::StringLit(_)));
                            let stmt_name = match form {
                                1 => {
                                    if is_name {
                                        "IMAGELIST_ADD_BITMAP_FILE"
                                    } else {
                                        "IMAGELIST_ADD_BITMAP"
                                    }
                                }
                                2 => {
                                    if is_name {
                                        "IMAGELIST_ADD_ICON_FILE"
                                    } else {
                                        "IMAGELIST_ADD_ICON"
                                    }
                                }
                                _ => {
                                    if is_name {
                                        "IMAGELIST_ADD_MASKED_FILE"
                                    } else {
                                        "IMAGELIST_ADD_MASKED"
                                    }
                                }
                            };
                            let mut it = ops.into_iter();
                            let h_lst = it.next().unwrap_or(Expr::IntegerLit(0));
                            let img = it.next().unwrap_or(Expr::IntegerLit(0));
                            let third = it.next();
                            let mut args = vec![h_lst, img];
                            match form {
                                1 => args.push(match third {
                                    Some(e) => e,
                                    None => {
                                        if is_name {
                                            Expr::StringLit(String::new())
                                        } else {
                                            Expr::IntegerLit(0)
                                        }
                                    }
                                }),
                                3 => args.push(third.unwrap_or(Expr::IntegerLit(0))),
                                _ => {}
                            }
                            args.push(Expr::IntegerLit(if has_to { 1 } else { 0 }));
                            if let Some(t) = to_arg {
                                args.push(t);
                            }
                            return Ok(Statement::Call(CallStmt {
                                name: stmt_name.to_string(),
                                args,
                                line,
                            }));
                        }
                    }
                }

                // MAT a() = CON | CON(expr) | IDN | ZER | src() | src()+src() |
                //      src()-src() | src()*src() | (expr)*src() | INV(src()) | TRN(src())
                if name_upper == "MAT" {
                    self.advance(); // consume MAT
                    return self.parse_mat_statement(line);
                }

                // REGEXPR mask$ IN target$ [AT start&] TO iPos& [, iLen&]
                if name_upper == "REGEXPR" {
                    self.advance(); // consume REGEXPR
                    let mask = self.parse_expression()?;
                    // expect IN
                    if !matches!(self.peek(), Token::Identifier(id) if id.eq_ignore_ascii_case("IN"))
                    {
                        return Err(PbError::parser(
                            "REGEXPR: expected IN".to_string(),
                            self.current_file(),
                            self.current_line(),
                        ));
                    }
                    self.advance();
                    let target = self.parse_expression()?;
                    let mut start = None;
                    if matches!(self.peek(), Token::Identifier(id) if id.eq_ignore_ascii_case("AT"))
                    {
                        self.advance();
                        start = Some(self.parse_expression()?);
                    }
                    // expect TO
                    if !matches!(self.peek(), Token::To) {
                        return Err(PbError::parser(
                            "REGEXPR: expected TO".to_string(),
                            self.current_file(),
                            self.current_line(),
                        ));
                    }
                    self.advance();
                    let pos_var = self.parse_expression()?;
                    let mut len_var = None;
                    if matches!(self.peek(), Token::Comma) {
                        self.advance();
                        len_var = Some(self.parse_expression()?);
                    }
                    self.consume_to_eol();
                    return Ok(Statement::Regexpr {
                        mask,
                        target,
                        start,
                        pos_var,
                        len_var,
                    });
                }

                // REGREPL mask$ IN target$ WITH repl$ [AT start&] TO iPos&, newtarget$
                if name_upper == "REGREPL" {
                    self.advance(); // consume REGREPL
                    let mask = self.parse_expression()?;
                    if !matches!(self.peek(), Token::Identifier(id) if id.eq_ignore_ascii_case("IN"))
                    {
                        return Err(PbError::parser(
                            "REGREPL: expected IN".to_string(),
                            self.current_file(),
                            self.current_line(),
                        ));
                    }
                    self.advance();
                    let target = self.parse_expression()?;
                    if !matches!(self.peek(), Token::Identifier(id) if id.eq_ignore_ascii_case("WITH"))
                    {
                        return Err(PbError::parser(
                            "REGREPL: expected WITH".to_string(),
                            self.current_file(),
                            self.current_line(),
                        ));
                    }
                    self.advance();
                    let repl = self.parse_expression()?;
                    let mut start = None;
                    if matches!(self.peek(), Token::Identifier(id) if id.eq_ignore_ascii_case("AT"))
                    {
                        self.advance();
                        start = Some(self.parse_expression()?);
                    }
                    if !matches!(self.peek(), Token::To) {
                        return Err(PbError::parser(
                            "REGREPL: expected TO".to_string(),
                            self.current_file(),
                            self.current_line(),
                        ));
                    }
                    self.advance();
                    let pos_var = self.parse_expression()?;
                    if !matches!(self.peek(), Token::Comma) {
                        return Err(PbError::parser(
                            "REGREPL: expected , before target".to_string(),
                            self.current_file(),
                            self.current_line(),
                        ));
                    }
                    self.advance();
                    let out_var = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Regrepl {
                        mask,
                        target,
                        repl,
                        start,
                        pos_var,
                        out_var,
                    });
                }

                // PROFILE filename$ — dump per-procedure call counts + ms
                if name_upper == "PROFILE" {
                    self.advance(); // consume PROFILE
                    let filename = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Profile(filename));
                }

                // CALLSTK filename$ — dump call stack to a sequential file
                if name_upper == "CALLSTK" {
                    self.advance(); // consume CALLSTK
                    let filename = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::CallStk(filename));
                }

                // FIELD #n, size AS var [, size2 AS var2 ...]
                // FIELD dyn$, size AS var [, ...]
                // FIELD RESET var [, ...]  /  FIELD STRING var [, ...]
                if name_upper == "FIELD" {
                    self.advance(); // consume FIELD
                    return self.parse_field_statement(line);
                }

                // ASM opcode ... — inline assembly (keyword form). Rebuild the
                // line text from tokens (brackets, commas, identifiers, ints).
                if name_upper == "ASM" {
                    self.advance(); // consume ASM
                    let mut parts: Vec<String> = Vec::new();
                    loop {
                        match self.peek() {
                            Token::Eol | Token::Eof | Token::Colon => break,
                            Token::Semicolon => {
                                self.consume_to_eol();
                                break;
                            }
                            Token::Identifier(w) => {
                                parts.push(w.clone());
                                self.advance();
                            }
                            Token::IntegerLiteral(v) => {
                                parts.push(v.to_string());
                                self.advance();
                            }
                            Token::Minus => {
                                parts.push("-".to_string());
                                self.advance();
                            }
                            Token::Comma => {
                                parts.push(",".to_string());
                                self.advance();
                            }
                            Token::LBracket => {
                                parts.push("[".to_string());
                                self.advance();
                            }
                            Token::RBracket => {
                                parts.push("]".to_string());
                                self.advance();
                            }
                            _ => {
                                self.advance();
                            }
                        }
                    }
                    // Rebuild with Intel spacing: "MOV EAX, 123", negatives merged.
                    let mut out = String::new();
                    let mut prev_minus = false;
                    for pt in &parts {
                        if pt == "-" {
                            prev_minus = true;
                            continue;
                        }
                        if prev_minus {
                            out.push('-');
                            out.push_str(pt);
                            prev_minus = false;
                            continue;
                        }
                        if pt == "," {
                            out.push_str(", ");
                            continue;
                        }
                        if !out.is_empty() {
                            out.push(' ');
                        }
                        out.push_str(pt);
                    }
                    return Ok(Statement::Asm(out));
                }

                // DIR mask [, [ONLY] attr] TO s$ | DIR NEXT TO s$ | DIR CLOSE | DIR$ CLOSE
                if name_upper == "DIR" || name_upper == "DIR$" {
                    self.advance(); // consume DIR / DIR$
                    let up2 = self.peek_plain_upper();
                    if up2 == "CLOSE" || matches!(self.peek(), Token::Close) {
                        self.advance();
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "DIR CLOSE".to_string(),
                            args: Vec::new(),
                            line,
                        }));
                    }
                    if up2 == "NEXT" || matches!(self.peek(), Token::Next) {
                        self.advance();
                        self.expect(&Token::To)?;
                        let target = self.parse_expression()?;
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "DIR NEXT".to_string(),
                            args: vec![target],
                            line,
                        }));
                    }
                    // DIR mask [, [ONLY] attr] TO target$
                    let mask = self.parse_expression()?;
                    let mut args = vec![mask];
                    while self.peek() == &Token::Comma {
                        self.advance();
                        if let Token::Identifier(w) = self.peek() {
                            if w.eq_ignore_ascii_case("ONLY") {
                                self.advance();
                                let attr = self.parse_expression()?;
                                args.push(Expr::IntegerLit(1)); // ONLY flag
                                args.push(attr);
                                continue;
                            }
                        }
                        let attr = self.parse_expression()?;
                        args.push(attr);
                    }
                    let target = if self.peek() == &Token::To {
                        self.advance();
                        self.parse_expression()?
                    } else {
                        Expr::IntegerLit(0)
                    };
                    args.push(target);
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "DIR".to_string(),
                        args,
                        line,
                    }));
                }

                // TRY ... CATCH ... [FINALLY ...] END TRY  (batch 26)
                // Structured error trap: body until CATCH/FINALLY/END TRY,
                // catch until FINALLY/END TRY, finally until END TRY.
                if name_upper == "TRY" {
                    self.advance(); // consume TRY
                    self.skip_eol();
                    let line_start = line;
                    let mut try_body = Vec::new();
                    let mut catch_body = Vec::new();
                    let mut finally_body = Vec::new();
                    let mut in_catch = false;
                    let mut in_finally = false;
                    loop {
                        // Block keywords: CATCH / FINALLY / END TRY
                        if let Token::Identifier(ref kw) = self.peek().clone() {
                            let kw_upper = kw.to_uppercase();
                            if kw_upper == "CATCH" && !in_catch && !in_finally {
                                self.advance();
                                self.consume_to_eol();
                                self.skip_eol();
                                in_catch = true;
                                continue;
                            }
                            if kw_upper == "FINALLY" && !in_finally {
                                self.advance();
                                self.consume_to_eol();
                                self.skip_eol();
                                in_finally = true;
                                in_catch = false;
                                continue;
                            }
                        }
                        if let Token::End = self.peek() {
                            if let Some(Token::Identifier(ref kw)) = self.peek_at(1) {
                                if kw.eq_ignore_ascii_case("TRY") {
                                    self.advance(); // consume END
                                    self.advance(); // consume TRY
                                    self.skip_eol();
                                    return Ok(Statement::Try(TryStmt {
                                        body: try_body,
                                        catch: catch_body,
                                        finally: finally_body,
                                        line: line_start,
                                    }));
                                }
                            }
                        }
                        if self.peek() == &Token::Eof {
                            break;
                        }
                        let stmt = self.parse_statement()?;
                        if in_finally {
                            finally_body.push(stmt);
                        } else if in_catch {
                            catch_body.push(stmt);
                        } else {
                            try_body.push(stmt);
                        }
                        self.skip_eol();
                    }
                    return Ok(Statement::Try(TryStmt {
                        body: try_body,
                        catch: catch_body,
                        finally: finally_body,
                        line: line_start,
                    }));
                }

                // SWAP var1, var2 — exchange two variables
                if name_upper == "SWAP" {
                    self.advance(); // consume SWAP
                    let mut args = Vec::new();
                    if !self.at_eol_or_eof() {
                        args.push(self.parse_expression()?);
                        while self.peek() == &Token::Comma {
                            self.advance();
                            args.push(self.parse_expression()?);
                        }
                    }
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "SWAP".to_string(),
                        args,
                        line,
                    }));
                }

                // DDT statements: DIALOG, CONTROL, MENU, TOOLBAR, STATUSBAR
                // COMBOBOX, LISTBOX, TREEVIEW, LISTVIEW, XPRINT — DDT UI verbs
                // SLEEP, RANDOMIZE, MKDIR, BEEP — parse arg, emit as function call
                if matches!(
                    name_upper.as_str(),
                    "SLEEP" | "RANDOMIZE" | "MKDIR" | "RMDIR" | "CHDIR" | "BEEP"
                ) {
                    let call_name = name_upper.clone();
                    self.advance();
                    let args = if !self.at_eol_or_eof() {
                        vec![self.parse_expression()?]
                    } else {
                        Vec::new()
                    };
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: call_name,
                        args,
                        line,
                    }));
                }

                // DESKTOP GET SIZE/CLIENT/LOC/PPI TO ...
                if name_upper == "DESKTOP"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase() == "GET")
                {
                    let what = match self.peek_at(2) {
                        Some(Token::Identifier(w)) => w.to_uppercase(),
                        _ => String::new(),
                    };
                    if matches!(what.as_str(), "SIZE" | "CLIENT" | "LOC" | "PPI") {
                        self.advance(); // consume DESKTOP
                        self.advance(); // consume GET
                        self.advance(); // consume SIZE/CLIENT/LOC/PPI
                        if self.peek() == &Token::To {
                            self.advance(); // consume TO (reserved word)
                        }
                        let mut args = vec![self.parse_expression()?];
                        while self.peek() == &Token::Comma {
                            self.advance();
                            args.push(self.parse_expression()?);
                        }
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: format!("DESKTOP GET {what}"),
                            args,
                            line,
                        }));
                    }
                }

                // DISPLAY common dialogs (batch 118): wire parser to existing codegen/runtime.
                //   DISPLAY OPENFILE title$, filter$, initialdir$ TO result$
                //   DISPLAY SAVEFILE title$, filter$, initialdir$ TO result$
                //   DISPLAY BROWSE   title$, initialdir$ TO result$
                //   DISPLAY COLOR    TO result&
                //   DISPLAY FONT     TO result$
                if name_upper == "DISPLAY" {
                    if let Some(Token::Identifier(w)) = self.peek_at(1) {
                        let sub = w.to_uppercase();
                        if matches!(
                            sub.as_str(),
                            "OPENFILE" | "SAVEFILE" | "BROWSE" | "COLOR" | "FONT"
                        ) {
                            self.advance(); // consume DISPLAY
                            self.advance(); // consume sub-op
                            let mut args = Vec::new();
                            if sub == "COLOR" || sub == "FONT" {
                                if self.peek() == &Token::To {
                                    self.advance(); // consume TO
                                }
                                args.push(self.parse_expression()?);
                            } else {
                                // input string expressions, then `TO result`
                                args.push(self.parse_expression()?);
                                while self.peek() == &Token::Comma {
                                    self.advance();
                                    args.push(self.parse_expression()?);
                                }
                                if self.peek() == &Token::To {
                                    self.advance(); // consume TO
                                    args.push(self.parse_expression()?);
                                }
                            }
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: format!("DISPLAY_{sub}"),
                                args,
                                line,
                            }));
                        }
                    }
                }

                if matches!(name_upper.as_str(), "LSET" | "RSET") {
                    self.advance(); // consume LSET/RSET
                                    // LSET target$ = value  (and RSET)
                    let target = self.parse_primary()?;
                    if self.peek() == &Token::Eq {
                        self.advance();
                    }
                    let value = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: name_upper.clone(),
                        args: vec![target, value],
                        line,
                    }));
                }

                if matches!(name_upper.as_str(), "CSET") {
                    self.advance(); // consume CSET
                                    // CSET [ABS] result_var = string_expression [USING ...]
                                    // (ABS / USING not supported yet — plain centering only)
                    let target = self.parse_primary()?;
                    if self.peek() == &Token::Eq {
                        self.advance();
                    }
                    let value = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CSET".to_string(),
                        args: vec![target, value],
                        line,
                    }));
                }

                // WRITE #filenum, expr, ...  (CSV-style record)
                if name_upper == "WRITE" && self.peek_at(1) == Some(&Token::Hash) {
                    self.advance(); // consume WRITE
                    self.advance(); // consume #
                    let file_num = self.parse_expression()?;
                    let mut args = vec![file_num];
                    while self.peek() == &Token::Comma {
                        self.advance();
                        args.push(self.parse_expression()?);
                    }
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "WRITE".to_string(),
                        args,
                        line,
                    }));
                }
                // RESOURCE SAVE FILE resname$, filename$ (batch 118 wiring fix)
                if name_upper == "RESOURCE"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase() == "SAVE")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase() == "FILE")
                {
                    self.advance(); // RESOURCE
                    self.advance(); // SAVE
                    self.advance(); // FILE
                    let rn = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let fn_ = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "RESOURCE_SAVE_FILE".to_string(),
                        args: vec![rn, fn_],
                        line,
                    }));
                }
                // PLAY WAVE "file.wav" — two-word statement
                if name_upper == "PLAY"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase() == "WAVE")
                {
                    self.advance(); // consume PLAY
                    self.advance(); // consume WAVE
                    let mut args = vec![self.parse_expression()?];
                    while self.peek() == &Token::Comma {
                        self.advance();
                        args.push(self.parse_expression()?);
                    }
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "PLAY WAVE".to_string(),
                        args,
                        line,
                    }));
                }
                // PLAY SOUND freq&, dur& - two-word statement (batch 118 wiring fix)
                if name_upper == "PLAY"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase() == "SOUND")
                {
                    self.advance(); // consume PLAY
                    self.advance(); // consume SOUND
                    let mut args = vec![self.parse_expression()?];
                    while self.peek() == &Token::Comma {
                        self.advance();
                        args.push(self.parse_expression()?);
                    }
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "PLAY SOUND".to_string(),
                        args,
                        line,
                    }));
                }
                // MESSAGE LOOP — run Windows message pump
                if name_upper == "MESSAGE" && self.peek_at(1) == Some(&Token::Loop) {
                    self.advance();
                    self.advance();
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "PB_MESSAGE_LOOP".to_string(),
                        args: vec![],
                        line,
                    }));
                }
                // WINDOW SET TEXT hwnd, text$ / WINDOW GET TEXT hwnd TO var$
                if name_upper == "WINDOW"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase() == "SET")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase() == "TEXT")
                {
                    self.advance(); // consume WINDOW
                    self.advance(); // consume SET
                    self.advance(); // consume TEXT
                    let mut args = vec![self.parse_expression()?]; // hwnd
                    self.expect(&Token::Comma)?;
                    args.push(self.parse_expression()?); // text
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "WINDOW SET TEXT".to_string(),
                        args,
                        line,
                    }));
                }
                if name_upper == "WINDOW"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase() == "GET")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase() == "TEXT")
                {
                    self.advance(); // consume WINDOW
                    self.advance(); // consume GET
                    self.advance(); // consume TEXT
                    let mut args = vec![self.parse_expression()?]; // hwnd
                    self.expect(&Token::To)?;
                    args.push(self.parse_expression()?); // target var
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "WINDOW GET TEXT".to_string(),
                        args,
                        line,
                    }));
                }
                // WINDOW title$, x, y, w, h TO hWnd&  (Tier-3 DDT GUI #1)
                if name_upper == "WINDOW"
                    && !matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="SET" || w.to_uppercase()=="GET")
                {
                    self.advance(); // consume WINDOW
                    let title = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let x = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let y = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let w = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let h = self.parse_expression()?;
                    let target = if self.peek() == &Token::To {
                        self.advance();
                        self.parse_expression()?
                    } else {
                        Expr::IntegerLit(0)
                    };
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "WINDOW_NEW".to_string(),
                        args: vec![title, x, y, w, h, target],
                        line,
                    }));
                }
                // CONTROL ADD BUTTON, hWnd, id, "text", x, y, w, h
                //   [, [style&] [, [exstyle&]]] [[,] CALL callback] [, TO hCtrl&]
                //   (official PBWin.chw syntax; batch 243)
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="ADD")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="BUTTON")
                {
                    self.advance(); // CONTROL
                    self.advance(); // ADD
                    self.advance(); // BUTTON
                    self.expect(&Token::Comma)?;
                    let hwnd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let text = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let x = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let y = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let w = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let h = self.parse_expression()?;
                    let mut style: Option<Expr> = None;
                    let mut exstyle: Option<Expr> = None;
                    let mut callback: Option<Expr> = None;
                    // batch 243: official syntax is
                    //   [, [style&] [, [exstyle&]]] [[,] CALL callback]
                    // A CALL may follow with OR WITHOUT a comma, so a plain
                    // "14 CALL OkButton" (no comma before CALL) must also be
                    // captured - the old code consumed it with consume_to_eol.
                    loop {
                        if self.peek() == &Token::Comma {
                            self.advance();
                            if self.peek() == &Token::Call {
                                self.advance(); // consume CALL
                                callback = Some(self.parse_expression()?);
                                break;
                            } else if style.is_none() {
                                style = Some(self.parse_expression()?);
                            } else if exstyle.is_none() {
                                exstyle = Some(self.parse_expression()?);
                            } else {
                                break;
                            }
                        } else if self.peek() == &Token::Call && callback.is_none() {
                            self.advance(); // consume CALL
                            callback = Some(self.parse_expression()?);
                            break;
                        } else {
                            break;
                        }
                    }
                    let target = if matches!(self.peek(), Token::To) {
                        self.advance();
                        self.parse_expression()?
                    } else {
                        Expr::Variable("_ctl_dummy".to_string())
                    };
                    self.consume_to_eol();
                    // fixed 11-arg layout: [hwnd,id,text,x,y,w,h, style, exstyle, callback, target]
                    // style/exstyle default 0; callback placeholder "_no_cb" when absent
                    let mut args = vec![hwnd, id, text, x, y, w, h];
                    args.push(style.unwrap_or(Expr::IntegerLit(0)));
                    args.push(exstyle.unwrap_or(Expr::IntegerLit(0)));
                    args.push(callback.unwrap_or(Expr::Variable("_no_cb".to_string())));
                    args.push(target);
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_ADD_BUTTON".to_string(),
                        args,
                        line,
                    }));
                }
                // CONTROL ADD EDITBOX, hWnd, id, "text", x, y, w, h TO hCtrl&  (Tier-3 DDT GUI #3)
                //   NOTE: TEXTBOX is NOT an alias here - it has its own branch below
                //   (batch 177).  Re-adding it silently kills that branch again.
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="ADD")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="EDITBOX")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    self.expect(&Token::Comma)?;
                    let hwnd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let text = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let x = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let y = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let w = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let h = self.parse_expression()?;
                    // batch 249: [, [style&] [, [exstyle&]]] [[,] CALL callback] [TO]
                    let mut style: Option<Expr> = None;
                    let mut exstyle: Option<Expr> = None;
                    let mut callback: Option<Expr> = None;
                    loop {
                        if self.peek() == &Token::Comma {
                            self.advance();
                            if self.peek() == &Token::Call {
                                self.advance();
                                callback = Some(self.parse_expression()?);
                                break;
                            } else if style.is_none() {
                                style = Some(self.parse_expression()?);
                            } else if exstyle.is_none() {
                                exstyle = Some(self.parse_expression()?);
                            } else {
                                break;
                            }
                        } else if self.peek() == &Token::Call && callback.is_none() {
                            self.advance();
                            callback = Some(self.parse_expression()?);
                            break;
                        } else {
                            break;
                        }
                    }
                    let target = if matches!(self.peek(), Token::To) {
                        self.advance();
                        self.parse_expression()?
                    } else {
                        Expr::Variable("_ctl_dummy".to_string())
                    };
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_ADD_EDITBOX".to_string(),
                        args: vec![
                            hwnd,
                            id,
                            text,
                            x,
                            y,
                            w,
                            h,
                            style.unwrap_or(Expr::IntegerLit(0)),
                            exstyle.unwrap_or(Expr::IntegerLit(0)),
                            callback.unwrap_or(Expr::Variable("_no_cb".to_string())),
                            target,
                        ],
                        line,
                    }));
                }
                // CONTROL GET TEXT hCtrl TO var$  OR  CONTROL GET TEXT hDlg, id TO var$  (Tier-3 DDT GUI)
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="GET")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="TEXT")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    let first = self.parse_expression()?;
                    if matches!(self.peek(), Token::Comma) {
                        // By-ID form: CONTROL GET TEXT hDlg, id TO var$
                        self.advance(); // comma
                        let id = self.parse_expression()?;
                        self.expect(&Token::To)?;
                        let target = self.parse_expression()?;
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "CONTROL_GET_TEXT_ID".to_string(),
                            args: vec![first, id, target],
                            line,
                        }));
                    } else {
                        // By-handle form: CONTROL GET TEXT hCtrl TO var$
                        self.expect(&Token::To)?;
                        let target = self.parse_expression()?;
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "CONTROL_GET_TEXT".to_string(),
                            args: vec![first, target],
                            line,
                        }));
                    }
                }
                // CONTROL SET TEXT hCtrl, "text"  (Tier-3 DDT GUI)
                // CHW official by-id: CONTROL SET TEXT hDlg, id&, txt$ (batch 251)
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="SET")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="TEXT")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    let first = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let second = self.parse_expression()?;
                    if matches!(self.peek(), Token::Comma) {
                        // CHW official: CONTROL SET TEXT hDlg, id&, txt$
                        self.advance();
                        let text = self.parse_expression()?;
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "CONTROL_SET_TEXT_ID".to_string(),
                            args: vec![first, second, text],
                            line,
                        }));
                    }
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_SET_TEXT".to_string(),
                        args: vec![first, second],
                        line,
                    }));
                }
                // CONTROL GET CLIENT hDlg, id& TO a&, b&   (batch 175)
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="GET")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="CLIENT")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    let hd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::To)?;
                    let a = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let b = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_GET_CLIENT".to_string(),
                        args: vec![hd, id, a, b],
                        line,
                    }));
                }
                // CONTROL GET LOC hDlg, id& TO a&, b&   (batch 175)
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="GET")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="LOC")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    let hd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::To)?;
                    let a = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let b = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_GET_LOC".to_string(),
                        args: vec![hd, id, a, b],
                        line,
                    }));
                }
                // CONTROL GET SIZE hDlg, id& TO a&, b&   (batch 175)
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="GET")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="SIZE")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    let hd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::To)?;
                    let a = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let b = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_GET_SIZE".to_string(),
                        args: vec![hd, id, a, b],
                        line,
                    }));
                }
                // CONTROL SET CLIENT hDlg, id&, a&, b&   (batch 175)
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="SET")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="CLIENT")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    let hd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let a = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let b = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_SET_CLIENT".to_string(),
                        args: vec![hd, id, a, b],
                        line,
                    }));
                }
                // CONTROL SET LOC hDlg, id&, a&, b&   (batch 175)
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="SET")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="LOC")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    let hd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let a = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let b = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_SET_LOC".to_string(),
                        args: vec![hd, id, a, b],
                        line,
                    }));
                }
                // CONTROL SET SIZE hDlg, id&, a&, b&   (batch 175)
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="SET")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="SIZE")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    let hd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let a = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let b = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_SET_SIZE".to_string(),
                        args: vec![hd, id, a, b],
                        line,
                    }));
                }
                // --- Batch 176: CONTROL messages / state (official hDlg + id syntax) ---
                // CONTROL HANDLE hDlg, id& TO hCtl&
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="HANDLE")
                {
                    self.advance();
                    self.advance();
                    let hd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::To)?;
                    let tgt = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_HANDLE".to_string(),
                        args: vec![hd, id, tgt],
                        line,
                    }));
                }
                // CONTROL SEND hDlg, id&, Msg&, wParam&, lParam& [TO lResult&]
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="SEND")
                {
                    self.advance();
                    self.advance();
                    let hd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let msg = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let wp = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let lp = self.parse_expression()?;
                    let mut args = vec![hd, id, msg, wp, lp];
                    if matches!(self.peek(), Token::To) {
                        self.advance();
                        args.push(self.parse_expression()?);
                    }
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_SEND".to_string(),
                        args,
                        line,
                    }));
                }
                // CONTROL POST hDlg, id&, Msg&, wParam&, lParam&
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="POST")
                {
                    self.advance();
                    self.advance();
                    let hd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let msg = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let wp = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let lp = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_POST".to_string(),
                        args: vec![hd, id, msg, wp, lp],
                        line,
                    }));
                }
                // CONTROL REDRAW hDlg, id&
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="REDRAW")
                {
                    self.advance();
                    self.advance();
                    let hd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_REDRAW".to_string(),
                        args: vec![hd, id],
                        line,
                    }));
                }
                // CONTROL SET FOCUS hDlg, id&
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="SET")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="FOCUS")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    let hd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_SET_FOCUS".to_string(),
                        args: vec![hd, id],
                        line,
                    }));
                }
                // CONTROL SET FONT hDlg, id&, FontHndl&
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="SET")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="FONT")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    let hd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let fnt = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_SET_FONT".to_string(),
                        args: vec![hd, id, fnt],
                        line,
                    }));
                }
                // CONTROL SHOW STATE hDlg, id&, showstate& [TO lResult&]
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="SHOW")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="STATE")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    let hd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let st = self.parse_expression()?;
                    let mut args = vec![hd, id, st];
                    if matches!(self.peek(), Token::To) {
                        self.advance();
                        args.push(self.parse_expression()?);
                    }
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_SHOW_STATE".to_string(),
                        args,
                        line,
                    }));
                }
                // CONTROL NORMALIZE hDlg, id&
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="NORMALIZE")
                {
                    self.advance();
                    self.advance();
                    let hd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_NORMALIZE".to_string(),
                        args: vec![hd, id],
                        line,
                    }));
                }
                // CONTROL SET USER hDlg, id&, index&, usrval&
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="SET")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="USER")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    let hd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let idx = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let val = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_SET_USER".to_string(),
                        args: vec![hd, id, idx, val],
                        line,
                    }));
                }
                // CONTROL GET USER hDlg, id&, index& TO retvar&
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="GET")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="USER")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    let hd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let idx = self.parse_expression()?;
                    self.expect(&Token::To)?;
                    let tgt = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_GET_USER".to_string(),
                        args: vec![hd, id, idx, tgt],
                        line,
                    }));
                }
                // CONTROL SHOW/HIDE/ENABLE/DISABLE/FOCUS hCtrl&  (Tier-3 DDT GUI)
                // CHW official by-id form: CONTROL <VERB> hDlg, id& (batch 251)
                if name_upper == "CONTROL" {
                    if let Some(Token::Identifier(w)) = self.peek_at(1) {
                        let op = w.to_uppercase();
                        let mapping = [
                            ("SHOW", "CONTROL_SHOW"),
                            ("HIDE", "CONTROL_HIDE"),
                            ("ENABLE", "CONTROL_ENABLE"),
                            ("DISABLE", "CONTROL_DISABLE"),
                            ("FOCUS", "CONTROL_FOCUS"),
                        ];
                        for (kw, arm) in mapping {
                            if op == kw {
                                self.advance();
                                self.advance();
                                let first = self.parse_expression()?;
                                if matches!(self.peek(), Token::Comma) {
                                    // CHW official: CONTROL <VERB> hDlg, id&
                                    self.advance();
                                    let id = self.parse_expression()?;
                                    self.consume_to_eol();
                                    return Ok(Statement::Call(CallStmt {
                                        name: format!("{}_ID", arm),
                                        args: vec![first, id],
                                        line,
                                    }));
                                }
                                self.consume_to_eol();
                                return Ok(Statement::Call(CallStmt {
                                    name: arm.to_string(),
                                    args: vec![first],
                                    line,
                                }));
                            }
                        }
                    }
                }
                // CONTROL CHECK/UNCHECK hCtrl&  (Tier-3 DDT GUI)
                if name_upper == "CONTROL" {
                    if let Some(Token::Identifier(w)) = self.peek_at(1) {
                        let op = w.to_uppercase();
                        let mapping = [
                            ("CHECK", "CONTROL_CHECK", false),
                            ("UNCHECK", "CONTROL_UNCHECK", false),
                        ];
                        for (kw, arm, _is_get) in mapping {
                            if op == kw {
                                self.advance();
                                self.advance();
                                let hctrl = self.parse_expression()?;
                                self.consume_to_eol();
                                return Ok(Statement::Call(CallStmt {
                                    name: arm.to_string(),
                                    args: vec![hctrl],
                                    line,
                                }));
                            }
                        }
                        // CONTROL GET CHECK hCtrl& TO var&  (Tier-3 DDT GUI)
                        // CHW official by-id: CONTROL GET CHECK hDlg, id& TO lResult& (batch 251)
                        if op == "GET"
                            && matches!(self.peek_at(2), Some(Token::Identifier(w2)) if w2.to_uppercase()=="CHECK")
                        {
                            self.advance(); // CONTROL
                            self.advance(); // GET
                            self.advance(); // CHECK
                            let first = self.parse_expression()?;
                            if matches!(self.peek(), Token::Comma) {
                                self.advance(); // comma
                                let id = self.parse_expression()?;
                                self.expect(&Token::To)?;
                                let target = self.parse_expression()?;
                                self.consume_to_eol();
                                return Ok(Statement::Call(CallStmt {
                                    name: "CONTROL_GET_CHECK_ID".to_string(),
                                    args: vec![first, id, target],
                                    line,
                                }));
                            }
                            self.expect(&Token::To)?;
                            let target = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "CONTROL_GET_CHECK".to_string(),
                                args: vec![first, target],
                                line,
                            }));
                        }
                    }
                }
                // CONTROL CMD hCtrl, subname  (Tier-3 DDT GUI event dispatch)
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="CMD")
                {
                    self.advance(); // CONTROL
                    self.advance(); // CMD
                    let hctrl = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let subname = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_CMD".to_string(),
                        args: vec![hctrl, subname],
                        line,
                    }));
                }
                // CONTROL ADD HSCROLLBAR, hWnd, id, x, y, w, h TO hCtrl&
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="ADD")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="HSCROLLBAR")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    self.expect(&Token::Comma)?;
                    let parent = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;

                    // optional text$ slot (official CONTROL ADD): may be
                    // a string literal, an empty '', ,' slot, or absent
                    // (legacy fork form id, x, y, w, h).
                    if matches!(self.peek(), Token::StringLiteral(_)) {
                        self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                    } else if self.peek() == &Token::Comma {
                        // empty text slot: id, , x
                        self.advance();
                    }
                    let x = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let y = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let w = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let h = self.parse_expression()?;
                    let target = if self.peek() == &Token::To {
                        self.advance();
                        self.parse_expression()?
                    } else {
                        Expr::IntegerLit(0)
                    };
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_ADD_HSCROLLBAR".to_string(),
                        args: vec![parent, id, x, y, w, h, target],
                        line,
                    }));
                }
                // CONTROL ADD SCROLLBAR, hWnd, id, x, y, w, h TO hCtrl&
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="ADD")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="SCROLLBAR")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    self.expect(&Token::Comma)?;
                    let parent = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;

                    // optional text$ slot (official CONTROL ADD): may be
                    // a string literal, an empty '', ,' slot, or absent
                    // (legacy fork form id, x, y, w, h).  batch 253: keep the
                    // text and pass it through (CHW has a real text$ slot).
                    let mut text: Option<Expr> = None;
                    if matches!(self.peek(), Token::StringLiteral(_)) {
                        text = Some(self.parse_expression()?);
                        self.expect(&Token::Comma)?;
                    } else if self.peek() == &Token::Comma {
                        // empty text slot: id, , x
                        self.advance();
                        text = Some(Expr::StringLit(String::new()));
                    }
                    let x = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let y = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let w = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let h = self.parse_expression()?;
                    // batch 249: [, [style&] [, [exstyle&]]] [[,] CALL callback] [TO]
                    let mut style: Option<Expr> = None;
                    let mut exstyle: Option<Expr> = None;
                    let mut callback: Option<Expr> = None;
                    loop {
                        if self.peek() == &Token::Comma {
                            self.advance();
                            if self.peek() == &Token::Call {
                                self.advance();
                                callback = Some(self.parse_expression()?);
                                break;
                            } else if style.is_none() {
                                style = Some(self.parse_expression()?);
                            } else if exstyle.is_none() {
                                exstyle = Some(self.parse_expression()?);
                            } else {
                                break;
                            }
                        } else if self.peek() == &Token::Call && callback.is_none() {
                            self.advance();
                            callback = Some(self.parse_expression()?);
                            break;
                        } else {
                            break;
                        }
                    }
                    let target = if self.peek() == &Token::To {
                        self.advance();
                        self.parse_expression()?
                    } else {
                        Expr::IntegerLit(0)
                    };
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_ADD_SCROLLBAR".to_string(),
                        args: vec![
                            parent,
                            id,
                            text.unwrap_or(Expr::StringLit(String::new())),
                            x,
                            y,
                            w,
                            h,
                            style.unwrap_or(Expr::IntegerLit(0)),
                            exstyle.unwrap_or(Expr::IntegerLit(0)),
                            callback.unwrap_or(Expr::Variable("_no_cb".to_string())),
                            target,
                        ],
                        line,
                    }));
                }
                // CONTROL SET POS hCtrl, pos / CONTROL GET POS hCtrl TO var
                if name_upper == "CONTROL" {
                    if let Some(Token::Identifier(w)) = self.peek_at(1) {
                        let op = w.to_uppercase();
                        if op == "SET"
                            && matches!(self.peek_at(2), Some(Token::Identifier(w2)) if w2.to_uppercase()=="POS")
                        {
                            self.advance();
                            self.advance();
                            self.advance();
                            let hc = self.parse_expression()?;
                            self.expect(&Token::Comma)?;
                            let pos = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "CONTROL_SET_POS".to_string(),
                                args: vec![hc, pos],
                                line,
                            }));
                        }
                        if op == "GET"
                            && matches!(self.peek_at(2), Some(Token::Identifier(w2)) if w2.to_uppercase()=="POS")
                        {
                            self.advance();
                            self.advance();
                            self.advance();
                            let hc = self.parse_expression()?;
                            self.expect(&Token::To)?;
                            let target = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "CONTROL_GET_POS".to_string(),
                                args: vec![hc, target],
                                line,
                            }));
                        }
                    }
                }
                // CONTROL ADD COMBOBOX, hWnd, id&, [items$() | txt$ | <empty>], x, y, w, h
                //   [, [style&] [, [exstyle&]]] [[,] CALL callback] [, TO hCtrl&]
                //   (official PBWin.chw syntax; batch 248 - array/text slot +
                //   style/exstyle passthrough, mirroring CONTROL ADD LISTBOX)
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="ADD")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="COMBOBOX")
                {
                    self.advance(); // CONTROL
                    self.advance(); // ADD
                    self.advance(); // COMBOBOX
                    self.expect(&Token::Comma)?;
                    let hwnd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    // items slot: array ref (sArray()), string literal, or empty (id, , x)
                    let items = if self.peek() == &Token::Comma {
                        Expr::IntegerLit(0)
                    } else {
                        self.parse_expression()?
                    };
                    self.expect(&Token::Comma)?;
                    let x = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let y = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let w = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let h = self.parse_expression()?;
                    let mut style: Option<Expr> = None;
                    let mut exstyle: Option<Expr> = None;
                    let mut callback: Option<Expr> = None;
                    loop {
                        if self.peek() == &Token::Comma {
                            self.advance();
                            if self.peek() == &Token::Call {
                                self.advance(); // consume CALL
                                callback = Some(self.parse_expression()?);
                                break;
                            } else if style.is_none() {
                                style = Some(self.parse_expression()?);
                            } else if exstyle.is_none() {
                                exstyle = Some(self.parse_expression()?);
                            } else {
                                break;
                            }
                        } else if self.peek() == &Token::Call && callback.is_none() {
                            self.advance(); // consume CALL
                            callback = Some(self.parse_expression()?);
                            break;
                        } else {
                            break;
                        }
                    }
                    let target = if matches!(self.peek(), Token::To) {
                        self.advance();
                        self.parse_expression()?
                    } else {
                        Expr::Variable("_ctl_dummy".to_string())
                    };
                    self.consume_to_eol();
                    // fixed 11-arg layout: [hwnd,id,items,x,y,w,h, style, exstyle, callback, target]
                    let mut args = vec![hwnd, id, items, x, y, w, h];
                    args.push(style.unwrap_or(Expr::IntegerLit(0)));
                    args.push(exstyle.unwrap_or(Expr::IntegerLit(0)));
                    args.push(callback.unwrap_or(Expr::Variable("_no_cb".to_string())));
                    args.push(target);
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_ADD_COMBOBOX".to_string(),
                        args,
                        line,
                    }));
                }
                // CONTROL ADD LISTBOX, hWnd, id&, [items$() | txt$ | <empty>], x, y, w, h
                //   [, [style&] [, [exstyle&]]] [[,] CALL callback] [, TO hCtrl&]
                //   (official PBWin.chw syntax; batch 245 - array/text slot + style/exstyle
                //   passthrough, mirroring the CONTROL ADD BUTTON full syntax)
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="ADD")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="LISTBOX")
                {
                    self.advance(); // CONTROL
                    self.advance(); // ADD
                    self.advance(); // LISTBOX
                    self.expect(&Token::Comma)?;
                    let hwnd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    // items slot: array ref (sArray()), string literal, or empty (id, , x)
                    // "id, , x" is two commas: the first is the id separator, the
                    // peeked comma is the items->x separator - leave it for expect
                    // below so the empty slot does not consume it.
                    let items = if self.peek() == &Token::Comma {
                        Expr::IntegerLit(0)
                    } else {
                        self.parse_expression()?
                    };
                    self.expect(&Token::Comma)?;
                    let x = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let y = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let w = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let h = self.parse_expression()?;
                    let mut style: Option<Expr> = None;
                    let mut exstyle: Option<Expr> = None;
                    let mut callback: Option<Expr> = None;
                    loop {
                        if self.peek() == &Token::Comma {
                            self.advance();
                            if self.peek() == &Token::Call {
                                self.advance(); // consume CALL
                                callback = Some(self.parse_expression()?);
                                break;
                            } else if style.is_none() {
                                style = Some(self.parse_expression()?);
                            } else if exstyle.is_none() {
                                exstyle = Some(self.parse_expression()?);
                            } else {
                                break;
                            }
                        } else if self.peek() == &Token::Call && callback.is_none() {
                            self.advance(); // consume CALL
                            callback = Some(self.parse_expression()?);
                            break;
                        } else {
                            break;
                        }
                    }
                    let target = if matches!(self.peek(), Token::To) {
                        self.advance();
                        self.parse_expression()?
                    } else {
                        Expr::Variable("_ctl_dummy".to_string())
                    };
                    self.consume_to_eol();
                    // fixed 11-arg layout: [hwnd,id,items,x,y,w,h, style, exstyle, callback, target]
                    let mut args = vec![hwnd, id, items, x, y, w, h];
                    args.push(style.unwrap_or(Expr::IntegerLit(0)));
                    args.push(exstyle.unwrap_or(Expr::IntegerLit(0)));
                    args.push(callback.unwrap_or(Expr::Variable("_no_cb".to_string())));
                    args.push(target);
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_ADD_LISTBOX".to_string(),
                        args,
                        line,
                    }));
                }
                // LISTVIEW <sub-command> ...  (batch 158)
                if name_upper == "LISTVIEW" {
                    return self.parse_listview_statement(line);
                }
                // TREEVIEW <sub-command> ...  (batch 158)
                if name_upper == "TREEVIEW" {
                    return self.parse_treeview_statement(line);
                }
                // SCROLLBAR <GET|SET> <noun> ...  (batch 168)
                if name_upper == "SCROLLBAR" {
                    return self.parse_scrollbar_statement(line);
                }
                // COMBOBOX / LISTBOX <verb> [<noun>] ...  (batch 168)
                if name_upper == "COMBOBOX" {
                    return self.parse_cblb_statement(line, "COMBOBOX", 0);
                }
                if name_upper == "LISTBOX" {
                    return self.parse_cblb_statement(line, "LISTBOX", 1);
                }
                // TAB <verb> [<noun>] ...  (batch 169)
                // Guarded on a known verb so a variable that happens to be
                // named TAB is still parsed as an ordinary identifier.
                if name_upper == "TAB"
                    && (matches!(self.peek_at(1), Some(Token::Identifier(w))
                                 if matches!(w.to_uppercase().as_str(),
                                             "DELETE" | "GET" | "INSERT" | "RESET" | "SET"))
                        // TAB SELECT: the verb is the reserved word SELECT, which
                        // the lexer emits as Token::Select, not an identifier.
                        || matches!(self.peek_at(1), Some(Token::Select)))
                {
                    return self.parse_tab_statement(line);
                }
                // CONTROL ADD OPTION, hWnd, id, "text", x, y, w, h [[,] CALL cb] [TO hCtrl&]
                //   (batch 177)  Official source: control_add_option.htm.
                //   The TO clause is optional (the official syntax shows only
                //   [[,] CALL callback]); txt$ may be omitted for a LINE.
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="ADD")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="OPTION")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    self.expect(&Token::Comma)?;
                    let hwnd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let text = if matches!(self.peek(), Token::Comma) {
                        Expr::StringLit(String::new())
                    } else {
                        self.parse_expression()?
                    };
                    self.expect(&Token::Comma)?;
                    let x = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let y = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let w = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let h = self.parse_expression()?;
                    let mut style: Option<Expr> = None;
                    let mut exstyle: Option<Expr> = None;
                    let mut callback: Option<Expr> = None;
                    loop {
                        if self.peek() == &Token::Comma {
                            self.advance();
                            if self.peek() == &Token::Call {
                                self.advance();
                                callback = Some(self.parse_expression()?);
                                break;
                            } else if style.is_none() {
                                style = Some(self.parse_expression()?);
                            } else if exstyle.is_none() {
                                exstyle = Some(self.parse_expression()?);
                            } else {
                                break;
                            }
                        } else if self.peek() == &Token::Call && callback.is_none() {
                            self.advance();
                            callback = Some(self.parse_expression()?);
                            break;
                        } else {
                            break;
                        }
                    }
                    let target = if matches!(self.peek(), Token::To) {
                        self.advance();
                        self.parse_expression()?
                    } else {
                        Expr::Variable("_ctl_dummy".to_string())
                    };
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_ADD_OPTION".to_string(),
                        args: vec![
                            hwnd,
                            id,
                            text,
                            x,
                            y,
                            w,
                            h,
                            style.unwrap_or(Expr::IntegerLit(0)),
                            exstyle.unwrap_or(Expr::IntegerLit(0)),
                            callback.unwrap_or(Expr::Variable("_no_cb".to_string())),
                            target,
                        ],
                        line,
                    }));
                }
                // CONTROL ADD CHECK3STATE, hWnd, id, "text", x, y, w, h [[,] CALL cb] [TO hCtrl&]
                //   (batch 177)  Official source: control_add_check3state.htm.
                //   The TO clause is optional (the official syntax shows only
                //   [[,] CALL callback]); txt$ may be omitted for a LINE.
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="ADD")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="CHECK3STATE")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    self.expect(&Token::Comma)?;
                    let hwnd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let text = if matches!(self.peek(), Token::Comma) {
                        Expr::StringLit(String::new())
                    } else {
                        self.parse_expression()?
                    };
                    self.expect(&Token::Comma)?;
                    let x = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let y = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let w = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let h = self.parse_expression()?;
                    let mut style: Option<Expr> = None;
                    let mut exstyle: Option<Expr> = None;
                    let mut callback: Option<Expr> = None;
                    loop {
                        if self.peek() == &Token::Comma {
                            self.advance();
                            if self.peek() == &Token::Call {
                                self.advance();
                                callback = Some(self.parse_expression()?);
                                break;
                            } else if style.is_none() {
                                style = Some(self.parse_expression()?);
                            } else if exstyle.is_none() {
                                exstyle = Some(self.parse_expression()?);
                            } else {
                                break;
                            }
                        } else if self.peek() == &Token::Call && callback.is_none() {
                            self.advance();
                            callback = Some(self.parse_expression()?);
                            break;
                        } else {
                            break;
                        }
                    }
                    let target = if matches!(self.peek(), Token::To) {
                        self.advance();
                        self.parse_expression()?
                    } else {
                        Expr::Variable("_ctl_dummy".to_string())
                    };
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_ADD_CHECK3STATE".to_string(),
                        args: vec![
                            hwnd,
                            id,
                            text,
                            x,
                            y,
                            w,
                            h,
                            style.unwrap_or(Expr::IntegerLit(0)),
                            exstyle.unwrap_or(Expr::IntegerLit(0)),
                            callback.unwrap_or(Expr::Variable("_no_cb".to_string())),
                            target,
                        ],
                        line,
                    }));
                }
                // CONTROL ADD FRAME, hWnd, id, "text", x, y, w, h [[,] CALL cb] [TO hCtrl&]
                //   (batch 177)  Official source: control_add_frame.htm.
                //   The TO clause is optional (the official syntax shows only
                //   [[,] CALL callback]); txt$ may be omitted for a LINE.
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="ADD")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="FRAME")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    self.expect(&Token::Comma)?;
                    let hwnd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let text = if matches!(self.peek(), Token::Comma) {
                        Expr::StringLit(String::new())
                    } else {
                        self.parse_expression()?
                    };
                    self.expect(&Token::Comma)?;
                    let x = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let y = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let w = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let h = self.parse_expression()?;
                    let mut style: Option<Expr> = None;
                    let mut exstyle: Option<Expr> = None;
                    let mut callback: Option<Expr> = None;
                    loop {
                        if self.peek() == &Token::Comma {
                            self.advance();
                            if self.peek() == &Token::Call {
                                self.advance();
                                callback = Some(self.parse_expression()?);
                                break;
                            } else if style.is_none() {
                                style = Some(self.parse_expression()?);
                            } else if exstyle.is_none() {
                                exstyle = Some(self.parse_expression()?);
                            } else {
                                break;
                            }
                        } else if self.peek() == &Token::Call && callback.is_none() {
                            self.advance();
                            callback = Some(self.parse_expression()?);
                            break;
                        } else {
                            break;
                        }
                    }
                    let target = if matches!(self.peek(), Token::To) {
                        self.advance();
                        self.parse_expression()?
                    } else {
                        Expr::Variable("_ctl_dummy".to_string())
                    };
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_ADD_FRAME".to_string(),
                        args: vec![
                            hwnd,
                            id,
                            text,
                            x,
                            y,
                            w,
                            h,
                            style.unwrap_or(Expr::IntegerLit(0)),
                            exstyle.unwrap_or(Expr::IntegerLit(0)),
                            callback.unwrap_or(Expr::Variable("_no_cb".to_string())),
                            target,
                        ],
                        line,
                    }));
                }
                // CONTROL ADD IMAGE, hWnd, id, image$, x, y, w, h [[,] CALL cb] [TO hCtrl&]
                //   (batch 178)  Official source: control_add_image.htm.
                //   image$ is "#998" for an integral resource id, otherwise the
                //   resource name; the format is discovered at run time.  As with every
                //   other CONTROL ADD form in this fork, a CALL clause is consumed and
                //   ignored - control callbacks are not wired to the parser yet.
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="ADD")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="IMAGE")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    self.expect(&Token::Comma)?;
                    let hwnd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let image = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let x = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let y = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let w = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let h = self.parse_expression()?;
                    let target = if matches!(self.peek(), Token::To) {
                        self.advance();
                        self.parse_expression()?
                    } else {
                        Expr::Variable("_ctl_dummy".to_string())
                    };
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_ADD_IMAGE".to_string(),
                        args: vec![hwnd, id, image, x, y, w, h, target],
                        line,
                    }));
                }
                // CONTROL ADD IMAGEX, hWnd, id, image$, x, y, w, h [[,] CALL cb] [TO hCtrl&]
                //   (batch 178)  Official source: control_add_imagex.htm.
                //   image$ is "#998" for an integral resource id, otherwise the
                //   resource name; the format is discovered at run time.  As with every
                //   other CONTROL ADD form in this fork, a CALL clause is consumed and
                //   ignored - control callbacks are not wired to the parser yet.
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="ADD")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="IMAGEX")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    self.expect(&Token::Comma)?;
                    let hwnd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let image = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let x = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let y = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let w = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let h = self.parse_expression()?;
                    let target = if matches!(self.peek(), Token::To) {
                        self.advance();
                        self.parse_expression()?
                    } else {
                        Expr::Variable("_ctl_dummy".to_string())
                    };
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_ADD_IMAGEX".to_string(),
                        args: vec![hwnd, id, image, x, y, w, h, target],
                        line,
                    }));
                }
                // CONTROL ADD IMGBUTTON, hWnd, id, image$, x, y, w, h [[,] CALL cb] [TO hCtrl&]
                //   (batch 178)  Official source: control_add_imgbutton.htm.
                //   image$ is "#998" for an integral resource id, otherwise the
                //   resource name; the format is discovered at run time.  As with every
                //   other CONTROL ADD form in this fork, a CALL clause is consumed and
                //   ignored - control callbacks are not wired to the parser yet.
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="ADD")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="IMGBUTTON")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    self.expect(&Token::Comma)?;
                    let hwnd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let image = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let x = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let y = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let w = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let h = self.parse_expression()?;
                    let target = if matches!(self.peek(), Token::To) {
                        self.advance();
                        self.parse_expression()?
                    } else {
                        Expr::Variable("_ctl_dummy".to_string())
                    };
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_ADD_IMGBUTTON".to_string(),
                        args: vec![hwnd, id, image, x, y, w, h, target],
                        line,
                    }));
                }
                // CONTROL ADD IMGBUTTONX, hWnd, id, image$, x, y, w, h [[,] CALL cb] [TO hCtrl&]
                //   (batch 178)  Official source: control_add_imgbuttonx.htm.
                //   image$ is "#998" for an integral resource id, otherwise the
                //   resource name; the format is discovered at run time.  As with every
                //   other CONTROL ADD form in this fork, a CALL clause is consumed and
                //   ignored - control callbacks are not wired to the parser yet.
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="ADD")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="IMGBUTTONX")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    self.expect(&Token::Comma)?;
                    let hwnd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let image = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let x = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let y = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let w = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let h = self.parse_expression()?;
                    let target = if matches!(self.peek(), Token::To) {
                        self.advance();
                        self.parse_expression()?
                    } else {
                        Expr::Variable("_ctl_dummy".to_string())
                    };
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_ADD_IMGBUTTONX".to_string(),
                        args: vec![hwnd, id, image, x, y, w, h, target],
                        line,
                    }));
                }
                // CONTROL SET COLOR hDlg, id&, foreclr&, backclr&   (batch 179)
                //   Official source: control_set_color.htm.  Four operands and, as
                //   for every other CONTROL SET <noun> form, no comma between the
                //   noun and hDlg.  foreclr& = -1& and backclr& = -1& mean "use the
                //   default colour"; backclr& = -2& means the text background is
                //   not painted at all, so whatever is behind the text shows.
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="SET")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="COLOR")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    let hwnd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let fore = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let back = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_SET_COLOR".to_string(),
                        args: vec![hwnd, id, fore, back],
                        line,
                    }));
                }
                // CONTROL SET IMAGE hDlg, id&, newimage$   (batch 178)
                //   Official source: control_set_image.htm.  The replacement must be the same format
                //   as the image already displayed; both SET forms are otherwise alike,
                //   the X form re-sizing the image to the control.
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="SET")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="IMAGE")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    let hwnd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let image = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_SET_IMAGE".to_string(),
                        args: vec![hwnd, id, image],
                        line,
                    }));
                }
                // CONTROL SET IMAGEX hDlg, id&, newimage$   (batch 178)
                //   Official source: control_set_imagex.htm.  The replacement must be the same format
                //   as the image already displayed; both SET forms are otherwise alike,
                //   the X form re-sizing the image to the control.
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="SET")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="IMAGEX")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    let hwnd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let image = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_SET_IMAGEX".to_string(),
                        args: vec![hwnd, id, image],
                        line,
                    }));
                }
                // CONTROL SET IMGBUTTON hDlg, id&, newimage$   (batch 178)
                //   Official source: control_set_imgbutton.htm.  The replacement must be the same format
                //   as the image already displayed; both SET forms are otherwise alike,
                //   the X form re-sizing the image to the control.
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="SET")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="IMGBUTTON")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    let hwnd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let image = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_SET_IMGBUTTON".to_string(),
                        args: vec![hwnd, id, image],
                        line,
                    }));
                }
                // CONTROL SET IMGBUTTONX hDlg, id&, newimage$   (batch 178)
                //   Official source: control_set_imgbuttonx.htm.  The replacement must be the same format
                //   as the image already displayed; both SET forms are otherwise alike,
                //   the X form re-sizing the image to the control.
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="SET")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="IMGBUTTONX")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    let hwnd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let image = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_SET_IMGBUTTONX".to_string(),
                        args: vec![hwnd, id, image],
                        line,
                    }));
                }
                // CONTROL ADD TEXTBOX, hWnd, id, "text", x, y, w, h [[,] CALL cb] [TO hCtrl&]
                //   (batch 177)  Official source: control_add_textbox.htm.
                //   The TO clause is optional (the official syntax shows only
                //   [[,] CALL callback]); txt$ may be omitted for a LINE.
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="ADD")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="TEXTBOX")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    self.expect(&Token::Comma)?;
                    let hwnd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let text = if matches!(self.peek(), Token::Comma) {
                        Expr::StringLit(String::new())
                    } else {
                        self.parse_expression()?
                    };
                    self.expect(&Token::Comma)?;
                    let x = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let y = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let w = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let h = self.parse_expression()?;
                    let mut style: Option<Expr> = None;
                    let mut exstyle: Option<Expr> = None;
                    let mut callback: Option<Expr> = None;
                    loop {
                        if self.peek() == &Token::Comma {
                            self.advance();
                            if self.peek() == &Token::Call {
                                self.advance();
                                callback = Some(self.parse_expression()?);
                                break;
                            } else if style.is_none() {
                                style = Some(self.parse_expression()?);
                            } else if exstyle.is_none() {
                                exstyle = Some(self.parse_expression()?);
                            } else {
                                break;
                            }
                        } else if self.peek() == &Token::Call && callback.is_none() {
                            self.advance();
                            callback = Some(self.parse_expression()?);
                            break;
                        } else {
                            break;
                        }
                    }
                    let target = if matches!(self.peek(), Token::To) {
                        self.advance();
                        self.parse_expression()?
                    } else {
                        Expr::Variable("_ctl_dummy".to_string())
                    };
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_ADD_TEXTBOX".to_string(),
                        args: vec![
                            hwnd,
                            id,
                            text,
                            x,
                            y,
                            w,
                            h,
                            style.unwrap_or(Expr::IntegerLit(0)),
                            exstyle.unwrap_or(Expr::IntegerLit(0)),
                            callback.unwrap_or(Expr::Variable("_no_cb".to_string())),
                            target,
                        ],
                        line,
                    }));
                }
                // CONTROL ADD LINE, hWnd, id, "text", x, y, w, h [[,] CALL cb] [TO hCtrl&]
                //   (batch 177)  Official source: control_add_line.htm.
                //   The TO clause is optional (the official syntax shows only
                //   [[,] CALL callback]); txt$ may be omitted for a LINE.
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="ADD")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="LINE")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    self.expect(&Token::Comma)?;
                    let hwnd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let text = if matches!(self.peek(), Token::Comma) {
                        Expr::StringLit(String::new())
                    } else {
                        self.parse_expression()?
                    };
                    self.expect(&Token::Comma)?;
                    let x = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let y = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let w = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let h = self.parse_expression()?;
                    let mut style: Option<Expr> = None;
                    let mut exstyle: Option<Expr> = None;
                    let mut callback: Option<Expr> = None;
                    loop {
                        if self.peek() == &Token::Comma {
                            self.advance();
                            if self.peek() == &Token::Call {
                                self.advance();
                                callback = Some(self.parse_expression()?);
                                break;
                            } else if style.is_none() {
                                style = Some(self.parse_expression()?);
                            } else if exstyle.is_none() {
                                exstyle = Some(self.parse_expression()?);
                            } else {
                                break;
                            }
                        } else if self.peek() == &Token::Call && callback.is_none() {
                            self.advance();
                            callback = Some(self.parse_expression()?);
                            break;
                        } else {
                            break;
                        }
                    }
                    let target = if matches!(self.peek(), Token::To) {
                        self.advance();
                        self.parse_expression()?
                    } else {
                        Expr::Variable("_ctl_dummy".to_string())
                    };
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_ADD_LINE".to_string(),
                        args: vec![
                            hwnd,
                            id,
                            text,
                            x,
                            y,
                            w,
                            h,
                            style.unwrap_or(Expr::IntegerLit(0)),
                            exstyle.unwrap_or(Expr::IntegerLit(0)),
                            callback.unwrap_or(Expr::Variable("_no_cb".to_string())),
                            target,
                        ],
                        line,
                    }));
                }
                // CONTROL SET OPTION hDlg, id&, minid&, maxid&   (batch 177)
                //   Official source: control_set_option.htm.  Note there is no
                //   comma between OPTION and hDlg and no TO clause: the
                //   statement sets the check state of id& and clears every
                //   other OPTION whose id falls in minid&..maxid& inclusive.
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="SET")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="OPTION")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    let hwnd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let minid = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let maxid = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_SET_OPTION".to_string(),
                        args: vec![hwnd, id, minid, maxid],
                        line,
                    }));
                }
                // CONTROL ADD CHECKBOX, hWnd, id, "text", x, y, w, h TO hCtrl&
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="ADD")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="CHECKBOX")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    self.expect(&Token::Comma)?;
                    let hwnd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let text = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let x = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let y = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let w = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let h = self.parse_expression()?;
                    // batch 249: official syntax
                    //   [, [style&] [, [exstyle&]]] [[,] CALL callback] [TO hCtrl&]
                    let mut style: Option<Expr> = None;
                    let mut exstyle: Option<Expr> = None;
                    let mut callback: Option<Expr> = None;
                    loop {
                        if self.peek() == &Token::Comma {
                            self.advance();
                            if self.peek() == &Token::Call {
                                self.advance();
                                callback = Some(self.parse_expression()?);
                                break;
                            } else if style.is_none() {
                                style = Some(self.parse_expression()?);
                            } else if exstyle.is_none() {
                                exstyle = Some(self.parse_expression()?);
                            } else {
                                break;
                            }
                        } else if self.peek() == &Token::Call && callback.is_none() {
                            self.advance();
                            callback = Some(self.parse_expression()?);
                            break;
                        } else {
                            break;
                        }
                    }
                    let target = if matches!(self.peek(), Token::To) {
                        self.advance();
                        self.parse_expression()?
                    } else {
                        Expr::Variable("_ctl_dummy".to_string())
                    };
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_ADD_CHECKBOX".to_string(),
                        args: vec![
                            hwnd,
                            id,
                            text,
                            x,
                            y,
                            w,
                            h,
                            style.unwrap_or(Expr::IntegerLit(0)),
                            exstyle.unwrap_or(Expr::IntegerLit(0)),
                            callback.unwrap_or(Expr::Variable("_no_cb".to_string())),
                            target,
                        ],
                        line,
                    }));
                }
                // CONTROL ADD RADIOBUTTON, hWnd, id, "text", x, y, w, h TO hCtrl&
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="ADD")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="RADIOBUTTON")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    self.expect(&Token::Comma)?;
                    let hwnd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let text = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let x = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let y = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let w = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let h = self.parse_expression()?;
                    let mut style: Option<Expr> = None;
                    let mut exstyle: Option<Expr> = None;
                    let mut callback: Option<Expr> = None;
                    loop {
                        if self.peek() == &Token::Comma {
                            self.advance();
                            if self.peek() == &Token::Call {
                                self.advance();
                                callback = Some(self.parse_expression()?);
                                break;
                            } else if style.is_none() {
                                style = Some(self.parse_expression()?);
                            } else if exstyle.is_none() {
                                exstyle = Some(self.parse_expression()?);
                            } else {
                                break;
                            }
                        } else if self.peek() == &Token::Call && callback.is_none() {
                            self.advance();
                            callback = Some(self.parse_expression()?);
                            break;
                        } else {
                            break;
                        }
                    }
                    let target = if self.peek() == &Token::To {
                        self.advance();
                        self.parse_expression()?
                    } else {
                        Expr::IntegerLit(0)
                    };
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_ADD_RADIOBUTTON".to_string(),
                        args: vec![
                            hwnd,
                            id,
                            text,
                            x,
                            y,
                            w,
                            h,
                            style.unwrap_or(Expr::IntegerLit(0)),
                            exstyle.unwrap_or(Expr::IntegerLit(0)),
                            callback.unwrap_or(Expr::Variable("_no_cb".to_string())),
                            target,
                        ],
                        line,
                    }));
                }
                // CONTROL ADD GROUPBOX, hWnd, id, "text", x, y, w, h TO hCtrl&
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="ADD")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="GROUPBOX")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    self.expect(&Token::Comma)?;
                    let hwnd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let text = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let x = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let y = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let w = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let h = self.parse_expression()?;
                    let mut style: Option<Expr> = None;
                    let mut exstyle: Option<Expr> = None;
                    let mut callback: Option<Expr> = None;
                    loop {
                        if self.peek() == &Token::Comma {
                            self.advance();
                            if self.peek() == &Token::Call {
                                self.advance();
                                callback = Some(self.parse_expression()?);
                                break;
                            } else if style.is_none() {
                                style = Some(self.parse_expression()?);
                            } else if exstyle.is_none() {
                                exstyle = Some(self.parse_expression()?);
                            } else {
                                break;
                            }
                        } else if self.peek() == &Token::Call && callback.is_none() {
                            self.advance();
                            callback = Some(self.parse_expression()?);
                            break;
                        } else {
                            break;
                        }
                    }
                    let target = if self.peek() == &Token::To {
                        self.advance();
                        self.parse_expression()?
                    } else {
                        Expr::IntegerLit(0)
                    };
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_ADD_GROUPBOX".to_string(),
                        args: vec![
                            hwnd,
                            id,
                            text,
                            x,
                            y,
                            w,
                            h,
                            style.unwrap_or(Expr::IntegerLit(0)),
                            exstyle.unwrap_or(Expr::IntegerLit(0)),
                            callback.unwrap_or(Expr::Variable("_no_cb".to_string())),
                            target,
                        ],
                        line,
                    }));
                }
                // CONTROL ADD LABEL, hWnd, id, "text", x, y, w, h TO hCtrl&
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="ADD")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="LABEL")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    self.expect(&Token::Comma)?;
                    let hwnd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let text = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let x = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let y = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let w = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let h = self.parse_expression()?;
                    let mut style: Option<Expr> = None;
                    let mut exstyle: Option<Expr> = None;
                    let mut callback: Option<Expr> = None;
                    loop {
                        if self.peek() == &Token::Comma {
                            self.advance();
                            if self.peek() == &Token::Call {
                                self.advance();
                                callback = Some(self.parse_expression()?);
                                break;
                            } else if style.is_none() {
                                style = Some(self.parse_expression()?);
                            } else if exstyle.is_none() {
                                exstyle = Some(self.parse_expression()?);
                            } else {
                                break;
                            }
                        } else if self.peek() == &Token::Call && callback.is_none() {
                            self.advance();
                            callback = Some(self.parse_expression()?);
                            break;
                        } else {
                            break;
                        }
                    }
                    let target = if matches!(self.peek(), Token::To) {
                        self.advance();
                        self.parse_expression()?
                    } else {
                        Expr::Variable("_ctl_dummy".to_string())
                    };
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_ADD_LABEL".to_string(),
                        args: vec![
                            hwnd,
                            id,
                            text,
                            x,
                            y,
                            w,
                            h,
                            style.unwrap_or(Expr::IntegerLit(0)),
                            exstyle.unwrap_or(Expr::IntegerLit(0)),
                            callback.unwrap_or(Expr::Variable("_no_cb".to_string())),
                            target,
                        ],
                        line,
                    }));
                }
                // CONTROL ADD PROGRESSBAR, hWnd, id, x, y, w, h TO hCtrl&
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="ADD")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="PROGRESSBAR")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    self.expect(&Token::Comma)?;
                    let hwnd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;

                    // optional text$ slot (official CONTROL ADD): may be
                    // a string literal, an empty '', ,' slot, or absent
                    // (legacy fork form id, x, y, w, h).
                    if matches!(self.peek(), Token::StringLiteral(_)) {
                        self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                    } else if self.peek() == &Token::Comma {
                        // empty text slot: id, , x
                        self.advance();
                    }
                    let x = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let y = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let w = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let h = self.parse_expression()?;
                    let target = if self.peek() == &Token::To {
                        self.advance();
                        self.parse_expression()?
                    } else {
                        Expr::IntegerLit(0)
                    };
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_ADD_PROGRESSBAR".to_string(),
                        args: vec![hwnd, id, x, y, w, h, target],
                        line,
                    }));
                }
                // CONTROL ADD MONTHCAL, hWnd, id, x, y, w, h TO hCtrl&  (batch 190)
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="ADD")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="MONTHCAL")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    self.expect(&Token::Comma)?;
                    let hwnd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;

                    // optional text$ slot (official CONTROL ADD): may be
                    // a string literal, an empty '', ,' slot, or absent
                    // (legacy fork form id, x, y, w, h).
                    if matches!(self.peek(), Token::StringLiteral(_)) {
                        self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                    } else if self.peek() == &Token::Comma {
                        // empty text slot: id, , x
                        self.advance();
                    }
                    let x = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let y = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let w = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let h = self.parse_expression()?;
                    let target = if self.peek() == &Token::To {
                        self.advance();
                        self.parse_expression()?
                    } else {
                        Expr::IntegerLit(0)
                    };
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_ADD_MONTHCAL".to_string(),
                        args: vec![hwnd, id, x, y, w, h, target],
                        line,
                    }));
                }
                // CONTROL ADD ANIMATE, hWnd, id, x, y, w, h TO hCtrl&  (batch 190)
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="ADD")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="ANIMATE")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    self.expect(&Token::Comma)?;
                    let hwnd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let x = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let y = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let w = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let h = self.parse_expression()?;
                    let target = if self.peek() == &Token::To {
                        self.advance();
                        self.parse_expression()?
                    } else {
                        Expr::IntegerLit(0)
                    };
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_ADD_ANIMATE".to_string(),
                        args: vec![hwnd, id, x, y, w, h, target],
                        line,
                    }));
                }
                // CONTROL ADD RICHEDIT, hWnd, id, x, y, w, h TO hCtrl&  (batch 190)
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="ADD")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="RICHEDIT")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    self.expect(&Token::Comma)?;
                    let hwnd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let x = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let y = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let w = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let h = self.parse_expression()?;
                    let target = if self.peek() == &Token::To {
                        self.advance();
                        self.parse_expression()?
                    } else {
                        Expr::IntegerLit(0)
                    };
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_ADD_RICHEDIT".to_string(),
                        args: vec![hwnd, id, x, y, w, h, target],
                        line,
                    }));
                }
                // CONTROL ADD TRACKBAR, hWnd, id, x, y, w, h TO hCtrl&  (batch 189)
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="ADD")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="TRACKBAR")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    self.expect(&Token::Comma)?;
                    let hwnd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let x = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let y = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let w = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let h = self.parse_expression()?;
                    let target = if self.peek() == &Token::To {
                        self.advance();
                        self.parse_expression()?
                    } else {
                        Expr::IntegerLit(0)
                    };
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_ADD_TRACKBAR".to_string(),
                        args: vec![hwnd, id, x, y, w, h, target],
                        line,
                    }));
                }
                // CONTROL ADD UPDOWN, hWnd, id, x, y, w, h TO hCtrl&  (batch 189)
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="ADD")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="UPDOWN")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    self.expect(&Token::Comma)?;
                    let hwnd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let x = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let y = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let w = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let h = self.parse_expression()?;
                    let target = if self.peek() == &Token::To {
                        self.advance();
                        self.parse_expression()?
                    } else {
                        Expr::IntegerLit(0)
                    };
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_ADD_UPDOWN".to_string(),
                        args: vec![hwnd, id, x, y, w, h, target],
                        line,
                    }));
                }
                // CONTROL ADD HOTKEY, hWnd, id, x, y, w, h TO hCtrl&  (batch 189)
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="ADD")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="HOTKEY")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    self.expect(&Token::Comma)?;
                    let hwnd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let x = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let y = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let w = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let h = self.parse_expression()?;
                    let target = if self.peek() == &Token::To {
                        self.advance();
                        self.parse_expression()?
                    } else {
                        Expr::IntegerLit(0)
                    };
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_ADD_HOTKEY".to_string(),
                        args: vec![hwnd, id, x, y, w, h, target],
                        line,
                    }));
                }
                // CONTROL ADD IPADDRESS, hWnd, id, x, y, w, h TO hCtrl&  (batch 189)
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="ADD")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="IPADDRESS")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    self.expect(&Token::Comma)?;
                    let hwnd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let x = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let y = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let w = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let h = self.parse_expression()?;
                    let target = if self.peek() == &Token::To {
                        self.advance();
                        self.parse_expression()?
                    } else {
                        Expr::IntegerLit(0)
                    };
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_ADD_IPADDRESS".to_string(),
                        args: vec![hwnd, id, x, y, w, h, target],
                        line,
                    }));
                }
                // CONTROL ADD TAB, hWnd, id, x, y, w, h TO hCtrl&  (batch 169)
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="ADD")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="TAB")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    self.expect(&Token::Comma)?;
                    let hwnd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;

                    // optional text$ slot (official CONTROL ADD): may be
                    // a string literal, an empty '', ,' slot, or absent
                    // (legacy fork form id, x, y, w, h).  batch 253: keep the
                    // text and pass it through (CHW has a real text$ slot).
                    let mut text: Option<Expr> = None;
                    if matches!(self.peek(), Token::StringLiteral(_)) {
                        text = Some(self.parse_expression()?);
                        self.expect(&Token::Comma)?;
                    } else if self.peek() == &Token::Comma {
                        // empty text slot: id, , x
                        self.advance();
                        text = Some(Expr::StringLit(String::new()));
                    }
                    let x = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let y = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let w = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let h = self.parse_expression()?;
                    // batch 249: [, [style&] [, [exstyle&]]] [[,] CALL callback] [TO]
                    let mut style: Option<Expr> = None;
                    let mut exstyle: Option<Expr> = None;
                    let mut callback: Option<Expr> = None;
                    loop {
                        if self.peek() == &Token::Comma {
                            self.advance();
                            if self.peek() == &Token::Call {
                                self.advance();
                                callback = Some(self.parse_expression()?);
                                break;
                            } else if style.is_none() {
                                style = Some(self.parse_expression()?);
                            } else if exstyle.is_none() {
                                exstyle = Some(self.parse_expression()?);
                            } else {
                                break;
                            }
                        } else if self.peek() == &Token::Call && callback.is_none() {
                            self.advance();
                            callback = Some(self.parse_expression()?);
                            break;
                        } else {
                            break;
                        }
                    }
                    let target = if self.peek() == &Token::To {
                        self.advance();
                        self.parse_expression()?
                    } else {
                        Expr::IntegerLit(0)
                    };
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_ADD_TAB".to_string(),
                        args: vec![
                            hwnd,
                            id,
                            text.unwrap_or(Expr::StringLit(String::new())),
                            x,
                            y,
                            w,
                            h,
                            style.unwrap_or(Expr::IntegerLit(0)),
                            exstyle.unwrap_or(Expr::IntegerLit(0)),
                            callback.unwrap_or(Expr::Variable("_no_cb".to_string())),
                            target,
                        ],
                        line,
                    }));
                }
                // CONTROL ADD LISTVIEW, hWnd, id, x, y, w, h TO hCtrl&  (batch 158)
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="ADD")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="LISTVIEW")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    self.expect(&Token::Comma)?;
                    let hwnd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;

                    // optional text$ slot (official CONTROL ADD): may be
                    // a string literal, an empty '', ,' slot, or absent
                    // (legacy fork form id, x, y, w, h).
                    if matches!(self.peek(), Token::StringLiteral(_)) {
                        self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                    } else if self.peek() == &Token::Comma {
                        // empty text slot: id, , x
                        self.advance();
                    }
                    let x = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let y = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let w = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let h = self.parse_expression()?;
                    let target = if self.peek() == &Token::To {
                        self.advance();
                        self.parse_expression()?
                    } else {
                        Expr::IntegerLit(0)
                    };
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_ADD_LISTVIEW".to_string(),
                        args: vec![hwnd, id, x, y, w, h, target],
                        line,
                    }));
                }
                // CONTROL ADD TREEVIEW, hWnd, id, x, y, w, h TO hCtrl&  (batch 158)
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="ADD")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="TREEVIEW")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    self.expect(&Token::Comma)?;
                    let hwnd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;

                    // optional text$ slot (official CONTROL ADD): may be
                    // a string literal, an empty '', ,' slot, or absent
                    // (legacy fork form id, x, y, w, h).
                    if matches!(self.peek(), Token::StringLiteral(_)) {
                        self.parse_expression()?;
                        self.expect(&Token::Comma)?;
                    } else if self.peek() == &Token::Comma {
                        // empty text slot: id, , x
                        self.advance();
                    }
                    let x = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let y = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let w = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let h = self.parse_expression()?;
                    // Optional [, [style&] [, [exstyle&]]] per the official
                    // CONTROL ADD TREEVIEW syntax. An explicit primary style
                    // REPLACES the documented default, which is how a caller
                    // reaches %TVS_CHECKBOXES for TREEVIEW SET/GET CHECK.
                    // target keeps slot 6 either way, so the codegen arm reads
                    // the same positions whether or not a style was supplied.
                    let mut style = Expr::IntegerLit(0);
                    let mut exstyle = Expr::IntegerLit(0);
                    if self.peek() == &Token::Comma {
                        self.advance();
                        style = self.parse_expression()?;
                        if self.peek() == &Token::Comma {
                            self.advance();
                            exstyle = self.parse_expression()?;
                        }
                    }
                    let target = if self.peek() == &Token::To {
                        self.advance();
                        self.parse_expression()?
                    } else {
                        Expr::IntegerLit(0)
                    };
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_ADD_TREEVIEW".to_string(),
                        args: vec![hwnd, id, x, y, w, h, target, style, exstyle],
                        line,
                    }));
                }
                // CONTROL ADD TOOLBAR, hDlg, ID, Txt$, x, y, nWide, nHigh
                //   [,style&] [,exstyle&] [,CALL callback]           (batch 164)
                // CONTROL ADD STATUSBAR, hDlg, id&, txt$, x, y, xx, yy
                //   [, [style&] [, [exstyle&]]] [[,] CALL callback]  (batch 164)
                // The x/y/width/height operands are parsed but ignored, exactly as
                // the official docs state: both controls dock themselves inside the
                // parent according to their style bits (%CCS_TOP / %CCS_BOTTOM /
                // %SBARS_SIZEGRIP), not according to those values.  A trailing CALL
                // clause registers a callback on the created control.  style and
                // exstyle always occupy their own slots so a trailing CALL clause
                // can never be mistaken for one of them.
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="ADD")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="TOOLBAR")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    self.expect(&Token::Comma)?;
                    let hwnd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let text = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let x = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let y = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let w = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let h = self.parse_expression()?;
                    // Optional operands in the order the help pages print them:
                    //   [, [style&] [, [exstyle&]]] [[,] CALL callback] [TO hCtrl&]
                    // The TO clause used to be read before style and exstyle, so a
                    // program that passed a style AND a TO target handed the style
                    // to the TO slot.  That slot is not an lvalue, so no store
                    // happened and the statement quietly did nothing (batch 179).
                    let mut target: Option<Expr> = None;
                    let mut extras: Vec<Expr> = Vec::new();
                    loop {
                        if self.peek() == &Token::To {
                            self.advance();
                            let t = self.parse_expression()?;
                            if target.is_none() {
                                target = Some(t);
                            }
                            continue;
                        }
                        if self.peek() != &Token::Comma
                            || matches!(self.peek_at(1), Some(Token::Call))
                            || extras.len() >= 2
                        {
                            break;
                        }
                        self.advance();
                        extras.push(self.parse_expression()?);
                    }
                    let mut args = vec![
                        hwnd,
                        id,
                        text,
                        x,
                        y,
                        w,
                        h,
                        target.unwrap_or(Expr::IntegerLit(0)),
                    ];
                    let mut extra = extras.len();
                    args.extend(extras);
                    if self.peek() == &Token::Comma {
                        self.advance();
                    }
                    while extra < 2 {
                        args.push(Expr::IntegerLit(0));
                        extra += 1;
                    }
                    if self.peek() == &Token::Call {
                        self.advance();
                        args.push(self.parse_expression()?);
                        if self.peek() == &Token::To {
                            self.advance();
                            let t = self.parse_expression()?;
                            args[7] = t;
                        }
                    }
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_ADD_TOOLBAR".to_string(),
                        args,
                        line,
                    }));
                }
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="ADD")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="STATUSBAR")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    self.expect(&Token::Comma)?;
                    let hwnd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let text = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let x = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let y = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let w = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let h = self.parse_expression()?;
                    // Optional operands in the order the help pages print them:
                    //   [, [style&] [, [exstyle&]]] [[,] CALL callback] [TO hCtrl&]
                    // The TO clause used to be read before style and exstyle, so a
                    // program that passed a style AND a TO target handed the style
                    // to the TO slot.  That slot is not an lvalue, so no store
                    // happened and the statement quietly did nothing (batch 179).
                    let mut target: Option<Expr> = None;
                    let mut extras: Vec<Expr> = Vec::new();
                    loop {
                        if self.peek() == &Token::To {
                            self.advance();
                            let t = self.parse_expression()?;
                            if target.is_none() {
                                target = Some(t);
                            }
                            continue;
                        }
                        if self.peek() != &Token::Comma
                            || matches!(self.peek_at(1), Some(Token::Call))
                            || extras.len() >= 2
                        {
                            break;
                        }
                        self.advance();
                        extras.push(self.parse_expression()?);
                    }
                    let mut args = vec![
                        hwnd,
                        id,
                        text,
                        x,
                        y,
                        w,
                        h,
                        target.unwrap_or(Expr::IntegerLit(0)),
                    ];
                    let mut extra = extras.len();
                    args.extend(extras);
                    if self.peek() == &Token::Comma {
                        self.advance();
                    }
                    while extra < 2 {
                        args.push(Expr::IntegerLit(0));
                        extra += 1;
                    }
                    if self.peek() == &Token::Call {
                        self.advance();
                        args.push(self.parse_expression()?);
                        if self.peek() == &Token::To {
                            self.advance();
                            let t = self.parse_expression()?;
                            args[7] = t;
                        }
                    }
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_ADD_STATUSBAR".to_string(),
                        args,
                        line,
                    }));
                }
                // CONTROL ADD GRAPHIC, hDlg, id&, txt$, x, y, xx, yy
                //   [, [style&] [, [exstyle&]]] [[,] CALL callback] [TO hCtrl&]
                //   (batch 179)  Official source: control_add_graphic.htm.  The eight-operand
                //   shape is the one the toolbar/statusbar branches above already
                //   use, so their operand handling is reused here verbatim: eight
                //   operands, style and exstyle in their own slots, an optional
                //   CALL clause, then TO hCtrl&.
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="ADD")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="GRAPHIC")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    self.expect(&Token::Comma)?;
                    let hwnd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let text = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let x = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let y = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let w = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let h = self.parse_expression()?;
                    // Optional operands in the order the help pages print them:
                    //   [, [style&] [, [exstyle&]]] [[,] CALL callback] [TO hCtrl&]
                    // The TO clause used to be read before style and exstyle, so a
                    // program that passed a style AND a TO target handed the style
                    // to the TO slot.  That slot is not an lvalue, so no store
                    // happened and the statement quietly did nothing (batch 179).
                    let mut target: Option<Expr> = None;
                    let mut extras: Vec<Expr> = Vec::new();
                    loop {
                        if self.peek() == &Token::To {
                            self.advance();
                            let t = self.parse_expression()?;
                            if target.is_none() {
                                target = Some(t);
                            }
                            continue;
                        }
                        if self.peek() != &Token::Comma
                            || matches!(self.peek_at(1), Some(Token::Call))
                            || extras.len() >= 2
                        {
                            break;
                        }
                        self.advance();
                        extras.push(self.parse_expression()?);
                    }
                    let mut args = vec![
                        hwnd,
                        id,
                        text,
                        x,
                        y,
                        w,
                        h,
                        target.unwrap_or(Expr::IntegerLit(0)),
                    ];
                    let mut extra = extras.len();
                    args.extend(extras);
                    if self.peek() == &Token::Comma {
                        self.advance();
                    }
                    while extra < 2 {
                        args.push(Expr::IntegerLit(0));
                        extra += 1;
                    }
                    if self.peek() == &Token::Call {
                        self.advance();
                        args.push(self.parse_expression()?);
                        if self.peek() == &Token::To {
                            self.advance();
                            let t = self.parse_expression()?;
                            args[7] = t;
                        }
                    }
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_ADD_GRAPHIC".to_string(),
                        args,
                        line,
                    }));
                }
                // CONTROL ADD HEADER, hDlg, id&, txt$, x, y, xx, yy
                //   [, [style&] [, [exstyle&]]] [[,] CALL callback] [TO hCtrl&]
                //   (batch 179)  Official source: CONTROL_ADD_HEADER_statement.htm.  The eight-operand
                //   shape is the one the toolbar/statusbar branches above already
                //   use, so their operand handling is reused here verbatim: eight
                //   operands, style and exstyle in their own slots, an optional
                //   CALL clause, then TO hCtrl&.
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="ADD")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="HEADER")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    self.expect(&Token::Comma)?;
                    let hwnd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let text = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let x = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let y = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let w = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let h = self.parse_expression()?;
                    // Optional operands in the order the help pages print them:
                    //   [, [style&] [, [exstyle&]]] [[,] CALL callback] [TO hCtrl&]
                    // The TO clause used to be read before style and exstyle, so a
                    // program that passed a style AND a TO target handed the style
                    // to the TO slot.  That slot is not an lvalue, so no store
                    // happened and the statement quietly did nothing (batch 179).
                    let mut target: Option<Expr> = None;
                    let mut extras: Vec<Expr> = Vec::new();
                    loop {
                        if self.peek() == &Token::To {
                            self.advance();
                            let t = self.parse_expression()?;
                            if target.is_none() {
                                target = Some(t);
                            }
                            continue;
                        }
                        if self.peek() != &Token::Comma
                            || matches!(self.peek_at(1), Some(Token::Call))
                            || extras.len() >= 2
                        {
                            break;
                        }
                        self.advance();
                        extras.push(self.parse_expression()?);
                    }
                    let mut args = vec![
                        hwnd,
                        id,
                        text,
                        x,
                        y,
                        w,
                        h,
                        target.unwrap_or(Expr::IntegerLit(0)),
                    ];
                    let mut extra = extras.len();
                    args.extend(extras);
                    if self.peek() == &Token::Comma {
                        self.advance();
                    }
                    while extra < 2 {
                        args.push(Expr::IntegerLit(0));
                        extra += 1;
                    }
                    if self.peek() == &Token::Call {
                        self.advance();
                        args.push(self.parse_expression()?);
                        if self.peek() == &Token::To {
                            self.advance();
                            let t = self.parse_expression()?;
                            args[7] = t;
                        }
                    }
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_ADD_HEADER".to_string(),
                        args,
                        line,
                    }));
                }
                // CONTROL ADD classname$, hDlg, id&, txt$, x, y, xx, yy
                //   [, [style&] [, [exstyle&]]] [[,] CALL callback] [TO hCtrl&]
                //   (batch 166)  The generic custom-control form documented on
                // control_add.htm.  The class name is a string expression
                // ("MSCTLS_TRACKBAR_CLASS32", or a classname$ variable), not one
                // of the fixed type words handled above, so this branch has to be
                // tested AFTER every typed CONTROL ADD branch.  DDT gives a custom
                // control no default style (Custom_Control_Style_Note.htm), so the
                // style operands are passed through exactly as written.
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="ADD")
                {
                    self.advance(); // CONTROL
                    self.advance(); // ADD
                    let cls = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let hwnd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let id = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let text = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let x = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let y = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let w = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let h = self.parse_expression()?;
                    // Optional [, style&] [, exstyle&] - a trailing CALL clause
                    // is left for consume_to_eol, exactly as the BUTTON branch does.
                    let mut style = Expr::IntegerLit(0);
                    let mut exstyle = Expr::IntegerLit(0);
                    let mut n_opt = 0;
                    while self.peek() == &Token::Comma && n_opt < 2 {
                        let is_call = matches!(self.peek_at(1),
                            Some(Token::Identifier(w)) if w.to_uppercase() == "CALL");
                        if is_call {
                            break;
                        }
                        self.advance(); // comma
                        let e = self.parse_expression()?;
                        if n_opt == 0 {
                            style = e;
                        } else {
                            exstyle = e;
                        }
                        n_opt += 1;
                    }
                    let target = if matches!(self.peek(), Token::To) {
                        self.advance();
                        self.parse_expression()?
                    } else {
                        Expr::Variable("_ctl_dummy".to_string())
                    };
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_ADD_CUSTOM".to_string(),
                        args: vec![cls, hwnd, id, text, x, y, w, h, style, exstyle, target],
                        line,
                    }));
                }
                // DIALOG NEW [PIXELS, | UNITS,] hParent, "title", x, y, w, h [, style& [, exstyle&]] [,] TO hDlg
                // batch 242: official syntax (PBWin.chw) allows an optional
                // PIXELS / UNITS prefix before hParent.  Units default to
                // UNITS (dialog units); PIXELS makes all size/position
                // parameters plain pixels.  The flag travels as arg[9].
                if name_upper == "DIALOG"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="NEW")
                {
                    self.advance();
                    self.advance();
                    let mut units = Expr::IntegerLit(0); // 0 = UNITS (default), 1 = PIXELS
                    if let Token::Identifier(kw) = self.peek() {
                        let up = kw.to_uppercase();
                        if up == "PIXELS" || up == "UNITS" {
                            if up == "PIXELS" {
                                units = Expr::IntegerLit(1);
                            }
                            self.advance();
                            self.expect(&Token::Comma)?;
                        }
                    }
                    let parent = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let title = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    // x: empty if comma immediately follows (,,)
                    let x = if matches!(self.peek(), Token::Comma) {
                        Expr::IntegerLit(0)
                    } else {
                        self.parse_expression()?
                    };
                    self.expect(&Token::Comma)?;
                    let y = if matches!(self.peek(), Token::Comma) {
                        Expr::IntegerLit(0)
                    } else {
                        self.parse_expression()?
                    };
                    self.expect(&Token::Comma)?;
                    let w = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let h = self.parse_expression()?;
                    // Optional style, exstyle: after h, either comma (style) or To
                    let style = if matches!(self.peek(), Token::Comma) {
                        self.advance();
                        self.parse_expression()?
                    } else {
                        Expr::IntegerLit(0)
                    };
                    let exstyle = if matches!(self.peek(), Token::Comma) {
                        self.advance();
                        self.parse_expression()?
                    } else {
                        Expr::IntegerLit(0)
                    };
                    let target = if self.peek() == &Token::To {
                        self.advance();
                        self.parse_expression()?
                    } else {
                        Expr::IntegerLit(0)
                    };
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "DIALOG_NEW".to_string(),
                        args: vec![parent, title, x, y, w, h, style, exstyle, target, units],
                        line,
                    }));
                }
                // DIALOG SHOW MODAL hDlg CALL proc
                //
                // The guard must test the noun as well as the verb.  It
                // used to test only SHOW, so it also captured
                // DIALOG SHOW STATE and DIALOG SHOW MODELESS: it skipped
                // no MODAL keyword (the noun is not MODAL), parsed the
                // noun as the handle and the real handle as the callback,
                // and codegen then reported
                // "DIALOG SHOW MODAL: unknown sub '<handle>'".  Both
                // sibling statements were silently dead - DIALOG SHOW
                // STATE had never worked.  Batch 165 found this while
                // adding DIALOG SHOW MODELESS.
                if name_upper == "DIALOG"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="SHOW")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="MODAL")
                {
                    self.advance();
                    self.advance();
                    self.advance(); // MODAL (now guaranteed by the guard)
                    let hd = self.parse_expression()?;
                    // DIALOG SHOW MODAL hDlg CALL proc   (fork callback form)
                    if self.peek() == &Token::Call {
                        self.advance();
                        let proc = self.parse_expression()?;
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "DIALOG_SHOW_MODAL".to_string(),
                            args: vec![hd, proc],
                            line,
                        }));
                    }
                    // DIALOG SHOW MODAL hDlg TO result   (official PB form;
                    // batch 238: HelloDDT from C:\PBWin10\samples\DDT failed
                    // with "Unexpected token in expression: To")
                    if self.peek() == &Token::To {
                        self.advance();
                        let target = self.parse_expression()?;
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "DIALOG_SHOW_MODAL_TO".to_string(),
                            args: vec![hd, target],
                            line,
                        }));
                    }
                    // DIALOG SHOW MODAL hDlg   (no callback, no result)
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "DIALOG_SHOW_MODAL".to_string(),
                        args: vec![hd, Expr::IntegerLit(0)],
                        line,
                    }));
                }
                // DIALOG END hDlg, result   (result is optional in official
                // PBWin syntax: DIALOG END hDlg& [, lResult&])
                if name_upper == "DIALOG" && self.peek_at(1) == Some(&Token::End) {
                    self.advance();
                    self.advance();
                    let hd = self.parse_expression()?;
                    if self.peek() == &Token::Comma {
                        self.advance();
                        let result = self.parse_expression()?;
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "DIALOG_END".to_string(),
                            args: vec![hd, result],
                            line,
                        }));
                    }
                    // DIALOG END hDlg   (no result)
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "DIALOG_END".to_string(),
                        args: vec![hd, Expr::IntegerLit(0)],
                        line,
                    }));
                }
                // DIALOG SET SIZE hDlg, nWide&, nHigh&
                // (this no longer has to precede DIALOG SET TEXT - that
                //  guard tests its noun as well since batch 165; see the
                //  note there)
                if name_upper == "DIALOG"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="SET")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="SIZE")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    let hd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let nw = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let nh = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "DIALOG_SET_SIZE".to_string(),
                        args: vec![hd, nw, nh],
                        line,
                    }));
                }
                // DIALOG SET TEXT hDlg, "title"
                //
                // The guard must test the noun as well as the verb.  It
                // used to test only the verb, which meant every other
                // "DIALOG SET ..." statement - SET SIZE, SET CLIENT,
                // SET LOC, SET USER, SET COLOR, SET ICON - was captured
                // here, compiled into a window-title change, and had its
                // remaining arguments eaten by consume_to_eol.  The
                // statement compiled, exited 0, and silently did the
                // wrong thing.  Batch 165 found this while adding the
                // rest of the SET family; DIALOG SET SIZE had been
                // dodging it by sitting earlier in the chain.
                if name_upper == "DIALOG"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="SET")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="TEXT")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    let hd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let txt = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "DIALOG_SET_TEXT".to_string(),
                        args: vec![hd, txt],
                        line,
                    }));
                }
                // DIALOG DOEVENTS - process pending messages non-blocking
                if name_upper == "DIALOG"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="DOEVENTS")
                {
                    self.advance();
                    self.advance();
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "DIALOG_DOEVENTS".to_string(),
                        args: vec![],
                        line,
                    }));
                }
                // DIALOG CENTER hDlg
                if name_upper == "DIALOG"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="CENTER")
                {
                    self.advance();
                    self.advance();
                    let hd = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "DIALOG_CENTER".to_string(),
                        args: vec![hd],
                        line,
                    }));
                }
                // DIALOG GET TEXT hDlg TO var$
                if name_upper == "DIALOG"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="GET")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="TEXT")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    let hd = self.parse_expression()?;
                    let target = if self.peek() == &Token::To {
                        self.advance();
                        self.parse_expression()?
                    } else {
                        Expr::IntegerLit(0)
                    };
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "DIALOG_GET_TEXT".to_string(),
                        args: vec![hd, target],
                        line,
                    }));
                }
                // DIALOG GET SIZE hDlg TO x&, y&
                if name_upper == "DIALOG"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="GET")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="SIZE")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    let hd = self.parse_expression()?;
                    self.expect(&Token::To)?;
                    let xv = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let yv = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "DIALOG_GET_SIZE".to_string(),
                        args: vec![hd, xv, yv],
                        line,
                    }));
                }
                // DIALOG SHOW STATE hDlg, nCmdShow (1=normal, 2=min, 3=max)
                if name_upper == "DIALOG"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="SHOW")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="STATE")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    let hd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let state = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "DIALOG_SHOW_STATE".to_string(),
                        args: vec![hd, state],
                        line,
                    }));
                }
                // CONTROL KILL hCtrl  (Tier-3 DDT GUI)
                // CHW official by-id: CONTROL KILL hDlg, id& (batch 251)
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="KILL")
                {
                    self.advance();
                    self.advance();
                    let first = self.parse_expression()?;
                    if matches!(self.peek(), Token::Comma) {
                        self.advance();
                        let id = self.parse_expression()?;
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "CONTROL_KILL_ID".to_string(),
                            args: vec![first, id],
                            line,
                        }));
                    }
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_KILL".to_string(),
                        args: vec![first],
                        line,
                    }));
                }
                // CONTROL SET CHECK hCtrl, state  (Tier-3 DDT GUI)
                // CHW official by-id: CONTROL SET CHECK hDlg, id&, checkstate& (batch 251)
                if name_upper == "CONTROL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="SET")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="CHECK")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    let first = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let second = self.parse_expression()?;
                    if matches!(self.peek(), Token::Comma) {
                        self.advance();
                        let st = self.parse_expression()?;
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "CONTROL_SET_CHECK_ID".to_string(),
                            args: vec![first, second, st],
                            line,
                        }));
                    }
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "CONTROL_SET_CHECK".to_string(),
                        args: vec![first, second],
                        line,
                    }));
                } // DIALOG MENU hDlg, hMenu
                if name_upper == "DIALOG"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="MENU")
                {
                    self.advance();
                    self.advance();
                    let hd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let hm = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "DIALOG_MENU".to_string(),
                        args: vec![hd, hm],
                        line,
                    }));
                }

                // --------------------------------------------------------
                // Batch 165 - the DIALOG statement family.  Every keyword
                // below is a plain Identifier token (verified against
                // token.rs), and each guard keys off the word after DIALOG,
                // so the order of these blocks does not matter.
                // --------------------------------------------------------
                // DIALOG ENABLE hDlg / DIALOG DISABLE hDlg
                if name_upper == "DIALOG"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w))
                        if matches!(w.to_uppercase().as_str(), "ENABLE" | "DISABLE"))
                {
                    let on = matches!(self.peek_at(1), Some(Token::Identifier(w))
                        if w.to_uppercase() == "ENABLE");
                    self.advance();
                    self.advance();
                    let hd = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "DIALOG_ENABLE".to_string(),
                        args: vec![hd, Expr::IntegerLit(if on { 1 } else { 0 })],
                        line,
                    }));
                }
                // DIALOG HIDE / NORMALIZE / MINIMIZE / MAXIMIZE hDlg
                if name_upper == "DIALOG"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w))
                        if matches!(w.to_uppercase().as_str(),
                            "HIDE" | "NORMALIZE" | "MINIMIZE" | "MAXIMIZE"))
                {
                    let cmd = match self.peek_at(1) {
                        Some(Token::Identifier(w)) => match w.to_uppercase().as_str() {
                            "HIDE" => 0,     // %SW_HIDE
                            "MINIMIZE" => 6, // %SW_MINIMIZE
                            "MAXIMIZE" => 3, // %SW_MAXIMIZE
                            _ => 9,          // %SW_RESTORE - DIALOG NORMALIZE
                        },
                        _ => 9,
                    };
                    self.advance();
                    self.advance();
                    let hd = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "DIALOG_SHOW".to_string(),
                        args: vec![hd, Expr::IntegerLit(cmd)],
                        line,
                    }));
                }
                // DIALOG SHOW MODELESS hDlg [[,] CALL callback] [TO lResult&]
                if name_upper == "DIALOG"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="SHOW")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="MODELESS")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    let hd = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "DIALOG_SHOW".to_string(),
                        // %SW_SHOW - display and activate; the message pump is
                        // the program's own (DIALOG DOEVENTS / PBMAIN loop).
                        args: vec![hd, Expr::IntegerLit(5)],
                        line,
                    }));
                }
                // DIALOG STABILIZE hDlg / DIALOG NONSTABLE hDlg
                if name_upper == "DIALOG"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w))
                        if matches!(w.to_uppercase().as_str(), "STABILIZE" | "NONSTABLE"))
                {
                    let stable = matches!(self.peek_at(1), Some(Token::Identifier(w))
                        if w.to_uppercase() == "STABILIZE");
                    self.advance();
                    self.advance();
                    let hd = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "DIALOG_STABILIZE".to_string(),
                        args: vec![hd, Expr::IntegerLit(if stable { 1 } else { 0 })],
                        line,
                    }));
                }
                // DIALOG REDRAW hDlg
                if name_upper == "DIALOG"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w))
                        if w.to_uppercase() == "REDRAW")
                {
                    self.advance();
                    self.advance();
                    let hd = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "DIALOG_REDRAW".to_string(),
                        args: vec![hd],
                        line,
                    }));
                }
                // DIALOG SEND hDlg, msg&, wParam&, lParam& [TO lResult&]
                if name_upper == "DIALOG"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w))
                        if w.to_uppercase() == "SEND")
                {
                    self.advance();
                    self.advance();
                    let hd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let msg = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let wp = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let lp = self.parse_expression()?;
                    let mut args = vec![hd, msg, wp, lp];
                    if self.peek() == &Token::To {
                        self.advance();
                        args.push(self.parse_expression()?);
                    }
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "DIALOG_SEND".to_string(),
                        args,
                        line,
                    }));
                }
                // DIALOG POST hDlg, msg&, wParam&, lParam&
                if name_upper == "DIALOG"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w))
                        if w.to_uppercase() == "POST")
                {
                    self.advance();
                    self.advance();
                    let hd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let msg = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let wp = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let lp = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "DIALOG_POST".to_string(),
                        args: vec![hd, msg, wp, lp],
                        line,
                    }));
                }
                // DIALOG SET USER hDlg, index&, usrval&
                if name_upper == "DIALOG"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="SET")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="USER")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    let hd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let idx = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let val = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "DIALOG_SET_USER".to_string(),
                        args: vec![hd, idx, val],
                        line,
                    }));
                }
                // DIALOG GET USER hDlg, index& TO retvar&
                if name_upper == "DIALOG"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="GET")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="USER")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    let hd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let idx = self.parse_expression()?;
                    self.expect(&Token::To)?;
                    let dst = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "DIALOG_GET_USER".to_string(),
                        args: vec![hd, idx, dst],
                        line,
                    }));
                }
                // DIALOG SET COLOR hDlg, foreclr&, backclr&
                if name_upper == "DIALOG"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="SET")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="COLOR")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    let hd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let fore = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let back = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "DIALOG_SET_COLOR".to_string(),
                        args: vec![hd, fore, back],
                        line,
                    }));
                }
                // DIALOG DEFAULT FONT fontname$ [, points& [, style& [, charset&]]]
                if name_upper == "DIALOG"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="DEFAULT")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="FONT")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    let mut args = vec![self.parse_expression()?];
                    while self.peek() == &Token::Comma && args.len() < 4 {
                        self.advance();
                        args.push(self.parse_expression()?);
                    }
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "DIALOG_DEFAULT_FONT".to_string(),
                        args,
                        line,
                    }));
                }
                // DIALOG FONT fontname$ [, points& [, style& [, charset&]]]
                //   (legacy spelling without DEFAULT; dialog_font.htm)
                if name_upper == "DIALOG"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="FONT")
                {
                    self.advance();
                    self.advance();
                    let mut args = vec![self.parse_expression()?];
                    while self.peek() == &Token::Comma && args.len() < 4 {
                        self.advance();
                        args.push(self.parse_expression()?);
                    }
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "DIALOG_DEFAULT_FONT".to_string(),
                        args,
                        line,
                    }));
                }
                // DIALOG SET ICON hDlg, newicon$
                if name_upper == "DIALOG"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="SET")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="ICON")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    let hd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let nm = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "DIALOG_SET_ICON".to_string(),
                        args: vec![hd, nm],
                        line,
                    }));
                }
                // DIALOG GET CLIENT hDlg TO nWide&, nHigh&
                if name_upper == "DIALOG"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="GET")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="CLIENT")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    let hd = self.parse_expression()?;
                    self.expect(&Token::To)?;
                    let wv = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let hv = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "DIALOG_GET_CLIENT".to_string(),
                        args: vec![hd, wv, hv],
                        line,
                    }));
                }
                // DIALOG SET CLIENT hDlg, x&, y&   (client-area size)
                if name_upper == "DIALOG"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="SET")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="CLIENT")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    let hd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let wv = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let hv = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "DIALOG_SET_CLIENT".to_string(),
                        args: vec![hd, wv, hv],
                        line,
                    }));
                }
                // DIALOG GET LOC hDlg TO x&, y&
                if name_upper == "DIALOG"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="GET")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="LOC")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    let hd = self.parse_expression()?;
                    self.expect(&Token::To)?;
                    let xv = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let yv = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "DIALOG_GET_LOC".to_string(),
                        args: vec![hd, xv, yv],
                        line,
                    }));
                }
                // DIALOG SET LOC hDlg, x&, y&
                if name_upper == "DIALOG"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase()=="SET")
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase()=="LOC")
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    let hd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let xv = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let yv = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "DIALOG_SET_LOC".to_string(),
                        args: vec![hd, xv, yv],
                        line,
                    }));
                }
                // DIALOG PIXELS hDlg, x&, y& TO UNITS xx&, yy&
                if name_upper == "DIALOG"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w))
                        if w.to_uppercase() == "PIXELS")
                {
                    self.advance();
                    self.advance();
                    let hd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let xv = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let yv = self.parse_expression()?;
                    self.expect(&Token::To)?;
                    if matches!(self.peek(), Token::Identifier(w) if w.to_uppercase() == "UNITS") {
                        self.advance();
                    }
                    let xo = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let yo = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "DIALOG_PIXELS".to_string(),
                        args: vec![hd, xv, yv, xo, yo],
                        line,
                    }));
                }
                // DIALOG UNITS hDlg, x&, y& TO PIXELS xx&, yy&
                if name_upper == "DIALOG"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w))
                        if w.to_uppercase() == "UNITS")
                {
                    self.advance();
                    self.advance();
                    let hd = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let xv = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let yv = self.parse_expression()?;
                    self.expect(&Token::To)?;
                    if matches!(self.peek(), Token::Identifier(w) if w.to_uppercase() == "PIXELS") {
                        self.advance();
                    }
                    let xo = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let yo = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "DIALOG_UNITS".to_string(),
                        args: vec![hd, xv, yv, xo, yo],
                        line,
                    }));
                }
                // ARRAY COPY src(), dest() / ARRAY SWAP a(), b()
                if name_upper == "ARRAY"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w))
                        if matches!(w.to_uppercase().as_str(), "COPY" | "SWAP" | "ADD"))
                {
                    let op = {
                        if let Token::Identifier(w) = self.peek_at(1).unwrap() {
                            w.to_uppercase()
                        } else {
                            unreachable!()
                        }
                    };
                    self.advance(); // consume ARRAY
                    self.advance(); // consume COPY/SWAP
                    let src = self.parse_primary()?;
                    self.expect(&Token::Comma)?;
                    let dst = self.parse_primary()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: format!("ARRAY {}", op),
                        args: vec![src, dst],
                        line,
                    }));
                }
                // ARRAY ASSIGN target() = expr1 [, expr2, ...]  (CHW spread form:
                // assign successive values to successive array elements).  The
                // single-source form target() = source() (fork extension, used
                // by batch023) is parsed the same way; codegen distinguishes a
                // lone array reference from scalar values.
                if name_upper == "ARRAY"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase() == "ASSIGN")
                {
                    self.advance(); // consume ARRAY
                    self.advance(); // consume ASSIGN
                    let target = self.parse_primary()?;
                    self.expect(&Token::Eq)?;
                    let mut values = vec![self.parse_expression()?];
                    while self.peek() == &Token::Comma {
                        self.advance(); // consume comma
                        values.push(self.parse_expression()?);
                    }
                    self.consume_to_eol();
                    let mut args = Vec::with_capacity(values.len() + 1);
                    args.push(target);
                    args.extend(values);
                    return Ok(Statement::Call(CallStmt {
                        name: "ARRAY ASSIGN".to_string(),
                        args,
                        line,
                    }));
                }
                // ARRAY UNIQUE arr() [FOR count]
                if name_upper == "ARRAY"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase() == "UNIQUE")
                {
                    self.advance(); // consume ARRAY
                    self.advance(); // consume UNIQUE
                    let arr = self.parse_primary()?;
                    let mut args = vec![arr];
                    if self.peek() == &Token::For {
                        self.advance(); // consume FOR
                        args.push(self.parse_expression()?);
                    }
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "ARRAY UNIQUE".to_string(),
                        args,
                        line,
                    }));
                }
                // ARRAY SCAN arr([index]) [FOR count] [, FROM s TO e]
                //   [, COLLATE {UCASE|cstr}] [,] op expr [,] TO target
                // op: = <> < > <= >=.  Args:
                //   [arr, count(0=rest), op_code, value, target, from(0), to(0),
                //    collate_mode(0=none 1=UCASE 2=custom string), collate_expr]
                if name_upper == "ARRAY"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase() == "SCAN")
                {
                    self.advance(); // consume ARRAY
                    self.advance(); // consume SCAN
                    let arr_expr = self.parse_primary()?;
                    let mut count = Expr::IntegerLit(0);
                    if self.peek() == &Token::For {
                        self.advance(); // consume FOR
                        count = self.parse_expression()?;
                    }
                    let mut from = Expr::IntegerLit(0);
                    let mut to = Expr::IntegerLit(0);
                    let mut collate_mode = 0i64;
                    let mut collate_expr = Expr::IntegerLit(0);
                    // optional leading commas before FROM / COLLATE options
                    loop {
                        if self.peek() == &Token::Comma {
                            self.advance();
                        }
                        if let Token::Identifier(w) = self.peek() {
                            let wu = w.to_uppercase();
                            if wu == "FROM" {
                                self.advance();
                                from = self.parse_expression()?;
                                if self.peek() == &Token::To {
                                    self.advance();
                                    to = self.parse_expression()?;
                                }
                                continue;
                            } else if wu == "COLLATE" {
                                self.advance();
                                if let Token::Identifier(w2) = self.peek() {
                                    if w2.to_uppercase() == "UCASE" {
                                        self.advance();
                                        collate_mode = 1;
                                    } else {
                                        collate_mode = 2;
                                        collate_expr = self.parse_expression()?;
                                    }
                                } else {
                                    collate_mode = 2;
                                    collate_expr = self.parse_expression()?;
                                }
                                continue;
                            }
                        }
                        break;
                    }
                    if self.peek() == &Token::Comma {
                        self.advance(); // optional comma before op
                    }
                    let op_code = match self.peek() {
                        Token::Eq => 0,
                        Token::Neq => 1,
                        Token::Lt => 2,
                        Token::Gt => 3,
                        Token::Lte => 4,
                        Token::Gte => 5,
                        _ => {
                            return Err(PbError::parser(
                                format!(
                                    "ARRAY SCAN: expected relational operator, got {:?}",
                                    self.peek()
                                ),
                                self.current_file(),
                                self.current_line(),
                            ));
                        }
                    };
                    let op_sym = match self.peek() {
                        Token::Eq => "=",
                        Token::Neq => "<>",
                        Token::Lt => "<",
                        Token::Gt => ">",
                        Token::Lte => "<=",
                        Token::Gte => ">=",
                        _ => "?",
                    };
                    self.advance(); // consume operator
                    let value = self.parse_expression()?;
                    if self.peek() == &Token::Comma {
                        self.advance(); // optional comma before TO
                    }
                    if self.peek() == &Token::To {
                        self.advance(); // consume TO
                    }
                    let target = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: format!("ARRAY SCAN {}", op_sym),
                        args: vec![
                            arr_expr,
                            count,
                            Expr::IntegerLit(op_code),
                            value,
                            target,
                            from,
                            to,
                            Expr::IntegerLit(collate_mode),
                            collate_expr,
                        ],
                        line,
                    }));
                }
                // ARRAY INSERT arr([index]) [FOR count] [,] value
                if name_upper == "ARRAY"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase() == "INSERT")
                {
                    self.advance(); // consume ARRAY
                    self.advance(); // consume INSERT
                    let arr_expr = self.parse_primary()?;
                    let mut args = vec![arr_expr];
                    if self.peek() == &Token::For {
                        self.advance(); // consume FOR
                        args.push(self.parse_expression()?);
                    } else {
                        args.push(Expr::IntegerLit(0));
                    }
                    if self.peek() == &Token::Comma {
                        self.advance();
                    }
                    args.push(self.parse_expression()?);
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "ARRAY INSERT".to_string(),
                        args,
                        line,
                    }));
                }
                // ARRAY DELETE arr([index]) [FOR count] [, expression]
                if name_upper == "ARRAY"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase() == "DELETE")
                {
                    self.advance(); // consume ARRAY
                    self.advance(); // consume DELETE
                    let arr_expr = self.parse_primary()?;
                    let mut args = vec![arr_expr];
                    if self.peek() == &Token::For {
                        self.advance(); // consume FOR
                        args.push(self.parse_expression()?);
                    } else {
                        args.push(Expr::IntegerLit(1));
                    }
                    if self.peek() == &Token::Comma {
                        self.advance();
                        args.push(self.parse_expression()?);
                    }
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "ARRAY DELETE".to_string(),
                        args,
                        line,
                    }));
                }
                // ARRAY ARRAYIX arr()
                if name_upper == "ARRAY"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase() == "ARRAYIX")
                {
                    self.advance(); // consume ARRAY
                    self.advance(); // consume ARRAYIX
                    let args = vec![self.parse_primary()?];
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "ARRAY ARRAYIX".to_string(),
                        args,
                        line,
                    }));
                }
                // FILESCAN [#] fnum&, RECORDS TO y& [, WIDTH TO x&]
                if name_upper == "FILESCAN" {
                    self.advance(); // consume FILESCAN
                    let mut args = Vec::new();
                    if self.peek() == &Token::Hash {
                        self.advance(); // consume #
                    }
                    args.push(self.parse_expression()?); // file number
                    self.expect(&Token::Comma)?;
                    if let Token::Identifier(w) = self.peek() {
                        if w.to_uppercase() == "RECORDS" {
                            self.advance(); // consume RECORDS
                            self.expect(&Token::To)?;
                            args.push(self.parse_expression()?); // records target
                        }
                    }
                    if self.peek() == &Token::Comma {
                        self.advance(); // consume comma
                        if let Token::Identifier(w) = self.peek() {
                            if w.to_uppercase() == "WIDTH" {
                                self.advance(); // consume WIDTH
                                self.expect(&Token::To)?;
                                args.push(self.parse_expression()?); // width target
                            }
                        }
                    }
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "FILESCAN".to_string(),
                        args,
                        line,
                    }));
                }
                // BIT {SET|RESET|TOGGLE} intvar, bitnumber / BIT CALC intvar, bitnumber, expr
                if name_upper == "BIT"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w))
                        if matches!(w.to_uppercase().as_str(), "SET" | "RESET" | "TOGGLE" | "CALC"))
                {
                    self.advance(); // consume BIT
                    let op = self.consume_identifier()?.to_uppercase(); // SET/RESET/TOGGLE/CALC
                    let mut args = vec![self.parse_expression()?];
                    self.expect(&Token::Comma)?;
                    args.push(self.parse_expression()?);
                    if op == "CALC" {
                        self.expect(&Token::Comma)?;
                        args.push(self.parse_expression()?);
                    }
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: format!("BIT {}", op),
                        args,
                        line,
                    }));
                }
                // PROCESS GET PRIORITY TO var / PROCESS SET PRIORITY pri
                if name_upper == "PROCESS"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w))
                        if matches!(w.to_uppercase().as_str(), "GET" | "SET"))
                    && matches!(self.peek_at(2), Some(Token::Identifier(w)) if w.to_uppercase() == "PRIORITY")
                {
                    self.advance(); // consume PROCESS
                    let op = self.consume_identifier()?.to_uppercase(); // GET / SET
                    self.advance(); // consume PRIORITY
                    let mut args = Vec::new();
                    if op == "GET" {
                        if self.peek() == &Token::To {
                            self.advance(); // consume TO
                        }
                        args.push(self.parse_expression()?);
                    } else {
                        args.push(self.parse_expression()?);
                    }
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: format!("PROCESS {} PRIORITY", op),
                        args,
                        line,
                    }));
                }
                // SPLIT [WORD] MainStr, Part1Len TO Part1Var, Part2Var
                if name_upper == "SPLIT" {
                    self.advance(); // consume SPLIT
                    if matches!(self.peek(), Token::Identifier(w) if w.to_uppercase() == "WORD") {
                        self.advance(); // consume optional WORD (ignored)
                    }
                    let mut args = vec![self.parse_expression()?];
                    self.expect(&Token::Comma)?;
                    args.push(self.parse_expression()?);
                    if self.peek() == &Token::To {
                        self.advance(); // consume TO
                    }
                    args.push(self.parse_expression()?);
                    self.expect(&Token::Comma)?;
                    args.push(self.parse_expression()?);
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "SPLIT".to_string(),
                        args,
                        line,
                    }));
                }
                // SHIFT [SIGNED] {LEFT|RIGHT} ivar, count / ROTATE {LEFT|RIGHT} ivar, count
                if (name_upper == "SHIFT" || name_upper == "ROTATE")
                    && matches!(self.peek_at(1), Some(Token::Identifier(w))
                        if matches!(w.to_uppercase().as_str(), "LEFT" | "RIGHT" | "SIGNED"))
                {
                    let mut parts = vec![name_upper.clone()];
                    self.advance(); // consume SHIFT/ROTATE
                    if name_upper == "SHIFT"
                        && matches!(self.peek(), Token::Identifier(w) if w.to_uppercase() == "SIGNED")
                    {
                        parts.push("SIGNED".to_string());
                        self.advance();
                    }
                    if let Token::Identifier(w) = self.peek() {
                        let d = w.to_uppercase();
                        if d == "LEFT" || d == "RIGHT" {
                            parts.push(d);
                            self.advance();
                        }
                    }
                    let stmt_name = parts.join(" ");
                    let mut args = vec![self.parse_expression()?];
                    while self.peek() == &Token::Comma {
                        self.advance();
                        args.push(self.parse_expression()?);
                    }
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: stmt_name,
                        args,
                        line,
                    }));
                }
                // ARRAY SHUFFLE arr() [FOR n]
                if name_upper == "ARRAY"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase() == "SHUFFLE")
                {
                    self.advance(); // consume ARRAY
                    self.advance(); // consume SHUFFLE
                    let mut args = vec![self.parse_expression()?];
                    while self.peek() == &Token::Comma {
                        self.advance();
                        args.push(self.parse_expression()?);
                    }
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "ARRAY SHUFFLE".to_string(),
                        args,
                        line,
                    }));
                }
                // ARRAY REVERSE arr() [FOR n]
                if name_upper == "ARRAY"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase() == "REVERSE")
                {
                    self.advance(); // consume ARRAY
                    self.advance(); // consume REVERSE
                    let mut args = vec![self.parse_expression()?];
                    while self.peek() == &Token::Comma {
                        self.advance();
                        args.push(self.parse_expression()?);
                    }
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "ARRAY REVERSE".to_string(),
                        args,
                        line,
                    }));
                }
                // ARRAY SORT arr([index]) [FOR count]
                //   [,TAGARRAY t()] [,{ASCEND|DESCEND}] [,FROM s TO e]
                //   [,COLLATE {UCASE|cstr}] [,CALL custfunc() | USING custfunc()]
                // Args: [arr, count, tag, descend, from, to, collate_mode,
                //        collate_expr, call_expr]
                if name_upper == "ARRAY"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase() == "SORT")
                {
                    self.advance(); // consume ARRAY
                    self.advance(); // consume SORT
                    let arr_expr = self.parse_expression()?;
                    let mut count = Expr::IntegerLit(0);
                    if self.peek() == &Token::For {
                        self.advance(); // consume FOR
                        count = self.parse_expression()?;
                    }
                    let mut tag = Expr::IntegerLit(0);
                    let mut descend = 0i64;
                    let mut from = Expr::IntegerLit(0);
                    let mut to = Expr::IntegerLit(0);
                    let mut collate_mode = 0i64;
                    let mut collate_expr = Expr::IntegerLit(0);
                    let mut call_expr = Expr::IntegerLit(0);
                    loop {
                        if self.peek() == &Token::Comma {
                            self.advance();
                        }
                        if let Token::Identifier(w) = self.peek() {
                            let wu = w.to_uppercase();
                            if wu == "TAGARRAY" {
                                self.advance();
                                tag = self.parse_expression()?;
                                continue;
                            } else if wu == "ASCEND" || wu == "DESCEND" {
                                descend = if wu == "DESCEND" { 1 } else { 0 };
                                self.advance();
                                continue;
                            } else if wu == "FROM" {
                                self.advance();
                                from = self.parse_expression()?;
                                if self.peek() == &Token::To {
                                    self.advance();
                                    to = self.parse_expression()?;
                                }
                                continue;
                            } else if wu == "COLLATE" {
                                self.advance();
                                if let Token::Identifier(w2) = self.peek() {
                                    if w2.to_uppercase() == "UCASE" {
                                        self.advance();
                                        collate_mode = 1;
                                    } else {
                                        collate_mode = 2;
                                        collate_expr = self.parse_expression()?;
                                    }
                                } else {
                                    collate_mode = 2;
                                    collate_expr = self.parse_expression()?;
                                }
                                continue;
                            } else if wu == "CALL" || wu == "USING" {
                                self.advance();
                                call_expr = self.parse_expression()?;
                                continue;
                            }
                        }
                        break;
                    }
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "ARRAY SORT".to_string(),
                        args: vec![
                            arr_expr,
                            count,
                            tag,
                            Expr::IntegerLit(descend),
                            from,
                            to,
                            Expr::IntegerLit(collate_mode),
                            collate_expr,
                            call_expr,
                        ],
                        line,
                    }));
                }
                // ARRAY SELECT arr(), start, end
                if name_upper == "ARRAY"
                    && (matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase() == "SELECT")
                        || matches!(self.peek_at(1), Some(&Token::Select)))
                {
                    self.advance();
                    self.advance();
                    let arr = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    // op form: = <> < > <= >= expr TO var
                    let op_code = match self.peek() {
                        Token::Eq => Some(0),
                        Token::Neq => Some(1),
                        Token::Lt => Some(2),
                        Token::Gt => Some(3),
                        Token::Lte => Some(4),
                        Token::Gte => Some(5),
                        _ => None,
                    };
                    if let Some(op) = op_code {
                        self.advance(); // consume op
                        let val = self.parse_expression()?;
                        // The operator form is written both as `> 25 TO i` and `> 25, TO i`;
                        // accept the comma when it is present.
                        if self.peek() == &Token::Comma {
                            self.advance();
                        }
                        self.expect(&Token::To)?;
                        let dst = self.parse_expression()?;
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "ARRAY_SELECT_OP".to_string(),
                            args: vec![arr, Expr::IntegerLit(op), val, dst],
                            line,
                        }));
                    }
                    // range form: start, end
                    let start = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let end = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "ARRAY_SELECT".to_string(),
                        args: vec![arr, start, end],
                        line,
                    }));
                }
                // ARRAY TAGARRAY arr(), tagarr() / ARRAY TAGARRAY ERASE arr()
                if name_upper == "ARRAY"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase() == "TAGARRAY")
                {
                    self.advance();
                    self.advance();
                    if self.peek_plain_upper() == "ERASE" {
                        self.advance();
                        let arr = self.parse_expression()?;
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "ARRAY_TAGARRAY_ERASE".to_string(),
                            args: vec![arr],
                            line,
                        }));
                    }
                    let arr = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let tag = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "ARRAY_TAGARRAY".to_string(),
                        args: vec![arr, tag],
                        line,
                    }));
                }

                // ACCEL ATTACH hDlg, id() — accelerator table (accepted; no-op until GUI)
                if name_upper == "ACCEL"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase() == "ATTACH")
                {
                    self.advance();
                    self.advance();
                    let mut args = Vec::new();
                    if self.peek() != &Token::Eol {
                        args.push(self.parse_expression()?);
                        while self.peek() == &Token::Comma {
                            self.advance();
                            args.push(self.parse_expression()?);
                        }
                    }
                    // Optional `TO hAccel`.  Recorded as a marker literal plus the
                    // target, the same convention IMAGELIST NEW uses, so codegen
                    // never has to guess whether the clause was written.
                    // `TO` is the keyword token Token::To here, which
                    // peek_plain_upper() does not map - that is why the first
                    // version of this clause was silently swallowed by
                    // consume_to_eol() and the handle never reached the caller.
                    if self.peek() == &Token::To
                        || matches!(self.peek(), Token::Identifier(id) if id.eq_ignore_ascii_case("TO"))
                    {
                        self.advance();
                        let target = self.parse_expression()?;
                        args.push(Expr::IntegerLit(1));
                        args.push(target);
                    }
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "ACCEL_ATTACH".to_string(),
                        args,
                        line,
                    }));
                }
                // TCP NOTIFY ... / UDP NOTIFY ... — async notify window (accepted; no-op)
                if (name_upper == "TCP" || name_upper == "UDP")
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase() == "NOTIFY")
                {
                    let kw = name_upper.clone();
                    self.advance();
                    self.advance();
                    let mut args = Vec::new();
                    if self.peek() != &Token::Eol {
                        args.push(self.parse_expression()?);
                        while self.peek() == &Token::Comma {
                            self.advance();
                            args.push(self.parse_expression()?);
                        }
                    }
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: format!("{}_NOTIFY", kw),
                        args,
                        line,
                    }));
                }
                // EVENT SOURCE id — only valid inside a CLASS/END CLASS block
                if name_upper == "EVENT"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase() == "SOURCE")
                {
                    self.error_count += 1;
                    eprintln!(
                        "Error: Parse error at line {}: EVENT SOURCE is only valid inside a CLASS/END CLASS block",
                        line
                    );
                    self.advance();
                    self.advance();
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "EVENT_SOURCE".to_string(),
                        args: vec![],
                        line,
                    }));
                }
                // EVENTS FROM obj / EVENTS END obj — subscribe / unsubscribe
                if name_upper == "EVENTS" {
                    let ev_kind = match self.peek_at(1) {
                        Some(Token::Identifier(k)) if k.to_uppercase() == "FROM" => "EVENTS_FROM",
                        Some(Token::End) => "EVENTS_END",
                        _ => {
                            self.error_count += 1;
                            eprintln!(
                                "Error: Parse error at line {}: EVENTS requires FROM <obj> or END <obj>",
                                line
                            );
                            self.consume_to_eol();
                            return Ok(Statement::Call(CallStmt {
                                name: "EVENTS".to_string(),
                                args: vec![],
                                line,
                            }));
                        }
                    };
                    self.advance(); // EVENTS
                    self.advance(); // FROM / END
                    let obj = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: ev_kind.to_string(),
                        args: vec![obj],
                        line,
                    }));
                }
                // RAISEEVENT Iface . Method ( args ) — fire event handlers
                if name_upper == "RAISEEVENT" {
                    self.advance(); // RAISEEVENT
                    let iface = self.consume_identifier()?;
                    if !matches!(self.peek(), Token::Dot) {
                        self.error_count += 1;
                        eprintln!(
                            "Error: Parse error at line {}: RAISEEVENT requires Iface.Method form",
                            line
                        );
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: "RAISEEVENT".to_string(),
                            args: vec![],
                            line,
                        }));
                    }
                    self.advance(); // .
                    let method = self.consume_identifier()?;
                    let mut args = vec![Expr::StringLit(iface), Expr::StringLit(method)];
                    if self.peek() == &Token::LParen {
                        self.advance();
                        loop {
                            if self.peek() == &Token::RParen {
                                break;
                            }
                            args.push(self.parse_expression()?);
                            if self.peek() == &Token::Comma {
                                self.advance();
                                continue;
                            }
                            break;
                        }
                        self.expect(&Token::RParen)?;
                    }
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "RAISEEVENT".to_string(),
                        args,
                        line,
                    }));
                }
                // INSTANCE — only valid at the start of a CLASS/END CLASS block
                if name_upper == "INSTANCE" {
                    self.error_count += 1;
                    eprintln!(
                        "Error: Parse error at line {}: INSTANCE is only valid inside a CLASS/END CLASS block",
                        line
                    );
                    self.advance();
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "INSTANCE".to_string(),
                        args: vec![],
                        line,
                    }));
                }
                // LET obj2 = expr (plain LET keyword) — consume LET, then normal assignment
                if name_upper == "LET" && self.peek_at(1) != Some(&Token::Star) {
                    self.advance(); // consume LET
                    let primary = self.parse_primary()?;
                    if self.peek() == &Token::Eq {
                        self.advance();
                        let value = self.parse_expression()?;
                        self.consume_to_eol();
                        return Ok(Statement::Assign(AssignStmt {
                            target: primary,
                            value,
                            line,
                        }));
                    }
                    self.consume_to_eol();
                    eprintln!("Error: LET: unsupported form on line {}", line);
                    self.error_count += 1;
                    return Ok(Statement::Noop("LET".to_string(), line));
                }
                // LET *ptr = obj / LET *ptr = variant — object/variant pointer assign (accepted; no-op)
                if name_upper == "LET" && self.peek_at(1) == Some(&Token::Star) {
                    self.advance(); // consume LET
                    self.advance(); // consume *
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "LET_PTR".to_string(),
                        args: vec![],
                        line,
                    }));
                }
                // DEF fnName(params) = expr — single-line function, stored for inline expansion
                if name_upper == "DEF" {
                    self.advance(); // consume DEF
                    let fname = self.consume_identifier()?;
                    self.expect(&Token::LParen)?;
                    let mut params: Vec<String> = Vec::new();
                    if self.peek() != &Token::RParen {
                        loop {
                            params.push(self.consume_identifier()?);
                            if self.peek() != &Token::Comma {
                                break;
                            }
                            self.advance();
                        }
                    }
                    self.expect(&Token::RParen)?;
                    self.expect(&Token::Eq)?;
                    let body = self.parse_expression()?;
                    self.def_funcs.insert(fname.to_uppercase(), (params, body));
                    return Ok(Statement::Call(CallStmt {
                        name: "DEF_FN".to_string(),
                        args: vec![],
                        line,
                    }));
                }

                // ARRAY REDIM INCR arr(), n / ARRAY REDIM DECR arr(), n
                if name_upper == "ARRAY"
                    && (matches!(self.peek_at(1), Some(Token::Identifier(w)) if w.to_uppercase() == "REDIM")
                        || matches!(self.peek_at(1), Some(&Token::Redim)))
                {
                    self.advance(); // ARRAY
                    self.advance(); // REDIM
                    let is_incr = if matches!(*self.peek(), Token::Incr) {
                        self.advance();
                        true
                    } else if matches!(*self.peek(), Token::Decr) {
                        self.advance();
                        false
                    } else if let Token::Identifier(w) = self.peek() {
                        let up = w.to_uppercase();
                        self.advance();
                        up == "INCR"
                    } else {
                        false
                    };
                    let arr = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let n = self.parse_expression()?;
                    self.consume_to_eol();
                    let call_name = if is_incr {
                        "ARRAY_REDIM_INCR"
                    } else {
                        "ARRAY_REDIM_DECR"
                    };
                    return Ok(Statement::Call(CallStmt {
                        name: call_name.to_string(),
                        args: vec![arr, n],
                        line,
                    }));
                }
                // GET #filenum [, pos], var / PUT #filenum [, pos], var
                // (binary file read/write; args = [filenum, (pos), var])
                // PUT$ [#] filenum&, StrgExpr — write ANSI string at file position
                if name_upper == "PUT$" {
                    self.advance(); // consume PUT$
                    if self.peek() == &Token::Hash {
                        self.advance(); // consume optional #
                    }
                    let mut args = vec![self.parse_expression()?];
                    while self.peek() == &Token::Comma {
                        self.advance();
                        args.push(self.parse_expression()?);
                    }
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "PUT_STR".to_string(),
                        args,
                        line,
                    }));
                }
                // PUT$$ [#] filenum&, StrgExpr — write WIDE (UTF-16LE) string at file position
                if name_upper == "PUT$$" {
                    self.advance(); // consume PUT$$
                    if self.peek() == &Token::Hash {
                        self.advance(); // consume optional #
                    }
                    let mut args = vec![self.parse_expression()?];
                    while self.peek() == &Token::Comma {
                        self.advance();
                        args.push(self.parse_expression()?);
                    }
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "PUT_WSTR".to_string(),
                        args,
                        line,
                    }));
                }
                // GET$ [#] filenum&, Count&, StrgVar — read Count bytes into a string var
                if name_upper == "GET$" {
                    self.advance(); // consume GET$
                    if self.peek() == &Token::Hash {
                        self.advance(); // consume optional #
                    }
                    let mut args = vec![self.parse_expression()?];
                    while self.peek() == &Token::Comma {
                        self.advance();
                        args.push(self.parse_expression()?);
                    }
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "GET_STR".to_string(),
                        args,
                        line,
                    }));
                }
                // GET$$ [#] filenum&, Count&, StrgVar — read Count WIDE chars into a string var
                if name_upper == "GET$$" {
                    self.advance(); // consume GET$$
                    if self.peek() == &Token::Hash {
                        self.advance(); // consume optional #
                    }
                    let mut args = vec![self.parse_expression()?];
                    while self.peek() == &Token::Comma {
                        self.advance();
                        args.push(self.parse_expression()?);
                    }
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "GET_WSTR".to_string(),
                        args,
                        line,
                    }));
                }
                if (name_upper == "GET" || name_upper == "PUT")
                    && self.peek_at(1) == Some(&Token::Hash)
                {
                    self.advance(); // consume GET/PUT
                    self.advance(); // consume #
                    let mut args = vec![self.parse_expression()?];
                    while self.peek() == &Token::Comma {
                        self.advance();
                        args.push(self.parse_expression()?);
                    }
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: name_upper,
                        args,
                        line,
                    }));
                }
                // SEEK #filenum, position&
                if name_upper == "SEEK" && self.peek_at(1) == Some(&Token::Hash) {
                    self.advance(); // consume SEEK
                    self.advance(); // consume #
                    let file_num = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let pos = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "SEEK".to_string(),
                        args: vec![file_num, pos],
                        line,
                    }));
                }
                // SETEOF #filenum — truncate at current position
                if name_upper == "SETEOF" && self.peek_at(1) == Some(&Token::Hash) {
                    self.advance(); // consume SETEOF
                    self.advance(); // consume #
                    let mut args = vec![self.parse_expression()?];
                    while self.peek() == &Token::Comma {
                        self.advance();
                        args.push(self.parse_expression()?);
                    }
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "SETEOF".to_string(),
                        args,
                        line,
                    }));
                }
                if name_upper == "CLIPBOARD" {
                    return self.parse_clipboard_statement(line);
                }
                /* batch 263: METHOD = expr inside a METHOD body sets the
                method's return value (CHW METHOD_END_METHOD_statements.htm
                official form "METHOD = expression").  Previously this fell
                into the type-declaration path and died with "DefType, Type
                id, or AS... required" (fork 516). */
                if name_upper == "METHOD" && self.peek_at(1) == Some(&Token::Eq) {
                    self.advance(); // METHOD
                    self.advance(); // =
                    let value = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::FunctionReturn(FunctionReturnStmt {
                        value,
                        line,
                    }));
                }
                if name_upper == "GLOBALMEM" {
                    return self.parse_globalmem_statement(line);
                }
                if name_upper == "MOUSEPTR" {
                    return self.parse_mouseptr_statement(line);
                }
                if name_upper == "UCODEPAGE" {
                    return self.parse_ucodepage_statement(line);
                }
                // TCP / UDP socket statements (batch 19)
                if name_upper == "TCP" || name_upper == "UDP" {
                    return self.parse_net_statement(line);
                }
                // HOST ADDR [hostname$] TO ip&  /  HOST NAME [ip&] TO hostname$
                if name_upper == "HOST"
                    && matches!(self.peek_at(1), Some(Token::Identifier(w))
                    if matches!(w.to_uppercase().as_str(), "ADDR" | "NAME"))
                {
                    let op = {
                        if let Token::Identifier(w) = self.peek_at(1).unwrap() {
                            w.to_uppercase()
                        } else {
                            unreachable!()
                        }
                    };
                    self.advance(); // consume HOST
                    self.advance(); // consume ADDR/NAME
                    let mut args = Vec::new();
                    if self.peek() != &Token::To {
                        args.push(self.parse_expression()?);
                    }
                    self.expect(&Token::To)?;
                    args.push(self.parse_expression()?);
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: format!("HOST {}", op),
                        args,
                        line,
                    }));
                }
                // LOCK / UNLOCK #filenum [, record& [, length&]]
                if (name_upper == "LOCK" || name_upper == "UNLOCK")
                    && self.peek_at(1) == Some(&Token::Hash)
                {
                    let stmt_name = name_upper.clone();
                    self.advance(); // consume LOCK/UNLOCK
                    self.advance(); // consume #
                    let file_num = self.parse_expression()?;
                    let mut args = vec![file_num];
                    while self.peek() == &Token::Comma {
                        self.advance();
                        args.push(self.parse_expression()?);
                    }
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: stmt_name,
                        args,
                        line,
                    }));
                }
                // FLUSH #filenum
                if name_upper == "FLUSH" && self.peek_at(1) == Some(&Token::Hash) {
                    self.advance(); // consume FLUSH
                    self.advance(); // consume #
                    let file_num = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "FLUSH".to_string(),
                        args: vec![file_num],
                        line,
                    }));
                }
                // NAME oldfile$ AS newfile$
                if name_upper == "NAME" {
                    self.advance(); // consume NAME
                    let old_name = self.parse_expression()?;
                    if matches!(self.peek(), Token::As) {
                        self.advance();
                    }
                    let new_name = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "NAME".to_string(),
                        args: vec![old_name, new_name],
                        line,
                    }));
                }
                // COMM serial port statements
                if name_upper == "COMM" {
                    return self.parse_comm_statement(line);
                }
                // THREAD statements
                if name_upper == "THREAD" {
                    return self.parse_thread_statement(line);
                }
                // LPRINT statements (direct line-printer output)
                if name_upper == "LPRINT" {
                    return self.parse_lprint_statement(line);
                }
                // TRACE statements (explicit trace file logging)
                if name_upper == "TRACE" {
                    return self.parse_trace_statement(line);
                }
                // IMPORT statements (explicit DLL loading)
                if name_upper == "IMPORT" {
                    return self.parse_import_statement(line);
                }
                // TOOLBAR / STATUSBAR statements, official PB syntax (batch 164).
                // Same reason as PROGRESSBAR / HEADER just below: these must be
                // handled before the bare-control-name fallback, which would
                // otherwise swallow the whole line as a Noop and make codegen
                // report the statement as unimplemented.  Supported forms:
                //   TOOLBAR ADD BUTTON hDlg, ID, image&, cmd&, style&, text$ [AT item&]
                //   TOOLBAR ADD SEPARATOR hDlg, ID, size& [,cmd&] [AT item&]
                //   TOOLBAR DELETE BUTTON hDlg, id&, [BYCMD] item&
                //   TOOLBAR GET STATE hDlg, ID, [BYCMD] item& TO datav&
                //   TOOLBAR GET COUNT hDlg, ID TO datav&
                //   TOOLBAR SET IMAGELIST hDlg, ID, hLst, ListType&
                //   TOOLBAR SET STATE hDlg, ID, [BYCMD] item&, state&
                //   STATUSBAR SET PARTS hDlg, id&, x& [,x&...]
                //   STATUSBAR SET TEXT hDlg, id&, item&, style&, text$
                if name_upper == "TOOLBAR" || name_upper == "STATUSBAR" {
                    let head = name_upper.clone();
                    let verb: Option<String> = match self.peek_at(1) {
                        Some(Token::Identifier(w)) => {
                            let u = w.to_uppercase();
                            if u == "ADD" || u == "DELETE" || u == "GET" || u == "SET" {
                                Some(u)
                            } else {
                                None
                            }
                        }
                        _ => None,
                    };
                    if let Some(verb) = verb {
                        self.advance(); // TOOLBAR / STATUSBAR
                        self.advance(); // verb
                                        // Optional noun: BUTTON / SEPARATOR / STATE / COUNT /
                                        // IMAGELIST / PARTS / TEXT
                        let mut noun = String::new();
                        if let Some(Token::Identifier(w2)) = self.peek_at(0) {
                            let t = w2.to_uppercase();
                            if t == "BUTTON"
                                || t == "SEPARATOR"
                                || t == "STATE"
                                || t == "COUNT"
                                || t == "IMAGELIST"
                                || t == "PARTS"
                                || t == "TEXT"
                            {
                                noun = t;
                                self.advance();
                            }
                        }
                        // Every form starts with  hDlg, id
                        let mut args = Vec::new();
                        args.push(self.parse_expression()?); // hDlg
                        self.expect(&Token::Comma)?;
                        args.push(self.parse_expression()?); // id
                                                             // Optional BYCMD between id& and item& (DELETE BUTTON /
                                                             // GET STATE / SET STATE).  BYCMD is an identifier, not a
                                                             // reserved token.
                        let mut bycmd = false;
                        if self.peek() == &Token::Comma {
                            if let Some(Token::Identifier(w3)) = self.peek_at(1) {
                                if w3.to_uppercase() == "BYCMD" {
                                    bycmd = true;
                                    self.advance(); // comma
                                    self.advance(); // BYCMD
                                                    // The item operand follows BYCMD directly -
                                                    // the comma before it was already consumed,
                                                    // so it has to be parsed here.
                                    args.push(self.parse_expression()?);
                                }
                            }
                        }
                        // Remaining comma-separated operands
                        while self.peek() == &Token::Comma {
                            self.advance();
                            args.push(self.parse_expression()?);
                        }
                        let n_before_at = args.len();
                        // [AT item&] - AT is an identifier, not a reserved token
                        let mut saw_at = false;
                        if let Some(Token::Identifier(w4)) = self.peek_at(0) {
                            if w4.to_uppercase() == "AT" {
                                saw_at = true;
                                self.advance();
                                args.push(self.parse_expression()?);
                            }
                        }
                        // [CALL callback] - CALL is a reserved token (token.rs:157)
                        let mut saw_call = false;
                        if self.peek() == &Token::Call {
                            saw_call = true;
                            self.advance();
                            args.push(self.parse_expression()?);
                        }
                        // Normalise the optional operands into a fixed layout so
                        // codegen never has to guess which optionals are present:
                        //   TOOLBAR ADD BUTTON    -> [hDlg, id, image, cmd, style,
                        //                             text, at, cb]
                        //   TOOLBAR ADD SEPARATOR -> [hDlg, id, size, cmd, at]
                        if head == "TOOLBAR" && verb == "ADD" && noun == "BUTTON" {
                            if saw_call && !saw_at {
                                args.push(Expr::IntegerLit(0)); // AT slot
                            }
                            while args.len() < 8 {
                                args.push(Expr::IntegerLit(0));
                            }
                        } else if head == "TOOLBAR" && verb == "ADD" && noun == "SEPARATOR" {
                            if saw_at && n_before_at == 3 {
                                args.push(Expr::IntegerLit(0)); // cmd slot
                            }
                            while args.len() < 5 {
                                args.push(Expr::IntegerLit(0));
                            }
                        }
                        // TO datav&  (GET STATE / GET COUNT)
                        if self.peek() == &Token::To {
                            self.advance();
                            args.push(self.parse_expression()?);
                        }
                        // Trailing BYCMD flag for codegen (0 or 1)
                        args.push(Expr::IntegerLit(if bycmd { 1 } else { 0 }));
                        self.consume_to_eol();
                        let stmt_name = if noun.is_empty() {
                            format!("{}_{}", head, verb)
                        } else {
                            format!("{}_{}_{}", head, verb, noun)
                        };
                        return Ok(Statement::Call(CallStmt {
                            name: stmt_name,
                            args,
                            line,
                        }));
                    }
                }
                // PROGRESSBAR / HEADER statements, official PB syntax.  These
                // must be handled before the bare-control-name fallback below,
                // which would otherwise swallow the whole line as a Noop and
                // make codegen report the statement as unimplemented.
                if name_upper == "PROGRESSBAR" || name_upper == "HEADER" {
                    let head = name_upper.clone();
                    // STEP is a keyword token, not an identifier (token.rs:214).
                    let verb: Option<String> = if matches!(self.peek_at(1), Some(Token::Step)) {
                        Some("STEP".to_string())
                    } else if let Some(Token::Identifier(w)) = self.peek_at(1) {
                        let u = w.to_uppercase();
                        if u == "GET" || u == "SET" || u == "SEND" || u == "STEP" {
                            Some(u)
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    if let Some(verb) = verb {
                        self.advance(); // PROGRESSBAR / HEADER
                        self.advance(); // verb
                        let want_noun = if head == "PROGRESSBAR" {
                            verb != "STEP"
                        } else {
                            verb == "GET" || verb == "SET"
                        };
                        let mut noun = String::new();
                        if want_noun {
                            if matches!(self.peek(), Token::Step) {
                                noun = "STEP".to_string();
                                self.advance();
                            } else if let Token::Identifier(w2) = self.peek() {
                                let t = w2.to_uppercase();
                                if t == "POS" || t == "RANGE" || t == "COUNT" || t == "ITEM" {
                                    noun = t;
                                    self.advance();
                                }
                            }
                        }
                        let mut args = Vec::new();
                        args.push(self.parse_expression()?);
                        self.expect(&Token::Comma)?;
                        args.push(self.parse_expression()?);
                        // A trailing TO target may follow any number of
                        // comma-separated operands, and a TO may itself be
                        // followed by further comma-separated outputs
                        // (PROGRESSBAR GET RANGE ... TO lo&, hi&), so both
                        // separators are consumed in one loop.  Previously TO
                        // was only tested *before* the comma loop, which
                        // silently dropped the target of
                        //   HEADER SEND hWin, ID&, Msg&, wParam&, lParam& TO r&
                        //   HEADER GET ITEM hWin, ID&, Index&, ItemPtr TO r&
                        // and made those statements store nothing at all.
                        loop {
                            // Both separators are accepted at every step: `TO` introduces the
                            // result variable(s) and a comma can follow it for a two-target
                            // form such as `HEADER GET ITEM ... TO lo, hi`.
                            if self.peek() == &Token::Comma || self.peek() == &Token::To {
                                self.advance();
                                args.push(self.parse_expression()?);
                            } else {
                                break;
                            }
                        }
                        // PROGRESSBAR STEP hDlg, id& [, incramt&] - the increment
                        // is optional, so default it to 0 for codegen.
                        if head == "PROGRESSBAR" && verb == "STEP" && args.len() == 2 {
                            args.push(Expr::IntegerLit(0));
                        }
                        self.consume_to_eol();
                        let stmt_name = if noun.is_empty() {
                            format!("{}_{}", head, verb)
                        } else {
                            format!("{}_{}_{}", head, verb, noun)
                        };
                        return Ok(Statement::Call(CallStmt {
                            name: stmt_name,
                            args,
                            line,
                        }));
                    }
                }
                if matches!(
                    name_upper.as_str(),
                    "DIALOG" | "CONTROL" | "MENU" | "TREEVIEW"
                ) {
                    self.advance();
                    self.consume_to_eol();
                    eprintln!("Error: DIALOG/CONTROL/MENU/TREEVIEW: statement has no sub-command on line {}", line);
                    self.error_count += 1;
                    return Ok(Statement::Noop(name_upper.clone(), line));
                }

                let name_clone = name.clone();

                // Check for label: IDENTIFIER:
                if self.peek_at(1) == Some(&Token::Colon) {
                    self.advance(); // consume identifier
                    self.advance(); // consume colon
                    return Ok(Statement::Label(name_clone));
                }

                // Or assignment: IDENTIFIER = expr
                // Or sub call: IDENTIFIER args
                // Or array assignment: IDENTIFIER(idx) = expr

                // Peek ahead to decide
                if self.peek_at(1) == Some(&Token::Eq) {
                    // Simple assignment: name = expr
                    // Inside a FUNCTION body, `FuncName = expr` is the official
                    // return-value assignment (CHW LIBMAIN example).  Same AST
                    // as the `FUNCTION = expr` form (batch 276).
                    if let Some(fn_name) = &self.current_function_name {
                        if *fn_name == name_upper {
                            self.advance(); // consume name
                            self.advance(); // consume =
                            let value = self.parse_expression()?;
                            self.consume_to_eol();
                            return Ok(Statement::FunctionReturn(FunctionReturnStmt {
                                value,
                                line,
                            }));
                        }
                    }
                    self.advance(); // consume name
                    self.advance(); // consume =
                    let value = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Assign(AssignStmt {
                        target: Expr::Variable(name_clone),
                        value,
                        line,
                    }));
                }

                if self.peek_at(1) == Some(&Token::Dot) {
                    // TYPE member assignment: name.member = expr
                    let primary = self.parse_primary()?;
                    if self.peek() == &Token::Eq {
                        self.advance(); // consume =
                        let value = self.parse_expression()?;
                        self.consume_to_eol();
                        return Ok(Statement::Assign(AssignStmt {
                            target: primary,
                            value,
                            line,
                        }));
                    }
                    // `o.M(args)` with no assignment: a statement-position method
                    // call (batch 217). Keep the receiver as argument 0 and mark the
                    // call name with a leading dot - codegen is the layer that knows
                    // the receiver's TYPE and lowers it to `<Class>_<Method>`. A
                    // plain call can never collide with this, because a PB
                    // identifier cannot begin with a dot.
                    if let Expr::MethodCall {
                        base,
                        name: mname,
                        args: margs,
                    } = primary
                    {
                        let mut call_args = Vec::with_capacity(margs.len() + 1);
                        call_args.push(*base);
                        call_args.extend(margs);
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: format!(".{}", mname),
                            args: call_args,
                            line,
                        }));
                    }
                    self.consume_to_eol();
                    eprintln!("Error: unrecognised statement on line {}", line);
                    self.error_count += 1;
                    return Ok(Statement::Noop(name_clone.clone(), line));
                }

                if self.peek_at(1) == Some(&Token::LParen) {
                    // Could be: arr(idx) = expr  (array assignment)
                    // Or:       Sub(args)        (sub call)
                    // Or:       arr(idx).member = expr (UDT array member)
                    // Parse just the primary expression (no comparison operators)
                    let primary = self.parse_primary()?;

                    if self.peek() == &Token::Eq {
                        // Array assignment: arr(idx) = expr
                        self.advance(); // consume =
                        let value = self.parse_expression()?;
                        self.consume_to_eol();
                        return Ok(Statement::Assign(AssignStmt {
                            target: primary,
                            value,
                            line,
                        }));
                    }

                    // It's a sub call with parenthesized args
                    if let Expr::FunctionCall(func_name, func_args) = primary {
                        self.consume_to_eol();
                        return Ok(Statement::Call(CallStmt {
                            name: func_name,
                            args: func_args,
                            line,
                        }));
                    }

                    self.consume_to_eol();
                    eprintln!("Error: unrecognised statement on line {}", line);
                    self.error_count += 1;
                    return Ok(Statement::Noop(name_clone.clone(), line));
                }

                // No LParen after identifier — could be a SUB call: SubName arg1, arg2
                // Or a label target in single-line IF: IF cond THEN LabelName ELSE ...
                self.advance(); // consume identifier
                let mut args = Vec::new();
                // PEEK/POKE datatype keyword as first arg: POKE LONG, addr, 12345
                if args.is_empty() && !self.at_eol_or_eof() {
                    let dt = match self.peek() {
                        Token::Long => Some("LONG"),
                        Token::Double => Some("DOUBLE"),
                        Token::Dword => Some("DWORD"),
                        Token::Integer => Some("INTEGER"),
                        Token::Quad => Some("QUAD"),
                        Token::Single => Some("SINGLE"),
                        _ => None,
                    };
                    if let Some(d) = dt {
                        self.advance();
                        args.push(Expr::Variable(d.to_string()));
                        if self.peek() == &Token::Comma {
                            self.advance();
                        }
                    }
                }
                if !self.at_eol_or_eof() && !matches!(self.peek(), Token::Else | Token::ElseIf) {
                    // BYVAL/BYCOPY modifier in no-paren calls: Foo a, BYVAL %NULL
                    let mut has_byval = matches!(self.peek(), Token::Byval);
                    if has_byval {
                        self.advance(); // consume BYVAL
                    }
                    if matches!(self.peek(), Token::Identifier(ref s) if s.eq_ignore_ascii_case("BYCOPY"))
                    {
                        has_byval = true;
                        self.advance(); // consume BYCOPY
                    }
                    let e = self.parse_expression()?;
                    args.push(if has_byval {
                        Expr::ByvalOverride(Box::new(e))
                    } else {
                        e
                    });
                    while self.peek() == &Token::Comma {
                        self.advance();
                        // PB official optional-arg empty slot: `MSGBOX "t", , "cap"`
                        // (CHW syntax `[, [style&], title$]`; PBWin accepts —
                        // batch 250 probe p3). Represented as Expr::Omitted;
                        // the statement's codegen arm decides the default.
                        if matches!(
                            self.peek(),
                            Token::Comma | Token::Eol | Token::Eof | Token::Colon
                        ) {
                            args.push(Expr::Omitted);
                            continue;
                        }
                        // Each subsequent arg may also carry BYVAL/BYCOPY
                        let mut hb = matches!(self.peek(), Token::Byval);
                        if hb {
                            self.advance();
                        }
                        if matches!(self.peek(), Token::Identifier(ref s2) if s2.eq_ignore_ascii_case("BYCOPY"))
                        {
                            hb = true;
                            self.advance();
                        }
                        let e2 = self.parse_expression()?;
                        args.push(if hb {
                            Expr::ByvalOverride(Box::new(e2))
                        } else {
                            e2
                        });
                    }
                }
                // Check for leftover tokens (syntax error like MSGBOX "text"hgkugyj)
                if !matches!(self.peek(), Token::Eol | Token::Eof | Token::Colon) {
                    let loc = self.peek_located();
                    let leftover = format!("Unexpected token after arguments: {:?}", self.peek());
                    return Err(PbError::parser_at(
                        leftover,
                        loc.file.as_deref(),
                        loc.line,
                        loc.col,
                    ));
                }
                self.consume_to_eol();
                Ok(Statement::Call(CallStmt {
                    name: name_clone,
                    args,
                    line,
                }))
            }
            _ => {
                self.consume_to_eol();
                eprintln!("Error: unrecognised statement on line {}", line);
                self.error_count += 1;
                Ok(Statement::Noop("<unknown token>".to_string(), line))
            }
        }
    }

    fn parse_local_decl(&mut self) -> PbResult<Vec<DimStatement>> {
        let line = self.current_line();
        self.advance(); // LOCAL

        let mut dims: Vec<DimStatement> = Vec::new();
        // PB semantics: `LOCAL aaa, bbb, ccc AS INTEGER` declares ALL names
        // with the trailing type. So a trailing AS both (a) back-fills every
        // previously untyped name in the list and (b) propagates forward to
        // later names until the next AS.
        let mut pending_type: Option<PbType> = None;
        let mut untyped: Vec<usize> = Vec::new(); // indices of names w/o AS yet

        loop {
            let name = self.consume_identifier()?;

            let mut bounds = Vec::new();
            let mut is_array = false;
            if self.peek() == &Token::LParen {
                is_array = true;
                self.advance();
                if self.peek() != &Token::RParen {
                    bounds = self.parse_dim_bounds()?;
                }
                self.expect(&Token::RParen)?;
            }

            let idx = dims.len();
            let pb_type = if self.peek() == &Token::As {
                self.advance();
                let t = self.parse_type()?;
                // Back-fill all previously untyped names with this type
                for i in &untyped {
                    dims[*i].pb_type = t.clone();
                }
                untyped.clear();
                pending_type = Some(t.clone());
                t
            } else if let Some(t) = &pending_type {
                t.clone()
            } else {
                let t = type_from_suffix(&name);
                untyped.push(idx);
                t
            };

            dims.push(DimStatement {
                scope: DimScope::Local,
                name,
                pb_type,
                bounds,
                line,
                is_redim: false,
                is_array,
            });

            if self.peek() == &Token::Comma {
                self.advance();
            } else {
                break;
            }
        }

        self.consume_to_eol();
        Ok(dims)
    }

    /// DEFINT A-E, G, Q, Y-Z — default type for implicitly-declared variables
    /// whose names start with the given letters (CHW DEFBYT_DEFCUR_..._DEFLNG.htm).
    /// Ranges are letter or letter-letter; case-insensitive; later DEFtype
    /// statements override earlier ones where ranges overlap.
    fn parse_deftype_statement(&mut self) -> PbResult<Statement> {
        let line = self.current_line();
        let name = match self.peek() {
            Token::Identifier(n) => n.clone(),
            _ => return Err(PbError::parser("DEFtype expected", None, line)),
        };
        self.advance(); // consume DEFxxx
        let pb_type = deftype_for_name(&name)
            .ok_or_else(|| PbError::parser("unknown DEFtype name", None, line))?;
        let mut ranges: Vec<(char, char)> = Vec::new();
        loop {
            // letter_range := letter | letter '-' letter
            let first = match self.peek() {
                Token::Identifier(c) if c.len() == 1 => {
                    let ch = c.chars().next().unwrap().to_ascii_uppercase();
                    if !ch.is_ascii_alphabetic() {
                        return Err(PbError::parser("DEFtype letter expected", None, line));
                    }
                    ch
                }
                _ => return Err(PbError::parser("DEFtype letter expected", None, line)),
            };
            self.advance();
            if self.peek() == &Token::Minus {
                self.advance();
                let last = match self.peek() {
                    Token::Identifier(c) if c.len() == 1 => {
                        let ch = c.chars().next().unwrap().to_ascii_uppercase();
                        if !ch.is_ascii_alphabetic() {
                            return Err(PbError::parser("DEFtype letter expected", None, line));
                        }
                        ch
                    }
                    _ => return Err(PbError::parser("DEFtype letter expected", None, line)),
                };
                self.advance();
                ranges.push((first, last));
            } else {
                ranges.push((first, first));
            }
            if self.peek() == &Token::Comma {
                self.advance();
            } else {
                break;
            }
        }
        self.consume_to_eol();
        Ok(Statement::DefType(DefTypeStmt {
            pb_type,
            ranges,
            line,
        }))
    }

    fn parse_if_statement(&mut self) -> PbResult<Statement> {
        let line = self.current_line();
        self.advance(); // IF
        let condition = self.parse_expression()?;
        self.expect(&Token::Then)?;

        // Check for single-line IF: IF cond THEN stmt1[:stmt2...] [ELSE stmt3[:stmt4...]]
        if !self.at_eol_or_eof() {
            // Single-line IF — parse one or more colon-separated THEN statements
            let if_line = self.current_line();
            let mut then_body = Vec::new();
            loop {
                let stmt = self.parse_statement()?;
                then_body.push(stmt);
                // Continue with more colon-separated statements on the same line
                if self.peek() == &Token::Colon && self.current_line() == if_line {
                    self.advance(); // consume :
                                    // But check if next token is ELSE (IF cond THEN stmt : ELSE stmt)
                    if self.peek() == &Token::Else {
                        break;
                    }
                } else {
                    break;
                }
            }

            let mut else_body = Vec::new();

            // Check for ELSE on the SAME line only.
            // After parse_statement, we may have crossed a line boundary
            // (e.g. PRINT calls consume_to_eol). Only match ELSE if still
            // on the same line to avoid stealing an outer block IF's ELSE.
            if self.peek() == &Token::Else && self.current_line() == if_line {
                self.advance(); // consume ELSE
                loop {
                    let stmt = self.parse_statement()?;
                    else_body.push(stmt);
                    if self.peek() == &Token::Colon && self.current_line() == if_line {
                        self.advance(); // consume :
                    } else {
                        break;
                    }
                }
            }

            return Ok(Statement::If(IfStmt {
                condition,
                then_body,
                elseif_clauses: Vec::new(),
                else_body,
                line,
            }));
        }

        self.skip_eol();

        // Multi-line IF
        let (then_body, matched) =
            self.parse_body_with_terminator(&[BodyEnd::EndIf, BodyEnd::Else, BodyEnd::ElseIf])?;

        let mut elseif_clauses = Vec::new();
        let mut else_body = Vec::new();

        // Only check for ELSEIF/ELSE if the then_body wasn't terminated by END IF.
        // When a nested IF...END IF is inside an ELSEIF body, the END IF belongs
        // to the nested IF, and the next ELSEIF/ELSE belongs to the OUTER IF.
        if matched != Some(BodyEnd::EndIf) {
            // Check for ELSEIF
            while self.peek() == &Token::ElseIf {
                self.advance();
                let elif_cond = self.parse_expression()?;
                self.expect(&Token::Then)?;
                self.consume_to_eol();
                let (elif_body, elif_matched) = self.parse_body_with_terminator(&[
                    BodyEnd::EndIf,
                    BodyEnd::Else,
                    BodyEnd::ElseIf,
                ])?;
                elseif_clauses.push(ElseIfClause {
                    condition: elif_cond,
                    body: elif_body,
                });
                // If this ELSEIF body ended with END IF, the IF statement is complete
                if elif_matched == Some(BodyEnd::EndIf) {
                    break;
                }
            }

            // Check for ELSE (only if not already terminated by END IF)
            if self.peek() == &Token::Else {
                self.advance();
                self.consume_to_eol();
                else_body = self.parse_body(&[BodyEnd::EndIf])?;
            }
        }

        Ok(Statement::If(IfStmt {
            condition,
            then_body,
            elseif_clauses,
            else_body,
            line,
        }))
    }

    fn parse_for_statement(&mut self) -> PbResult<Statement> {
        let line = self.current_line();
        // batch296: FOR EACH var IN collection ... NEXT — enumeration of a
        // PowerCollection / LinkListCollection object (CHW
        // FOR_EACH_NEXT_statements).  The header form is
        //   FOR EACH VariantVar IN CollectionObjectVar
        // and the loop ends with an optional `NEXT [VariantVar]`.
        if matches!(self.peek_at(1), Some(Token::Identifier(k))
                            if k.eq_ignore_ascii_case("EACH"))
        {
            self.advance(); // FOR
            self.advance(); // EACH
            let var = self.consume_identifier()?;
            let in_kw = self.consume_identifier()?;
            if !in_kw.eq_ignore_ascii_case("IN") {
                return Err(PbError::parser(
                    "FOR EACH expects `IN CollectionObjectVar` after the loop variable",
                    self.current_file(),
                    self.current_line(),
                ));
            }
            let collection = self.parse_expression()?;
            self.consume_to_eol();

            let body = self.parse_body(&[BodyEnd::Next])?;

            // NEXT [var]
            if self.peek() == &Token::Next {
                self.advance();
                if let Token::Identifier(_) = self.peek() {
                    self.advance();
                }
                self.consume_to_eol();
            }

            return Ok(Statement::ForEach(ForEachStmt {
                var,
                collection,
                body,
                line,
            }));
        }
        self.advance(); // FOR
        let var = self.consume_identifier()?;
        self.expect(&Token::Eq)?;
        let start = self.parse_expression()?;
        self.expect(&Token::To)?;
        let end = self.parse_expression()?;
        let step = if self.peek() == &Token::Step {
            self.advance();
            Some(self.parse_expression()?)
        } else {
            None
        };
        self.consume_to_eol();

        let body = self.parse_body(&[BodyEnd::Next])?;

        // NEXT [var]
        if self.peek() == &Token::Next {
            self.advance();
            // Optionally consume the variable name
            if let Token::Identifier(_) = self.peek() {
                self.advance();
            }
            self.consume_to_eol();
        }

        Ok(Statement::For(ForStmt {
            var,
            start,
            end,
            step,
            body,
            line,
        }))
    }

    fn parse_do_loop(&mut self) -> PbResult<Statement> {
        let line = self.current_line();
        self.advance(); // DO

        let mut condition = None;
        let mut is_while = true;
        let mut is_pre_test = false;

        // DO WHILE/UNTIL cond
        if self.peek() == &Token::While {
            self.advance();
            condition = Some(self.parse_expression()?);
            is_while = true;
            is_pre_test = true;
        } else if self.peek() == &Token::Until {
            self.advance();
            condition = Some(self.parse_expression()?);
            is_while = false;
            is_pre_test = true;
        }
        self.consume_to_eol();

        let body = self.parse_body(&[BodyEnd::Loop])?;

        // LOOP [WHILE/UNTIL cond]
        if self.peek() == &Token::Loop {
            self.advance();
            if condition.is_none() {
                if self.peek() == &Token::While {
                    self.advance();
                    condition = Some(self.parse_expression()?);
                    is_while = true;
                } else if self.peek() == &Token::Until {
                    self.advance();
                    condition = Some(self.parse_expression()?);
                    is_while = false;
                }
            }
            self.consume_to_eol();
        }

        Ok(Statement::DoLoop(DoLoopStmt {
            condition,
            is_while,
            is_pre_test,
            body,
            line,
        }))
    }

    fn parse_while_wend(&mut self) -> PbResult<Statement> {
        let line = self.current_line();
        self.advance(); // WHILE
        let condition = self.parse_expression()?;
        self.consume_to_eol();

        let body = self.parse_body(&[BodyEnd::Wend])?;

        if self.peek() == &Token::Wend {
            self.advance();
            self.consume_to_eol();
        }

        Ok(Statement::WhileWend(WhileWendStmt {
            condition,
            body,
            line,
        }))
    }

    fn parse_select_case(&mut self) -> PbResult<Statement> {
        let line = self.current_line();
        self.advance(); // SELECT
        self.expect(&Token::Case)?;
        // Optional AS LONG / AS QUAD type specifier
        if matches!(self.peek(), Token::As) {
            self.advance(); // AS
                            // Skip the type keyword (LONG, QUAD, SINGLE, etc.)
            self.advance();
        }
        let expr = self.parse_expression()?;
        self.consume_to_eol();
        self.skip_eol();

        let mut cases = Vec::new();
        let mut else_body = Vec::new();

        loop {
            if self.at_end() {
                break;
            }
            if self.peek() == &Token::End {
                if self.peek_at(1) == Some(&Token::Select) {
                    self.advance(); // END
                    self.advance(); // SELECT
                    self.consume_to_eol();
                    break;
                }
                // Break on END SUB/FUNCTION — we've overrun into parent scope
                if matches!(self.peek_at(1), Some(&Token::Sub) | Some(&Token::Function)) {
                    break;
                }
            }
            // Break on top-level keywords (new SUB/FUNCTION declaration)
            if self.is_top_level_keyword() {
                break;
            }
            if self.peek() == &Token::Case {
                self.advance();
                if self.peek() == &Token::Else {
                    self.advance();
                    self.consume_to_eol();
                    else_body = self.parse_body(&[BodyEnd::EndSelect, BodyEnd::Case])?;
                    continue;
                }
                // Parse case patterns
                let patterns = self.parse_case_patterns()?;
                self.consume_to_eol();
                let body =
                    self.parse_body(&[BodyEnd::EndSelect, BodyEnd::Case, BodyEnd::CaseElse])?;
                cases.push(CaseClause { patterns, body });
            } else {
                let saved = self.pos;
                self.consume_to_eol();
                self.skip_eol();
                // Safety: prevent infinite loop if consume_to_eol makes no progress
                // (e.g., stuck on Token::Else which consume_to_eol won't consume)
                if self.pos == saved {
                    self.advance();
                }
            }
        }

        Ok(Statement::SelectCase(SelectCaseStmt {
            expr,
            cases,
            else_body,
            line,
        }))
    }

    fn parse_case_patterns(&mut self) -> PbResult<Vec<CasePattern>> {
        let mut patterns = Vec::new();
        loop {
            if self.at_eol_or_eof() {
                break;
            }
            // CASE IS <op> expr
            if self.peek() == &Token::Is {
                self.advance();
                let op = match self.peek() {
                    Token::Lt => CaseOp::Lt,
                    Token::Gt => CaseOp::Gt,
                    Token::Lte => CaseOp::Lte,
                    Token::Gte => CaseOp::Gte,
                    Token::Eq => CaseOp::Eq,
                    Token::Neq => CaseOp::Neq,
                    _ => CaseOp::Eq,
                };
                self.advance();
                let expr = self.parse_expression()?;
                patterns.push(CasePattern::Is(op, expr));
            // CASE < expr, CASE > expr, CASE <= expr, CASE >= expr (without IS keyword)
            } else if matches!(self.peek(), Token::Lt | Token::Gt | Token::Lte | Token::Gte) {
                let op = match self.peek() {
                    Token::Lt => CaseOp::Lt,
                    Token::Gt => CaseOp::Gt,
                    Token::Lte => CaseOp::Lte,
                    Token::Gte => CaseOp::Gte,
                    _ => unreachable!(),
                };
                self.advance();
                let expr = self.parse_expression()?;
                patterns.push(CasePattern::Is(op, expr));
            } else {
                let expr = self.parse_expression()?;
                // Check for range: expr TO expr
                if self.peek() == &Token::To {
                    self.advance();
                    let upper = self.parse_expression()?;
                    patterns.push(CasePattern::Range(expr, upper));
                } else {
                    patterns.push(CasePattern::Value(expr));
                }
            }
            if self.peek() == &Token::Comma {
                self.advance();
            } else {
                break;
            }
        }
        Ok(patterns)
    }

    fn parse_comm_statement(&mut self, line: usize) -> PbResult<Statement> {
        self.advance(); // consume COMM
        let op = match self.peek() {
            Token::Identifier(w) => w.to_uppercase(),
            Token::Open => "OPEN".to_string(),
            Token::Close => "CLOSE".to_string(),
            Token::Print => "PRINT".to_string(),
            _ => {
                self.consume_to_eol();
                eprintln!("Error: COMM: unsupported form on line {}", line);
                self.error_count += 1;
                return Ok(Statement::Noop("COMM".to_string(), line));
            }
        };
        self.advance(); // consume op
        match op.as_str() {
            "OPEN" => {
                // COMM OPEN "COM1:" AS #1 [, BAUD n] [, PARITY p$] [, DATA n] [, STOP n]
                let port = self.parse_expression()?;
                let mut args = vec![port];
                if !matches!(self.peek(), Token::As) {
                    // Batch 206: the channel is not optional.  Without `AS #` this used to
                    // leave the channel out of the argument list, codegen then read the BAUD
                    // slot as the channel, and the port was silently opened on channel 0 -
                    // a session on the wrong channel looked exactly like success.
                    self.consume_to_eol();
                    eprintln!(
                        "Error: COMM OPEN needs its channel - write COMM OPEN \"COM1\" AS #1 [, BAUD n] [, PARITY p$] [, DATA n] [, STOP n] on line {}",
                        line
                    );
                    self.error_count += 1;
                    return Ok(Statement::Noop("COMM OPEN".to_string(), line));
                }
                {
                    self.advance();
                    if self.peek() == &Token::Hash {
                        self.advance();
                    }
                    let ch = self.parse_expression()?;
                    args.push(ch);
                }
                let mut baud = Expr::IntegerLit(0);
                let mut parity = Expr::StringLit(String::new());
                let mut data = Expr::IntegerLit(0);
                let mut stop = Expr::IntegerLit(0);
                let mut wide = Expr::IntegerLit(0);
                while self.peek() == &Token::Comma {
                    self.advance();
                    if let Token::Identifier(w) = self.peek() {
                        let opt = w.to_uppercase();
                        self.advance();
                        let v = self.parse_expression()?;
                        match opt.as_str() {
                            "BAUD" => baud = v,
                            "PARITY" => parity = v,
                            "DATA" => data = v,
                            "STOP" => stop = v,
                            "CHR" => {
                                // CHR = ANSI | WIDE (CHW COMM_OPEN_statement.htm)
                                if let Expr::Variable(w) = &v {
                                    if w.eq_ignore_ascii_case("WIDE") {
                                        wide = Expr::IntegerLit(1);
                                    }
                                }
                            }
                            _ => {}
                        }
                    } else {
                        break;
                    }
                }
                self.consume_to_eol();
                args.push(baud);
                args.push(parity);
                args.push(data);
                args.push(stop);
                args.push(wide);
                Ok(Statement::Call(CallStmt {
                    name: "COMM OPEN".to_string(),
                    args,
                    line,
                }))
            }
            "LINE" => {
                // COMM LINE [INPUT] [#]f, var$ (CHW COMM_LINE_statement.htm:
                // the INPUT keyword is optional)
                if let Token::Input = self.peek() {
                    self.advance();
                }
                if self.peek() == &Token::Hash {
                    self.advance();
                }
                let f = self.parse_expression()?;
                self.expect(&Token::Comma)?;
                let var = self.parse_expression()?;
                self.consume_to_eol();
                Ok(Statement::Call(CallStmt {
                    name: "COMM LINE INPUT".to_string(),
                    args: vec![f, var],
                    line,
                }))
            }
            "PRINT" => {
                // COMM PRINT [#]f, exprlist [;] [TO CharCountVar]
                // (CHW COMM_PRINT_statement.htm: CR/LF auto-appended unless a
                // trailing ';' suppresses it; TO receives the char count.)
                if self.peek() == &Token::Hash {
                    self.advance();
                }
                let mut args = vec![self.parse_expression()?];
                while self.peek() == &Token::Comma {
                    self.advance();
                    args.push(self.parse_expression()?);
                }
                let mut semi = false;
                if self.peek() == &Token::Semicolon {
                    self.advance();
                    semi = true;
                }
                let mut to_var = None;
                if let Token::Identifier(w) = self.peek() {
                    if w.eq_ignore_ascii_case("TO") {
                        self.advance();
                        to_var = Some(self.parse_expression()?);
                    }
                }
                self.consume_to_eol();
                let name = match (semi, to_var.is_some()) {
                    (false, false) => "COMM PRINT".to_string(),
                    (true, false) => "COMM PRINT;".to_string(),
                    (false, true) => "COMM PRINT TO".to_string(),
                    (true, true) => "COMM PRINT;TO".to_string(),
                };
                if let Some(tv) = to_var {
                    args.push(tv);
                }
                Ok(Statement::Call(CallStmt { name, args, line }))
            }
            "RECV" => {
                // COMM RECV [#]f, bytes&, var$
                if self.peek() == &Token::Hash {
                    self.advance();
                }
                let f = self.parse_expression()?;
                self.expect(&Token::Comma)?;
                let bytes = self.parse_expression()?;
                self.expect(&Token::Comma)?;
                let var = self.parse_expression()?;
                self.consume_to_eol();
                Ok(Statement::Call(CallStmt {
                    name: "COMM RECV".to_string(),
                    args: vec![f, bytes, var],
                    line,
                }))
            }
            "RESET" => {
                // COMM RESET [#] hComm , FLOW (CHW COMM_RESET_statement.htm)
                if self.peek() == &Token::Hash {
                    self.advance();
                }
                let f = self.parse_expression()?;
                if let Token::Identifier(w) = self.peek() {
                    if w.eq_ignore_ascii_case("FLOW") {
                        self.advance();
                    }
                }
                self.consume_to_eol();
                Ok(Statement::Call(CallStmt {
                    name: "COMM RESET".to_string(),
                    args: vec![f],
                    line,
                }))
            }
            "SEND" => {
                // COMM SEND [#]f, string$ [TO CharCountVar] (CHW)
                if self.peek() == &Token::Hash {
                    self.advance();
                }
                let f = self.parse_expression()?;
                self.expect(&Token::Comma)?;
                let s = self.parse_expression()?;
                let mut to_var = None;
                if let Token::Identifier(w) = self.peek() {
                    if w.eq_ignore_ascii_case("TO") {
                        self.advance();
                        to_var = Some(self.parse_expression()?);
                    }
                }
                self.consume_to_eol();
                let name = if to_var.is_some() {
                    "COMM SEND TO".to_string()
                } else {
                    "COMM SEND".to_string()
                };
                let mut args = vec![f, s];
                if let Some(tv) = to_var {
                    args.push(tv);
                }
                Ok(Statement::Call(CallStmt { name, args, line }))
            }
            "SET" => {
                // COMM SET [#]f, Comfunc = value  (Comfunc: BAUD/PARITY/STOP/FLOW/DTR/RTS/BREAK...)
                if self.peek() == &Token::Hash {
                    self.advance();
                }
                let f = self.parse_expression()?;
                self.expect(&Token::Comma)?;
                // Comfunc is a keyword, not a variable: take it as a literal token.
                let opt_word = match self.peek() {
                    Token::Identifier(w) => {
                        let w = w.clone();
                        self.advance();
                        w.to_uppercase()
                    }
                    tok => {
                        return Err(crate::error::PbError::parser(
                            format!("COMM SET requires a keyword option after the comma: COMM SET [#] hComm , Comfunc = value (got {:?})", tok),
                            None,
                            line,
                        ));
                    }
                };
                // optional '=' then the value; or bare ON/OFF (DTR/RTS/BREAK legacy form)
                let value = if self.peek() == &Token::Eq {
                    self.advance();
                    self.parse_expression()?
                } else if matches!(self.peek(), Token::Identifier(w) if w.eq_ignore_ascii_case("ON"))
                {
                    self.advance();
                    Expr::IntegerLit(1)
                } else if matches!(self.peek(), Token::Identifier(w) if w.eq_ignore_ascii_case("OFF"))
                {
                    self.advance();
                    Expr::IntegerLit(0)
                } else {
                    Expr::IntegerLit(0)
                };
                self.consume_to_eol();
                Ok(Statement::Call(CallStmt {
                    name: "COMM SET".to_string(),
                    args: vec![f, Expr::StringLit(opt_word), value],
                    line,
                }))
            }
            "TIMEOUT" => {
                // COMM TIMEOUT [#]f, ms&
                if self.peek() == &Token::Hash {
                    self.advance();
                }
                let f = self.parse_expression()?;
                self.expect(&Token::Comma)?;
                let ms = self.parse_expression()?;
                self.consume_to_eol();
                Ok(Statement::Call(CallStmt {
                    name: "COMM TIMEOUT".to_string(),
                    args: vec![f, ms],
                    line,
                }))
            }
            "CLOSE" => {
                // COMM CLOSE [#] hComm [, [#] hComm ...] (CHW: synonym of CLOSE)
                let mut args = Vec::new();
                loop {
                    if self.peek() == &Token::Hash {
                        self.advance();
                    }
                    args.push(self.parse_expression()?);
                    if self.peek() == &Token::Comma {
                        self.advance();
                    } else {
                        break;
                    }
                }
                self.consume_to_eol();
                Ok(Statement::Call(CallStmt {
                    name: "COMM CLOSE".to_string(),
                    args,
                    line,
                }))
            }
            _ => {
                self.consume_to_eol();
                eprintln!("Error: COMM: unsupported sub-command on line {}", line);
                self.error_count += 1;
                Ok(Statement::Noop(format!("COMM {op}"), line))
            }
        }
    }

    fn parse_thread_statement(&mut self, line: usize) -> PbResult<Statement> {
        self.advance(); // consume THREAD
        let op = match self.peek() {
            Token::Identifier(w) => w.to_uppercase(),
            Token::Close => "CLOSE".to_string(),
            Token::Resume => "RESUME".to_string(),
            _ => {
                self.consume_to_eol();
                eprintln!("Error: THREAD: unsupported form on line {}", line);
                self.error_count += 1;
                return Ok(Statement::Noop("THREAD".to_string(), line));
            }
        };
        self.advance(); // consume op
        match op.as_str() {
            "CREATE" => {
                // THREAD CREATE func [TO id&]  /  THREAD CREATE func(args) TO id&
                let func = self.parse_primary()?;
                let mut args = vec![func];
                if matches!(self.peek(), Token::To) {
                    self.advance();
                    let id = self.parse_expression()?;
                    args.push(id);
                }
                self.consume_to_eol();
                Ok(Statement::Call(CallStmt {
                    name: "THREAD CREATE".to_string(),
                    args,
                    line,
                }))
            }
            "CLOSE" | "SUSPEND" | "RESUME" => {
                let id = self.parse_expression()?;
                self.consume_to_eol();
                Ok(Statement::Call(CallStmt {
                    name: format!("THREAD {op}"),
                    args: vec![id],
                    line,
                }))
            }
            "STATUS" => {
                // THREAD STATUS id& TO st&
                let id = self.parse_expression()?;
                let mut args = vec![id];
                if matches!(self.peek(), Token::To) {
                    self.advance();
                    let st = self.parse_expression()?;
                    args.push(st);
                }
                self.consume_to_eol();
                Ok(Statement::Call(CallStmt {
                    name: "THREAD STATUS".to_string(),
                    args,
                    line,
                }))
            }
            "GET" => {
                // THREAD GET PRIORITY id& TO p&
                let sub = match self.peek() {
                    Token::Identifier(w) => w.to_uppercase(),
                    _ => String::new(),
                };
                self.advance();
                let id = self.parse_expression()?;
                let mut args = vec![id];
                if matches!(self.peek(), Token::To) {
                    self.advance();
                    let p = self.parse_expression()?;
                    args.push(p);
                }
                self.consume_to_eol();
                Ok(Statement::Call(CallStmt {
                    name: format!("THREAD GET {sub}"),
                    args,
                    line,
                }))
            }
            "SET" => {
                // THREAD SET PRIORITY id&, p&
                let sub = match self.peek() {
                    Token::Identifier(w) => w.to_uppercase(),
                    _ => String::new(),
                };
                self.advance();
                let id = self.parse_expression()?;
                self.expect(&Token::Comma)?;
                let p = self.parse_expression()?;
                self.consume_to_eol();
                Ok(Statement::Call(CallStmt {
                    name: format!("THREAD SET {sub}"),
                    args: vec![id, p],
                    line,
                }))
            }
            _ => {
                self.consume_to_eol();
                eprintln!("Error: THREAD: unsupported sub-command on line {}", line);
                self.error_count += 1;
                Ok(Statement::Noop(format!("THREAD {op}"), line))
            }
        }
    }

    // LPRINT: LPRINT [expr list] / LPRINT ATTACH device$ / LPRINT CLOSE / FLUSH / FORMFEED
    /// Upper-case the next token as a plain word, mapping reserved-word
    /// tokens that double as TRACE/IMPORT operators (CLOSE, PRINT, OPEN, ON).
    fn peek_plain_upper(&self) -> String {
        match self.peek() {
            Token::Identifier(w) => w.to_uppercase(),
            Token::Close => "CLOSE".to_string(),
            Token::Print => "PRINT".to_string(),
            Token::Open => "OPEN".to_string(),
            Token::On => "ON".to_string(),
            Token::Dword => "DWORD".to_string(),
            Token::End => "END".to_string(),
            _ => String::new(),
        }
    }

    /// Uppercase spelling of the next word, for DDT family dispatch.
    ///
    /// `peek_plain_upper()` only reports identifiers plus a handful of keyword
    /// tokens, but some DDT verbs and nouns are keywords: `SELECT` is lexed as
    /// `Token::Select`, so `COMBOBOX SELECT hDlg, id&, item&` and
    /// `COMBOBOX GET SELECT hDlg, id& TO datav&` were silently falling through
    /// to expression parsing ("Unexpected token in expression: Select").
    ///
    /// Kept separate from `peek_plain_upper()` on purpose: that one has ~50 call
    /// sites, and widening it could change unrelated dispatch (e.g. `SELECT CASE`).
    fn peek_word_upper(&self) -> String {
        match self.peek() {
            Token::Identifier(w) => w.to_uppercase(),
            Token::Select => "SELECT".to_string(),
            _ => String::new(),
        }
    }

    /// Uppercase spelling of the next word for the GRAPHIC verb slot.
    ///
    /// `INPUT` is `Token::Input` - a keyword, not an identifier - so
    /// `peek_plain_upper()` reports "" for `GRAPHIC INPUT FLUSH` and the whole
    /// statement fell through to "unknown statement `GRAPHIC`".  Scoped to the
    /// GRAPHIC family on purpose, following `peek_treeview_word_upper()`: the
    /// plain helper drives ~50 dispatch sites and must not be widened.
    fn peek_graphic_word_upper(&self) -> String {
        match self.peek() {
            Token::Identifier(w) => w.to_uppercase(),
            Token::Input => "INPUT".to_string(),
            _ => String::new(),
        }
    }

    /// Uppercase spelling of the next word for the TREEVIEW verb/noun slots.
    ///
    /// Two TREEVIEW nouns are keyword tokens rather than identifiers:
    /// `NEXT` is `Token::Next` (the FOR/NEXT keyword) and `SELECT` is
    /// `Token::Select`. Neither `peek_plain_upper()` nor the wider
    /// `peek_word_upper()` reports `NEXT`, so `TREEVIEW GET NEXT ...` parsed
    /// as `Statement::Noop("TREEVIEW GET")` - the compiler reported
    /// "success" while the statement silently did nothing, which is exactly
    /// the defect class this fork exists to remove.
    ///
    /// Kept local to the TREEVIEW family instead of widening
    /// `peek_word_upper()`: that one drives the COMBOBOX/LISTBOX dispatch,
    /// and a new `NEXT` spelling there would be an unrelated behaviour
    /// change. `SELECT` is reported by both on purpose.
    fn peek_treeview_word_upper(&self) -> String {
        match self.peek() {
            Token::Identifier(w) => w.to_uppercase(),
            Token::Next => "NEXT".to_string(),
            Token::Select => "SELECT".to_string(),
            _ => String::new(),
        }
    }
    /// Like `peek_plain_upper()`, but also reports the STRING keyword token as
    /// the word "STRING".  The lexer emits Token::String_ for the word STRING
    /// (it is a type keyword, not an identifier), so plain identifier matching
    /// silently misses it - the cause of several batch 159 parser defects.
    fn peek_ident_or_string_keyword(&self) -> Option<String> {
        match self.peek() {
            Token::Identifier(w) => Some(w.clone()),
            Token::String_ => Some("STRING".to_string()),
            _ => None,
        }
    }

    fn parse_lprint_statement(&mut self, line: usize) -> PbResult<Statement> {
        self.advance(); // consume LPRINT
        match self.peek() {
            Token::Identifier(w) if w.eq_ignore_ascii_case("ATTACH") => {
                self.advance();
                let device = self.parse_expression()?;
                self.consume_to_eol();
                Ok(Statement::Call(CallStmt {
                    name: "LPRINT ATTACH".to_string(),
                    args: vec![device],
                    line,
                }))
            }
            Token::Close => {
                self.advance();
                self.consume_to_eol();
                Ok(Statement::Call(CallStmt {
                    name: "LPRINT CLOSE".to_string(),
                    args: vec![],
                    line,
                }))
            }
            Token::Identifier(w) if w.eq_ignore_ascii_case("CLOSE") => {
                self.advance();
                self.consume_to_eol();
                Ok(Statement::Call(CallStmt {
                    name: "LPRINT CLOSE".to_string(),
                    args: vec![],
                    line,
                }))
            }
            Token::Identifier(w) if w.eq_ignore_ascii_case("FLUSH") => {
                self.advance();
                self.consume_to_eol();
                Ok(Statement::Call(CallStmt {
                    name: "LPRINT FLUSH".to_string(),
                    args: vec![],
                    line,
                }))
            }
            Token::Identifier(w) if w.eq_ignore_ascii_case("FORMFEED") => {
                self.advance();
                self.consume_to_eol();
                Ok(Statement::Call(CallStmt {
                    name: "LPRINT FORMFEED".to_string(),
                    args: vec![],
                    line,
                }))
            }
            _ => {
                // LPRINT [expr][SPC(n)][TAB(n)][,][;] — same arg grammar as PRINT
                let (args, _trailing) = self.parse_print_args()?;
                self.consume_to_eol();
                Ok(Statement::Call(CallStmt {
                    name: "LPRINT".to_string(),
                    args,
                    line,
                }))
            }
        }
    }

    // TRACE: TRACE NEW fname$ / ON / OFF / PRINT expr / CLOSE
    fn parse_trace_statement(&mut self, line: usize) -> PbResult<Statement> {
        self.advance(); // consume TRACE
        let op = self.peek_plain_upper();
        self.advance();
        match op.as_str() {
            "NEW" => {
                let fname = self.parse_expression()?;
                self.consume_to_eol();
                Ok(Statement::Call(CallStmt {
                    name: "TRACE NEW".to_string(),
                    args: vec![fname],
                    line,
                }))
            }
            "PRINT" => {
                let expr = self.parse_expression()?;
                self.consume_to_eol();
                Ok(Statement::Call(CallStmt {
                    name: "TRACE PRINT".to_string(),
                    args: vec![expr],
                    line,
                }))
            }
            "ON" | "OFF" => {
                self.consume_to_eol();
                Ok(Statement::Call(CallStmt {
                    name: format!("TRACE {op}"),
                    args: vec![],
                    line,
                }))
            }
            "CLOSE" => {
                self.consume_to_eol();
                Ok(Statement::Call(CallStmt {
                    name: "TRACE CLOSE".to_string(),
                    args: vec![],
                    line,
                }))
            }
            _ => {
                self.consume_to_eol();
                eprintln!("Error: TRACE: unsupported form on line {}", line);
                self.error_count += 1;
                Ok(Statement::Noop("TRACE".to_string(), line))
            }
        }
    }

    // IMPORT: IMPORT ADDR ProcName$, LibName$ TO AddrVar& [,HndlVar&] / IMPORT CLOSE Hndl
    fn parse_import_statement(&mut self, line: usize) -> PbResult<Statement> {
        self.advance(); // consume IMPORT
        let op = self.peek_plain_upper();
        self.advance();
        match op.as_str() {
            "ADDR" => {
                let procname = self.parse_expression()?;
                if self.peek() == &Token::Comma {
                    self.advance();
                }
                let libname = self.parse_expression()?;
                let mut args = vec![procname, libname];
                if matches!(self.peek(), Token::To) {
                    self.advance();
                    args.push(self.parse_expression()?);
                }
                if self.peek() == &Token::Comma {
                    self.advance();
                    args.push(self.parse_expression()?);
                }
                self.consume_to_eol();
                Ok(Statement::Call(CallStmt {
                    name: "IMPORT ADDR".to_string(),
                    args,
                    line,
                }))
            }
            "CLOSE" => {
                let hndl = self.parse_expression()?;
                self.consume_to_eol();
                Ok(Statement::Call(CallStmt {
                    name: "IMPORT CLOSE".to_string(),
                    args: vec![hndl],
                    line,
                }))
            }
            _ => {
                self.consume_to_eol();
                eprintln!("Error: IMPORT: unsupported form on line {}", line);
                self.error_count += 1;
                Ok(Statement::Noop("IMPORT".to_string(), line))
            }
        }
    }

    // CALL DWORD TargetPtr [USING proto(args)] [TO result]
    fn parse_call_dword(&mut self, line: usize) -> PbResult<Statement> {
        self.advance(); // consume DWORD
        let target = self.parse_expression()?;
        let mut args = vec![target];
        let mut has_result = false;
        if let Token::Identifier(w) = self.peek() {
            if w.eq_ignore_ascii_case("USING") {
                self.advance();
                // Proto name: consume a PLAIN IDENTIFIER only.  parse_primary()
                // would swallow the "(args)" list as a call expression, which is
                // exactly what used to happen here: the argument list was never
                // collected and CALL DWORD always called the target with zero
                // arguments (measured: IsWindow(hwnd) returned 0 for a live hwnd,
                // while the no-argument GetTickCount() worked).
                if matches!(self.peek(), Token::Identifier(_)) {
                    self.advance();
                } else {
                    let _proto = self.parse_primary()?;
                }
                if self.peek() == &Token::LParen {
                    self.advance();
                    while !self.at_eol_or_eof() && self.peek() != &Token::RParen {
                        args.push(self.parse_expression()?);
                        if self.peek() == &Token::Comma {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    if self.peek() == &Token::RParen {
                        self.advance();
                    }
                }
                if matches!(self.peek(), Token::To) {
                    self.advance();
                    args.push(self.parse_expression()?);
                    has_result = true;
                }
            }
        }
        self.consume_to_eol();
        Ok(Statement::Call(CallStmt {
            name: if has_result {
                "CALL DWORD TO".to_string()
            } else {
                "CALL DWORD".to_string()
            },
            args,
            line,
        }))
    }

    fn parse_clipboard_format(&mut self) -> Option<ClipboardFormat> {
        if let Token::Identifier(s) = self.peek() {
            if s.eq_ignore_ascii_case("TEXT") {
                self.advance();
                return Some(ClipboardFormat::Text);
            }
            if s.eq_ignore_ascii_case("OEMTEXT") {
                self.advance();
                return Some(ClipboardFormat::OemText);
            }
            if s.eq_ignore_ascii_case("UNICODE") {
                self.advance();
                return Some(ClipboardFormat::Unicode);
            }
            if s.eq_ignore_ascii_case("BITMAP") {
                self.advance();
                return Some(ClipboardFormat::Bitmap);
            }
        }
        None
    }

    fn parse_clipboard_statement(&mut self, line: usize) -> PbResult<Statement> {
        self.advance(); // consume CLIPBOARD
        match self.peek() {
            Token::Identifier(s) if s.eq_ignore_ascii_case("SET") => {
                self.advance();
                let fmt = self.parse_clipboard_format();
                if let Some(format) = fmt {
                    let value = self.parse_expression()?;
                    /* batch 262: PBWin10 accepts BOTH `, ClipResult` (CHW
                    syntax) and `TO ClipResult` for the optional result
                    (probe: CLIPBOARD SET TEXT "abc" TO r -> r = -1). */
                    let result = if self.peek() == &Token::Comma || self.peek() == &Token::To {
                        self.advance();
                        Some(self.parse_expression()?)
                    } else {
                        None
                    };
                    self.consume_to_eol();
                    return Ok(Statement::ClipboardSet {
                        format,
                        value,
                        result,
                    });
                }
                self.consume_to_eol();
                eprintln!("Error: CLIPBOARD SET: unsupported form on line {}", line);
                self.error_count += 1;
                Ok(Statement::Noop("CLIPBOARD SET".to_string(), line))
            }
            Token::Identifier(s) if s.eq_ignore_ascii_case("GET") => {
                self.advance();
                let fmt = self.parse_clipboard_format();
                if let Some(format) = fmt {
                    if self.peek() == &Token::To {
                        self.advance();
                        let target = self.parse_expression()?;
                        let result = if self.peek() == &Token::Comma {
                            self.advance();
                            Some(self.parse_expression()?)
                        } else {
                            None
                        };
                        self.consume_to_eol();
                        return Ok(Statement::ClipboardGet {
                            format,
                            target,
                            result,
                        });
                    }
                }
                self.consume_to_eol();
                eprintln!("Error: CLIPBOARD GET: unsupported form on line {}", line);
                self.error_count += 1;
                Ok(Statement::Noop("CLIPBOARD GET".to_string(), line))
            }
            Token::Identifier(s) if s.eq_ignore_ascii_case("RESET") => {
                self.advance();
                let result = if self.peek() == &Token::Comma {
                    self.advance();
                    Some(self.parse_expression()?)
                } else {
                    None
                };
                self.consume_to_eol();
                Ok(Statement::ClipboardReset { result })
            }
            _ => {
                self.consume_to_eol();
                eprintln!("Error: CLIPBOARD: unknown sub-command on line {}", line);
                self.error_count += 1;
                Ok(Statement::Noop("CLIPBOARD".to_string(), line))
            }
        }
    }

    fn parse_globalmem_statement(&mut self, line: usize) -> PbResult<Statement> {
        self.advance(); // consume GLOBALMEM
        let op = match self.peek() {
            Token::Identifier(w) => w.to_uppercase(),
            _ => {
                self.consume_to_eol();
                eprintln!("Error: GLOBALMEM: expected a sub-command on line {}", line);
                self.error_count += 1;
                return Ok(Statement::Noop("GLOBALMEM".to_string(), line));
            }
        };
        if !matches!(op.as_str(), "ALLOC" | "FREE" | "LOCK" | "SIZE" | "UNLOCK") {
            self.consume_to_eol();
            eprintln!("Error: GLOBALMEM: unsupported sub-command on line {}", line);
            self.error_count += 1;
            return Ok(Statement::Noop(format!("GLOBALMEM {op}"), line));
        }
        self.advance(); // consume op
        let mut args = vec![self.parse_expression()?];
        // Official syntax puts TO directly after the first argument, with no comma:
        //   GLOBALMEM ALLOC 64 TO h     GLOBALMEM LOCK m TO p
        // Only the comma form was consumed, so the official spelling parsed to a single
        // argument, failed the `args.len() >= 2` guard in codegen, and became a silent
        // no-op (probe: `mem handle = 0`).
        if matches!(self.peek(), Token::To) {
            self.advance();
            if !matches!(self.peek(), Token::Eol | Token::Colon) {
                args.push(self.parse_expression()?);
            }
        }
        // Parse remaining comma-separated args
        while self.peek() == &Token::Comma {
            self.advance();
            if self.peek() == &Token::To {
                self.advance();
            }
            if !matches!(self.peek(), Token::Eol | Token::Colon) {
                args.push(self.parse_expression()?);
            }
        }
        self.consume_to_eol();
        Ok(Statement::Call(CallStmt {
            name: format!("GLOBALMEM {op}"),
            args,
            line,
        }))
    }

    fn parse_mouseptr_statement(&mut self, line: usize) -> PbResult<Statement> {
        self.advance(); // consume MOUSEPTR
        let mut args = vec![self.parse_expression()?];
        if self.peek() == &Token::To {
            self.advance(); // consume TO (reserved word)
            args.push(self.parse_expression()?);
        }
        self.consume_to_eol();
        Ok(Statement::Call(CallStmt {
            name: "MOUSEPTR".to_string(),
            args,
            line,
        }))
    }

    fn parse_net_statement(&mut self, line: usize) -> PbResult<Statement> {
        let proto = if let Token::Identifier(w) = self.peek() {
            w.to_uppercase()
        } else {
            self.consume_to_eol();
            eprintln!(
                "Error: NET: expected TCP or UDP followed by a sub-command on line {}",
                line
            );
            self.error_count += 1;
            return Ok(Statement::Noop("NET".to_string(), line));
        };
        self.advance(); // consume TCP/UDP
        let op = match self.peek() {
            Token::Identifier(w) => w.to_uppercase(),
            Token::Open => "OPEN".to_string(),
            Token::Print => "PRINT".to_string(),
            Token::Close => "CLOSE".to_string(),
            _ => {
                self.consume_to_eol();
                eprintln!(
                    "Error: NET: expected a sub-command after the protocol on line {}",
                    line
                );
                self.error_count += 1;
                return Ok(Statement::Noop(proto.clone(), line));
            }
        };
        self.advance(); // consume op

        match op.as_str() {
            "OPEN" => {
                if proto == "UDP" {
                    // UDP OPEN [PORT expr] AS #f [TIMEOUT t]
                    let mut port = Expr::IntegerLit(0);
                    if let Token::Identifier(w) = self.peek() {
                        if w.eq_ignore_ascii_case("PORT") {
                            self.advance();
                            port = self.parse_expression()?;
                        }
                    }
                    let mut filenum = Expr::IntegerLit(0);
                    if self.peek() == &Token::As {
                        self.advance();
                        if self.peek() == &Token::Hash {
                            self.advance();
                        }
                        filenum = self.parse_expression()?;
                    }
                    let mut timeout = Expr::IntegerLit(-1);
                    if let Token::Identifier(w) = self.peek() {
                        if w.eq_ignore_ascii_case("TIMEOUT") {
                            self.advance();
                            timeout = self.parse_expression()?;
                        }
                    }
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "UDP OPEN".to_string(),
                        args: vec![port, filenum, timeout],
                        line,
                    }));
                }
                // TCP OPEN [SERVER] [ADDR bind_expr] {PORT expr | srvc_expr} [AT addr_expr] AS #f [TIMEOUT t]
                let mut mode = Expr::IntegerLit(0);
                let mut port: Option<Expr> = None;
                let mut addr = Expr::StringLit("".to_string());
                let mut bind_addr = Expr::IntegerLit(0);
                let mut timeout = Expr::IntegerLit(-1);
                if let Token::Identifier(w) = self.peek() {
                    if w.eq_ignore_ascii_case("SERVER") {
                        self.advance();
                        mode = Expr::IntegerLit(1);
                        // optional ADDR ip& after SERVER: numeric bind address in
                        // PBWin network order (CHW TCP OPEN server syntax)
                        if let Token::Identifier(w2) = self.peek() {
                            if w2.eq_ignore_ascii_case("ADDR") {
                                self.advance();
                                bind_addr = self.parse_expression()?;
                            }
                        }
                    }
                }
                if let Token::Identifier(w) = self.peek() {
                    if w.eq_ignore_ascii_case("PORT") {
                        self.advance();
                        port = Some(self.parse_expression()?);
                    } else if !w.eq_ignore_ascii_case("AT")
                        && !w.eq_ignore_ascii_case("AS")
                        && !w.eq_ignore_ascii_case("TIMEOUT")
                    {
                        // service name or numeric service string
                        port = Some(self.parse_expression()?);
                    }
                }
                let port_v = port.unwrap_or(Expr::IntegerLit(0));
                if let Token::Identifier(w) = self.peek() {
                    if w.eq_ignore_ascii_case("AT") {
                        self.advance();
                        addr = self.parse_expression()?;
                    }
                }
                // AS #f
                let mut filenum = Expr::IntegerLit(0);
                if self.peek() == &Token::As {
                    self.advance();
                    if self.peek() == &Token::Hash {
                        self.advance();
                    }
                    filenum = self.parse_expression()?;
                }
                while let Token::Identifier(w) = self.peek() {
                    if w.eq_ignore_ascii_case("TIMEOUT") {
                        self.advance();
                        timeout = self.parse_expression()?;
                        break;
                    }
                    self.advance();
                }
                self.consume_to_eol();
                Ok(Statement::Call(CallStmt {
                    name: "TCP OPEN".to_string(),
                    args: vec![mode, port_v, addr, filenum, timeout, bind_addr],
                    line,
                }))
            }
            "ACCEPT" => {
                if self.peek() == &Token::Hash {
                    self.advance();
                }
                let srv = self.parse_expression()?;
                self.expect(&Token::As)?;
                if self.peek() == &Token::Hash {
                    self.advance();
                }
                let newf = self.parse_expression()?;
                self.consume_to_eol();
                Ok(Statement::Call(CallStmt {
                    name: format!("{proto} ACCEPT"),
                    args: vec![srv, newf],
                    line,
                }))
            }
            "LINE" => {
                if let Token::Input = self.peek() {
                    self.advance();
                }
                if self.peek() == &Token::Hash {
                    self.advance();
                }
                let f = self.parse_expression()?;
                self.expect(&Token::Comma)?;
                let var = self.parse_expression()?;
                self.consume_to_eol();
                Ok(Statement::Call(CallStmt {
                    name: format!("{proto} LINE INPUT"),
                    args: vec![f, var],
                    line,
                }))
            }
            "PRINT" => {
                if self.peek() == &Token::Hash {
                    self.advance();
                }
                let f = self.parse_expression()?;
                self.expect(&Token::Comma)?;
                let data = self.parse_expression()?;
                let mut nl = Expr::IntegerLit(1);
                if self.peek() == &Token::Semicolon {
                    self.advance();
                    nl = Expr::IntegerLit(0);
                }
                self.consume_to_eol();
                Ok(Statement::Call(CallStmt {
                    name: format!("{proto} PRINT"),
                    args: vec![f, data, nl],
                    line,
                }))
            }
            "SEND" => {
                if proto == "UDP" {
                    // UDP SEND #f, AT ip&, port&, data$
                    if self.peek() == &Token::Hash {
                        self.advance();
                    }
                    let f = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    if let Token::Identifier(w) = self.peek() {
                        if w.eq_ignore_ascii_case("AT") {
                            self.advance();
                        }
                    }
                    let ip = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let port = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let data = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "UDP SEND".to_string(),
                        args: vec![f, ip, port, data],
                        line,
                    }));
                }
                if self.peek() == &Token::Hash {
                    self.advance();
                }
                let f = self.parse_expression()?;
                self.expect(&Token::Comma)?;
                let data = self.parse_expression()?;
                self.consume_to_eol();
                Ok(Statement::Call(CallStmt {
                    name: "TCP SEND".to_string(),
                    args: vec![f, data],
                    line,
                }))
            }
            "RECV" => {
                if proto == "UDP" {
                    // UDP RECV #f, FROM ip&, port&, buf$
                    if self.peek() == &Token::Hash {
                        self.advance();
                    }
                    let f = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    if let Token::Identifier(w) = self.peek() {
                        if w.eq_ignore_ascii_case("FROM") {
                            self.advance();
                        }
                    }
                    let ip = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let port = self.parse_expression()?;
                    self.expect(&Token::Comma)?;
                    let buf = self.parse_expression()?;
                    self.consume_to_eol();
                    return Ok(Statement::Call(CallStmt {
                        name: "UDP RECV".to_string(),
                        args: vec![f, ip, port, buf],
                        line,
                    }));
                }
                if self.peek() == &Token::Hash {
                    self.advance();
                }
                let f = self.parse_expression()?;
                self.expect(&Token::Comma)?;
                let count = self.parse_expression()?;
                self.expect(&Token::Comma)?;
                let var = self.parse_expression()?;
                self.consume_to_eol();
                Ok(Statement::Call(CallStmt {
                    name: "TCP RECV".to_string(),
                    args: vec![f, count, var],
                    line,
                }))
            }
            "CLOSE" => {
                if self.peek() == &Token::Hash {
                    self.advance();
                }
                let f = self.parse_expression()?;
                self.consume_to_eol();
                Ok(Statement::Call(CallStmt {
                    name: format!("{proto} CLOSE"),
                    args: vec![f],
                    line,
                }))
            }
            _ => {
                self.consume_to_eol();
                eprintln!("Error: NET: unsupported sub-command on line {}", line);
                self.error_count += 1;
                Ok(Statement::Noop(format!("{proto} {op}"), line))
            }
        }
    }

    fn parse_ucodepage_statement(&mut self, line: usize) -> PbResult<Statement> {
        self.advance(); // consume UCODEPAGE
        let first = match self.peek() {
            Token::Identifier(w) => {
                let u = w.to_uppercase();
                if u == "ANSI" {
                    self.advance();
                    Expr::IntegerLit(0)
                } else if u == "OEM" {
                    self.advance();
                    Expr::IntegerLit(1)
                } else {
                    self.parse_expression()?
                }
            }
            _ => self.parse_expression()?,
        };
        let mut args = vec![first];
        if self.peek() == &Token::To {
            self.advance();
            args.push(self.parse_expression()?);
        }
        self.consume_to_eol();
        Ok(Statement::Call(CallStmt {
            name: "UCODEPAGE".to_string(),
            args,
            line,
        }))
    }

    fn parse_print_statement(&mut self) -> PbResult<Statement> {
        let line = self.current_line();
        self.advance(); // PRINT

        // PRINT #filenum, args
        if self.peek() == &Token::Hash {
            self.advance();
            let file_num = self.parse_expression()?;
            self.expect(&Token::Comma)?;
            let (args, trailing) = self.parse_print_args()?;
            self.consume_to_eol();
            return Ok(Statement::PrintFile(PrintFileStmt {
                file_num,
                args,
                trailing,
                line,
            }));
        }

        let (args, trailing) = self.parse_print_args()?;
        self.consume_to_eol();
        Ok(Statement::Print(PrintStmt {
            args,
            trailing,
            line,
        }))
    }

    /// PRINT / LPRINT argument list, plus the statement's trailing separator.
    ///
    /// `PRINT a;` and `PRINT a,` suppress the newline so the next PRINT
    /// continues on the same line.  A separator followed by another argument is
    /// an inter-argument separator and does not set the flag - only one left at
    /// the end of the statement does.
    fn parse_print_args(&mut self) -> PbResult<(Vec<Expr>, Option<char>)> {
        let mut args = Vec::new();
        let mut trailing: Option<char> = None;
        while !self.at_eol_or_eof() {
            args.push(self.parse_expression()?);
            // PRINT uses ; or , both between arguments and at the end
            match self.peek() {
                Token::Semicolon => {
                    trailing = Some(';');
                    self.advance();
                }
                Token::Comma => {
                    trailing = Some(',');
                    self.advance();
                }
                _ => break,
            }
            // A separator followed by more input was an inter-argument one.
            if !self.at_eol_or_eof() {
                trailing = None;
            }
        }
        Ok((args, trailing))
    }

    fn parse_open_statement(&mut self) -> PbResult<Statement> {
        let line = self.current_line();
        self.advance(); // OPEN

        let filename = self.parse_expression()?;

        self.expect(&Token::For)?;

        let mode = match self.peek() {
            Token::Output => {
                self.advance();
                OpenMode::Output
            }
            Token::Append => {
                self.advance();
                OpenMode::Append
            }
            Token::Input => {
                self.advance();
                OpenMode::Input
            }
            Token::Identifier(w) if w.to_uppercase() == "BINARY" => {
                self.advance();
                OpenMode::Binary
            }
            Token::Identifier(w) if w.to_uppercase() == "RANDOM" => {
                self.advance();
                OpenMode::Random
            }
            _ => {
                self.consume_to_eol();
                eprintln!("Error: OPEN: unknown file mode on line {}", line);
                self.error_count += 1;
                return Ok(Statement::Noop("OPEN (unknown mode)".to_string(), line));
            }
        };

        self.expect(&Token::As)?;
        // # is optional: OPEN file FOR OUTPUT AS #num  or  AS num
        if self.peek() == &Token::Hash {
            self.advance();
        }
        let file_num = self.parse_expression()?;

        // Optional LEN=nnn for RANDOM mode (record length)
        let mut reclen: Option<Expr> = None;
        if let Token::Identifier(w) = self.peek() {
            if w.eq_ignore_ascii_case("LEN") {
                self.advance();
                // '=' may be Eq token or part of identifier context; consume any '='
                // '=' is Token::Eq
                if self.peek() == &Token::Eq {
                    self.advance();
                }
                reclen = Some(self.parse_expression()?);
            }
        }

        self.consume_to_eol();
        Ok(Statement::Open(OpenStmt {
            filename,
            mode,
            file_num,
            reclen,
            line,
        }))
    }

    fn parse_close_statement(&mut self) -> PbResult<Statement> {
        let line = self.current_line();
        self.advance(); // CLOSE
                        // # is optional in PowerBASIC: CLOSE #filenum or CLOSE filenum
        if self.peek() == &Token::Hash {
            self.advance();
        }
        if self.at_eol_or_eof() {
            // Bare CLOSE with no file number — close all files
            self.consume_to_eol();
            eprintln!(
                "Error: CLOSE: bare CLOSE (close all files) is not implemented on line {}",
                line
            );
            self.error_count += 1;
            return Ok(Statement::Noop("CLOSE (no file)".to_string(), line));
        }
        let file_num = self.parse_expression()?;
        self.consume_to_eol();
        Ok(Statement::Close(CloseStmt { file_num, line }))
    }

    /// OBJECT CALL/GET/LET/SET obj.member[(args)] [TO result] — DISPATCH access
    /// (batch278-74).  The `OBJECT CALL/GET/LET/SET` keyword run has already been
    /// consumed; `kind` is the uppercase form ("CALL"/"GET"/"LET"/"SET").
    /// Per CHW: args/result must be COM-compatible VARIABLES (literals and
    /// expressions are rejected).
    fn parse_object_stmt(&mut self, kind: &str, line: usize) -> PbResult<Statement> {
        // interface.member[.member...] — parse manually so `o.M` stays a plain
        // object reference and is NOT lowered to Expr::MethodCall (which would
        // route it through the internal CLASS method machinery).
        let object = match self.peek().clone() {
            Token::Identifier(v) => {
                self.advance();
                Expr::Variable(v)
            }
            _ => {
                self.consume_to_eol();
                eprintln!(
                    "Error: OBJECT: expected an object variable on line {}",
                    line
                );
                self.error_count += 1;
                return Ok(Statement::Noop("OBJECT".to_string(), line));
            }
        };
        // Dot + member identifier, repeated for each segment of the chain
        // (compound references: `o.A.B`, `o.Range(x).Value`, ...).  Each
        // segment may carry its own optional argument list.
        let mut members: Vec<ObjectMember> = Vec::new();
        while self.peek() == &Token::Dot {
            self.advance();
            let Some(name) = self.peek_identifier() else {
                self.consume_to_eol();
                eprintln!("Error: OBJECT: expected a member name on line {}", line);
                self.error_count += 1;
                return Ok(Statement::Noop("OBJECT".to_string(), line));
            };
            let _ = name;
            let name = self.consume_identifier()?;
            let mut margs: Vec<Expr> = Vec::new();
            if self.peek() == &Token::LParen {
                self.advance();
                while self.peek() != &Token::RParen && !self.at_eol_or_eof() {
                    let arg = self.parse_expression()?;
                    margs.push(arg);
                    if self.peek() == &Token::Comma {
                        self.advance();
                    } else {
                        break;
                    }
                }
                if self.peek() == &Token::RParen {
                    self.advance();
                }
            }
            members.push(ObjectMember { name, args: margs });
        }
        let mut result: Option<String> = None;
        if kind == "LET" || kind == "SET" {
            // = ValueVar — appended to the last member's argument list
            if self.peek() == &Token::Eq {
                self.advance();
                let value = self.parse_expression()?;
                if let Some(last) = members.last_mut() {
                    last.args.push(value);
                }
            }
        } else if self.peek() == &Token::To {
            // TO is the reserved Token::To (not an Identifier)
            self.advance();
            if self.peek_identifier().is_some() {
                result = Some(self.consume_identifier()?);
            }
        }
        self.consume_to_eol();
        Ok(Statement::ObjectCall(ObjectCallStmt {
            object,
            members,
            result,
            kind: kind.to_string(),
            line,
        }))
    }

    /// FIELD #n, size AS var [, size2 AS var2 ...]
    /// FIELD dyn$, size AS var [, FROM nStart TO nEnd AS var ...]
    /// FIELD RESET var [, ...]  /  FIELD STRING var [, ...]
    /// MAT a() = RHS — matrix algebra (batch 32).
    /// RHS: CON | CON(expr) | IDN | ZER | src() | src()+src() | src()-src() |
    ///      src()*src() | (expr)*src() | INV(src()) | TRN(src())
    fn parse_mat_statement(&mut self, line: usize) -> PbResult<Statement> {
        // Destination array: identifier with optional ()
        let dst = match self.peek().clone() {
            Token::Identifier(v) => {
                self.advance();
                v
            }
            _ => {
                self.consume_to_eol();
                eprintln!("Error: MAT: expected an array destination on line {}", line);
                self.error_count += 1;
                return Ok(Statement::Noop("MAT".to_string(), line));
            }
        };
        // optional ()
        if self.peek() == &Token::LParen {
            self.advance();
            if self.peek() == &Token::RParen {
                self.advance();
            } else {
                // DIM-style bounds inside — skip to closing paren
                while !matches!(self.peek(), Token::RParen | Token::Eol | Token::Eof) {
                    self.advance();
                }
                if self.peek() == &Token::RParen {
                    self.advance();
                }
            }
        }
        self.expect(&Token::Eq)?;

        // RHS
        let mut op;
        let mut src1: Option<String> = None;
        let mut src2: Option<String> = None;
        let mut scalar: Option<Expr> = None;

        match self.peek().clone() {
            Token::Identifier(w) => {
                let up = w.to_uppercase();
                self.advance();
                match up.as_str() {
                    "CON" | "IDN" | "ZER" => {
                        // optional (expr) for CON
                        if self.peek() == &Token::LParen {
                            self.advance();
                            scalar = Some(self.parse_expression()?);
                            self.expect(&Token::RParen)?;
                            op = MatOp::ConScalar;
                        } else if up == "CON" {
                            op = MatOp::Con;
                        } else if up == "IDN" {
                            op = MatOp::Idn;
                        } else {
                            op = MatOp::Zer;
                        }
                    }
                    "INV" | "TRN" => {
                        self.expect(&Token::LParen)?;
                        if let Token::Identifier(sv) = self.peek().clone() {
                            self.advance();
                            src1 = Some(sv);
                        }
                        // optional ()
                        if self.peek() == &Token::LParen {
                            self.advance();
                            if self.peek() == &Token::RParen {
                                self.advance();
                            }
                        }
                        self.expect(&Token::RParen)?;
                        op = if up == "INV" { MatOp::Inv } else { MatOp::Trn };
                    }
                    _ => {
                        // bare array or a() op b()
                        src1 = Some(w);
                        op = MatOp::Assign;
                        // optional ()
                        if self.peek() == &Token::LParen {
                            self.advance();
                            if self.peek() == &Token::RParen {
                                self.advance();
                            }
                        }
                        // operator?
                        if let Token::Plus = self.peek() {
                            self.advance();
                            op = MatOp::Add;
                            src2 = self.parse_mat_array_name()?;
                        } else if let Token::Minus = self.peek() {
                            self.advance();
                            op = MatOp::Sub;
                            src2 = self.parse_mat_array_name()?;
                        } else if let Token::Star = self.peek() {
                            self.advance();
                            op = MatOp::Mul;
                            src2 = self.parse_mat_array_name()?;
                        }
                    }
                }
            }
            Token::LParen => {
                // (expr) * array
                self.advance();
                scalar = Some(self.parse_expression()?);
                self.expect(&Token::RParen)?;
                self.expect(&Token::Star)?;
                src1 = self.parse_mat_array_name()?;
                op = MatOp::Scale;
            }
            _ => {
                self.consume_to_eol();
                eprintln!("Error: MAT: unsupported form on line {}", line);
                self.error_count += 1;
                return Ok(Statement::Noop("MAT".to_string(), line));
            }
        }

        self.consume_to_eol();
        Ok(Statement::Mat(MatStmt {
            dst,
            op,
            src1,
            src2,
            scalar,
            line,
        }))
    }

    /// Parse an array name with optional () after a MAT operator.
    fn parse_mat_array_name(&mut self) -> PbResult<Option<String>> {
        let name = match self.peek().clone() {
            Token::Identifier(v) => {
                self.advance();
                v
            }
            _ => return Ok(None),
        };
        if self.peek() == &Token::LParen {
            self.advance();
            if self.peek() == &Token::RParen {
                self.advance();
            }
        }
        Ok(Some(name))
    }

    fn parse_field_statement(&mut self, line: usize) -> PbResult<Statement> {
        // FIELD RESET / FIELD STRING
        // The lexer maps the word STRING to Token::String_, so match through the
        // helper rather than Token::Identifier alone.
        if let Some(w) = self.peek_ident_or_string_keyword() {
            let up = w.to_uppercase();
            if up == "RESET" || up == "STRING" {
                self.advance();
                let kind = if up == "RESET" {
                    FieldKind::Reset
                } else {
                    FieldKind::ToStr
                };
                let mut specs = Vec::new();
                loop {
                    if matches!(self.peek(), Token::Eol | Token::Eof | Token::Colon) {
                        break;
                    }
                    if let Token::Identifier(v) = self.peek().clone() {
                        self.advance();
                        specs.push(FieldSpec {
                            size: 0,
                            name: v,
                            offset: -1,
                        });
                    } else {
                        break;
                    }
                    if self.peek() == &Token::Comma {
                        self.advance();
                    } else {
                        break;
                    }
                }
                self.consume_to_eol();
                return Ok(Statement::Field(FieldStmt {
                    kind,
                    filenum: None,
                    dyn_expr: None,
                    specs,
                    line,
                }));
            }
        }

        // First token after FIELD: #filenum  or  dynamic-string expr
        let mut filenum: Option<Expr> = None;
        let mut dyn_expr: Option<Expr> = None;
        if self.peek() == &Token::Hash {
            self.advance();
            filenum = Some(self.parse_expression()?);
        } else {
            dyn_expr = Some(self.parse_expression()?);
        }

        self.expect(&Token::Comma)?;

        let mut specs = Vec::new();
        loop {
            // Optional FROM nStart TO nEnd
            if let Token::Identifier(w) = self.peek() {
                if w.eq_ignore_ascii_case("FROM") {
                    self.advance();
                    if let Token::IntegerLiteral(n) = self.peek().clone() {
                        self.advance();
                        // optional TO nEnd
                        if let Token::Identifier(w2) = self.peek() {
                            if w2.eq_ignore_ascii_case("TO") {
                                self.advance();
                                if let Token::IntegerLiteral(e) = self.peek().clone() {
                                    self.advance();
                                    specs.push(FieldSpec {
                                        size: e - n + 1,
                                        name: String::new(), // filled below
                                        offset: n - 1,
                                    });
                                }
                            }
                        }
                    }
                    // FROM form still requires "AS var" next
                }
            }
            // nSize
            let size = match self.peek().clone() {
                Token::IntegerLiteral(n) => {
                    self.advance();
                    n
                }
                _ => 0,
            };
            self.expect(&Token::As)?;
            let name = match self.peek().clone() {
                Token::Identifier(v) => {
                    self.advance();
                    v
                }
                _ => String::new(),
            };
            // Merge with pending FROM-spec (if any)
            if let Some(last) = specs.last_mut() {
                if last.name.is_empty() && last.size > 0 {
                    last.name = name;
                } else {
                    specs.push(FieldSpec {
                        size,
                        name,
                        offset: -1,
                    });
                }
            } else {
                specs.push(FieldSpec {
                    size,
                    name,
                    offset: -1,
                });
            }
            if self.peek() == &Token::Comma {
                self.advance();
            } else {
                break;
            }
        }
        self.consume_to_eol();
        Ok(Statement::Field(FieldStmt {
            kind: if filenum.is_some() {
                FieldKind::File
            } else {
                FieldKind::Str
            },
            filenum,
            dyn_expr,
            specs,
            line,
        }))
    }

    // ===== Expression parsing with precedence climbing =====

    fn parse_expression(&mut self) -> PbResult<Expr> {
        self.parse_imp_expr()
    }

    fn parse_imp_expr(&mut self) -> PbResult<Expr> {
        let mut left = self.parse_eqv_expr()?;
        while self.peek() == &Token::Imp {
            self.advance();
            let right = self.parse_eqv_expr()?;
            left = Expr::BinaryOp(BinaryOp::Imp, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_eqv_expr(&mut self) -> PbResult<Expr> {
        let mut left = self.parse_or_expr()?;
        while self.peek() == &Token::Eqv {
            self.advance();
            let right = self.parse_or_expr()?;
            left = Expr::BinaryOp(BinaryOp::Eqv, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_or_expr(&mut self) -> PbResult<Expr> {
        let mut left = self.parse_xor_expr()?;
        while self.peek() == &Token::Or {
            self.advance();
            let right = self.parse_xor_expr()?;
            left = Expr::BinaryOp(BinaryOp::Or, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_xor_expr(&mut self) -> PbResult<Expr> {
        let mut left = self.parse_and_expr()?;
        while self.peek() == &Token::Xor {
            self.advance();
            let right = self.parse_and_expr()?;
            left = Expr::BinaryOp(BinaryOp::Xor, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_and_expr(&mut self) -> PbResult<Expr> {
        let mut left = self.parse_not_expr()?;
        while self.peek() == &Token::And {
            self.advance();
            let right = self.parse_not_expr()?;
            left = Expr::BinaryOp(BinaryOp::And, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_not_expr(&mut self) -> PbResult<Expr> {
        if self.peek() == &Token::Not {
            self.advance();
            let expr = self.parse_comparison()?;
            return Ok(Expr::UnaryOp(UnaryOp::Not, Box::new(expr)));
        }
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> PbResult<Expr> {
        let mut left = self.parse_concat_expr()?;
        loop {
            let op = match self.peek() {
                Token::Eq => {
                    // PB allows =< and => as alternatives to <= and >=
                    match self.peek_at(1) {
                        Some(Token::Lt) => {
                            self.advance(); // consume =
                            BinaryOp::Lte
                        }
                        Some(Token::Gt) => {
                            self.advance(); // consume =
                            BinaryOp::Gte
                        }
                        _ => BinaryOp::Eq,
                    }
                }
                Token::Neq => BinaryOp::Neq,
                Token::Lt => BinaryOp::Lt,
                Token::Gt => BinaryOp::Gt,
                Token::Lte => BinaryOp::Lte,
                Token::Gte => BinaryOp::Gte,
                _ => break,
            };
            self.advance();
            let right = self.parse_concat_expr()?;
            left = Expr::BinaryOp(op, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_concat_expr(&mut self) -> PbResult<Expr> {
        let mut left = self.parse_add_sub()?;
        while self.peek() == &Token::Ampersand {
            self.advance();
            let right = self.parse_add_sub()?;
            left = Expr::BinaryOp(BinaryOp::StrConcat, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_add_sub(&mut self) -> PbResult<Expr> {
        let mut left = self.parse_mul_div()?;
        loop {
            let op = match self.peek() {
                Token::Plus => BinaryOp::Add,
                Token::Minus => BinaryOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_mul_div()?;
            left = Expr::BinaryOp(op, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_mul_div(&mut self) -> PbResult<Expr> {
        let mut left = self.parse_int_div()?;
        loop {
            let op = match self.peek() {
                Token::Star => BinaryOp::Mul,
                Token::Slash => BinaryOp::Div,
                _ => break,
            };
            self.advance();
            let right = self.parse_int_div()?;
            left = Expr::BinaryOp(op, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_int_div(&mut self) -> PbResult<Expr> {
        let mut left = self.parse_mod_expr()?;
        while self.peek() == &Token::Backslash {
            self.advance();
            let right = self.parse_mod_expr()?;
            left = Expr::BinaryOp(BinaryOp::IntDiv, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_mod_expr(&mut self) -> PbResult<Expr> {
        let mut left = self.parse_power()?;
        while self.peek() == &Token::Mod {
            self.advance();
            let right = self.parse_power()?;
            left = Expr::BinaryOp(BinaryOp::Mod, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_power(&mut self) -> PbResult<Expr> {
        let base = self.parse_unary()?;
        if self.peek() == &Token::Caret {
            self.advance();
            let exp = self.parse_unary()?;
            return Ok(Expr::BinaryOp(BinaryOp::Pow, Box::new(base), Box::new(exp)));
        }
        Ok(base)
    }

    fn parse_unary(&mut self) -> PbResult<Expr> {
        if self.peek() == &Token::Minus {
            self.advance();
            let expr = self.parse_primary()?;
            return Ok(Expr::Negate(Box::new(expr)));
        }
        if self.peek() == &Token::Plus {
            self.advance();
            return self.parse_primary();
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> PbResult<Expr> {
        // CB.MSG / CB.HNDL / CB.CTL / CB.CTLMSG / CB.WPARAM / CB.LPARAM
        if let Token::Identifier(w) = self.peek() {
            if w.to_uppercase() == "CB" && self.peek_at(1) == Some(&Token::Dot) {
                if let Some(Token::Identifier(field)) = self.peek_at(2) {
                    let fn_name = match field.to_uppercase().as_str() {
                        "MSG" => "PB_GET_CB_MSG",
                        "HNDL" => "PB_GET_CB_HNDL",
                        "CTL" => "PB_GET_CB_CTL",
                        "CTLMSG" => "PB_GET_CB_CTLMSG",
                        "WPARAM" => "PB_GET_CB_WPARAM",
                        "LPARAM" => "PB_GET_CB_LPARAM",
                        "NMCODE" => "PB_GET_CB_NMCODE",
                        "NMHDR" => "PB_GET_CB_NMHDR",
                        "NMHDR$" => "PB_GET_CB_NMHDR_STR",
                        "NMHWND" => "PB_GET_CB_NMHWND",
                        "NMID" => "PB_GET_CB_NMID",
                        _ => "",
                    };
                    if !fn_name.is_empty() {
                        self.advance();
                        self.advance();
                        self.advance();
                        // optional parenthesised args, e.g. CB.NMHDR$(size)
                        let mut args = Vec::new();
                        if self.peek() == &Token::LParen {
                            self.advance();
                            if self.peek() != &Token::RParen {
                                args.push(self.parse_expression()?);
                                while self.peek() == &Token::Comma {
                                    self.advance();
                                    args.push(self.parse_expression()?);
                                }
                            }
                            self.expect(&Token::RParen)?;
                        }
                        return Ok(Expr::FunctionCall(fn_name.to_string(), args));
                    }
                }
            }
        }
        // METRICS(Scroll.Horz) - the official metric names are dotted identifiers
        // (Frame.Fixed.X has three segments).  Fold them to integer literals so the
        // argument parses at all; a plain numeric expression is accepted as well.
        if let Token::Identifier(w) = self.peek() {
            let mut parts = vec![w.clone()];
            let mut used = 1usize;
            while parts.len() < 3 && self.peek_at(used) == Some(&Token::Dot) {
                match self.peek_at(used + 1) {
                    Some(Token::Identifier(next)) => {
                        parts.push(next.clone());
                        used += 2;
                    }
                    _ => break,
                }
            }
            if parts.len() >= 2 {
                if let Some(idx) = metric_index(&parts) {
                    for _ in 0..used {
                        self.advance();
                    }
                    return Ok(Expr::IntegerLit(idx));
                }
            }
        }
        match self.peek().clone() {
            Token::IntegerLiteral(n) => {
                self.advance();
                Ok(Expr::IntegerLit(n))
            }
            Token::FloatLiteral(f) => {
                self.advance();
                Ok(Expr::FloatLit(f))
            }
            Token::StringLiteral(s) => {
                self.advance();
                Ok(Expr::StringLit(s))
            }
            Token::WideStringLiteral(s) => {
                self.advance();
                Ok(Expr::WideStringLit(s))
            }
            Token::PercentConstant(name) => {
                self.advance();
                Ok(Expr::PercentConst(name))
            }
            Token::FreeFile => {
                self.advance();
                Ok(Expr::FunctionCall("FREEFILE".to_string(), Vec::new()))
            }
            Token::Ubound | Token::Lbound => {
                let name = if matches!(self.peek(), Token::Ubound) {
                    "UBOUND"
                } else {
                    "LBOUND"
                };
                let name = name.to_string();
                self.advance();
                if self.peek() == &Token::LParen {
                    self.advance();
                    let args = self.parse_arg_list()?;
                    self.expect(&Token::RParen)?;
                    Ok(Expr::FunctionCall(name, args))
                } else {
                    Ok(Expr::FunctionCall(name, Vec::new()))
                }
            }
            Token::LParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(&Token::RParen)?;
                Ok(expr)
            }
            Token::Identifier(name) if name.eq_ignore_ascii_case("DIR$") => {
                // DIR$(NEXT) or DIR$(mask [, ONLY attr]) — NEXT is a keyword token
                self.advance();
                self.expect(&Token::LParen)?;
                let mut args = Vec::new();
                if self.peek() == &Token::Next {
                    self.advance();
                    args.push(Expr::StringLit("\u{0}DIRNEXT".to_string())); // sentinel
                } else if self.peek() != &Token::RParen {
                    args = self.parse_arg_list()?;
                }
                self.expect(&Token::RParen)?;
                Ok(Expr::FunctionCall("DIR$".to_string(), args))
            }
            Token::Identifier(name) if name.eq_ignore_ascii_case("CLASS") => {
                // CLASS "MyClass" — object instantiation expression (CHW
                // How_do_you_create_an_object_: `LET objvar = CLASS "MyClass"`,
                // LET optional).  The class name must be a string literal.
                self.advance();
                if let Token::StringLiteral(cls) = self.peek().clone() {
                    self.advance();
                    Ok(Expr::ClassNew(cls))
                } else {
                    Err(PbError::parser(
                        "CLASS requires a string class name: CLASS \"MyClass\"".to_string(),
                        None,
                        self.current_line(),
                    ))
                }
            }
            Token::Identifier(name)
                if name.eq_ignore_ascii_case("NEWCOM")
                    || name.eq_ignore_ascii_case("GETCOM")
                    || name.eq_ignore_ascii_case("ANYCOM") =>
            {
                // NEWCOM / GETCOM / ANYCOM — external COM object instantiation
                // (CHW How_do_you_create_an_object_):
                //   NEWCOM "ProgID"
                //   NEWCOM CLSID ClassID$            (16-byte GUID, no ProgID)
                //   NEWCOM CLSID ClassID$ LIB DllPath$  (registry-free load)
                // GETCOM opens an existing running instance (GetActiveObject);
                // ANYCOM prefers an existing instance, else creates a new one.
                // All IDs / paths are string literals (PBWin requires them).
                let mode = if name.eq_ignore_ascii_case("GETCOM") {
                    1
                } else if name.eq_ignore_ascii_case("ANYCOM") {
                    2
                } else {
                    0
                };
                self.advance();
                let mut clsid: Option<Box<Expr>> = None;
                let mut lib: Option<Box<Expr>> = None;
                let progid: String;
                if let Token::StringLiteral(s) = self.peek().clone() {
                    self.advance();
                    progid = s;
                } else if let Token::Identifier(kw) = self.peek().clone() {
                    if kw.eq_ignore_ascii_case("CLSID") {
                        self.advance();
                        // ClassID$ is an expression (a 16-byte binary GUID,
                        // e.g. CLSID$("ProgID")); the parser accepts any
                        // expression so string variables work exactly as CHW.
                        clsid = Some(Box::new(self.parse_expression()?));
                        if let Token::Identifier(libkw) = self.peek().clone() {
                            if libkw.eq_ignore_ascii_case("LIB") {
                                self.advance();
                                lib = Some(Box::new(self.parse_expression()?));
                            }
                        }
                        progid = String::new();
                    } else {
                        return Err(PbError::parser(
                            "NEWCOM/GETCOM/ANYCOM requires a ProgID string: NEWCOM \"ProgID\""
                                .to_string(),
                            None,
                            self.current_line(),
                        ));
                    }
                } else {
                    return Err(PbError::parser(
                        "NEWCOM/GETCOM/ANYCOM requires a ProgID string: NEWCOM \"ProgID\""
                            .to_string(),
                        None,
                        self.current_line(),
                    ));
                }
                Ok(Expr::ComNew {
                    mode,
                    progid,
                    clsid,
                    lib,
                })
            }
            Token::Identifier(name) if name.eq_ignore_ascii_case("COMM") => {
                // COMM([#] hComm, Comfunc) - CHW comm_function.htm.  The #
                // before hComm is optional; Comfunc is a keyword (BAUD/BYTE/
                // PARITY/...), taken as a literal string so codegen can hand
                // it to the runtime query function.
                self.advance();
                self.expect(&Token::LParen)?;
                let mut args = Vec::new();
                if self.peek() == &Token::Hash {
                    self.advance();
                }
                args.push(self.parse_expression()?);
                while self.peek() == &Token::Comma {
                    self.advance();
                    if let Token::Identifier(ref w) = self.peek().clone() {
                        args.push(Expr::StringLit(w.to_uppercase()));
                        self.advance();
                    } else {
                        args.push(self.parse_expression()?);
                    }
                }
                self.expect(&Token::RParen)?;
                Ok(Expr::FunctionCall("COMM".to_string(), args))
            }
            Token::Identifier(name) if name.eq_ignore_ascii_case("FILEATTR") => {
                // FILEATTR([#] filenum&, fattr) — optional # before the file number.
                self.advance();
                self.expect(&Token::LParen)?;
                let mut args = Vec::new();
                if self.peek() == &Token::Hash {
                    self.advance();
                }
                args.push(self.parse_expression()?);
                while self.peek() == &Token::Comma {
                    self.advance();
                    args.push(self.parse_expression()?);
                }
                self.expect(&Token::RParen)?;
                Ok(Expr::FunctionCall("FILEATTR".to_string(), args))
            }
            Token::Identifier(name)
                if name.eq_ignore_ascii_case("BITS") || name.eq_ignore_ascii_case("BITS$") =>
            {
                // BITS(datatype, expr) / BITS$(director, s$): the datatype /
                // director word is a compile-time keyword (STRING/WSTRING are
                // keyword/Identifier tokens, BYTE/WORD are Identifiers,
                // DWORD/INTEGER/LONG/QUAD are keyword tokens).  It is mapped to
                // an internal type code consumed by codegen, and the function
                // name keeps its original spelling so BITS and BITS$ never
                // collide.  (batch 256B: the old branch only matched
                // STRING/WSTRING/Identifiers, so BITS(DWORD,...) failed
                // "Expected RParen, got Dword", and every call was mislabeled
                // "BITS$".)
                let is_bits_str = name.eq_ignore_ascii_case("BITS$");
                self.advance();
                self.expect(&Token::LParen)?;
                let mut args = Vec::new();
                let dir_code = match self.peek().clone() {
                    Token::String_ => Some(23), // STRING
                    Token::Long => Some(4),
                    Token::Integer => Some(3),
                    Token::Dword => Some(2),
                    Token::Quad => Some(5),
                    Token::Identifier(w) => {
                        let u = w.to_uppercase();
                        match u.as_str() {
                            "WSTRING" => Some(27),
                            "BYTE" => Some(0),
                            "WORD" => Some(1),
                            _ => None,
                        }
                    }
                    _ => None,
                };
                if let Some(code) = dir_code {
                    self.advance();
                    args.push(Expr::IntegerLit(code));
                    if self.peek() == &Token::Comma {
                        self.advance();
                    }
                    args.extend(self.parse_arg_list()?);
                } else if self.peek() != &Token::RParen {
                    args = self.parse_arg_list()?;
                }
                self.expect(&Token::RParen)?;
                let fname = if is_bits_str { "BITS$" } else { "BITS" };
                Ok(Expr::FunctionCall(fname.to_string(), args))
            }
            Token::Identifier(name)
                if name.eq_ignore_ascii_case("EXTRACT")
                    || name.eq_ignore_ascii_case("EXTRACT$") =>
            {
                // EXTRACT$([start,] MainStr, [ANY] MatchStr) — ANY is an Identifier
                // token here; convert it to a sentinel string so codegen can reorder.
                self.advance();
                self.expect(&Token::LParen)?;
                let mut args = Vec::new();
                loop {
                    if let Token::Identifier(ref w) = self.peek().clone() {
                        if w.eq_ignore_ascii_case("ANY") {
                            args.push(Expr::StringLit("__ANY__".to_string()));
                            self.advance();
                            // batch 234: official EXTRACT$(MainStr, ANY MatchStr)
                            // has NO comma between ANY and the match string.
                            if self.peek() != &Token::RParen
                                && self.peek() != &Token::Comma
                                && !matches!(self.peek(), Token::Eol | Token::Eof)
                            {
                                args.push(self.parse_expression()?);
                            }
                        } else {
                            args.push(self.parse_expression()?);
                        }
                    } else {
                        args.push(self.parse_expression()?);
                    }
                    if self.peek() != &Token::Comma {
                        break;
                    }
                    self.advance();
                }
                self.expect(&Token::RParen)?;
                Ok(Expr::FunctionCall("EXTRACT".to_string(), args))
            }
            Token::Identifier(name)
                if name.eq_ignore_ascii_case("PATHSCAN")
                    || name.eq_ignore_ascii_case("PATHSCAN$") =>
            {
                // PATHSCAN$(director, filespec$ [, pathspec$]) — director words are
                // Identifier tokens (FULL/PATH/NAME/EXTN/NAMEX); convert to strings.
                self.advance();
                self.expect(&Token::LParen)?;
                let mut args = Vec::new();
                if let Token::Identifier(ref w) = self.peek().clone() {
                    args.push(Expr::StringLit(w.to_uppercase()));
                    self.advance();
                } else {
                    args.push(self.parse_expression()?);
                }
                while self.peek() == &Token::Comma {
                    self.advance();
                    args.push(self.parse_expression()?);
                }
                self.expect(&Token::RParen)?;
                Ok(Expr::FunctionCall("PATHSCAN".to_string(), args))
            }
            Token::Identifier(name)
                if name.eq_ignore_ascii_case("HI")
                    || name.eq_ignore_ascii_case("HI$")
                    || name.eq_ignore_ascii_case("LO")
                    || name.eq_ignore_ascii_case("LO$") =>
            {
                // HI(DataType, value) / LO(DataType, value) — DataType is a type keyword
                // (BYTE/WORD are Identifiers; INTEGER/DWORD/LONG are keyword tokens).
                self.advance();
                self.expect(&Token::LParen)?;
                let dtype = match self.peek().clone() {
                    Token::Integer => "INTEGER".to_string(),
                    Token::Dword => "DWORD".to_string(),
                    Token::Long => "LONG".to_string(),
                    Token::String_ => "STRING".to_string(),
                    Token::Identifier(ref w) => w.to_uppercase(),
                    _ => {
                        return Err(PbError::parser(
                            format!("HI/LO: expected a data type, got {:?}", self.peek()),
                            self.peek_located().file.as_deref(),
                            self.current_line(),
                        ))
                    }
                };
                self.advance();
                self.expect(&Token::Comma)?;
                let val = self.parse_expression()?;
                self.expect(&Token::RParen)?;
                Ok(Expr::FunctionCall(
                    name.to_uppercase().trim_end_matches('$').to_string(),
                    vec![Expr::StringLit(dtype), val],
                ))
            }
            Token::Identifier(name)
                if name.eq_ignore_ascii_case("PATHNAME")
                    || name.eq_ignore_ascii_case("PATHNAME$") =>
            {
                // PATHNAME$(FULL|PATH|NAME|EXTN|NAMEX, filespec$) — director words are Identifier tokens
                self.advance();
                self.expect(&Token::LParen)?;
                let mut args = Vec::new();
                if let Token::Identifier(ref w) = self.peek().clone() {
                    let u = w.to_uppercase();
                    if matches!(u.as_str(), "FULL" | "PATH" | "NAME" | "EXTN" | "NAMEX") {
                        self.advance();
                        args.push(Expr::StringLit(u));
                        if self.peek() == &Token::Comma {
                            self.advance();
                        }
                        args.extend(self.parse_arg_list()?);
                    } else if self.peek() != &Token::RParen {
                        args = self.parse_arg_list()?;
                    }
                }
                self.expect(&Token::RParen)?;
                Ok(Expr::FunctionCall("PATHNAME$".to_string(), args))
            }
            Token::Identifier(name)
                if name.eq_ignore_ascii_case("CLIP") || name.eq_ignore_ascii_case("CLIP$") =>
            {
                // CLIP$(LEFT str, n) / CLIP$(RIGHT str, n) / CLIP$(MID str, start, n)
                // LEFT/RIGHT/MID are keyword tokens — map them to string modes.
                self.advance();
                self.expect(&Token::LParen)?;
                let mut args = Vec::new();
                let mode = match self.peek() {
                    Token::Identifier(ref w) if w.eq_ignore_ascii_case("LEFT") => {
                        self.advance();
                        "LEFT"
                    }
                    Token::Identifier(ref w) if w.eq_ignore_ascii_case("RIGHT") => {
                        self.advance();
                        "RIGHT"
                    }
                    Token::Identifier(ref w) if w.eq_ignore_ascii_case("MID") => {
                        self.advance();
                        "MID"
                    }
                    _ => "",
                };
                if !mode.is_empty() {
                    args.push(Expr::StringLit(mode.to_string()));
                    if self.peek() == &Token::Comma {
                        self.advance();
                    }
                    args.extend(self.parse_arg_list()?);
                } else if self.peek() != &Token::RParen {
                    args = self.parse_arg_list()?;
                }
                self.expect(&Token::RParen)?;
                Ok(Expr::FunctionCall("CLIP".to_string(), args))
            }
            Token::Identifier(name) if name.eq_ignore_ascii_case("VARPTR") => {
                self.advance();
                self.expect(&Token::LParen)?;
                let inner = self.parse_expression()?;
                self.expect(&Token::RParen)?;
                Ok(Expr::Varptr(Box::new(inner)))
            }
            Token::Identifier(name)
                if (name.eq_ignore_ascii_case("TAB") || name.eq_ignore_ascii_case("TAB$"))
                    && matches!(self.peek_at(1), Some(Token::LParen))
                    && (matches!(
                        self.peek_at(2),
                        Some(Token::Identifier(ref w))
                            if matches!(w.to_uppercase().as_str(),
                                        "COUNT" | "DIALOG" | "IMAGE" | "PAGE" | "TEXT")
                    ) || matches!(self.peek_at(2), Some(Token::Select))) =>
            {
                // Official TAB Function Form (CHW TAB_statement.htm):
                //   TAB(COUNT|DIALOG|IMAGE|PAGE|SELECT, ...) / TAB$(TEXT, ...).
                // Director word is the first arg; COUNT/DIALOG/IMAGE/PAGE/TEXT
                // are Identifier tokens, SELECT is a reserved token.  Non-director
                // TAB(x) (a user function/array) falls through to the general
                // Identifier arm below.
                self.advance(); // TAB
                self.advance(); // (
                let mut args = Vec::new();
                args.push(Expr::StringLit(self.peek_word_upper()));
                self.advance(); // director word
                while self.peek() == &Token::Comma {
                    self.advance();
                    args.push(self.parse_expression()?);
                }
                self.expect(&Token::RParen)?;
                Ok(Expr::FunctionCall("TAB".to_string(), args))
            }
            Token::Identifier(name)
                if {
                    let n = name.trim_end_matches(['&', '$']).to_uppercase();
                    n == "CHOOSE"
                } =>
            {
                // CHOOSE[/&/$]([BIT|BITS] index, choice1[, choice2]... [ELSE default])
                // batch258: encode mode/default into the callee name so codegen
                // can emit a lazy switch (only the selected choice is evaluated).
                self.advance();
                self.expect(&Token::LParen)?;
                let mut mode = String::new();
                if let Token::Identifier(w) = self.peek().clone() {
                    if w.eq_ignore_ascii_case("BIT") {
                        mode.push_str(":BIT");
                        self.advance();
                    } else if w.eq_ignore_ascii_case("BITS") {
                        mode.push_str(":BITS");
                        self.advance();
                    }
                }
                let mut args = Vec::new();
                if self.peek() != &Token::RParen {
                    args.push(self.parse_expression()?);
                    while self.peek() == &Token::Comma {
                        self.advance();
                        args.push(self.parse_expression()?);
                    }
                }
                if matches!(self.peek(), Token::Else) {
                    mode.push_str(":ELSE");
                    self.advance();
                    args.push(self.parse_expression()?);
                }
                self.expect(&Token::RParen)?;
                Ok(Expr::FunctionCall(format!("CHOOSE{}", mode), args))
            }
            Token::Identifier(name) => {
                // DEF fn inline expansion: name(params) -> subst body
                let def_key = name.to_uppercase();
                if self.def_funcs.contains_key(&def_key) && self.peek_at(1) == Some(&Token::LParen)
                {
                    self.advance(); // consume name
                    self.advance(); // consume (
                    let args = if self.peek() == &Token::RParen {
                        Vec::new()
                    } else {
                        self.parse_arg_list()?
                    };
                    self.expect(&Token::RParen)?;
                    let (params, body) = self.def_funcs.get(&def_key).unwrap().clone();
                    return Ok(subst_def_params(&body, &params, &args));
                }
                self.advance();
                // Check for function call or array access
                let mut expr = if self.peek() == &Token::LParen {
                    self.advance();
                    // CHR$(a TO b) — range form returns CHR$(a)+CHR$(a+1)+..+CHR$(b)
                    // (CHW CHR$_function.htm).  Encode as CHR$RANGE so codegen
                    // can emit a loop; the plain multi-arg path stays unchanged.
                    if name.eq_ignore_ascii_case("CHR$") && self.peek() != &Token::RParen {
                        let first = self.parse_expression()?;
                        if self.peek() == &Token::To {
                            self.advance();
                            let last = self.parse_expression()?;
                            self.expect(&Token::RParen)?;
                            Expr::FunctionCall("CHR$RANGE".to_string(), vec![first, last])
                        } else {
                            let mut args = vec![first];
                            while self.peek() == &Token::Comma {
                                self.advance();
                                args.push(self.parse_expression()?);
                            }
                            self.expect(&Token::RParen)?;
                            Expr::FunctionCall(name, args)
                        }
                    } else {
                        let args = if self.peek() == &Token::RParen {
                            Vec::new()
                        } else {
                            self.parse_arg_list()?
                        };
                        self.expect(&Token::RParen)?;
                        // Determine if this is a function call or array access
                        // We can't tell at parse time — interpreter will resolve
                        // METRICS also accepts the single-word metric names
                        // (Caption, Menubar) as bare identifiers; fold those to
                        // their SM_ index here, where the callee name is known.
                        let args = if name.eq_ignore_ascii_case("METRICS") {
                            args.into_iter().map(fold_metric_name).collect()
                        } else {
                            args
                        };
                        Expr::FunctionCall(name, args)
                    }
                } else if name.eq_ignore_ascii_case("DATACOUNT")
                    || name.eq_ignore_ascii_case("THREADCOUNT")
                    || name.eq_ignore_ascii_case("PRINTERCOUNT")
                    || name.eq_ignore_ascii_case("COMMAND")
                    || name.eq_ignore_ascii_case("COMMAND$")
                    || name.eq_ignore_ascii_case("ERL")
                    || name.eq_ignore_ascii_case("ERL$")
                    || name.eq_ignore_ascii_case("ERROR")
                    || name.eq_ignore_ascii_case("ERROR$")
                    || name.eq_ignore_ascii_case("FRE")
                    || name.eq_ignore_ascii_case("THREADID")
                    || name.eq_ignore_ascii_case("FUNCNAME")
                    || name.eq_ignore_ascii_case("FUNCNAME$")
                    // batch271: OBJRESULT must be called WITHOUT parentheses in
                    // PBWin (Error 477 with parens), so the bare form is the
                    // official one.
                    || name.eq_ignore_ascii_case("OBJRESULT")
                {
                    // No-argument functions without parentheses (PB syntax: n = DATACOUNT)
                    Expr::FunctionCall(name.to_uppercase(), Vec::new())
                } else {
                    Expr::Variable(name)
                };
                // Chained .member access: obj.field1.field2..., and the dotted
                // method call `obj.M(args)` (batch 217).  The receiver is kept in
                // the AST: codegen is the layer that knows the receiver's TYPE and
                // can therefore lower `obj.M(a)` to `<Class>_<Method>(obj, a)`.
                while self.peek() == &Token::Dot {
                    self.advance();
                    // Member name may be a reserved word used as a TYPE member
                    // (batch 278, official MyDLL.bas: BYREF / DWORD members).
                    let member = if let Token::Identifier(w) = self.peek() {
                        let w = w.clone();
                        self.advance();
                        w
                    } else if let Some(spelling) = Self::keyword_field_spelling(self.peek()) {
                        self.advance();
                        spelling
                    } else {
                        return Err(PbError::parser(
                            "Expected member name after '.'",
                            self.current_file(),
                            self.current_line(),
                        ));
                    };
                    if self.peek() == &Token::LParen {
                        self.advance();
                        let args = if self.peek() == &Token::RParen {
                            Vec::new()
                        } else {
                            self.parse_arg_list()?
                        };
                        self.expect(&Token::RParen)?;
                        expr = Expr::MethodCall {
                            base: Box::new(expr),
                            name: member,
                            args,
                        };
                    } else {
                        expr = Expr::TypeMember(Box::new(expr), member);
                    }
                }
                Ok(expr)
            }
            _ => Err(PbError::parser(
                format!("Unexpected token in expression: {:?}", self.peek()),
                self.current_file(),
                self.current_line(),
            )),
        }
    }

    fn parse_arg_list(&mut self) -> PbResult<Vec<Expr>> {
        let mut args = Vec::new();
        loop {
            if self.peek() == &Token::RParen || self.at_end() {
                break;
            }
            // PEEK/POKE datatype keyword as first argument: PEEK(LONG, addr)
            if args.is_empty() {
                let dt = match self.peek() {
                    Token::Long => Some("LONG"),
                    Token::Double => Some("DOUBLE"),
                    Token::Dword => Some("DWORD"),
                    Token::Integer => Some("INTEGER"),
                    Token::Quad => Some("QUAD"),
                    Token::Single => Some("SINGLE"),
                    _ => None,
                };
                if let Some(d) = dt {
                    self.advance();
                    args.push(Expr::Variable(d.to_string()));
                    if self.peek() == &Token::Comma {
                        self.advance();
                    }
                    continue;
                }
            }
            // ANY modifier (REMOVE$/RETAIN$/REMAIN$): REMOVE$(s, ANY "lo")
            // Convert the keyword to a sentinel string so codegen can reorder the
            // arguments; EXTRACT$ uses the same sentinel.
            if matches!(self.peek(), Token::Identifier(ref s) if s.eq_ignore_ascii_case("ANY"))
                && matches!(self.peek_at(1), Some(Token::StringLiteral(_)))
            {
                self.advance();
                args.push(Expr::StringLit("__ANY__".to_string()));
            }
            // BYVAL/BYCOPY modifier at call site: Foo(BYVAL x) or Foo(BYCOPY s$)
            // Both override BYREF parameter to pass value directly
            let mut has_byval = matches!(self.peek(), Token::Byval);
            if has_byval {
                self.advance(); // consume BYVAL
            }
            if matches!(self.peek(), Token::Identifier(ref s) if s.eq_ignore_ascii_case("BYCOPY")) {
                has_byval = true;
                self.advance(); // consume BYCOPY
            }
            let expr = self.parse_expression()?;
            args.push(if has_byval {
                Expr::ByvalOverride(Box::new(expr))
            } else {
                expr
            });
            if self.peek() == &Token::Comma {
                self.advance();
            } else if matches!(self.peek(), Token::Identifier(ref s) if s.eq_ignore_ascii_case("USING"))
            {
                // CSET$(s$, len& USING pad$) / LSET$/RSET$ — USING acts as a
                // parameter separator (CHW syntax), producing the next argument.
                self.advance();
            } else {
                break;
            }
        }
        Ok(args)
    }
}

/// Infer PB type from variable name suffix
fn type_from_suffix(name: &str) -> PbType {
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
    } else if name.ends_with("@@") || name.ends_with('@') {
        PbType::Cur
    } else if name.ends_with('$') {
        PbType::String
    } else {
        PbType::Long // PB default numeric type
    }
}

fn has_type_suffix(name: &str) -> bool {
    name.ends_with("##")
        || name.ends_with('#')
        || name.ends_with("&&")
        || name.ends_with('&')
        || name.ends_with("%%")
        || name.ends_with('%')
        || name.ends_with('!')
        || name.ends_with("@@")
        || name.ends_with('@')
        || name.ends_with('$')
}

fn subst_def_params(e: &Expr, params: &[String], args: &[Expr]) -> Expr {
    match e {
        Expr::Variable(n) => {
            if let Some(i) = params.iter().position(|p| p.eq_ignore_ascii_case(n)) {
                args[i].clone()
            } else {
                Expr::Variable(n.clone())
            }
        }
        Expr::UnaryOp(op, x) => {
            Expr::UnaryOp(op.clone(), Box::new(subst_def_params(x, params, args)))
        }
        Expr::BinaryOp(op, a, b) => Expr::BinaryOp(
            op.clone(),
            Box::new(subst_def_params(a, params, args)),
            Box::new(subst_def_params(b, params, args)),
        ),
        Expr::FunctionCall(n, xs) => Expr::FunctionCall(
            n.clone(),
            xs.iter()
                .map(|x| subst_def_params(x, params, args))
                .collect(),
        ),
        Expr::ArrayAccess(n, xs) => Expr::ArrayAccess(
            n.clone(),
            xs.iter()
                .map(|x| subst_def_params(x, params, args))
                .collect(),
        ),
        Expr::TypeMember(x, m) => {
            Expr::TypeMember(Box::new(subst_def_params(x, params, args)), m.clone())
        }
        Expr::Negate(x) => Expr::Negate(Box::new(subst_def_params(x, params, args))),
        Expr::Varptr(x) => Expr::Varptr(Box::new(subst_def_params(x, params, args))),
        Expr::ByvalOverride(x) => Expr::ByvalOverride(Box::new(subst_def_params(x, params, args))),
        _ => e.clone(),
    }
}
/// Win32 `GetSystemMetrics` index for an official PowerBASIC `METRICS` metric
/// name.  Dotted names arrive as several segments (`Scroll.Horz`,
/// `Frame.Fixed.X`); the two single-word names (`Caption`, `Menubar`) arrive as
/// one.  Every index below was read out of the Windows SDK `WinUser.h`
/// (10.0.26100.0), including the aliases SM_CXFIXEDFRAME -> SM_CXDLGFRAME and
/// SM_CXSIZEFRAME -> SM_CXFRAME.
fn metric_index(parts: &[String]) -> Option<i64> {
    let key = parts
        .iter()
        .map(|p| p.to_ascii_uppercase())
        .collect::<Vec<_>>()
        .join(".");
    Some(match key.as_str() {
        "BORDER.X" => 5,        // SM_CXBORDER
        "BORDER.Y" => 6,        // SM_CYBORDER
        "CAPTION" => 4,         // SM_CYCAPTION
        "EDGE.X" => 45,         // SM_CXEDGE
        "EDGE.Y" => 46,         // SM_CYEDGE
        "FRAME.FIXED.X" => 7,   // SM_CXFIXEDFRAME = SM_CXDLGFRAME
        "FRAME.FIXED.Y" => 8,   // SM_CYFIXEDFRAME = SM_CYDLGFRAME
        "FRAME.RESIZE.X" => 32, // SM_CXSIZEFRAME = SM_CXFRAME
        "FRAME.RESIZE.Y" => 33, // SM_CYSIZEFRAME = SM_CYFRAME
        "ICON.X" => 11,         // SM_CXICON
        "ICON.Y" => 12,         // SM_CYICON
        "ICONSPACE.X" => 38,    // SM_CXICONSPACING
        "ICONSPACE.Y" => 39,    // SM_CYICONSPACING
        "MAXIMIZED.X" => 61,    // SM_CXMAXIMIZED
        "MAXIMIZED.Y" => 62,    // SM_CYMAXIMIZED
        "MENUBAR" => 15,        // SM_CYMENU
        "MINIMUM.X" => 28,      // SM_CXMIN
        "MINIMUM.Y" => 29,      // SM_CYMIN
        "SCROLL.HORZ" => 3,     // SM_CYHSCROLL
        "SCROLL.VERT" => 2,     // SM_CXVSCROLL
        _ => return None,
    })
}

/// `METRICS(Caption)` reaches the call site as a variable reference; turn a bare
/// metric name into its index so codegen only ever sees an integer literal.
fn fold_metric_name(e: Expr) -> Expr {
    if let Expr::Variable(n) = &e {
        if let Some(idx) = metric_index(std::slice::from_ref(n)) {
            return Expr::IntegerLit(idx);
        }
    }
    e
}
