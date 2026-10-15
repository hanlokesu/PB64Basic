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

' stmt_batch1.bas  test CLS / ERROR / ENVIRON / FILECOPY / SETATTR
' Compile: pbcompiler build stmt_batch1.bas --exe --target x86_64-pc-windows-msvc --runtime-lib pb_runtime_x64.obj
FUNCTION PBMAIN() AS LONG
    LOCAL waitk AS STRING

    ' 1. ERROR n  set PB error code, readable via ERR
    ERRCLEAR
    ERROR 75
    IF ERR = 75 THEN
        ConPrint "ERROR: OK"
    ELSE
        ConPrint "ERROR: FAIL got " & STR$(ERR)
    END IF

    ' 2. ENVIRON "VAR=value"  set env var, read back via ENVIRON$
    ENVIRON "PB_TEST_VAR=hello123"
    IF ENVIRON$("PB_TEST_VAR") = "hello123" THEN
        ConPrint "ENVIRON: OK"
    ELSE
        ConPrint "ENVIRON: FAIL got " & ENVIRON$("PB_TEST_VAR")
    END IF

    ' 3. FILECOPY src$, dst$  copy file, ERR = 0 on success
    OPEN "fc_src.txt" FOR OUTPUT AS #1
    PRINT #1, "filecopy test data"
    CLOSE #1
    ERRCLEAR
    FILECOPY "fc_src.txt", "fc_dst.txt"
    IF ERR = 0 AND ISFILE("fc_dst.txt") THEN
        ConPrint "FILECOPY: OK"
    ELSE
        ConPrint "FILECOPY: FAIL err=" & STR$(ERR)
    END IF

    ' 4. SETATTR "path", attr&  set hidden attribute (2), ERR = 0 on success
    ERRCLEAR
    SETATTR "fc_dst.txt", 2
    IF ERR = 0 THEN
        ConPrint "SETATTR: OK"
    ELSE
        ConPrint "SETATTR: FAIL err=" & STR$(ERR)
    END IF

    ' 5. CLS  clear console (no assertion; must compile & run without crash)
    CLS
    ConPrint "CLS: OK"

    KILL "fc_src.txt"
    KILL "fc_dst.txt"

    ConPrint "ALL DONE"
    ConPrint "Press any key to exit..."
    waitk = WAITKEY$
END FUNCTION


#ENDIF
