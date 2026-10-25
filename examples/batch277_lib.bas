#COMPILE SLL
' batch277_lib.bas - Static Link Library source (CHW "Creating a Static
' Link Library"): compile with PB64Basic -> batch277_lib.SLL; a host
' program links it with #LINK "batch277_lib.SLL".
' COMMON descriptor (SLL sharing): accepted by fork, no behavioural
' difference (static link makes all symbols visible).

FUNCTION MyAdd (a AS LONG, b AS LONG) COMMON AS LONG
    FUNCTION = a + b
END FUNCTION

FUNCTION MyMul (a AS LONG, b AS LONG) COMMON AS LONG
    FUNCTION = a * b
END FUNCTION
