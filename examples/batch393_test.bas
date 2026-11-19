' batch393_test.bas - PB file-number namespace shared by files and sockets.
' TCP ACCEPT now allocates through pb_freefile(), and TCP/UDP OPEN reject a
' number that is already occupied by a regular file.
#COMPILE EXE
#CONSOLE ON
#DIM ALL

FUNCTION PBMAIN() AS LONG
    LOCAL failures AS LONG
    LOCAL f AS LONG
    LOCAL nf AS LONG

    ' A regular file owns #1; FREEFILE must skip it.
    OPEN "b393_tmp.dat" FOR OUTPUT AS #1
    f = FREEFILE
    IF f <> 2 THEN
        PRINT "FAIL freefile-after-open=" + STR$(f)
        INCR failures
    END IF

    ' TCP OPEN must not steal a number owned by a regular file.
    TCP OPEN SERVER ADDR &H0100007F PORT 46393 AS #1
    IF ERR <> 52 THEN
        PRINT "FAIL tcp-open-file-collision err=" + STR$(ERR)
        INCR failures
    END IF

    ' UDP OPEN has the same file-number guard.
    UDP OPEN PORT 46396 AS #1
    IF ERR <> 52 THEN
        PRINT "FAIL udp-open-file-collision err=" + STR$(ERR)
        INCR failures
    END IF

    CLOSE #1
    KILL "b393_tmp.dat"

    ' A listening socket owns #1 and a connected client owns #2; the accepted
    ' connection must receive #3, not either of the occupied numbers.
    TCP OPEN SERVER ADDR &H0100007F PORT 46393 AS #1
    TCP OPEN PORT 46393 AT "127.0.0.1" AS #2
    TCP ACCEPT #1 AS nf
    IF nf <> 3 THEN
        PRINT "FAIL tcp-accept-slot=" + STR$(nf)
        INCR failures
    END IF
    TCP CLOSE nf
    TCP CLOSE #2
    TCP CLOSE #1

    PRINT "batch393 failures=" + STR$(failures)
    OPEN "batch393_result.txt" FOR OUTPUT AS #9
    PRINT #9, "batch393 failures="; failures
    CLOSE #9
    FUNCTION = failures
END FUNCTION
