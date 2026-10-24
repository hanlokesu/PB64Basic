'==============================================================================
'
'  Address.bas for PowerBASIC for Windows
'  Copyright (c) 1999-2011 PowerBASIC, Inc.
'  All Rights Reserved.
'
'  A simple application to test MyDLL.DLL
'
'==============================================================================
'
'  This file is the official TestDLL.bas shape
'  (C:\PBWin10\samples\DLL\TestDLL.bas), the host application for the
'  MyDLL.bas in this directory.  Build batch276_MyDLL.bas first with PB64Basic
'  (it emits batch276_MyDLL.lib), then build this EXE - the DECLARE ... LIB
'  "batch276_MyDLL.DLL" resolves to batch276_MyDLL.lib next to the source.
'
'  Build as an EXE:
'    PB64Basic build batch276_testDLL.bas --exe --target x86_64-pc-windows-msvc
'        --runtime-lib ..\pb_runtime_x64.obj   (run from examples; .. = repo root)
'
'==============================================================================

#COMPILER PBWIN 10
#COMPILE EXE

'--------------------------------------------------------------------
DECLARE FUNCTION MyFunction1 LIB "batch276_MyDLL.DLL" _
          ALIAS "MyFunction1" (BYVAL Param1 AS LONG) AS LONG
'--------------------------------------------------------------------

FUNCTION PBMAIN () AS LONG
    LOCAL lRes AS LONG

    lRes = MyFunction1(lRes)  ' call MyFunction1 in My.DLL

    MSGBOX "Exe result from MYDLL.DLL:" + STR$(lRes)

END FUNCTION
