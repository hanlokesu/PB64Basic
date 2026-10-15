#COMPILE EXE
#DIM ALL
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

' PB64Basic Enhanced - batch 29: Inline ASM (! and ASM statements)
' 32-bit and 64-bit targets. Each ! line becomes an LLVM inline-asm block.


FUNCTION PBMAIN() AS LONG
    LOCAL x AS LONG
    LOCAL y AS LONG
    LOCAL w AS WORD
    LOCAL b AS BYTE
    LOCAL q AS QUAD
    LOCAL pass AS LONG
    LOCAL fail AS LONG
    LOCAL waitk AS STRING

    pass = 0
    fail = 0

    ' 1. write a LONG via ASM shortcut
    x = 5
    ! MOV x, 123
    IF x = 123 THEN
        pass = pass + 1
    ELSE
        fail = fail + 1
        ConPrint "FAIL 1: x=" & STR$(x)
    END IF

    ' 2. copy variable to variable
    ! MOV y, x
    IF y = 123 THEN
        pass = pass + 1
    ELSE
        fail = fail + 1
        ConPrint "FAIL 2: y=" & STR$(y)
    END IF

    ' 3. add immediate
    ! ADD x, 10
    IF x = 133 THEN
        pass = pass + 1
    ELSE
        fail = fail + 1
        ConPrint "FAIL 3: x=" & STR$(x)
    END IF

    ' 4. ASM keyword form with % suffix
    ASM MOV x%, 7
    IF x = 7 THEN
        pass = pass + 1
    ELSE
        fail = fail + 1
        ConPrint "FAIL 4: x=" & STR$(x)
    END IF

    ' 5. BYTE width
    ! MOV b, 100
    IF b = 100 THEN
        pass = pass + 1
    ELSE
        fail = fail + 1
        ConPrint "FAIL 5: b=" & STR$(b)
    END IF

    ' 6. WORD width
    ! MOV w, 30000
    IF w = 30000 THEN
        pass = pass + 1
    ELSE
        fail = fail + 1
        ConPrint "FAIL 6: w=" & STR$(w)
    END IF

    ' 7. QUAD width (qword ptr)
    ! MOV q, 123456789012345
    IF q = 123456789012345 THEN
        pass = pass + 1
    ELSE
        fail = fail + 1
        ConPrint "FAIL 7: q=" & STR$(q)
    END IF

    ' 8. register-only instructions (no PB operands)
    ! MOV EAX, 1
    ! ADD EAX, 2
    ! NOP
    pass = pass + 1

    ' 9. register state preserved across consecutive ASM lines
    ! MOV EAX, [x]
    ! MOV y, EAX
    IF y = 7 THEN
        pass = pass + 1
    ELSE
        fail = fail + 1
        ConPrint "FAIL 9: y=" & STR$(y)
    END IF

    ' 10. x87 floating-point: FLD1 then FSTP to a DOUBLE variable
    LOCAL d AS DOUBLE
    ! FLD1
    ! FSTP d
    IF d > 0.9 AND d < 1.1 THEN
        pass = pass + 1
    ELSE
        fail = fail + 1
        ConPrint "FAIL 10: d=" & STR$(d)
    END IF

    ' 11. MMX + SSE instructions accepted (compile-time proof)
    ! PXOR MM0, MM0
    ! EMMS
    ! XORPS XMM0, XMM0
    pass = pass + 1

    ConPrint "batch29 ASM: pass=" & STR$(pass) & " fail=" & STR$(fail)
    IF fail = 0 THEN
        ConPrint "ALL PASS"
    ELSE
        ConPrint "SOME FAILED"
    END IF

    ConPrint "Press any key to exit..."
    ConWaitKey
END FUNCTION



#ENDIF
