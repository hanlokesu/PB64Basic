#COMPILE EXE
#DIM ALL

#LINK "batch277_conversion.sll"

' COMMON procedures from an SLL may be DECLAREd in the host for
' self-documentation and correct return typing (CHW "Creating a Static
' Link Library"). Without a DECLARE, fork infers LONG from the name.
DECLARE FUNCTION MillimetersToInches(BYVAL mm AS DOUBLE) AS DOUBLE

FUNCTION PBMAIN () AS LONG
  LOCAL Inches      AS DOUBLE
  LOCAL MilliMeters AS DOUBLE

  MilliMeters = 1000.0#
  Inches = MillimetersToInches(MilliMeters)
  OPEN "out277c.txt" FOR OUTPUT AS #1
  PRINT #1, Inches
  CLOSE #1
  FUNCTION = 0
END FUNCTION
