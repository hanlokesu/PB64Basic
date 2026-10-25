' batch280_test.bas - WSTRING true UTF-16 type (CHW WSTRING_Data_Type).
' Fork now models WSTRING as a real UTF-16 BSTR slot and marshals it to COM
' as-is (type code 12); previously WSTRING was a STRING alias and "..."$$
' literals were silently lexed as plain strings and re-encoded at the COM
' boundary.  Behavior on both sides must stay identical.
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
    ConPrint "batch280 fail: no FSO object"
    ok = 0
  END IF

  IF ok THEN
    wpath = "C:\Windows\win.ini"$$
    OBJECT CALL o.FileExists(wpath) TO exists
    IF exists <> -1 THEN
      ConPrint "batch280 fail: exists=" & STR$(exists)
      ok = 0
    END IF
  END IF

  IF ok THEN
    wpath = "C:\no_such_file_xyz_12345.txt"$$
    OBJECT CALL o.FileExists(wpath) TO exists
    IF exists <> 0 THEN
      ConPrint "batch280 fail: missing=" & STR$(exists)
      ok = 0
    END IF
  END IF

  IF ok THEN
    ConPrint "batch280 test ok"
  END IF
  FUNCTION = 0
END FUNCTION
