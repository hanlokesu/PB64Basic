' batch378_newend - CON.NEW / CON.END lifecycle probe.
' Must be launched with CREATE_NO_WINDOW (no inherited console) so CON.NEW
' can allocate a fresh console; results go to batch378_newend.txt because the
' process has no stdout before CON.NEW.
#COMPILE EXE
#CONSOLE ON
#DIM ALL

FUNCTION PBMAIN() AS LONG
    LOCAL h0 AS QUAD
    LOCAL h1 AS QUAD
    LOCAL h2 AS QUAD
    LOCAL e1 AS LONG
    LOCAL e2 AS LONG

    OPEN "batch378_newend.txt" FOR OUTPUT AS #1
    h0 = CON.HANDLE
    PRINT #1, "start handle="; h0; " err="; ERR

    ' Start from a known no-console state: this also proves CON.END is safe
    ' when the process was launched without a usable console window.
    CON.END
    PRINT #1, "pre-end handle="; CON.HANDLE; " err="; ERR

    CON.NEW
    h1 = CON.HANDLE
    e1 = ERR
    PRINT #1, "new1 handle="; h1; " err="; e1

    CON.END
    PRINT #1, "end handle="; CON.HANDLE; " err="; ERR

    CON.NEW
    h2 = CON.HANDLE
    e2 = ERR
    PRINT #1, "new2 handle="; h2; " err="; e2

    CLOSE #1
    CON.END
END FUNCTION
