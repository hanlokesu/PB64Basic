#COMPILE EXE
#DIM ALL
CLASS CBox305
  INSTANCE vv AS LONG
  INTERFACE IBox305
    INHERIT IUNKNOWN
    METHOD SetV(x AS LONG)
      vv = x
    END METHOD
    METHOD GetV() AS LONG
      METHOD = vv
    END METHOD
  END INTERFACE
END CLASS
FUNCTION PBMAIN() AS LONG
    LOCAL oo AS IBox305
    LOCAL oo2 AS IBox305
    LOCAL a AS LONG
    LOCAL b AS LONG
    oo = CLASS "CBox305"
    oo2 = oo
    oo2.SetV(42)
    a = oo.GetV()
    oo.SetV(7)
    b = oo2.GetV()
    OPEN "batch305b_out.txt" FOR OUTPUT AS #9
    PRINT #9, "a=" & STR$(a) & " b=" & STR$(b)
    CLOSE #9
    FUNCTION = 0
END FUNCTION
