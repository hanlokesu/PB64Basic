' batch259_test.bas - CLSID$ function dual assertion (batch259)
' CHW: CLSID$(ProgramID$) -> 16-byte binary GUID string from HKCR; empty on
' any failure and ERR is NOT set.  WScript.Shell's CLSID is the registry
' anchor used on both sides: {72C24DD5-D70A-438B-8A42-98424B88AFB8}
#COMPILE EXE
FUNCTION PBMAIN() AS LONG
    LOCAL a, s AS STRING
    LOCAL i, h AS LONG
    LOCAL ok AS LONG
    ok = 1
    h = FREEFILE
    OPEN "batch259_out.txt" FOR OUTPUT AS #h
    a = CLSID$("WScript.Shell")
    IF LEN(a) <> 16 THEN ok = 0
    s = ""
    FOR i = 1 TO LEN(a)
        s = s & HEX$(ASC(a, i), 2)
    NEXT i
    IF s <> "D54DC2720AD78B438A4298424B88AFB8" THEN ok = 0
    a = CLSID$("nonexistent.progid.xyz")
    IF LEN(a) <> 0 THEN ok = 0
    IF ok THEN
        s = "batch259 test ok"
        PRINT #h, s
    ELSE
        s = "batch259 test FAIL"
        PRINT #h, s
    END IF
    CLOSE #h
    FUNCTION = 0
END FUNCTION
