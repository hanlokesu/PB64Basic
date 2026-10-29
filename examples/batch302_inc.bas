' batch302_inc.bas - included by batch302_meta_test.bas (#INCLUDE).
%INC_CONST = 42
FUNCTION IncAdd(a AS LONG) AS LONG
    FUNCTION = a + %INC_CONST
END FUNCTION
