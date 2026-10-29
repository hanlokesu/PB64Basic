#COMPILE EXE
#DIM ALL

' compile-only probe: DATE$/TIME$ assignment side + INPUTBOX$ arities
' NEVER RUN the resulting exe - DATE$= / TIME$= set the real system clock.
FUNCTION PBMAIN() AS LONG
    DIM s AS STRING
    DATE$ = "12/31/2025"
    TIME$ = "23:59:59"
    s = INPUTBOX$("prompt", "title", "default", 100, 200)
    s = INPUTBOX$("prompt", "title", "default")
    s = INPUTBOX$("prompt")
    END
END FUNCTION
