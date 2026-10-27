#COMPILE EXE
#DIM ALL
FUNCTION PBMAIN() AS LONG
    LOCAL o1 AS IDISPATCH
    LOCAL o2 AS IDISPATCH
    LOCAL o3 AS IDISPATCH
    LOCAL o4 AS IDISPATCH
    LOCAL o5 AS IDISPATCH
    LOCAL r1 AS LONG, r2 AS LONG, r3 AS LONG, r4 AS LONG, r5 AS LONG
    LOCAL cid AS STRING
    LOCAL failures AS LONG
    failures = 0
    o1 = NEWCOM "Shell.Application"
    IF ISOBJECT(o1) THEN r1 = -1 ELSE r1 = 0
    o2 = GETCOM "Shell.Application"
    IF ISOBJECT(o2) THEN r2 = -1 ELSE r2 = 0
    o3 = ANYCOM "Shell.Application"
    IF ISOBJECT(o3) THEN r3 = -1 ELSE r3 = 0
    cid = CLSID$("Shell.Application")
    o4 = NEWCOM CLSID cid
    IF ISOBJECT(o4) THEN r4 = -1 ELSE r4 = 0
    o5 = NEWCOM "No.Such.Prog.12345"
    IF ISOBJECT(o5) THEN r5 = -1 ELSE r5 = 0
    OPEN "b284_out.txt" FOR OUTPUT AS #1
    PRINT #1, "NEWCOM   = "; r1
    PRINT #1, "GETCOM   = "; r2
    PRINT #1, "ANYCOM   = "; r3
    PRINT #1, "NEWCLSID = "; r4
    PRINT #1, "FAIL     = "; r5
    CLOSE #1
    IF r1 <> -1 OR r3 <> -1 OR r4 <> -1 OR r5 <> 0 THEN INCR failures
    FUNCTION = failures
END FUNCTION
