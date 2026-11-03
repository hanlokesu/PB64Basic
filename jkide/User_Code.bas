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
%JKCOMP_RUN_PBWIN10 = %IDC_Userbase + 15   ' compile (sync) then run - PBWin 10 (HAN 2026-11-01)
%JKCOMP_RUN_PB64    = %IDC_Userbase + 16   ' compile then run - PB64Basic
%JKCOMP_RUN_PBXB64  = %IDC_Userbase + 17   ' compile then run - PBXB64
%JKCOMP_RUN_DEFAULT = %IDC_Userbase + 18   ' compile (sync, default compiler) then run
%JKCOMP_PBHELP  = %IDC_Userbase + 19   ' open the PowerBASIC help file (PB.chm) via hh.exe (HAN 2026-11-03)
%JKIDE_Open     = 101                  ' WM_COPYDATA dwData: (re)open file in JK-IDE (VCS protocol)
%JKIDE_Save     = 103                  ' WM_COPYDATA dwData: save active file in JK-IDE (VCS protocol, HAN 2026-11-01)

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

' ============ Open JKIDE.ini in JK-IDE itself (HAN 2026-10-28) ============
' JK-IDE's VCS protocol (see Addon\VCS_Stub.bas): WM_COPYDATA with
' dwData = %IDE_Open (101) and lpData = ASCIIZ file path opens/reopens
' the file in the IDE editor. @hw.main is the IDE main window handle.
SUB JK_OpenIni()
  LOCAL iniFile AS STRING
  LOCAL cds AS COPYDATASTRUCT
  LOCAL fp AS ASCIIZ * 1024
  iniFile = JK_IniFile$()
  IF LEN(iniFile) = 0 THEN
    JK_SendOutput "Compilers: cannot locate JKIDE.ini (userdll.dll folder).", &HFF
    EXIT SUB
  END IF
  IF ISFALSE ISFILE(iniFile) THEN
    JK_SendOutput "Compilers: " + iniFile + " not found - run setup.bat first.", &HFF
    EXIT SUB
  END IF
  IF @hw.main = 0 THEN
    JK_SendOutput "Compilers: JK-IDE main window handle unavailable.", &HFF
    EXIT SUB
  END IF
  fp = iniFile
  cds.dwData = %JKIDE_Open
  cds.cbData = LEN(fp) + 1
  cds.lpData = VARPTR(fp)
  SendMessage @hw.main, %WM_COPYDATA, LEN(cds), VARPTR(cds)
END SUB

SUB JK_OpenPbHelp()
' Open the PowerBASIC help file named by PbHelpFile in JKIDE.ini. The entry is
' always the .CHM (never the .CHW); the matching PB.chw kept in the same folder
' is loaded by hh.exe by itself as the merged index. The path is wrapped in
' double quotes so a path containing spaces still parses as one argument.
  LOCAL helpFile AS STRING
  LOCAL zVerb AS ASCIIZ * 32
  LOCAL zFile AS ASCIIZ * 260
  LOCAL zPar  AS ASCIIZ * 2048
  helpFile = JK_IniValue$("Compilers", "PbHelpFile")
  IF LEN(helpFile) = 0 THEN
    JK_SendOutput "Compilers: key PbHelpFile missing in " + JK_IniFile$ + ". Edit JKIDE.ini, then restart JK-IDE.", &HFF
    EXIT SUB
  END IF
  IF ISFALSE ISFILE(helpFile) THEN
    JK_SendOutput "Compilers: help file not found: " + helpFile + ". Check PbHelpFile in JKIDE.ini.", &HFF
    EXIT SUB
  END IF
  zVerb = "open"
  zFile = "hh.exe"
  zPar  = $DQ + helpFile + $DQ
  ShellExecuteA 0, zVerb, zFile, zPar, "", %SW_SHOW
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
' MDI children are JK_PBFrameClass frames kept in z-order; the topmost
' frame is the active document and its direct child is the Scintilla.
  LOCAL hMDI AS DWORD, hChild AS DWORD, hSci AS DWORD
  LOCAL cls AS ASCIIZ * 64
  hMDI = @hw.mdi
  IF hMDI = 0 THEN
    hMDI = GetWindow(@hw.main, %GW_CHILD)
    DO WHILE hMDI
      GetClassName hMDI, cls, 64
      IF UCASE$(cls) = "MDICLIENT" THEN EXIT LOOP
      hMDI = GetWindow(hMDI, %GW_HWNDNEXT)
    LOOP
  END IF
  IF hMDI = 0 THEN EXIT FUNCTION
  hChild = GetWindow(hMDI, %GW_CHILD)
  DO WHILE hChild
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
    hChild = GetWindow(hChild, %GW_HWNDNEXT)
  LOOP
END FUNCTION

' ---- hover error tooltip (native yellow) - HAN 2026-11-01 ----

TYPE JK_SCN
  hwndFrom AS LONG
  idFrom AS LONG
  code AS LONG
  position AS LONG
  ch AS LONG
  modifiers AS LONG
  modificationType AS LONG
  text AS LONG
END TYPE

GLOBAL gErrLine AS LONG
GLOBAL gErrText AS STRING
GLOBAL gSciHwnd AS DWORD
GLOBAL gSciOldProc AS DWORD

FUNCTION JK_FrameProc(BYVAL hWnd AS DWORD, BYVAL wMsg AS LONG, BYVAL wParam AS DWORD, BYVAL lParam AS LONG) AS LONG
' Subclass kept as a plain passthrough. The inline error is rendered with a
' Scintilla annotation inside JK_MarkError (light-pink background, black
' text, shown immediately - no hover, dwell, timer or calltip).
  FUNCTION = CallWindowProc(gSciOldProc, hWnd, wMsg, wParam, lParam)
END FUNCTION

SUB JK_MarkError(BYVAL ln AS LONG, BYVAL errText AS STRING)
' Jump to the error line, place a red roundrect marker on it and show an
' inline annotation directly under the line: light-pink background, black
' text, displayed automatically without hover or timer. The annotation text
' is the pure compiler message after the "(line:col):" location part.
  LOCAL hSci AS DWORD
  LOCAL annText AS STRING, p AS LONG, annOff AS LONG
  hSci = JK_EditorScintilla()
  IF ISTRUE IsWindow(hSci) THEN
    IF gSciHwnd <> hSci OR gSciOldProc = 0 THEN
      gSciHwnd = hSci
      gSciOldProc = SetWindowLong(GetParent(hSci), %GWL_WNDPROC, CODEPTR(JK_FrameProc))
    END IF
    gErrLine = ln
    gErrText = errText
    ' red roundrect marker (accepted behaviour, kept unchanged)
    SendMessage hSci, %SCI_MARKERDELETEALL, 0, 0
    SendMessage hSci, %SCI_MARKERDEFINE, 1, %SC_MARK_ROUNDRECT
    SendMessage hSci, %SCI_MARKERSETFORE, 1, RGB(255, 0, 0)
    SendMessage hSci, %SCI_MARKERSETBACK, 1, RGB(255, 0, 0)
    ' annotation style: reuse the container offset; default to 256 when unset
    annOff = SendMessage(hSci, %SCI_ANNOTATIONGETSTYLEOFFSET, 0, 0)
    IF annOff = 0 THEN annOff = 256
    SendMessage hSci, %SCI_ANNOTATIONSETSTYLEOFFSET, annOff, 0
    SendMessage hSci, %SCI_STYLESETFORE, annOff, RGB(0, 0, 0)
    SendMessage hSci, %SCI_STYLESETBACK, annOff, RGB(255, 220, 220)
    ' pure message text after the "):" location marker
    annText = errText
    p = INSTR(annText, "):")
    IF p > 0 THEN annText = LTRIM$(MID$(annText, p + 2))
    SendMessage hSci, %SCI_ANNOTATIONCLEARALL, 0, 0
    SendMessage hSci, %SCI_ANNOTATIONSETTEXT, ln - 1, STRPTR(annText)
    SendMessage hSci, %SCI_ANNOTATIONSETSTYLE, ln - 1, 0
    SendMessage hSci, %SCI_ANNOTATIONSETVISIBLE, %ANNOTATION_STANDARD, 0
    SendMessage hSci, %SCI_GOTOLINE, ln - 1, 0
    SendMessage hSci, %SCI_MARKERADD, ln - 1, 1
  END IF
END SUB

SUB JK_ClearAnnotations()
' Remove every inline annotation from the active editor. Called at the start
' of each compile so a successful build clears the previous error bar.
  LOCAL hSci AS DWORD
  hSci = JK_EditorScintilla()
  IF ISTRUE IsWindow(hSci) THEN
    SendMessage hSci, %SCI_ANNOTATIONCLEARALL, 0, 0
  END IF
END SUB

FUNCTION JK_SourceLine$(BYVAL f AS STRING, BYVAL ln AS LONG) AS STRING
' read one source line (1-based) from the active .bas to render the
' "Line N: <source>" follow-up line in the same red as the error line.
  LOCAL ff AS LONG, s AS STRING, n AS LONG
  IF ISFALSE ISFILE(f) THEN EXIT FUNCTION
  ff = FREEFILE
  OPEN f FOR INPUT AS #ff
  n = 0
  WHILE NOT EOF(ff)
    LINE INPUT #ff, s
    INCR n
    IF n = ln THEN
      CLOSE #ff
      FUNCTION = TRIM$(s)
      EXIT FUNCTION
    END IF
  WEND
  CLOSE #ff
END FUNCTION

FUNCTION JK_RunCompiler(BYVAL which AS LONG) AS LONG
  LOCAL f AS STRING, exe AS STRING, wd AS STRING, args AS STRING
  LOCAL runtime AS STRING
  LOCAL compname AS STRING
  LOCAL jbat AS STRING, jlog AS STRING, jcmd AS STRING, jlogtxt AS STRING
  LOCAL ff AS LONG
  LOCAL i AS LONG, j AS LONG, ln AS LONG, col AS LONG
  LOCAL k AS LONG, j2 AS LONG, k2 AS LONG
  LOCAL rest AS STRING, outline AS STRING
  LOCAL errMain AS STRING, cat AS STRING, code AS STRING
  LOCAL exitcode AS LONG
  LOCAL cnt_err AS LONG
  LOCAL last_ln AS LONG
  LOCAL prepath AS STRING
  f = JK_ActiveFile$()
  IF LEN(f) = 0 THEN
    JK_SendOutput "Compilers: no active file with a full path. Save the file first, then retry.", &H00C040
    FUNCTION = 0
    EXIT FUNCTION
  END IF
  JK_ClearAnnotations
  SELECT CASE which
    CASE %JKCOMP_PBWIN10
      ' Delegate to JK-IDE's own compile pipeline (F9 / IDM_COMPILE=1501): the IDE
      ' parses the PBWin .log itself, shows errors in the Output window and
      ' marks the offending line in the editor + jumps to it, exactly like the
      ' original compile button. No external SHELL needed.
      SendMessage FindWindow("PBWindowClass:0", ""), %WM_COMMAND, 1501, 0   'IDM_COMPILE
      FUNCTION = -2   ' asynchronous native pipeline - result not knowable here
      EXIT FUNCTION
    CASE %JKCOMP_PB64
      compname = "PB64Basic"
      exe = JK_IniValue$("Compilers", "Pb64Exe")
      wd  = JK_IniValue$("Compilers", "Pb64Cwd")
      runtime = JK_IniValue$("Compilers", "Pb64Runtime")
      IF LEN(exe) = 0 THEN
        JK_SendOutput "Compilers: key Pb64Exe missing in " + JK_IniFile$() + ". Edit JKIDE.ini (or run setup.bat again), then restart JK-IDE.", &HFF
        FUNCTION = 0
        EXIT FUNCTION
      END IF
      IF LEN(runtime) = 0 THEN runtime = "pb_runtime_x64.obj"
      prepath = "set PATH=C:\Program Files\LLVM\bin;%PATH%" + $CRLF   ' clang must be findable by the LLVM backend
      args = "build " + CHR$(34) + f + CHR$(34) + " --exe --target x86_64-pc-windows-msvc --runtime-lib " + runtime
    CASE %JKCOMP_PBXB64
      compname = "PBXB64"
      exe = JK_IniValue$("Compilers", "Pbxb64Exe")
      wd  = JK_IniValue$("Compilers", "Pbxb64Cwd")
      IF LEN(exe) = 0 THEN
        JK_SendOutput "Compilers: key Pbxb64Exe missing in " + JK_IniFile$() + ". Edit JKIDE.ini (or run setup.bat again), then restart JK-IDE.", &HFF
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
  jcmd = "@echo off" + $CRLF + prepath + "cd /d " + CHR$(34) + wd + CHR$(34) + $CRLF + CHR$(34) + exe + CHR$(34) + " " + args + " > " + CHR$(34) + jlog + CHR$(34) + " 2>&1" + $CRLF + "echo EXITCODE=%errorlevel% >> " + CHR$(34) + jlog + CHR$(34)
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
      IF INSTR(outline, "warning") > 0 OR INSTR(outline, "Warning") > 0 THEN
        JK_SendOutput outline, &H0096C8
        ITERATE DO
      END IF
      ln = 0
      col = 0
      j = INSTR(outline, "line ")
      IF j = 0 THEN j = INSTR(outline, "Line ")
      IF j > 0 THEN ln = VAL(MID$(outline, j + 5))
      IF ln = 0 THEN
        k = 0
        j = INSTR(outline, "(")
        DO WHILE j > 0
          k = j
          j = INSTR(k + 1, outline, "(")
        LOOP
        IF k > 0 THEN ln = VAL(MID$(outline, k + 1))
      END IF
      IF ln = 0 AND which = %JKCOMP_PBXB64 THEN
        ' PBXB64 shape: "<file>:<ln>:<col>: error ..." - parse after the ".bas:"
        k = INSTR(outline, ".bas:")
        IF k > 0 THEN
          k = k + 5
          ln = VAL(MID$(outline, k))
          col = ln
          k = k + LEN(TRIM$(STR$(ln)))
          IF MID$(outline, k, 1) = ":" THEN
            k = k + 1
            col = VAL(MID$(outline, k))
          END IF
        END IF
      END IF
      IF ln = 0 THEN ITERATE DO   ' summary line w/o a real source location - skip it
      cnt_err = cnt_err + 1
      IF ln > 0 THEN last_ln = ln
      IF which = %JKCOMP_PB64 THEN
        ' PB64 shape: "Error: <cat>: ... at line <ln>:<col>: <msg>" - build the
        ' PBWin10-shaped main line with the fork's REAL error category + text
        ' (each compiler has its own error codes/messages - keep them real and
        ' unify only the shape: Error <exitcode> in <file>(<ln>:<col>):  <cat>: <msg>)
        cat = ""
        j2 = INSTR(outline, "Error: ")
        IF j2 > 0 THEN
          cat = MID$(outline, j2 + 7)
          k2 = INSTR(cat, ":")
          IF k2 > 0 THEN cat = LEFT$(cat, k2 - 1)
        END IF
        rest = MID$(outline, j + 5)
        IF VAL(rest) > 0 THEN
          ln = VAL(rest)
          rest = LTRIM$(MID$(rest, LEN(TRIM$(STR$(VAL(rest)))) + 1))
          IF LEFT$(rest, 1) = ":" THEN
            rest = MID$(rest, 2)
            IF VAL(rest) > 0 THEN
              col = VAL(rest)
              rest = LTRIM$(MID$(rest, LEN(TRIM$(STR$(VAL(rest)))) + 1))
              IF LEFT$(rest, 1) = ":" THEN rest = MID$(rest, 2)
            END IF
          END IF
        END IF
        rest = LTRIM$(rest)
        IF LEN(cat) > 0 THEN
          outline = "Error " + TRIM$(STR$(exitcode)) + " in " + f + "(" + TRIM$(STR$(ln)) + ":" + FORMAT$(col, "000") + "):  " + cat + ": " + rest
        ELSE
          outline = "Error " + TRIM$(STR$(exitcode)) + " in " + f + "(" + TRIM$(STR$(ln)) + ":" + FORMAT$(col, "000") + "):  " + rest
        END IF
        errMain = outline
        JK_SendOutput outline, &HFF
        JK_SendOutput "[PB64Basic] " + rest + " (exitcode=" + TRIM$(STR$(exitcode)) + ")", &H808080
        JK_SendOutput "Line " + TRIM$(STR$(ln)) + ": " + JK_SourceLine$(f, ln), &HFF
      ELSEIF which = %JKCOMP_PBXB64 THEN
        ' PBXB64 shape: "<file>:<ln>:<col>: error [CODE]: <msg>: [<src>]" - build
        ' the PBWin10-shaped main line with the fork's REAL error code + text:
        '   Error <CODE> in <file>(<ln>:<col>):  <msg>
        code = ""
        j = INSTR(outline, " error [")
        IF j > 0 THEN
          code = MID$(outline, j + 8)
          k = INSTR(code, "]:")
          IF k > 0 THEN code = LEFT$(code, k - 1)
        END IF
        rest = outline
        j = INSTR(rest, " error ")
        IF j > 0 THEN rest = MID$(rest, j + 7)
        j = INSTR(rest, "]:")
        IF j > 0 THEN rest = MID$(rest, j + 2)
        rest = LTRIM$(rest)
        k = INSTR(rest, " [")
        IF k > 0 AND RIGHT$(rest, 1) = "]" THEN rest = LEFT$(rest, k - 1)
        IF RIGHT$(rest, 1) = ":" THEN rest = LEFT$(rest, LEN(rest) - 1)
        IF LEN(code) > 0 THEN
          outline = "Error " + code + " in " + f + "(" + TRIM$(STR$(ln)) + ":" + FORMAT$(col, "000") + "):  " + rest
        ELSE
          outline = "Error 1 in " + f + "(" + TRIM$(STR$(ln)) + ":" + FORMAT$(col, "000") + "):  " + rest
        END IF
        errMain = outline
        JK_SendOutput outline, &HFF
        JK_SendOutput "[PBXB64] " + rest + " (exitcode=" + TRIM$(STR$(exitcode)) + ")", &H808080
        JK_SendOutput "Line " + TRIM$(STR$(ln)) + ": " + JK_SourceLine$(f, ln), &HFF
      ELSE
        JK_SendOutput outline, &HFF
      END IF
    ELSEIF INSTR(outline, "warning") > 0 OR INSTR(outline, "Warning") > 0 THEN
      JK_SendOutput outline, &H0096C8
    ELSE
      JK_SendOutput outline, &H808080
    END IF
  LOOP

  ' ---- fake the JK-IDE error jump for external compilers ----
  IF (which = %JKCOMP_PB64 OR which = %JKCOMP_PBXB64) AND cnt_err > 0 AND last_ln > 0 THEN
    JK_MarkError last_ln, errMain
  END IF

  ' ---- verdict line ----
  IF cnt_err = 0 AND exitcode = 0 THEN
    JK_SendOutput "Compile OK (EXITCODE=0)", &HFF00
    FUNCTION = 0
  ELSE
    JK_SendOutput "Compile FAILED - EXITCODE=" + TRIM$(STR$(exitcode)) + " (" + TRIM$(STR$(cnt_err)) + " error line(s))", &HFF
    FUNCTION = exitcode
  END IF
END FUNCTION

' ============ Compile & Run entries (HAN 2026-11-01) ============
' The four new "- Run" menu items compile the ACTIVE file synchronously
' (so the result is known), and only when the compile succeeded do they
' start the produced executable.  PBWin's SHELL statement waits for the
' child to terminate, so the program is launched via "cmd /c start"
' which returns immediately and detaches the executable.

SUB JK_RunCompiledExe(BYVAL exePath AS STRING)
  LOCAL exeDir AS STRING
  IF ISFILE(exePath) THEN
    JK_SendOutput "Compile OK - starting: " + exePath, &HFF00
    exeDir = exePath
    DO WHILE INSTR(exeDir, "\") > 0
      exeDir = MID$(exeDir, INSTR(exeDir, "\") + 1)
    LOOP
    exeDir = LEFT$(exePath, LEN(exePath) - LEN(exeDir))
    SHELL "cmd.exe /c cd /d " + CHR$(34) + exeDir + CHR$(34) + " && start """" " + CHR$(34) + exePath + CHR$(34)
  ELSE
    JK_SendOutput "Compile OK but executable not found: " + exePath, &HFF
  END IF
END SUB

FUNCTION JK_CompilePBWinSync() AS LONG
' Synchronous PBWin 10 command-line compile: writes the same bat+log
' mechanism as JK_RunCompiler and returns the compile EXITCODE (0 = OK).
' The produced .exe lands in the compiler cwd (PbwinCwd from JKIDE.ini).
  LOCAL f AS STRING, exe AS STRING, wd AS STRING, inc AS STRING
  LOCAL jbat AS STRING, jlog AS STRING, jcmd AS STRING, jlogtxt AS STRING
  LOCAL ff AS LONG, i AS LONG, j AS LONG, exitcode AS LONG
  LOCAL outline AS STRING
  LOCAL cnt_err AS LONG, last_ln AS LONG, ln AS LONG, k AS LONG
  LOCAL stem AS STRING, pblog AS STRING, pbdet AS STRING
  LOCAL srcDir AS STRING, errMain AS STRING
  f = JK_ActiveFile$()
  IF LEN(f) = 0 THEN
    JK_SendOutput "Compilers: no active file with a full path. Save the file first, then retry.", &H00C040
    FUNCTION = -1
    EXIT FUNCTION
  END IF
  JK_ClearAnnotations
  exe = JK_IniValue$("Compilers", "PbwinExe")
  wd  = JK_IniValue$("Compilers", "PbwinCwd")
  inc = JK_IniValue$("Compilers", "PbwinInc")
  IF LEN(exe) = 0 THEN
    JK_SendOutput "Compilers: key PbwinExe missing in " + JK_IniFile$() + ". Edit JKIDE.ini (or run setup.bat again), then restart JK-IDE.", &HFF
    FUNCTION = -1
    EXIT FUNCTION
  END IF
  IF LEN(wd) = 0 THEN wd = "C:\PBWin10\bin"
  IF LEN(inc) = 0 THEN inc = "C:\PBWin10\WINAPI"
  jbat = ENVIRON$("TEMP") + "\jk_pbwin_run.bat"
  jlog = ENVIRON$("TEMP") + "\jk_pbwin_run.log"
  srcDir = f
  DO WHILE INSTR(srcDir, "\") > 0
    srcDir = MID$(srcDir, INSTR(srcDir, "\") + 1)
  LOOP
  srcDir = LEFT$(f, LEN(f) - LEN(srcDir))
  jcmd = "@echo off" + $CRLF + "cd /d " + CHR$(34) + srcDir + CHR$(34) + $CRLF _
       + CHR$(34) + exe + CHR$(34) + " /I" + inc + " /L /Q " + CHR$(34) + f + CHR$(34) _
       + " > " + CHR$(34) + jlog + CHR$(34) + " 2>&1" + $CRLF _
       + "echo EXITCODE=%errorlevel% >> " + CHR$(34) + jlog + CHR$(34)
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
  ' PBWin writes its DETAILED log (Error lines) next to the compile
  ' target in the compiler cwd, not to stdout under /Q - read it in.
  stem = f
  DO WHILE INSTR(stem, "\") > 0
    stem = MID$(stem, INSTR(stem, "\") + 1)
  LOOP
  IF INSTR(stem, ".") > 0 THEN stem = LEFT$(stem, INSTR(stem, ".") - 1)
  pblog = srcDir + "\" + stem + ".log"
  IF ISFILE(pblog) THEN
    ff = FREEFILE
    OPEN pblog FOR BINARY AS #ff
    pbdet = SPACE$(LOF(ff))
    IF LEN(pbdet) THEN GET$ #ff, LOF(ff), pbdet
    CLOSE #ff
    KILL pblog
    IF INSTR(pbdet, "Error") > 0 THEN jlogtxt = pbdet + jlogtxt
  END IF
  exitcode = -1
  i = INSTR(jlogtxt, "EXITCODE=")
  IF i > 0 THEN
    exitcode = VAL(MID$(jlogtxt, i + 9))
    IF i > 1 THEN jlogtxt = LEFT$(jlogtxt, i - 1)
  END IF
  JK_SendOutput "==== PBWin 10 (sync) - " + f + " ====", &H808080
  i = 1
  cnt_err = 0
  last_ln = 0
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
      errMain = outline
      ln = 0
      j = INSTR(outline, "Line ")
      IF j = 0 THEN j = INSTR(outline, "line ")
      IF j > 0 THEN ln = VAL(MID$(outline, j + 5))
      IF ln = 0 THEN
        k = 0
        j = INSTR(outline, "(")
        DO WHILE j > 0
          k = j
          j = INSTR(k + 1, outline, "(")
        LOOP
        IF k > 0 THEN ln = VAL(MID$(outline, k + 1))
      END IF
      IF ln > 0 THEN last_ln = ln
      JK_SendOutput outline, &HFF
    ELSEIF LEFT$(LTRIM$(outline), 5) = "Line " THEN
      ' PBWin "  Line N: <source>" follow-up line -> same red as the error
      JK_SendOutput LTRIM$(outline), &HFF
      ln = VAL(MID$(LTRIM$(outline), 6))
      IF ln > 0 THEN last_ln = ln
    ELSEIF INSTR(outline, "warning") > 0 OR INSTR(outline, "Warning") > 0 THEN
      JK_SendOutput outline, &H0096C8
    ELSE
      JK_SendOutput outline, &H808080
    END IF
  LOOP

  ' jump to the first error line in the editor (same as PB64/PBXB64)
  IF cnt_err > 0 AND last_ln > 0 THEN JK_MarkError last_ln, errMain

  FUNCTION = exitcode
END FUNCTION

FUNCTION JK_CompileAndRun(BYVAL which AS LONG) AS LONG
' Dispatch for the four "- Run" menu items: compile synchronously, and
' only on success start the executable produced by that compiler.
  LOCAL rc AS LONG
  LOCAL f AS STRING, stem AS STRING, exePath AS STRING, wd AS STRING, srcDir AS STRING
  LOCAL cds AS COPYDATASTRUCT
  LOCAL fp AS ASCIIZ * 1024
  f = JK_ActiveFile$()
  IF LEN(f) = 0 THEN
    JK_SendOutput "Compilers: no active file with a full path. Save the file first, then retry.", &H00C040
    FUNCTION = 0
    EXIT FUNCTION
  END IF

  ' --- save the active file to disk FIRST: the compile below reads the .bas
  '     from disk, so an unsaved edit in the editor would otherwise compile
  '     STALE bytes and run a program the user already changed (HAN 2026-11-01) ---
  fp = f
  cds.dwData = %JKIDE_Save
  cds.cbData = LEN(fp) + 1
  cds.lpData = VARPTR(fp)
  IF @hw.main > 0 THEN SendMessage @hw.main, %WM_COPYDATA, LEN(cds), VARPTR(cds)

  SELECT CASE which
    CASE %JKCOMP_RUN_PBWIN10, %JKCOMP_RUN_DEFAULT
      wd = JK_IniValue$("Compilers", "PbwinCwd")
      IF LEN(wd) = 0 THEN wd = "C:\PBWin10\bin"
      stem = f
      DO WHILE INSTR(stem, "\") > 0
        stem = MID$(stem, INSTR(stem, "\") + 1)
      LOOP
      IF INSTR(stem, ".") > 0 THEN stem = LEFT$(stem, INSTR(stem, ".") - 1)
      srcDir = LEFT$(f, LEN(f) - LEN(stem) - 4)
      exePath = srcDir + "\" + stem + ".exe"
      IF JK_DeleteStaleExe(exePath, stem) = 0 THEN
        FUNCTION = -1
        EXIT FUNCTION
      END IF
      rc = JK_CompilePBWinSync()
      IF rc = 0 THEN
        JK_RunCompiledExe exePath
      ELSE
        JK_DeleteStaleExe exePath, stem
        JK_SendOutput "PBWin 10 compile failed - not running (EXITCODE=" + TRIM$(STR$(rc)) + ")", &HFF
      END IF
    CASE %JKCOMP_RUN_PB64
      exePath = LEFT$(f, LEN(f) - 4) + ".exe"
      stem = f
      DO WHILE INSTR(stem, "\") > 0
        stem = MID$(stem, INSTR(stem, "\") + 1)
      LOOP
      IF INSTR(stem, ".") > 0 THEN stem = LEFT$(stem, INSTR(stem, ".") - 1)
      IF JK_DeleteStaleExe(exePath, stem) = 0 THEN
        FUNCTION = -1
        EXIT FUNCTION
      END IF
      rc = JK_RunCompiler(%JKCOMP_PB64)
      IF rc = 0 THEN
        JK_RunCompiledExe exePath
      ELSE
        JK_DeleteStaleExe exePath, stem
        JK_SendOutput "PB64Basic compile failed - not running (EXITCODE=" + TRIM$(STR$(rc)) + ")", &HFF
      END IF
    CASE %JKCOMP_RUN_PBXB64
      exePath = LEFT$(f, LEN(f) - 4) + ".exe"
      stem = f
      DO WHILE INSTR(stem, "\") > 0
        stem = MID$(stem, INSTR(stem, "\") + 1)
      LOOP
      IF INSTR(stem, ".") > 0 THEN stem = LEFT$(stem, INSTR(stem, ".") - 1)
      IF JK_DeleteStaleExe(exePath, stem) = 0 THEN
        FUNCTION = -1
        EXIT FUNCTION
      END IF
      rc = JK_RunCompiler(%JKCOMP_PBXB64)
      IF rc = 0 THEN
        JK_RunCompiledExe exePath
      ELSE
        JK_DeleteStaleExe exePath, stem
        JK_SendOutput "PBXB64 compile failed - not running (EXITCODE=" + TRIM$(STR$(rc)) + ")", &HFF
      END IF
  END SELECT
  FUNCTION = 1
END FUNCTION

FUNCTION JK_DeleteStaleExe(BYVAL exePath AS STRING, BYVAL stem AS STRING) AS LONG
' KILL-FIRST user rule: the stale product EXE must be GONE before compiling,
' so a failed compile leaves no EXE and can never run the old binary.
' Returns TRUE when the exe is gone (deleted or never existed).
  IF ISFILE(exePath) THEN
    ON ERROR GOTO KillFail
    KILL exePath
    ON ERROR GOTO 0
  END IF
  IF ISFILE(exePath) THEN
    ' locked by a still-running process -> force-kill it, then retry delete
    SHELL "cmd.exe /c taskkill /F /IM " + stem + ".exe >nul 2>&1"
    ON ERROR GOTO KillFail
    KILL exePath
    ON ERROR GOTO 0
  END IF
  IF ISFILE(exePath) THEN
    JK_SendOutput "Run: cannot delete stale EXE (still locked by a running process?). Close it and retry: " + exePath, &HFF
    FUNCTION = 0
    EXIT FUNCTION
  END IF
  FUNCTION = 1
  EXIT FUNCTION
KillFail:
  RESUME NEXT
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
