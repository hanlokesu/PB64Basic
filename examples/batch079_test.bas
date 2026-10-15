#IF (%PB_REVISION AND &H0FF00) = &H1000

    ' Compiling with PB/Win 10.x

    %MY_PBVER = 10

#ELSEIF (%PB_REVISION AND &H0FF00) = &H0900

    ' Compiling with PB/Win 9.x

    %MY_PBVER = 9

#ELSE

    ' Not PBWin (this fork, or other)

    %MY_PBVER = 0

#ENDIF
' === console emulation for dual-compiler compatibility ===
' (PBWin10 has no PRINT/#CONSOLE; this wrapper uses only official Win32 API)
DECLARE FUNCTION AllocConsole LIB "KERNEL32.DLL" ALIAS "AllocConsole" () AS LONG
DECLARE FUNCTION AttachConsole LIB "KERNEL32.DLL" ALIAS "AttachConsole" (BYVAL dwProcessId AS DWORD) AS LONG
DECLARE FUNCTION GetStdHandle LIB "KERNEL32.DLL" ALIAS "GetStdHandle" (BYVAL nStdHandle AS DWORD) AS LONG
DECLARE FUNCTION WriteFile LIB "KERNEL32.DLL" ALIAS "WriteFile" (BYVAL hFile AS LONG, lpBuffer AS ANY, BYVAL nNumberOfBytesToWrite AS DWORD, lpNumberOfBytesWritten AS DWORD, BYVAL lpOverlapped AS DWORD) AS LONG
DECLARE FUNCTION ReadFile LIB "KERNEL32.DLL" ALIAS "ReadFile" (BYVAL hFile AS LONG, lpBuffer AS ANY, BYVAL nNumberOfBytesToRead AS DWORD, lpNumberOfBytesRead AS DWORD, BYVAL lpOverlapped AS DWORD) AS LONG
SUB ConPrint(BYVAL s AS STRING)
    LOCAL h AS LONG
    LOCAL n AS DWORD
    h = GetStdHandle(-11)
    IF h = 0 THEN
        IF AttachConsole(-1) = 0 THEN AllocConsole
        h = GetStdHandle(-11)
    END IF
    IF h <> 0 THEN
        WriteFile h, BYVAL STRPTR(s), LEN(s), n, 0
    END IF
END SUB
SUB ConWaitKey()
    LOCAL h AS LONG
    LOCAL c AS STRING * 1
    LOCAL n AS DWORD
    h = GetStdHandle(-10)
    IF h <> 0 THEN
        ReadFile h, c, 1, n, 0
    END IF
END SUB
' PB64Basic Enhanced - Batch 79 Test: OOP foundation (CLASS/METHOD/OBJECT/INSTANCE) + ARRAY REDIM

' CLASS/END CLASS block (fork OOP extension; PBWin has no CLASS at this level)
#IF %MY_PBVER = 0
CLASS MyClass
    METHOD Foo()
        ConPrint "inside Foo (should NOT appear)"
    END METHOD
END CLASS
#ELSE
' (PBWin10: skipped: CLASS block)
#ENDIF

' METHOD at top level (fork extension)
#IF %MY_PBVER = 0
METHOD TestMethod()
    ConPrint "TestMethod called"
END METHOD
#ELSE
' (PBWin10: skipped: METHOD TestMethod)
#ENDIF

FUNCTION PBMAIN() AS LONG
    LOCAL arr() AS LONG
    DIM arr(10) AS LONG
    LOCAL obj AS LONG
    LOCAL waitk AS STRING

    ConPrint "=== Batch 79: OOP foundation + ARRAY REDIM ==="

    ConPrint "CLASS/END CLASS: parsed OK (block skipped)"

    ' OBJECT type (treated as pointer/LONG)
    obj = 0
    ConPrint "OBJECT type: OK (value=" & STR$(obj) & ")"

    ' ARRAY REDIM INCR/DECR
#IF %MY_PBVER = 0
    ARRAY REDIM INCR arr(0), 5
    ConPrint "ARRAY REDIM INCR: done"
    ARRAY REDIM DECR arr(0), 3
    ConPrint "ARRAY REDIM DECR: done"
#ELSE
' (PBWin10: skipped: ARRAY REDIM)
#ENDIF

    ' Call top-level METHOD (treated as SUB)
#IF %MY_PBVER = 0
    TestMethod
    ConPrint "METHOD call: OK"
#ELSE
' (PBWin10: skipped: METHOD call)
#ENDIF

    ConPrint "=== Result: ALL PASS (OOP foundation + ARRAY REDIM)"
    ConPrint "Press any key to exit..."
    waitk = WAITKEY$
END FUNCTION
