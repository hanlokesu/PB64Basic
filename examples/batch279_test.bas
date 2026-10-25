' batch279_test.bas - OBJECT compound member chain (CHW OBJECT_statement):
' o.Drives.Count walks Drives (property GET -> IDispatch) then Count (LONG).
' ConPrint template from batch256A_test.bas (PBWin #COMPILE EXE has no console).
#COMPILE EXE
#DIM ALL
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
  LOCAL o AS IDISPATCH
  LOCAL cnt AS LONG
  LOCAL ok AS LONG

  ok = 1
  LET o = NEWCOM "Scripting.FileSystemObject"
  IF ISOBJECT(o) = 0 THEN
    ConPrint "batch279 fail: no FSO object"
    ok = 0
  END IF

  IF ok THEN
    OBJECT GET o.Drives.Count TO cnt
    IF cnt <> 14 THEN
      ConPrint "batch279 fail: drives=" & STR$(cnt)
      ok = 0
    END IF
  END IF

  IF ok THEN
    ConPrint "batch279 test ok"
  END IF
  FUNCTION = 0
END FUNCTION
