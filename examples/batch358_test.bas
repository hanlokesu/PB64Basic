#COMPILE EXE
#CONSOLE ON
' batch 358 regression example: CSET ABS/USING, wide multi-char equates
' ($$CRLF / $$DQ2 / $$SQ2 / $$QCQ / $$WHITESPACE) with LEN in code units,
' UNION/TYPE bit fields, and a BYVAL VARIANT argument that is an expression
' (the callee sees the same VT a Variant variable would carry).

UNION BitUnion
  v AS DWORD
  b0 AS BIT * 4 IN DWORD
  b1 AS BIT * 4
END UNION

TYPE BitType
  lo AS BIT * 3 IN BYTE
  hi AS BIT * 5
END TYPE

FUNCTION VariantCode (BYVAL v AS VARIANT) AS LONG
  FUNCTION = VARIANTVT(v)
END FUNCTION

FUNCTION PBMAIN () AS LONG
  LOCAL failures AS LONG
  LOCAL a AS STRING * 24
  LOCAL b AS STRING * 20
  LOCAL n AS LONG
  LOCAL n2 AS LONG
  LOCAL bu AS BitUnion
  LOCAL bt AS BitType
  LOCAL s AS STRING

  failures = 0

  a = "COOL ..PowerBASIC.. COOL"
  CSET ABS a = "PowerBASIC"
  PRINT "cset_abs=["; a; "]"
  IF a <> "COOL ..PowerBASIC.. COOL" THEN
    PRINT "FAIL cset_abs"
    failures = failures + 1
  END IF

  b = SPACE$(20)
  CSET b = "PowerBASIC" USING "*"
  PRINT "cset_using=["; b; "]"
  IF b <> "*****PowerBASIC*****" THEN
    PRINT "FAIL cset_using"
    failures = failures + 1
  END IF

  n = LEN($$CRLF)
  n2 = LEN($$WHITESPACE)
  PRINT "equate_len crlf="; n; " ws="; n2
  IF n <> 2 OR n2 <> 4 THEN
    PRINT "FAIL equate_len"
    failures = failures + 1
  END IF
  IF $$DQ2 <> CHR$$(34, 34) THEN
    PRINT "FAIL equate_value_dq2"
    failures = failures + 1
  END IF

  bu.v = &H12345678
  PRINT "union_bits="; bu.b0; bu.b1
  IF bu.b0 <> 8 OR bu.b1 <> 7 THEN
    PRINT "FAIL union_bits"
    failures = failures + 1
  END IF

  bt.lo = 5
  bt.hi = 17
  PRINT "type_bits="; bt.lo; bt.hi
  IF bt.lo <> 5 OR bt.hi <> 17 THEN
    PRINT "FAIL type_bits"
    failures = failures + 1
  END IF

  n = VariantCode(1 + 2)
  n2 = VariantCode(1.5)
  PRINT "variant_vt="; n; n2
  IF n <> 3 OR n2 <> 5 THEN
    PRINT "FAIL variant_vt"
    failures = failures + 1
  END IF

  s = "abc"
  n = VariantCode(s)
  IF n <> 8 THEN
    PRINT "FAIL variant_vt_string"
    failures = failures + 1
  END IF

  IF failures = 0 THEN
    PRINT "FAILURES: 0"
  ELSE
    PRINT "FAILURES:"; failures
  END IF
  FUNCTION = failures
END FUNCTION
