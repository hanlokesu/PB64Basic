' batch218_test.bas - GLOBALMEM ALLOC/SIZE/LOCK/UNLOCK/FREE (batch 218: malformed form must be loud)
' (rebuilt from the batch's real optimization topic; see SKILL.md)
#COMPILE EXE
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
    LOCAL h AS LONG
    LOCAL sz AS LONG
    LOCAL p AS LONG
    LOCAL scratch AS LONG
    LOCAL ok AS LONG
    ok = 0
    GLOBALMEM ALLOC 64 TO h
    IF h <> 0 THEN ok = ok OR 1
    GLOBALMEM SIZE h TO sz
    IF sz >= 64 THEN ok = ok OR 2
    GLOBALMEM LOCK h TO p
    IF p <> 0 THEN ok = ok OR 4
    GLOBALMEM UNLOCK h TO scratch
    GLOBALMEM FREE h TO scratch
    IF ok = 7 THEN
        ConPrint "batch218 test ok"
    ELSE
        ConPrint "batch218 FAILURES: " & STR$(ok)
    END IF
    FUNCTION = 0
END FUNCTION

