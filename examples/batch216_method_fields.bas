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

' --- PBWin10 stub branch: this sample exercises PB64Basic-only ---
'     syntax that official PBWin10 does not provide; it compiles but
'     does nothing here.  The fork branch (#ELSE) is the real test.
#IF %MY_PBVER = 10
FUNCTION PBMAIN() AS LONG
    ' PB64Basic-only sample: PBWin10 stub (compiles, does nothing).
END FUNCTION
#ELSE

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

'=====================================================================
' PBXB64 Example - batch 216 witness: a METHOD changes the object it was
' called on, and two objects keep separate state.
'---------------------------------------------------------------------
' Before batch 216 a METHOD inside a CLASS could not touch INSTANCE fields
' at all.  A bare field name in the body missed the symbol table, fell
' through to the auto-declare path and became a fresh zero-initialised
' local: the method ran, reported success, and the object was unchanged.
' Measured on the pre-batch-216 compiler (same shape as this file):
'   dotted a.v =11   dotted b.v =22
'   method set =33   method get =0   dotted after method: a.v =11
'
' What this witness pins down:
'   * the implicit BYREF receiver: <Class>_<Method>(object, args)
'   * a bare INSTANCE field name in a method body means this.<field>,
'     on both sides - writing (SetIt) and reading (GetIt)
'   * per-object state: a.v and b.v move independently
' Expected output (run mode):
'   a.v after SetIt(33) = 33
'   b.v untouched       = 22
'   a.v via GetIt()     = 33
'   b.v via GetIt()     = 22
'   === FAILURES:0 ===
' Complexity: O(1) - four field reads, five comparisons.
'=====================================================================

CLASS Box
    INSTANCE v AS LONG
    METHOD SetIt(n AS LONG) AS LONG
        v = n
        FUNCTION = v
    END METHOD
    METHOD GetIt() AS LONG
        FUNCTION = v
    END METHOD
END CLASS

FUNCTION PBMAIN() AS LONG
    LOCAL a AS Box
    LOCAL b AS Box
    LOCAL got AS LONG
    LOCAL fails AS LONG

    a.v = 11
    b.v = 22

    got = Box_SetIt(a, 33)
    ConPrint "a.v after SetIt(33) =" & STR$(a.v)
    ConPrint "b.v untouched       =" & STR$(b.v)
    ConPrint "a.v via GetIt()     =" & STR$(Box_GetIt(a))
    ConPrint "b.v via GetIt()     =" & STR$(Box_GetIt(b))

    IF got <> 33 THEN fails = fails + 1
    IF a.v <> 33 THEN fails = fails + 1
    IF b.v <> 22 THEN fails = fails + 1
    IF Box_GetIt(a) <> 33 THEN fails = fails + 1
    IF Box_GetIt(b) <> 22 THEN fails = fails + 1

    IF fails = 0 THEN
        ConPrint "=== FAILURES:0 ==="
    ELSE
        ConPrint "=== FAILURES:" & STR$(fails) & " ==="
    END IF
    FUNCTION = 0
' Press any key to exit...
WAITKEY$
END FUNCTION


#ENDIF
