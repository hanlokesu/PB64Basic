#COMPILE EXE
#DIM ALL
' batch304_test.bas - legacy #1: optional-parameter missing semantics
' (ISMISSING) + ISINTERFACE (CHW ISMISSING_function.htm /
' ISINTERFACE_Function.htm / SUB_END_SUB_statements.htm).
'   * BYREF optional parameter missing  -> pointer 0 -> ISMISSING = -1
'   * VARIANT (BYVAL or BYREF) missing  -> VT_ERROR + DISP_E_PARAMNOTFOUND
'   * once a parameter is OPTIONAL, every following parameter is optional
'     too, with or without its own OPTIONAL/OPT marker
'   * ISINTERFACE(ObjectVar, InterfaceName) = -1 when supported, else 0
' Class shape follows the PBWin-official model (batch264 precedent): the
' CLASS block contains the INTERFACE block, an object variable is declared
' AS the interface name and instantiated with `oo = CLASS "CBox304"`.
' Names avoid PBWin Error 466 reserved-name clashes.
CLASS CBox304
  INSTANCE vv AS LONG
  INTERFACE IBox304
    INHERIT IUNKNOWN
    METHOD SetV(BYVAL v AS LONG)
      vv = v
    END METHOD
    METHOD GetV() AS LONG
      METHOD = vv
    END METHOD
  END INTERFACE
END CLASS

INTERFACE IOther304
  INHERIT IUNKNOWN
  METHOD Other() AS LONG
END INTERFACE

SUB OptByRef(OPTIONAL BYREF nn1 AS LONG)
    LOCAL rr AS LONG
    IF ISMISSING(nn1) THEN rr = -1 ELSE rr = 0
    PRINT #9, "ref=" & STR$(rr)
END SUB

SUB OptMix(OPTIONAL BYVAL vv1 AS VARIANT, OPTIONAL BYREF ss1 AS STRING)
    LOCAL rr1 AS LONG
    LOCAL rr2 AS LONG
    IF ISMISSING(vv1) THEN rr1 = -1 ELSE rr1 = 0
    IF ISMISSING(ss1) THEN rr2 = -1 ELSE rr2 = 0
    PRINT #9, "mix=" & STR$(rr1) & "," & STR$(rr2)
END SUB

SUB OptChain(OPTIONAL BYVAL bb1 AS LONG, BYREF cc1 AS LONG)
    LOCAL rr AS LONG
    IF ISMISSING(cc1) THEN rr = -1 ELSE rr = 0
    PRINT #9, "chain=" & STR$(rr)
END SUB

FUNCTION PBMAIN() AS LONG
    LOCAL oo AS IBox304
    LOCAL nn1 AS LONG
    LOCAL vv1 AS VARIANT
    LOCAL ss1 AS STRING
    LOCAL cc1 AS LONG
    OPEN "batch304_out.txt" FOR OUTPUT AS #9
    oo = CLASS "CBox304"
    oo.SetV(42)
    ' full-argument calls: nothing is missing
    OptByRef nn1
    OptMix vv1, ss1
    OptChain 7, cc1
    ' trailing optional arguments omitted
    OptByRef
    OptMix
    OptChain 7
    ' ISINTERFACE: supported interface -1, unknown interface 0
    PRINT #9, "if1=" & STR$(ISINTERFACE(oo, IBox304))
    PRINT #9, "if2=" & STR$(ISINTERFACE(oo, IOther304))
    CLOSE #9
    FUNCTION = 0
END FUNCTION
