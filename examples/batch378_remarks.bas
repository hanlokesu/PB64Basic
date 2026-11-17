#COMPILE EXE
#DIM ALL

DECLARE FUNCTION HiScore CDECL (BYVAL x AS LONG, BYVAL Winner AS ASCIIZ PTR, BYVAL PathSpec AS ASCIIZ PTR) AS LONG
DECLARE FUNCTION RemarksData CDECL (BYVAL x AS LONG) AS ASCIIZ PTR
DECLARE FUNCTION lstrlenA LIB "KERNEL32.DLL" ALIAS "lstrlenA" (BYVAL lpString AS ASCIIZ PTR) AS LONG

FUNCTION PBMAIN () AS LONG
    LOCAL p AS ASCIIZ PTR
    LOCAL fails AS LONG
    fails = 0

    IF HiScore(-1, "", "") <> 1 THEN INCR fails
    p = RemarksData(0)
    IF p = 0 THEN INCR fails
    IF lstrlenA(p) <> 21 THEN INCR fails
    IF PEEK(BYTE, p) <> 78 THEN INCR fails
    PRINT "hiscore="; HiScore(-1, "", ""); " p="; p; " fis="; lstrlenA(p); " fails="; fails

    PRINT "batch378 remarks, failures="; fails
    OPEN "batch378_remarks_result.txt" FOR OUTPUT AS #1
    PRINT #1, "batch378 remarks, failures="; fails
    CLOSE #1
END FUNCTION
