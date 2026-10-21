' batch256B_probe.bas - print version to observe PBWin/fork real values
DECLARE FUNCTION AllocConsole LIB "KERNEL32.DLL" ALIAS "AllocConsole" () AS LONG
DECLARE FUNCTION AttachConsole LIB "KERNEL32.DLL" ALIAS "AttachConsole" (BYVAL dwProcessId AS DWORD) AS LONG
DECLARE FUNCTION GetStdHandle LIB "KERNEL32.DLL" ALIAS "GetStdHandle" (BYVAL nStdHandle AS DWORD) AS LONG
DECLARE FUNCTION WriteFile LIB "KERNEL32.DLL" ALIAS "WriteFile" (BYVAL hFile AS LONG, lpBuffer AS ANY, BYVAL nBytesToWrite AS DWORD, lpBytesWritten AS DWORD, BYVAL lpOverlapped AS LONG) AS LONG
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
FUNCTION PBMAIN() AS LONG
    LOCAL i AS LONG
    LOCAL x AS LONG
    LOCAL dwStatus AS DWORD

    x = 7
    BIT SET x, 2
    ConPrint "bit_set_x=" & STR$(x)
    BIT RESET x, 1
    ConPrint "bit_reset_x=" & STR$(x)
    BIT TOGGLE x, 4
    ConPrint "bit_toggle_x=" & STR$(x)
    BIT TOGGLE x, 4
    ConPrint "bit_toggle2_x=" & STR$(x)

    DIM z(1 TO 3) AS LONG
    BIT SET z(1), 37
    ConPrint "bit_arr37=" & STR$(BIT(z(1), 37)) & " b5=" & STR$(BIT(z(1), 5)) & " b32=" & STR$(BIT(z(1), 32)) & " z1=" & STR$(z(1)) & " z2=" & STR$(z(2))

    x = 7
    ConPrint "bit_fn_2=" & STR$(BIT(x, 2)) & " b0=" & STR$(BIT(x, 0)) & " b3=" & STR$(BIT(x, 3))

    dwStatus = 0
    FOR i = 0 TO 7
        BIT CALC dwStatus, i, i MOD 3
    NEXT i
    ConPrint "bitcalc_dw=" & STR$(dwStatus)

    ConPrint "bits_byte300=" & STR$(BITS(BYTE, 300))
    ConPrint "bits_byte_1=" & STR$(BITS(BYTE, -1))
    ConPrint "bits_word_1=" & STR$(BITS(WORD, -1))
    ConPrint "bits_dword_1=" & STR$(BITS(DWORD, -1))
    ConPrint "bits_int65535=" & STR$(BITS(INTEGER, 65535))
    ConPrint "bits_int32768=" & STR$(BITS(INTEGER, 32768))
    ConPrint "bits_long4294967295=" & STR$(BITS(LONG, 4294967295))
    ConPrint "bits_long2147483648=" & STR$(BITS(LONG, 2147483648))

    LOCAL s AS STRING
    LOCAL t AS STRING
    s = "alpha"
    t = BITS$(STRING, s)
    ConPrint "bits_str=[" & t & "] len=" & STR$(LEN(t))

    ConPrint "bitse_16=" & STR$(BITSE(-1, 65535, 16))
    ConPrint "bitse_16b=" & STR$(BITSE(1, 2, 16))
    ConPrint "bitse_32=" & STR$(BITSE(-1, 65535, 32))
    ConPrint "bitse_32b=" & STR$(BITSE(65535, 65535, 32))
END FUNCTION
