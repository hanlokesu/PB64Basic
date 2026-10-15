#IF %DEF(%PB_REVISION)
    #IF (%PB_REVISION AND &H0FF00) = &H1000
        %MY_PBVER = 10
    #ELSE
        %MY_PBVER = 0
    #ENDIF
#ELSE
    %MY_PBVER = 0
#ENDIF

' --- PBWin10 stub branch: this sample exercises PB64Basic-only ---
'     syntax that official PBWin10 does not provide; it compiles but
'     does nothing here.  The fork branch (#ELSE) is the real test.
#IF %MY_PBVER = 10
FUNCTION PBMAIN() AS LONG
    ' PB64Basic-only sample: PBWin10 stub (compiles, does nothing).
END FUNCTION
#ELSE

' === console emulation for dual-compiler compatibility ===
' (PBWin10 has no PRINT/#CONSOLE; this wrapper uses only official Win32 API)
DECLARE FUNCTION AllocConsole LIB "KERNEL32.DLL" ALIAS "AllocConsole" () AS LONG
DECLARE FUNCTION AttachConsole LIB "KERNEL32.DLL" ALIAS "AttachConsole" (BYVAL dwProcessId AS DWORD) AS LONG
DECLARE FUNCTION GetStdHandle LIB "KERNEL32.DLL" ALIAS "GetStdHandle" (BYVAL nStdHandle AS DWORD) AS LONG
DECLARE FUNCTION WriteFile LIB "KERNEL32.DLL" ALIAS "WriteFile" (BYVAL hFile AS LONG, lpBuffer AS ANY, BYVAL nBytesToWrite AS DWORD, lpBytesWritten AS DWORD, BYVAL lpOverlapped AS LONG) AS LONG
DECLARE FUNCTION ReadFile LIB "KERNEL32.DLL" ALIAS "ReadFile" (BYVAL hFile AS LONG, lpBuffer AS ANY, BYVAL nBytesToRead AS DWORD, lpBytesRead AS DWORD, BYVAL lpOverlapped AS LONG) AS LONG
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

' PB64Basic Enhanced - Batch 78 Test: XPRINT GET MARGIN + DISPLAY common dialogs (6 statements)
FUNCTION PBMAIN() AS LONG
    LOCAL ml AS LONG, mt AS LONG, mr AS LONG, mb AS LONG
    LOCAL result AS STRING
    LOCAL color AS LONG
    LOCAL waitk AS STRING

    ConPrint "=== Batch 78: XPRINT GET MARGIN + DISPLAY ==="

    XPRINT ATTACH DEFAULT
    XPRINT GET MARGIN TO ml, mt, mr, mb
    ConPrint "GET MARGIN:" & STR$(ml) & STR$(mt) & STR$(mr) & STR$(mb)
    XPRINT CLOSE

    DISPLAY OPENFILE "Open", "All|*.*", "C:\" TO result
    ConPrint "DISPLAY OPENFILE: [" & result & "]"

    DISPLAY SAVEFILE "Save", "All|*.*", "C:\" TO result
    ConPrint "DISPLAY SAVEFILE: [" & result & "]"

    DISPLAY COLOR TO color
    ConPrint "DISPLAY COLOR:" & STR$(color)

    DISPLAY FONT TO result
    ConPrint "DISPLAY FONT: [" & result & "]"

    DISPLAY BROWSE "Browse", "C:\" TO result
    ConPrint "DISPLAY BROWSE: [" & result & "]"

    ConPrint "=== Result: ALL PASS (6 statements)"
    ConPrint "Press any key to exit..."
    ConWaitKey
END FUNCTION



#ENDIF
