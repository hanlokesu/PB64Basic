#COMPILE EXE
#DIM ALL
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


' batch 27: ON CALL / GET$$+PUT$$ / MACRO
FUNCTION PBMAIN() AS LONG
    LOCAL waitk AS STRING
    LOCAL i AS LONG
    LOCAL n AS LONG
    LOCAL r AS LONG
    LOCAL s AS STRING
    LOCAL f AS LONG
    LOCAL ok AS LONG
    LOCAL x AS LONG

    ok = 0

    ' --- 1. ON CALL: SUB targets, selected by n ---
    ON 1 CALL OneProc(77), TwoProc(11), ThreeProc()
    ON 2 CALL OneProc(77), TwoProc(11), ThreeProc()
    ON 3 CALL OneProc(77), TwoProc(11), ThreeProc()
    ON 5 CALL OneProc(77), TwoProc(11), ThreeProc()  ' out of range: no call

    ' --- 2. ON CALL with FUNCTION TO var ---
    r = 0
    n = 0
    ON 1 CALL GetDouble() TO r, GetDouble() TO n
    IF r = 42 AND n = 0 THEN ok = ok + 1
    ConPrint "oncall-fn1: r=" & STR$(r) & " n=" & STR$(n)
    r = 0
    n = 0
    ON 2 CALL GetDouble() TO r, GetDouble() TO n
    IF r = 0 AND n = 84 THEN ok = ok + 1
    ConPrint "oncall-fn2: r=" & STR$(r) & " n=" & STR$(n)

    ' --- 3. GET$$ / PUT$$: wide string round-trip ---
    OPEN "WIDETEST.DAT" FOR BINARY AS #1
    PUT$$ #1, "AB"
    PUT$$ #1, "CD"
    SEEK #1, 1
    GET$$ #1, 2, s
    ConPrint "wide-read1: " & s
    IF s = "AB" THEN ok = ok + 1
    GET$$ #1, 2, s
    ConPrint "wide-read2: " & s
    IF s = "CD" THEN ok = ok + 1
    CLOSE #1
    KILL "WIDETEST.DAT"

    ' --- 4. Single-line MACRO in an expression ---
    MACRO muldivide(p1, p2, p3) = ((p1 * p2) / p3)
    x = muldivide(3, 3, 2) + 10
    ConPrint "macro-expr: " & STR$(x)
    IF x = 14 THEN ok = ok + 1

    ' --- 5. No-arg single-line macro ---
    MACRO AppTitle = "PB27-MACRO"
    s = AppTitle
    ConPrint "macro-const: " & s
    IF s = "PB27-MACRO" THEN ok = ok + 1

    ' --- 6. Multi-line MACRO at statement position ---
    MACRO Swap2(a, b)
    DIM t AS LONG
    t = a
    a = b
    b = t
    END MACRO
    n = 5
    i = 9
    Swap2(n, i)
    ConPrint "macro-swap: n=" & STR$(n) & " i=" & STR$(i)
    IF n = 9 AND i = 5 THEN ok = ok + 1

    ConPrint "batch27 ok=" & STR$(ok) & " / 7"
    IF ok = 7 THEN
        ConPrint "ALL PASS"
    ELSE
        ConPrint "FAIL"
    END IF
    ConPrint "Press any key to exit..."
    waitk = WAITKEY$
END FUNCTION

SUB OneProc(v AS LONG)
    ConPrint "oncall-sub1: " & STR$(v)
END SUB

SUB TwoProc(v AS LONG)
    ConPrint "oncall-sub2: " & STR$(v)
END SUB

SUB ThreeProc()
    ConPrint "oncall-sub3"
END SUB

FUNCTION GetDouble() AS LONG
    STATIC ctr AS LONG
    INCR ctr
    FUNCTION = ctr * 42
END FUNCTION


#ENDIF
