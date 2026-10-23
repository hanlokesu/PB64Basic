' batch213_test.bas
' Topic (batch 213): INSTANCE array members must fail LOUDLY (never silently
' misbehave); the legal CLASS + INSTANCE (string) declaration must compile
' and run on both sides. Rebuilt from the batch's real topic; see SKILL.md.
' PBWin-official CLASS shape (CHW CLASS_END_CLASS_Block / INTERFACE block):
' a CLASS block MUST contain an INTERFACE block, otherwise PBWin reports
' 562 "INTERFACE expected". Output goes to b213_out.txt (GUI subsystem:
' no console PRINT; PRINT# to a file is the portable path).
#COMPILE EXE
CLASS C213
  INSTANCE inst AS STRING
  INTERFACE I213
    INHERIT IUNKNOWN
  END INTERFACE
END CLASS
FUNCTION PBMAIN() AS LONG
  OPEN "b213_out.txt" FOR OUTPUT AS #1
  PRINT #1, "batch213 test ok"
  CLOSE #1
  FUNCTION = 0
END FUNCTION
