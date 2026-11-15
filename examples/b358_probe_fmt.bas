#COMPILE EXE
#CONSOLE ON
FUNCTION PBMAIN () AS LONG
  PRINT "s1=["; STR$(1e20); "]"
  PRINT "s2=["; STR$(0.1); "]"
  PRINT "s3=["; STR$(1/3); "]"
  PRINT "s4=["; STR$(1.5); "]"
  OPEN "b358_fmt.txt" FOR OUTPUT AS #1
  PRINT #1, "p1=["; 0.1; "]"
  PRINT #1, "p2=["; 1e20#; "]"
  PRINT #1, "p3=["; 1e20; "]"
  CLOSE #1
  OPEN "b358_fmt.txt" FOR INPUT AS #1
  DO WHILE NOT EOF(1)
    LINE INPUT #1, s$
    PRINT "file:"; s$
  LOOP
  CLOSE #1
  FUNCTION = 0
END FUNCTION