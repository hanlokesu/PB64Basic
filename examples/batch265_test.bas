' batch265_test.bas - ERR 54 (Bad file mode) alignment + ON ERROR/RESUME fixes.
' CHW error_54: PUT/GET (PUT$/GET$) on a file opened in sequential mode
' (INPUT/OUTPUT/APPEND) generates error 54. BINARY/RANDOM are legal.
' PBWin10 dual-run: g1-g4 all 54, b1-b4 all 0 (probe p54_put/p54ok).
#COMPILE EXE
GLOBAL g1, g2, g3, g4, b1, b2, b3, b4 AS LONG
FUNCTION PBMAIN() AS LONG
  LOCAL x AS LONG
  LOCAL s AS STRING
  g1 = 0 : g2 = 0 : g3 = 0 : g4 = 0
  b1 = 0 : b2 = 0 : b3 = 0 : b4 = 0
  ON ERROR GOTO 100
  OPEN "b265_seq_a.txt" FOR OUTPUT AS #1
  PUT$ #1, "abc"
  g1 = ERR
  CLOSE #1
  OPEN "b265_seq_b.dat" FOR OUTPUT AS #2
  PUT #2, 1, x
  g2 = ERR
  CLOSE #2
  OPEN "b265_seq_c.txt" FOR OUTPUT AS #3
  GET$ #3, 3, s
  g3 = ERR
  CLOSE #3
  OPEN "b265_seq_d.dat" FOR OUTPUT AS #4
  GET #4, 1, x
  g4 = ERR
  CLOSE #4
  OPEN "b265_bin.dat" FOR BINARY AS #1
  x = 123
  PUT #1, 1, x
  b1 = ERR
  GET #1, 1, x
  b2 = ERR
  CLOSE #1
  OPEN "b265_ran.dat" FOR RANDOM AS #2
  x = 456
  PUT #2, 1, x
  b3 = ERR
  GET #2, 1, x
  b4 = ERR
  CLOSE #2
  OPEN "b265_out.txt" FOR OUTPUT AS #9
  PRINT #9, "batch265 test "; g1; g2; g3; g4; b1; b2; b3; b4
  CLOSE #9
  FUNCTION = 0
100 RESUME NEXT
END FUNCTION
