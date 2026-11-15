' batch 360: global STRING FIELD slot and CHR=WIDE no-op for non-sequential files.
' The STRING global reserves four pointer slots so the FIELD descriptor does
' not clobber the adjacent LONG global.  RANDOM/BINARY CHR=WIDE has no effect
' per CHW; sequential WIDE remains an explicit compile-time diagnostic.
GLOBAL Tag AS STRING
GLOBAL Sentinel AS LONG

FUNCTION PBMAIN() AS LONG
    Sentinel = 777
    OPEN "b360_field.dat" FOR RANDOM AS #1 LEN=24 CHR=WIDE
    FIELD #1, 10 AS Tag
    IF Sentinel <> 777 THEN FUNCTION = 1 : EXIT FUNCTION
    Tag = "alpha"
    PUT #1, 1
    GET #1, 1
    IF Tag <> "alpha" THEN FUNCTION = 2 : EXIT FUNCTION
    CLOSE #1
    OPEN "b360_field.bin" FOR BINARY AS #2 CHR=WIDE
    CLOSE #2
    PRINT "TAG="; Tag; " SENTINEL="; Sentinel
    FUNCTION = 0
END FUNCTION