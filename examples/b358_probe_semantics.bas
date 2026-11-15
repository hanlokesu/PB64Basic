#COMPILE EXE
#CONSOLE ON
FUNCTION PBMAIN () AS LONG
  LOCAL i AS LONG
  REDIM a(1 TO 3) AS LONG
  a(1) = 11
  a(2) = 22
  a(3) = 33
  REDIM PRESERVE a(1 TO 6) AS LONG
  a(4) = 44
  PRINT "grow:"; a(1); a(2); a(3); a(4)
  REDIM PRESERVE a(1 TO 2) AS LONG
  PRINT "shrink:"; a(1); a(2)
  REDIM PRESERVE a(1 TO 5) AS LONG
  PRINT "regrow:"; a(1); a(2); a(5)
  OPEN "b358_fmt.txt" FOR OUTPUT AS #1
  PRINT #1, 1.5
  PRINT #1, 1/3
  PRINT #1, 1e20
  PRINT #1, -2.25
  CLOSE #1
  OPEN "b358_fmt.txt" FOR INPUT AS #1
  DO WHILE NOT EOF(1)
    LINE INPUT #1, s$
    PRINT "line=["; s$; "] len="; LEN(s$)
  LOOP
  CLOSE #1
  FUNCTION = 0
END FUNCTION