#COMPILE EXE

' batch329 self-check: hash (#) metastatement family CHW conformance.
'   1) Built-in compiler equates are defined with their real identity
'      (PBxB64-style): PB_WIN32 / PB_DLL32 / PB_CC32 = 0, PB_REVLETTER = 32,
'      PB_EXE = -1 for an EXE build, PB_REVISION = 631 for this release.
'   2) #IF compile-time arithmetic with banker's rounding (half to even).
'      "/" and "\" are identical: both divide then round to the nearest even
'      integer. CVQ assembles bytes little-endian. A wrong or missing
'      implementation takes the #ELSE branch and increments the counter.
'   3) #RESOURCE STRING embeds an RT_STRING block, read back at run time with
'      RESOURCE$(STRING, id).
' Headless only: every assertion is counted, output goes through ConPrint,
' no WAITKEY / MSGBOX / dialog.

#RESOURCE STRING, 1, "Batch329 string resource"

DECLARE FUNCTION GetStdHandle LIB "KERNEL32.DLL" ALIAS "GetStdHandle" (BYVAL nStdHandle AS DWORD) AS LONG
DECLARE FUNCTION AttachConsole LIB "KERNEL32.DLL" ALIAS "AttachConsole" (BYVAL dwProcessId AS DWORD) AS LONG
DECLARE FUNCTION AllocConsole LIB "KERNEL32.DLL" ALIAS "AllocConsole" () AS LONG
DECLARE FUNCTION WriteFile LIB "KERNEL32.DLL" ALIAS "WriteFile" (BYVAL hFile AS LONG, lpBuffer AS ANY, BYVAL nBytesToWrite AS DWORD, lpBytesWritten AS DWORD, BYVAL lpOverlapped AS LONG) AS LONG

SUB ConPrint(BYVAL s AS STRING)
    LOCAL h AS LONG
    LOCAL n AS DWORD
    h = GetStdHandle(-11)
    IF h = 0 THEN
        IF AttachConsole(-1) = 0 THEN AllocConsole
        h = GetStdHandle(-11)
    END IF
    IF h <> 0 THEN WriteFile h, BYVAL STRPTR(s), LEN(s), n, 0
END SUB

GLOBAL g_failures AS LONG

SUB CheckLong(BYVAL got AS LONG, BYVAL expected AS LONG, BYVAL tag AS STRING)
    IF got <> expected THEN
        g_failures = g_failures + 1
        ConPrint "FAIL " + tag + " got=" + FORMAT$(got) + " expected=" + FORMAT$(expected) + $CRLF
    END IF
END SUB

SUB CheckStr(BYVAL got AS STRING, BYVAL expected AS STRING, BYVAL tag AS STRING)
    IF got <> expected THEN
        g_failures = g_failures + 1
        ConPrint "FAIL " + tag + " got=[" + got + "] expected=[" + expected + "]" + $CRLF
    END IF
END SUB

SUB ArithFail(BYVAL tag AS STRING)
    g_failures = g_failures + 1
    ConPrint "FAIL arith " + tag + $CRLF
END SUB

FUNCTION PBMAIN() AS LONG
    LOCAL v AS LONG
    LOCAL s AS STRING

    ' --- 1) built-in compiler equates: real identity, all defined ---
    v = %PB_WIN32
    CheckLong v, 0, "PB_WIN32"
    v = %PB_DLL32
    CheckLong v, 0, "PB_DLL32"
    v = %PB_CC32
    CheckLong v, 0, "PB_CC32"
    v = %PB_REVLETTER
    CheckLong v, 32, "PB_REVLETTER"
    v = %PB_EXE
    CheckLong v, -1, "PB_EXE"
    v = %PB_REVISION
    CheckLong v, 631, "PB_REVISION"

    ' --- 2) #IF compile-time arithmetic (banker's rounding, half to even) ---
    #IF 7/2 = 4
    #ELSE
        ArithFail "7/2"
    #ENDIF
    #IF 5/2 = 2
    #ELSE
        ArithFail "5/2"
    #ENDIF
    #IF 9/2 = 4
    #ELSE
        ArithFail "9/2"
    #ENDIF
    #IF 20/4 = 5
    #ELSE
        ArithFail "20/4"
    #ENDIF
    #IF 1/3 = 0
    #ELSE
        ArithFail "1/3"
    #ENDIF
    #IF 17\3 = 6
    #ELSE
        ArithFail "17\3"
    #ENDIF
    #IF 16\3 = 5
    #ELSE
        ArithFail "16\3"
    #ENDIF
    #IF 14\3 = 5
    #ELSE
        ArithFail "14\3"
    #ENDIF
    #IF 17\2 = 8
    #ELSE
        ArithFail "17\2"
    #ENDIF
    #IF 13\2 = 6
    #ELSE
        ArithFail "13\2"
    #ENDIF
    #IF (7/2)*2 = 8
    #ELSE
        ArithFail "(7/2)*2"
    #ENDIF
    #IF 2+3 = 5
    #ELSE
        ArithFail "2+3"
    #ENDIF
    #IF 10-4 = 6
    #ELSE
        ArithFail "10-4"
    #ENDIF
    #IF 3*4 = 12
    #ELSE
        ArithFail "3*4"
    #ENDIF
    #IF CVQ("A") = 65
    #ELSE
        ArithFail "CVQ(A)"
    #ENDIF
    #IF CVQ("ABCD") = 1145258561
    #ELSE
        ArithFail "CVQ(ABCD)"
    #ENDIF

    ' --- 3) RT_STRING resource embedded and read back ---
    s = RESOURCE$(STRING, 1)
    CheckStr s, "Batch329 string resource", "STRING"

    ConPrint "=== FAILURES: " + FORMAT$(g_failures) + $CRLF
    FUNCTION = g_failures
END FUNCTION
