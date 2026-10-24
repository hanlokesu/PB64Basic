' PB64Basic Enhanced - batch 268 test: ARRAY SORT fixed-string array with
' custom CALL comparison function (CHW ARRAY_SORT_statement.htm CALL form).
' PBWin probe: user fn takes two BYREF STRING*N, returns -1/0/1; fork now
' passes the user function's address to pb_array_sort2, which calls it with
' each element's address (a fixed-string element IS a BSTR-pointer slot, the
' same shape a BYREF parameter expects). Both sides: scall=a|bb|ccc|xxxx.
#COMPILE EXE
#DIM ALL
FUNCTION CmpLen(BYREF a AS STRING*4, BYREF b AS STRING*4) AS LONG
    IF LEN(a) < LEN(b) THEN FUNCTION = -1 : EXIT FUNCTION
    IF LEN(a) > LEN(b) THEN FUNCTION = 1 : EXIT FUNCTION
    IF a < b THEN FUNCTION = -1 : EXIT FUNCTION
    IF a > b THEN FUNCTION = 1 : EXIT FUNCTION
    FUNCTION = 0
END FUNCTION
FUNCTION PBMAIN() AS LONG
    DIM arr(1 TO 4) AS STRING * 4
    LOCAL i AS LONG
    LOCAL s AS STRING
    arr(1) = "xxxx"
    arr(2) = "a"
    arr(3) = "ccc"
    arr(4) = "bb"
    ARRAY SORT arr(), CALL CmpLen
    s = ""
    FOR i = 1 TO 4
        IF i > 1 THEN s = s + "|"
        s = s + RTRIM$(arr(i))
    NEXT i
    OPEN "b268_out.txt" FOR OUTPUT AS #2
    PRINT #2, "scall=" + s
    CLOSE #2
    FUNCTION = 0
END FUNCTION
