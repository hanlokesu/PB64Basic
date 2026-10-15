' === console emulation for dual-compiler compatibility ===
' (PBWin10 has no PRINT/#CONSOLE; this wrapper uses only official Win32 API)
DECLARE FUNCTION AllocConsole LIB "KERNEL32.DLL" ALIAS "AllocConsole" () AS LONG
DECLARE FUNCTION AttachConsole LIB "KERNEL32.DLL" ALIAS "AttachConsole" (BYVAL dwProcessId AS DWORD) AS LONG
DECLARE FUNCTION GetStdHandle LIB "KERNEL32.DLL" ALIAS "GetStdHandle" (BYVAL nStdHandle AS DWORD) AS LONG
DECLARE FUNCTION WriteFile LIB "KERNEL32.DLL" ALIAS "WriteFile" (BYVAL hFile AS LONG, lpBuffer AS ANY, BYVAL nBytesToWrite AS DWORD, lpBytesWritten AS DWORD, BYVAL lpOverlapped AS LONG) AS LONG
DECLARE FUNCTION ReadFile LIB "KERNEL32.DLL" ALIAS "ReadFile" (BYVAL hFile AS LONG, lpBuffer AS ANY, BYVAL nBytesToRead AS DWORD, lpBytesRead AS DWORD, BYVAL lpOverlapped AS LONG) AS LONG
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
SUB ConWaitKey()
    LOCAL h AS LONG
    LOCAL c AS STRING * 1
    LOCAL n AS DWORD
    h = GetStdHandle(-10)
    IF h <> 0 THEN
        ReadFile h, c, 1, n, 0
    END IF
END SUB

' PB64Basic Enhanced - batch 37: CVx family (binary string -> value)
GLOBAL failures AS LONG

SUB Check(cond AS LONG, msg AS STRING)
    IF cond = 0 THEN
        ConPrint "FAIL: " & msg
        failures = failures + 1
    END IF
END SUB

FUNCTION PBMAIN() AS LONG
    LOCAL waitk AS STRING
    LOCAL b AS BYTE
    LOCAL w AS WORD
    LOCAL l AS LONG
    LOCAL q AS QUAD
    LOCAL d AS DOUBLE
    LOCAL s AS SINGLE
    LOCAL x AS STRING
    failures = 0

    ' CVBYT: 1 byte
    x = MKBYT$(65)
    b = CVBYT(x)
    Check(b = 65, "cvbyt")

    ' CVW: 2 bytes little-endian
    x = MKWRD$(&H1234)
    w = CVW(x)
    Check(w = &H1234, "cvw")

    ' CVL: 4 bytes (signed LONG)
    x = MKL$(123456)
    l = CVL(x)
    Check(l = 123456, "cvl")
    x = MKL$(-7)
    l = CVL(x)
    Check(l = -7, "cvl neg")

    ' CVDWD: 4 bytes unsigned DWORD
    x = MKDWD$(&HFFFFFFFF)
    q = CVDWD(x)
    Check(q = 4294967295, "cvdwd")

    ' CVQ: 8 bytes QUAD
    x = MKQ$(1234567890123)
    q = CVQ(x)
    Check(q = 1234567890123, "cvq")

    ' CVS: 4-byte single
    x = MKS$(1.5)
    s = CVS(x)
    Check(s = 1.5, "cvs")

    ' CVD: 8-byte double
    x = MKD$(2.25)
    d = CVD(x)
    Check(d = 2.25, "cvd")

    ' CVE: EXT = 8-byte double here
    x = MKE$(3.5)
    d = CVE(x)
    Check(d = 3.5, "cve")

    ' offset argument (1-based)
    x = "AB" + MKL$(999)
    l = CVL(x, 3)
    Check(l = 999, "cvl offset")

    IF failures = 0 THEN
        ConPrint "batch37: ALL PASS"
    ELSE
        ConPrint "batch37: FAILURES=" & STR$(failures)
    END IF
    ConPrint "Press any key to exit..."
    waitk = WAITKEY$
END FUNCTION

FUNCTION CVW(BYVAL s AS STRING) AS WORD
    LOCAL w AS WORD
    IF LEN(s) >= 2 THEN w = ASC(s) + 256 * ASC(MID$(s, 2, 1))
    FUNCTION = w
END FUNCTION


