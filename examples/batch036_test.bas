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

' PB64Basic Enhanced - batch 36: MKE$ (EXT = 8-byte double in this compiler)
#IF %DEF(%PB_REVISION)
    #IF (%PB_REVISION AND &H0FF00) = &H1000
        %MY_PBVER = 10
    #ELSE
        %MY_PBVER = 0
    #ENDIF
#ELSE
    %MY_PBVER = 0
#ENDIF
GLOBAL failures AS LONG

SUB Check(cond AS LONG, msg AS STRING)
    IF cond = 0 THEN
        ConPrint "FAIL: " & msg
        failures = failures + 1
    END IF
END SUB

FUNCTION PBMAIN() AS LONG
    LOCAL waitk AS STRING
    LOCAL e AS EXT
    LOCAL a AS STRING
    LOCAL b AS STRING
    failures = 0

    e = 1.5
    a = MKE$(e)
    b = MKD$(e)
#IF %MY_PBVER = 0
    ' fork: EXT is an 8-byte IEEE-754 double here, so MKE$ == MKD$
    Check(a = b, "mke = mkd bytes")
    Check(LEN(a) = 8, "mke length 8")

    e = -123.25
    Check(MKE$(e) = MKD$(e), "mke neg")
    Check(LEN(MKE$(e)) = 8, "mke neg len")
#ELSE
    ' PBWin10: EXT is the 80-bit format (10 bytes), so MKE$ differs from MKD$
    Check(a <> b, "mke differs from mkd (EXT 10-byte)")
    Check(LEN(a) = 10, "mke length 10")

    e = -123.25
    Check(MKE$(e) <> MKD$(e), "mke neg differs")
    Check(LEN(MKE$(e)) = 10, "mke neg len 10")
#ENDIF

    IF failures = 0 THEN
        ConPrint "batch36: ALL PASS"
    ELSE
        ConPrint "batch36: FAILURES=" & STR$(failures)
    END IF
    ConPrint "Press any key to exit..."
    ConWaitKey
END FUNCTION


