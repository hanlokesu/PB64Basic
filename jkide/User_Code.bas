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

FUNCTION JK_RunCompiler(BYVAL which AS LONG) AS LONG
  LOCAL f AS STRING, exe AS STRING, wd AS STRING, args AS STRING
  LOCAL jbat AS STRING, jlog AS STRING, jcmd AS STRING, jlogtxt AS STRING
  LOCAL z AS ASCIIZ * 40000
  LOCAL ff AS LONG
  f = JK_ActiveFile$()
  IF LEN(f) = 0 THEN
    MSGBOX "No active file with a full path. Save the file first, then retry.", %MB_TOPMOST, "Compilers"
    FUNCTION = 0
    EXIT FUNCTION
  END IF
  SELECT CASE which
    CASE %JKCOMP_PBWIN10
      exe = "C:\PBWin10\bin\PBWin.exe"
      wd  = "C:\PBWin10\bin"
      args = "/IC:\PBWin10\WINAPI /L /Q " + CHR$(34) + f + CHR$(34)
    CASE %JKCOMP_PB64
      exe = "C:\Users\HANLO\Doubao\chats\2026-09-05\new-chat\powerbasilisk_test\pbsrc\target\release\PB64Basic.exe"
      wd  = "C:\Users\HANLO\Doubao\chats\2026-09-05\new-chat\powerbasilisk_test\pbsrc"
      args = "build " + CHR$(34) + f + CHR$(34) + " --exe --target x86_64-pc-windows-msvc --runtime-lib pb_runtime_x64.obj"
    CASE %JKCOMP_PBXB64
      exe = "C:\Users\HANLO\Doubao\chats\2026-09-05\new-chat\PBXB64\V6\PBX-Editor\PBXB64\PBXB64.exe"
      wd  = "C:\Users\HANLO\Doubao\chats\2026-09-05\new-chat\PBXB64\V6\PBX-Editor"
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
  IF ISFILE(jlog) THEN
    ff = FREEFILE
    OPEN jlog FOR BINARY AS #ff
    jlogtxt = SPACE$(LOF(ff))
    IF LEN(jlogtxt) THEN GET$ #ff, LOF(ff), jlogtxt
    CLOSE #ff
    IF LEN(jlogtxt) > 1500 THEN jlogtxt = RIGHT$(jlogtxt, 1500)
    z = jlogtxt
  ELSE
    z = "(no compiler output)"
  END IF
  KILL jbat
  MSGBOX z, %MB_TOPMOST, "Compilers - result"
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
