' batch297_pblibmain_test.bas - PBLIBMAIN DLL entry (CHW pblibmain_function.htm)
' FUNCTION PBLIBMAIN [()] [AS LONG] - called on DLL load/unload
#COMPILE DLL
#DIM ALL
FUNCTION PBLIBMAIN AS LONG
    OPEN "batch297dll_out.txt" FOR OUTPUT AS #1
    PRINT #1, "pblibmain called"
    CLOSE #1
    FUNCTION = 1
END FUNCTION
