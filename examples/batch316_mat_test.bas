#COMPILE EXE
#DIM ALL
' batch316_mat_test.bas - regression probe for the M-family coverage group:
'   MAT statement (IDN/ZER/CON/CON(expr)/assign/ADD/SUB/SCALE/MUL/1-D/TRN/INV)
'   MID$ statement on a fixed-length-string target (STRING*n)
'   MEMORY COPY / SWAP / FILL (BYTE/WORD/DWORD) / FILLS (string)
' Headless: prints a single banner and a FAILURES count; no MSGBOX/WAITKEY$/MODAL.
FUNCTION PBMAIN() AS LONG
  DIM a(1 TO 2, 1 TO 2) AS LONG
  DIM b(1 TO 2, 1 TO 2) AS LONG
  DIM c(1 TO 2, 1 TO 2) AS LONG
  DIM t(1 TO 2, 1 TO 2) AS LONG
  DIM d(1 TO 4) AS LONG
  DIM bb(1 TO 6) AS BYTE
  DIM ww(1 TO 2) AS WORD
  DIM qq(1 TO 2) AS LONG
  DIM da(1 TO 2, 1 TO 2) AS DOUBLE
  DIM dm(1 TO 2, 1 TO 2) AS DOUBLE
  LOCAL fs AS STRING * 10
  LOCAL fails AS LONG
  LOCAL s AS STRING

  fails = 0

  ' ---- MAT statement -------------------------------------------------------
  MAT a = IDN
  IF a(1,1)<>1 OR a(1,2)<>0 OR a(2,1)<>0 OR a(2,2)<>1 THEN INCR fails

  MAT a = ZER
  IF a(1,1)<>0 OR a(2,2)<>0 THEN INCR fails

  MAT a = CON
  IF a(1,1)<>1 OR a(1,2)<>1 OR a(2,1)<>1 OR a(2,2)<>1 THEN INCR fails

  MAT a = CON(7)
  IF a(1,1)<>7 OR a(2,2)<>7 THEN INCR fails

  b(1,1)=1: b(1,2)=2: b(2,1)=3: b(2,2)=4
  c(1,1)=5: c(1,2)=6: c(2,1)=7: c(2,2)=8

  MAT a = b
  IF a(1,1)<>1 OR a(1,2)<>2 OR a(2,1)<>3 OR a(2,2)<>4 THEN INCR fails

  MAT a = b + c
  IF a(1,1)<>6 OR a(1,2)<>8 OR a(2,1)<>10 OR a(2,2)<>12 THEN INCR fails

  MAT a = b - c
  IF a(1,1)<>-4 OR a(1,2)<>-4 OR a(2,1)<>-4 OR a(2,2)<>-4 THEN INCR fails

  MAT a = (2) * b
  IF a(1,1)<>2 OR a(1,2)<>4 OR a(2,1)<>6 OR a(2,2)<>8 THEN INCR fails

  MAT a = b * c
  ' [[1,2],[3,4]] * [[5,6],[7,8]] = [[19,22],[43,50]]
  IF a(1,1)<>19 OR a(1,2)<>22 OR a(2,1)<>43 OR a(2,2)<>50 THEN INCR fails

  MAT d = CON(9)
  IF d(1)<>9 OR d(2)<>9 OR d(3)<>9 OR d(4)<>9 THEN INCR fails

  MAT t = TRN(b)
  IF t(1,1)<>1 OR t(1,2)<>3 OR t(2,1)<>2 OR t(2,2)<>4 THEN INCR fails

  dm(1,1)=1: dm(1,2)=2: dm(2,1)=3: dm(2,2)=5
  MAT da = INV(dm)
  ' inverse of [[1,2],[3,5]] (det=-1) is [[-5,2],[3,-1]]
  ' INV is floating-point; compare the rounded integer value (display matches
  ' PBWin byte-for-byte, but the raw double may carry a tiny Gauss-Jordan epsilon).
  IF CLNG(da(1,1))<>-5 OR CLNG(da(1,2))<>2 OR CLNG(da(2,1))<>3 OR CLNG(da(2,2))<>-1 THEN INCR fails

  ' ---- MID$ statement on fixed-length strings (STRING*10) -------------------
  fs = "1234567890": MID$(fs,1,1)="P"
  IF fs<>"P234567890" THEN INCR fails
  fs = "1234567890": MID$(fs,1,2)="Po"
  IF fs<>"Po34567890" THEN INCR fails
  fs = "1234567890": MID$(fs,1,10)="PowerBASIC"
  IF fs<>"PowerBASIC" THEN INCR fails
  fs = "1234567890": MID$(fs,1,3)="XY"
  IF fs<>"XY34567890" THEN INCR fails
  fs = "1234567890": MID$(fs,1)="AB"
  IF fs<>"AB34567890" THEN INCR fails
  fs = "1234567890": MID$(fs,-1,1)="Z"
  IF fs<>"123456789Z" THEN INCR fails
  fs = "1234567890": MID$(fs,-2,2)="XY"
  IF fs<>"12345678XY" THEN INCR fails
  fs = "1234567890": MID$(fs,1,-3)="PowerBASIC"
  IF fs<>"PowerBA890" THEN INCR fails
  fs = "1234567890": MID$(fs,0,1)="Z"
  IF fs<>"1234567890" THEN INCR fails
  fs = "1234567890": MID$(fs,11,1)="Z"
  IF fs<>"1234567890" THEN INCR fails
  fs = "1234567890": MID$(fs,2 TO 4)="XYZ"
  IF fs<>"1XYZ567890" THEN INCR fails
  fs = "1234567890": MID$(fs,9,5)="WXYZ"
  IF fs<>"12345678WX" THEN INCR fails
  fs = "1234567890": MID$(fs,1 TO 1)="Q"
  IF fs<>"Q234567890" THEN INCR fails
  fs = "1234567890": MID$(fs,-3 TO -2)="AB"
  IF fs<>"1234567AB0" THEN INCR fails

  ' ---- MEMORY family (Count is BYTES per CHW) ------------------------------
  MEMORY FILL VARPTR(bb(1)), 6, BYTE 170
  IF bb(1)<>170 OR bb(6)<>170 THEN INCR fails

  MEMORY FILL VARPTR(ww(1)), 2, WORD &H1234
  IF ww(1)<>&H1234 OR ww(2)<>0 THEN INCR fails

  MEMORY FILL VARPTR(qq(1)), 2, DWORD &H11223344
  ' only 2 bytes written -> low half of qq(1) = 0x3344
  IF (qq(1) AND &HFFFF&)<>&H3344 OR qq(2)<>0 THEN INCR fails

  MEMORY FILL VARPTR(bb(1)), 6, "AB"
  IF bb(1)<>65 OR bb(2)<>66 OR bb(3)<>65 OR bb(4)<>66 OR bb(5)<>65 OR bb(6)<>66 THEN INCR fails

  MEMORY COPY VARPTR(bb(2)), VARPTR(bb(1)), 4
  ' bb was A B A B A B ; copy bb(2..5) over bb(1..4) -> B A B A A B
  IF bb(1)<>66 OR bb(2)<>65 OR bb(3)<>66 OR bb(4)<>65 OR bb(5)<>65 OR bb(6)<>66 THEN INCR fails

  bb(1)=11: bb(2)=22: bb(3)=33: bb(4)=44
  MEMORY SWAP VARPTR(bb(1)), VARPTR(bb(3)), 2
  IF bb(1)<>33 OR bb(2)<>44 OR bb(3)<>11 OR bb(4)<>22 THEN INCR fails

  PRINT "batch316 mat ok"
  PRINT "FAILURES: "; LTRIM$(STR$(fails))
  FUNCTION = 0
END FUNCTION
