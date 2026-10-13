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

#COMPILE EXE
' === console emulation for dual-compiler compatibility ===
' (PBWin10 has no PRINT/#CONSOLE; this wrapper uses only official Win32 API)
DECLARE FUNCTION AllocConsole LIB "KERNEL32.DLL" ALIAS "AllocConsole" () AS LONG
DECLARE FUNCTION AttachConsole LIB "KERNEL32.DLL" ALIAS "AttachConsole" (BYVAL dwProcessId AS DWORD) AS LONG
DECLARE FUNCTION GetStdHandle LIB "KERNEL32.DLL" ALIAS "GetStdHandle" (BYVAL nStdHandle AS DWORD) AS LONG
DECLARE FUNCTION WriteFile LIB "KERNEL32.DLL" ALIAS "WriteFile" (BYVAL hFile AS LONG, lpBuffer AS ANY, BYVAL nBytesToWrite AS DWORD, lpBytesWritten AS DWORD, BYVAL lpOverlapped AS LONG) AS LONG
DECLARE FUNCTION ReadFile LIB "KERNEL32.DLL" ALIAS "ReadFile" (BYVAL hFile AS LONG, lpBuffer AS ANY, BYVAL nBytesToRead AS DWORD, lpBytesRead AS DWORD, BYVAL lpOverlapped AS LONG) AS LONG
' console-title API: PBWin10-only DECLARE (fork's WINDOW SET/GET TEXT)
#IF %MY_PBVER <> 0
DECLARE FUNCTION SetConsoleTitleA LIB "KERNEL32.DLL" ALIAS "SetConsoleTitleA" (lpConsoleTitle AS ASCIIZ) AS LONG
DECLARE FUNCTION GetConsoleTitleA LIB "KERNEL32.DLL" ALIAS "GetConsoleTitleA" (lpConsoleTitle AS ASCIIZ, BYVAL nSize AS DWORD) AS DWORD
#ENDIF
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


TYPE MyType
    n AS LONG
    d AS DOUBLE
    s AS STRING * 12
END TYPE

FUNCTION IncStatic() AS LONG
    STATIC counter AS LONG
    counter = counter + 1
    FUNCTION = counter
END FUNCTION

FUNCTION PBMAIN() AS LONG
    LOCAL i AS LONG
    LOCAL a() AS LONG
    DIM a(4) AS LONG
    LOCAL b() AS LONG
    DIM b(4) AS LONG
    LOCAL j AS LONG
    LOCAL t1 AS MyType
    LOCAL t2 AS MyType
    LOCAL title AS STRING

    ConPrint "=== Batch 23: STATIC / ARRAY ASSIGN / WINDOW / TYPE SET ==="+$CRLF

    ' 1. STATIC persists across calls
    FOR i = 1 TO 3
        ConPrint "IncStatic = " & STR$(IncStatic())+$CRLF
    NEXT i

    ' 2. ARRAY ASSIGN b() = a()
    a(0) = 10 : a(1) = 20 : a(2) = 30 : a(3) = 40 : a(4) = 50
#IF %MY_PBVER = 0
    ARRAY ASSIGN b() = a()
#ELSE
    ' (PBWin10 has no ARRAY ASSIGN from an array; same result via loop)
    FOR j = 0 TO 4 : b(j) = a(j) : NEXT j
#ENDIF
    ConPrint "b(0) = " & STR$(b(0)) & " b(4) = " & STR$(b(4))+$CRLF

    ' 3. WINDOW SET TEXT / WINDOW GET TEXT (console title)
#IF %MY_PBVER = 0
    WINDOW SET TEXT 0, "PowerBasilisk Batch23"
#ELSE
    ' (PBWin10 WINDOW SET TEXT targets a DIALOG, not the console;
    '  same effect on the attached console via SetConsoleTitleA)
    SetConsoleTitleA "PowerBasilisk Batch23"
#ENDIF
#IF %MY_PBVER = 0
    WINDOW GET TEXT 0 TO title
#ELSE
    title = SPACE$(128)
    GetConsoleTitleA BYVAL STRPTR(title), 128
    title = LEFT$(title, INSTR(title, CHR$(0)) - 1)
#ENDIF
    ConPrint "Title = " & title+$CRLF

    ' 4. TYPE SET from a TYPE variable
    t1.n = 123
    t1.d = 4.5
    t1.s = "hello"
#IF %MY_PBVER = 0
    TYPE SET t2 = t1
#ELSE
    ' (PBWin10 supports UDT-to-UDT via plain assignment)
    t2 = t1
#ENDIF
    ConPrint "t2.n = " & STR$(t2.n) & " t2.d = " & STR$(t2.d) & " t2.s = " & TRIM$(t2.s)+$CRLF

    ' 5. TYPE SET from a STRING (fills the UDT bytes)
#IF %MY_PBVER = 0
    TYPE SET t2 = "TYPE SET FROM STRING"
#ELSE
    ' (PBWin10 has no TYPE SET from string; the first 4 UDT bytes are
    '  the little-endian bytes of "TYPE", reproduced here as the same LONG)
    t2.n = ASC("T") + (ASC("Y") * 256) + (ASC("P") * 65536) + (ASC("E") * 16777216)
#ENDIF
    ConPrint "t2.n(1st4 bytes) = " & STR$(t2.n)+$CRLF

    ConPrint "Press any key to exit..."+$CRLF
    ConWaitKey
END FUNCTION
