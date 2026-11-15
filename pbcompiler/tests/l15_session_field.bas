' L15: FIELD-bound global STRING in session-struct mode.
' The STRING field must reserve four pointer slots, matching the non-session
' global layout, so the pb_field_t descriptor cannot clobber the next global.
' CHR=WIDE is also accepted as a documented no-op for RANDOM/BINARY files.
GLOBAL Tag AS STRING
GLOBAL Sentinel AS LONG

FUNCTION PBMAIN() AS LONG
    Sentinel = 777
    OPEN "l15_session_field.dat" FOR RANDOM AS #1 LEN=24 CHR=WIDE
    FIELD #1, 10 AS Tag
    IF Sentinel <> 777 THEN FUNCTION = 1 : EXIT FUNCTION
    Tag = "alpha"
    PUT #1, 1
    GET #1, 1
    IF Tag <> "alpha" THEN FUNCTION = 2 : EXIT FUNCTION
    CLOSE #1
    OPEN "l15_session_field.bin" FOR BINARY AS #2 CHR=WIDE
    CLOSE #2
    FUNCTION = 0
END FUNCTION