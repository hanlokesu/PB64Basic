' batch296: FOR EACH / NEXT -- enumeration of a PowerCollection /
' LinkListCollection object (CHW FOR_EACH_NEXT_statements).  Lowered to the
' official enumeration pattern (CHW Enumerating_Collections): COUNT member
' call, then a 1-based ITEM(Index) loop into the loop variable.
' Fork-side probe: no PowerBASIC collection ProgID exists on this machine
' (PBWin rc=407), so the collection is a local CLASS instance supplying the
' COUNT / ITEM protocol; expected output is derived from that protocol.
#COMPILE EXE
#DIM ALL

CLASS NumberList
    INSTANCE a AS LONG
    INSTANCE b AS LONG
    INSTANCE c AS LONG
    METHOD Add3(x AS LONG, y AS LONG, z AS LONG)
        a = x
        b = y
        c = z
    END METHOD
    METHOD Count() AS LONG
        FUNCTION = 3
    END METHOD
    METHOD Item(BYVAL i AS LONG) AS LONG
        SELECT CASE i
            CASE 1: FUNCTION = a
            CASE 2: FUNCTION = b
            CASE ELSE: FUNCTION = c
        END SELECT
    END METHOD
END CLASS

FUNCTION PBMAIN() AS LONG
    LOCAL list AS NumberList
    LOCAL x AS LONG
    LOCAL s AS STRING
    list.Add3(10, 20, 30)
    FOR EACH x IN list
        s = s + STR$(x) + " "
    NEXT
    PRINT s
END FUNCTION
