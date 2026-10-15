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
#IF %MY_PBVER = 10
FUNCTION PBMAIN() AS LONG
    ' PB64Basic-only sample: PBWin10 stub (compiles, does nothing).
END FUNCTION
#ELSE

'=====================================================================
' Batch 158 test - LISTVIEW + TREEVIEW common controls
'---------------------------------------------------------------------
' Exercises all 14 statements added in batch 158:
'   CONTROL ADD LISTVIEW / CONTROL ADD TREEVIEW
'   LISTVIEW  INSERT COLUMN / INSERT ITEM / GET COUNT / GET TEXT /
'             SET TEXT / DELETE ITEM / RESET
'   TREEVIEW  INSERT ITEM / GET COUNT / GET TEXT / DELETE / RESET
'
' NOTE: the official DDT syntax addresses a control by (dialog handle,
' control id) - NOT by the control handle returned from CONTROL ADD.
'
' NOTE (batch 170): every item and column number below is 1-based
' (First = 1), which is the convention the official PowerBASIC help
' documents and the one the runtime now implements.
'=====================================================================
' #CONSOLE OFF (commented: PBWin10 rejects #CONSOLE; fork ignores it)


GLOBAL g_hDlg   AS QUAD
GLOBAL hLV      AS QUAD
GLOBAL hTV      AS QUAD
GLOBAL g_hRoot  AS QUAD
GLOBAL g_hChild AS QUAD

FUNCTION PBMAIN() AS LONG
    DIALOG NEW 0, "Batch 158: LISTVIEW + TREEVIEW", 100, 100, 340, 175 TO g_hDlg
    DIALOG CENTER g_hDlg

    CONTROL ADD LISTVIEW, g_hDlg, 201, 10, 10, 155, 95 TO hLV
    CONTROL ADD TREEVIEW, g_hDlg, 202, 175, 10, 155, 95 TO hTV

    ' --- ListView: two columns, three rows --------------------------
    LISTVIEW INSERT COLUMN g_hDlg, 201, 1, "Item", 70, 0
    LISTVIEW INSERT COLUMN g_hDlg, 201, 2, "Qty", 50, 0
    LISTVIEW INSERT ITEM g_hDlg, 201, 1, 0, "Apple"
    LISTVIEW INSERT ITEM g_hDlg, 201, 2, 0, "Banana"
    LISTVIEW INSERT ITEM g_hDlg, 201, 3, 0, "Cherry"
    LISTVIEW SET TEXT g_hDlg, 201, 1, 2, "10"
    LISTVIEW SET TEXT g_hDlg, 201, 2, 2, "20"
    LISTVIEW SET TEXT g_hDlg, 201, 3, 2, "30"

    ' --- TreeView: root + two children ------------------------------
    TREEVIEW INSERT ITEM g_hDlg, 202, 0, 0, 0, 0, "Root" TO g_hRoot
    TREEVIEW INSERT ITEM g_hDlg, 202, g_hRoot, 0, 0, 0, "Child A" TO g_hChild
    TREEVIEW INSERT ITEM g_hDlg, 202, g_hRoot, 0, 0, 0, "Child B" TO 0

    ' --- Buttons ----------------------------------------------------
    CONTROL ADD BUTTON, g_hDlg, 301, "LV Add",    10, 112, 48, 14 TO 0
    CONTROL ADD BUTTON, g_hDlg, 302, "LV Count",  62, 112, 48, 14 TO 0
    CONTROL ADD BUTTON, g_hDlg, 303, "LV GetTxt", 114, 112, 48, 14 TO 0
    CONTROL ADD BUTTON, g_hDlg, 304, "LV SetTxt", 166, 112, 48, 14 TO 0
    CONTROL ADD BUTTON, g_hDlg, 305, "LV Del",    218, 112, 48, 14 TO 0
    CONTROL ADD BUTTON, g_hDlg, 306, "LV Reset",  270, 112, 48, 14 TO 0
    CONTROL ADD BUTTON, g_hDlg, 311, "TV Add",    10, 132, 48, 14 TO 0
    CONTROL ADD BUTTON, g_hDlg, 312, "TV Count",  62, 132, 48, 14 TO 0
    CONTROL ADD BUTTON, g_hDlg, 313, "TV GetTxt", 114, 132, 48, 14 TO 0
    CONTROL ADD BUTTON, g_hDlg, 314, "TV Del",    166, 132, 48, 14 TO 0
    CONTROL ADD BUTTON, g_hDlg, 315, "TV Reset",  218, 132, 48, 14 TO 0

    DIALOG SHOW MODAL g_hDlg CALL DlgProc
    MSGBOX "Batch 158 finished. Press OK to exit.", 0, "Done"
    FUNCTION = 0
END FUNCTION

CALLBACK FUNCTION DlgProc()
    LOCAL n     AS LONG
    LOCAL s     AS STRING
    LOCAL hItem AS QUAD
    SELECT CASE CB.MSG
        CASE 2
            DIALOG END g_hDlg, 1
        CASE 273
            SELECT CASE CB.CTL
                CASE 301
                    LISTVIEW INSERT ITEM g_hDlg, 201, 4, 0, "Date"
                    LISTVIEW SET TEXT g_hDlg, 201, 4, 2, "40"
                    MSGBOX "ListView: added a 4th row."
                CASE 302
                    LISTVIEW GET COUNT g_hDlg, 201 TO n
                    MSGBOX "ListView row count = " & STR$(n)
                CASE 303
                    LISTVIEW GET TEXT g_hDlg, 201, 1, 1 TO s
                    MSGBOX "ListView cell (row1,col1) = " & s
                CASE 304
                    LISTVIEW SET TEXT g_hDlg, 201, 1, 1, "Apricot"
                    MSGBOX "ListView: row 1 col 1 set to Apricot."
                CASE 305
                    LISTVIEW DELETE ITEM g_hDlg, 201, 1
                    MSGBOX "ListView: deleted row 1."
                CASE 306
                    LISTVIEW RESET g_hDlg, 201
                    MSGBOX "ListView: cleared."
                CASE 311
                    TREEVIEW INSERT ITEM g_hDlg, 202, 0, 0, 0, 0, "New Root" TO hItem
                    MSGBOX "TreeView: added a new root node."
                CASE 312
                    TREEVIEW GET COUNT g_hDlg, 202 TO n
                    MSGBOX "TreeView node count = " & STR$(n)
                CASE 313
                    TREEVIEW GET TEXT g_hDlg, 202, g_hChild TO s
                    MSGBOX "TreeView text of Child A = " & s
                CASE 314
                    TREEVIEW DELETE g_hDlg, 202, g_hChild
                    MSGBOX "TreeView: deleted Child A."
                CASE 315
                    TREEVIEW RESET g_hDlg, 202
                    MSGBOX "TreeView: cleared."
            END SELECT
    END SELECT
END FUNCTION


#ENDIF
