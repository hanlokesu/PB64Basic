' batch264_test.bas - CLASS interface object model (CHW CLASS_END_CLASS_Block /
' INTERFACE___END_INTERFACE_Block_(Direct) / How_do_you_create_an_object_ /
' How_do_you_call_a_Direct_Method_).  The PBWin-official shape: CLASS block
' contains an INTERFACE block, an object variable is declared AS the interface
' name, instantiated with `o = CLASS "C213"`, and methods are called through the
' dot sugar.  Output goes to b264_out.txt (PBWin EXE targets are GUI-subsystem
' and have no console PRINT; PRINT# to a file is the portable path).
#COMPILE EXE
CLASS C213
  INSTANCE n AS LONG
  INTERFACE I213
    INHERIT IUNKNOWN
    METHOD TWICE(BYVAL x AS LONG) AS LONG
      METHOD = x * 2
    END METHOD
  END INTERFACE
END CLASS
FUNCTION PBMAIN() AS LONG
  LOCAL o AS I213
  LOCAL p AS I213
  LOCAL r AS LONG
  LOCAL ok AS LONG
  ok = 0
  o = CLASS "C213"
  p = CLASS "C213"
  r = o.TWICE(21)
  IF r = 42 THEN ok = ok + 1
  r = p.TWICE(30)
  IF r = 60 THEN ok = ok + 10
  OPEN "b264_out.txt" FOR OUTPUT AS #1
  PRINT #1, "batch264 test "; ok
  IF ok <> 11 THEN PRINT #1, "FAILURES: 1"
  CLOSE #1
  FUNCTION = 0
END FUNCTION
