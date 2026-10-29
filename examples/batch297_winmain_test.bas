' batch297_winmain_test.bas - WINMAIN entry point (CHW WINMAIN_function.htm)
' FUNCTION {WINMAIN | MAIN} (BYVAL hInstance AS DWORD, BYVAL hPrevInst AS DWORD,
'   BYVAL lpszCmdLine AS WSTRINGZ PTR, BYVAL nCmdShow AS LONG) AS LONG
#COMPILE EXE
#DIM ALL
FUNCTION WINMAIN (BYVAL hInstance AS DWORD, BYVAL hPrevInst AS DWORD, BYVAL lpszCmdLine AS WSTRINGZ PTR, BYVAL nCmdShow AS LONG) AS LONG
    OPEN "batch297_out.txt" FOR OUTPUT AS #1
    PRINT #1, "hprev="; hPrevInst
    PRINT #1, "nshow="; nCmdShow
    PRINT #1, "cmdnull="; (lpszCmdLine = 0)
    PRINT #1, "cmdarg="; COMMAND$
    PRINT #1, "hinst_nz="; (hInstance <> 0)
    CLOSE #1
    FUNCTION = 0
END FUNCTION
