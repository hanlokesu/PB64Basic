#COMPILE EXE
#DIM ALL

' PB64Basic Enhanced - batch 310 test
' Fix: 32-bit (i686) codegen pointer width. STRPTR/VARPTR were hard-coded via
' ptrtoint i64; on 32-bit, BYVAL STRPTR/VARPTR in a stdcall frame pushed 8
' bytes, shifting every later argument and the callee stack cleanup, which
' caused a deterministic access violation (C0000005). ptrtoint now uses the
' native pointer width: i32 on 32-bit, i64 on 64-bit.
' Headless smoke: it passes BYVAL STRPTR to the stdcall WriteFile and prints
' verifiable output; there is no WAITKEY/MSGBOX tail.
DECLARE FUNCTION GetStdHandle LIB "KERNEL32.DLL" ALIAS "GetStdHandle" (BYVAL nStdHandle AS DWORD) AS LONG
DECLARE FUNCTION WriteFile LIB "KERNEL32.DLL" ALIAS "WriteFile" (BYVAL hFile AS LONG, lpBuffer AS ANY, BYVAL nBytesToWrite AS DWORD, lpBytesWritten AS DWORD, BYVAL lpOverlapped AS LONG) AS LONG

FUNCTION PBMAIN() AS LONG
    LOCAL s AS STRING
    LOCAL h AS LONG
    LOCAL written AS DWORD
    h = GetStdHandle(-11)
    s = "batch310 BYVAL STRPTR stdcall ok" & $CRLF
    WriteFile h, BYVAL STRPTR(s), LEN(s), written, 0
    PRINT "len="; LEN(s)
    PRINT "OK=1"
    FUNCTION = 0
END FUNCTION
