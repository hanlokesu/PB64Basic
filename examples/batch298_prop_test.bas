' batch298_prop_test.bas - fork CLASS + PROPERTY GET/SET probe
#COMPILE EXE
#DIM ALL
#IF %DEF(%PB_REVISION)
    #IF (%PB_REVISION AND &H0FF00) = &H1000
        %MY_PBVER = 10
    #ELSE
        %MY_PBVER = 0
    #ENDIF
#ELSE
    %MY_PBVER = 0
#ENDIF

#IF %MY_PBVER = 10
FUNCTION PBMAIN() AS LONG
    ' PBWin stub branch (the CLASS/PROPERTY object model is fork-side).
END FUNCTION
#ELSE

CLASS Counter
    INSTANCE value AS LONG
    PROPERTY GET Count() AS LONG
        PROPERTY = value
    END PROPERTY
    PROPERTY SET Count(BYVAL v AS LONG)
        value = v
    END PROPERTY
    PROPERTY GET Indexed(BYVAL i AS LONG) AS LONG
        PROPERTY = value + i
    END PROPERTY
    PROPERTY SET Indexed(BYVAL i AS LONG, BYVAL v AS LONG)
        value = v + i
    END PROPERTY
END CLASS

FUNCTION PBMAIN() AS LONG
    LOCAL c AS Counter
    LOCAL x AS LONG
    c.Count = 42
    x = c.Count
    PRINT "x="; x
    c.Count = 7
    x = c.Count
    PRINT "x2="; x
    x = c.Indexed(3)
    PRINT "x3="; x
    c.Indexed(5) = 100
    x = c.Indexed(0)
    PRINT "x4="; x
    FUNCTION = 0
END FUNCTION

#ENDIF
