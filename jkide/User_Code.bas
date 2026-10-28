'***********************************************************************************************
' this file is for integration of user code (aka "addon") into JK-IDE, you may freely add to it
' or adapt it to your needs. You are given the chance here of adding your own features to this
' IDE.

' If you managed to crash the IDE, delete "userdll.dll" from the IDE directory and rename
' "userdll.dll.old" back to "userdll.dll" (which is always the last version, you just overwrote)
' restart the IDE and try again to produce working code.

' you should add own code only in special areas of this file embordered by these two lines:
#utility begin usercode area +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++

' ============ Compiler menu helpers - HAN 2026-10-27 ============
%JKCOMP_PBWIN10 = %IDC_Userbase + 11
%JKCOMP_PB64    = %IDC_Userbase + 12
%JKCOMP_PBXB64  = %IDC_Userbase + 13
%JKCOMP_JKINI   = %IDC_Userbase + 14   ' open JKIDE.ini for editing (HAN 2026-10-28)

' ============ JKIDE.ini reader - HAN 2026-10-28 ============
' Every compiler path comes from JKIDE.ini (the file next to userdll.dll),
' so the addon works on any customer machine without recompiling.  The
' JK-IDE setup.bat writes this file during installation.
' NOTE: GetModuleHandleA / GetModuleFileNameA are already DECLAREd by
' Win32API.inc (included by Userdll.bas) - do NOT re-declare them here
' (PBWin Error 417).

FUNCTION JK_IniFile$() AS STRING
  LOCAL hMod AS DWORD
  LOCAL buf AS ASCIIZ * 2048
  LOCAL s AS STRING
  LOCAL i AS LONG
  hMod = GetModuleHandleA("userdll.dll")
  IF hMod = 0 THEN hMod = GetModuleHandleA("userdll.DLL")
  IF ISTRUE hMod THEN
    IF GetModuleFileNameA(hMod, BYVAL VARPTR(buf), 2048) > 0 THEN
      s = buf
      i = LEN(s)
      DO WHILE i > 0
        IF MID$(s, i, 1) = "\" THEN EXIT LOOP
        DECR i
      LOOP
      IF i > 0 THEN FUNCTION = LEFT$(s, i) + "JKIDE.ini"
    END IF
  END IF
END FUNCTION

FUNCTION JK_IniValue$(BYVAL cfgSection AS STRING, BYVAL cfgKey AS STRING) AS STRING
  LOCAL iniFile AS STRING
  LOCAL ff AS LONG
  LOCAL iniLine AS STRING
  LOCAL sect AS STRING
  LOCAL k AS STRING
  LOCAL v AS STRING
  LOCAL eq AS LONG
  iniFile = JK_IniFile$()
  IF LEN(iniFile) = 0 THEN EXIT FUNCTION
  IF ISFALSE ISFILE(iniFile) THEN EXIT FUNCTION
  ff = FREEFILE
  OPEN iniFile FOR INPUT AS #ff
  WHILE NOT EOF(ff)
    LINE INPUT #ff, iniLine
    iniLine = TRIM$(iniLine)
    IF LEN(iniLine) > 0 AND LEFT$(iniLine, 1) <> ";" THEN
      IF LEFT$(iniLine, 1) = "[" THEN
        IF RIGHT$(iniLine, 1) = "]" THEN sect = TRIM$(MID$(iniLine, 2, LEN(iniLine) - 2))
      ELSE
        eq = INSTR(iniLine, "=")
        IF eq > 0 THEN
          k = TRIM$(LEFT$(iniLine, eq - 1))
          IF UCASE$(sect) = UCASE$(cfgSection) AND UCASE$(k) = UCASE$(cfgKey) THEN
            v = TRIM$(MID$(iniLine, eq + 1))
            IF LEN(v) >= 2 AND LEFT$(v, 1) = CHR$(34) AND RIGHT$(v, 1) = CHR$(34) THEN
              v = MID$(v, 2, LEN(v) - 2)
            END IF
            CLOSE #ff
            FUNCTION = v
            EXIT FUNCTION
          END IF
        END IF
      END IF
    END IF
  WEND
  CLOSE #ff
END FUNCTION

' ============ Open JKIDE.ini for editing (HAN 2026-10-28) ============
' jk-ide.exe ignores command-line file arguments (restores last session),
' so open the ini with its associated editor (usually Notepad) instead.
SUB JK_OpenIni()
  LOCAL iniFile AS STRING
  LOCAL op AS ASCIIZ * 8
  LOCAL nf AS ASCIIZ * 1024
  LOCAL ip AS ASCIIZ * 4
  LOCAL dir AS ASCIIZ * 4
  iniFile = JK_IniFile$()
  IF LEN(iniFile) = 0 THEN
    JK_SendOutput "Compilers: cannot locate JKIDE.ini (userdll.dll folder).", &HFF4040
    EXIT SUB
  END IF
  IF ISFALSE ISFILE(iniFile) THEN
    JK_SendOutput "Compilers: " + iniFile + " not found - run setup.bat first.", &HFF4040
    EXIT SUB
  END IF
  op = "open"
  nf = iniFile
  ip = ""
  dir = ""
  ShellExecute 0, op, nf, ip, dir, %SW_SHOWNORMAL
END SUB

' ============ Output window protocol (JK-IDE VCS convention) ============
%IDE_OUTPUT      = 104                                   'WM_COPYDATA dwData: send string to outputwindow

FUNCTION JK_ActiveFile$() AS STRING
  LOCAL buf AS ASCIIZ * 1024
  LOCAL s AS STRING
  LOCAL i AS LONG
  LOCAL j AS LONG
  LOCAL hm AS DWORD
  hm = FindWindow("PBWindowClass:0", "")
  IF ISTRUE IsWindow(hm) THEN
    GetWindowText hm, buf, 1024
    s = buf
    i = INSTR(s, "[")
    j = INSTR(s, "]")
    IF i > 0 AND j > i THEN
      FUNCTION = MID$(s, i + 1, j - i - 1)
    ELSE
      FUNCTION = ""
    END IF
  END IF
END FUNCTION

FUNCTION JK_SendOutput(BYVAL txt AS STRING, BYVAL rgbVal AS LONG) AS LONG
' write one text line to the JK-IDE Output window, using the official
' protocol from Userdll.bas "how to" (sendmessage @hw.out, %WM_Settext,
' 2, STRPTR(txt) = append line).  JK-IDE colors a line RED automatically
' if it contains "Error " (case sensitive), so error lines are sent as
' plain text; any other color is requested with the CHR$(1)+8-hex prefix.
' Fallback: the WM_COPYDATA %IDE_Output protocol (VCS_Stub style).
  LOCAL cds AS COPYDATASTRUCT
  LOCAL hwMain AS DWORD
  LOCAL body AS STRING
  IF INSTR(txt, "Error ") > 0 THEN
    body = txt
  ELSE
    body = CHR$(1) + HEX$(rgbVal, 8) + txt
  END IF
  IF ISTRUE @hw.out THEN
    sendmessage @hw.out, %WM_Settext, 2, BYVAL STRPTR(body)
    FUNCTION = 1
  ELSE
    hwMain = FindWindow("PBWindowClass:0", "")
    IF ISTRUE IsWindow(hwMain) THEN
      cds.dwData = %IDE_OUTPUT
      cds.lpData = STRPTR(body)
      cds.cbData = LEN(body) + 1
      SendMessage hwMain, %WM_COPYDATA, LEN(cds), VARPTR(cds)
      FUNCTION = 1
    END IF
  END IF
END FUNCTION

FUNCTION JK_EditorScintilla() AS DWORD
' locate the Scintilla editor window of the active MDI child, so we can
' fake the JK-IDE error jump for compilers the IDE does not know about.
  LOCAL hMDI AS DWORD, hChild AS DWORD, hSci AS DWORD
  LOCAL cls AS ASCIIZ * 64
  hMDI = @hw.mdi
  IF hMDI = 0 THEN EXIT FUNCTION
  hChild = SendMessage(hMDI, %WM_MDIGETACTIVE, 0, 0)
  IF hChild = 0 THEN EXIT FUNCTION
  GetClassName hChild, cls, 64
  IF UCASE$(cls) = "SCINTILLA" THEN
    FUNCTION = hChild
    EXIT FUNCTION
  END IF
  hSci = GetWindow(hChild, %GW_CHILD)
  DO WHILE hSci
    GetClassName hSci, cls, 64
    IF UCASE$(cls) = "SCINTILLA" THEN
      FUNCTION = hSci
      EXIT FUNCTION
    END IF
    hSci = GetWindow(hSci, %GW_HWNDNEXT)
  LOOP
END FUNCTION

SUB JK_MarkError(BYVAL ln AS LONG)
' jump the active editor to line ln and put a red roundrect marker on it.
  LOCAL hSci AS DWORD
  hSci = JK_EditorScintilla()
  IF ISTRUE IsWindow(hSci) THEN
    SendMessage hSci, %SCI_MARKERDEFINE, 1, %SC_MARK_ROUNDRECT
    SendMessage hSci, %SCI_MARKERSETFORE, 1, RGB(255, 64, 64)
    SendMessage hSci, %SCI_MARKERSETBACK, 1, RGB(255, 64, 64)
    SendMessage hSci, %SCI_GOTOLINE, ln - 1, 0          'Scintilla lines are 0-based
    SendMessage hSci, %SCI_MARKERADD, ln - 1, 1
  END IF
END SUB

FUNCTION JK_RunCompiler(BYVAL which AS LONG) AS LONG
  LOCAL f AS STRING, exe AS STRING, wd AS STRING, args AS STRING
  LOCAL runtime AS STRING
  LOCAL compname AS STRING
  LOCAL jbat AS STRING, jlog AS STRING, jcmd AS STRING, jlogtxt AS STRING
  LOCAL ff AS LONG
  LOCAL i AS LONG, j AS LONG, ln AS LONG
  LOCAL rest AS STRING, outline AS STRING
  LOCAL exitcode AS LONG
  LOCAL cnt_err AS LONG
  LOCAL last_ln AS LONG
  f = JK_ActiveFile$()
  IF LEN(f) = 0 THEN
    JK_SendOutput "Compilers: no active file with a full path. Save the file first, then retry.", &H00C040
    FUNCTION = 0
    EXIT FUNCTION
  END IF
  SELECT CASE which
    CASE %JKCOMP_PBWIN10
      ' Delegate to JK-IDE's own compile pipeline (F9 / IDM_COMPILE=1501): the IDE
      ' parses the PBWin .log itself, shows errors in the Output window and
      ' marks the offending line in the editor + jumps to it, exactly like the
      ' original compile button. No external SHELL needed.
      SendMessage FindWindow("PBWindowClass:0", ""), %WM_COMMAND, 1501, 0   'IDM_COMPILE
      FUNCTION = 1
      EXIT FUNCTION
    CASE %JKCOMP_PB64
      compname = "PB64Basic"
      exe = JK_IniValue$("Compilers", "Pb64Exe")
      wd  = JK_IniValue$("Compilers", "Pb64Cwd")
      runtime = JK_IniValue$("Compilers", "Pb64Runtime")
      IF LEN(exe) = 0 THEN
        JK_SendOutput "Compilers: key Pb64Exe missing in " + JK_IniFile$() + ". Edit JKIDE.ini (or run setup.bat again), then restart JK-IDE.", &HFF4040
        FUNCTION = 0
        EXIT FUNCTION
      END IF
      IF LEN(runtime) = 0 THEN runtime = "pb_runtime_x64.obj"
      args = "build " + CHR$(34) + f + CHR$(34) + " --exe --target x86_64-pc-windows-msvc --runtime-lib " + runtime
    CASE %JKCOMP_PBXB64
      compname = "PBXB64"
      exe = JK_IniValue$("Compilers", "Pbxb64Exe")
      wd  = JK_IniValue$("Compilers", "Pbxb64Cwd")
      IF LEN(exe) = 0 THEN
        JK_SendOutput "Compilers: key Pbxb64Exe missing in " + JK_IniFile$() + ". Edit JKIDE.ini (or run setup.bat again), then restart JK-IDE.", &HFF4040
        FUNCTION = 0
        EXIT FUNCTION
      END IF
      args = "--gui -O3 -o " + CHR$(34) + LEFT$(f, LEN(f) - 4) + ".exe" + CHR$(34) + " " + CHR$(34) + f + CHR$(34)
    CASE ELSE
      FUNCTION = 0
      EXIT FUNCTION
  END SELECT
  jbat = ENVIRON$("TEMP") + "\jk_comp.bat"
  jlog = ENVIRON$("TEMP") + "\jk_comp.log"
  jcmd = "@echo off" + $CRLF + "cd /d " + CHR$(34) + wd + CHR$(34) + $CRLF + CHR$(34) + exe + CHR$(34) + " " + args + " > " + CHR$(34) + jlog + CHR$(34) + " 2>&1" + $CRLF + "echo EXITCODE=%errorlevel% >> " + CHR$(34) + jlog + CHR$(34)
  ff = FREEFILE
  OPEN jbat FOR OUTPUT AS #ff
  PRINT #ff, jcmd
  CLOSE #ff
  SHELL CHR$(34) + jbat + CHR$(34)
  jlogtxt = ""
  IF ISFILE(jlog) THEN
    ff = FREEFILE
    OPEN jlog FOR BINARY AS #ff
    jlogtxt = SPACE$(LOF(ff))
    IF LEN(jlogtxt) THEN GET$ #ff, LOF(ff), jlogtxt
    CLOSE #ff
  END IF
  KILL jbat
  KILL jlog

  ' ---- parse EXITCODE ----
  exitcode = -1
  i = INSTR(jlogtxt, "EXITCODE=")
  IF i > 0 THEN
    exitcode = VAL(MID$(jlogtxt, i + 9))
    IF i > 1 THEN jlogtxt = LEFT$(jlogtxt, i - 1)
  END IF

  ' ---- send compiler output to the JK-IDE Output window ----
  JK_SendOutput "==== " + compname + " - " + f + " ====", &H808080
  i = 1
  cnt_err = 0
  DO WHILE i <= LEN(jlogtxt)
    j = INSTR(i, jlogtxt, $CRLF)
    IF j = 0 THEN j = INSTR(i, jlogtxt, $LF)
    IF j = 0 THEN
      outline = MID$(jlogtxt, i)
      i = LEN(jlogtxt) + 1
    ELSE
      outline = MID$(jlogtxt, i, j - i)
      IF MID$(jlogtxt, j, 2) = $CRLF THEN i = j + 2 ELSE i = j + 1
    END IF
    IF LEN(outline) = 0 THEN ITERATE DO

    IF INSTR(outline, "Error") > 0 OR INSTR(outline, "error") > 0 THEN
      cnt_err = cnt_err + 1
      ln = 0
      j = INSTR(outline, "line ")
      IF j > 0 THEN ln = VAL(MID$(outline, j + 5))
      IF ln > 0 THEN last_ln = ln
      IF which = %JKCOMP_PB64 AND ln > 0 THEN
        ' rewrite "Parse error at line N: msg" into the PBWin shape JK-IDE
        ' jumps on:  Error 1 in <file>(N:000):  msg
        rest = MID$(outline, j + 5)
        IF VAL(rest) > 0 THEN rest = LTRIM$(MID$(rest, LEN(TRIM$(STR$(VAL(rest)))) + 1))
        IF LEFT$(rest, 1) = ":" THEN rest = MID$(rest, 2)
        rest = LTRIM$(rest)
        outline = "Error " + TRIM$(STR$(cnt_err)) + " in " + f + "(" + TRIM$(STR$(ln)) + ":000):  " + rest
      END IF
      JK_SendOutput outline, &HFF4040
    ELSEIF INSTR(outline, "warning") > 0 OR INSTR(outline, "Warning") > 0 THEN
      JK_SendOutput outline, &H0096C8
    ELSE
      JK_SendOutput outline, &H808080
    END IF
  LOOP

  ' ---- fake the JK-IDE error jump for external compilers ----
  IF (which = %JKCOMP_PB64 OR which = %JKCOMP_PBXB64) AND cnt_err > 0 AND last_ln > 0 THEN
    JK_MarkError last_ln
  END IF

  ' ---- verdict line ----
  IF cnt_err = 0 AND exitcode = 0 THEN
    JK_SendOutput "Compile OK (EXITCODE=0)", &H00A040
  ELSE
    JK_SendOutput "Compile FAILED - EXITCODE=" + TRIM$(STR$(exitcode)) + " (" + TRIM$(STR$(cnt_err)) + " error line(s))", &HFF4040
  END IF
  FUNCTION = 1
END FUNCTION
#utility end usercode area +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
' they were added to separate "no change" areas (left for better readabilty) from areas for
' user code.

' Don't edit the lines with metastatements for conditional compiling (#if ..., #endif),
' otherwise you will raise compile errors.
'***********************************************************************************************

#if 0
'***********************************************************************************************
' add own equates, declares, include files and procedures here ...
'***********************************************************************************************
#endif
#utility begin usercode area +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++


'#include "Shlwapi.inc"                                'needed for GetMenuPosFromID


thread function printtext(byval sp as dword)
'***********************************************************************************************
' process and print p$
'***********************************************************************************************
local p as string ptr


  p = sp                                              'get string pointer
'  msgbox @p                                           'show what to print

' printing code here ....


  if p then
    @p = ""                                             'clean up
  end if


end function


#utility end usercode area +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
