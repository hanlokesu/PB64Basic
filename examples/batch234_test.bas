' Batch 234: REMOVE$/RETAIN$/REMAIN$ ANY + CHOOSE out-of-range default
' dual-compile: PBWin10 + PB64Basic; console via Win32 API
DECLARE FUNCTION AllocConsole LIB "KERNEL32.DLL" ALIAS "AllocConsole" () AS LONG
DECLARE FUNCTION GetStdHandle LIB "KERNEL32.DLL" ALIAS "GetStdHandle" (BYVAL nStdHandle AS DWORD) AS LONG
DECLARE FUNCTION WriteFile LIB "KERNEL32.DLL" ALIAS "WriteFile" (BYVAL hFile AS LONG, lpBuffer AS ANY, BYVAL nBytesToWrite AS DWORD, lpBytesWritten AS DWORD, BYVAL lpOverlapped AS LONG) AS LONG
DECLARE FUNCTION ReadFile LIB "KERNEL32.DLL" ALIAS "ReadFile" (BYVAL hFile AS LONG, lpBuffer AS ANY, BYVAL nBytesToRead AS DWORD, lpBytesRead AS DWORD, BYVAL lpOverlapped AS LONG) AS LONG

SUB ConPrint(BYVAL s AS STRING)
    LOCAL h AS LONG
    LOCAL n AS DWORD
    h = GetStdHandle(-11)
    IF h = 0 THEN
        AllocConsole
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
    LOCAL s AS STRING
    LOCAL i AS LONG
    LOCAL fails AS LONG

    fails = 0

    ' REMOVE$ ANY: remove any character in the match set
    s = "hello world"
    s = REMOVE$(s, ANY "lo")
    IF s = "he wrd" THEN
        ConPrint "REMOVE-ANY-PASS"
    ELSE
        ConPrint "REMOVE-ANY-FAIL [" & s & "]"
        fails = fails + 1
    END IF

    ' RETAIN$ ANY: keep only characters in the match set
    s = "a1b2c3"
    s = RETAIN$(s, ANY "0123456789")
    IF s = "123" THEN
        ConPrint "RETAIN-ANY-PASS"
    ELSE
        ConPrint "RETAIN-ANY-FAIL [" & s & "]"
        fails = fails + 1
    END IF

    ' REMAIN$ ANY: portion after first match character
    s = "hello123world"
    s = REMAIN$(s, ANY "0123456789")
    IF s = "23world" THEN
        ConPrint "REMAIN-ANY-PASS"
    ELSE
        ConPrint "REMAIN-ANY-FAIL [" & s & "]"
        fails = fails + 1
    END IF

    ' CHOOSE out of range: PBWin returns type default (0)
    i = CHOOSE(9, 10, 20, 30)
    IF i = 0 THEN
        ConPrint "CHOOSE-OOR-PASS"
    ELSE
        ConPrint "CHOOSE-OOR-FAIL [" & STR$(i) & "]"
        fails = fails + 1
    END IF

    ConPrint "FAILURES: " & STR$(fails)
    ConPrint "waiting for any key to exit..."
    ConWaitKey
END FUNCTION
