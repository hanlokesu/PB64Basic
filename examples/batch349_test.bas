#COMPILE EXE

' batch 349 self-check: front-end nesting depth.
' The release build always handled ordinary sources, but a 200-level
' parenthesised expression overflowed the default main-thread stack (0xC00000FD)
' in both the debug and the release compiler. The compiler now runs its front
' end on a 32 MB stack thread, so this example pins the nesting depth that
' used to overflow it: a 200-level add chain and a 24-level nested IF block.

DECLARE FUNCTION GetStdHandle LIB "KERNEL32.DLL" ALIAS "GetStdHandle" (BYVAL nStdHandle AS DWORD) AS LONG
DECLARE FUNCTION AttachConsole LIB "KERNEL32.DLL" ALIAS "AttachConsole" (BYVAL dwProcessId AS DWORD) AS LONG
DECLARE FUNCTION AllocConsole LIB "KERNEL32.DLL" ALIAS "AllocConsole" () AS LONG
DECLARE FUNCTION WriteFile LIB "KERNEL32.DLL" ALIAS "WriteFile" (BYVAL hFile AS LONG, lpBuffer AS ANY, BYVAL nBytesToWrite AS DWORD, lpBytesWritten AS DWORD, BYVAL lpOverlapped AS LONG) AS LONG
SUB ConPrint(BYVAL s AS STRING)
    LOCAL h AS LONG
    LOCAL n AS DWORD
    h = GetStdHandle(-11)
    IF h = 0 THEN
        IF AttachConsole(-1) = 0 THEN AllocConsole
        h = GetStdHandle(-11)
    END IF
    IF h <> 0 THEN WriteFile h, BYVAL STRPTR(s), LEN(s), n, 0
END SUB

GLOBAL g_failures AS LONG

SUB Expect(BYVAL got AS LONG, BYVAL want AS LONG, BYVAL what AS STRING)
    IF got <> want THEN
        ConPrint "FAIL " + what + " got=" + FORMAT$(got) + " want=" + FORMAT$(want)
        g_failures = g_failures + 1
    END IF
END SUB

' 200 nested parentheses: ((((1+1)+1)+1) ... ) -> 201
FUNCTION DeepAdd() AS LONG
    DeepAdd = ((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((1+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)
END FUNCTION

' 24 nested IF blocks, one increment inside each level -> 24 increments
FUNCTION DeepIf() AS LONG
    LOCAL acc AS LONG
    acc = 0
    IF 1 = 1 THEN
        acc = acc + 1
        IF 1 = 1 THEN
            acc = acc + 1
            IF 1 = 1 THEN
                acc = acc + 1
                IF 1 = 1 THEN
                    acc = acc + 1
                    IF 1 = 1 THEN
                        acc = acc + 1
                        IF 1 = 1 THEN
                            acc = acc + 1
                            IF 1 = 1 THEN
                                acc = acc + 1
                                IF 1 = 1 THEN
                                    acc = acc + 1
                                    IF 1 = 1 THEN
                                        acc = acc + 1
                                        IF 1 = 1 THEN
                                            acc = acc + 1
                                            IF 1 = 1 THEN
                                                acc = acc + 1
                                                IF 1 = 1 THEN
                                                    acc = acc + 1
                                                    IF 1 = 1 THEN
                                                        acc = acc + 1
                                                        IF 1 = 1 THEN
                                                            acc = acc + 1
                                                            IF 1 = 1 THEN
                                                                acc = acc + 1
                                                                IF 1 = 1 THEN
                                                                    acc = acc + 1
                                                                    IF 1 = 1 THEN
                                                                        acc = acc + 1
                                                                        IF 1 = 1 THEN
                                                                            acc = acc + 1
                                                                            IF 1 = 1 THEN
                                                                                acc = acc + 1
                                                                                IF 1 = 1 THEN
                                                                                    acc = acc + 1
                                                                                    IF 1 = 1 THEN
                                                                                        acc = acc + 1
                                                                                        IF 1 = 1 THEN
                                                                                            acc = acc + 1
                                                                                            IF 1 = 1 THEN
                                                                                                acc = acc + 1
                                                                                                IF 1 = 1 THEN
                                                                                                    acc = acc + 1
                                                                                                END IF
                                                                                            END IF
                                                                                        END IF
                                                                                    END IF
                                                                                END IF
                                                                            END IF
                                                                        END IF
                                                                    END IF
                                                                END IF
                                                            END IF
                                                        END IF
                                                    END IF
                                                END IF
                                            END IF
                                        END IF
                                    END IF
                                END IF
                            END IF
                        END IF
                    END IF
                END IF
            END IF
        END IF
    END IF
    DeepIf = acc
END FUNCTION

FUNCTION PBMAIN () AS LONG
    Expect DeepAdd(), 201, "deep parenthesised add chain"
    Expect DeepIf(), 24, "nested IF body"
    ConPrint "deep_add=" + FORMAT$(DeepAdd()) + " expected=201"
    ConPrint "deep_if=" + FORMAT$(DeepIf()) + " expected=24"
    ConPrint "=== FAILURES: " + FORMAT$(g_failures)
    FUNCTION = 0
END FUNCTION
