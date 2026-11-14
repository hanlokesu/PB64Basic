' Negative probe - MUST FAIL TO COMPILE.  Not part of the runnable corpus.
' Checked by the skill script check_negative_probes.py.
' Expected: non-zero exit, a diagnostic naming a THREADED array dimensioned with two shapes, no .exe produced.
THREADED ta%()
SUB ShapeA()
    DIM ta(1 TO 5)
END SUB
SUB ShapeB()
    DIM ta(1 TO 9)
END SUB
FUNCTION PBMAIN () AS LONG
    ShapeA
    ShapeB
    FUNCTION = 0
END FUNCTION
