'***********************************************************************************************
' this file is for integration of user code (aka "addon") into JK-IDE, you may freely add to it
' or adapt it to your needs. You are given the chance here of adding your own features to this
' IDE.

' If you managed to crash the IDE, delete "userdll.dll" from the IDE directory and rename
' "userdll.dll.old" back to "userdll.dll" (which is always the last version, you just overwrote)
' restart the IDE and try again to produce working code.

' you should add own code only in special areas of this file embordered by these two lines:
#utility begin usercode area +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++

  ' ===== Compilers submenu (HAN 2026-10-27, startup install) =====
  LOCAL hcomp AS DWORD
  STATIC did AS LONG
  IF did = 0 THEN
    hcomp = CreatePopupMenu()
    IF hcomp THEN
      AppendMenu hcomp, %MF_STRING, %JKCOMP_PBWIN10, "PBWin 10"
      AppendMenu hcomp, %MF_STRING, %JKCOMP_PB64, "PB64Basic"
      AppendMenu hcomp, %MF_STRING, %JKCOMP_PBXB64, "PBXB64"
      AppendMenu hcomp, %MF_SEPARATOR, 0, ""
      AppendMenu hcomp, %MF_STRING, %IDM_COMPILE, "Compile (default)"
      AppendMenu hcomp, %MF_STRING, %JKCOMP_JKINI, "JKIDE.INI"
      InsertMenu hmenu, %ID_TOOLS, %MF_BYCOMMAND OR %MF_POPUP, hcomp, "Compilers"
      did = 1
    END IF
  END IF
#utility end usercode area +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
' they were added to separate "no change" areas (left for better readabilty) from areas for
' user code.

' Don�t edit the lines with metastatements for conditional compiling (#if ..., #endif),
' otherwise you will raise compile errors.
'***********************************************************************************************

#if 0
function usersetup alias "usersetup" (byval hmenu as dword, haccel as dword, byval p as JK_Handles ptr) export as long
'***********************************************************************************************
' setup a new accelerator table, if you want
' you may modfiy the main menu using it�s handle (hmenu)
'***********************************************************************************************


'  ModifyMenu hmenu, %IDM_HELP, %MF_Bycommand, %IDM_HELP, "test"


'***********************************************************************************************
' create a new accelerator table
'***********************************************************************************************
'local n as long
'static acc() as accelapi
'
'
'  redim acc(1 to 100)                                 'ensure this is enough for you ...
#endif
#utility begin usercode area +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++


'  DeleteMenu hmenu, %IDM_CALCULATOR, %MF_Bycommand
'  DeleteMenu hmenu, %IDM_NOTEPAD, %MF_Bycommand
'  DeleteMenu hmenu, %IDM_MSPAINT, %MF_Bycommand


  AccelAdd(%FVIRTKEY OR %FCONTROL,           %VK_LEFT, %IDC_Userbase + 3)
                                                      'change the Ctrl + Left behavior
                                                      'by assigning a new command id and do further processing
                                                      'usercommand


'***********************************************************************************************
' you may simply ADD new assignments here. But if you want to redefine assignments, you MUST
' setup the entire accelerator table (copy it form userdll.bas and adapt here) and you MUST
' leave with the following code ...
'***********************************************************************************************


'  redim preserve acc(1 to n)
'  haccel = CreateAcceleratorTable(acc(1), n)
'  function = 1
'  exit function


#utility end usercode area +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
#if 0
end function
#endif
