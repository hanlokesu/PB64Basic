#COMPILE EXE
#DIM ALL

FUNCTION PBMAIN() AS LONG
    LOCAL s AS STRING
    LOCAL o AS IUNKNOWN
    LOCAL p AS DWORD
    LOCAL isok AS LONG, r AS LONG
    LOCAL hp AS LONG
    s = CLSID$( "Scripting.FileSystemObject" )
    OPEN "b289c_out.txt" FOR OUTPUT AS #1
    PRINT #1, "clsid_len="; LEN(s)
    PRINT #1, "b1="; ASC(s, 1); "b2="; ASC(s, 2)
    s = CLSID$( "No.Such.ProgID.999" )
    PRINT #1, "bad_len="; LEN(s)
    LET o = NEWCOM "Scripting.FileSystemObject"
    isok = ISOBJECT(o)
    PRINT #1, "isobj="; isok
    p = OBJPTR(o)
    IF p <> 0 THEN hp = 1 ELSE hp = 0
    PRINT #1, "hasptr="; hp
    r = OBJRESULT
    PRINT #1, "objres="; r
    CLOSE #1
END FUNCTION
