' batch249: CONTROL ADD full official syntax (CHW) for 12 controls
'   CONTROL ADD <CTL>, hDlg, id&, [txt$|items$()], x, y, xx, yy
'     [, [style&] [, [exstyle&]]] [[,] CALL callback]   (no TO; handle via CONTROL HANDLE)
#COMPILE EXE
#DIM ALL
GLOBAL hDlg AS LONG
SUB ConPrint(BYVAL s AS STRING)
    LOCAL h, n AS LONG
    OPEN "CONS" FOR OUTPUT AS #2
    PRINT #2, s
    CLOSE #2
END SUB
FUNCTION PBMAIN() AS LONG
    LOCAL hB, hC, hC3, hO, hF, hL, hLn, hT, hS, hLB, hCB, hTab AS LONG
    LOCAL fail AS LONG
    LOCAL items() AS STRING
    DIM items(1 TO 3)
    items(1) = "Alpha"
    items(2) = "Beta"
    items(3) = "Gamma"
    DIALOG NEW 0, "Control Full Syntax", 0, 0, 280, 460 TO hDlg
    CONTROL ADD BUTTON, hDlg, 101, "Btn", 10, 10, 60, 14 CALL BtnCB()
    CONTROL HANDLE hDlg, 101 TO hB
    CONTROL ADD CHECKBOX, hDlg, 102, "Chk", 10, 30, 60, 14 CALL ChkCB()
    CONTROL HANDLE hDlg, 102 TO hC
    CONTROL ADD CHECK3STATE, hDlg, 103, "Chk3", 10, 50, 60, 14 CALL Chk3CB()
    CONTROL HANDLE hDlg, 103 TO hC3
    CONTROL ADD OPTION, hDlg, 104, "Opt", 10, 70, 60, 14 CALL OptCB()
    CONTROL HANDLE hDlg, 104 TO hO
    CONTROL ADD FRAME, hDlg, 105, "Frame", 10, 90, 120, 60
    CONTROL HANDLE hDlg, 105 TO hF
    CONTROL ADD LABEL, hDlg, 106, "Lbl", 10, 160, 60, 14 CALL LblCB()
    CONTROL HANDLE hDlg, 106 TO hL
    CONTROL ADD LINE, hDlg, 107, "", 10, 180, 100, 4
    CONTROL HANDLE hDlg, 107 TO hLn
    CONTROL ADD TEXTBOX, hDlg, 108, "", 10, 190, 100, 16 CALL TxtCB()
    CONTROL HANDLE hDlg, 108 TO hT
    CONTROL ADD SCROLLBAR, hDlg, 109, "", 10, 220, 100, 20
    CONTROL HANDLE hDlg, 109 TO hS
    CONTROL ADD LISTBOX, hDlg, 110, items(), 10, 250, 100, 60 CALL LbCB()
    CONTROL HANDLE hDlg, 110 TO hLB
    CONTROL ADD COMBOBOX, hDlg, 111, items(), 120, 250, 100, 80 CALL CbCB()
    CONTROL HANDLE hDlg, 111 TO hCB
    CONTROL ADD TAB, hDlg, 112, "", 120, 10, 100, 60
    CONTROL HANDLE hDlg, 112 TO hTab
    DIALOG END hDlg
    IF hB = 0 THEN fail = fail + 1
    IF hC = 0 THEN fail = fail + 1
    IF hC3 = 0 THEN fail = fail + 1
    IF hO = 0 THEN fail = fail + 1
    IF hF = 0 THEN fail = fail + 1
    IF hL = 0 THEN fail = fail + 1
    IF hLn = 0 THEN fail = fail + 1
    IF hT = 0 THEN fail = fail + 1
    IF hS = 0 THEN fail = fail + 1
    IF hLB = 0 THEN fail = fail + 1
    IF hCB = 0 THEN fail = fail + 1
    IF hTab = 0 THEN fail = fail + 1
    IF fail = 0 THEN
        OPEN "batch249_result.txt" FOR OUTPUT AS #1
        PRINT #1, "batch249 control add full ok"
        CLOSE #1
    ELSE
        OPEN "batch249_result.txt" FOR OUTPUT AS #1
        PRINT #1, "FAILURES=" + STR$(fail) + " B=" + STR$(hB) + " C=" + STR$(hC)
        CLOSE #1
    END IF
END FUNCTION
CALLBACK FUNCTION BtnCB() AS LONG
    FUNCTION = 0
END FUNCTION
CALLBACK FUNCTION ChkCB() AS LONG
    FUNCTION = 0
END FUNCTION
CALLBACK FUNCTION Chk3CB() AS LONG
    FUNCTION = 0
END FUNCTION
CALLBACK FUNCTION OptCB() AS LONG
    FUNCTION = 0
END FUNCTION
CALLBACK FUNCTION LblCB() AS LONG
    FUNCTION = 0
END FUNCTION
CALLBACK FUNCTION TxtCB() AS LONG
    FUNCTION = 0
END FUNCTION
CALLBACK FUNCTION LbCB() AS LONG
    FUNCTION = 0
END FUNCTION
CALLBACK FUNCTION CbCB() AS LONG
    FUNCTION = 0
END FUNCTION
