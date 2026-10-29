#COMPILE EXE
#IF %DEF(%PB_REVISION)
    #IF (%PB_REVISION AND &H0FF00) = &H1000
        %MY_PBVER = 10
    #ELSE
        %MY_PBVER = 0
    #ENDIF
#ELSE
    %MY_PBVER = 0
#ENDIF
#IF %MY_PBVER = 10
FUNCTION PBMAIN() AS LONG
    ' PBWin10 stub: compiles, does nothing (VARIANT$/$ verified on the fork).
END FUNCTION
#ELSE
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

'=====================================================================
' PBXB64 Example - batch 300 witness: VARIANT$ / VARIANT$$ (CHW
' VARIANT_function1).  VARIANT$(v) extracts the ANSI form of the dynamic
' string held in a Variant (VT_BSTR); VARIANT$$(v) returns the WSTRING
' (UTF-16 BSTR) form.  Non-string variants yield "" / empty.
' Expected output (run mode):
'   str=hello
'   wlen=5
'   empty_str=0
'   vt=8
'   === FAILURES:0 ===
'=====================================================================

FUNCTION PBMAIN() AS LONG
    LOCAL v AS VARIANT
    LOCAL s AS STRING
    LOCAL w AS WSTRING
    LOCAL fails AS LONG

    v = "hello"
    s = VARIANT$(v)
    ConPrint "str=" & s
    w = VARIANT$$(v)
    ConPrint "wlen=" & STR$(LEN(w))
    IF s <> "hello" THEN fails = fails + 1
    IF LEN(w) <> 5 THEN fails = fails + 1

    v = 999&
    s = VARIANT$(v)
    IF LEN(s) = 0 THEN
        ConPrint "empty_str=0"
    ELSE
        ConPrint "empty_str=1"
        fails = fails + 1
    END IF
    ConPrint "vt=" & STR$(VARIANTVT(v))

    IF fails = 0 THEN
        ConPrint "=== FAILURES:0 ==="
    ELSE
        ConPrint "=== FAILURES:" & STR$(fails) & " ==="
    END IF
    FUNCTION = 0
WAITKEY$
END FUNCTION
#ENDIF
