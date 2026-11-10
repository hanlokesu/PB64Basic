' batch332: runtime Error 11 division-by-zero trap + Error 482 mapping fix
FUNCTION PBMAIN() AS LONG
  ' Normal integer division and MOD must still work
  DIM a AS LONG
  DIM b AS LONG
  a = 10
  b = 3
  DIM c AS LONG
  c = a \ b
  IF c <> 3 THEN PRINT "FAIL: 10\3 should be 3, got "; c : FUNCTION = 1 : EXIT FUNCTION
  DIM m AS LONG
  m = a MOD b
  IF m <> 1 THEN PRINT "FAIL: 10 MOD 3 should be 1, got "; m : FUNCTION = 1 : EXIT FUNCTION
  PRINT "batch332: normal div/mod OK"
  FUNCTION = 0
END FUNCTION
