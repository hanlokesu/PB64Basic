#COMPILE EXE
#DIM ALL
#IF %DEF(%PB_REVISION)
    #IF (%PB_REVISION AND &H0FF00) = &H1000
        %MY_PBVER = 10
    #ELSE
        %MY_PBVER = 0
    #ENDIF
#ELSE
    %MY_PBVER = 0
#ENDIF

' --- batch 285: CPUID via inline ASM (official PBWin Common32.bas style).
'     PBWin10 branch uses the official 32-bit form with a label; the
'     PB64Basic branch uses the label-free 64-bit form (each ! line is its
'     own LLVM inline-asm block, so branches jump to PB-level IFs instead).
'     Both write the same probe file so the outputs can be compared.
#IF %MY_PBVER = 10
FUNCTION PBMAIN() AS LONG
    LOCAL mmx AS LONG
    LOCAL sse AS LONG
    LOCAL eax_v AS LONG
    LOCAL edx_v AS LONG
    OPEN "out285cpuid.txt" FOR OUTPUT AS #1
    ! mov eax, 1
    ! cpuid
    ! mov eax_v, eax
    ! mov edx_v, edx
    ! test edx, &H00800000
    ! jz  NoMMX
    mmx = 1
NoMMX:
    ! test edx, &H02000000
    ! jz  NoSSE
    sse = 1
NoSSE:
    PRINT #1, "cpuid eax="; eax_v
    PRINT #1, "mmx="; mmx; " sse="; sse
    CLOSE #1
END FUNCTION
#ELSE
FUNCTION PBMAIN() AS LONG
    LOCAL mmx AS LONG
    LOCAL sse AS LONG
    LOCAL eax_v AS LONG
    LOCAL edx_v AS LONG
    OPEN "out285cpuid.txt" FOR OUTPUT AS #1
    ! MOV EAX, 1
    ! CPUID
    ! MOV eax_v, EAX
    ! MOV edx_v, EDX
    IF (edx_v AND &H00800000) <> 0 THEN mmx = 1
    IF (edx_v AND &H02000000) <> 0 THEN sse = 1
    PRINT #1, "cpuid eax="; eax_v
    PRINT #1, "mmx="; mmx; " sse="; sse
    CLOSE #1
END FUNCTION
#ENDIF
