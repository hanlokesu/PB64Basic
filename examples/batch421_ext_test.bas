#COMPILE EXE
#DIM ALL

'=====================================================================
' PB64Basic Enhanced - batch 421 regression test
'---------------------------------------------------------------------
' Locks the 10-byte x87 EXT implementation and PBWin-compatible layout:
'   - LEN(EXT) = 10 and decimal literals land exactly in 80 bits;
'   - MKE$/CVE$ produce and consume the 10-byte x87 image;
'   - EXT arrays use a 10-byte stride and UDT fields keep PBWin offsets;
'   - consecutive global EXT scalars are 12 bytes apart;
'   - BYVAL EXT parameters and EXT results keep full precision;
'   - VAL() parses into EXT rather than rounding through DOUBLE.
'
' Expected output:
'   len/str -> 10 / .333333333333333333 / .3333333333333333 / .333333333333333
'   mke/cve -> 10 192 255 63 / 1.5
'   array -> 10 10 / 1.5 2.5 -3.25
'   udt -> 24 / 10 14
'   global -> 12 / 1.5 -2.25
'   val/func -> .3333333333333333 / .333333333333333333
'   math/sqr -> 1.41421356237309505
'   math/exp -> 2.71828182845904524
'   math/log -> .693147180559945309
'   math/pow -> 1.9952623149688796
'   math/powneg -> -8
'   math/stepdiv -> .333333333333333333
'   === FAILURES: 0 ===
'   exit code 0
'=====================================================================

TYPE TX
    x AS EXT
    y AS LONG
    z AS EXT
END TYPE

TYPE TA
    a(0 TO 2) AS EXT
    b AS EXT
END TYPE

GLOBAL gx AS EXT
GLOBAL gy AS EXT

DECLARE FUNCTION AllocConsole LIB "KERNEL32.DLL" ALIAS "AllocConsole" () AS LONG
DECLARE FUNCTION AttachConsole LIB "KERNEL32.DLL" ALIAS "AttachConsole" (BYVAL dwProcessId AS DWORD) AS LONG
DECLARE FUNCTION GetStdHandle LIB "KERNEL32.DLL" ALIAS "GetStdHandle" (BYVAL nStdHandle AS DWORD) AS LONG
DECLARE FUNCTION WriteFile LIB "KERNEL32.DLL" ALIAS "WriteFile" (BYVAL hFile AS LONG, lpBuffer AS ANY, BYVAL nBytesToWrite AS DWORD, lpBytesWritten AS DWORD, BYVAL lpOverlapped AS LONG) AS LONG

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

FUNCTION Third(BYVAL x AS EXT) AS EXT
    FUNCTION = x / 3
END FUNCTION

FUNCTION PBMAIN () AS LONG
    LOCAL fail AS LONG
    LOCAL e1 AS EXT
    LOCAL e2 AS EXT
    LOCAL mk AS STRING
    LOCAL arr(0 TO 2) AS EXT
    LOCAL t AS TX
    LOCAL ta AS TA
    LOCAL f AS EXT
    LOCAL v AS EXT

    e1 = 1 / 3
    e2 = 0.3333333333333333

    ConPrint "len/str -> " & STR$(LEN(e1)) & " / " & STR$(e1, 18) & " / " & STR$(e2, 18) & " / " & STR$(e2) & CHR$(13, 10)
    IF LEN(e1) <> 10 THEN fail = fail + 1
    IF STR$(e1, 18) <> " .333333333333333333" THEN fail = fail + 1
    IF STR$(e2, 18) <> " .3333333333333333" THEN fail = fail + 1
    IF STR$(e2) <> " .333333333333333" THEN fail = fail + 1

    mk = MKE$(1.5)
    ConPrint "mke/cve -> " & STR$(LEN(mk)) & " " & STR$(ASC(mk, 8)) & " " & STR$(ASC(mk, 9)) & " " & STR$(ASC(mk, 10)) & " / " & STR$(CVE$(mk)) & CHR$(13, 10)
    IF LEN(mk) <> 10 THEN fail = fail + 1
    IF ASC(mk, 8) <> 192 OR ASC(mk, 9) <> 255 OR ASC(mk, 10) <> 63 THEN fail = fail + 1
    IF ABS(CVE$(mk) - 1.5) > 0.000000000000000001 THEN fail = fail + 1

    arr(0) = 1.5 : arr(1) = 2.5 : arr(2) = -3.25
    ConPrint "array -> " & STR$(VARPTR(arr(1)) - VARPTR(arr(0))) & " " & STR$(VARPTR(arr(2)) - VARPTR(arr(1))) & " / " & STR$(arr(0)) & " " & STR$(arr(1)) & " " & STR$(arr(2)) & CHR$(13, 10)
    IF VARPTR(arr(1)) - VARPTR(arr(0)) <> 10 THEN fail = fail + 1
    IF VARPTR(arr(2)) - VARPTR(arr(1)) <> 10 THEN fail = fail + 1

    t.x = 1.5 : t.y = 7 : t.z = -2.25
    ConPrint "udt -> " & STR$(LEN(t)) & " / " & STR$(VARPTR(t.y) - VARPTR(t.x)) & " " & STR$(VARPTR(t.z) - VARPTR(t.x)) & CHR$(13, 10)
    IF LEN(t) <> 24 THEN fail = fail + 1
    IF VARPTR(t.y) - VARPTR(t.x) <> 10 THEN fail = fail + 1
    IF VARPTR(t.z) - VARPTR(t.x) <> 14 THEN fail = fail + 1

    ta.a(0) = 1.5 : ta.a(1) = 2.5 : ta.a(2) = -3.25 : ta.b = 4.5
    ConPrint "udtarr -> " & STR$(LEN(ta)) & " / " & STR$(VARPTR(ta.a(1)) - VARPTR(ta.a(0))) & " " & STR$(VARPTR(ta.b) - VARPTR(ta.a(0))) & " / " & STR$(ta.a(0)) & " " & STR$(ta.a(1)) & " " & STR$(ta.a(2)) & " " & STR$(ta.b) & CHR$(13, 10)
    IF LEN(ta) <> 40 THEN fail = fail + 1
    IF VARPTR(ta.a(1)) - VARPTR(ta.a(0)) <> 10 THEN fail = fail + 1
    IF VARPTR(ta.b) - VARPTR(ta.a(0)) <> 30 THEN fail = fail + 1

    gx = 1.5 : gy = -2.25
    ConPrint "global -> " & STR$(VARPTR(gy) - VARPTR(gx)) & " / " & STR$(gx) & " " & STR$(gy) & CHR$(13, 10)
    IF VARPTR(gy) - VARPTR(gx) <> 12 THEN fail = fail + 1

    v = VAL("0.3333333333333333")
    f = Third(1)
    ConPrint "val/func -> " & STR$(v, 18) & " / " & STR$(f, 18) & CHR$(13, 10)
    IF STR$(v, 18) <> " .3333333333333333" THEN fail = fail + 1
    IF STR$(f, 18) <> " .333333333333333333" THEN fail = fail + 1

    f = SQR(2)
    ConPrint "math/sqr -> " & STR$(f, 18) & CHR$(13, 10)
    IF STR$(f, 18) <> " 1.41421356237309505" THEN fail = fail + 1
    f = EXP(1)
    ConPrint "math/exp -> " & STR$(f, 18) & CHR$(13, 10)
    IF STR$(f, 18) <> " 2.71828182845904524" THEN fail = fail + 1
    f = LOG(2)
    ConPrint "math/log -> " & STR$(f, 18) & CHR$(13, 10)
    IF STR$(f, 18) <> " .693147180559945309" THEN fail = fail + 1
    f = 10 ^ 0.3##
    ConPrint "math/pow -> " & STR$(f, 18) & CHR$(13, 10)
    IF STR$(f, 18) <> " 1.9952623149688796" THEN fail = fail + 1
    f = 10 ^ 0.3
    ConPrint "math/powsingle -> " & STR$(f, 18) & CHR$(13, 10)
    IF STR$(f, 18) <> " 1.99526236973674247" THEN fail = fail + 1
    f = (-2) ^ 3
    ConPrint "math/powneg -> " & STR$(f) & CHR$(13, 10)
    IF STR$(f) <> "-8" THEN fail = fail + 1
    f = 1
    f = f / 3
    ConPrint "math/stepdiv -> " & STR$(f, 18) & CHR$(13, 10)
    IF STR$(f, 18) <> " .333333333333333333" THEN fail = fail + 1

    ConPrint CHR$(13, 10)
    ConPrint "=== FAILURES: " & STR$(fail) & " ===" & CHR$(13, 10)
    FUNCTION = fail
END FUNCTION
