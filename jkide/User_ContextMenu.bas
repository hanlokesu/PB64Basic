'***********************************************************************************************
' this file is for integration of user code (aka "addon") into JK-IDE, you may freely add to it
' or adapt it to your needs. You are given the chance here of adding your own features to this
' IDE.

' If you managed to crash the IDE, delete "userdll.dll" from the IDE directory and rename
' "userdll.dll.old" back to "userdll.dll" (which is always the last version, you just overwrote)
' restart the IDE and try again.

' you should add own code only in special areas of this file embordered by these two lines:
#utility begin usercode area +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
#utility end usercode area +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
' they were added to separate "no change" areas (left for better readabilty) from areas for
' user code.

' Don't edit the lines with metastatements for conditional compiling (#if ..., #endif),
' otherwise you will raise compile errors.
'***********************************************************************************************

#if 0
function usercontextmenu alias "usercontextmenu" (byval hmenu as dword) export as long
'***********************************************************************************************
' this function is called each time the contextmenu is shown,
' hmenu is a handle to the context menu, you are free to modify this menu here ...
'***********************************************************************************************
#endif
#utility begin usercode area +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++

  ' ===== Right-click menu replicating the PB_IDE tab menu (HAN 2026-11-06) =====
  ' Guard against duplicate stacking: usercontextmenu fires on every
  ' right-click, so only append when our first item is not already present.
  IF JK_MenuHasId(hmenu, %JKCTX_COPYPATH) = 0 THEN
    AppendMenu hmenu, %MF_SEPARATOR, 0, ""
    AppendMenu hmenu, %MF_STRING, %JKCTX_COPYPATH, "Copy Path"
    AppendMenu hmenu, %MF_STRING, %JKCTX_COPYNAME, "Copy File Name"
    AppendMenu hmenu, %MF_STRING, %JKCTX_COPY,     "Copy"
    AppendMenu hmenu, %MF_STRING, %JKCTX_OPENWITH, "Open with..."
    AppendMenu hmenu, %MF_STRING, %JKCTX_EXPLORER, "Open in Explorer"
    AppendMenu hmenu, %MF_STRING, %JKCTX_RENAME,   "Rename..."
    AppendMenu hmenu, %MF_SEPARATOR, 0, ""
    AppendMenu hmenu, %MF_STRING, %JKCTX_CLOSETAB, "Close Tab"
  END IF

#utility end usercode area +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
#if 0
end function
#endif
