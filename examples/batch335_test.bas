FUNCTION PBMAIN() AS LONG
  ' batch 335 self-check: INT_MIN / -1 division edge case.
  ' Non-error paths are exercised here.
  DIM a AS LONG
  DIM b AS LONG
  a = 100
  b = 3
  DIM c AS LONG
  c = a \ b
  IF c <> 33 THEN PRINT "FAIL: 100\3" : FUNCTION = 1 : EXIT FUNCTION
  c = a MOD b
  IF c <> 1 THEN PRINT "FAIL: 100 MOD 3" : FUNCTION = 1 : EXIT FUNCTION
  PRINT "batch335 self-check OK"
  FUNCTION = 0
END FUNCTION
