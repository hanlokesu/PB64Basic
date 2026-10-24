'===============================================================
' batch276_test.bas - #COMPILE DLL output chain (batch 276)
' Official MyDLL.bas shape (C:\PBWin10\samples\DLL\MyDLL.bas)
' minus the Win32api.inc include (full Win32api.inc parsing -
' UNION/ENUM/TYPE structures - is a separate large feature).
'
' Equates use the official bare `%name = value` form.  The
' `%CONSTANT %name = value` keyword form is a fork extension and
' is NOT accepted by PBWin (Error 415); the fork accepts both.
'
' Build as a DLL:
'   PB64Basic build batch276_test.bas -o batch276_test.obj --dll
'       --target x86_64-pc-windows-msvc --runtime-lib pb_runtime_x64.obj
' Dual-verified against PBWin10: LIBMAIN executes on load
' (PROCESS_ATTACH), exported function doubles its argument.
'===============================================================
#COMPILER PBWIN 10
#COMPILE DLL

%DLL_PROCESS_ATTACH = 1
%DLL_PROCESS_DETACH = 0
%DLL_THREAD_ATTACH = 2
%DLL_THREAD_DETACH = 3

GLOBAL ghInstance AS DWORD

'--- Main DLL entry point called by Windows (official LIBMAIN shape) ---
FUNCTION LIBMAIN (BYVAL hInstance   AS LONG, _
                  BYVAL fwdReason   AS LONG, _
                  BYVAL lpvReserved AS LONG) AS LONG

    SELECT CASE fwdReason

    CASE %DLL_PROCESS_ATTACH
        ghInstance = hInstance
        FUNCTION = 1   'success!

    CASE %DLL_PROCESS_DETACH
        FUNCTION = 1   'success!

    CASE %DLL_THREAD_ATTACH
        FUNCTION = 1   'success!

    CASE %DLL_THREAD_DETACH
        FUNCTION = 1   'success!

    END SELECT

END FUNCTION

'--- Exported function: doubles the argument ---
FUNCTION MyFunction1 ALIAS "MyFunction1" (BYVAL Param1 AS LONG) EXPORT AS LONG
    FUNCTION = Param1 * 2
END FUNCTION

'--- Exported sub (official MySub1 shape) ---
SUB MySub1 ALIAS "MySub1" (BYVAL Param1 AS LONG) EXPORT
END SUB
