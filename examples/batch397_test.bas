#COMPILE EXE
#CONSOLE ON
#DIM ALL

' batch 397: a type clause in LOCAL/DIM applies only to its own
' comma-separated group. `LOCAL a, b AS LONG, c, d AS DOUBLE` must leave
' c and d as DOUBLE; the old parser carried the LONG type across the comma
' and truncated 1.25 to 1.
FUNCTION PBMAIN () AS LONG
    LOCAL fails AS LONG
    LOCAL a, b AS LONG, c, d AS DOUBLE
    LOCAL e, f AS LONG, g AS STRING, h AS DOUBLE

    fails = 0
    a = 7
    b = 9
    c = 1.25
    d = -2.5
    e = 3
    f = 4
    g = "ok"
    h = 0.125

    IF a + b <> 16 THEN INCR fails
    IF c <> 1.25 THEN INCR fails
    IF d <> -2.5 THEN INCR fails
    IF c * 4 <> 5 THEN INCR fails
    IF e + f <> 7 THEN INCR fails
    IF g <> "ok" THEN INCR fails
    IF h <> 0.125 THEN INCR fails

    PRINT "batch397 grouped LOCAL types failures="; fails
    FUNCTION = fails
END FUNCTION
