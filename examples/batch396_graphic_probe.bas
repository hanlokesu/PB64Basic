' batch396_graphic_probe.bas - compile-only coverage for GRAPHIC LINE
' STEP and omitted-start forms. The SUB is never called, so no DC is needed.
#COMPILE EXE
#CONSOLE ON
#DIM ALL

SUB DrawProbe()
    GRAPHIC LINE (0, 0) - (10, 10)
    GRAPHIC LINE STEP - (20, 20)
    GRAPHIC LINE STEP (1, 1) - STEP (5, 5)
    GRAPHIC LINE - (30, 30), &HFF0000
END SUB

FUNCTION PBMAIN() AS LONG
END FUNCTION
