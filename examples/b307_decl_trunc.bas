#COMPILE EXE
#UNIQUE VAR ON
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
    WriteFile h, BYVAL STRPTR(s), LEN(s), n, 0
END SUB

%WIN32_WINNT = &H0601
#IF %WIN32_WINNT >= &H0501
DECLARE SUB DnsTruncTest LIB "DnsApi.dll" ( _
    pRecordList AS DWORD, _
    FreeType AS DWORD _
#ENDIF ' %WIN32_WINNT >= &H0501

FUNCTION PBMAIN() AS LONG
    ConPrint "DECL-TRUNC PASS"
END FUNCTION
