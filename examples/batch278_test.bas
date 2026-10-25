' batch278_test.bas - NEWCOM method call via OBJECT CALL (CHW OBJECT_statement /
' How_do_you_call_a_DISPATCH_METHOD_): IDispatch::GetIDsOfNames + Invoke with
' VARIANT marshalling.  FSO.FileExists(path) returns VARIANT_BOOL -> LONG -1/0.
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
  LOCAL exists AS LONG
  LOCAL ok AS LONG
  LOCAL wpath AS WSTRING

  ok = 1
  LET o = NEWCOM "Scripting.FileSystemObject"
  IF ISOBJECT(o) = 0 THEN
    ConPrint "batch278 fail: no FSO object"
    ok = 0
  END IF

  IF ok THEN
    wpath = "C:\Windows\win.ini"$$
    OBJECT CALL o.FileExists(wpath) TO exists
    IF exists <> -1 THEN
      ConPrint "batch278 fail: exists=" & STR$(exists)
      ok = 0
    END IF
  END IF

  IF ok THEN
    wpath = "C:\no_such_file_xyz_12345.txt"$$
    OBJECT CALL o.FileExists(wpath) TO exists
    IF exists <> 0 THEN
      ConPrint "batch278 fail: missing=" & STR$(exists)
      ok = 0
    END IF
  END IF

  IF ok THEN
    ConPrint "batch278 test ok"
  END IF
  FUNCTION = 0
END FUNCTION
