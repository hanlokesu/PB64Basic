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
    ' PBWin10 stub: compiles, does nothing (ME/MYBASE verified on the fork).
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
' PBXB64 Example - batch 301 witness: ME / MYBASE pseudo-variables (CHW
' ME_pseudo-variable / MYBASE_pseudo-variable).  ME is automatically defined
' in every Method/Property as a reference to the current object, usable as
' ME.Method(...), assignable to an object variable and usable as a return
' value / argument.  MYBASE refers to the original (inherited) object - the
' fork has no INHERIT vtable chain yet, so it resolves to the receiver.
' Expected output (run mode):
'   me_double=20
'   me_alias=99
'   me_return=77
'   mybase_field=5
'   === FAILURES:0 ===
'=====================================================================

CLASS Box
    INSTANCE v AS LONG
    METHOD SetIt(n AS LONG) AS LONG
        v = n
        FUNCTION = v
    END METHOD
    METHOD DoubleIt(n AS LONG) AS LONG
        FUNCTION = ME.SetIt(n * 2)
    END METHOD
    METHOD TouchViaAlias() AS LONG
        LOCAL r AS Box
        r = ME
        r.v = 99
        FUNCTION = r.v
    END METHOD
    METHOD ReturnSelf() AS Box
        FUNCTION = ME
    END METHOD
    METHOD MyBaseField() AS LONG
        FUNCTION = MYBASE.v
    END METHOD
END CLASS

FUNCTION PBMAIN() AS LONG
    LOCAL a AS Box
    LOCAL fails AS LONG
    LOCAL got AS LONG

    got = Box_DoubleIt(a, 10)
    ConPrint "me_double=" & STR$(got)
    IF got <> 20 THEN fails = fails + 1

    got = Box_TouchViaAlias(a)
    ConPrint "me_alias=" & STR$(got)
    IF got <> 99 THEN fails = fails + 1

    a.v = 77
    LOCAL r2 AS Box
    r2 = Box_ReturnSelf(a)
    got = r2.v
    ConPrint "me_return=" & STR$(got)
    IF got <> 77 THEN fails = fails + 1

    a.v = 5
    got = Box_MyBaseField(a)
    ConPrint "mybase_field=" & STR$(got)
    IF got <> 5 THEN fails = fails + 1

    IF fails = 0 THEN
        ConPrint "=== FAILURES:0 ==="
    ELSE
        ConPrint "=== FAILURES:" & STR$(fails) & " ==="
    END IF
    FUNCTION = 0
WAITKEY$
END FUNCTION
#ENDIF
