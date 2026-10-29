#COMPILE EXE
#IF %DEF(%PB_REVISION)
    #IF (%PB_REVISION AND &H0FF00) = &H1000
        %MY_PBVER = 10
    #ELSE
        %MY_PBVER = 0
    #ENDIF
#ELSE
    %MY_PBVER = 0
#ENDIF
#IF %MY_PBVER = 10
FUNCTION PBMAIN() AS LONG
    ' PBWin10 stub: compiles, does nothing (GUID$/GUIDTXT$ verified on the fork).
END FUNCTION
#ELSE
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

'=====================================================================
' PBXB64 Example - batch 299 witness: GUID$ / GUIDTXT$ (CHW GUID_function /
' GUIDTXT_function).  GUID$() returns a NEW 16-byte unique GUID; GUID$(txt)
' parses a 38-byte "{XXXXXXXX-...}" text back to 16 bytes; GUIDTXT$(guid16$)
' is the complement (16 bytes -> 38-byte text).
' Expected output (run mode):
'   len_new=16
'   len_txt=38
'   txt0={ 
'   roundtrip=1
'   empty_arg_len=16
'   === FAILURES:0 ===
'=====================================================================

FUNCTION PBMAIN() AS LONG
    LOCAL g AS STRING
    LOCAL t AS STRING
    LOCAL g3 AS STRING
    LOCAL ge AS STRING
    LOCAL fails AS LONG

    g = GUID$()
    ConPrint "len_new=" & STR$(LEN(g))
    t = GUIDTXT$(g)
    ConPrint "len_txt=" & STR$(LEN(t))
    ConPrint "txt0=" & LEFT$(t, 1)
    g3 = GUID$(t)
    IF g3 = g THEN
        ConPrint "roundtrip=1"
    ELSE
        ConPrint "roundtrip=0"
        fails = fails + 1
    END IF
    ge = GUID$("")
    ConPrint "empty_arg_len=" & STR$(LEN(ge))
    IF LEN(g) <> 16 THEN fails = fails + 1
    IF LEN(t) <> 38 THEN fails = fails + 1
    IF LEN(ge) <> 16 THEN fails = fails + 1
    IF g3 <> g THEN fails = fails + 1

    IF fails = 0 THEN
        ConPrint "=== FAILURES:0 ==="
    ELSE
        ConPrint "=== FAILURES:" & STR$(fails) & " ==="
    END IF
    FUNCTION = 0
WAITKEY$
END FUNCTION
#ENDIF
