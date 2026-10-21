' _b256b2_probe.bas - print version to observe PBWin/fork values
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
    BEEP
    ConPrint "beep_ok"

    ConPrint "bgr3_r=" & STR$(BGR(255, 0, 0))
    ConPrint "bgr3_g=" & STR$(BGR(0, 255, 0))
    ConPrint "bgr3_b=" & STR$(BGR(0, 0, 255))
    ConPrint "bgr1_ff0000=" & STR$(BGR(&HFF0000))
    ConPrint "bgr1_0000ff=" & STR$(BGR(&H0000FF))

    LOCAL s AS STRING
    s = BIN$(5)
    ConPrint "bin5=[" & s & "]"
    s = BIN$(0)
    ConPrint "bin0=[" & s & "]"
    s = BIN$(5, 8)
    ConPrint "bin5_8=[" & s & "]"
    s = BIN$(5, -6)
    ConPrint "bin5_m6=[" & s & "]"
    s = BIN$(5, 0, 2, 1)
    ConPrint "bin5_021=[" & s & "]"
    s = BIN$(-1)
    ConPrint "binm1=[" & s & "]"
    s = BIN$(255)
    ConPrint "bin255=[" & s & "]"
    s = OCT$(10)
    ConPrint "oct10=[" & s & "]"
    s = OCT$(10, 5)
    ConPrint "oct10_5=[" & s & "]"
    s = DEC$(12)
    ConPrint "dec12=[" & s & "]"
    s = DEC$(12, 5)
    ConPrint "dec12_5=[" & s & "]"

    LOCAL a AS STRING
    LOCAL b AS STRING
    a = "ab"
    b = "cd"
    s = BUILD$(a, b, "ef")
    ConPrint "build=[" & s & "]"
    s = BUILD$(a, "bb", b)
    ConPrint "build2=[" & s & "]"
    s = BUILD$(a, "x", b)
    ConPrint "build3=[" & s & "]"
    s = BUILD$("x")
    ConPrint "build1=[" & s & "]"
END FUNCTION
