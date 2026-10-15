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

' PB64Basic Enhanced - batch 35: REGEXPR / REGREPL (documented subset)
GLOBAL failures AS LONG

SUB Check(cond AS LONG, msg AS STRING)
    IF cond = 0 THEN
        ConPrint "FAIL: " & STR$(msg)
        failures = failures + 1
    END IF
END SUB

FUNCTION PBMAIN() AS LONG
    LOCAL waitk AS STRING
    LOCAL p AS LONG
    LOCAL l AS LONG
    LOCAL new$ AS STRING
    failures = 0

    ' basic literal, case-insensitive default
    REGEXPR "cat" IN "the cat sat" TO p, l
    Check(p = 5 AND l = 3, "literal")
    REGEXPR "CAT" IN "the cat sat" TO p, l
    Check(p = 5 AND l = 3, "case-insensitive")

    ' . wildcard
    REGEXPR "c.t" IN "the cat sat" TO p, l
    Check(p = 5 AND l = 3, "dot")

    ' anchors
    REGEXPR "^the" IN "the cat sat" TO p, l
    Check(p = 1 AND l = 3, "caret")
    REGEXPR "sat$" IN "the cat sat" TO p, l
    Check(p = 9 AND l = 3, "dollar")

    ' character class + range + negated class
    REGEXPR "[a-z]at" IN "the cat sat" TO p, l
    Check(p = 5 AND l = 3, "class")
    REGEXPR "[^x]at" IN "the cat sat" TO p, l
    Check(p = 5 AND l = 3, "neg class")

    ' alternation, leftmost
    REGEXPR "dog|cat" IN "the cat sat" TO p, l
    Check(p = 5 AND l = 3, "alternation")

    ' quantifier *
    REGEXPR "ca*t" IN "the caat sat" TO p, l
    Check(p = 5 AND l = 4, "star")

    ' no match
    REGEXPR "xyz" IN "the cat sat" TO p, l
    Check(p = 0 AND l = 0, "no match")

    ' AT start
    REGEXPR "cat" IN "cat cat" AT 5 TO p, l
    Check(p = 5 AND l = 3, "at start")

    ' REGREPL: replace first match
    REGREPL "cat" IN "the cat sat" WITH "dog" TO p, new$
    Check(new$ = "the dog sat", "regrepl text")
    Check(p = 8, "regrepl pos")

    ' REGREPL: no match copies target
    REGREPL "zzz" IN "the cat sat" WITH "dog" TO p, new$
    Check(new$ = "the cat sat", "regrepl no-match")
    Check(p = 0, "regrepl no-match pos")

    ' REGREPL with wildcard
    REGREPL "c.t" IN "the cat sat" WITH "cow" TO p, new$
    Check(new$ = "the cow sat", "regrepl wildcard")

    IF failures = 0 THEN
        ConPrint "batch35: ALL PASS"
    ELSE
        ConPrint "batch35: FAILURES=" & STR$(failures)
    END IF
    ConPrint "Press any key to exit..."
    waitk = WAITKEY$
END FUNCTION


#ENDIF
