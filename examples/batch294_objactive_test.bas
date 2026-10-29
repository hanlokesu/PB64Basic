' batch294_objactive_test.bas - OBJACTIVE(progid$) + OBJEQUAL(ObjectVar1,
' ObjectVar2) (CHW OBJACTIVE_function / OBJEQUAL_function).  OBJACTIVE:
' TRUE (-1) if an EXE-based COM object for the ProgID is currently running,
' FALSE (0) otherwise (DLL/OCX servers always report FALSE).  OBJEQUAL:
' TRUE (-1) if both object variables refer to the same object or both refer to
' NOTHING; FALSE (0) otherwise.
#COMPILE EXE
#DIM ALL
FUNCTION PBMAIN() AS LONG
    LOCAL o1 AS IUNKNOWN, o2 AS IUNKNOWN, o3 AS IUNKNOWN
    LOCAL a AS LONG, b AS LONG
    OPEN "batch294_out.txt" FOR OUTPUT AS #1
    o1 = NEWCOM "Scripting.FileSystemObject"
    o2 = o1
    a = OBJEQUAL(o1, o2)
    PRINT #1, "eq_same="; a
    b = OBJEQUAL(o1, o3)
    PRINT #1, "eq_nothing="; b
    PRINT #1, "active_dll="; OBJACTIVE("Scripting.FileSystemObject")
    PRINT #1, "active_bad="; OBJACTIVE("No.Such.ProgID.999")
    CLOSE #1
    FUNCTION = 0
END FUNCTION
