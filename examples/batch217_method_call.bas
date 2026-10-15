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
' PBXB64 Example - batch 217 witness: the dotted method call `o.M(args)`.
'---------------------------------------------------------------------
' batch 216 gave a METHOD an implicit BYREF receiver, so
'     Box_SetIt(a, 33)
' already mutated the object.  What was still missing is the dot form a PB
' programmer actually writes:
'     a.SetIt(33)          ' statement position
'     x = a.GetIt()        ' expression position
' Before batch 217 both spellings stopped the build:
'     Codegen warning: FUNCTION PBMAIN -> Runtime error: Unknown field: C.GETIT
'     [pbcompiler] 1 codegen errors (functions skipped)
'     Error: I/O error: clang EXE link failed:
'     lld-link: error: undefined symbol: PBMAIN
' The parser built `o.GetIt` as a field access and dropped the argument list,
' so the call never reached codegen at all.  It was a loud failure - no silent
' wrong value - but a gap.
'
' What this witness pins down:
'   * `o.M(args)` lowers to the qualified procedure `<Class>_<Method>(o, args)`
'   * statement position: the call runs, the result is discarded, the object changes
'   * expression position: the method's return value is what gets assigned
'   * a real argument list after the dot: receiver first, then the written args
'   * per-object state survives the sugar (b is untouched by a's calls)
'   * `o.M(args)` and `Box_M(o, args)` are interchangeable - both are checked
' Expected output (run mode):
'   a.v after a.SetIt(33) = 33
'   b.v untouched         = 22
'   a.GetIt() / b.GetIt() = 33 / 22
'   a.SetIt(44) then GetIt = 44
'   a.Add(7)              = 51
'   long form Box_Add(a,7)= 51
'   === FAILURES:0 ===
' Complexity: O(1) - a handful of field reads, eight comparisons.
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
    METHOD Add(n AS LONG) AS LONG
        FUNCTION = v + n
    END METHOD
END CLASS

FUNCTION PBMAIN() AS LONG
    LOCAL a AS Box
    LOCAL b AS Box
    LOCAL got AS LONG
    LOCAL set1 AS LONG
    LOCAL set2 AS LONG
    LOCAL geta AS LONG
    LOCAL getb AS LONG
    LOCAL added AS LONG
    LOCAL longform AS LONG
    LOCAL fails AS LONG

    a.v = 11
    b.v = 22

    ' expression position: the return value is assigned
    set1 = a.SetIt(33)
    geta = a.GetIt()
    getb = b.GetIt()
    ConPrint "a.v after a.SetIt(33) =" & STR$(a.v)
    ConPrint "b.v untouched         =" & STR$(b.v)
    ConPrint "a.GetIt() / b.GetIt() =" & STR$(geta) & "/" & STR$(getb)

    ' statement position: no assignment, the object still changes
    a.SetIt(44)
    set2 = a.GetIt()
    ConPrint "a.SetIt(44) then GetIt =" & STR$(set2)

    ' a real argument list after the dot: receiver + written args
    added = a.Add(7)
    ConPrint "a.Add(7)              =" & STR$(added)

    ' the long form must agree with the sugar
    longform = Box_Add(a, 7)
    ConPrint "long form Box_Add(a,7)= " & STR$(longform)

    IF set1 <> 33 THEN fails = fails + 1
    IF a.v <> 44 THEN fails = fails + 1
    IF b.v <> 22 THEN fails = fails + 1
    IF getha_check(geta, getb) <> 1 THEN fails = fails + 1
    IF set2 <> 44 THEN fails = fails + 1
    IF added <> 51 THEN fails = fails + 1
    IF longform <> 51 THEN fails = fails + 1
    IF Box_GetIt(b) <> 22 THEN fails = fails + 1

    IF fails = 0 THEN
        ConPrint "=== FAILURES:0 ==="
    ELSE
        ConPrint "=== FAILURES:" & STR$(fails) & " ==="
    END IF
    FUNCTION = 0
END FUNCTION

' 1 when geta is 33 and getb is 22, else 0
FUNCTION getha_check(BYVAL x AS LONG, BYVAL y AS LONG) AS LONG
    IF x = 33 AND y = 22 THEN
        FUNCTION = 1
    ELSE
        FUNCTION = 0
    END IF
' Press any key to exit...
WAITKEY$
END FUNCTION


#ENDIF
