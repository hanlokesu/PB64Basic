' batch401_line_input.bas - compile-only probe for GRAPHIC LINE INPUT
' The statement is interactive and would block a headless run, so this
' sample is compiled but not executed by the normal example gates.
#COMPILE EXE
FUNCTION PBMAIN()
    LOCAL k AS STRING
    GRAPHIC LINE INPUT "Name: " k
    GRAPHIC LINE (10, 10) - (100, 100)
END FUNCTION
