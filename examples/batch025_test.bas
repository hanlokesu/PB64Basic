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

FUNCTION PBMAIN() AS LONG
    LOCAL x AS LONG
    LOCAL errcode AS LONG
    LOCAL waitk AS STRING

    ' --- REGISTER is accepted as an optimization hint (LOCAL semantics) ---
    REGISTER counter AS LONG
    REGISTER regname AS STRING
    counter = 5
    regname = "reg"
    ConPrint "REGISTER counter=" & STR$(counter) & " regname=" & STR$(regname)

    ' --- Test 1: ON ERROR GOTO traps a run-time error, RESUME NEXT skips it ---
    ON ERROR GOTO handler1
    MKDIR "Z:\no_such_dir_xyz\sub"
    ConPrint "t1: resumed after MKDIR (OK)"
    GOTO after1
handler1:
    errcode = ERR
    ConPrint "handler1 caught ERR=" & STR$(errcode)
    RESUME NEXT
after1:
    ConPrint "t1 done, errcode=" & STR$(errcode)

    ' --- Test 2: RESUME re-executes the failing statement after fixing it ---
    MKDIR "batch25_dir"
    ERRCLEAR
    ON ERROR GOTO handler2
    MKDIR "batch25_dir"
    ConPrint "t2: retried MKDIR succeeded"
    GOTO skip2
handler2:
    errcode = ERR
    ConPrint "handler2 caught ERR=" & STR$(errcode)
    RMDIR "batch25_dir"
    RESUME
skip2:
    ConPrint "t2 done (mkdir retried OK)"

    ' --- Test 3: RESUME label jumps to a specific label ---
    ON ERROR GOTO handler3
    MKDIR "Z:\no_such_dir_xyz\sub"
    ConPrint "t3: should NOT print"
    GOTO after3
handler3:
    errcode = ERR
    ConPrint "handler3 caught ERR=" & STR$(errcode)
    RESUME cont3
cont3:
    ConPrint "t3 resumed at label"
after3:

    ' --- Test 4: ON ERROR GOTO 0 disables trapping ---
    ON ERROR GOTO 0
    MKDIR "Z:\no_such_dir_xyz\sub"
    ConPrint "t4 not trapped (ERR=" & STR$(ERR) & ")"

    ' --- Test 5: RESUME FLUSH continues after the RESUME ---
    ON ERROR GOTO handler5
    MKDIR "Z:\no_such_dir_xyz\sub"
    ConPrint "t5: should NOT print"
    GOTO after5
handler5:
    errcode = ERR
    ConPrint "handler5 caught ERR=" & STR$(errcode)
    RESUME FLUSH
    ConPrint "t5 after RESUME FLUSH (same line continues)"
after5:

    ' --- Test 6: no error -> handler never fires ---
    ON ERROR GOTO handler6
    MKDIR "batch25_dir2"
    ConPrint "t6 no error trapped (OK)"
    GOTO after6
handler6:
    ConPrint "t6: should NOT print"
    RESUME NEXT
after6:
    RMDIR "batch25_dir2"
    ERRCLEAR

    ConPrint "Press any key to exit..."
    waitk = WAITKEY$
END FUNCTION


#ENDIF
