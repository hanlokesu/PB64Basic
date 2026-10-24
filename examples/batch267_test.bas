' PB64Basic Enhanced - batch 267 test: COMM SEND on a FILE channel -> ERR 54
' PBWin probe: OPEN "f" FOR OUTPUT AS #1 then COMM SEND #1,"x" raises
' runtime ERR 54 (Bad file mode). Fork used to treat #1 as a COMM channel
' and send anyway; runtime now checks file_modes[channel] first.
#COMPILE EXE
#DIM ALL
FUNCTION PBMAIN() AS LONG
    LOCAL e AS LONG
    LOCAL fail AS LONG
    OPEN "b267_out.txt" FOR OUTPUT AS #2
    OPEN "b267_probe.dat" FOR OUTPUT AS #1
    ON ERROR GOTO 100
    COMM SEND #1, "x"
    PRINT #2, "SEND-on-file: no error"
    GOTO 200
100:
    e = ERR
    IF e = 54 THEN
        PRINT #2, "SEND-on-file: ERR=54 OK"
    ELSE
        PRINT #2, "SEND-on-file: ERR=" + STR$(e) + " (expected 54)"
        fail = fail + 1
    END IF
200:
    ON ERROR GOTO 0
    CLOSE #1
    KILL "b267_probe.dat"
    CLOSE #2
    IF fail = 0 THEN
        PRINT #2, "batch267 test ok"
    ELSE
        PRINT #2, "FAILURES: " + STR$(fail)
    END IF
    FUNCTION = fail
END FUNCTION
