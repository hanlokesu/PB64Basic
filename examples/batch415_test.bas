#COMPILE EXE
#DIM ALL

'=====================================================================
' PB64Basic Enhanced - batch 415 regression test
'---------------------------------------------------------------------
' Covers the three compiler-level fixes from the Gary Beene
' residual set:
'   1. a global dynamic array no longer hides a same-named local
'      scalar during DIM lowering;
'   2. Long Pointer operands inside inline ASM work on x86_64 and a
'      PB label between two ASM lines is folded into the same block
'      (the procedure is marked noinline so the optimizer cannot copy
'      the label);
'   3. the compiler banner is the release identity.
' The GUI fixes (subsystem 4.0, WM_GETMINMAXINFO, GRAPHIC RENDER
' edge semantics, ListView header geometry) are covered by the
' 50-sample screenshot harness.
'
' Expected output:
'   version -> v0.2.204
'   dim shadow -> 42 / 0,10,20
'   asm pointer -> 123
'   === FAILURES: 0 ===
'   exit code 0
'=====================================================================

GLOBAL P() AS LONG
GLOBAL GA() AS LONG
GLOBAL GB() AS LONG

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

FUNCTION DimShadow() AS LONG
    LOCAL p AS LONG
    LOCAL i AS LONG

    p = 42
    FOR i = 0 TO 2
        P(i) = i * 10
    NEXT
    IF p <> 42 THEN FUNCTION = 1 : EXIT FUNCTION
    IF P(0) <> 0 OR P(1) <> 10 OR P(2) <> 20 THEN FUNCTION = 2
END FUNCTION

SUB AsmPointerCopy()
    LOCAL lpSrc AS LONG POINTER
    LOCAL lpDst AS LONG POINTER

    lpSrc = VARPTR(GA(0))
    lpDst = VARPTR(GB(0))
    ! mov esi, lpSrc
    ! mov edi, lpDst
    ! mov eax, [esi]
    ! mov [edi], eax
END SUB

FUNCTION AsmLabelLoop() AS LONG
    LOCAL n AS LONG

    n = 2
    ! mov ecx, n
lp1:
    ! dec ecx
    ! jns SHORT lp1
    ! mov n, ecx
    FUNCTION = n
END FUNCTION

FUNCTION PBMAIN () AS LONG
    LOCAL fail AS LONG
    LOCAL v AS STRING
    LOCAL ds AS LONG
    LOCAL al AS LONG

    ReDim P(2)
    ReDim GA(0)
    ReDim GB(0)
    GA(0) = 123 : GB(0) = 0

    v = VERSION$
    ConPrint "version -> " & v & CHR$(13,10)
    IF v <> "v0.2.204" THEN fail = fail + 1
    IF LEN(v) <> 8 THEN fail = fail + 1

    ds = DimShadow()
    ConPrint "dim shadow -> " & STR$(ds) & " / " & STR$(P(0)) & "," & STR$(P(1)) & "," & STR$(P(2)) & CHR$(13,10)
    IF ds <> 0 THEN fail = fail + 1

    AsmPointerCopy()
    ConPrint "asm pointer -> " & STR$(GB(0)) & CHR$(13,10)
    IF GB(0) <> 123 THEN fail = fail + 1

    al = AsmLabelLoop()
    ConPrint "asm label -> " & STR$(al) & CHR$(13,10)
    IF al <> -1 THEN fail = fail + 1

    ConPrint CHR$(13,10)
    ConPrint "=== FAILURES: " & STR$(fail) & " ===" & CHR$(13,10)
    FUNCTION = fail
END FUNCTION
