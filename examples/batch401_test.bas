#COMPILE EXE
#CONSOLE ON
#DIM ALL

' batch 401: array parameters, BYVAL strings and numeric ? placeholders.
' Each helper exercises a path that previously lowered to a fixed-element
' pointer or rejected the sample outright.

FUNCTION SumLongs(BYREF a() AS LONG) AS LONG
    LOCAL i AS LONG
    LOCAL s AS LONG
    FOR i = LBOUND(a) TO UBOUND(a)
        s = s + a(i)
    NEXT
    FUNCTION = s
END FUNCTION

FUNCTION SumGrid(BYREF a() AS LONG) AS LONG
    LOCAL i AS LONG
    LOCAL s AS LONG
    FOR i = 1 TO 2
        s = s + a(1, i) + a(2, i)
    NEXT
    FUNCTION = s
END FUNCTION

SUB FillBytes(BYREF a() AS BYTE, BYVAL text AS STRING)
    LOCAL i AS LONG
    REDIM a(0 TO LEN(text) - 1)
    FOR i = 1 TO LEN(text)
        a(i - 1) = ASC(MID$(text, i, 1))
    NEXT
END SUB

FUNCTION ShowStr(BYVAL s AS STRING) AS LONG
    FUNCTION = LEN(s) * 100 + ASC(MID$(s, 1, 1))
END FUNCTION

SUB StoreFirst(BYREF a() AS LONG, BYVAL v AS LONG)
    a(LBOUND(a)) = v
END SUB

FUNCTION PBMAIN () AS LONG
    LOCAL fails AS LONG
    LOCAL x() AS LONG
    LOCAL y(1 TO 3) AS LONG
    LOCAL g(1 TO 2, 1 TO 2) AS LONG
    LOCAL b() AS BYTE
    LOCAL i AS LONG

    fails = 0

    REDIM x(1 TO 3)
    x(1) = 10
    x(2) = 20
    x(3) = 30
    y(1) = 1
    y(2) = 2
    y(3) = 3
    FOR i = 1 TO 4
        g(((i - 1) \ 2) + 1, ((i - 1) MOD 2) + 1) = i
    NEXT

    IF SumLongs(x()) <> 60 THEN INCR fails
    IF SumLongs(y()) <> 6 THEN INCR fails
    IF SumGrid(g()) <> 10 THEN INCR fails

    StoreFirst(y(), 99)
    IF y(1) <> 99 THEN INCR fails

    FillBytes(b(), "XYZ")
    IF UBOUND(b) <> 2 THEN INCR fails
    IF LBOUND(b) <> 0 THEN INCR fails
    IF b(0) <> 88 THEN INCR fails
    IF b(1) <> 89 THEN INCR fails
    IF b(2) <> 90 THEN INCR fails

    IF ShowStr("XYZ") <> 388 THEN INCR fails

    IF 1? <> 1 THEN INCR fails
    IF &H0??? <> 0 THEN INCR fails

    PRINT "batch401 failures="; fails
    PRINT "FAILURES:"; fails
    FUNCTION = fails
END FUNCTION
