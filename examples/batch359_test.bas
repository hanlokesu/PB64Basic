' PB64Basic Enhanced - batch 359: OPEN ACCESS/LOCK/BASE and CLOSE all/multiple
#CONSOLE ON
FUNCTION PBMAIN () AS LONG
    LOCAL s AS STRING
    LOCAL fld AS STRING
    LOCAL lofv AS LONG
    LOCAL errv AS LONG

    ' 1) ACCESS WRITE + LOCK SHARED
    OPEN "b359_a.txt" FOR OUTPUT ACCESS WRITE LOCK SHARED AS #1
    PRINT #1, "alpha"
    CLOSE #1

    ' 2) ACCESS READ on input
    OPEN "b359_a.txt" FOR INPUT ACCESS READ LOCK SHARED AS #1
    LINE INPUT #1, s
    CLOSE #1
    PRINT "READ="; s

    ' 3) one CLOSE with several file numbers
    OPEN "b359_b.txt" FOR OUTPUT AS #1
    OPEN "b359_c.txt" FOR OUTPUT AS #2
    PRINT #1, "bbb"
    PRINT #2, "ccc"
    CLOSE #1, #2
    OPEN "b359_b.txt" FOR INPUT AS #1
    LINE INPUT #1, s
    CLOSE #1
    PRINT "MULTI="; s

    ' 4) bare CLOSE closes every open file
    OPEN "b359_d.txt" FOR OUTPUT AS #1
    OPEN "b359_e.txt" FOR OUTPUT AS #2
    PRINT #1, "ddd"
    PRINT #2, "eee"
    CLOSE
    OPEN "b359_d.txt" FOR INPUT AS #1
    LINE INPUT #1, s
    CLOSE #1
    PRINT "CLOSEALL="; s

    ' 5) LOCK WRITE denies a second writer
    OPEN "b359_lock.txt" FOR OUTPUT LOCK WRITE AS #1
    PRINT #1, "locked"
    OPEN "b359_lock.txt" FOR OUTPUT AS #2
    errv = ERR
    CLOSE #1
    PRINT "LOCKERR="; errv

    ' 6) BASE=0: record 5 starts at offset 5*8 = 40, LOF = 48
    OPEN "b359_base0.dat" FOR RANDOM AS #3 LEN=8 BASE=0
    PRINT "BASE0="; FILEATTR(#3, -2)
    FIELD #3, 8 AS fld
    fld = "ABCDEFGH"
    PUT #3, 5
    lofv = LOF(3)
    PRINT "LOF0="; lofv
    CLOSE #3

    ' 7) default BASE=1: record 5 starts at offset 4*8 = 32, LOF = 40
    OPEN "b359_base1.dat" FOR RANDOM AS #3 LEN=8
    PRINT "BASE1="; FILEATTR(#3, -2)
    FIELD #3, 8 AS fld
    fld = "ABCDEFGH"
    PUT #3, 5
    lofv = LOF(3)
    PRINT "LOF1="; lofv
    CLOSE #3

    FUNCTION = 0
END FUNCTION