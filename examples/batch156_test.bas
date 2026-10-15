#COMPILE EXE
'=====================================================================
' Batch 156 test — #RESOURCE VERSIONINFO + hard-fail diagnostics
'=====================================================================
#COMPILER PBWIN 10


' Version info embedded into the EXE
#RESOURCE VERSIONINFO
#RESOURCE FILEVERSION 10, 0, 0, 0
#RESOURCE PRODUCTVERSION 10, 0, 0, 0
#RESOURCE STRINGINFO "0409", "04B0"
#RESOURCE VERSION$ "FileDescription", "Batch 156 Test"
#RESOURCE VERSION$ "ProductName", "PB64Basic Enhanced"

FUNCTION PBMAIN () AS LONG
    MSGBOX "Batch 156: VERSIONINFO + hard-fail diagnostics test", 0, "Batch 156"
    FUNCTION = 0
END FUNCTION

