FUNCTION PBMAIN() AS LONG
  ' batch 334 self-check: Error 6 add/sub overflow and Error 5 math-domain traps.
  ' Non-error paths are exercised here; the error paths are verified separately.
  DIM a AS LONG
  DIM b AS LONG
  a = 100
  b = 200
  DIM c AS LONG
  c = a + b
  IF c <> 300 THEN PRINT "FAIL: add" : FUNCTION = 1 : EXIT FUNCTION
  c = b - a
  IF c <> 100 THEN PRINT "FAIL: sub" : FUNCTION = 1 : EXIT FUNCTION
  DIM x AS DOUBLE
  x = SQR(16.0)
  IF x <> 4.0 THEN PRINT "FAIL: sqr" : FUNCTION = 1 : EXIT FUNCTION
  PRINT "batch334 self-check OK"
  FUNCTION = 0
END FUNCTION
