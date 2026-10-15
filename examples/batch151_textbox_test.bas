#COMPILE EXE
#DIM ALL
' Tier-3 DDT: TEXTBOX + LABEL + BUTTON (EnterMov)
' Auto-generated: PB64Basic fork batch test



%IDC_TEXT1 = 141
%IDC_TEXT2 = 142
%IDC_TEXT3 = 143
%IDC_TEXT4 = 144
%IDOK = 1
%IDCANCEL = 2
%WM_INITDIALOG = 272
%WM_COMMAND = 273
%BN_CLICKED = 0
%MB_TASKMODAL = 524288

FUNCTION PBMAIN() AS LONG
    LOCAL hDlg AS LONG
    DIALOG NEW 0, "Input text, press Enter",,, 191, 104, 0, 0 TO hDlg
    CONTROL ADD LABEL, hDlg, -1, "&Name", 5, 7, 30, 10
    CONTROL ADD TEXTBOX, hDlg, %IDC_TEXT1, "", 35, 5, 150, 13
    CONTROL ADD LABEL, hDlg, -1, "&Address", 5, 27, 30, 10
    CONTROL ADD TEXTBOX, hDlg, %IDC_TEXT2, "", 35, 25, 150, 13
    CONTROL ADD LABEL, hDlg, -1, "&Phone", 5, 47, 30, 10
    CONTROL ADD TEXTBOX, hDlg, %IDC_TEXT3, "", 35, 45, 150, 13
    CONTROL ADD BUTTON, hDlg, %IDOK, "Ok", 82, 85, 50, 14
    CONTROL ADD BUTTON, hDlg, %IDCANCEL, "Cancel", 136, 85, 50, 14
    DIALOG SHOW MODAL hDlg CALL DlgProc
END FUNCTION

CALLBACK FUNCTION DlgProc()
    LOCAL c AS LONG
    LOCAL sText AS STRING
    SELECT CASE AS LONG CB.MSG
        CASE %WM_INITDIALOG
        CASE %WM_COMMAND
            SELECT CASE AS LONG CB.CTL
                CASE %IDOK
                    IF CB.CTLMSG = %BN_CLICKED THEN
                        FOR c = %IDC_TEXT1 TO %IDC_TEXT3
                            CONTROL GET TEXT CB.HNDL, c TO sText
                            MSGBOX sText, %MB_TASKMODAL, "TextBox contents"
                        NEXT
                    END IF
                CASE %IDCANCEL
                    IF CB.CTLMSG = %BN_CLICKED THEN
                        DIALOG END CB.HNDL, 0
                    END IF
            END SELECT
    END SELECT
END FUNCTION

