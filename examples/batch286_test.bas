' PB64Basic Enhanced - batch 286 test
' batch 286 ships the JK-IDE Compilers menu integration in the release.
' This example is the plain console smoke: after installing the zip's
' setup.bat (JK-IDE integration step), compile this file from the
' Compilers menu with PBWin 10 / PB64Basic / PBXB64.
FUNCTION PBMAIN() AS LONG
    PRINT "PB64Basic batch 286: JK-IDE Compilers menu integration."
    PRINT "Press any key to exit..."
    WAITKEY$
END FUNCTION
