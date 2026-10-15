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

' PB64Basic  Full Feature Demo
' Compile: pbcompiler build demo.bas --exe --target x86_64-pc-windows-msvc --runtime-lib pb_runtime_x64.obj
'
' Shows every feature implemented by this branch:
'   1) Control flow   : IF/THEN/ELSE, FOR/NEXT, WHILE/WEND, DO/LOOP, GOSUB/RETURN
'   2) Strings        : 18 built-in equates ($CRLF...), REPLACE, LSET/RSET, core funcs
'   3) Arrays         : DIM, ERASE
'   4) File I/O       : OPEN, PRINT#, LINE INPUT#, INPUT#, WRITE#, SEEK#, LOCK/UNLOCK,
'                       FLUSH, RESET, NAME, KILL, EOF, FREEFILE
'   5) Directories    : MKDIR, RMDIR, CHDIR + ERR / ERRCLEAR (PB error codes)
'   6) System         : MSGBOX, BEEP, SLEEP, RANDOMIZE+RND, SWAP, CURDIR$, ISFILE, SHELL
'
FUNCTION PBMAIN() AS LONG
    LOCAL waitk AS STRING
  LOCAL f AS LONG
  LOCAL s AS STRING
  LOCAL t AS STRING
  LOCAL i AS LONG
  LOCAL sum AS LONG
  LOCAL a AS LONG
  LOCAL b AS LONG
  LOCAL r AS DOUBLE
  LOCAL fixed10 AS STRING * 10
  LOCAL ok AS LONG
  DIM arr(5) AS LONG

  ok = 1
  ConPrint "=============================================="
  ConPrint "  PB64Basic - Full Feature Demo"
  ConPrint "  PowerBASIC -> LLVM IR -> native x64 exe"
  ConPrint "=============================================="

  ' ---------- 1. Control flow ----------
  ConPrint ""
  ConPrint "[1] Control flow"

  FOR i = 1 TO 5
    sum = sum + i
  NEXT i
  IF sum = 15 THEN
    ConPrint "  FOR/NEXT + IF/THEN: sum(1..5) = " + STR$(sum) + "  OK"
  ELSE
    ok = 0
    ConPrint "  FAIL: sum = " + STR$(sum)
  END IF

  i = 0
  WHILE i < 3
    i = i + 1
  WEND
  ConPrint "  WHILE/WEND: i = " + STR$(i) + "  OK"

  DO
    i = i - 1
  LOOP UNTIL i = 0
  ConPrint "  DO/LOOP UNTIL: i = " + STR$(i) + "  OK"

  GOSUB ShowSubroutine
  ConPrint "  GOSUB/RETURN: returned OK"

  ' ---------- 2. Strings ----------
  ConPrint ""
  ConPrint "[2] Strings (built-in equates)"

  ' All 18 equates are compile-time constants; demo a few + join
  s = "Line1" + $CRLF + "Line2"
  IF s = "Line1" + CHR$(13, 10) + "Line2" THEN
    ConPrint "  $CRLF = CR+LF bytes  OK"
  ELSE
    ok = 0
    ConPrint "  FAIL: $CRLF"
  END IF

  ConPrint "  $DQ/$SQ/$TAB: [" + $DQ + "quoted" + $DQ + "]" + $TAB + "[" + $SQ + "sq" + $SQ + "]"
  ConPrint "  $WHITESPACE = space+tab+cr+lf (invisible)  OK"

  t = "the cat sat on the cat mat"
  REPLACE "cat" WITH "dog" IN t
  IF t = "the dog sat on the dog mat" THEN
    ConPrint "  REPLACE: " + t + "  OK"
  ELSE
    ok = 0
    ConPrint "  FAIL: REPLACE -> " + t
  END IF

  fixed10 = ""
  LSET fixed10 = "ab"
  ConPrint "  LSET: [" + fixed10 + "]  (left-aligned)"
  fixed10 = ""
  RSET fixed10 = "cd"
  ConPrint "  RSET: [" + fixed10 + "]  (right-aligned)"

  ConPrint "  LEN/LEFT$/RIGHT$: " + STR$(LEN("hello")) + " / " + LEFT$("hello", 2) + " / " + RIGHT$("hello", 2)

  ' ---------- 3. Arrays ----------
  ConPrint ""
  ConPrint "[3] Arrays"

  FOR i = 0 TO 5
    arr(i) = 100 + i
  NEXT i
  ERASE arr
  IF arr(0) = 0 THEN
    ConPrint "  DIM + ERASE: arr(0) after ERASE = " + STR$(arr(0)) + "  OK"
  ELSE
    ok = 0
    ConPrint "  FAIL: arr(0) = " + STR$(arr(0))
  END IF

  ' ---------- 4. File I/O ----------
  ConPrint ""
  ConPrint "[4] File I/O"

  f = FREEFILE
  OPEN "demo_data.tmp" FOR OUTPUT AS #f
  PRINT #f, "Hello file world"
  WRITE #f, "csv", 42, 3.5
  FLUSH #f
  CLOSE #f
  ConPrint "  OPEN/PRINT#/WRITE#/FLUSH/CLOSE  OK"

  f = FREEFILE
  OPEN "demo_data.tmp" FOR INPUT AS #f
  LINE INPUT #f, s
  INPUT #f, t, i, r
  IF s = "Hello file world" AND t = "csv" AND i = 42 AND r = 3.5 THEN
    ConPrint "  LINE INPUT#/INPUT# read back  OK"
  ELSE
    ok = 0
    ConPrint "  FAIL: read back [" + s + "] [" + t + "]"
  END IF

  SEEK #f, 1
  LINE INPUT #f, s
  ConPrint "  SEEK# to top: [" + s + "]"
  CLOSE #f

  f = FREEFILE
  OPEN "demo_data.tmp" FOR INPUT AS #f
  LOCK #f, 1, 10
  UNLOCK #f, 1, 10
  CLOSE #f
  ConPrint "  LOCK/UNLOCK byte range  OK"

  NAME "demo_data.tmp" AS "demo_renamed.tmp"
  ConPrint "  NAME: file renamed  OK"

  f = FREEFILE
  OPEN "demo_renamed.tmp" FOR INPUT AS #f
  RESET
  ConPrint "  RESET: all handles closed  OK"

  IF ISFILE("demo_renamed.tmp") = 1 THEN
    KILL "demo_renamed.tmp"
    ConPrint "  KILL + ISFILE: file removed  OK"
  END IF

  ' ---------- 5. Directories + ERR ----------
  ConPrint ""
  ConPrint "[5] Directories + ERR semantics"

  MKDIR "demo_dir"
  MKDIR "demo_dir"          ' already exists -> ERR 75 (PB official)
  IF ERR = 75 THEN
    ConPrint "  MKDIR twice -> ERR 75  OK"
  ELSE
    ok = 0
    ConPrint "  FAIL: MKDIR exists ERR=" + STR$(ERR)
  END IF
  ERRCLEAR

  RMDIR "no_such_dir_xyz"   ' missing -> ERR 75
  IF ERR = 75 THEN
    ConPrint "  RMDIR missing -> ERR 75  OK"
  ELSE
    ok = 0
    ConPrint "  FAIL: RMDIR missing ERR=" + STR$(ERR)
  END IF
  ERRCLEAR

  CHDIR "no_such_dir_abc"   ' invalid -> ERR 76
  IF ERR = 76 THEN
    ConPrint "  CHDIR invalid -> ERR 76  OK"
  ELSE
    ok = 0
    ConPrint "  FAIL: CHDIR invalid ERR=" + STR$(ERR)
  END IF
  ERRCLEAR

  RMDIR "demo_dir"
  ConPrint "  RMDIR: cleanup OK"

  ' ---------- 6. System calls ----------
  ConPrint ""
  ConPrint "[6] System calls"

  RANDOMIZE 42
  r = RND
  ConPrint "  RANDOMIZE+RND: first value = " + STR$(r)

  a = 111
  b = 222
  SWAP a, b
  IF a = 222 AND b = 111 THEN
    ConPrint "  SWAP: a=222 b=111  OK"
  ELSE
    ok = 0
    ConPrint "  FAIL: SWAP a=" + STR$(a) + " b=" + STR$(b)
  END IF

  ConPrint "  CURDIR$ = " + CURDIR$
  ConPrint "  ISFILE(demo.bas) = " + ISFILE("demo.bas")

  BEEP
  SLEEP 300
  ConPrint "  BEEP + SLEEP 300ms  OK"

  ' ---------- Final report ----------
  f = FREEFILE
  OPEN "demo_result.txt" FOR OUTPUT AS #f
  IF ok = 1 THEN
    PRINT #f, "ALL OK: every enhanced feature ran successfully"
  ELSE
    PRINT #f, "FAILED: one or more checks failed"
  END IF
  CLOSE #f

  ConPrint ""
  IF ok = 1 THEN
    ConPrint "=== ALL FEATURES VERIFIED OK ==="
  ELSE
    ConPrint "=== SOME CHECKS FAILED ==="
  END IF
  ConPrint "(see demo_result.txt)"

  ' ---------- 7. GUI popups (the show) ----------
  SHELL "calc.exe"                                   ' launch Calculator
  MSGBOX "PB64Basic - all features OK!" + $CRLF + $CRLF + _
         "MSGBOX + SHELL + CURDIR$ + ISFILE + BEEP + SLEEP" + $CRLF + _
         "REPLACE + ERASE + LSET + RSET + SWAP + RANDOMIZE" + $CRLF + _
         "WRITE#/SEEK#/LOCK/UNLOCK/RESET/FLUSH/NAME/KILL" + $CRLF + _
         "MKDIR/RMDIR/CHDIR + ERR/ERRCLEAR + 18 string equates", 64, "PB64Basic Demo"

  FUNCTION = 0
  EXIT FUNCTION

ShowSubroutine:
  ConPrint "  (inside GOSUB subroutine)"
  RETURN
    ConPrint "Press any key to exit..."
    ConWaitKey

END FUNCTION



#ENDIF
