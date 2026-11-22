#COMPILE EXE

FUNCTION PBMAIN () AS LONG
  LOCAL s AS STRING * 5
  LOCAL z AS ASCIIZ * 5
  LOCAL e AS LONG

  OPEN "get_probe.bin" FOR BINARY AS #1
  PUT$ #1, "ABCDEFGH" + CHR$(0) + "IJ"
  CLOSE #1

  OPEN "get_result.txt" FOR OUTPUT AS #2
  OPEN "get_probe.bin" FOR BINARY AS #1

  GET$ #1, 3, s
  PRINT #2, "T1_LEN="; LEN(s); " TEXT=["; s; "]"
  PRINT #2, "T1_BYTES="; ASC(s,1); ","; ASC(s,2); ","; ASC(s,3); ","; ASC(s,4); ","; ASC(s,5)

  s = SPACE$(5)
  SEEK #1, 1
  GET$ #1, 7, s
  e = ERR
  PRINT #2, "T2_LEN="; LEN(s); " TEXT=["; s; "] ERR="; e
  PRINT #2, "T2_BYTES="; ASC(s,1); ","; ASC(s,2); ","; ASC(s,3); ","; ASC(s,4); ","; ASC(s,5)

  s = SPACE$(5)
  SEEK #1, 10
  GET$ #1, 5, s
  e = ERR
  PRINT #2, "T3_LEN="; LEN(s); " TEXT=["; s; "] ERR="; e
  PRINT #2, "T3_BYTES="; ASC(s,1); ","; ASC(s,2); ","; ASC(s,3); ","; ASC(s,4); ","; ASC(s,5)

  s = SPACE$(5)
  SEEK #1, 1
  GET$ #1, 5, s
  PRINT #2, "T4_BYTES="; ASC(s,1); ","; ASC(s,2); ","; ASC(s,3); ","; ASC(s,4); ","; ASC(s,5)

  s = SPACE$(5)
  SEEK #1, 1
  GET$ #1, 0, s
  e = ERR
  PRINT #2, "T5_LEN="; LEN(s); " TEXT=["; s; "] ERR="; e

  z = SPACE$(5)
  SEEK #1, 1
  GET$ #1, 3, z
  PRINT #2, "T6_LEN="; LEN(z); " TEXT=["; z; "]"

  z = SPACE$(5)
  SEEK #1, 1
  GET$ #1, 5, z
  PRINT #2, "T7_LEN="; LEN(z); " TEXT=["; z; "]"

  z = SPACE$(5)
  SEEK #1, 1
  GET$ #1, 6, z
  PRINT #2, "T8_LEN="; LEN(z); " TEXT=["; z; "]"

  z = SPACE$(5)
  SEEK #1, 1
  GET$ #1, 0, z
  PRINT #2, "T9_LEN="; LEN(z); " TEXT=["; z; "]"

  z = SPACE$(5)
  SEEK #1, 8
  GET$ #1, 4, z
  PRINT #2, "T10_LEN="; LEN(z); " BYTES="; ASC(z,1); ","; ASC(z,2); ","; ASC(z,3); ","; ASC(z,4)

  z = SPACE$(5)
  SEEK #1, 10
  GET$ #1, 5, z
  PRINT #2, "T11_LEN="; LEN(z); " TEXT=["; z; "]"
  s = SPACE$(5)
  SEEK #1, 1
  GET$ #1, -1, s
  e = ERR
  PRINT #2, "T12_LEN="; LEN(s); " ERR="; e
  CLOSE #1
  CLOSE #2
  KILL "get_probe.bin"
END FUNCTION