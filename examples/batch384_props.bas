' batch384_props - error-code and round-trip assertions for CON setters.
' Must run in a real console window (Git CMD / conhost).
#COMPILE EXE
#CONSOLE ON
#DIM ALL

FUNCTION PBMAIN() AS LONG
    LOCAL w AS LONG, h AS LONG, x AS LONG, y AS LONG
    LOCAL oldp AS LONG, id AS LONG, e AS LONG
    LOCAL fails AS LONG

    OPEN "batch384_props.txt" FOR OUTPUT AS #1
    CON.PAGE.ACTIVE = 1
    CON.PAGE.VISIBLE = 1
    oldp = CON.PAGE.ACTIVE

    CON.PAGE.ACTIVE = 9
    e = ERR
    PRINT #1, "page9 err="; e; " page="; CON.PAGE.ACTIVE; " old="; oldp
    IF e <> 5 THEN fails = fails + 1
    IF CON.PAGE.ACTIVE <> oldp THEN fails = fails + 1

    CON.PAGE.VISIBLE = 0
    e = ERR
    PRINT #1, "pagev0 err="; e; " visible="; CON.PAGE.VISIBLE
    IF e <> 5 THEN fails = fails + 1
    IF CON.PAGE.VISIBLE <> 1 THEN fails = fails + 1

    CON.PAGE.VISIBLE = 9
    e = ERR
    PRINT #1, "pagev9 err="; e; " visible="; CON.PAGE.VISIBLE
    IF e <> 5 THEN fails = fails + 1

    CON.SIZE TO w, h
    PRINT #1, "size="; w; ","; h
    IF w < 1 OR h < 1 THEN fails = fails + 1
    CON.SIZE = 0, 0
    e = ERR
    PRINT #1, "size0 err="; e; " read="; CON.SIZE.X; ","; CON.SIZE.Y
    IF e <> 5 THEN fails = fails + 1
    IF CON.SIZE.X <> w OR CON.SIZE.Y <> h THEN fails = fails + 1

    CON.SIZE = w, h
    e = ERR
    PRINT #1, "size_same err="; e; " read="; CON.SIZE.X; ","; CON.SIZE.Y
    IF e <> 0 THEN fails = fails + 1
    IF CON.SIZE.X <> w OR CON.SIZE.Y <> h THEN fails = fails + 1

    CON.LOC TO x, y
    PRINT #1, "loc="; x; ","; y
    CON.LOC = x, y
    e = ERR
    PRINT #1, "loc_same err="; e
    IF e <> 0 THEN fails = fails + 1

    id = CON.GETSTDVID(0)
    e = ERR
    PRINT #1, "stdvid0="; id; " err="; e
    IF id = 0 THEN fails = fails + 1
    IF e <> 0 THEN fails = fails + 1

    id = CON.GETSTDVID(9)
    e = ERR
    PRINT #1, "stdvid9="; id; " err="; e
    IF id <> 0 THEN fails = fails + 1
    IF e <> 5 THEN fails = fails + 1

    CON.FOCUS
    e = ERR
    PRINT #1, "focus err="; e
    IF e <> 0 THEN fails = fails + 1

    PRINT #1, "failures="; fails
    CLOSE #1
    FUNCTION = fails
END FUNCTION
