#COMPILE EXE
#DIM ALL

GLOBAL g_fails AS LONG

SUB Fail(tag AS STRING)
    PRINT "FAIL: "; tag
    g_fails = g_fails + 1
END SUB

TYPE Payload
    member1 AS LONG
    member2 AS LONG
END TYPE

CLASS DataClass
    INTERFACE DataFace
        INHERIT DUAL
        METHOD GetANumber() AS LONG
            METHOD = 77
        END METHOD
    END INTERFACE
END CLASS

CLASS Worker
    INSTANCE ThreadParam AS DataFace
    THREAD METHOD MAIN() AS LONG
        SLEEP 150
        METHOD = ThreadParam.GetANumber()
    END METHOD
    INTERFACE MyFace
        INHERIT IPOWERTHREAD
    END INTERFACE
END CLASS

CLASS PtrWorker
    INSTANCE ThreadParam AS Payload POINTER
    THREAD METHOD MAIN() AS LONG
        SLEEP 100
        METHOD = ThreadParam.member1 + ThreadParam.member2
    END METHOD
    INTERFACE PtrFace
        INHERIT IPOWERTHREAD
    END INTERFACE
END CLASS

FUNCTION PBMAIN () AS LONG
    LOCAL xx AS MyFace
    LOCAL pp AS PtrFace
    LOCAL oo AS DataFace
    LOCAL payload AS Payload
    LOCAL v AS LONG

    xx = CLASS "Worker"
    oo = CLASS "DataClass"

    xx.STACKSIZE_SET(65536)
    v = xx.STACKSIZE_GET()
    IF v <> 65536 THEN Fail "stacksize get=" + STR$(v)
    xx.PRIORITY_SET(-1)
    v = xx.PRIORITY_GET()
    IF v <> -1 THEN Fail "priority get=" + STR$(v)

    xx.LAUNCH(oo)
    IF xx.ISALIVE() = 0 THEN Fail "isalive after launch"
    xx.JOIN(xx, 5000)
    v = xx.RESULT()
    IF v <> 77 THEN Fail "result=" + STR$(v)

    xx.CLOSE()
    IF xx.HANDLE() <> 0 THEN Fail "handle after close"

    payload.member1 = 123
    payload.member2 = 456
    pp = CLASS "PtrWorker"
    pp.LAUNCH(BYVAL VARPTR(payload))
    pp.JOIN(pp, 5000)
    v = pp.RESULT()
    IF v <> 579 THEN Fail "pointer ThreadParam=" + STR$(v)
    pp.CLOSE()

    PRINT "result             ="; v
    PRINT "=== FAILURES:"; g_fails; "==="
    FUNCTION = g_fails
END FUNCTION
