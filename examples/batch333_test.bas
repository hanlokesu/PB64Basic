FUNCTION PBMAIN() AS LONG
  ' batch 333 self-check: Error 6 overflow and Error 9 subscript traps.
  ' This program exercises the non-error paths; the error paths are verified
  ' separately by running the built exe against known-bad inputs.
  DIM a(5) AS LONG
  DIM i AS LONG
  FOR i = 0 TO 5
    a(i) = i * 10
  NEXT i
  IF a(3) <> 30 THEN PRINT "FAIL: a(3)" : FUNCTION = 1 : EXIT FUNCTION
  IF a(5) <> 50 THEN PRINT "FAIL: a(5)" : FUNCTION = 1 : EXIT FUNCTION
  DIM x AS LONG
  x = 6 * 7
  IF x <> 42 THEN PRINT "FAIL: 6*7" : FUNCTION = 1 : EXIT FUNCTION
  PRINT "batch333 self-check OK"
  FUNCTION = 0
END FUNCTION
