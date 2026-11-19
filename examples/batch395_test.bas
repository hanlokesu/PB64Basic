' batch395_test.bas - DIALOG NEW optional separator comma (discussion #4).
' PBWin10 accepts `..., style, TO hDlg`; the created dialog is never shown,
' so the test stays headless and exits with the failure count.
#COMPILE EXE
#CONSOLE ON
#DIM ALL

FUNCTION PBMAIN() AS LONG
    LOCAL failures AS LONG
    LOCAL hDialog AS LONG

    DIALOG NEW 0, "DDT Button Example", , , 150, 75, %WS_SYSMENU OR %WS_CAPTION, TO hDialog
    IF hDialog = 0 THEN failures = failures + 1

    PRINT "=== FAILURES:" + STR$(failures) + " ==="
    FUNCTION = failures
END FUNCTION