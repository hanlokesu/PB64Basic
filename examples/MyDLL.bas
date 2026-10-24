'===============================================================================
'
'  Generic DLL Template for PowerBASIC for Windows
'  Copyright (c) 1997-2011 PowerBASIC, Inc.
'  All Rights Reserved.
'
'  LIBMAIN function Purpose:
'
'    User-defined function called by Windows each time a DLL is loaded into,
'    and unloaded from, memory. In 32-bit Windows, LibMain is called each
'    time a DLL is loaded by an application or process.  Your code should
'    never call LibMain explicitly.
'
'    hInstance is the DLL instance handle.  This handle is used by the
'    calling application to identify the DLL being called.  To access
'    resources in the DLL, this handle will need to be stored in a global
'    variable.  Use the GetModuleHandle(BYVAL 0&) to get the instance
'    handle of the calling EXE.
'
'    fdwReason specifies a flag indicating why the DLL entry-point
'    (LibMain) is being called by Windows.
'
'    lpvReserved specifies further aspects of the DLL initialization
'    and cleanup.  If fdwReason is %DLL_PROCESS_ATTACH, lpvReserved is
'    NULL (zero) for dynamic loads and non-NULL for static loads.  If
'    fdwReason is %DLL_PROCESS_DETACH, lpvReserved is NULL if LibMain
'    has been called by using the FreeLibrary API call and non-NULL if
'    LibMain has been called during process termination.
'
'  Return
'
'    If LibMain is called with %DLL_PROCESS_ATTACH, your LibMain function
'    should return a zero (0) if any part of your initialization process
'    fails or a one (1) if no errors were encountered.  If a zero is
'    returned, Windows will abort and unload the DLL from memory. When
'    LibMain is called with any other value than %DLL_PROCESS_ATTACH, the
'    return value is ignored.
'
'===============================================================================
'
'  This file is the official MyDLL.bas shape (C:\PBWin10\samples\DLL\MyDLL.bas)
'  minus the "#INCLUDE ONCE "Win32api.inc"" line: full Win32api.inc parsing
'  (nested TYPE/UNION, $ string equates, keywords-as-field-names) is a
'  separate large feature.  The equates used here are declared with the
'  official bare "%name = value" form.
'
'  Build as a DLL (the source's "#COMPILE DLL" puts PB64Basic into DLL mode):
'    PB64Basic build MyDLL.bas --target x86_64-pc-windows-msvc
'        --runtime-lib pb_runtime_x64.obj
'  This emits MyDLL.dll and MyDLL.lib next to the source.
'
'===============================================================================

#COMPILER PBWIN 10
#COMPILE DLL

%DLL_PROCESS_ATTACH = 1
%DLL_PROCESS_DETACH = 0
%DLL_THREAD_ATTACH = 2
%DLL_THREAD_DETACH = 3

GLOBAL ghInstance AS DWORD


'-------------------------------------------------------------------------------
' Main DLL entry point called by Windows...
'
FUNCTION LIBMAIN (BYVAL hInstance   AS LONG, _
                  BYVAL fwdReason   AS LONG, _
                  BYVAL lpvReserved AS LONG) AS LONG

    SELECT CASE fwdReason

    CASE %DLL_PROCESS_ATTACH
        'Indicates that the DLL is being loaded by another process (a DLL
        'or EXE is loading the DLL).  DLLs can use this opportunity to
        'initialize any instance or global data, such as arrays.

        ghInstance = hInstance

        FUNCTION = 1   'success!

        'FUNCTION = 0   'failure!  This will prevent the EXE from running.

    CASE %DLL_PROCESS_DETACH
        'Indicates that the DLL is being unloaded or detached from the
        'calling application.  DLLs can take this opportunity to clean
        'up all resources for all threads attached and known to the DLL.

        FUNCTION = 1   'success!

        'FUNCTION = 0   'failure!

    CASE %DLL_THREAD_ATTACH
        'Indicates that the DLL is being loaded by a new thread in the
        'calling application.  DLLs can use this opportunity to
        'initialize any thread local storage (TLS).

        FUNCTION = 1   'success!

        'FUNCTION = 0   'failure!

    CASE %DLL_THREAD_DETACH
        'Indicates that the thread is exiting cleanly.  If the DLL has
        'allocated any thread local storage, it should be released.

        FUNCTION = 1   'success!

        'FUNCTION = 0   'failure!

    END SELECT

END FUNCTION


'-------------------------------------------------------------------------------
' Examples of exported Subs and functions...
'
FUNCTION MyFunction1 ALIAS "MyFunction1" (BYVAL Param1 AS LONG) EXPORT AS LONG

    ' code goes here
    MSGBOX "MYDLL.DLL has recevied: " + STR$(Param1)
    FUNCTION = 1  ' return 1 to calling program

END FUNCTION


SUB MySub1 ALIAS "MySub1" (BYVAL Param1 AS LONG) EXPORT

    ' code goes here

END SUB
