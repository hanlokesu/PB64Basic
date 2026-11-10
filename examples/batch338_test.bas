 ' batch338_test.bas - Error 7 (Out of memory) malloc NULL check
 ' Verifies that string functions still work after adding malloc_or_raise.
 ' Error 7 itself is hard to trigger (malloc rarely fails on x64 Windows),
 ' so we verify the happy path: CHR$, STRING$, SPACE$, STR$, HEX$ all work.

 #COMPILE EXE
 #DIM ALL

 FUNCTION PBMAIN()
     ' Test CHR$ range
     LOCAL s1 AS STRING
     s1 = CHR$(0 TO 255)
     IF LEN(s1) <> 256 THEN
         ? "FAIL: CHR$(0 TO 255) length = " + STR$(LEN(s1))
         WAITKEY$
         EXIT FUNCTION
     END IF

     ' Test CHR$ multi
     s1 = CHR$(65, 66, 67)
     IF s1 <> "ABC" THEN
         ? "FAIL: CHR$(65,66,67) = " + s1
         WAITKEY$
         EXIT FUNCTION
     END IF

     ' Test STRING$
     s1 = STRING$(10, 65)
     IF s1 <> "AAAAAAAAAA" THEN
         ? "FAIL: STRING$(10,65) = " + s1
         WAITKEY$
         EXIT FUNCTION
     END IF

     ' Test SPACE$
     s1 = SPACE$(5)
     IF LEN(s1) <> 5 THEN
         ? "FAIL: SPACE$(5) length = " + STR$(LEN(s1))
         WAITKEY$
         EXIT FUNCTION
     END IF

     ' Test STR$
     s1 = STR$(42)
     IF TRIM$(s1) <> "42" THEN
         ? "FAIL: STR$(42) = " + s1
         WAITKEY$
         EXIT FUNCTION
     END IF

     ' Test HEX$
     s1 = HEX$(255)
     IF s1 <> "FF" THEN
         ? "FAIL: HEX$(255) = " + s1
         WAITKEY$
         EXIT FUNCTION
     END IF

     ? "PASS: all string functions work with malloc_or_raise"
     WAITKEY$
 END FUNCTION
