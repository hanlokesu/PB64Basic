' batch260_link_test.bas - #LINK "sum.obj" external object module demo
' (batch260; see batch260_make_obj.txt for how sum.obj is built)
' Prints 50 (20 + 30) via the external sum() from sum.obj.
' FORK-ONLY: PBWin 10 rejects #LINK .obj with Error 630, so this file is an
' ONLY-FORK example.  ConPrint + WAITKEY$ keep the console open when run by
' double-click; the dual-run harness feeds DEVNULL stdin, so WAITKEY$ returns
' at once and the output is still captured and checked.
#COMPILE EXE
#LINK "sum.obj"
DECLARE FUNCTION sum ALIAS "sum" (BYVAL a AS LONG, BYVAL b AS LONG) AS LONG

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
    LOCAL t AS LONG
    LOCAL w AS STRING
    t = sum(20, 30)
    ConPrint STR$(t) & $CRLF
    ConPrint "Press any key to exit..."
    w = WAITKEY$
    FUNCTION = 0
END FUNCTION
