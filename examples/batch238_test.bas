' batch238_test.bas - DIALOG SHOW MODAL hDlg TO Result (official PBWin syntax)
' batch 238 parser addition: HelloDDT-style TO result form.
' Compile-only GUI example (interactive: never run headless in CI).
' Both PBWin10 and PB64Basic must compile this file unchanged.
#COMPILE EXE
#DIM ALL

FUNCTION PBMAIN() AS LONG
    LOCAL hDlg   AS LONG
    LOCAL Result AS LONG

    DIALOG NEW 0, "HelloDDT", 100, 100, 220, 90 TO hDlg
    CONTROL ADD BUTTON, hDlg, 100, "OK", 70, 35, 80, 20
    DIALOG SHOW MODAL hDlg TO Result
    DIALOG END hDlg

    FUNCTION = 0
END FUNCTION
