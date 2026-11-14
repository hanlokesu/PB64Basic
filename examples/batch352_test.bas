#COMPILE EXE
#DIM ALL

' PB64Basic Enhanced - batch 352 test
' Fix (x86_64 DECLARE pointer ABI): two codegen defects against the 32-bit-era
' Win32 header prototypes.
'  1. A BYVAL DWORD/LONG/i32 parameter receiving a 64-bit STRPTR/VARPTR/HANDLE
'     was truncated i64 -> i32, dropping the upper 32 bits and handing the
'     callee an invalid pointer (WriteFile wrote zero bytes). On x86_64 every
'     argument occupies a full 64-bit register/8-byte slot, so the i64 is kept
'     at full width; the i686 truncation is retained for the 32-bit stack-slot
'     constraint.
'  2. A BYREF pointer parameter passed a literal 0 was stored in a temp slot
'     and that slot address was passed (a non-NULL pointer to a zeroed struct).
'     The Win32 NULL convention means a literal 0 reaches the callee as a
'     null pointer directly.
' Headless smoke: writes a verifiable string through WriteFile with a DWORD
' lpBuffer prototype and a BYREF overlapped argument of 0; no WAITKEY/MSGBOX.
DECLARE FUNCTION GetStdHandle LIB "KERNEL32.DLL" ALIAS "GetStdHandle" (BYVAL nStdHandle AS DWORD) AS LONG
DECLARE FUNCTION WriteFile LIB "KERNEL32.DLL" ALIAS "WriteFile" (BYVAL hFile AS DWORD, BYVAL lpBuffer AS DWORD, BYVAL nBytesToWrite AS DWORD, lpBytesWritten AS DWORD, lpOverlapped AS DWORD) AS LONG

FUNCTION PBMAIN() AS LONG
    LOCAL s AS STRING
    LOCAL h AS LONG
    LOCAL written AS DWORD
    h = GetStdHandle(-11)
    s = "batch352 x64 pointer ABI ok" & $CRLF
    WriteFile h, BYVAL STRPTR(s), LEN(s), written, 0
    PRINT "written="; written
    PRINT "OK=1"
    FUNCTION = 0
END FUNCTION
