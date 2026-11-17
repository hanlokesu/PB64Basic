#COMPILE EXE
#DIM ALL

GLOBAL g_fails AS LONG

SUB Fail(tag AS STRING)
    PRINT "FAIL: "; tag
    g_fails = g_fails + 1
END SUB

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
        LOCAL n AS LONG
        n = ThreadParam.GetANumber()
        SLEEP 300
        METHOD = n
    END METHOD
    INTERFACE MyFace
        INHERIT IPOWERTHREAD
    END INTERFACE
END CLASS

CLASS NumWorker
    INSTANCE ThreadParam AS LONG
    THREAD METHOD MAIN() AS LONG
        SLEEP 100
        METHOD = ThreadParam + 1
    END METHOD
    INTERFACE NumFace
        INHERIT IPOWERTHREAD
    END INTERFACE
END CLASS

FUNCTION PBMAIN () AS LONG
    LOCAL xx AS MyFace
    LOCAL oo AS DataFace
    LOCAL yy AS NumFace
    LOCAL alive1 AS LONG
    LOCAL result1 AS LONG
    LOCAL handle1 AS LONG
    LOCAL id1 AS LONG
    LOCAL tc AS QUAD
    LOCAL v AS LONG

    xx = CLASS "Worker"
    oo = CLASS "DataClass"

    IF xx.ISALIVE() <> 0 THEN Fail "isalive before launch"
    v = xx.SUSPEND()
    IF v <> 1 THEN Fail "suspend before launch=" + STR$(v)
    xx.STACKSIZE_SET(65536)
    v = xx.STACKSIZE_GET()
    IF v <> 65536 THEN Fail "stacksize get=" + STR$(v)
    xx.StackSize = 65536
    v = xx.StackSize
    IF v <> 65536 THEN Fail "StackSize property=" + STR$(v)
    xx.PRIORITY_SET(-1)
    v = xx.PRIORITY_GET()
    IF v <> -1 THEN Fail "priority get=" + STR$(v)
    xx.Priority = -1
    v = xx.Priority
    IF v <> -1 THEN Fail "Priority property=" + STR$(v)

    xx.LAUNCH(oo)
    alive1 = xx.ISALIVE()
    IF alive1 = 0 THEN Fail "isalive after launch"

    handle1 = xx.HANDLE()
    id1 = xx.ID()
    IF handle1 = 0 THEN Fail "handle"
    IF id1 = 0 THEN Fail "id"
    IF xx.EQUALS(xx) <> -1 THEN Fail "equals self"
    IF xx.EQUALS(oo) <> 0 THEN Fail "equals other"

    v = xx.RESUME()
    IF v <> 1 THEN Fail "resume=" + STR$(v)
    xx.JOIN(xx, 5000)

    result1 = xx.RESULT()
    IF result1 <> 77 THEN Fail "result=" + STR$(result1)
    IF xx.ISALIVE() <> 0 THEN Fail "isalive after join"

    tc = xx.TIMECREATE()
    IF tc = 0 THEN Fail "timecreate=0"
    IF xx.TIMEEXIT() = 0 THEN Fail "timeexit=0"
    v = xx.TIMEKERNEL()
    IF v < 0 THEN Fail "timekernel<0"
    v = xx.TIMEUSER()
    IF v < 0 THEN Fail "timeuser<0"

    xx.CLOSE()
    IF xx.HANDLE() <> 0 THEN Fail "handle after close"

    yy = CLASS "NumWorker"
    yy.LAUNCH(BYVAL 41)
    yy.JOIN(yy, 5000)
    IF yy.RESULT() <> 42 THEN Fail "numeric result=" + STR$(yy.RESULT())
    yy.CLOSE()

    PRINT "alive after launch ="; alive1
    PRINT "result             ="; result1
    PRINT "handle             ="; handle1
    PRINT "id                 ="; id1
    PRINT "timecreate         ="; tc
    PRINT "=== FAILURES:"; g_fails; "==="
    FUNCTION = g_fails
END FUNCTION
