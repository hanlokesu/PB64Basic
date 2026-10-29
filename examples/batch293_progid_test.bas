' batch293_progid_test.bas - PROGID$(ClassID$) is the complement of CLSID$
' (CHW PROGID$ function): s = CLSID$("Scripting.FileSystemObject") returns the
' 16-byte binary class ID; PROGID$(s) formats it and looks up
' HKCR\CLSID\{...}\ProgID, returning the alphanumeric ProgID string.  Both
' compilers read the same registry values, so the output must be identical.
#COMPILE EXE
#DIM ALL
FUNCTION PBMAIN() AS LONG
    LOCAL s AS STRING, p AS STRING
    OPEN "batch293_out.txt" FOR OUTPUT AS #1
    s = CLSID$( "Scripting.FileSystemObject" )
    PRINT #1, "clen="; LEN(s)
    PRINT #1, "b1="; ASC(s, 1); "b2="; ASC(s, 2); "b3="; ASC(s, 3); "b4="; ASC(s, 4)
    p = PROGID$(s)
    PRINT #1, "plen="; LEN(p)
    IF p = "" THEN
        PRINT #1, "batch293 progid empty"
    ELSE
        PRINT #1, "batch293 " + p
    END IF
    PRINT #1, "neg_len="; LEN(PROGID$("1234567890abcdef"))   ' 16 bytes, no registry entry
    CLOSE #1
    FUNCTION = 0
END FUNCTION
