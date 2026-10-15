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

' PB64Basic Enhanced - batch118_test.bas
' Batch 118: wire up previously half-finished statements.
'   GRAPHIC PRINT ........ real GDI text on the attached bitmap (pb_graphic_print_str)
'   XPRINT COLOR ......... 1/2/3-arg printer text color (pb_xprint_set_color / _rgb)
'   RESOURCE SAVE FILE ... real embedded-resource extraction (pb_resource_save_file)
'   DISPLAY OPENFILE/SAVEFILE/BROWSE/COLOR/FONT ... real Win32 common dialogs
'   PLAY SOUND freq,ms ... Beep() tone (pb_play_sound)
'   FONT END ............. parser fix (Token::End in peek_plain_upper)
'
' Dialog / printer / embedded-resource statements need an interactive desktop,
' a printer, or a compiled-in resource, so they are guarded by showDialogs=0
' (they still compile to real Win32 calls; flip the flag to try them).
FUNCTION PBMAIN() AS LONG
    LOCAL hbmp AS QUAD
    LOCAL waitk AS STRING
    LOCAL fails AS LONG
    LOCAL showDialogs AS LONG
    LOCAL f AS STRING
    LOCAL c AS LONG

    showDialogs = 0

    ConPrint "=== Batch 118: half-finished wiring completion ==="

    ' --- GRAPHIC PRINT: draw text onto an offscreen bitmap, save it ----------
    GRAPHIC BITMAP NEW 240, 80 TO hbmp
    IF hbmp = 0 THEN
        ConPrint "FAIL: GRAPHIC BITMAP NEW returned zero"
        INCR fails
    ELSE
        GRAPHIC ATTACH hbmp
        GRAPHIC CLEAR
        GRAPHIC COLOR 0, 16777215          ' black text on white background
        GRAPHIC PRINT "Hello PB64Basic"
        KILL "batch118_out.bmp"
        GRAPHIC SAVE "batch118_out.bmp"
        GRAPHIC DETACH
        GRAPHIC BITMAP END
        IF ISFILE("batch118_out.bmp") THEN
            ConPrint "OK: GRAPHIC PRINT rendered and saved batch118_out.bmp"
        ELSE
            ConPrint "FAIL: GRAPHIC SAVE did not produce batch118_out.bmp"
            INCR fails
        END IF
    END IF

    ' --- PLAY SOUND: a short 440 Hz beep (pb_play_sound -> Beep) -------------
    PLAY SOUND 440, 120
    ConPrint "OK: PLAY SOUND 440,120 issued"

    ' --- Interactive common dialogs / printer / resource (compile-verified) -
    IF showDialogs THEN
        DISPLAY OPENFILE "Open a file", "Text files", "" TO f
        DISPLAY SAVEFILE "Save a file", "Text files", "" TO f
        DISPLAY BROWSE "Pick a folder", "" TO f
        DISPLAY COLOR TO c
        DISPLAY FONT TO f
        XPRINT ATTACH "default"
        XPRINT COLOR 0, 0, 255
        XPRINT PRINT "blue text"
        XPRINT CLOSE
        RESOURCE SAVE FILE "MYRES", "res_out.bin"
    END IF

    IF fails = 0 THEN
        ConPrint "=== Batch 118: ALL PASS ==="
    ELSE
        ConPrint "=== Batch 118: FAILURES=" & STR$(fails) & " ==="
    END IF

    ConPrint "Press any key to exit..."
    waitk = WAITKEY$
END FUNCTION


#ENDIF
