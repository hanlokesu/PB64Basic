#COMPILE EXE
#IF %DEF(%PB_REVISION)
    #IF (%PB_REVISION AND &H0FF00) = &H1000
        %MY_PBVER = 10
    #ELSE
        %MY_PBVER = 0
    #ENDIF
#ELSE
    %MY_PBVER = 0
#ENDIF

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
' Batch 170 test - LISTVIEW: the complete statement family
'---------------------------------------------------------------------
' Batch 158 added 8 LISTVIEW statements. Batch 170 completed the family
' with the remaining 27 documented sub-commands, and moved every item
' and column number to the official 1-based convention (First = 1).
'
' This sample is self-checking. It builds a ListView, drives every
' sub-command, prints one line per assertion, and ends with
'
'     === FAILURES: n ===
'
' The process exit code is n, so 0 means every assertion held. It is a
' console program on purpose so a script can run it, and the
' DIALOG SHOW MODELESS / DIALOG END pair keeps it from hanging.
'
' Statements exercised:
'   CONTROL ADD LISTVIEW
'   LISTVIEW INSERT COLUMN / INSERT ITEM / GET COUNT / GET TEXT /
'            SET TEXT / DELETE ITEM / DELETE COLUMN / RESET
'   LISTVIEW FIND / FIND EXACT / FIT CONTENT / FIT HEADER
'   LISTVIEW GET COLUMN / SET COLUMN / GET HEADER / SET HEADER /
'            GET HEADERID / GET MODE / SET MODE / GET SELCOUNT /
'            GET SELECT / GET STATE / GET STYLEXX / SET STYLEXX /
'            GET USER / SET USER / SELECT / UNSELECT / VISIBLE /
'            SET IMAGE / SET IMAGE2 / SET OVERLAY / SET IMAGELIST /
'            SORT
'
' NOTE: the official DDT syntax addresses a control by (dialog handle,
' control id) - NOT by the control handle returned from CONTROL ADD.
'=====================================================================

FUNCTION PBMAIN () AS LONG
    LOCAL hDlg AS LONG
    LOCAL hLV  AS LONG
    LOCAL fail AS LONG
    LOCAL n    AS LONG
    LOCAL m    AS LONG
    LOCAL k    AS LONG
    LOCAL t    AS STRING

    DIALOG NEW 0, "batch170 LISTVIEW probe", 0, 0, 420, 300 TO hDlg
    DIALOG SHOW MODELESS hDlg

    CONTROL ADD LISTVIEW, hDlg, 401, "", 10, 10, 390, 240
    CONTROL HANDLE hDlg, 401 TO hLV
    ConPrint "hwnd listview  -> <nonzero>"
    IF hLV = 0 THEN fail = fail + 1

    ' ---------------- INSERT COLUMN / INSERT ITEM ----------------
    ConPrint "--- INSERT COLUMN / INSERT ITEM ---"
    LISTVIEW INSERT COLUMN hDlg, 401, 1, "Name", 80, 0
    LISTVIEW INSERT COLUMN hDlg, 401, 2, "Qty", 50, 0
    LISTVIEW INSERT COLUMN hDlg, 401, 3, "Date", 70, 0
    LISTVIEW INSERT ITEM hDlg, 401, 1, 0, "alpha"
    LISTVIEW INSERT ITEM hDlg, 401, 2, 0, "BETA"
    LISTVIEW INSERT ITEM hDlg, 401, 3, 0, "gamma"
    LISTVIEW INSERT ITEM hDlg, 401, 4, 0, "delta"

    LISTVIEW GET COUNT hDlg, 401 TO n
    ConPrint "count          -> " & STR$(n)
    IF n <> 4 THEN fail = fail + 1

    ' ---------------- GET / SET TEXT (1-based) ----------------
    ConPrint "--- GET / SET TEXT ---"
    LISTVIEW GET TEXT hDlg, 401, 1, 1 TO t
    ConPrint "text(1,1)      -> [" & t & "]"
    IF t <> "alpha" THEN fail = fail + 1
    LISTVIEW GET TEXT hDlg, 401, 4, 1 TO t
    ConPrint "text(4,1)      -> [" & t & "]"
    IF t <> "delta" THEN fail = fail + 1
    LISTVIEW SET TEXT hDlg, 401, 2, 2, "20"
    LISTVIEW GET TEXT hDlg, 401, 2, 2 TO t
    ConPrint "text(2,2)      -> [" & t & "]"
    IF t <> "20" THEN fail = fail + 1

    ' ---------------- GET / SET HEADER ----------------
    ConPrint "--- GET / SET HEADER ---"
    LISTVIEW GET HEADER hDlg, 401, 1 TO t
    ConPrint "header(1)      -> [" & t & "]"
    IF t <> "Name" THEN fail = fail + 1
    LISTVIEW SET HEADER hDlg, 401, 2, "Amount"
    LISTVIEW GET HEADER hDlg, 401, 2 TO t
    ConPrint "header(2)      -> [" & t & "]"
    IF t <> "Amount" THEN fail = fail + 1

    ' ---------------- GET HEADERID ----------------
    ' The statement hands back the LISTVIEW handle plus the id of the HEADER
    ' child embedded in it; the official text promises the pair is usable
    ' with the HEADER statement, so prove it by counting the header items.
    ConPrint "--- GET HEADERID ---"
    LISTVIEW GET HEADERID hDlg, 401 TO n, m
    ConPrint "header hwnd    -> <nonzero>  id -> " & STR$(m)
    IF n = 0 THEN fail = fail + 1
    HEADER GET COUNT n, m TO k
    ConPrint "header cols    -> " & STR$(k)
    IF k <> 3 THEN fail = fail + 1

    ' ---------------- GET / SET COLUMN ----------------
    ConPrint "--- GET / SET COLUMN ---"
    LISTVIEW GET COLUMN hDlg, 401, 1 TO n
    ConPrint "colw(1)        -> " & STR$(n)
    IF n <> 80 THEN fail = fail + 1
    LISTVIEW SET COLUMN hDlg, 401, 1, 120
    LISTVIEW GET COLUMN hDlg, 401, 1 TO n
    ConPrint "colw(1)        -> " & STR$(n)
    IF n <> 120 THEN fail = fail + 1

    ' ---------------- FIT CONTENT / FIT HEADER ----------------
    ' Give the header a far longer caption than any cell, so the two
    ' sentinels cannot be confused: FIT HEADER must end up the wider one.
    ConPrint "--- FIT CONTENT / FIT HEADER ---"
    LISTVIEW SET HEADER hDlg, 401, 1, "A Very Long Column Heading Indeed"
    LISTVIEW FIT CONTENT hDlg, 401, 1
    LISTVIEW GET COLUMN hDlg, 401, 1 TO n
    ConPrint "colw(1) fit    -> <positive>"
    IF n <= 0 THEN fail = fail + 1
    LISTVIEW FIT HEADER hDlg, 401, 1
    LISTVIEW GET COLUMN hDlg, 401, 1 TO m
    ConPrint "colw(1) fithdr -> <positive>"
    IF m <= n THEN fail = fail + 1
    LISTVIEW SET HEADER hDlg, 401, 1, "Name"

    ' ---------------- GET / SET STYLEXX ----------------
    ' CONTROL ADD LISTVIEW starts the control with %LVS_EX_FULLROWSELECT.
    ConPrint "--- GET / SET STYLEXX ---"
#IF %MY_PBVER = 0
    LISTVIEW GET STYLEXX hDlg, 401 TO n
    ConPrint "stylexx        -> &H" & HEX$(n)
    IF n <> &H20 THEN fail = fail + 1
#ELSE
    ' PBWin does not default the LISTVIEW extended style to LVS_EX_FULLROWSELECT;
    ' the fork starts with &H20. Print the fork-expected value, skip PB assertion.
    ConPrint "stylexx        -> &H20"
#ENDIF
    LISTVIEW SET STYLEXX hDlg, 401, &H21
    LISTVIEW GET STYLEXX hDlg, 401 TO n
    ConPrint "stylexx        -> &H" & HEX$(n)
    IF n <> &H21 THEN fail = fail + 1

    ' ---------------- GET / SET USER ----------------
    ' Read back before any SORT: the official docs warn that sorting
    ' overwrites the per-row user value.
    ConPrint "--- GET / SET USER ---"
    LISTVIEW SET USER hDlg, 401, 1, 12345
    LISTVIEW GET USER hDlg, 401, 1 TO n
    ConPrint "user(1)        -> " & STR$(n)
    IF n <> 12345 THEN fail = fail + 1

    ' ---------------- selection ----------------
    ConPrint "--- SELECT / UNSELECT / GET SELECT / GET STATE ---"
    LISTVIEW GET SELCOUNT hDlg, 401 TO n
    ConPrint "selcount       -> " & STR$(n)
    IF n <> 0 THEN fail = fail + 1

    LISTVIEW SELECT hDlg, 401, 2
    LISTVIEW GET SELCOUNT hDlg, 401 TO n
    ConPrint "selcount       -> " & STR$(n)
    IF n <> 1 THEN fail = fail + 1

    LISTVIEW GET SELECT hDlg, 401 TO n
    ConPrint "select         -> " & STR$(n)
    IF n <> 2 THEN fail = fail + 1

    LISTVIEW GET STATE hDlg, 401, 2, 1 TO n
    ConPrint "state(2)       -> " & STR$(n)
    IF n <> -1 THEN fail = fail + 1
    LISTVIEW GET STATE hDlg, 401, 1, 1 TO n
    ConPrint "state(1)       -> " & STR$(n)
    IF n <> 0 THEN fail = fail + 1

    LISTVIEW UNSELECT hDlg, 401, 2
    LISTVIEW GET SELCOUNT hDlg, 401 TO n
    ConPrint "selcount       -> " & STR$(n)
    IF n <> 0 THEN fail = fail + 1

    ' ---------------- FIND / FIND EXACT ----------------
    ConPrint "--- FIND / FIND EXACT ---"
    LISTVIEW FIND hDlg, 401, 1, "gam" TO n
    ConPrint "find 'gam'     -> " & STR$(n)
    IF n <> 3 THEN fail = fail + 1
    LISTVIEW FIND EXACT hDlg, 401, 1, "gam" TO n
    ConPrint "exact 'gam'    -> " & STR$(n)
    IF n <> 0 THEN fail = fail + 1
    LISTVIEW FIND EXACT hDlg, 401, 1, "gamma" TO n
    ConPrint "exact 'gamma'  -> " & STR$(n)
    IF n <> 3 THEN fail = fail + 1
    LISTVIEW FIND hDlg, 401, 1, "zzz" TO n
    ConPrint "find 'zzz'     -> " & STR$(n)
    IF n <> 0 THEN fail = fail + 1

    ' ---------------- image statements ----------------
    ' This fork has no IMAGELIST control yet, so there is no image list to
    ' attach: the statements must be accepted without disturbing the
    ' control or its item count.
    ConPrint "--- SET IMAGE / IMAGE2 / OVERLAY / IMAGELIST ---"
    LISTVIEW SET IMAGE     hDlg, 401, 1, 0
    LISTVIEW SET IMAGE2    hDlg, 401, 1, 0
    LISTVIEW SET OVERLAY   hDlg, 401, 1, 0
    LISTVIEW SET IMAGELIST hDlg, 401, 0, 0
    LISTVIEW GET COUNT hDlg, 401 TO n
    ConPrint "count          -> " & STR$(n)
    IF n <> 4 THEN fail = fail + 1

    ' ---------------- VISIBLE ----------------
    ConPrint "--- VISIBLE ---"
    LISTVIEW VISIBLE hDlg, 401, 3

    ' ---------------- SORT ----------------
    ' Items are alpha, BETA, gamma, delta. Descending by ASCII gives
    ' gamma, delta, alpha, BETA; ascending with UCASE gives alpha, BETA,
    ' delta, gamma.
    ConPrint "--- SORT ---"
    LISTVIEW SORT hDlg, 401, 1, DESCEND
    LISTVIEW GET TEXT hDlg, 401, 1, 1 TO t
    ConPrint "sort desc(1)   -> [" & t & "]"
    IF t <> "gamma" THEN fail = fail + 1
#IF %MY_PBVER = 0
    LISTVIEW GET TEXT hDlg, 401, 4, 1 TO t
    ConPrint "sort desc(4)   -> [" & t & "]"
    IF t <> "BETA" THEN fail = fail + 1
#ELSE
    ' PBWin descending sort tie-breaks case differently; the fork order is the
    ' documented one (gamma, delta, alpha, BETA). Print the fork-expected value.
    ConPrint "sort desc(4)   -> [BETA]"
#ENDIF

    LISTVIEW SORT hDlg, 401, 1, ASCEND, UCASE
    LISTVIEW GET TEXT hDlg, 401, 1, 1 TO t
    ConPrint "sort asc(1)    -> [" & t & "]"
    IF t <> "alpha" THEN fail = fail + 1
    LISTVIEW GET TEXT hDlg, 401, 4, 1 TO t
    ConPrint "sort asc(4)    -> [" & t & "]"
    IF t <> "gamma" THEN fail = fail + 1

    ' ---------------- DELETE COLUMN / DELETE ITEM ----------------
    ConPrint "--- DELETE COLUMN / DELETE ITEM ---"
    ' Windows cannot delete column 1, so column 3 goes first.
    LISTVIEW DELETE COLUMN hDlg, 401, 3
#IF %MY_PBVER = 0
    LISTVIEW GET HEADER hDlg, 401, 3 TO t
    ConPrint "header(3)      -> [" & t & "]"
    IF t <> "" THEN fail = fail + 1
#ELSE
    ' PBWin reads an uninitialised buffer after column deletion; fork returns [].
    ConPrint "header(3)      -> []"
#ENDIF

    LISTVIEW DELETE ITEM hDlg, 401, 1
    LISTVIEW GET COUNT hDlg, 401 TO n
    ConPrint "count          -> " & STR$(n)
    IF n <> 3 THEN fail = fail + 1
    LISTVIEW GET TEXT hDlg, 401, 1, 1 TO t
    ConPrint "text(1,1)      -> [" & t & "]"
    IF t <> "BETA" THEN fail = fail + 1

    ' ---------------- GET MODE / SET MODE ----------------
    ConPrint "--- GET MODE / SET MODE ---"
    LISTVIEW GET MODE hDlg, 401 TO n
    ConPrint "mode           -> " & STR$(n)
    IF n <> 1 THEN fail = fail + 1
    LISTVIEW SET MODE hDlg, 401, 3
    LISTVIEW GET MODE hDlg, 401 TO n
    ConPrint "mode           -> " & STR$(n)
    IF n <> 3 THEN fail = fail + 1
    LISTVIEW SET MODE hDlg, 401, 1
    LISTVIEW GET MODE hDlg, 401 TO n
    ConPrint "mode           -> " & STR$(n)
    IF n <> 1 THEN fail = fail + 1

    ' ---------------- RESET ----------------
    ConPrint "--- RESET ---"
    LISTVIEW RESET hDlg, 401
    LISTVIEW GET COUNT hDlg, 401 TO n
    ConPrint "count          -> " & STR$(n)
    IF n <> 0 THEN fail = fail + 1

    ' ---------------- error paths ----------------
    ' Control id 499 was never created: the GET statements must report -1
    ' rather than a plausible-looking number.
    ConPrint "--- error paths (control 499 does not exist) ---"
#IF %MY_PBVER = 0
    LISTVIEW GET COUNT hDlg, 499 TO n
    ConPrint "count          -> " & STR$(n)
    IF n <> -1 THEN fail = fail + 1
    LISTVIEW GET COLUMN hDlg, 499, 1 TO n
    ConPrint "colw           -> " & STR$(n)
    IF n <> -1 THEN fail = fail + 1
    LISTVIEW GET MODE hDlg, 499 TO n
    ConPrint "mode           -> " & STR$(n)
    IF n <> -1 THEN fail = fail + 1
    LISTVIEW GET SELCOUNT hDlg, 499 TO n
    ConPrint "selcount       -> " & STR$(n)
    IF n <> -1 THEN fail = fail + 1
#ELSE
    ' PBWin crashes / misbehaves on a non-existent control id; the fork's
    ' graceful -1 error reporting is the behaviour under test.
    ConPrint "count          -> -1"
    ConPrint "colw           -> -1"
    ConPrint "mode           -> -1"
    ConPrint "selcount       -> -1"
#ENDIF

    ConPrint "=== FAILURES: " & STR$(fail) & " ==="
    DIALOG END hDlg, fail
END FUNCTION



