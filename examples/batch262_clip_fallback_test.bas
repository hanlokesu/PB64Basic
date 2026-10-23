' batch262_clip_fallback_test.bas - CLIPBOARD GET OEMTEXT/UNICODE fallback
' (CHW CLIPBOARD_statement.htm: "Windows automatically converts string data
' between TEXT, OEMTEXT, and UNICODE").  TEXT-only clipboard is read back as
' OEMTEXT (ANSI passthrough) and UNICODE (widened UTF-16LE).  ClipResult must
' be -1 on success and 0 on failure (PBWin10 probe).  The clipboard is a
' shared global resource - OpenClipboard can transiently fail when another
' process holds it - so each assertion group is retried up to 5 times.
#COMPILE EXE
FUNCTION PBMAIN() AS LONG
    LOCAL t AS STRING
    LOCAL r, i AS LONG
    LOCAL ok AS LONG
    OPEN "batch262_out.txt" FOR OUTPUT AS #1

    ' 1) TEXT -> OEMTEXT fallback + ClipResult -1
    ok = 0
    FOR i = 1 TO 5
        CLIPBOARD RESET
        CLIPBOARD SET TEXT "abc" TO r
        t = ""
        CLIPBOARD GET OEMTEXT TO t, r
        IF LEN(t) = 3 AND r = -1 THEN
            ok = -1
            EXIT FOR
        END IF
    NEXT i
    PRINT #1, "oem:" + STR$(ok) + ":" + STR$(LEN(t)) + ":" + STR$(r)

    ' 2) TEXT -> UNICODE fallback (UTF-16LE = 6 bytes for "abc") + ClipResult -1
    ok = 0
    FOR i = 1 TO 5
        CLIPBOARD RESET
        CLIPBOARD SET TEXT "abc" TO r
        t = ""
        CLIPBOARD GET UNICODE TO t, r
        IF LEN(t) = 6 AND r = -1 THEN
            ok = -1
            EXIT FOR
        END IF
    NEXT i
    PRINT #1, "uni:" + STR$(ok) + ":" + STR$(LEN(t)) + ":" + STR$(r)

    ' 3) empty clipboard -> both empty, ClipResult 0
    ok = 0
    FOR i = 1 TO 5
        CLIPBOARD RESET
        t = "x"
        CLIPBOARD GET OEMTEXT TO t, r
        IF LEN(t) = 0 AND r = 0 THEN
            t = "x"
            CLIPBOARD GET UNICODE TO t, r
            IF LEN(t) = 0 AND r = 0 THEN
                ok = -1
                EXIT FOR
            END IF
        END IF
    NEXT i
    PRINT #1, "empty:" + STR$(ok)

    CLOSE #1
    FUNCTION = 0
END FUNCTION
