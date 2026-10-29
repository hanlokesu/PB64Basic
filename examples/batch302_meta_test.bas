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
    ' PBWin10 stub: compiles, does nothing (meta-statements verified on the fork).
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
' PBXB64 Example - batch 302 witness: the # meta-statement family + #INCLUDE
' (CHW _MESSAGES/_OPTIMIZE/_STACK/_TOOLS/_UTILITY/_DEBUG_CODE/_INCLUDE
' metastatement pages).  All of these must compile without error; #INCLUDE
' must actually expand the included file (its %constant and FUNCTION become
' available); #STACK must reach the linker (/STACK:bytes) without breaking
' the build.
' Expected output (run mode):
'   inc_const=42
'   inc_add=52
'   === FAILURES:0 ===
'=====================================================================

#INCLUDE "batch302_inc.bas"
#MESSAGES NOTIFY
#OPTIMIZE CODE ON
#OPTIMIZE SIZE
#STACK 1048576
#TOOLS OFF
#UTILITY "line ignored by the compiler"
#DEBUG CODE OFF

FUNCTION PBMAIN() AS LONG
    LOCAL fails AS LONG

    ConPrint "inc_const=" & STR$(%INC_CONST)
    ConPrint "inc_add=" & STR$(IncAdd(10))
    IF %INC_CONST <> 42 THEN fails = fails + 1
    IF IncAdd(10) <> 52 THEN fails = fails + 1

    IF fails = 0 THEN
        ConPrint "=== FAILURES:0 ==="
    ELSE
        ConPrint "=== FAILURES:" & STR$(fails) & " ==="
    END IF
    FUNCTION = 0
WAITKEY$
END FUNCTION
#ENDIF
