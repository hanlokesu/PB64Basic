#COMPILE EXE
#CONSOLE ON
TYPE TW
    S AS WSTRING * 8
END TYPE
FUNCTION PBMAIN () AS LONG
    LOCAL fails AS LONG
    LOCAL f AS LONG
    LOCAL i AS LONG
    LOCAL w(1 TO 2) AS WSTRING * 8
    LOCAL t AS TW

    fails = 0
    OPEN "batch379_wstring_array.dat" FOR OUTPUT AS #1
    PRINT #1, "ABC"
    PRINT #1, "DEFGH"
    CLOSE #1

    OPEN "batch379_wstring_array.dat" FOR INPUT AS #1
    LINE INPUT #1, w(1)
    LINE INPUT #1, w(2)
    CLOSE #1
    IF LEFT$(w(1), 3) <> "ABC" THEN INCR fails
    IF LEFT$(w(2), 5) <> "DEFGH" THEN INCR fails
    IF LEN(w(1)) <> 8 THEN INCR fails

    OPEN "batch379_wstring_array.dat" FOR INPUT AS #1
    LINE INPUT #1, t.S
    CLOSE #1
    IF LEFT$(t.S, 3) <> "ABC" THEN INCR fails
    IF LEN(t.S) <> 8 THEN INCR fails

    OPEN "batch379_wstring_array.txt" FOR OUTPUT AS #1
    PRINT #1, "batch379 wstring array, failures="; fails
    PRINT #1, "w1=["; w(1); "] len="; LEN(w(1)); " ts=["; t.S; "]"
    CLOSE #1
    FUNCTION = fails
END FUNCTION
