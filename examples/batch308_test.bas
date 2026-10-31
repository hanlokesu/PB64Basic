#COMPILE EXE
#DIM ALL
DECLARE FUNCTION GetStdHandle LIB "KERNEL32.DLL" ALIAS "GetStdHandle" (BYVAL nStdHandle AS DWORD) AS LONG
DECLARE FUNCTION WriteFile LIB "KERNEL32.DLL" ALIAS "WriteFile" (BYVAL hFile AS LONG, lpBuffer AS ANY, BYVAL nBytesToWrite AS DWORD, lpBytesWritten AS DWORD, BYVAL lpOverlapped AS LONG) AS LONG

SUB ConPrint(BYVAL s AS STRING)
    LOCAL h, n AS LONG
    h = GetStdHandle(-11)
    WriteFile h, BYVAL STRPTR(s), LEN(s), n, 0
END SUB

GLOBAL gv&
GLOBAL gs AS STRING

FUNCTION DoGosub() AS LONG
    LOCAL i AS LONG
    i = 0
    GOSUB lbl1
    FUNCTION = i
lbl1:
    i = i + 7
    RETURN
END FUNCTION

FUNCTION DoGoto() AS LONG
    LOCAL i AS LONG
    i = 1
    GOTO lbl2
    i = 99
lbl2:
    FUNCTION = i
END FUNCTION

FUNCTION PBMAIN() AS LONG
    LOCAL i, n AS LONG, s AS STRING
    LOCAL x&, y&, i&, siz1&, ip&
    OPEN "b308_g2.dat" FOR OUTPUT AS #3
    PRINT #3, "x"
    CLOSE #3
    ConPrint "G1:attr=" & STR$(GETATTR("b308_g2.dat")) & "|"
    KILL "b308_g2.dat"
    GLOBALMEM ALLOC 64 TO i&
    GLOBALMEM SIZE i& TO siz1&
    GLOBALMEM FREE i& TO i&
    ConPrint "G2:mem=" & STR$(siz1&) & "|"
    y& = DoGosub()
    ConPrint "G3:gosub=" & STR$(y&) & "|"
    ConPrint "G4:goto=" & STR$(DoGoto()) & "|"
    gv& = 123
    gs = "abc"
    ConPrint "G5:gv=" & STR$(gv&) & "|gs=" & gs & "|"
    ConPrint "H1:hex=" & HEX$(255) & "|"
    ConPrint "H2:hex4=" & HEX$(255, 4) & "|"
    ConPrint "H3:hexn=" & HEX$(-1) & "|"
    ConPrint "H4:hi=" & STR$(HI(WORD, &H12345678)) & "|"
    ConPrint "H5:hib=" & STR$(HI(BYTE, &H12345678)) & "|"
    ConPrint "H6:hiwrd=" & STR$(HIWRD(&H12345678)) & "|"
    HOST ADDR "localhost" TO ip&
    ConPrint "H7:ip=" & STR$(ip&) & "|"
    ConPrint "I1:iif=" & STR$(IIF(1=1, 10, 20)) & "|"
    ConPrint "I2:iifs=" & IIF$(2>1, "a", "b") & "|"
    ConPrint "I3:imp00=" & STR$(0 IMP 0) & "|"
    ConPrint "I4:imp10=" & STR$(1 IMP 0) & "|"
    ConPrint "I5:imp11=" & STR$(1 IMP 1) & "|"
    i = 5
    INCR i
    ConPrint "I6:incr=" & STR$(i) & "|"
    OPEN "b308_i_tmp.txt" FOR OUTPUT AS #1
    PRINT #1, "42 hello"
    CLOSE #1
    OPEN "b308_i_tmp.txt" FOR INPUT AS #1
    INPUT #1, n, s
    CLOSE #1
    KILL "b308_i_tmp.txt"
    ConPrint "I7:in=" & STR$(n) & "," & s & "|"
    ConPrint "I8:instr=" & STR$(INSTR(2, "hello", "l")) & "|"
    ConPrint "I9:instr0=" & STR$(INSTR("hello", "z")) & "|"
    ConPrint "I10:int=" & STR$(INT(3.7)) & "|"
    ConPrint "I11:isfalse=" & STR$(ISFALSE 0) & "|"
    ConPrint "I12:istrue=" & STR$(ISTRUE 1) & "|"
    ConPrint "I13:isfalse5=" & STR$(ISFALSE 5) & "|"
    ConPrint "I14:isfile=" & STR$(ISFILE("b308_i_tmp2.dat")) & "|"
    OPEN "b308_i_tmp2.dat" FOR OUTPUT AS #2
    CLOSE #2
    ConPrint "I15:isfile2=" & STR$(ISFILE("b308_i_tmp2.dat")) & "|"
    KILL "b308_i_tmp2.dat"
    ConPrint "I16:isfolder=" & STR$(ISFOLDER("C:\")) & "|"
    ConPrint "I17:isnull=" & STR$(ISNULL(s)) & "|"
    s = ""
    ConPrint "I18:isnull2=" & STR$(ISNULL(s)) & "|"
    ConPrint "I19:isnotnull=" & STR$(ISNOTNULL("abc")) & "|"
    ConPrint "I20:iswin0=" & STR$(ISWIN(0)) & "|"
    n = 0
    FOR i = 1 TO 5
        IF i = 3 THEN ITERATE FOR
        n = n + i
    NEXT i
    ConPrint "I21:iter=" & STR$(n) & "|"
    ConPrint "OK=1"
END FUNCTION
