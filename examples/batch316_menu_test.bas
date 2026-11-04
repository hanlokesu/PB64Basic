#IF %DEF(%PB_REVISION)
    #IF (%PB_REVISION AND &H0FF00) = &H1000
        %MY_PBVER = 10
    #ELSE
        %MY_PBVER = 0
    #ENDIF
#ELSE
    %MY_PBVER = 0
#ENDIF

#COMPILE EXE
#DIM ALL

' batch316 - MENU statement family headless round-trip on an unattached HMENU.
' Covers: MENU NEW BAR / NEW POPUP, ADD STRING, ADD POPUP (called, result not
' asserted -- BLOCKED), GET/SET STATE, GET/SET TEXT, MOUSEPTR (called, result
' not asserted -- BLOCKED).  Values are written to mout.txt AND self-checked.
'
' BLOCKED entries (documented in report_menu.md, not self-checked here):
'   - MENU ADD POPUP: codegen arg order skips txt$ and misplaces hSub.
'   - MENU ADD STRING separator: fork does not create a real MF_SEPARATOR.
'   - MENU ADD STRING with MF_DEFAULT: default bit not preserved on add.
'   - MENU DELETE: off-by-one (1-based position not converted to 0-based);
'     BYCMD form not accepted by the parser.
'   - MOUSEPTR TO var&: runtime returns 1/0 instead of the previous cursor
'     handle (CHW: "handle of the previous cursor assigned to var&").
'   - MENU ATTACH / DRAW BAR / CONTEXT / MSGBOX: require a real window +
'     message loop; compile/link only (see report_menu.md).

DECLARE FUNCTION AllocConsole LIB "KERNEL32.DLL" ALIAS "AllocConsole" () AS LONG
DECLARE FUNCTION AttachConsole LIB "KERNEL32.DLL" ALIAS "AttachConsole" (BYVAL dwProcessId AS DWORD) AS LONG
DECLARE FUNCTION GetStdHandle LIB "KERNEL32.DLL" ALIAS "GetStdHandle" (BYVAL nStdHandle AS DWORD) AS LONG
DECLARE FUNCTION WriteFile LIB "KERNEL32.DLL" ALIAS "WriteFile" (BYVAL hFile AS LONG, lpBuffer AS ANY, BYVAL nBytesToWrite AS DWORD, lpBytesWritten AS DWORD, BYVAL lpOverlapped AS LONG) AS LONG

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

%MF_ENABLED = 0
%MF_GRAYED  = 1
%MF_CHECKED = 8

#IF %MY_PBVER = 10
FUNCTION PBMAIN() AS LONG
    LOCAL f AS LONG
    LOCAL hBar AS LONG, hPop AS LONG, hSub AS LONG
    LOCAL s AS STRING
    LOCAL st AS LONG

    f = FREEFILE
    OPEN "mout.txt" FOR OUTPUT AS #f

    MENU NEW BAR TO hBar
    MENU NEW POPUP TO hPop
    MENU NEW POPUP TO hSub
    PRINT #f, "BAR_H=" + TRIM$(STR$(hBar))
    PRINT #f, "POP_H=" + TRIM$(STR$(hPop))
    PRINT #f, "SUB_H=" + TRIM$(STR$(hSub))

    MENU ADD STRING, hPop, "&Alpha", 101, %MF_ENABLED
    MENU ADD STRING, hPop, "&Beta", 102, %MF_CHECKED

    MENU ADD POPUP, hBar, "&File", hSub, %MF_ENABLED

    MENU GET TEXT hPop, 1 TO s
    PRINT #f, "TXT1=[" + s + "]"
    MENU GET TEXT hPop, 2 TO s
    PRINT #f, "TXT2=[" + s + "]"

    MENU GET STATE hPop, BYCMD 101 TO st
    PRINT #f, "ST101=" + TRIM$(STR$(st))
    MENU GET STATE hPop, BYCMD 102 TO st
    PRINT #f, "ST102=" + TRIM$(STR$(st))
    MENU GET STATE hPop, BYCMD 999 TO st
    PRINT #f, "STMISS=" + TRIM$(STR$(st))

    MENU SET STATE hPop, BYCMD 101, %MF_CHECKED OR %MF_GRAYED
    MENU GET STATE hPop, BYCMD 101 TO st
    PRINT #f, "ST101B=" + TRIM$(STR$(st))

    MENU SET TEXT hPop, BYCMD 101, "&Alpha2"
    MENU GET TEXT hPop, BYCMD 101 TO s
    PRINT #f, "TXT1B=[" + s + "]"

    MOUSEPTR 11 TO st
    MOUSEPTR 1 TO st

    CLOSE #f
END FUNCTION
#ELSE
FUNCTION PBMAIN() AS LONG
    LOCAL f AS LONG, nfail AS LONG
    LOCAL hBar AS LONG, hPop AS LONG, hSub AS LONG
    LOCAL s AS STRING
    LOCAL st AS LONG

    nfail = 0
    f = FREEFILE
    OPEN "mout.txt" FOR OUTPUT AS #f

    MENU NEW BAR TO hBar
    MENU NEW POPUP TO hPop
    MENU NEW POPUP TO hSub
    PRINT #f, "BAR_H=" + TRIM$(STR$(hBar))
    PRINT #f, "POP_H=" + TRIM$(STR$(hPop))
    PRINT #f, "SUB_H=" + TRIM$(STR$(hSub))
    IF hBar = 0 THEN nfail = nfail + 1
    IF hPop = 0 THEN nfail = nfail + 1
    IF hSub = 0 THEN nfail = nfail + 1

    MENU ADD STRING, hPop, "&Alpha", 101, %MF_ENABLED
    MENU ADD STRING, hPop, "&Beta", 102, %MF_CHECKED

    MENU ADD POPUP, hBar, "&File", hSub, %MF_ENABLED

    MENU GET TEXT hPop, 1 TO s
    PRINT #f, "TXT1=[" + s + "]"
    IF s <> "&Alpha" THEN nfail = nfail + 1
    MENU GET TEXT hPop, 2 TO s
    PRINT #f, "TXT2=[" + s + "]"
    IF s <> "&Beta" THEN nfail = nfail + 1

    MENU GET STATE hPop, BYCMD 101 TO st
    PRINT #f, "ST101=" + TRIM$(STR$(st))
    IF st <> 0 THEN nfail = nfail + 1
    MENU GET STATE hPop, BYCMD 102 TO st
    PRINT #f, "ST102=" + TRIM$(STR$(st))
    IF st <> %MF_CHECKED THEN nfail = nfail + 1
    MENU GET STATE hPop, BYCMD 999 TO st
    PRINT #f, "STMISS=" + TRIM$(STR$(st))
    ' NOTE: CHW documents result=-1 for a missing item, but the fork's runtime
    ' leaves the destination unchanged on GetMenuState failure (LOCAL init=0).
    ' Empirically PBWin returns 0; the fork may return the state of the nearest
    ' item depending on menu contents.  This is a known BLOCKED sub-case; do
    ' not assert here.

    MENU SET STATE hPop, BYCMD 101, %MF_CHECKED OR %MF_GRAYED
    MENU GET STATE hPop, BYCMD 101 TO st
    PRINT #f, "ST101B=" + TRIM$(STR$(st))
    IF st <> (%MF_CHECKED OR %MF_GRAYED) THEN nfail = nfail + 1

    MENU SET TEXT hPop, BYCMD 101, "&Alpha2"
    MENU GET TEXT hPop, BYCMD 101 TO s
    PRINT #f, "TXT1B=[" + s + "]"
    IF s <> "&Alpha2" THEN nfail = nfail + 1

    MOUSEPTR 11 TO st
    MOUSEPTR 1 TO st

    CLOSE #f

    ConPrint "=== FAILURES:" + TRIM$(STR$(nfail)) + " ==="
    IF nfail = 0 THEN ConPrint "batch316 menu ok"
    FUNCTION = nfail
END FUNCTION
#ENDIF
