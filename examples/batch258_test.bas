' batch258_test.bas - CHW C-family conversion probe (batch258 part 1):
' CBYT/CWRD/CQUD (official spellings) + CCUR/CCUX/CEXT rounding +
' CVI/CVWRD sign semantics + CSET$ (incl. USING) + BSTR NUL concat.
' Expected values = PBWin10 measured (_b258a_probe.bas MATCH).
DECLARE FUNCTION AllocConsole LIB "KERNEL32.DLL" ALIAS "AllocConsole" () AS LONG
DECLARE FUNCTION AttachConsole LIB "KERNEL32.DLL" ALIAS "AttachConsole" (BYVAL dwProcessId AS DWORD) AS LONG
DECLARE FUNCTION GetStdHandle LIB "KERNEL32.DLL" ALIAS "GetStdHandle" (BYVAL nStdHandle AS DWORD) AS LONG
DECLARE FUNCTION WriteFile LIB "KERNEL32.DLL" ALIAS "WriteFile" (BYVAL hFile AS LONG, lpBuffer AS ANY, BYVAL nBytesToWrite AS DWORD, lpBytesWritten AS DWORD, BYVAL lpOverlapped AS LONG) AS LONG
SUB ConPrint(BYVAL s AS STRING)
    LOCAL h AS LONG
    LOCAL n AS DWORD
    h = GetStdHandle(-11)
    IF h = 0 THEN
        IF AttachConsole(-1) = 0 THEN AllocConsole
        h = GetStdHandle(-11)
    END IF
    IF h <> 0 THEN
        WriteFile h, BYVAL STRPTR(s), LEN(s), n, 0
    END IF
END SUB
FUNCTION PBMAIN() AS LONG
    LOCAL ok AS LONG
    LOCAL s AS STRING
    ok = 0
    ' --- CBYT/CWRD/CQUD conversions (official PBWin spellings) ---
    IF CBYT(300) = 44 THEN ok = ok OR 1
    IF CBYT(-1) = 255 THEN ok = ok OR 2
    IF CWRD(70000) = 4464 THEN ok = ok OR 4
    IF CWRD(-1) = 65535 THEN ok = ok OR 8
    IF CQUD(4294967297&&) = 4294967297&& THEN ok = ok OR 16
    IF CQUD(-3.7) = -4 THEN ok = ok OR 32
    ' --- CCUR/CCUX/CEXT ---
    IF STR$(CCUR(1234.5678)) = " 1234.5678" THEN ok = ok OR 64
    IF STR$(CCUR(-0.125)) = "-.125" THEN ok = ok OR 128
    IF STR$(CCUX(12.345)) = " 12.35" THEN ok = ok OR 256
    IF STR$(CEXT(1.5)) = " 1.5" THEN ok = ok OR 512
    IF STR$(CEXT(2.0)) = " 2" THEN ok = ok OR 1024
    ' --- CVI (signed) / CVWRD (unsigned) ---
    s = CHR$(255) & CHR$(255)
    IF CVI(s) = -1 THEN ok = ok OR 2048
    IF CVWRD(s) = 65535 THEN ok = ok OR 4096
    s = CHR$(0) & CHR$(128)
    IF LEN(s) = 2 THEN ok = ok OR 8192
    IF CVI(s) = -32768 THEN ok = ok OR 16384
    IF CVWRD(s) = 32768 THEN ok = ok OR 32768
    ' --- CSET$ ---
    IF CSET$("ab", 6) = "  ab  " THEN ok = ok OR 65536
    IF CSET$("ab", 5) = " ab  " THEN ok = ok OR 131072
    IF CSET$("abcde", 3) = "abc" THEN ok = ok OR 262144
    IF CSET$("ab", 6 USING ".") = "..ab.." THEN ok = ok OR 524288
    IF CSET$("", 4) = "    " THEN ok = ok OR 1048576
    ConPrint "batch258 ok=" & STR$(ok)
    ' All 21 bits set = every assertion passed (values PBWin10-measured).  Return
    ' 0 only then, so the headless harness sees exit 0 on a clean run.
    IF ok = 2097151 THEN FUNCTION = 0 ELSE FUNCTION = 1
END FUNCTION
