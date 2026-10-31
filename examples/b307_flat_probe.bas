#COMPILE EXE
#DIM ALL
' _b307_flat_probe.bas - standalone multi-dim array support check.
FUNCTION PBMAIN() AS LONG
    LOCAL arr(2,3) AS LONG
    LOCAL i AS LONG
    LOCAL j AS LONG
    LOCAL n AS LONG
    n = 0
    FOR i = 0 TO 2
        FOR j = 0 TO 3
            arr(i,j) = n
            n = n + 1
        NEXT j
    NEXT i
    OPEN "b307_flat_out.txt" FOR OUTPUT AS #1
    PRINT #1, "a00=" & STR$(arr(0,0)) & " a10=" & STR$(arr(1,0)) & " a01=" & STR$(arr(0,1)) & _
               " a23=" & STR$(arr(2,3)) & " n=" & STR$(n)
    CLOSE #1
    FUNCTION = 0
END FUNCTION
