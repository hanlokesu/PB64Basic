#COMPILE EXE
' E-project probe v2: ENUM forms per CHW (default start 0, BITS 0/1/2/4/8,
' SINGULAR %member ref, explicit-value chain), EXTRACT$ forms, EQV, ERASE.
DECLARE FUNCTION AllocConsole LIB "KERNEL32.DLL" ALIAS "AllocConsole" () AS LONG
DECLARE FUNCTION GetStdHandle LIB "KERNEL32.DLL" ALIAS "GetStdHandle" (BYVAL nStdHandle AS DWORD) AS LONG
DECLARE FUNCTION WriteFile LIB "KERNEL32.DLL" ALIAS "WriteFile" (BYVAL hFile AS LONG, lpBuffer AS ANY, BYVAL nBytesToWrite AS DWORD, lpBytesWritten AS DWORD, BYVAL lpOverlapped AS LONG) AS LONG

SUB ConPrint(BYVAL s AS STRING)
    LOCAL h AS LONG
    LOCAL n AS DWORD
    h = GetStdHandle(-11)
    IF h = 0 THEN
        AllocConsole
        h = GetStdHandle(-11)
    END IF
    IF h <> 0 THEN
        WriteFile h, BYVAL STRPTR(s), LEN(s), n, 0
    END IF
END SUB

ENUM MyEnum
    ME_ONE
    ME_TWO
    ME_FOUR = 4
    ME_FIVE
END ENUM

ENUM MyBits BITS
    MB_B0
    MB_B1
    MB_B2
    MB_B3
    MB_B4
END ENUM

ENUM MySing SINGULAR
    MS_ALPHA
    MS_BETA
END ENUM

ENUM MyBitsX BITS
    MX_D0 = 8
    MX_D1
END ENUM

FUNCTION PBMAIN() AS LONG
    LOCAL s AS STRING
    LOCAL v AS LONG
    LOCAL a1() AS LONG
    LOCAL a2() AS STRING
    LOCAL ok AS LONG
    s = "abc|def"
    ok = 1
    REDIM a1(5)
    REDIM a2(5)
    ConPrint "E1=" & TRIM$(STR$(%MyEnum.ME_ONE)) & $CRLF     ' 0
    ConPrint "E2=" & TRIM$(STR$(%MyEnum.ME_TWO)) & $CRLF     ' 1
    ConPrint "E4=" & TRIM$(STR$(%MyEnum.ME_FOUR)) & $CRLF    ' 4
    ConPrint "E5=" & TRIM$(STR$(%MyEnum.ME_FIVE)) & $CRLF    ' 5
    ConPrint "B0=" & TRIM$(STR$(%MyBits.MB_B0)) & $CRLF      ' 0
    ConPrint "B1=" & TRIM$(STR$(%MyBits.MB_B1)) & $CRLF      ' 1
    ConPrint "B2=" & TRIM$(STR$(%MyBits.MB_B2)) & $CRLF      ' 2
    ConPrint "B3=" & TRIM$(STR$(%MyBits.MB_B3)) & $CRLF      ' 4
    ConPrint "B4=" & TRIM$(STR$(%MyBits.MB_B4)) & $CRLF      ' 8
    ConPrint "S0=" & TRIM$(STR$(%MS_ALPHA)) & $CRLF          ' 0
    ConPrint "S1=" & TRIM$(STR$(%MS_BETA)) & $CRLF           ' 1
    ConPrint "X0=" & TRIM$(STR$(%MyBitsX.MX_D0)) & $CRLF     ' 8
    ConPrint "X1=" & TRIM$(STR$(%MyBitsX.MX_D1)) & $CRLF     ' 16
    ConPrint "XS=" & EXTRACT$(s, "|") & $CRLF                ' abc
    ConPrint "X3=" & EXTRACT$("abcdef", "zz") & $CRLF        ' abcdef
    v = 5
    v = v EQV 3
    ConPrint "Q=" & TRIM$(STR$(v)) & $CRLF                   ' 5 EQV 3 = NOT(5 XOR 3) = NOT 6 = -7
    a1(0) = 7
    a2(0) = "s"
    ERASE a1, a2
    ConPrint "ER=OK" & $CRLF
    IF %MyEnum.ME_ONE <> 0 THEN ok = 0
    IF %MyEnum.ME_FIVE <> 5 THEN ok = 0
    IF %MyBits.MB_B4 <> 8 THEN ok = 0
    IF %MS_BETA <> 1 THEN ok = 0
    IF %MyBitsX.MX_D1 <> 16 THEN ok = 0
    IF EXTRACT$(s, "|") <> "abc" THEN ok = 0
    IF EXTRACT$("abcdef", "zz") <> "abcdef" THEN ok = 0
    ConPrint "OK=" & TRIM$(STR$(ok)) & $CRLF
END FUNCTION
