' batch 359 probe: FIELD over GLOBAL / STATIC / LOCAL string slots.
' The pb_field_t struct must fit the variable's storage on both targets.
GLOBAL gs AS STRING

FUNCTION PBMAIN () AS LONG
    LOCAL fs AS STRING
    STATIC ss AS STRING
    LOCAL n AS LONG
    OPEN "b359_fg.dat" FOR RANDOM AS #1 LEN=24
    FIELD #1, 10 AS fs, 10 AS gs, 4 AS ss
    fs = "LOCAL"
    gs = "GLOBAL"
    ss = "STAT"
    PUT #1, 1
    GET #1, 1
    PRINT "FS="; fs
    PRINT "GS="; gs
    PRINT "SS="; ss
    n = LOF(1)
    PRINT "LOF="; n
    CLOSE #1
    FUNCTION = 0
END FUNCTION