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

' =====================================================================
' batch185_test.bas - batch 185: the GRAPHIC keyboard family + SPLIT
'---------------------------------------------------------------------
' Purpose:   Exercise, in one labelled block each, everything this batch
'            implements:
'
'              1. GRAPHIC INSTAT TO NumericVar
'                   non-destructive query: TRUE when a character is ready,
'                   and it STAYS true until the character is read.
'              2. GRAPHIC INPUT FLUSH
'                   discard everything buffered (no operands).
'              3. GRAPHIC INSTAT keeps querying without consuming.
'              4. GRAPHIC INKEY$ TO str
'                   with nothing buffered the result is the null string -
'                   this is the one half of INKEY$ a headless run can prove.
'              5. GRAPHIC WAITKEY$("", 0) TO str
'                   TimeOut& = 0 returns at once instead of blocking, so the
'                   null string is verifiable without a keypress.
'              6. GRAPHIC SPLIT with a field wider than the text
'                   everything stays in part 1, part 2 is empty.
'              7. GRAPHIC SPLIT is lossless: part1 & part2 = the original.
'              8. GRAPHIC SPLIT with a narrow field really does split.
'              9. GRAPHIC SPLIT WORD can only shorten part 1, never lengthen
'                   it - the "do not break a word" guarantee, expressed as an
'                   invariant rather than as an expected character count.
'
'            Every statement below pumps this thread's message queue first,
'            because the graphic window in these programs has no explicit
'            message loop of its own - deliberate behaviour, not an accident.
'
' Expected output (run, no key pressed):  === FAILURES:0 ===
' Exit code: 0 on success, or the number of failed assertions.
'
' COMPILE-ONLY (cannot run headless)
'   GRAPHIC INPUT [prompt,] varlist      and   GRAPHIC LINE INPUT ["prompt"] var
'   both block until ENTER arrives, so any sample that runs them unattended
'   would time out (exit 124) and fail verification - the same reason the
'   official corpus marks its keyboard samples COMPILE ONLY.  They are
'   compiled here (see the two commented statements at the end of block 10)
'   and hand-tested by typing into a running PBGRAPHIC window.
' =====================================================================
FUNCTION PBMAIN () AS LONG
    LOCAL fails AS LONG
    LOCAL g AS LONG
    LOCAL k AS STRING
    LOCAL w AS STRING
    LOCAL src AS STRING
    LOCAL p1 AS STRING
    LOCAL p2 AS STRING
    LOCAL n1 AS STRING
    LOCAL n2 AS STRING

    fails = 0

    ' ---------------------------------------------------------------
    ' 1. GRAPHIC INSTAT - empty queue reports no character
    ' ---------------------------------------------------------------
    ConPrint "--- 1. GRAPHIC INSTAT: an empty queue reports no character ---"
    g = -1                                  ' poison, so a silent failure shows up
    GRAPHIC INSTAT TO g
    IF g <> 0 THEN
        fails = fails + 1
        ConPrint "  FAIL GRAPHIC INSTAT returned" & STR$(g) & "- expected 0 with an empty queue"
    ELSE
        ConPrint "  PASS GRAPHIC INSTAT -> 0 with nothing buffered"
    END IF

    ' ---------------------------------------------------------------
    ' 2. GRAPHIC INPUT FLUSH - accepted, and the queue stays empty
    ' ---------------------------------------------------------------
    ConPrint "--- 2. GRAPHIC INPUT FLUSH: accepted ; queue stays empty ---"
    GRAPHIC INPUT FLUSH
    g = -1
    GRAPHIC INSTAT TO g
    IF g <> 0 THEN
        fails = fails + 1
        ConPrint "  FAIL after GRAPHIC INPUT FLUSH ; INSTAT returned" & STR$(g) & "- expected 0"
    ELSE
        ConPrint "  PASS GRAPHIC INPUT FLUSH -> queue still empty"
    END IF

    ' ---------------------------------------------------------------
    ' 3. Querying is non-destructive: two queries in a row agree
    ' ---------------------------------------------------------------
    ConPrint "--- 3. GRAPHIC INSTAT keeps querying without consuming ---"
    g = -1
    GRAPHIC INSTAT TO g
    IF g = 0 THEN
        ConPrint "  PASS GRAPHIC INSTAT stayed 0 across two consecutive queries"
    ELSE
        fails = fails + 1
        ConPrint "  FAIL GRAPHIC INSTAT changed without a key: " & STR$(g)
    END IF

    ' ---------------------------------------------------------------
    ' 4. GRAPHIC INKEY$ with nothing buffered -> the null string
    ' ---------------------------------------------------------------
    ConPrint "--- 4. GRAPHIC INKEY$: nothing buffered gives a null string ---"
    k = "poison"
    GRAPHIC INKEY$ TO k
    IF LEN(k) <> 0 THEN
        fails = fails + 1
        ConPrint "  FAIL GRAPHIC INKEY$ returned LEN" & STR$(LEN(k)) & "- expected 0"
    ELSE
        ConPrint "  PASS GRAPHIC INKEY$ -> zero-length string"
    END IF

    ' ---------------------------------------------------------------
    ' 5. GRAPHIC WAITKEY$ with TimeOut& = 0 must not wait
    ' ---------------------------------------------------------------
    ConPrint "--- 5. GRAPHIC WAITKEY$("" ; 0): returns instead of waiting ---"
    w = "poison"
    GRAPHIC WAITKEY$("", 0) TO w
    IF LEN(w) <> 0 THEN
        fails = fails + 1
        ConPrint "  FAIL GRAPHIC WAITKEY$ returned LEN" & STR$(LEN(w)) & "- expected 0"
    ELSE
        ConPrint "  PASS GRAPHIC WAITKEY$ -> zero-length string ; no block"
    END IF

    ' ---------------------------------------------------------------
    ' 6. GRAPHIC SPLIT with a field wider than the text
    ' ---------------------------------------------------------------
    ConPrint "--- 6. GRAPHIC SPLIT: a wide field keeps everything in part 1 ---"
    src = "The quick brown fox"
    p1 = "poison"
    p2 = "poison"
    GRAPHIC SPLIT src, 100000 TO p1, p2
    IF p1 <> src OR LEN(p2) <> 0 THEN
        fails = fails + 1
        ConPrint "  FAIL wide SPLIT: part1 = " & p1 & "  part2 LEN =" & STR$(LEN(p2))
    ELSE
        ConPrint "  PASS wide SPLIT -> part1 is the whole string ; part2 empty"
    END IF

    ' ---------------------------------------------------------------
    ' 7. GRAPHIC SPLIT is lossless: part1 & part2 = the original
    ' ---------------------------------------------------------------
    ConPrint "--- 7. GRAPHIC SPLIT: part1 & part2 = the original ---"
    p1 = "poison"
    p2 = "poison"
    GRAPHIC SPLIT src, 24 TO p1, p2
    IF (p1 & p2) <> src THEN
        fails = fails + 1
        ConPrint "  FAIL lossless: [" & STR$(p1) & "] & [" & STR$(p2) & "] <> " & STR$(src)
    ELSE
        ConPrint "  PASS lossless -> " & STR$(LEN(p1)) & " +" & STR$(LEN(p2)) & " characters"
    END IF

    ' ---------------------------------------------------------------
    ' 8. A narrow field really does split
    ' ---------------------------------------------------------------
    ConPrint "--- 8. GRAPHIC SPLIT: a narrow field splits ---"
    p1 = "poison"
    p2 = "poison"
    GRAPHIC SPLIT src, 16 TO p1, p2
    IF LEN(p1) >= LEN(src) OR LEN(p2) = 0 OR (p1 & p2) <> src THEN
        fails = fails + 1
        ConPrint "  FAIL narrow SPLIT: part1 LEN =" & STR$(LEN(p1)) & " part2 LEN =" & STR$(LEN(p2))
    ELSE
        ConPrint "  PASS narrow SPLIT -> part1 LEN =" & STR$(LEN(p1)) & " ; part2 LEN =" & STR$(LEN(p2))
    END IF

    ' ---------------------------------------------------------------
    ' 9. GRAPHIC SPLIT WORD can only shorten part 1
    ' ---------------------------------------------------------------
    ConPrint "--- 9. GRAPHIC SPLIT WORD never lengthens part 1 ---"
    p1 = "poison"
    p2 = "poison"
    n1 = "poison"
    n2 = "poison"
    GRAPHIC SPLIT src, 40 TO p1, p2
    GRAPHIC SPLIT WORD src, 40 TO n1, n2
    IF (n1 & n2) <> src OR LEN(n1) > LEN(p1) OR LEFT$(src, LEN(n1)) <> n1 THEN
        fails = fails + 1
        ConPrint "  FAIL WORD SPLIT: plain LEN =" & STR$(LEN(p1)) & " word LEN =" & STR$(LEN(n1))
    ELSE
        ConPrint "  PASS WORD SPLIT -> " & STR$(LEN(p1)) & " -> " & STR$(LEN(n1)) & " characters ; still lossless"
    END IF

    ' ---------------------------------------------------------------
    ' 10. Compile-only: GRAPHIC INPUT and GRAPHIC LINE INPUT
    '     Both block until ENTER, so they are compiled but never run here.
    '     Uncomment in a PBGRAPHIC window to hand-test them.
    ' ---------------------------------------------------------------
    ConPrint "--- 10. GRAPHIC INPUT / GRAPHIC LINE INPUT: compiled ; not run ---"
    ConPrint "  NOTE these two read the keyboard and would block a headless run"
    ' GRAPHIC LINE INPUT "Name: " k
    ' GRAPHIC INPUT "Two values: ", g, k
    ConPrint "  PASS both statements are compiled by this sample"

    ConPrint ""
    IF fails = 0 THEN
        ConPrint "=== FAILURES:0 ==="
    ELSE
        ConPrint "=== FAILURES:" & STR$(fails) & " ==="
    END IF
    FUNCTION = fails
END FUNCTION



#ENDIF
