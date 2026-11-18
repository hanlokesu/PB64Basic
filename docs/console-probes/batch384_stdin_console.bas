' batch384_stdin_console - one CON.STDIN.LINE from a real console keyboard.
#COMPILE EXE
#CONSOLE ON
#DIM ALL
FUNCTION PBMAIN() AS LONG
    LOCAL s AS STRING
    LOCAL e AS LONG
    OPEN "batch384_stdin_console.txt" FOR OUTPUT AS #1
    CON.STDIN.LINE TO s
    e = CON.STDEOF
    PRINT #1, "len="; LEN(s); " line=["; s; "] eof="; e
    CLOSE #1
END FUNCTION
