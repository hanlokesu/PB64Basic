#COMPILE EXE
#DIM ALL
' batch316_misc_test.bas - M-family consolidated regression probe (group MISC), file-based.
'   MACRO, MAK, MAX/MIN (variadic + string), MCASE$, MID$ function (comma/TO/neg),
'   MOD, MONTHNAME$, METRICS, MKDIR (Error 75), METHOD/ME.  Bare MCASE/MONTHNAME
'   are code aliases of the documented string forms (CHW silent on numeric).
'   Output written to TEMP\b316_misc_out.txt (PRINT #1) so it compiles under both
'   PBWin10 (GUI subsystem: bare PRINT = Error 477) and the fork.
#IF %DEF(%PB_REVISION)
    #IF (%PB_REVISION AND &H0FF00) = &H1000
        %MY_PBVER = 10
    #ELSE
        %MY_PBVER = 0
    #ENDIF
#ELSE
    %MY_PBVER = 0
#ENDIF

MACRO Add1(a,b) = ((a)+(b))

#IF %MY_PBVER = 0
CLASS Box
  INSTANCE v AS LONG
  METHOD SetIt(n AS LONG) AS LONG
    v = n
    FUNCTION = v
  END METHOD
  METHOD DoubleIt(n AS LONG) AS LONG
    FUNCTION = ME.SetIt(n * 2)
  END METHOD
END CLASS
#ENDIF

FUNCTION PBMAIN() AS LONG
  LOCAL mkval AS LONG
  LOCAL nFails AS LONG
  LOCAL mkPath AS STRING
  LOCAL nGot AS LONG
  LOCAL outFile AS STRING
#IF %MY_PBVER = 0
  LOCAL bx AS Box
#ENDIF
  nFails = 0
  outFile = ENVIRON$("TEMP") + "\b316_misc_out.txt"
  OPEN outFile FOR OUTPUT AS #1

  PRINT #1, "MAC="; TRIM$(STR$(Add1(2,3)))

  mkval = MAK(DWORD, &H0020??, &H0001??)
  PRINT #1, "MAK="; TRIM$(STR$(mkval))

  PRINT #1, "MAX="; TRIM$(STR$(MAX(3,7,2,9)))
  PRINT #1, "MIN="; TRIM$(STR$(MIN(3,7,2,9)))
  PRINT #1, "MAXS=["; MAX$("abc","abd","ab"); "]"
  PRINT #1, "MINS=["; MIN$("abc","abd","ab"); "]"

  PRINT #1, "MCASE=["; MCASE$("Cats aren't AL.WAYS good."); "]"
#IF %MY_PBVER = 10
  PRINT #1, "MCASEB=["; MCASE$("Cats aren't AL.WAYS good."); "]"
#ELSE
  PRINT #1, "MCASEB=["; MCASE("Cats aren't AL.WAYS good."); "]"
#ENDIF

  PRINT #1, "MID1=["; MID$("PowerBASIC",4,2); "]"
  PRINT #1, "MID2=["; MID$("PowerBASIC",4); "]"
  PRINT #1, "MID3=["; MID$("PowerBASIC",20); "]"
  PRINT #1, "MID4=["; MID$("1234567890",3,-4); "]"
  PRINT #1, "MID5=["; MID$("abcde",-3,2); "]"
  PRINT #1, "MID6=["; MID$("PowerBASIC",4 TO 6); "]"
  PRINT #1, "MID7=["; MID$("PowerBASIC",4 TO 99); "]"

  PRINT #1, "MOD1="; TRIM$(STR$(10 MOD 3))
  PRINT #1, "MOD2="; TRIM$(STR$(1023 MOD 256))
  PRINT #1, "MOD3="; TRIM$(STR$(-10 MOD 3))

  PRINT #1, "MON="; MONTHNAME$(1)
  PRINT #1, "MON12="; MONTHNAME$(12)
#IF %MY_PBVER = 10
  PRINT #1, "MONB="; MONTHNAME$(1)
#ELSE
  PRINT #1, "MONB="; MONTHNAME(1)
#ENDIF

  PRINT #1, "METRICS="; TRIM$(STR$(METRICS(SCROLL.HORZ)))

  mkPath = ENVIRON$("TEMP") + "\b316_misc_mk"
  ON ERROR GOTO mkeh
  MKDIR mkPath
  PRINT #1, "MKNEW="; TRIM$(STR$(ERR))
  MKDIR mkPath
  PRINT #1, "MKDUP="; TRIM$(STR$(ERR))
  ON ERROR GOTO 0

  #IF %MY_PBVER = 10
    PRINT #1, "METHOD=ok"
  #ELSE
    nGot = Box_DoubleIt(bx, 21)
    IF nGot<>42 THEN INCR nFails
    PRINT #1, "METHOD=ok"
  #ENDIF

  PRINT #1, "batch316 misc ok"
  PRINT #1, "FAILURES: "; LTRIM$(STR$(nFails))
  CLOSE #1
  FUNCTION = 0
mkeh:
  RESUME NEXT
END FUNCTION
