' batch256B_test.bas - CHW B-family probe: BIT statement/function,
' BIT CALC, BITS, BITS$, BITSE (batch 256B).  Expected values = PBWin10
' measured behavior (probe _b256b_probe.bas, dual-run MATCH 24/24).
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
    LOCAL ok AS LONG
    LOCAL i AS LONG
    LOCAL x AS LONG
    LOCAL dwStatus AS DWORD
    ok = 0

    ' ===== 1. BIT statement (CHW BIT_statement.htm) =====
    x = 3
    BIT SET x, 2                      ' bit2 already 0 -> 7
    IF x = 7 THEN ok = ok + 1
    BIT RESET x, 1                    ' 7 & ~2 = 5
    IF x = 5 THEN ok = ok + 2
    BIT TOGGLE x, 4                   ' 5 xor 16 = 21
    IF x = 21 THEN ok = ok + 4
    BIT TOGGLE x, 4                   ' back to 5
    IF x = 5 THEN ok = ok + 8

    ' implied bit-array across array elements (CHW: BIT SET A?(1), 128)
    DIM z(1 TO 3) AS LONG
    BIT SET z(1), 37                  ' bit 5 of the 2nd element
    IF BIT(z(1), 37) = 1 AND BIT(z(1), 5) = 0 THEN ok = ok + 16
    IF z(2) = 32 THEN ok = ok + 32

    ' ===== 2. BIT function (CHW BIT_function.htm) =====
    x = 7
    IF BIT(x, 2) = 1 AND BIT(x, 0) = 1 AND BIT(x, 3) = 0 THEN ok = ok + 64

    ' ===== 3. BIT CALC (CHW BIT_CALC_statement.htm): calcexpr AND 1 = set/reset =====
    dwStatus = 0
    FOR i = 0 TO 7
        BIT CALC dwStatus, i, i MOD 3   ' bit0 of i MOD 3: 0,1,0,0,1,0,0,1
    NEXT i
    ' set bits 1,4,7 -> dwStatus = 2+16+128 = 146 (PBWin measured)
    IF dwStatus = 146 THEN ok = ok + 128

    ' ===== 4. BITS function (CHW BITS_function.htm): bit-pattern reinterpret =====
    IF BITS(BYTE, 300) = 44 THEN ok = ok + 256
    IF BITS(BYTE, -1) = 255 THEN ok = ok + 512
    IF BITS(WORD, -1) = 65535 THEN ok = ok + 1024
    IF BITS(DWORD, -1) = 4294967295 THEN ok = ok + 2048
    IF BITS(INTEGER, 65535) = -1 THEN ok = ok + 4096
    IF BITS(INTEGER, 32768) = -32768 THEN ok = ok + 8192
    IF BITS(LONG, 4294967295) = -1 THEN ok = ok + 16384
    IF BITS(LONG, 2147483648) = -2147483648 THEN ok = ok + 32768

    ' ===== 5. BITS$ function (CHW BITS$_function.htm): copy without conversion =====
    LOCAL s AS STRING
    LOCAL t AS STRING
    s = "alpha"
    t = BITS$(STRING, s)
    IF t = "alpha" AND LEN(t) = 5 THEN ok = ok + 65536

    ' ===== 6. BITSE function (CHW BITSE_function.htm): compare bit patterns =====
    IF BITSE(-1, 65535, 16) = -1 THEN ok = ok + 131072   ' equal 16-bit pattern -> TRUE
    IF BITSE(1, 2, 16) = 0 THEN ok = ok + 262144         ' differ -> FALSE
    IF BITSE(-1, 65535, 32) = 0 THEN ok = ok + 524288    ' 32-bit patterns differ
    IF BITSE(65535, 65535, 32) = -1 THEN ok = ok + 1048576

    ConPrint "batch256B ok=" & STR$(ok)
    IF ok = 2097151 THEN
        ConPrint "batch256B test ok"
    ELSE
        ConPrint "batch256B test FAILED"
    END IF
END FUNCTION
