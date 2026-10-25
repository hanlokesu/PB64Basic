// =====================================================================
// Modified by the hanlokesu/PB64Basic fork (https://github.com/hanlokesu/PB64Basic).
// Derived from benstopics/powerbasilisk (https://github.com/benstopics/powerbasilisk), Apache-2.0.
// See LICENSE for the full license terms.
// =====================================================================
/// PowerBASIC type names
#[derive(Debug, Clone, PartialEq)]
pub enum PbType {
    Long,
    Integer,
    Double,
    Single,
    String,
    FixedString(usize), // STRING * N
    Quad,
    Dword,
    Ext,
    Cur, // CURRENCY
    Byte,
    Word,
    UserDefined(String), // TYPE name
    /// Interface name used as an object-variable type: `LOCAL o AS MyInterface`
    /// (CHW object model, batch 264).  The variable holds a heap object
    /// reference (a pointer), so it maps to IrType::Ptr, never to a struct.
    Interface(String),
    Variant, // untyped / default
    Field,   // FIELD variable (16-byte: {ptr, len, pad})
}

impl PbType {
    pub fn suffix(&self) -> &str {
        match self {
            PbType::Long => "&",
            PbType::Integer => "%",
            PbType::Double => "#",
            PbType::Single => "!",
            PbType::String | PbType::FixedString(_) => "$",
            PbType::Quad => "&&",
            PbType::Dword => "???",
            PbType::Ext => "##",
            PbType::Cur => "@",
            _ => "",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Program {
    pub items: Vec<TopLevel>,
}

#[derive(Debug, Clone)]
pub enum TopLevel {
    GlobalDecl(VarDecl),
    GlobalDeclList(Vec<VarDecl>),
    DimDecl(DimStatement),
    DimDeclList(Vec<DimStatement>),
    ConstDecl(ConstDecl),
    TypeDecl(TypeDecl),
    EnumDecl(EnumDecl), // ENUM ... END ENUM — members register as equates
    SubDecl(SubDecl),
    FunctionDecl(FunctionDecl),
    DeclareStmt(DeclareStmt), // DECLARE SUB/FUNCTION — skip at runtime
    Statement(Statement),     // top-level statements (before PBMAIN)
    AsmData(AsmDataDecl),     // ASMDATA ... END ASMDATA read-only data block
    LinkFile(String),         // #LINK "filespec" — extra object/library for the linker
}

#[derive(Debug, Clone)]
pub struct AsmDataDecl {
    pub name: String, // canonical uppercase block name
    pub items: Vec<AsmDataItem>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub enum AsmDataItem {
    Db(Vec<AsmDataValue>),
    Dw(Vec<AsmDataValue>),
    Dd(Vec<AsmDataValue>),
    Dq(Vec<AsmDataValue>),
}

#[derive(Debug, Clone)]
pub enum AsmDataValue {
    Num(i64),    // decimal or &H hex literal
    Str(String), // ANSI literal (DB) or WIDE literal (DW, UTF-16LE)
}

#[derive(Debug, Clone)]
pub struct VarDecl {
    pub name: String, // canonical uppercase with suffix
    pub pb_type: PbType,
    pub is_array: bool, // declared with ()
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct DimStatement {
    pub scope: DimScope,
    pub name: String,
    pub pb_type: PbType,
    pub bounds: Vec<DimBound>, // one per dimension
    pub line: usize,
    pub is_redim: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DimScope {
    Local,
    Global,
    Static,
    Threaded, // THREADED: thread-local storage variable
    Dim,      // plain DIM inside a sub/function
}

/* CHW CLIPBOARD data formats (9 statement forms; 4 formats x SET/GET + RESET). */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardFormat {
    Text,
    OemText,
    Unicode,
    Bitmap,
}

#[derive(Debug, Clone)]
pub struct DimBound {
    pub lower: Expr,
    pub upper: Expr,
}

#[derive(Debug, Clone)]
pub struct ConstDecl {
    pub name: String,
    pub value: Expr,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct TypeDecl {
    pub name: String,
    pub fields: Vec<TypeField>,
    pub line: usize,
    /// True when declared as `UNION name ... END UNION` (CHW UNION/END UNION):
    /// all members overlap at offset 0; the structure size equals the longest
    /// member. LLVM has no union type, so codegen lays the struct out as
    /// `[<max-size> x i8]` and member access becomes offset-0 bitcast.
    pub is_union: bool,
    /// EVENT SOURCE interfaces advertised by this class (batch 222).
    pub event_sources: Vec<String>,
    /// Interface blocks implemented by this CLASS (batch 264).  Each entry is
    /// the normalized (uppercase) interface name; codegen maps the interface
    /// name used in `LOCAL o AS <Iface>` back to this class so `o.M(a)` can be
    /// lowered to the compiled `<Class>_<Method>` function.
    pub interfaces: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct EnumDecl {
    pub name: String,
    /// Members in declaration order; value = explicit value or auto-increment.
    pub items: Vec<EnumItem>,
    pub line: usize,
    /// BITS option: members auto-assign 1, 2, 4, 8 ... (CHW ENUM/END ENUM).
    pub bits: bool,
    /// SINGULAR option: members register without the ENUM-name prefix.
    pub singular: bool,
}

#[derive(Debug, Clone)]
pub struct EnumItem {
    pub name: String, // %MEMBER or MEMBER (equate name)
    pub value: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct TypeField {
    pub name: String,
    pub pb_type: PbType,
    /// CHW "Structures within structures" (TYPE/END TYPE block, batch 278):
    /// a bare nested structure name on its own line embeds that TYPE/UNION.
    /// Members are flattened into the outer type (X.H works, not only
    /// X.ABCD2.ABCD1.H). Some(name) = nested embed; name/pb_type are empty.
    pub nested: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SubDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub body: Vec<Statement>,
    pub alias: Option<String>,
    pub export: bool,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct FunctionDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: PbType,
    pub body: Vec<Statement>,
    pub alias: Option<String>,
    pub export: bool,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub pb_type: PbType,
    pub is_byval: bool,
    pub is_optional: bool,
    pub is_array: bool,
}

#[derive(Debug, Clone)]
pub struct DeclareStmt {
    pub name: String,
    pub is_function: bool, // true=FUNCTION, false=SUB
    pub params: Vec<Param>,
    pub return_type: PbType,   // Void for SUB
    pub lib: Option<String>,   // LIB "dll.dll"
    pub alias: Option<String>, // ALIAS "ExternalName"
    pub line: usize,
}

// Statements
#[derive(Debug, Clone)]
pub enum Statement {
    /* CHW CLIPBOARD forms: SET/GET <format>; format drives the Win32 clipboard
    data format and any character-set conversion (TEXT/OEMTEXT/UNICODE/BITMAP). */
    ClipboardSet {
        format: ClipboardFormat,
        value: Expr,
        result: Option<Expr>,
    },
    ClipboardGet {
        format: ClipboardFormat,
        target: Expr,
        result: Option<Expr>,
    },
    ClipboardReset {
        result: Option<Expr>,
    },
    Assign(AssignStmt),
    Call(CallStmt),
    If(IfStmt),
    For(ForStmt),
    DoLoop(DoLoopStmt),
    WhileWend(WhileWendStmt),
    SelectCase(SelectCaseStmt),
    Print(PrintStmt),
    PrintFile(PrintFileStmt),
    Open(OpenStmt),
    Field(FieldStmt),
    Mat(MatStmt),
    Close(CloseStmt),
    Dim(DimStatement),
    Redim(DimStatement),
    Incr(IncrDecrStmt),
    Decr(IncrDecrStmt),
    ExitSub,
    ExitFunction,
    ExitFor,
    ExitDo,
    ExitSelect,
    FunctionReturn(FunctionReturnStmt),
    GoSub(String),
    Return,
    GoTo(String),
    Label(String),
    OnGoTo {
        expr: Box<Expr>,
        labels: Vec<String>,
    },
    OnGoSub {
        expr: Box<Expr>,
        labels: Vec<String>,
    },
    OnCall {
        expr: Box<Expr>,
        targets: Vec<OnCallTarget>,
    },
    InputFlush,
    OnErrorGoto(String),
    OnErrorGotoZero,
    ResumeNext,
    Resume,
    ResumeFlush,
    ResumeLabel(String),
    Try(TryStmt),
    ExitTry,
    Iterate(IterateTarget),
    InputFile(InputFileStmt),
    LineInputFile(LineInputFileStmt),
    InputConsole(InputConsoleStmt),
    LineInputConsole(LineInputConsoleStmt),
    Kill(Expr),
    CallStk(Expr), // CALLSTK filename$ — dump the call stack to a file (batch 33)
    Profile(Expr), // PROFILE filename$ — dump per-procedure call counts + ms (batch 34)
    Regexpr {
        mask: Expr,
        target: Expr,
        start: Option<Expr>,
        pos_var: Expr,
        len_var: Option<Expr>,
    }, // REGEXPR mask$ IN target$ [AT start&] TO iPos& [, iLen&] (batch 35)
    Regrepl {
        mask: Expr,
        target: Expr,
        repl: Expr,
        start: Option<Expr>,
        pos_var: Expr,
        out_var: Expr,
    }, // REGREPL mask$ IN target$ WITH repl$ [AT start&] TO iPos&, newtarget$ (batch 35)
    Block(Vec<Statement>), // multiple statements from one parse (e.g. LOCAL a, b)
    Data(Vec<DataItem>), // DATA item1, item2, ... (string constants)
    Restore,       // RESTORE — reset DATA read cursor
    Noop(String, usize), // (unimplemented statement name, source line)
    Asm(String),   // inline assembly text (from ! or ASM)
    DefType(DefTypeStmt), // DEFINT/DEFSTR/... — default type for letters (CHW DEFBYT_..._DEFLNG.htm)
    /* batch278-74: OBJECT CALL/GET/LET/SET obj.member[(args)] [TO result] —
     * DISPATCH method/property access with VARIANT marshalling (CHW OBJECT
     * statement / How_do_you_call_a_DISPATCH_METHOD).  `object` is the object
     * variable (IDispatch*), `args` are COM-compatible variables (literal and
     * expression args are rejected by the parser, matching CHW), `result` is
     * the optional result variable name. */
    ObjectCall(ObjectCallStmt),
}

/// OBJECT CALL/GET/LET/SET — DISPATCH interface access (batch278-74).
/// kind: "CALL" | "GET" | "LET" | "SET".
/// members: the dotted member chain, e.g. `o.Drives.Count` = [Drives, Count].
/// Every segment carries its own optional argument list, so both official
/// shapes work: `o.A.B(x)` (args on the last segment) and `o.A(x).B` (args on
/// an intermediate segment, like the Excel sample `Range(vRange).Value`).
#[derive(Debug, Clone)]
pub struct ObjectCallStmt {
    pub object: Expr,
    pub members: Vec<ObjectMember>,
    pub result: Option<String>,
    pub kind: String,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct ObjectMember {
    pub name: String,
    pub args: Vec<Expr>,
}

/// DEFtype statement: `DEFINT A-E, G, Q, Y-Z` — letters (case-insensitive) that
/// get the given default type when used as implicitly-declared variables.
/// A single letter is stored as (c, c); a range as (from, to) inclusive.
#[derive(Debug, Clone)]
pub struct DefTypeStmt {
    pub pb_type: PbType,
    pub ranges: Vec<(char, char)>,
    pub line: usize,
}

/// One DATA item.  `quoted` records whether the source wrote it inside quotes:
/// quoted items keep leading/trailing spaces; unquoted items are trimmed at
/// compile time (CHW DATA_statement.htm).
#[derive(Debug, Clone)]
pub struct DataItem {
    pub text: String,
    pub quoted: bool,
}

#[derive(Debug, Clone)]
pub enum IterateTarget {
    For,
    Do,
}

#[derive(Debug, Clone)]
pub struct AssignStmt {
    pub target: Expr, // variable, array element, or type member
    pub value: Expr,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct CallStmt {
    pub name: String,
    pub args: Vec<Expr>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct IfStmt {
    pub condition: Expr,
    pub then_body: Vec<Statement>,
    pub elseif_clauses: Vec<ElseIfClause>,
    pub else_body: Vec<Statement>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct ElseIfClause {
    pub condition: Expr,
    pub body: Vec<Statement>,
}

/// TRY ... CATCH ... [FINALLY ...] END TRY structured error trap (batch 26).
/// CATCH is mandatory in official PB; finally is optional. We accept
/// catch-less TRY/END TRY too (treated as no-op catch) for robustness.
#[derive(Debug, Clone)]
pub struct TryStmt {
    pub body: Vec<Statement>,
    pub catch: Vec<Statement>,
    pub finally: Vec<Statement>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct ForStmt {
    pub var: String,
    pub start: Expr,
    pub end: Expr,
    pub step: Option<Expr>,
    pub body: Vec<Statement>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct DoLoopStmt {
    pub condition: Option<Expr>,
    pub is_while: bool,    // true=WHILE, false=UNTIL
    pub is_pre_test: bool, // condition at DO vs LOOP
    pub body: Vec<Statement>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct WhileWendStmt {
    pub condition: Expr,
    pub body: Vec<Statement>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct SelectCaseStmt {
    pub expr: Expr,
    pub cases: Vec<CaseClause>,
    pub else_body: Vec<Statement>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct CaseClause {
    pub patterns: Vec<CasePattern>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum CasePattern {
    Value(Expr),
    Range(Expr, Expr),
    Is(CaseOp, Expr),
}

#[derive(Debug, Clone)]
pub enum CaseOp {
    Lt,
    Gt,
    Lte,
    Gte,
    Eq,
    Neq,
}

#[derive(Debug, Clone)]
pub struct PrintStmt {
    pub args: Vec<Expr>,
    /// Trailing separator of the statement, if the source ended with one.
    ///
    /// PowerBASIC documents `;` and `,` at the end of a PRINT as suppressing
    /// the newline, so the next PRINT continues on the same line.  `None`
    /// means the statement prints its own newline.  A separator *between* two
    /// arguments is consumed by the parser and does not set this field.
    pub trailing: Option<char>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct PrintFileStmt {
    pub file_num: Expr,
    pub args: Vec<Expr>,
    /// Trailing separator (`;` or `,`) suppresses the CR/LF, exactly as for
    /// PRINT (CHW PRINT#: "If the final argument of a PRINT# statement is a
    /// semicolon or comma, PRINT# will not append the (default) CR/LF byte
    /// pair").  `None` means the statement writes its own newline.
    pub trailing: Option<char>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct OpenStmt {
    pub filename: Expr,
    pub mode: OpenMode,
    pub file_num: Expr,
    pub reclen: Option<Expr>, // LEN=n (RANDOM mode record length)
    pub line: usize,
}

#[derive(Debug, Clone)]
pub enum OpenMode {
    Output,
    Append,
    Input,
    Binary,
    Random,
}

/// MAT matrix-algebra statement (batch 32).
/// `MAT dst() = RHS` where RHS is one of CON / CON(expr) / IDN / ZER /
/// src() / src() + src() / src() - src() / src() * src() /
/// (expr) * src() / INV(src()) / TRN(src()).
#[derive(Debug, Clone)]
pub struct MatStmt {
    pub dst: String, // normalized destination array name
    pub op: MatOp,
    pub src1: Option<String>, // normalized source array name
    pub src2: Option<String>, // second source (add/sub/mul)
    pub scalar: Option<Expr>, // CON(expr) / (expr) * array
    pub line: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MatOp {
    Assign,    // dst = src
    Con,       // dst = CON         (all ones)
    ConScalar, // dst = CON(expr)
    Zer,       // dst = ZER
    Idn,       // dst = IDN         (2-D square identity)
    Add,       // dst = a + b
    Sub,       // dst = a - b
    Mul,       // dst = a * b       (2-D matrix multiply)
    Scale,     // dst = (expr) * a
    Trn,       // dst = TRN(a)      (2-D transpose)
    Inv,       // dst = INV(a)      (2-D square inverse)
}

#[derive(Debug, Clone)]
pub struct FieldStmt {
    pub kind: FieldKind,
    pub filenum: Option<Expr>,  // File mode: FIELD #n, ...
    pub dyn_expr: Option<Expr>, // Str mode: FIELD dyn$, ...
    pub specs: Vec<FieldSpec>,  // (size, field var name)
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FieldKind {
    File,  // FIELD #n, size AS var
    Str,   // FIELD dyn$, size AS var
    Reset, // FIELD RESET var
    ToStr, // FIELD STRING var
}

#[derive(Debug, Clone)]
pub struct FieldSpec {
    pub size: i64,
    pub name: String,
    pub offset: i64, // -1 = auto (cumulative); >= 0 = explicit FROM nStart-1
}

#[derive(Debug, Clone)]
pub struct CloseStmt {
    pub file_num: Expr,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct InputFileStmt {
    pub file_num: Expr,
    pub vars: Vec<Expr>, // variables to read into (Variable or ArrayAccess)
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct LineInputFileStmt {
    pub file_num: Expr,
    pub var: Expr, // string variable to read into
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct InputConsoleStmt {
    pub prompt: Option<Expr>, // optional prompt string
    pub no_newline: bool,     // ; means no newline after prompt
    pub vars: Vec<Expr>,      // variables to read into (strings first)
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct LineInputConsoleStmt {
    pub prompt: Option<Expr>, // optional prompt string
    pub var: Expr,            // string variable to read whole line into
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct IncrDecrStmt {
    pub target: Expr,
    pub amount: Option<Expr>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct FunctionReturnStmt {
    pub value: Expr,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct OnCallTarget {
    pub name: String,
    pub args: Vec<Expr>,
    pub ret_var: Option<String>,
}

// Expressions
#[derive(Debug, Clone)]
pub enum Expr {
    IntegerLit(i64),
    FloatLit(f64),
    StringLit(String),
    Variable(String),
    ArrayAccess(String, Vec<Expr>),
    TypeMember(Box<Expr>, String),
    FunctionCall(String, Vec<Expr>),
    /// `o.M(a, b)` — a method call written with a dot on a TYPE instance
    /// (the batch 216/217 sugar). `base` is the receiver expression on the
    /// left of the dot. codegen lowers the whole node to the qualified free
    /// function `<Class>_<Method>(o, a, b)` that METHOD was compiled as; it
    /// never falls back to a silent value when the receiver or the method
    /// cannot be resolved.
    MethodCall {
        base: Box<Expr>,
        name: String,
        args: Vec<Expr>,
    },
    /// `CLASS "Name"` — object instantiation (CHW How_do_you_create_an_object_).
    /// The class name is always a string literal; the result is a heap object
    /// (a fresh instance of the CLASS's INSTANCE storage) or NULL on failure,
    /// matching PBWin's "object variable = NOTHING when creation fails".
    ClassNew(String),
    /// `NEWCOM "ProgID"` — COM object instantiation (CHW Creating_a_DISPATCH_
    /// Object).  The ProgID is always a string literal; the result is a COM
    /// IDispatch* (or NULL when CoCreateInstance fails), matching PBWin.
    NewCom(String),
    UnaryOp(UnaryOp, Box<Expr>),
    BinaryOp(BinaryOp, Box<Expr>, Box<Expr>),
    PercentConst(String),
    Negate(Box<Expr>),
    Varptr(Box<Expr>),        // VARPTR(variable) or VARPTR(array(idx))
    ByvalOverride(Box<Expr>), // BYVAL expr at call site — pass value, not address
    /// An empty argument slot (PB official optional-arg comma placeholder):
    /// `MSGBOX "text", , "title"` leaves style as Omitted.  Only ever
    /// produced in call argument lists; never valid inside an expression.
    Omitted,
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    IntDiv,
    Mod,
    Pow,
    Eq,
    Neq,
    Lt,
    Gt,
    Lte,
    Gte,
    And,
    Or,
    Xor,
    Eqv,
    Imp,
    StrConcat, // &
}
