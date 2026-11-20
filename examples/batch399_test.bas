#COMPILE EXE
#DIM ALL

' batch 399: GUI programs enter through WINMAIN, not PBMAIN.
'
' Jose Roca's WINAPI_III_107 header set contains thousands of GUI/COM
' wrappers.  Several of them use array parameters, IWinHttpRequest and
' other features this fork does not implement yet.  A small WINMAIN sample
' must not fail because of helpers it never calls: WINMAIN starts the same
' reachable-body closure as PBMAIN, and the deliberately broken SUB below
' is never compiled.

FUNCTION WINMAIN (BYVAL hInstance AS DWORD, BYVAL hPrevInstance AS DWORD, _
                  BYVAL lpCmdLine AS ASCIIZ PTR, BYVAL nCmdShow AS LONG) AS LONG
    hInstance = hInstance
    hPrevInstance = hPrevInstance
    lpCmdLine = lpCmdLine
    nCmdShow = nCmdShow
    FUNCTION = 0
END FUNCTION

SUB UnusedBrokenArrayParam (BYREF rg() AS BYTE)
    LOCAL n AS LONG
    n = UBOUND(rg) - LBOUND(rg)
END SUB
