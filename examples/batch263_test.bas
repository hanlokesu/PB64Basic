' batch263_test.bas - METHOD = expr return-value statement (CHW METHOD_END_METHOD)
' A CLASS with two METHODS; Get() sets its return value with the official
' "METHOD = expression" form.  Fork form: object variable typed as the CLASS
' name (fork superset; PBWin needs AS <interface> + COM instantiation, batch263
' skill note).  Headless: writes a file, no console blocking statements.
#COMPILE EXE

CLASS C263
    INSTANCE inst AS STRING
    METHOD SetValue(s AS STRING)
        inst = s
    END METHOD
    METHOD GetValue() AS STRING
        METHOD = inst
    END METHOD
END CLASS

FUNCTION PBMAIN() AS LONG
    LOCAL o AS C263
    LOCAL s AS STRING
    o.SetValue("method ok")
    s = o.GetValue()
    OPEN "batch263_out.txt" FOR OUTPUT AS #1
    PRINT #1, "batch263:" + s
    CLOSE #1
    FUNCTION = 0
END FUNCTION
