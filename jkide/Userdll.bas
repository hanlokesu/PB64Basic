'***********************************************************************************************
' Please don´t edit this file, because it will get overwritten with updates !
' This file contains a structure (JK_Handles) with most window handles in JK-IDE, code samples
' in a "how to" section further down and other code you could study.
'***********************************************************************************************

#COMPILER PBWIN 10
#COMPILE DLL "Userdll.dll"
#DIM ALL

#utility compile10                                    'use PBWIN 10
#utility roca2                                        'use José Roca´s includes (=> V 2.05)
#utility shell "userdll.bat"                          'copy userdll.dll

#INCLUDE ONCE "Win32API.inc"                          'JR includes
#INCLUDE ONCE "Scintilla.inc"                         'JR includes
#INCLUDE ONCE "SciCtrl.inc"                           'JR includes


TYPE JK_Handles                                       'window handles
  main    AS DWORD                                    'IDE main window
  mdi     AS DWORD                                    'mdi client

  pro     AS DWORD                                    'project explorer
  prop    AS DWORD                                    'project properties
  propc   AS DWORD                                    'child dialog or project properties window
  def     AS DWORD                                    'definition window
  out     AS DWORD                                    'outputwindow
  find    AS DWORD                                    'find results window
  dep     AS DWORD                                    'dependencies window
  tree    AS DWORD                                    'code tree window
  var     AS DWORD                                    'variables window
  wait    AS DWORD                                    'wait animation
  todo    AS DWORD                                    'todo window (docked)
  todo2   AS DWORD                                    'todo window (undocked)
  list    AS DWORD                                    'code finder
  listc   AS DWORD                                    'code finder combo
  fd      AS DWORD                                    'find dialog
  rd      AS DWORD                                    'replace dialog
  flag    AS DWORD                                    '(for internal use only)
  time    AS DWORD                                    'time panel
  date    AS DWORD                                    'date panel
  comp    AS DWORD                                    'file compare
  wnd     AS DWORD                                    'handles + identifiers window (visual designer)
  mac     AS DWORD                                    'keyboard macros window
  lib     AS DWORD                                    'library
  libtb   AS DWORD                                    'library toolbar
  libc    AS DWORD                                    'library toolbar combo
  nav     AS DWORD                                    'navigation
  book    AS DWORD                                    'bookmarks

  hfont   AS DWORD                                    'IDE font
END TYPE


global hw    as JK_Handles ptr
GLOBAL hInst AS DWORD                                 'instance handle


%Enum_level = 2                                       'use all enums in jk_user_enum.bas (please don´t edit)
#include "Jk_User_Enum.bas"                           'command and other identifiers (JK-IDE)
#include "Jk_VD_Enum.bas"                             'command and other identifiers (JK-VD)


#include "..\Visualdesigner\controls\link.inc"
'%MLGSLL = 1               'if you use the payed, registered version, otherwise comment this line out
#IF %DEF(%MLGSLL)
  #LINK "..\visualdesigner\controls\mlg.SLL"
#endif
#INCLUDE "..\visualdesigner\controls\MLG.INC"



macro AccelAdd (modifier, vkey, idc)
'***********************************************************************************************
' add one single hotkey definition to accelapi array
'***********************************************************************************************


  n = n + 1
  if n mod 100 = 0 then
    REDIM PRESERVE acc(1 to n + 100)
  end if

  acc(n).fvirt = modifier
  acc(n).key   = vkey
  acc(n).cmd   = idc


end macro


'***********************************************************************************************
' add own equates, declares, include files and procedures here ...
'***********************************************************************************************
#include "User_Code.bas"


'***********************************************************************************************
'***********************************************************************************************


FUNCTION LIBMAIN (BYVAL hInstance   AS LONG, _
                  BYVAL fwdReason   AS LONG, _
                  BYVAL lpvReserved AS LONG) AS LONG
'***********************************************************************************************
' dll entry point
'***********************************************************************************************


  SELECT CASE fwdReason
    CASE %DLL_PROCESS_ATTACH
      hInst = hInstance
      FUNCTION = 1   'success!


    CASE %DLL_PROCESS_DETACH
      FUNCTION = 1   'success!


    CASE %DLL_THREAD_ATTACH
      FUNCTION = 1   'success!


    CASE %DLL_THREAD_DETACH
      FUNCTION = 1   'success!


  END SELECT


END FUNCTION


'***********************************************************************************************
'***********************************************************************************************


function usersetup alias "usersetup" (byval hmenu as dword, haccel as dword, byval p as JK_Handles ptr) export as long
'***********************************************************************************************
' setup a new accelerator table, if you want
' you may modfiy the main menu using it´s handle (hmenu)
'***********************************************************************************************


  hw = p                                              'save pointer to IDE handles


'  ModifyMenu hmenu, %IDM_HELP, %MF_Bycommand, %IDM_HELP, "test"


'***********************************************************************************************
' this is generic example of setting a user command
'***********************************************************************************************
  InsertMenu (hmenu, %idc_todo, 0, %IDC_Userbase + 2, "Rearrange Labels")
  InsertMenu (hmenu, %idc_todo, 0, %IDC_Userbase + 1, "User Command(s)" + $tab + "Ctrl+Q")
  InsertMenu (hmenu, %idc_todo, %MF_SEPARATOR, 0, "")


'  function = 0                                        'leave acceleratortable as it is
'exit function                                         'exit here, if you don´t want your own hotkeys


'***********************************************************************************************
' create a new accelerator table
'***********************************************************************************************
local n as long
static acc() as accelapi


  redim acc(1 to 100)                                 'ensure this is enough for you ...


'***********************************************************************************************
' your code here ...
'***********************************************************************************************
#include "User_Setup.bas"


'***********************************************************************************************
' this is generic example of setting a user command hotkey
'***********************************************************************************************
  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_Q, %IDC_Userbase + 1)              'set user command hotkey


'***********************************************************************************************
' function + special keys
'***********************************************************************************************

'help
  AccelAdd(%FVIRTKEY,                         %VK_F1, %IDM_HELP)          'F1   - context help
  AccelAdd(%FVIRTKEY OR %FSHIFT,              %VK_F1, %IDM_SDKHELP)       '#F1  - sdk help
  AccelAdd(%FVIRTKEY OR %FCONTROL,            %VK_F1, %IDM_DEFHELP)       '^F1  - definiton
  AccelAdd(%FVIRTKEY OR %FCONTROL OR %FSHIFT, %VK_F1, %IDM_CODEHELP)      '^F1  - search code files
  AccelAdd(%FVIRTKEY OR %FALT,                %VK_F1, %IDM_FORUMHELP)     '@F1  - pb threads
  AccelAdd(%FVIRTKEY OR %FSHIFT OR %FALT,     %VK_F1, %IDM_DEPENDHELP)    '#@F1 - dependencies

'save
  AccelAdd(%FVIRTKEY,                         %VK_F2, %IDM_SAVE)          'F2   - save file
  AccelAdd(%FVIRTKEY OR %FSHIFT,              %VK_F2, %IDM_SAVEASP)       '#F2  - save project as
  AccelAdd(%FVIRTKEY OR %FCONTROL,            %VK_F2, %IDM_SAVEP)         '^F2  - save project
  AccelAdd(%FVIRTKEY OR %FALT,                %VK_F2, %IDM_SAVEAS)        '@F2  - save file as

'open
  AccelAdd(%FVIRTKEY,                         %VK_F3, %IDM_OPEN)          'F3   - open file
  AccelAdd(%FVIRTKEY OR %FSHIFT,              %VK_F3, %IDM_LICENSE)       '#F3  - License Manager
  AccelAdd(%FVIRTKEY OR %FCONTROL,            %VK_F3, %IDM_OPENP)         '^F3  - open project
  AccelAdd(%FVIRTKEY OR %FALT,                %VK_F3, %IDM_OPENI)         '@F3  - open include files

'window
  AccelAdd(%FVIRTKEY,                         %VK_F4, %IDM_Split)         'F4   - toggle split state
  AccelAdd(%FVIRTKEY OR %FSHIFT,              %VK_F4, %IDM_SWITCHWINDOW)  '#F4  - switch file
  AccelAdd(%FVIRTKEY OR %FCONTROL,            %VK_F4, %IDM_CLOSEFILE)     '^F4  - close file
  AccelAdd(%FVIRTKEY OR %FALT,                %VK_F4, %IDM_EXIT)          '@F4  - exit

'tools
  AccelAdd(%FVIRTKEY,                         %VK_F5, %IDM_EXPLORER)      'F5   - open explorer (current dir)
  AccelAdd(%FVIRTKEY OR %FSHIFT,              %VK_F5, %IDM_TOOLSMENUHEADER + 1) '#F5 - tool #1
  AccelAdd(%FVIRTKEY OR %FCONTROL,            %VK_F5, %IDM_TOOLSMENUHEADER + 2) '^F5 - toll #2
  AccelAdd(%FVIRTKEY OR %FALT,                %VK_F5, %IDM_TOOLSMENUHEADER + 3) '@F5 - tool #3

'***********************************************************************************************
' f6: some are assigned inside scintilla
'***********************************************************************************************
'line commands
'  AccelAdd(%FVIRTKEY,                         %VK_F6, %IDM_PASTE          'done with key assign in scintilla
'  AccelAdd(%FVIRTKEY OR %FSHIFT,              %VK_F6, %IDM_DUPLINE        'done with key assign in scintilla
  AccelAdd(%FVIRTKEY OR %FCONTROL,            %VK_F6, %IDM_GOCLEAR)        '^F6  - clear jump history
  AccelAdd(%FVIRTKEY OR %FALT,                %VK_F6, %IDM_SAVEPOS)        '@F6  - save current pos to history

'VD tools
  AccelAdd(%FVIRTKEY,                         %VK_F7, %IDM_PBFORMS)       'F7   - PBFORMS/VD
  AccelAdd(%FVIRTKEY OR %FSHIFT,              %VK_F7, %IDM_VD_Preview)    '#F7  - VD Preview
  AccelAdd(%FVIRTKEY OR %FCONTROL,            %VK_F7, %IDM_RM)            '^F7  - Resource Manager
  AccelAdd(%FVIRTKEY OR %FALT,                %VK_F7, %IDM_Navigation)    '@F7  - Toggle Navigation
  AccelAdd(%FVIRTKEY OR %FSHIFT or %FALT,     %VK_F7, %IDM_Allhandles)    '#@F7 - Handles and Ids Window

'special functions
  AccelAdd(%FVIRTKEY,                         %VK_F8, %IDM_CODEFINDER)    'F8   - codefinder
  AccelAdd(%FVIRTKEY OR %FSHIFT,              %VK_F8, %IDC_Var)           '#F8  - variables window
  AccelAdd(%FVIRTKEY OR %FCONTROL,            %VK_F8, %IDC_Tree)          '^F8  - code tree
  AccelAdd(%FVIRTKEY OR %FALT,                %VK_F8, %IDC_Project)       '@F8  - project window
  AccelAdd(%FVIRTKEY OR %FSHIFT OR %FALT,     %VK_F8, %IDC_Prop)          '#@F8 - properties window

'run
  AccelAdd(%FVIRTKEY,                         %VK_F9, %IDM_COMPILEANDRUN) 'F9   - compile and run
  AccelAdd(%FVIRTKEY OR %FShift,              %VK_F9, %IDM_RUNTOCARET)    '#F9  - run to caret
  AccelAdd(%FVIRTKEY OR %FControl,            %VK_F9, %IDM_EXECUTE)       '#F9  - run last compiled
  AccelAdd(%FVIRTKEY OR %FALT,                %VK_F9, %IDM_COMPILE)       '@F9  - compile

' context menu/ fold / show output/find
  AccelAdd(%FVIRTKEY,                         %VK_F10, %ID_CONTEXT)       'F10  - context menu
  AccelAdd(%FVIRTKEY OR %FSHIFT,              %VK_F10, %IDM_TOGGLE)       '#F10 - toggle current fold
  AccelAdd(%FVIRTKEY OR %FCONTROL,            %VK_F10, %IDC_Output)       '^F10 - output window
  AccelAdd(%FVIRTKEY OR %FALT,                %VK_F10, %IDC_Find)         '@F10 - find window

'debug
  AccelAdd(%FVIRTKEY,                         %VK_F11, %IDM_DEBUG)        'F11  - Debug/step over
  AccelAdd(%FVIRTKEY OR %FSHIFT,              %VK_F11, %IDM_Stepout)      '#F11 - step out )of current procedure)
  AccelAdd(%FVIRTKEY OR %FCONTROL,            %VK_F11, %IDM_Stepin)       '^F11 - step in
  AccelAdd(%FVIRTKEY OR %FALT,                %VK_F11, %IDM_Togglebp)     '@F11 - toggle breakpoint
  AccelAdd(%FVIRTKEY OR %FSHIFT or %FALT,     %VK_F11, %IDM_Togglebpstate) '#@F11 - toggle breakpoint state (in/active)

'bookmarks
  AccelAdd(%FVIRTKEY,                         %VK_F12, %IDM_NEXTBOOKMARK)     'F12  - goto next bookmark
  AccelAdd(%FVIRTKEY OR %FSHIFT,              %VK_F12, %IDM_PREVIOUSBOOKMARK) '#F12 - goto previous bookmark
  AccelAdd(%FVIRTKEY OR %FCONTROL,            %VK_F12, %IDM_BOOKMARKS )       '^F12 - show bookmarks
  AccelAdd(%FVIRTKEY OR %FSHIFT or %FALT,     %VK_F12, %IDM_DELETEBOOKMARKS ) '^F12 - delete bookmarks
  AccelAdd(%FVIRTKEY OR %FALT,                %VK_F12, %IDM_TOGGLEBOOKMARK)   '@F12 - toggle bookmark

  AccelAdd(%FVIRTKEY OR %FALT,                %VK_RETURN, %IDM_GOTOPROCEDURE) 'Goto Procedure, if any

  AccelAdd(%FVIRTKEY,                         %VK_APPS, %IDM_FINDNEXT)    'find next
  AccelAdd(%FVIRTKEY OR %FSHIFT,              %VK_APPS, %IDM_FINDPREV)    'find backwards


  AccelAdd(%FVIRTKEY,                         %VK_TAB, %IDM_SetComment)     'set/adjust comment
  AccelAdd(%FVIRTKEY OR %FCONTROL,            %VK_TAB, %IDM_Commentline)    'insert separator line
  AccelAdd(%FVIRTKEY OR %FSHIFT,              %VK_TAB, %IDK_COMMENT)        'block comment
  AccelAdd(%FVIRTKEY OR %FSHIFT OR %FCONTROL, %VK_TAB, %IDK_UNCOMMENT)      'block uncomment


  AccelAdd(%FVIRTKEY , %VK_Escape,            %IDM_Escape)                   '<Esc> key
  AccelAdd(%FVIRTKEY OR %FCONTROL,            %VK_SPACE, %IDM_AUTOCOMPLETE)  '^Space - Code-tip/Autocomplete
  AccelAdd(%FVIRTKEY OR %FALT,                %VK_SPACE, %IDM_Handleslist)   '@Space - Show handles (VD)


  AccelAdd(%FVIRTKEY OR %FALT,                %VK_PGUP, %IDM_GOTOBEGINPROC)   'Goto to the begining of the current procedure or function
  AccelAdd(%FVIRTKEY OR %FALT,                %VK_PGDN, %IDM_GOTONEXTPROC)    'Goto to the end of the current procedure or function
  AccelAdd(%FVIRTKEY OR %FALT or %FSHIFT,     %VK_PGDN, %IDM_GOTOENDTHISPROC) 'Goto to the end of the current procedure or function
  AccelAdd(%FVIRTKEY OR %FALT or %FSHIFT,     %VK_PGUP, %IDM_GOTOENDPREVPROC) 'Goto to the end of the current procedure or function


  AccelAdd(%FVIRTKEY OR %FALT,                %VK_LEFT, %IDM_GOBACK)      'jump history - go back
  AccelAdd(%FVIRTKEY OR %FALT,                %VK_RIGHT, %IDM_GONEXT)     'jump history - og foreward


'***********************************************************************************************
' ctrl + alpha keys
'***********************************************************************************************

'  %VK_A                                               'mark all

  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_B, %IDM_BLOCKFILL)                 '^B  - Fill block

'  %VK_C                                               'copy

  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_D, %IDM_DUPLINE)                   '^D  - duplicate current line
  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_E, %IDM_COMPILEANDRUN)             '^E  - compile and execute
  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_F, %IDM_FIND)                      '^F  - find
  AccelAdd(%FVIRTKEY OR %FSHIFT OR %FCONTROL, %VK_F, %IDM_FINDC)          '#^F - find caretword
  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_G, %IDM_GOTOLINE)                  '^G  - go to line...
  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_H, %IDM_HOTKEYS)                   '^H  - show hotkeys
  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_I, %IDM_INSERTFILE)                '^I  - insert file
  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_J, %IDM_JK)                        '^J  - test
  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_K, %IDM_SetCommentk)               '^K  - set/adjust comment, block comment (if selection)
  AccelAdd(%FVIRTKEY OR %FSHIFT or %FCONTROL, %VK_K, %IDK_UNCOMMENT)      '^#K - block uncomment
  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_L, %IDM_SELTOLOWERCASE)            '^L  - convert to lower case
  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_M, %IDM_SELTOMIXEDCASE)            '^M  - convert to mixed case
  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_N, %IDM_NEW)                       '^N  - new
  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_O, %IDM_OPEN)                      '^O  - open
  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_P, %IDM_PRINT)                     '^P  - print

  AccelAdd(%FVIRTKEY OR %FCONTROL, 222, %IDM_JK1)                         '^ä  - test

  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_R, %IDM_REPLACE)                   '^R  - find and replace
  AccelAdd(%FVIRTKEY OR %FSHIFT OR %FCONTROL, %VK_R, %IDM_REPLACEC)       '#^R - find and replace (current word)
  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_S, %IDM_SAVE)                      '^S  - Save
  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_T, %IDC_TODO)                      '^T  - todo window
  AccelAdd(%FVIRTKEY OR %FCONTROL or %FSHIFT, %VK_T, %IDM_TESTWINDOW)     '^#T - test window

  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_U, %IDM_SELTOUPPERCASE)            '^U  - convert to upper case

'  %VK_V                                                 'paste

  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_W, %IDM_Highlightword)             '^W  - highlight word
  AccelAdd(%FVIRTKEY OR %FCONTROL or %FSHIFT, %VK_W, %IDM_Highlightword1) '#^W - highlight word (match case)

'  %VK_X                                                 'cut

  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_Y, %IDM_LINEDELETE)                '^Y  - Delete line

' %VK_Z                                               'undo


'***********************************************************************************************
' alt + alpha keys
'***********************************************************************************************

  AccelAdd(%FVIRTKEY OR %FALT, %VK_X, %IDM_EXIT)                          '@X  - Quit the application


'***********************************************************************************************
' keyboard macros (please don´t change these !)
'***********************************************************************************************

  AccelAdd(%FVIRTKEY OR %FALT, %VK_0, %IDM_ALT0)      'Alt+0  - Keyboard Macros
  AccelAdd(%FVIRTKEY OR %FALT, %VK_1, %IDM_ALT1)      'Alt+1  - Keyboard Macros
  AccelAdd(%FVIRTKEY OR %FALT, %VK_2, %IDM_ALT2)      'Alt+2  - Keyboard Macros
  AccelAdd(%FVIRTKEY OR %FALT, %VK_3, %IDM_ALT3)      'Alt+3  - Keyboard Macros
  AccelAdd(%FVIRTKEY OR %FALT, %VK_4, %IDM_ALT4)      'Alt+4  - Keyboard Macros
  AccelAdd(%FVIRTKEY OR %FALT, %VK_5, %IDM_ALT5)      'Alt+5  - Keyboard Macros
  AccelAdd(%FVIRTKEY OR %FALT, %VK_6, %IDM_ALT6)      'Alt+6  - Keyboard Macros
  AccelAdd(%FVIRTKEY OR %FALT, %VK_7, %IDM_ALT7)      'Alt+7  - Keyboard Macros
  AccelAdd(%FVIRTKEY OR %FALT, %VK_8, %IDM_ALT8)      'Alt+8  - Keyboard Macros
  AccelAdd(%FVIRTKEY OR %FALT, %VK_9, %IDM_ALT9)      'Alt+9  - Keyboard Macros
  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_0, %IDM_CTRL0) 'Ctrl+0 - Keyboard Macros
  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_1, %IDM_CTRL1) 'Ctrl+1 - Keyboard Macros
  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_2, %IDM_CTRL2) 'Ctrl+2 - Keyboard Macros
  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_3, %IDM_CTRL3) 'Ctrl+3 - Keyboard Macros
  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_4, %IDM_CTRL4) 'Ctrl+4 - Keyboard Macros
  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_5, %IDM_CTRL5) 'Ctrl+5 - Keyboard Macros
  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_6, %IDM_CTRL6) 'Ctrl+6 - Keyboard Macros
  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_7, %IDM_CTRL7) 'Ctrl+7 - Keyboard Macros
  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_8, %IDM_CTRL8) 'Ctrl+8 - Keyboard Macros
  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_9, %IDM_CTRL9) 'Ctrl+9 - Keyboard Macros


  redim preserve acc(1 to n)
  haccel = CreateAcceleratorTable(acc(1), n)
  function = 1


end function


'***********************************************************************************************


function userinitmenu alias "userinitmenu" (byval hmenu as dword) export as long
'***********************************************************************************************
' this function is called each time the main menu is shown (%WM_INITMENU processing)
' hmenu is a handle to the main menu, you are free to modify this menu here ...
'***********************************************************************************************


'***********************************************************************************************
' your code here ...
'***********************************************************************************************
#include "User_Menu.bas"


end function


'***********************************************************************************************


function usercontextmenu alias "usercontextmenu" (byval hmenu as dword) export as long
'***********************************************************************************************
' this function is called each time the contextmenu is shown,
' hmenu is a handle to the context menu, you are free to modify this menu here ...
'***********************************************************************************************


'***********************************************************************************************
' your code here ...
'***********************************************************************************************
#include "User_ContextMenu.bas"


end function


'***********************************************************************************************


function usercommand alias "usercommand" (byval cmd as long, byval hsci as dword) export as long
'***********************************************************************************************
' user command processing, cmd = command identifier, hsci = handle to active scintilla window
'***********************************************************************************************
local n as long
local flag as long
local d$


  select case as long cmd
    case %IDC_Userbase + 1
      msgbox "You may insert your own text processing" + $crlf + "Please read about userdll in jk-ide help", _
             %MB_Topmost or %MB_Applmodal, "usercommand"

'***********************************************************************************************
' sample code: you find more scintilla stuff in José Rocas´s includes "Scintilla.inc"
'***********************************************************************************************
      d$ = SCI_GetSelText(hsci)                       'get selection
      if len(d$) = 0 then
        flag = 1
        d$ = SCI_GetText(hSci)                        'get all text, if there is no selection
      END IF

'***********************************************************************************************
'  your text processing here ....

'      msgbox d$
'***********************************************************************************************


'      d$ = RTRIM$(d$, CHR$(0))
'      IF flag = 1 THEN
'        SCI_SetText(hsci, d$)                         'set new text (all of it!)

'      else
'        SCI_ReplaceSel(hsci, d$)                      'replace text
'      END IF
'***********************************************************************************************


    case %IDC_Userbase + 2
      d$ = SCI_GetSelText(hsci)                       'get selection
      if len(d$) = 0 then
        flag = 1
        d$ = SCI_GetText(hSci)                        'get all text, if there is no selection
      END IF

      rearrange(d$)

      d$ = RTRIM$(d$, CHR$(0))
      IF flag = 1 THEN
        SCI_SetText(hsci, d$)                         'set new text (all of it!)

      else
        SCI_ReplaceSel(hsci, d$)                      'replace text
      END IF


'***********************************************************************************************
' more own command processing here ...
'***********************************************************************************************
#include "User_Command.bas"


  end select


end function


'***********************************************************************************************


function userformat alias "userformat" (t$, byval usetabs as dword, byval tabsize as dword, _
                                        byval indentsize as dword, byval commentwidth as dword) export as string
'***********************************************************************************************
' format t$, return formatted string, usetabs = 0 don´t use tabs, = 1 use tabs as spaces, 2 = use tabs as tabs
' return formatted text for own processing, return a null string for default (IDE) processing
'***********************************************************************************************


'***********************************************************************************************
' your code here ...
'***********************************************************************************************
#include "User_Format.bas"


  function = t$                                       'return


end function


'***********************************************************************************************


function userprint alias "userprint" (t$, byval hsci as dword) export as long
'***********************************************************************************************
' print t$, hsci is a handle to the current scintilla edit control so you could use scintilla´s
' printing capabilities ...  return 1 for own processing, return 0 for default (IDE) processing
'***********************************************************************************************
local i as dword
static p$                                             'text to print


'***********************************************************************************************
' your code here ...
'***********************************************************************************************
#include "User_Print.bas"


end function


'***********************************************************************************************


function userexit alias "userexit" export as long
'***********************************************************************************************
' JK-IDE exit
'***********************************************************************************************
local hwnd as dword


  hwnd = findwindow("JK_VISDES", "Visual Designer")
  if iswindow(hwnd) then
    postmessage hwnd, %WM_Command, %IDM_VD_EXIT, 0    'close visual designer
  end if


'***********************************************************************************************
' your code here ...
'***********************************************************************************************
#include "User_Exit.bas"


end function


'***********************************************************************************************
'***********************************************************************************************


function vd_accel alias "vd_accel" (haccel as dword) export as long
'***********************************************************************************************
' make accelerator table for visual designer
'***********************************************************************************************
local n as long
static acc() as accelapi


  redim acc(1 to 100)                                 'ensure this is enough for you ...


'***********************************************************************************************
' your code here ...
'***********************************************************************************************
#include "User_VD_Accel.bas"


'***********************************************************************************************
' function + special keys
'***********************************************************************************************

''help
  AccelAdd(%FVIRTKEY,                         %VK_F1, %IDM_VD_HELP)          'F1   - context help
  AccelAdd(%FVIRTKEY OR %FSHIFT,              %VK_F1, %IDM_VD_SDKHELP)       '#F1  - sdk help
'  AccelAdd(%FVIRTKEY OR %FCONTROL,            %VK_F1,            )       '^F1  -
'  AccelAdd(%FVIRTKEY OR %FCONTROL OR %FSHIFT, %VK_F1,            )       '^F1  -
'  AccelAdd(%FVIRTKEY OR %FALT,                %VK_F1,            )       '@F1  -
'  AccelAdd(%FVIRTKEY OR %FSHIFT OR %FALT,     %VK_F1,            )       '#@F1 -

'save
  AccelAdd(%FVIRTKEY,                         %VK_F2, %IDM_VD_SAVEsec)       'F2   - save layout (restorepoint)
'  AccelAdd(%FVIRTKEY OR %FSHIFT,              %VK_F2,            )       '#F2  -
  AccelAdd(%FVIRTKEY OR %FCONTROL,            %VK_F2, %IDM_VD_SAVEtpl)      '^F2  - save template
  AccelAdd(%FVIRTKEY OR %FALT,                %VK_F2, %IDM_VD_SAVEsec2)      '@F2  - save layout (ask for name)

''open
  AccelAdd(%FVIRTKEY,                         %VK_F3, %IDM_VD_OPENsec)        'F3   - load restore point
'  AccelAdd(%FVIRTKEY OR %FSHIFT,              %VK_F3,            )        '#F3
  AccelAdd(%FVIRTKEY OR %FCONTROL,            %VK_F3, %IDM_VD_OPENtpl)       '^F3   - load template

''window
  AccelAdd(%FVIRTKEY,                         %VK_F4, %IDM_VD_Taborder)       'F4   - toggle taborder
'  AccelAdd(%FVIRTKEY OR %FSHIFT,              %VK_F4,             )       '#F4  -
'  AccelAdd(%FVIRTKEY OR %FCONTROL,            %VK_F4,             )       '^F4  -
'  AccelAdd(%FVIRTKEY OR %FALT,                %VK_F4, %IDM_VD_EXIT)          '@F4  - exit

''tools
  AccelAdd(%FVIRTKEY,                         %VK_F5, %IDM_VD_EXPLORER)       'F5   - open explorer (current dir)
'  AccelAdd(%FVIRTKEY OR %FSHIFT,              %VK_F5,             )       '#F5  -
'  AccelAdd(%FVIRTKEY OR %FCONTROL,            %VK_F5,             )       '^F5  -
'  AccelAdd(%FVIRTKEY OR %FALT,                %VK_F5,             )       '@F5  -

  AccelAdd(%FVIRTKEY,                         %VK_F6, %IDM_VD_Cprop1)         'F6   - Control Properties
'  AccelAdd(%FVIRTKEY OR %FSHIFT,              %VK_F6,           )         '#F6  -
'  AccelAdd(%FVIRTKEY OR %FCONTROL,            %VK_F6,           )         '^F6  -
'  AccelAdd(%FVIRTKEY OR %FALT,                %VK_F6,           )         '@F6  -

 AccelAdd(%FVIRTKEY,                          %VK_F7, %IDM_VD_Wprop)      'F7   - Window Properties
' AccelAdd(%FVIRTKEY OR %FSHIFT,               %VK_F7,          )         '#F7  -
 AccelAdd(%FVIRTKEY OR %FCONTROL,             %VK_F7, %IDM_VD_RM)         '^F7  - Resource Manager
'  AccelAdd(%FVIRTKEY OR %FALT,                %VK_F7,          )          '@F7  -

  AccelAdd(%FVIRTKEY,                         %VK_F8, %IDM_VD_List)           'F8   - Controls List
'  AccelAdd(%FVIRTKEY OR %FSHIFT,              %VK_F8,         )           '#F8  -
'  AccelAdd(%FVIRTKEY OR %FCONTROL,            %VK_F8,         )           '^F8  -
'  AccelAdd(%FVIRTKEY OR %FALT,                %VK_F8,         )           '@F8  -
'  AccelAdd(%FVIRTKEY OR %FSHIFT OR %FALT,     %VK_F8,         )           '#@F8 -

  AccelAdd(%FVIRTKEY,                         %VK_F9, %IDM_VD_COMPILE)        'F9   - compile
'  AccelAdd(%FVIRTKEY OR %FShift,              %VK_F9,            )        '#F9  -
'' AccelAdd(%FVIRTKEY OR %FControl,            %VK_F9,            )        '^F9  -
'  AccelAdd(%FVIRTKEY OR %FALT,                %VK_F9,            )        '@F9  -

' context menu
  AccelAdd(%FVIRTKEY,                         %VK_F10, %IDM_VD_CONTEXT)       'F10  - context menu
'  AccelAdd(%FVIRTKEY OR %FSHIFT,              %VK_F10,            )       '#F10 -
'  AccelAdd(%FVIRTKEY OR %FCONTROL,            %VK_F10,            )       '^F10 -
'  AccelAdd(%FVIRTKEY OR %FALT,                %VK_F10,            )       '@F10 -

  AccelAdd(%FVIRTKEY,                         %VK_F11, %IDM_VD_Resize)        'F11  - Resize Options
'  AccelAdd(%FVIRTKEY OR %FSHIFT,              %VK_F11,           )        '#F11 -
'  AccelAdd(%FVIRTKEY OR %FCONTROL,            %VK_F11,           )        '^F11 -
'  AccelAdd(%FVIRTKEY OR %FALT,                %VK_F11,           )        '@F11 -
'  AccelAdd(%FVIRTKEY OR %FSHIFT or %FALT,     %VK_F11,           )        '#@F11 -

  AccelAdd(%FVIRTKEY,                         %VK_F12, %IDM_VD_Controlmenu)   'F12   - Control menu
'  AccelAdd(%FVIRTKEY OR %FSHIFT,              %VK_F12,                )   '#F12 - goto previous bookmark
'  AccelAdd(%FVIRTKEY OR %FCONTROL,            %VK_F12,                )   '^F12 - delete bookmarks
'  AccelAdd(%FVIRTKEY OR %FALT,                %VK_F12,                )   '@F12 - toggle bookmark



  AccelAdd(%FVIRTKEY,                         %VK_TAB, %IDM_VD_Tab)           'Tab
  AccelAdd(%FVIRTKEY OR %FSHIFT,              %VK_TAB, %IDM_VD_BackTab)       'Back Tab


  AccelAdd(%FVIRTKEY , %VK_Escape,            %IDM_VD_Escape)                '<Esc> key

  AccelAdd(%FVIRTKEY , %VK_Delete,            %IDM_VD_Delete)                '<Del> key


  AccelAdd(%FVIRTKEY,                         %VK_PGUP, %IDM_VD_PgUp)
  AccelAdd(%FVIRTKEY,                         %VK_PGDN, %IDM_VD_PgDn)

  AccelAdd(%FVIRTKEY,                         %VK_Up,   %IDM_VD_Up)
  AccelAdd(%FVIRTKEY,                         %VK_Down, %IDM_VD_Down)
  AccelAdd(%FVIRTKEY,                         %VK_Left, %IDM_VD_Left)
  AccelAdd(%FVIRTKEY,                         %VK_Right,%IDM_VD_Right)


''***********************************************************************************************
' alpha keys + numbers 0-5 + 9 (=user)
''***********************************************************************************************

  AccelAdd(%FVirtkey,              %VK_End,  %IDM_VD_End)
  AccelAdd(%FVirtkey,              %VK_Home, %IDM_VD_Home)
  AccelAdd(%FVirtkey or %FControl, %VK_Tab,  %IDM_VD_CtrlTab)


  AccelAdd(%FVIRTKEY,              %VK_1, %IDM_VD_NOSNAP)                    'no snapping (position)
  AccelAdd(%FVIRTKEY,              %VK_2, %IDM_VD_SNAP2)                     'snap 2
  AccelAdd(%FVIRTKEY,              %VK_3, %IDM_VD_SNAP3)                     'snap 3
  AccelAdd(%FVIRTKEY,              %VK_4, %IDM_VD_SNAP4)                     'snap 4
  AccelAdd(%FVIRTKEY,              %VK_5, %IDM_VD_SNAP5)                     'snap 5
  AccelAdd(%FVIRTKEY,              %VK_0, %IDM_VD_SNAP10)                    'snap 10
  AccelAdd(%FVIRTKEY,              %VK_9, %IDM_VD_SNAPU)                     'snap user

  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_1, %IDM_VD_NOSNAP_s)                  'no snapping (size)
  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_2, %IDM_VD_SNAP2_s)                   'snap 2
  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_3, %IDM_VD_SNAP3_s)                   'snap 3
  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_4, %IDM_VD_SNAP4_s)                   'snap 4
  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_5, %IDM_VD_SNAP5_s)                   'snap 5
  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_0, %IDM_VD_SNAP10_s)                  'snap 10


  AccelAdd(%FVIRTKEY , %VK_A, %IDM_VD_A)
  AccelAdd(%FVIRTKEY , %VK_B, %IDM_VD_B)
  AccelAdd(%FVIRTKEY , %VK_C, %IDM_VD_C)
  AccelAdd(%FVIRTKEY , %VK_D, %IDM_VD_D)
  AccelAdd(%FVIRTKEY , %VK_E, %IDM_VD_E)
  AccelAdd(%FVIRTKEY , %VK_F, %IDM_VD_F)
  AccelAdd(%FVIRTKEY , %VK_G, %IDM_VD_G)
  AccelAdd(%FVIRTKEY , %VK_H, %IDM_VD_H)
  AccelAdd(%FVIRTKEY , %VK_I, %IDM_VD_I)
  AccelAdd(%FVIRTKEY , %VK_J, %IDM_VD_J)
  AccelAdd(%FVIRTKEY , %VK_K, %IDM_VD_K)
  AccelAdd(%FVIRTKEY , %VK_L, %IDM_VD_L)
  AccelAdd(%FVIRTKEY , %VK_M, %IDM_VD_M)
  AccelAdd(%FVIRTKEY , %VK_N, %IDM_VD_N)
  AccelAdd(%FVIRTKEY , %VK_O, %IDM_VD_O)
  AccelAdd(%FVIRTKEY , %VK_P, %IDM_VD_P)
  AccelAdd(%FVIRTKEY , %VK_Q, %IDM_VD_Q)
  AccelAdd(%FVIRTKEY , %VK_R, %IDM_VD_R)
  AccelAdd(%FVIRTKEY , %VK_S, %IDM_VD_S)
  AccelAdd(%FVIRTKEY , %VK_T, %IDM_VD_T)
  AccelAdd(%FVIRTKEY , %VK_U, %IDM_VD_U)
  AccelAdd(%FVIRTKEY , %VK_V, %IDM_VD_V)
  AccelAdd(%FVIRTKEY , %VK_W, %IDM_VD_W)
  AccelAdd(%FVIRTKEY , %VK_X, %IDM_VD_X)
  AccelAdd(%FVIRTKEY , %VK_Y, %IDM_VD_Y)
  AccelAdd(%FVIRTKEY , %VK_Z, %IDM_VD_Z)


''***********************************************************************************************
'' ctrl + alpha keys
''***********************************************************************************************

' AccelAdd( %FVIRTKEY OR %FCONTROL, %VK_A,

  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_B, %IDM_VD_BOTTOM)                    '^B  - set to bottom

'  %VK_C                                                                  'copy
  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_C, %IDM_VD_COPYC1)                 '^C  - Copy

'  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_D,                                 '^D  -
'  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_E,                   )             '^E  -
'  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_F,                                 '^F  -
  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_G, %IDM_VD_GRID)                       '^G  - toggle grid
  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_H, %IDM_VD_HOTKEYS)                    '^H  - show hotkeys
'  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_I,                                 '^I  -
'  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_J,                                 '^J  -
'  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_K,                                 '^K  -
'  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_L, %IDM_VD_Layout)                  '^L  -
'  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_M,                                  '^M  -
''  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_N,                                 '^N  -
  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_O, %IDM_VD_OPTION)                     '^O  - open
'  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_P,                                 '^P  -
'  AccelAdd(%FVIRTKEY OR %FCONTROL, "Q",                                   '^Q  -

'  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_R,                                 '^R  -
  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_S, %IDM_VD_ShowSP)                  '^S  -
  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_T, %IDM_VD_Top)   '°T  - set to top
'  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_U,                                 '^U  -

'  %VK_V                                                                   'paste

  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_V, %IDM_VD_PASTEC1)                 '^V  - Paste
'  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_W,                                 '^W  -

'  %VK_X                                                                   'cut

'  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_Y,                                 '^Y  -
  AccelAdd(%FVIRTKEY OR %FCONTROL, %VK_Z, %IDM_VD_Zoom)                       '^Z  - toggle zoom


'***********************************************************************************************
' alt + alpha keys
'***********************************************************************************************


  AccelAdd(%FVIRTKEY OR %FALT, %VK_X, %IDM_VD_EXIT)                           '@X  - Quit the application


  redim preserve acc(1 to n)
  haccel = CreateAcceleratorTable(acc(1), n)
  function = 1


end function


'***********************************************************************************************


function vd_init alias "vd_init" (byval hinst as dword) export as long
'***********************************************************************************************
' initialize user added controls
'***********************************************************************************************
local hsciLib as dword
local hmlg    as dword


  hSciLib = LoadLibrary(EXE.PATH$ & "visualdesigner\controls\SciLexer.dll")   'Load Scintilla
  IF hSciLib = 0 then
    Msgbox "Couldn´t Load " + $dq + "SciLexer.dll" + $dq, %MB_Applmodal + %MB_Iconexclamation, " Fatal Error"
    Function = 1
    EXIT FUNCTION
  end if


  link_init                                           'load link custom control


#IF %DEF(%MLGSLL)
  MLG_init
#else
  hmlg = LoadLibrary(EXE.PATH$ & "visualdesigner\controls\mlg.dll")   'Load Scintilla
  IF hSciLib = 0 then
    Msgbox "Couldn´t Load " + $dq + "mlg.dll" + $dq, %MB_Applmodal + %MB_Iconexclamation, " Fatal Error"
    Function = 1
    EXIT FUNCTION
  end if
#endif


'***********************************************************************************************
' your code here ...
'***********************************************************************************************
#include "User_VD_Init.bas"


end function


'***********************************************************************************************


function vd_setup alias "vd_setup" (byval hwnd as dword, c$, d$, byval hfont as dword, fc as long, bc as long) export as long
'function vd_setup alias "vd_setup" (byval hwnd as dword, c$, d$, byval hfont as dword, byval fc as long, byval bc as long) export as long
'***********************************************************************************************
' setup user added controls, c$ = window name, d$ = caption text, hfont (font)
'***********************************************************************************************
local tm as textmetricA
local hdc as dword
local y   as long
local ofont as dword                                  'old font
local p   as long                                     'point size
local fname as asciiz * 64                            'font name
local s$


  select case ucase$(c$)
    case "SCINTILLA"                                  'window name
'***********************************************************************************************
' unfortunately scintilla doesn´t support the %WM_Setfont message
' you must set the default font some other way ...
'***********************************************************************************************
      hdc = getdc(0)
        ofont = selectobject(hdc, hfont)              'we got hfont as starting point
        gettextmetricsA hdc, tm
        gettextfaceA hdc, 64, fname                   'read back font name
        selectobject(hdc, ofont)

        y = tm.tmascent                               'character height (we use char height - not cell height)
        p = y * 72 / GetDeviceCaps(hDC, %LOGPIXELSY)  'point size
      releasedc 0, hdc

      setclasslong hwnd, %GCL_HCURSOR, loadcursor(0, byval %IDC_ARROW)


      sendmessage hwnd, %SCI_StyleSetFont, %STYLE_DEFAULT, byval varptr(fname) 'font name
      sendmessage hwnd, %SCI_StyleSetSize, %STYLE_DEFAULT, p              'point size
      sendmessage hwnd, %SCI_StyleSetCharacterSet, %STYLE_DEFAULT, %SC_CHARSET_DEFAULT
      sendmessage hwnd, %SCI_SETEOLMODE, 0, 0         'line end  = $crlf
      sendmessage hwnd, %SCI_CONVERTEOLS, 0, 0        'ensure line endings are all $crlf

      sendmessage hwnd, %SCI_Settext, 0, strptr(d$)


    case "LINK"                                       'window name
' maybe set different colors here...


      if fc => 0 then
        sendmessage hwnd, %LCM_SETCOLOR, 1 , fc       'set control´s fore color
      else
        fc = getwindowlong(hwnd, 4)                   'get fore color for VD
      end if

      if bc => 0 then
        sendmessage hwnd, %LCM_SETCOLOR, 0 , bc       'set control´s back color
      else
        bc = getwindowlong(hwnd, 8)                   'get back color for VD
      end if


    case "MYLITTLEGRID"
      SendMessage hwnd,%MLG_SETCELL, 0, 0             'This line will remove the X on the registered dll and sll

      s$ = parse$(d$, " ", 1)
      SendMessage hwnd, %MLG_SETCELLR ,MAKLNG(1,1),STRPTR(s)

      s$ = parse$(d$, " ", 2)
      SendMessage hwnd, %MLG_SETCELLR ,MAKLNG(1,2),STRPTR(s)


'***********************************************************************************************
' your code here ...
'***********************************************************************************************
#include "User_VD_Setup.bas"


  end select


end function


'***********************************************************************************************
'***********************************************************************************************
'***********************************************************************************************


SUB rearrange(d$)
'***********************************************************************************************
' rearrange all labels starting with "L" by ascending numbers
' work in progress - experimental !!!!
'***********************************************************************************************
'local d$                                   'must end with $crlf
LOCAL t$
LOCAL i AS LONG
LOCAL n AS LONG
LOCAL x AS LONG
LOCAL y AS LONG
LOCAL z AS LONG

LOCAL lv() AS LONG                                    'label value
LOCAL ll() AS LONG                                    'label line
LOCAL ln() AS LONG                                    'label (new)
LOCAL lp() AS LONG                                    'label paragraph
LOCAL s$()



  REPLACE $CRLF WITH " " + $CRLF IN d$                'ensure there is a trailing space before line end
  n = TALLY(d$, ":")
  REDIM lv(1 TO n)
  REDIM ll(1 TO n)


  n = 0 : x = 1 : y = 0
  i = INSTR(d$, $CRLF)
  WHILE i > 0
    y = y + 1
    t$ = MID$(d$, x TO i)
    x = i + 2

    t$ = LTRIM$(t$)
    IF UCASE$(LEFT$(t$, 1)) = "L" THEN
      t$ = EXTRACT$(2, t$, ":")
      IF TALLY(t$, ANY "0123456789") = LEN(t$) THEN
        n = n + 1
        ll(n) = y                                     'line #
        lv(n) = VAL(t$)                               'label value
      END IF
    END IF

    i = INSTR(x, d$, $CRLF)
  WEND


  IF n = 0 THEN EXIT SUB                              'no labels found



  REDIM PRESERVE lv(1 TO n)                           'n = # of labels
  REDIM PRESERVE ll(1 TO n)
  REDIM ln(1 TO n)
  REDIM lp(1 TO n)
  REDIM s$(1 TO y)


  z = 0
  x = INT(lv(1)/10)                                   'current base
  ln(1) = z


  FOR i = 2 TO n
    IF INT(lv(i)/10) > INT(lv(i-1)/10) THEN           'different base
      IF lv(i) - lv(i-1) = 1 THEN                     'but consecutive
        z = z + 1
        ln(i) = z
        x = INT(lv(i)/10)                             'update current base

      ELSE
        x = INT((z + 10)/10)                          'increase base (all following are bigger)
        z = x * 10
        ln(i) = z
        lp(i) = 1                                     'mark as paragraph
      END IF

    ELSE                                              'equal base
      z = z + 1
      ln(i) = z
    END IF
l1:
  NEXT i


  FOR i = 1 TO n
    REPLACE "l"+FORMAT$(lv(i))+" " WITH "lxxxxxxxxxx"+FORMAT$(ln(i))+" " IN d$ 'lower case
    REPLACE "L"+FORMAT$(lv(i))+" " WITH "lxxxxxxxxxx"+FORMAT$(ln(i))+" " IN d$ 'upper case
    REPLACE "l"+FORMAT$(lv(i))+":" WITH "lxxxxxxxxxx"+FORMAT$(ln(i))+":" IN d$ 'lower case
    REPLACE "L"+FORMAT$(lv(i))+":" WITH "lxxxxxxxxxx"+FORMAT$(ln(i))+":" IN d$ 'upper case
  NEXT i


  REPLACE "lxxxxxxxxxx" WITH "l" IN d$


'***********************************************************************************************
' format: one blank line before each label, two blank lines before each paragraph
'***********************************************************************************************


  PARSE d$, s$(), $CRLF

  y = 1
  FOR i = 1 TO n
    IF ll(y) = i THEN                                 'if a label line
      s$(i) = LTRIM$(s$(i))                           'left align labels

      IF lp(y) = 1 THEN                               'paragraph
        z = i - 1
        IF z > 0 THEN
          IF TRIM$(s$(z)) <> "" THEN
            s$(z) = s$(z) + $CRLF + $CRLF

          ELSE
            z = z - 1
            IF z > 0 THEN
              IF TRIM$(s$(z)) <> "" THEN
                s$(z) = s$(z) + $CRLF
              END IF
            END IF
          END IF
        END IF

      ELSE
        z = i - 1
        IF z > 0 THEN
          IF TRIM$(s$(z)) <> "" THEN
            s$(z) = s$(z) + $CRLF
          END IF
        END IF
      END IF

      y = y + 1
    END IF
  NEXT i


  d$ = JOIN$(s$(), $CRLF)


END SUB


#if 0
'***********************************************************************************************
' how to ...
'***********************************************************************************************

' scintilla is the edit control implemented in JK-IDE
' scintilla documentation: http://www.scintilla.org/ScintillaDoc.html

' not everthing you find in scintilla´s documentation can be used in JK-IDE´s edit controls,
' because these controls are a special build for JK-IDE based on Version 3.39 of scintilla
' as a rule of thumb: you can use everything you find in José´s includes V 2.05
' you need "Scintilla. inc" and "SciCtrl.inc" from José Roca´s includes (=> V2.05)

'***********************************************************************************************
' get a handle to the active scintilla control (if not available as a procedure argument)
'***********************************************************************************************
  hsci = GetWindow(SendMessage(@hw.mdi, %WM_MDIGETACTIVE, 0, 0), %GW_Child)


'***********************************************************************************************
' get the file name of the active (code) file
'***********************************************************************************************
local mtext as asciiz * %Max_Path
  GetWindowTextA(SendMessage(@hw.mdi, %WM_MDIGETACTIVE, 0, 0), mtext, %MAX_PATH)


'***********************************************************************************************
' get the file name of a (code) file from it´s scintilla handle
'***********************************************************************************************
local mtext as asciiz * %Max_Path
  GetWindowTextA(GetParent(hsci), mtext, %MAX_PATH)


'***********************************************************************************************
' get all handles to all scintilla controls
'***********************************************************************************************
local d$
local h() as dword

  enumchildwindows(@hw.mdi, byval codeptr(ChildEnumProc), byval varptr(d$))

  if len(d$) then
    redim h(1 to len(d$)/4) at strptr(d$)

  else
'   no open mdi child windows
  end if

  ...

  FUNCTION ChildEnumProc(BYVAL hwnd AS DWORD, BYVAL lParam AS LONG) AS LONG
  '***********************************************************************************************
  ' enumerate mdi childs, scintilla is a child of the mdi child
  '***********************************************************************************************
  local d as string ptr


    d = lparam
    @d = @d + mkdwd(getwindow(hwnd, %GW_Child))

    function = %true


  end function


'***********************************************************************************************
' get all text or a selection
'***********************************************************************************************
  d$ = SCI_GetSelText(hsci)                       'get selected text
  if len(d$) = 0 then
    flag = 1                                      'no selection
    d$ = SCI_GetText(hSci)                        'get all text, if there is no selection
  END IF

'***********************************************************************************************
'  your text processing here ....
'***********************************************************************************************


'***********************************************************************************************
' replace processed text
'***********************************************************************************************
  d$ = RTRIM$(d$, CHR$(0))
  IF flag = 1 THEN                                    'no selection
    SCI_SetText(hsci, d$)                             'set new text (all of it!)

  else
    SCI_ReplaceSel(hsci, d$)                          'replace (selected) text
  END IF


'***********************************************************************************************
' get current line, n = caret x position
'***********************************************************************************************
local ztext as asciiz * 260
  n = sendmessage(hsci, %SCI_GETCURLINE, 256, VARPTR(ztext))


'***********************************************************************************************
' get current word
'***********************************************************************************************
  w$ = SCI_GETWORD(hsci)


'***********************************************************************************************
' send text to the output window
'***********************************************************************************************
  t = "test text"
  t = chr$(1) + hex$(%magenta, 8) + "test text"
  t = chr$(3) + hex$(%magenta, 8) + "appended text"
  t = chr$(2) + hex$(%magenta, 8) + "bold text"

  sendmessage @hw.out, %WM_COMMAND, %WM_Settext, BYVAL VARPTR(t)          't = asciiz
  sendmessage @hw.out, %WM_COMMAND, %WM_Settext, BYVAL STRPTR(t$)         't = string


'***********************************************************************************************
' send unicode text to the output window, the text (t/t$) must be utf8 encoded
' t/t$ = CHRTOUTF8$(wide_string)
'***********************************************************************************************
  sendmessageA @hw.out, %WM_Settext, 2, BYVAL varptr(t)                    't = asciiz
  sendmessageA @hw.out, %WM_Settext, 2, BYVAL STRPTR(t$)                   't = string


' rules:
' "text"                            -> add "text" in new line (may contain additional $CRLF)

' if a line contains (case sensitive)
' "Error "                          -> line is shown in red
' "Successfully created "           -> line is shown in blue

' additionally you may specify a color, a bold font, or append to the previous line
' chr$(1) + hex$(color, 8) + "text" -> line is shown in color
' chr$(2) + hex$(color, 8) + "text" -> line is shown with bold font + color
' chr$(3) + hex$(color, 8) + "text" -> append (color is discarded)

' this prefix applies to the first line only, if text contains more than one line ($CRLF)
' if you want consecutive lines to be colored, you must send it line by line.


'***********************************************************************************************
' close outputwindow (don´t destroy it !!!)
'***********************************************************************************************
  showwindow @hw.out, %SW_Hide


#endif
'***********************************************************************************************
'***********************************************************************************************
'***********************************************************************************************

