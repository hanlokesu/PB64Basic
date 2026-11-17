' batch378_input_redirect - CON.INPUT must not consume redirected stdin.
' Run with:  echo 7,pipe| batch378_input_redirect.exe
' CHW: CON.INPUT always reads the keyboard (CONIN$), never Standard Input.
' With no console keyboard this returns an empty line instead of reading the
' pipe, so the output must be n=0 s=[] if the rule holds.
#COMPILE EXE
#CONSOLE ON
#DIM ALL

FUNCTION PBMAIN() AS LONG
    LOCAL n AS LONG
    LOCAL s AS STRING
    CON.INPUT "num,word", n, s
    OPEN "batch378_input_redirect.txt" FOR OUTPUT AS #1
    PRINT #1, "n=" + STR$(n) + " s=[" + s + "]"
    CLOSE #1
END FUNCTION
