' batch258e_test.bas - CLIPBOARD SET/GET TEXT/OEMTEXT/UNICODE + RESET + ClipResult
' The Windows clipboard is a shared system resource: OpenClipboard can fail
' transiently when another process owns it.  Every SET/GET is retried up to
' 5 times to keep the assertion stable across machine states.
#COMPILE EXE
FUNCTION PBMAIN() AS LONG
    LOCAL h, rc, i AS LONG
    LOCAL s, u AS STRING
    LOCAL ok AS LONG
    ok = 1
    h = FREEFILE
    OPEN "batch258e_out.txt" FOR OUTPUT AS #h
    FOR i = 1 TO 5
        CLIPBOARD SET TEXT "hello", rc
        IF rc = -1 THEN EXIT FOR
    NEXT i
    IF rc <> -1 THEN ok = 0 : PRINT #h, "A1 SET_TEXT FAIL"
    s = ""
    FOR i = 1 TO 5
        CLIPBOARD GET TEXT TO s
        IF s <> "" THEN EXIT FOR
    NEXT i
    IF s <> "hello" THEN ok = 0 : PRINT #h, "A2 GET_TEXT FAIL"
    FOR i = 1 TO 5
        CLIPBOARD RESET, rc
        IF rc = -1 THEN EXIT FOR
    NEXT i
    IF rc <> -1 THEN ok = 0 : PRINT #h, "A3 RESET FAIL"
    s = "x"
    FOR i = 1 TO 5
        CLIPBOARD GET TEXT TO s
        IF s = "" THEN EXIT FOR
    NEXT i
    IF s <> "" THEN ok = 0 : PRINT #h, "A4 GET_EMPTY FAIL"
    FOR i = 1 TO 5
        CLIPBOARD SET TEXT "abc", rc
        IF rc = -1 THEN EXIT FOR
    NEXT i
    IF rc <> -1 THEN ok = 0 : PRINT #h, "A5 SET_ABC FAIL"
    FOR i = 1 TO 5
        CLIPBOARD SET OEMTEXT "oem", rc
        IF rc = -1 THEN EXIT FOR
    NEXT i
    IF rc <> -1 THEN ok = 0 : PRINT #h, "A6 SET_OEM FAIL"
    s = ""
    FOR i = 1 TO 5
        CLIPBOARD GET OEMTEXT TO s
        IF s <> "" THEN EXIT FOR
    NEXT i
    IF s <> "oem" THEN ok = 0 : PRINT #h, "A7 GET_OEM FAIL"
    u = CHR$(&H41) & CHR$(&H00) & CHR$(&H42) & CHR$(&H00)   ' "AB" wide
    FOR i = 1 TO 5
        CLIPBOARD SET UNICODE u, rc
        IF rc = -1 THEN EXIT FOR
    NEXT i
    IF rc <> -1 THEN ok = 0 : PRINT #h, "A8 SET_UNI FAIL"
    IF ok THEN PRINT #h, "batch258e test ok" ELSE PRINT #h, "batch258e test FAIL"
    CLOSE #h
END FUNCTION
