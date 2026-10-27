'***********************************************************************************************
' this file is for integration of user code (aka "addon") into JK-IDE, you may freely add to it
' or adapt it to your needs. You are given the chance here of adding your own features to this
' IDE.

' If you managed to crash the IDE, delete "userdll.dll" from the IDE directory and rename
' "userdll.dll.old" back to "userdll.dll" (which is always the last version, you just overwrote)
' restart the IDE and try again to produce working code.

' you should add own code only in special areas of this file embordered by these two lines:
#utility begin usercode area +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
#utility end usercode area +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
' they were added to separate "no change" areas (left for better readabilty) from areas for
' user code.

' Don´t edit the lines with metastatements for conditional compiling (#if ..., #endif),
' otherwise you will raise compile errors.
'***********************************************************************************************

#if 0
function usercommand alias "usercommand" (byval cmd as long, byval hsci as dword) as long
'***********************************************************************************************
' user command processing, %IDC_Userbase + 3 and following
' cmd = command identifier, hsci = handle to active scintilla window
'***********************************************************************************************
local n as long
local flag as long
local d$


  select case as long cmd
#endif
#utility begin usercode area +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++


    case %IDC_Userbase + 3
'***********************************************************************************************
' code for john here ...
'***********************************************************************************************
local ztext as asciiz * 260
      n = sendmessage(hsci, %SCI_GETCURLINE, 256, VARPTR(ztext))

      if n = 0 then
        sendmessage hsci, %SCI_WORDLEFT , 0, 0

      else
        if tally(left$(ztext, n), any $tab + $spc) = n then
          sendmessage hsci, %SCI_Home , 0, 0

        else
          sendmessage hsci, %SCI_WORDLEFT , 0, 0
        end if
      end if



    case %IDC_Userbase + 4






    CASE %JKCOMP_PBWIN10, %JKCOMP_PB64, %JKCOMP_PBXB64
      JK_RunCompiler(cmd)
#utility end usercode area +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
#if 0
  end select


end function
#endif
