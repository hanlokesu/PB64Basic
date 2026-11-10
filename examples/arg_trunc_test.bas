#COMPILE EXE
#DIM ALL

' Regression guard: an i64-returning built-in (ASC on WSTRING/STRING) nested
' directly into an i32 BYVAL argument must be truncated to the target width.
' On i686 cdecl the missing trunc let the i64 occupy two stack slots, shifting
' every later argument so the STRING tag was read as a bad pointer and
' dereferenced (0xC0000005). x64 passes args in registers and never showed it.
GLOBAL g_failures AS LONG

SUB CheckLong(BYVAL got AS LONG, BYVAL expected AS LONG, BYVAL tag AS STRING)
    IF got <> expected THEN
        PRINT "FAIL: " + tag + " got=" + FORMAT$(got) + " want=" + FORMAT$(expected)
        g_failures = g_failures + 1
    END IF
END SUB

FUNCTION PBMAIN() AS LONG
    LOCAL w AS WSTRING
    LOCAL s AS STRING

    w = "A" + CHR$$(0) + "B"
    s = "ABC"

    ' i64 ASC result nested straight into i32 BYVAL parameters (the crash).
    CheckLong ASC(w, 1), &H41, "wstr asc1"
    CheckLong ASC(w, 2), 0,     "wstr asc2 embedded nul"
    CheckLong ASC(w, 3), &H42, "wstr asc3"
    CheckLong ASC(s, 1), &H41, "str asc1"
    CheckLong ASC(s, 3), &H43, "str asc3"

    PRINT "FAILURES:"; g_failures
    FUNCTION = g_failures
END FUNCTION