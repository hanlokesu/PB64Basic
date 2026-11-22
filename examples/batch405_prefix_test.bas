#COMPILE EXE
#DIM ALL
' batch 405: user procedure names that share a compiler keyword prefix.
' GRAPHICBOX / PRINTFOO / GRAPHIC_PROBE / MENU_PROBE / XPRINT_PROBE must
' dispatch as user SUBs, not be swallowed by the builtin family guards.
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

SUB GRAPHICBOX()
    ConPrint "graphicbox-user"
END SUB

SUB PRINTFOO()
    ConPrint "printfoo-user"
END SUB

SUB GRAPHIC_PROBE()
    ConPrint "graphic-user"
END SUB

SUB MENU_PROBE()
    ConPrint "menu-user"
END SUB

SUB XPRINT_PROBE()
    ConPrint "xprint-user"
END SUB

FUNCTION PBMAIN() AS LONG
    GRAPHICBOX
    PRINTFOO
    GRAPHIC_PROBE
    MENU_PROBE
    XPRINT_PROBE
    ConPrint "prefix-all-ok"
END FUNCTION