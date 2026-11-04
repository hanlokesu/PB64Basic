#COMPILE EXE
#DIM ALL
' batch316_test.bas - M-family umbrella regression probe (batch 316).
'   Representative checks across all four groups: MKI$/CVI packer round trip,
'   MAT identity, MID$ function, variadic MAX, and an unattached MENU bar.
'   File-based output (PRINT #1) so it compiles under both PBWin10 (GUI
'   subsystem: bare PRINT = Error 477) and the fork.
#IF %DEF(%PB_REVISION)
    #IF (%PB_REVISION AND &H0FF00) = &H1000
        %MY_PBVER = 10
    #ELSE
        %MY_PBVER = 0
    #ENDIF
#ELSE
    %MY_PBVER = 0
#ENDIF

FUNCTION PBMAIN() AS LONG
  LOCAL nFails AS LONG
  LOCAL vLong AS LONG
  LOCAL sPack AS STRING
  LOCAL mm(1 TO 2, 1 TO 2) AS LONG
  LOCAL hBar AS LONG
  LOCAL outFile AS STRING
  nFails = 0
  outFile = ENVIRON$("TEMP") + "\b316_out.txt"
  OPEN outFile FOR OUTPUT AS #1

  ' --- MK group: MKI$ / CVI round trip ---
  vLong = 4242
  sPack = MKI$(vLong)
  PRINT #1, "MK_LEN="; LTRIM$(STR$(LEN(sPack)))
  IF LEN(sPack) <> 2 OR CVI(sPack, 1) <> 4242 THEN INCR nFails

  ' --- MAT group: 2x2 identity ---
  MAT mm = IDN
  PRINT #1, "MAT_D1="; LTRIM$(STR$(mm(1,1))); " D2="; LTRIM$(STR$(mm(1,2)))
  IF mm(1,1) <> 1 OR mm(1,2) <> 0 OR mm(2,2) <> 1 THEN INCR nFails

  ' --- MISC group: MID$ function and variadic MAX ---
  PRINT #1, "MID=["; MID$("PowerBASIC", 6, 3); "]"
  IF MID$("PowerBASIC", 6, 3) <> "BAS" THEN INCR nFails
  PRINT #1, "MAX="; LTRIM$(STR$(MAX(3, 9, 5)))
  IF MAX(3, 9, 5) <> 9 THEN INCR nFails

  ' --- MENU group: unattached bar, handle observable ---
  MENU NEW BAR TO hBar
  PRINT #1, "MENU_H="; LTRIM$(STR$(hBar))
  IF hBar = 0 THEN INCR nFails

  PRINT #1, "batch316 test ok"
  PRINT #1, "FAILURES: "; LTRIM$(STR$(nFails))
  CLOSE #1
  FUNCTION = 0
END FUNCTION
