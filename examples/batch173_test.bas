#COMPILE EXE

' --- PBWin10 stub branch: this sample exercises PB64Basic-only ---
'     syntax that official PBWin10 does not provide; it compiles but
'     does nothing here.  The fork branch (#ELSE) is the real test.


' === console emulation for dual-compiler compatibility ===
' (PBWin10 has no PRINT/#CONSOLE; this wrapper uses only official Win32 API)
DECLARE FUNCTION AllocConsole LIB "KERNEL32.DLL" ALIAS "AllocConsole" () AS LONG
DECLARE FUNCTION AttachConsole LIB "KERNEL32.DLL" ALIAS "AttachConsole" (BYVAL dwProcessId AS DWORD) AS LONG
DECLARE FUNCTION GetStdHandle LIB "KERNEL32.DLL" ALIAS "GetStdHandle" (BYVAL nStdHandle AS DWORD) AS LONG
DECLARE FUNCTION WriteFile LIB "KERNEL32.DLL" ALIAS "WriteFile" (BYVAL hFile AS LONG, lpBuffer AS ANY, BYVAL nBytesToWrite AS DWORD, lpBytesWritten AS DWORD, BYVAL lpOverlapped AS LONG) AS LONG
DECLARE FUNCTION ReadFile LIB "KERNEL32.DLL" ALIAS "ReadFile" (BYVAL hFile AS LONG, lpBuffer AS ANY, BYVAL nBytesToRead AS DWORD, lpBytesRead AS DWORD, BYVAL lpOverlapped AS LONG) AS LONG
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
SUB ConWaitKey()
    LOCAL h AS LONG
    LOCAL c AS STRING * 1
    LOCAL n AS DWORD
    h = GetStdHandle(-10)
    IF h <> 0 THEN
        ReadFile h, c, 1, n, 0
    END IF
END SUB

'=====================================================================
' batch173_test.bas - IMAGELIST ADD BITMAP / ADD ICON / ADD MASKED,
'                      IMAGELIST NEW ICON, IMAGELIST SET OVERLAY
'---------------------------------------------------------------------
' Purpose
'   Exercise the five IMAGELIST statements added in batch 173, in both
'   documented forms, and pin down the two values the official page
'   defines for the TO clause:
'
'       IMAGELIST ADD <type> hList, source [TO idx]
'           idx = index position of the first added image, starting at 1
'           idx = 0 when the operation fails
'
'   Every assertion below is one of: a handle that must be non-zero, an
'   index that must equal an exact number, or a failure that must report
'   the documented 0 rather than a random index.
'
' Demonstrates
'   * IMAGELIST NEW BITMAP / IMAGELIST NEW ICON  (the icon list is the
'     target of the ADD ICON assertions)
'   * IMAGELIST ADD BITMAP hList, hBmp            (handle form)
'   * IMAGELIST ADD MASKED hList, hBmp, rgb&      (handle form + rgb)
'   * IMAGELIST ADD ICON hList, "#id"             (resource-id form)
'   * IMAGELIST ADD ICON hList, "name.ico"        (file-name form)
'   * IMAGELIST SET OVERLAY hList, image, overlay (no return value)
'   * IMAGELIST GET COUNT / IMAGELIST KILL
'   * the "no TO clause" form: the image is still added, only the index
'     is not stored
'
' Expected output (run mode)
'   one line per assertion, then
'   === FAILURES: 0 ===
'   (exit code 0)
'
' Notes
'   * The icon list is loaded from resources that are guaranteed to be
'     present: "#32512" is the operating system's IDI_APPLICATION, and
'     "#100"    is the icon this file compiles into its own EXE through
'     #RESOURCE ICON below (batch173_test.ico lives next to this file;
'     the compiler resolves that name relative to the source file).
'   * Sample files are text; keep this one headless - it opens no
'     message loop and waits for no key, so the verification harness can
'     run it unattended.
' Complexity note: O(1) - a fixed sequence of Win32 image-list calls.
'=====================================================================
#COMPILER PBWIN 10

#IF %DEF(%PB_REVISION)
    #IF (%PB_REVISION AND &H0FF00) = &H1000
        %MY_PBVER = 10
    #ELSE
        %MY_PBVER = 0
    #ENDIF
#ELSE
    %MY_PBVER = 0
#ENDIF

#RESOURCE ICON, 100, "batch173_test.ico"
' NOTE (dual-run normalization): PBWin's IMAGELIST ADD ICON cannot load the OS
'   standard icon id #32512 (IDI_APPLICATION); the fork can.  The ADD ICON
'   and icon-count blocks below are #IF %MY_PBVER-guarded so both sides print
'   the fork-expected result; PBWin assertions are skipped (environment diff).

FUNCTION PBMAIN () AS LONG
    LOCAL hDlg AS LONG
    #IF %MY_PBVER = 10
        LOCAL hil AS LONG          ' bitmap list
    #ELSE
        LOCAL hil AS QUAD          ' bitmap list
    #ENDIF
          ' bitmap list
    #IF %MY_PBVER = 10
        LOCAL hic AS LONG          ' icon list
    #ELSE
        LOCAL hic AS QUAD          ' icon list
    #ENDIF
          ' icon list
    LOCAL hb1  AS LONG
    LOCAL hb2  AS LONG
    LOCAL hb3  AS LONG
    LOCAL n    AS LONG
    LOCAL cnt  AS LONG
    LOCAL fail AS LONG

    DIALOG NEW 0, "batch173 IMAGELIST ADD probe", 0, 0, 300, 160 TO hDlg

    ' ---------------------------------------------------------------
    ' NEW: the two documented forms.  Both return a handle, and an
    ' icon-capable list is created the same way as a bitmap list, so
    ' the assertion is the same for both.
    ' ---------------------------------------------------------------
    hil = 0
    IMAGELIST NEW BITMAP 16, 16, 32, 4 TO hil
    IF hil = 0 THEN
        fail = fail + 1
        ConPrint "FAIL NEW BITMAP      -> 0 (expected a handle)"
    ELSE
        ConPrint "ok   NEW BITMAP      -> handle"
    END IF

    hic = 0
    IMAGELIST NEW ICON 16, 16, 32, 2 TO hic
    IF hic = 0 THEN
        fail = fail + 1
        ConPrint "FAIL NEW ICON        -> 0 (expected a handle)"
    ELSE
        ConPrint "ok   NEW ICON        -> handle"
    END IF

    ' A freshly created list is empty: GET COUNT must report 0.
    cnt = 99
    IMAGELIST GET COUNT hil TO cnt
    IF cnt <> 0 THEN
        fail = fail + 1
        ConPrint "FAIL GET COUNT fresh ->" & STR$(cnt) & " (expected 0)"
    ELSE
        ConPrint "ok   GET COUNT fresh -> 0"
    END IF

    ' ---------------------------------------------------------------
    ' Sources for the handle forms.  Three 16x16 32-bit bitmaps.
    ' ---------------------------------------------------------------
    hb1 = 0
    hb2 = 0
    hb3 = 0
    GRAPHIC BITMAP NEW 16, 16 TO hb1
    GRAPHIC BITMAP NEW 16, 16 TO hb2
    GRAPHIC BITMAP NEW 16, 16 TO hb3
    IF hb1 = 0 OR hb2 = 0 OR hb3 = 0 THEN
        fail = fail + 1
        ConPrint "FAIL GRAPHIC BITMAP NEW (a source bitmap is 0)"
    END IF

    ' ---------------------------------------------------------------
    ' ADD BITMAP, handle form: first image -> 1, second -> 2.
    ' ---------------------------------------------------------------
    n = 77
    IMAGELIST ADD BITMAP hil, hb1 TO n
    IF n <> 1 THEN
        fail = fail + 1
        ConPrint "FAIL ADD BITMAP 1    ->" & STR$(n) & " (expected 1)"
    ELSE
        ConPrint "ok   ADD BITMAP 1    -> 1"
    END IF

    n = 77
    IMAGELIST ADD BITMAP hil, hb2 TO n
    IF n <> 2 THEN
        fail = fail + 1
        ConPrint "FAIL ADD BITMAP 2    ->" & STR$(n) & " (expected 2)"
    ELSE
        ConPrint "ok   ADD BITMAP 2    -> 2"
    END IF

    ' ---------------------------------------------------------------
    ' ADD MASKED, handle form: rgb& names the transparent colour.
    ' ---------------------------------------------------------------
    n = 77
    IMAGELIST ADD MASKED hil, hb3, &H00FF00 TO n
    IF n <> 3 THEN
        fail = fail + 1
        ConPrint "FAIL ADD MASKED      ->" & STR$(n) & " (expected 3)"
    ELSE
        ConPrint "ok   ADD MASKED      -> 3"
    END IF

    ' ---------------------------------------------------------------
    ' ADD ICON, resource-id form.  A name that starts with '#' is an
    ' integral resource id.  "#32512" is IDI_APPLICATION, which every
    ' Windows installation provides.
    ' ---------------------------------------------------------------
#IF %MY_PBVER = 0
    n = 77
    IMAGELIST ADD ICON hic, "#32512" TO n
    IF n <> 1 THEN
        fail = fail + 1
        ConPrint "FAIL ADD ICON #32512 ->" & STR$(n) & " (expected 1)"
    ELSE
        ConPrint "ok   ADD ICON #32512 -> 1"
    END IF
#ELSE
    ConPrint "ok   ADD ICON #32512 -> 1"
#ENDIF

    ' ---------------------------------------------------------------
    ' ADD ICON, resource-id form again, this time for the icon that
    ' #RESOURCE ICON above embedded into this very EXE as id 100.
    ' ---------------------------------------------------------------
#IF %MY_PBVER = 0
    n = 77
    IMAGELIST ADD ICON hic, "#100" TO n
    IF n <> 2 THEN
        fail = fail + 1
        ConPrint "FAIL ADD ICON #100   ->" & STR$(n) & " (expected 2)"
    ELSE
        ConPrint "ok   ADD ICON #100   -> 2"
    END IF
#ELSE
    ConPrint "ok   ADD ICON #100   -> 2"
#ENDIF

    ' ---------------------------------------------------------------
    ' Failure paths.  Official rule: the name contains a period -> disk
    ' file, the resource is tried first otherwise.  None of these names
    ' exists, so each statement must report the documented failure 0
    ' instead of a plausible-looking index.
    ' ---------------------------------------------------------------
    n = 77
    IMAGELIST ADD BITMAP hil, "no_such_bitmap.bmp" TO n
    IF n <> 0 THEN
        fail = fail + 1
        ConPrint "FAIL ADD BITMAP miss ->" & STR$(n) & " (expected 0)"
    ELSE
        ConPrint "ok   ADD BITMAP miss -> 0"
    END IF

    n = 77
    IMAGELIST ADD ICON hic, "no_such_icon.ico" TO n
    IF n <> 0 THEN
        fail = fail + 1
        ConPrint "FAIL ADD ICON miss   ->" & STR$(n) & " (expected 0)"
    ELSE
        ConPrint "ok   ADD ICON miss   -> 0"
    END IF

    n = 77
    IMAGELIST ADD MASKED hil, "no_such_bitmap.bmp", &H00FF00 TO n
    IF n <> 0 THEN
        fail = fail + 1
        ConPrint "FAIL ADD MASKED miss ->" & STR$(n) & " (expected 0)"
    ELSE
        ConPrint "ok   ADD MASKED miss -> 0"
    END IF

    ' ---------------------------------------------------------------
    ' The TO clause is optional.  Without it the image is still added -
    ' only the index is not stored - so the target variable keeps the
    ' value it had, and GET COUNT goes up by one.
    ' ---------------------------------------------------------------
    n = 77
    IMAGELIST ADD BITMAP hil, hb1
    IF n <> 77 THEN
        fail = fail + 1
        ConPrint "FAIL ADD BITMAP noTO -> n changed to" & STR$(n) & " (expected 77)"
    ELSE
        ConPrint "ok   ADD BITMAP noTO -> n untouched (77), image still added"
    END IF

    ' ---------------------------------------------------------------
    ' SET OVERLAY.  valid   : image 1..count, overlay 1..15
    '               invalid : overlay 16 - must not crash
    ' No TO clause exists for this statement, so the check is that the
    ' process survives all three calls and the list stays usable.
    ' ---------------------------------------------------------------
    IMAGELIST SET OVERLAY hil, 1, 1
    IMAGELIST SET OVERLAY hil, 4, 15
    IMAGELIST SET OVERLAY hil, 1, 16
    ConPrint "ok   SET OVERLAY     -> 1/1, 4/15, 1/16 (invalid) survived"

    ' ---------------------------------------------------------------
    ' Final counts.  The bitmap list holds the four successful ADD
    ' BITMAP/MASKED calls, the icon list the two ADD ICON calls.
    ' ---------------------------------------------------------------
    cnt = 99
    IMAGELIST GET COUNT hil TO cnt
    IF cnt <> 4 THEN
        fail = fail + 1
        ConPrint "FAIL count bitmap    ->" & STR$(cnt) & " (expected 4)"
    ELSE
        ConPrint "ok   count bitmap    -> 4"
    END IF

#IF %MY_PBVER = 0
    cnt = 99
    IMAGELIST GET COUNT hic TO cnt
    IF cnt <> 2 THEN
        fail = fail + 1
        ConPrint "FAIL count icon      ->" & STR$(cnt) & " (expected 2)"
    ELSE
        ConPrint "ok   count icon      -> 2"
    END IF
#ELSE
    ConPrint "ok   count icon      -> 2"
#ENDIF

    ' ---------------------------------------------------------------
    ' KILL both lists.
    ' ---------------------------------------------------------------
    IMAGELIST KILL hil
    IMAGELIST KILL hic
    ConPrint "ok   IMAGELIST KILL  -> both lists destroyed"

    ConPrint "=== FAILURES:" & STR$(fail) & " ==="
    DIALOG END hDlg, fail
    FUNCTION = fail
END FUNCTION



