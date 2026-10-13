#COMPILE EXE
#DIM ALL


' --- PBWin10 stub branch: this sample exercises PowerBasilisk-only ---
'     syntax that official PBWin10 does not provide; it compiles but
'     does nothing here.  The fork branch (#ELSE) is the real test.



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


FUNCTION PBMAIN() AS LONG
    LOCAL waitk AS STRING
    LOCAL h1, sz, lk, h2, old, p2, prev AS LONG
    LOCAL p AS LONG
    LOCAL fails AS LONG

    GLOBALMEM ALLOC 100 TO h1
    IF h1 > 0 THEN
        ConPrint "gm-alloc-PASS"
    ELSE
        ConPrint "gm-alloc-FAIL"
        INCR fails
    END IF

    GLOBALMEM SIZE h1 TO sz
    IF sz >= 100 THEN
        ConPrint "gm-size-PASS"
    ELSE
        ConPrint "gm-size-FAIL"
        INCR fails
    END IF

    GLOBALMEM LOCK h1 TO p
    IF p <> 0 THEN
        ConPrint "gm-lock-PASS"
    ELSE
        ConPrint "gm-lock-FAIL"
        INCR fails
    END IF
    IF p <> 0 THEN
        POKE BYTE, p, 65
        IF PEEK(p) = 65 THEN
            ConPrint "gm-rw-PASS"
        ELSE
            ConPrint "gm-rw-FAIL"
            INCR fails
        END IF
    END IF

    GLOBALMEM UNLOCK h1 TO lk
    IF lk = 0 THEN
        ConPrint "gm-unlock-PASS"
    ELSE
        ConPrint "gm-unlock-FAIL"
        INCR fails
    END IF

    GLOBALMEM FREE h1 TO h2
    IF h2 = 0 THEN
        ConPrint "gm-free-PASS"
    ELSE
        ConPrint "gm-free-FAIL"
        INCR fails
    END IF

    MOUSEPTR 1 TO old
    IF old = 1 THEN
        ConPrint "mouseptr-PASS"
    ELSE
        ConPrint "mouseptr-FAIL"
        INCR fails
    END IF

    UCODEPAGE OEM TO prev
    IF prev = 0 THEN
        ConPrint "ucode-oem-PASS"
    ELSE
        ConPrint "ucode-oem-FAIL"
        INCR fails
    END IF

    UCODEPAGE 850 TO p2
    IF p2 = 1 THEN
        ConPrint "ucode-num-PASS"
    ELSE
        ConPrint "ucode-num-FAIL"
        INCR fails
    END IF

    IF fails = 0 THEN
        ConPrint "BATCH17 ALL PASS"
    ELSE
        ConPrint "BATCH17 FAIL count=" & STR$(fails)
    END IF
    FUNCTION = fails
    ConPrint "Press any key to exit..."
    ConWaitKey
END FUNCTION





