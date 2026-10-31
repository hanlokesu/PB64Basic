#COMPILE EXE
#DIM ALL
' _b307_mdarr2_probe.bas - isolate: member array element ASSIGN (no VARPTR).
TYPE T1
    a(3) AS LONG
    b(2,3) AS LONG
END TYPE
TYPE T2
    x AS LONG
    s AS STRING * 8
    arr(1 TO 3) AS LONG
END TYPE
FUNCTION PBMAIN() AS LONG
    LOCAL t AS T1
    LOCAL u AS T2
    LOCAL ok1 AS LONG
    LOCAL ok2 AS LONG
    LOCAL ok3 AS LONG
    t.a(0) = 11
    t.a(3) = 14
    ok1 = t.a(0) + t.a(3)
    t.b(0,0) = 21
    t.b(2,3) = 24
    ok2 = t.b(0,0) + t.b(2,3)
    u.x = 31
    u.s = "ABCDEFGH"
    u.arr(1) = 41
    u.arr(3) = 44
    ok3 = u.x + u.arr(1) + u.arr(3)
    OPEN "b307_mdarr2_out.txt" FOR OUTPUT AS #1
    PRINT #1, "ok1=" & STR$(ok1) & " ok2=" & STR$(ok2) & " ok3=" & STR$(ok3)
    CLOSE #1
    FUNCTION = 0
END FUNCTION
