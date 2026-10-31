' batch257_test.bas - CHW A-family probe: ARRAY SCAN (index/FOR/FROM/
' COLLATE UCASE/rel-op), ARRAY INSERT FOR, ARRAY DELETE FOR+fill,
' ARRAY SORT range+TAGARRAY+DESCEND, ASC(pos).  GBK+CRLF.
' Expected values = PBWin10 measured behavior (_b257_probe.bas MATCH).
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
FUNCTION PBMAIN() AS LONG
    LOCAL i AS LONG
    LOCAL r AS LONG
    LOCAL ok AS LONG
    ok = 0
    ' --- ARRAY SCAN numeric (PBWin measured) ---
    DIM a(1 TO 8) AS LONG
    a(1)=10: a(2)=20: a(3)=30: a(4)=40: a(5)=20: a(6)=60: a(7)=20: a(8)=80
    ARRAY SCAN a(), =20, TO r
    IF r = 2 THEN ok = ok OR 1
    ARRAY SCAN a(3), =20, TO r
    IF r = 3 THEN ok = ok OR 2
    ARRAY SCAN a(3) FOR 4, =20, TO r
    IF r = 3 THEN ok = ok OR 4
    ARRAY SCAN a(), >50, TO r
    IF r = 6 THEN ok = ok OR 8
    ARRAY SCAN a(), <=15, TO r
    IF r = 1 THEN ok = ok OR 16
    ARRAY SCAN a(), <>20, TO r
    IF r = 1 THEN ok = ok OR 32
    ARRAY SCAN a(), =99, TO r
    IF r = 0 THEN ok = ok OR 64
    ' --- ARRAY SCAN string + FROM/COLLATE UCASE ---
    DIM s(1 TO 5) AS STRING
    s(1)="apple": s(2)="Banana": s(3)="cherry": s(4)="APPLE": s(5)="fig"
    ARRAY SCAN s(), ="banana", TO r
    IF r = 0 THEN ok = ok OR 128
    ARRAY SCAN s(), COLLATE UCASE, ="BANANA", TO r
    IF r = 2 THEN ok = ok OR 256
    ARRAY SCAN s(), FROM 2 TO 4, COLLATE UCASE, ="ANA", TO r
    IF r = 2 THEN ok = ok OR 512
    ARRAY SCAN s(3), COLLATE UCASE, ="APPLE", TO r
    IF r = 2 THEN ok = ok OR 1024
    ' --- ARRAY INSERT FOR ---
    DIM ins(1 TO 6) AS LONG
    FOR i=1 TO 6: ins(i)=i*10: NEXT
    ARRAY INSERT ins(3) FOR 2, 999
    IF ins(1)=10 AND ins(2)=20 AND ins(3)=999 AND ins(4)=30 AND ins(5)=50 AND ins(6)=60 THEN ok = ok OR 2048
    DIM ins2(1 TO 5) AS LONG
    FOR i=1 TO 5: ins2(i)=i: NEXT
    ARRAY INSERT ins2(2), 77
    IF ins2(1)=1 AND ins2(2)=77 AND ins2(3)=2 AND ins2(4)=3 AND ins2(5)=4 THEN ok = ok OR 4096
    ' --- ARRAY DELETE FOR + fill ---
    DIM del(1 TO 6) AS LONG
    FOR i=1 TO 6: del(i)=i*10: NEXT
    ARRAY DELETE del(3) FOR 2
    IF del(1)=10 AND del(2)=20 AND del(3)=40 AND del(4)=0 AND del(5)=50 AND del(6)=60 THEN ok = ok OR 8192
    ARRAY DELETE del(1), 55
    IF del(1)=20 AND del(2)=40 AND del(3)=0 AND del(4)=50 AND del(5)=60 AND del(6)=55 THEN ok = ok OR 16384
    ' --- ARRAY SORT range + tag + descend ---
    DIM sor(1 TO 6) AS LONG
    sor(1)=50: sor(2)=10: sor(3)=40: sor(4)=20: sor(5)=60: sor(6)=30
    DIM tag(1 TO 6) AS LONG
    FOR i=1 TO 6: tag(i)=i: NEXT
    ARRAY SORT sor(2) FOR 3, TAGARRAY tag(), DESCEND
    IF sor(1)=50 AND sor(2)=40 AND sor(3)=20 AND sor(4)=10 AND sor(5)=60 AND sor(6)=30 THEN ok = ok OR 32768
    IF tag(1)=1 AND tag(2)=3 AND tag(3)=4 AND tag(4)=2 AND tag(5)=5 AND tag(6)=6 THEN ok = ok OR 65536
    ARRAY SORT sor(), ASCEND
    IF sor(1)=10 AND sor(2)=20 AND sor(3)=30 AND sor(4)=40 AND sor(5)=50 AND sor(6)=60 THEN ok = ok OR 131072
    ' --- ASC function pos ---
    LOCAL c AS LONG
    c = ASC("hello")
    IF c = 104 THEN ok = ok OR 262144
    c = ASC("hello", 2)
    IF c = 101 THEN ok = ok OR 524288
    c = ASC("hello", -1)
    IF c = 111 THEN ok = ok OR 1048576
    c = ASC("hello", 0)
    IF c = -1 THEN ok = ok OR 2097152
    c = ASC("hello", 99)
    IF c = -1 THEN ok = ok OR 4194304
    c = ASC("")
    IF c = -1 THEN ok = ok OR 8388608
    ConPrint "batch257 ok=" & STR$(ok)
    ' All 24 bits set = every assertion passed (values PBWin10-measured).  Return
    ' 0 only then, so the headless harness sees exit 0 on a clean run.
    IF ok = 16777215 THEN FUNCTION = 0 ELSE FUNCTION = 1
END FUNCTION
