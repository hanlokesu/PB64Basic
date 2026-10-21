' batch256C_test.bas - CHW B-family probe: BEEP, BGR, BIN$/OCT$/DEC$
' 4-arg formatting, BUILD$ (batch 256B part 2).  Expected values = PBWin10
' measured behavior (_b256b2_probe.bas dual-run MATCH 21/21).
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
    LOCAL s AS STRING
    LOCAL a AS STRING
    LOCAL b AS STRING
    ok = 0

    ' ===== 1. BEEP (CHW BEEP_statement.htm): default waveform sound, no crash =====
    BEEP
    ok = ok + 1

    ' ===== 2. BGR function (CHW BGR_function.htm) =====
    IF BGR(255, 0, 0) = 16711680 THEN ok = ok + 2       ' &HFF0000 (byte3 red)
    IF BGR(0, 255, 0) = 65280 THEN ok = ok + 4          ' &H00FF00
    IF BGR(0, 0, 255) = 255 THEN ok = ok + 8            ' &H0000FF (byte1 blue)
    IF BGR(&HFF0000) = 255 THEN ok = ok + 16            ' single-arg swap: &H0000FF
    IF BGR(&H0000FF) = 16711680 THEN ok = ok + 32       ' swap back: &HFF0000

    ' ===== 3. BIN$ (CHW BIN$_function.htm): 4-arg formatting =====
    s = BIN$(5)
    IF s = "101" THEN ok = ok + 64
    s = BIN$(0)
    IF s = "0" THEN ok = ok + 128
    s = BIN$(5, 8)
    IF s = "00000101" THEN ok = ok + 256
    s = BIN$(5, -6)
    IF s = "   101" THEN ok = ok + 512
    s = BIN$(5, 0, 2, 1)
    IF s = "  101 " THEN ok = ok + 1024
    s = BIN$(-1)
    IF s = "1111111111111111111111111111111111111111111111111111111111111111" THEN ok = ok + 2048
    s = BIN$(255)
    IF s = "11111111" THEN ok = ok + 4096

    ' ===== 4. OCT$ / DEC$ (same CHW syntax as BIN$) =====
    s = OCT$(10)
    IF s = "12" THEN ok = ok + 8192
    s = OCT$(10, 5)
    IF s = "00012" THEN ok = ok + 16384
    s = DEC$(12)
    IF s = "12" THEN ok = ok + 32768
    s = DEC$(12, 5)
    IF s = "00012" THEN ok = ok + 65536

    ' ===== 5. BUILD$ (CHW BUILD$_function.htm): efficient concatenation =====
    a = "ab"
    b = "cd"
    s = BUILD$(a, b, "ef")
    IF s = "abcdef" THEN ok = ok + 131072
    s = BUILD$(a, "bb", b)
    IF s = "abbbcd" THEN ok = ok + 262144
    s = BUILD$(a, "x", b)
    IF s = "abxcd" THEN ok = ok + 524288
    s = BUILD$("x")
    IF s = "x" THEN ok = ok + 1048576

    ConPrint "batch256C ok=" & STR$(ok)
    IF ok = 2097151 THEN
        ConPrint "batch256C test ok"
    ELSE
        ConPrint "batch256C test FAILED"
    END IF
END FUNCTION
