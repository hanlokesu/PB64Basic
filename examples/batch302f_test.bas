' batch302f_test.bas - PRINT # float-item formatting (fork num_to_string %g vs PBWin)
' Legacy #13: integers were byte-verified in batch284; float items were never
' compared byte-for-byte with PBWin. This probe writes representative floats
' (positive/negative/zero/whole/small/large/scientific) as PRINT #1 items.
#COMPILE EXE
FUNCTION PBMAIN() AS LONG
    OPEN "batch302f_out.txt" FOR OUTPUT AS #1
    PRINT #1, 1.5
    PRINT #1, -0.25
    PRINT #1, 0.0
    PRINT #1, 1.0
    PRINT #1, -1.0
    PRINT #1, 3.14159
    PRINT #1, 1.0E20
    PRINT #1, 1.0E-20
    PRINT #1, 123456.789
    PRINT #1, 0.1; -0.1; 100.0
    PRINT #1, 1.5E-9
    PRINT #1, 12345.6789
    PRINT #1, 2.5E15
    CLOSE #1
    FUNCTION = 0
END FUNCTION
