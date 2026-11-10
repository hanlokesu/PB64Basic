' batch339_test.bas - #284 Error 51 (Internal error) and Error 99 (Object error) conformance
' This test verifies that Error 99 (Object error) is triggered when
' NEWCOM fails with an invalid ProgID.

FUNCTION PBMAIN()
    PRINT "Testing #284 Error 99 (Object error) conformance..."
    
    ON ERROR GOTO ErrorHandler
    
    ' Attempt to create a COM object with an invalid ProgID
    DIM obj AS IDISPATCH
    obj = NEWCOM "NonExistent.ProgID.12345"
    
    ' If we reach here, Error 99 was not triggered
    PRINT "FAIL: Error 99 was not triggered"
    PRINT "FAILURES: 1"
    EXIT FUNCTION
    
ErrorHandler:
    PRINT "Error number:"; ERR
    IF ERR = 99 THEN
        PRINT "PASS: Error 99 (Object error) triggered correctly"
        PRINT "FAILURES: 0"
    ELSE
        PRINT "FAIL: Expected Error 99, got"; ERR
        PRINT "FAILURES: 1"
    END IF
END FUNCTION
