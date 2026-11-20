' batch396_test.bas - DIALOG SHOW MODELESS callback/result and
' DIALOG SHOW MODAL callback+TO regression.
' Both dialogs are ended from the posted WM_COMMAND, so the test is
' self-terminating and can run headless.
#COMPILE EXE
#CONSOLE ON
#DIM ALL

GLOBAL g_modeless_called AS LONG
GLOBAL g_modal_called AS LONG

CALLBACK FUNCTION ModelessProc() AS LONG
    g_modeless_called = 1
    DIALOG END CB.HNDL, 42
    FUNCTION = 0
END FUNCTION

CALLBACK FUNCTION ModalProc() AS LONG
    g_modal_called = 1
    DIALOG END CB.HNDL, 77
    FUNCTION = 0
END FUNCTION

FUNCTION PBMAIN() AS LONG
    LOCAL failures AS LONG
    LOCAL hDialog AS LONG
    LOCAL result AS LONG
    LOCAL modeless_result AS LONG
    LOCAL modal_result AS LONG
    LOCAL prev_show AS LONG
    LOCAL prev_hide AS LONG
    LOCAL live_count AS LONG

    ' MODELESS: the callback must be registered and the TO target must
    ' receive the DIALOG END result.
    DIALOG NEW 0, "batch396 modeless",,, 160, 60 TO hDialog
    DIALOG SHOW MODELESS hDialog, CALL ModelessProc TO result
    DIALOG POST hDialog, %WM_COMMAND, 100, 0
    DIALOG DOEVENTS
    modeless_result = result
    IF g_modeless_called <> 1 THEN failures = failures + 1
    IF modeless_result <> 42 THEN failures = failures + 1

    ' MODAL: CALL + TO previously dropped the result.
    DIALOG NEW 0, "batch396 modal",,, 160, 60 TO hDialog
    DIALOG POST hDialog, %WM_COMMAND, 100, 0
    DIALOG SHOW MODAL hDialog, CALL ModalProc TO result
    modal_result = result
    IF g_modal_called <> 1 THEN failures = failures + 1
    IF modal_result <> 77 THEN failures = failures + 1

    ' SHOW STATE TO and DOEVENTS [sleep] [TO count].
    DIALOG NEW 0, "batch396 state",,, 160, 60 TO hDialog
    DIALOG SHOW STATE hDialog, 5 TO prev_show
    DIALOG SHOW STATE hDialog, 0 TO prev_hide
    IF prev_show <> 0 THEN failures = failures + 1
    ' ShowWindow returns a non-zero "was previously visible" value, not
    ' necessarily 1, so only the zero/non-zero contract is asserted.
    IF prev_hide = 0 THEN failures = failures + 1
    DIALOG END hDialog, 0
    DIALOG DOEVENTS 25 TO live_count
    IF live_count <> 0 THEN failures = failures + 1

    PRINT "=== FAILURES:" + STR$(failures) + " ==="
    OPEN "batch396_result.txt" FOR OUTPUT AS #1
    PRINT #1, "modeless=" + STR$(g_modeless_called) + ",result=" + STR$(modeless_result)
    PRINT #1, "modal=" + STR$(g_modal_called) + ",result=" + STR$(modal_result)
    PRINT #1, "state=" + STR$(prev_show) + "," + STR$(prev_hide) + ",live=" + STR$(live_count)
    PRINT #1, "failures=" + STR$(failures)
    CLOSE #1
    FUNCTION = failures
END FUNCTION
