#COMPILE EXE
#CONSOLE ON
#DIM ALL

' batch 398: fixed-length strings must reserve their declared byte width.
' A GLOBAL ASCIIZ * 260 used to be declared as a pointer-sized slot in IR,
' so the linker placed following CRT data (the atexit table) inside the
' variable and PBNote2 crashed while exiting.  This test checks the storage
' width of GLOBAL and LOCAL fixed strings and their text semantics.

GLOBAL gz AS ASCIIZ * 260
GLOBAL gs AS STRING * 20

FUNCTION PBMAIN () AS LONG
    LOCAL fails AS LONG
    LOCAL la AS ASCIIZ * 8
    LOCAL ls AS STRING * 16

    fails = 0
    gz = "HELLO"
    gs = "WORLD"
    la = "LOCAL"
    ls = "BUFFER"

    IF SIZEOF(gz) <> 260 THEN INCR fails
    IF LEN(gz) <> 5 THEN INCR fails
    IF SIZEOF(gs) <> 20 THEN INCR fails
    IF TRIM$(gs) <> "WORLD" THEN INCR fails
    IF SIZEOF(la) <> 8 THEN INCR fails
    IF la <> "LOCAL" THEN INCR fails
    IF SIZEOF(ls) <> 16 THEN INCR fails
    IF TRIM$(ls) <> "BUFFER" THEN INCR fails

    PRINT "batch398 fixed string widths failures="; fails
    PRINT "FAILURES:"; fails
    FUNCTION = fails
END FUNCTION
