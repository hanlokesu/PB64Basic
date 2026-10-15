#COMPILE EXE
#DIM ALL
' EnterMov.bas - PB64Basic modified version



DECLARE FUNCTION GetFocus LIB "user32" ALIAS "GetFocus" () AS DWORD
DECLARE FUNCTION GetNextDlgTabItem LIB "user32" ALIAS "GetNextDlgTabItem" (BYVAL hDlg AS DWORD, BYVAL hCtl AS DWORD, BYVAL lPrevious AS LONG) AS DWORD
DECLARE FUNCTION GetKeyState LIB "user32" ALIAS "GetKeyState" (BYVAL nVirtKey AS LONG) AS INTEGER
DECLARE SUB SetFocus LIB "user32" ALIAS "SetFocus" (BYVAL hWnd AS DWORD)

%IDC_TEXT1 = 141
%IDC_TEXT2 = 142
%IDC_TEXT3 = 143
%IDC_TEXT4 = 144
%IDOK = 1
%IDCANCEL = 2
%WM_INITDIALOG = 272
%WM_COMMAND = 273
%BN_CLICKED = 0
%VK_SHIFT = 16
%MB_TASKMODAL = 8192
' $CRLF is a compiler built-in string equate (= CHR$(13,10)) - assigning to
' a built-in equate is not valid PB, so the definition is not needed here.

FUNCTION PBMAIN() AS LONG
    LOCAL hDlg AS LONG
    DIALOG NEW 0, "Input text, Press Enter or Shift+Enter",,, 191, 104, 0, 0 TO hDlg
    CONTROL ADD LABEL, hDlg, -1, "&Name", 5, 7, 30, 10
    CONTROL ADD TEXTBOX, hDlg, %IDC_TEXT1, "", 35, 5, 150, 13
    CONTROL ADD LABEL, hDlg, -1, "&Address", 5, 27, 30, 10
    CONTROL ADD TEXTBOX, hDlg, %IDC_TEXT2, "", 35, 25, 150, 13
    CONTROL ADD LABEL, hDlg, -1, "&Phone", 5, 47, 30, 10
    CONTROL ADD TEXTBOX, hDlg, %IDC_TEXT3, "", 35, 45, 150, 13
    CONTROL ADD LABEL, hDlg, -1, "&E-mail", 5, 67, 30, 10
    CONTROL ADD TEXTBOX, hDlg, %IDC_TEXT4, "", 35, 65, 150, 13
    CONTROL ADD BUTTON, hDlg, %IDOK, "Ok", 82, 85, 50, 14
    CONTROL ADD BUTTON, hDlg, %IDCANCEL, "&Cancel", 136, 85, 50, 14
    DIALOG SHOW MODAL hDlg CALL DlgProc
END FUNCTION

CALLBACK FUNCTION DlgProc()
    LOCAL c AS LONG
    LOCAL sBuf AS STRING
    LOCAL sText AS STRING
    SELECT CASE AS LONG CB.MSG
        CASE %WM_INITDIALOG
        CASE %WM_COMMAND
            SELECT CASE AS LONG CB.CTL
                CASE %IDOK
                    IF CB.CTLMSG = %BN_CLICKED THEN
                        FOR c = %IDC_TEXT1 TO %IDC_TEXT4
                            CONTROL GET TEXT CB.HNDL, c TO sBuf
                            sText = sText + sBuf + $CRLF
                        NEXT
                        MSGBOX sText, %MB_TASKMODAL, "TextBox contents"
                    END IF
                CASE %IDCANCEL
                    IF CB.CTLMSG = %BN_CLICKED THEN
                        DIALOG END CB.HNDL, 0
                    END IF
            END SELECT
    END SELECT
END FUNCTION


