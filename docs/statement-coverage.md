# PB64Basic — Official Statement Coverage Matrix

> **Last updated from batch 237 (v0.2.076)** — 2026-10-14. All statements through batch 237 are reflected in this matrix.

Ground truth: **PowerBASIC official documentation** (MIT license, 735 keywords / 1282 topic pages, PB/Win 10+11 / PB/CC 6+7).

> **Status-column key**: `Implemented` = this branch generates real code. `Established` = a keyword that is *mature in the official PB documentation* - it is **NOT** a claim that upstream benstopics/powerbasilisk implemented it. `Not implemented` = an official keyword this project does not implement yet. `FORK EXTENSION` = implemented by this project but **not** an official PB keyword. This matrix does not publish a count of upstream's own keyword set.

- Rows in this matrix: **864** (10 BLOCK, 253 FUNCTION, 5 OPERATOR, 593 STATEMENT, 3 FORK EXTENSION).
- **861** official keywords currently available (✅ Implemented by this project + ✅ Established in the official PB docs).
- **0** documented keywords still not implemented; **0** Tier-3 DDT GUI items deferred; **3** fork extension (implemented here, not an official PB keyword).

## Summary

| Status | Count | Notes |
|--------|-------|-------|
| ✅ Implemented (this project) | 713 | Real codegen added by this project (Win32 calls / runtime helpers / control flow) |
| ✅ Established | 148 | Mature in the official PB documentation (not a claim about upstream's shipped set) |
| ✅ **Total available** | **861** | Implemented + Established |
| ❌ Not implemented | 0 | Official PB keywords this project does not implement yet |
| 🔧 Fork extension | 3 | Implemented here but not an official PB keyword (ARRAY SELECT, DIALOG CENTER, GRAPHIC BITMAP CAPTURE) |
| 🔲 Proposed (not yet implemented) | 0 | Documented upstream, no codegen evidence yet |
| 🛠 Tier-3 DDT (deferred) | 0 | DDT GUI / window-callback framework, high effort, deferred |

## All keywords (864 rows)

| Keyword | Kind | Platform | Status |
|---------|------|----------|--------|
| ABS | FUNCTION | LLVM intrinsic | Implemented |
| ACCEL ATTACH | STATEMENT | PB/Win only | Implemented |
| ACODE | FUNCTION | PB/Win + PB/CC | Implemented |
| ACODE$ | FUNCTION | Win32 | Implemented |
| ACOS | FUNCTION | LLVM intrinsic | Implemented |
| ACOSH | FUNCTION | LLVM intrinsic | Implemented |
| AND | OPERATOR | PB/Win + PB/CC | Established |
| ARRAY ADD | STATEMENT | PB/Win + PB/CC | Implemented |
| ARRAY ARRAYIX | STATEMENT | PB/Win + PB/CC | Implemented |
| ARRAY ASSIGN | STATEMENT | PB/Win + PB/CC | Implemented |
| ARRAY COPY | STATEMENT | PB/Win + PB/CC | Implemented |
| ARRAY DELETE | STATEMENT | PB/Win + PB/CC | Implemented |
| ARRAY INSERT | STATEMENT | PB/Win + PB/CC | Implemented |
| ARRAY REDIM DECR | STATEMENT | PB/Win + PB/CC | Implemented |
| ARRAY REDIM INCR | STATEMENT | PB/Win + PB/CC | Implemented |
| ARRAY REVERSE | STATEMENT | PB/Win + PB/CC | Implemented |
| ARRAY SCAN | STATEMENT | PB/Win + PB/CC | Implemented |
| ARRAY SELECT | FORK EXTENSION | PB/Win + PB/CC | Implemented |
| ARRAY SHUFFLE | STATEMENT | PB/Win + PB/CC | Implemented |
| ARRAY SORT | STATEMENT | PB/Win + PB/CC | Established |
| ARRAY SWAP | STATEMENT | PB/Win + PB/CC | Implemented |
| ARRAY TAGARRAY | STATEMENT | PB/Win + PB/CC | Implemented |
| ARRAY TAGARRAY ERASE | STATEMENT | PB/Win + PB/CC | Implemented |
| ARRAY UNIQUE | STATEMENT | PB/Win + PB/CC | Implemented |
| ASC | STATEMENT | PB/Win + PB/CC | Established |
| ASIN | FUNCTION | LLVM intrinsic | Implemented |
| ASINH | FUNCTION | LLVM intrinsic | Implemented |
| ASM | STATEMENT | PB/Win + PB/CC | Established |
| ASMDATA / END ASMDATA | BLOCK | PB/Win + PB/CC | Established |
| ATANH | FUNCTION | LLVM intrinsic | Implemented |
| ATN | FUNCTION | LLVM intrinsic | Implemented |
| ATN2 | FUNCTION | codegen builtin | Implemented |
| BEEP | STATEMENT | PB/Win + PB/CC | Established |
| BGR | FUNCTION | codegen builtin | Implemented |
| BIN | FUNCTION | codegen builtin | Implemented |
| BIN$ | FUNCTION | Win32 | Implemented |
| BIT | STATEMENT | PB/Win + PB/CC | Established |
| BIT CALC | STATEMENT | PB/Win + PB/CC | Established |
| BITS | FUNCTION | codegen builtin | Implemented |
| BITS$ | FUNCTION | Win32 | Implemented |
| BITSE | FUNCTION | codegen builtin (lvalue test-and-set) | Implemented |
| BUILD | FUNCTION | codegen builtin | Implemented |
| BUILD$ | FUNCTION | Win32 | Implemented |
| BYTE | FUNCTION | codegen builtin | Implemented |
| CALL | STATEMENT | PB/Win + PB/CC | Established |
| CALL DWORD | STATEMENT | PB/Win + PB/CC | Established |
| CALLSTK | STATEMENT | PB/Win + PB/CC | Established |
| CALLSTK$ | FUNCTION | Win32 | Implemented |
| CALLSTKCOUNT | FUNCTION | codegen builtin | Implemented |
| CBOOL | FUNCTION | codegen builtin | Implemented |
| CBRT | FUNCTION | C runtime (MSVCRT) | Implemented |
| CBYTE | FUNCTION | codegen builtin | Implemented |
| CDBL | FUNCTION | codegen builtin | Implemented |
| CDWORD | FUNCTION | codegen builtin | Implemented |
| CEIL | FUNCTION | LLVM intrinsic | Implemented |
| CFLT | FUNCTION | codegen builtin | Implemented |
| CHDIR | STATEMENT | PB/Win + PB/CC | Established |
| CHDRIVE | STATEMENT | PB/Win + PB/CC | Established |
| CHOOSE | FUNCTION | codegen builtin | Implemented |
| CHR | FUNCTION | codegen builtin | Implemented |
| CHR$ | FUNCTION | Win32 | Implemented |
| CHRBYTES | FUNCTION | codegen builtin | Implemented |
| CHRTOOEM | FUNCTION | codegen builtin | Implemented |
| CHRTOOEM$ | FUNCTION | Win32 | Implemented |
| CHRTOUTF8 | FUNCTION | codegen builtin | Implemented |
| CHRTOUTF8$ | FUNCTION | Win32 | Implemented |
| CLASS/END CLASS | BLOCK | PB/Win + PB/CC | Established |
| CLIP | FUNCTION | codegen builtin | Implemented |
| CLIP$ | FUNCTION | Win32 | Implemented |
| CLIPBOARD | STATEMENT | PB/Win + PB/CC | Established |
| CLOSE | STATEMENT | PB/Win + PB/CC | Implemented |
| CLS | STATEMENT | PB/CC only | Established |
| CODEPTR | FUNCTION | codegen builtin | Implemented |
| COLOR | STATEMENT | PB/CC only | Established |
| COMBOBOX ADD | STATEMENT | PB/Win only | Implemented |
| COMBOBOX DELETE | STATEMENT | PB/Win only | Implemented |
| COMBOBOX FIND | STATEMENT | PB/Win only | Implemented |
| COMBOBOX FIND EXACT | STATEMENT | PB/Win only | Implemented |
| COMBOBOX GET COUNT | STATEMENT | PB/Win only | Implemented |
| COMBOBOX GET SELCOUNT | STATEMENT | PB/Win only | Implemented |
| COMBOBOX GET SELECT | STATEMENT | PB/Win only | Implemented |
| COMBOBOX GET STATE | STATEMENT | PB/Win only | Implemented |
| COMBOBOX GET TEXT | STATEMENT | PB/Win only | Implemented |
| COMBOBOX GET USER | STATEMENT | PB/Win only | Implemented |
| COMBOBOX INSERT | STATEMENT | PB/Win only | Implemented |
| COMBOBOX RESET | STATEMENT | PB/Win only | Implemented |
| COMBOBOX SELECT | STATEMENT | PB/Win only | Implemented |
| COMBOBOX SET TEXT | STATEMENT | PB/Win only | Implemented |
| COMBOBOX SET USER | STATEMENT | PB/Win only | Implemented |
| COMBOBOX UNSELECT | STATEMENT | PB/Win only | Implemented |
| COMM CLOSE | STATEMENT | PB/Win + PB/CC | Established |
| COMM LINE | STATEMENT | PB/Win + PB/CC | Established |
| COMM LINE INPUT | STATEMENT | PB/Win | Implemented |
| COMM OPEN | STATEMENT | PB/Win + PB/CC | Established |
| COMM PRINT | STATEMENT | PB/Win + PB/CC | Established |
| COMM RECV | STATEMENT | PB/Win + PB/CC | Established |
| COMM RESET | STATEMENT | PB/Win + PB/CC | Established |
| COMM SEND | STATEMENT | PB/Win + PB/CC | Established |
| COMM SET | STATEMENT | PB/Win + PB/CC | Established |
| COMM TIMEOUT | STATEMENT | PB/Win + PB/CC | Established |
| COMMAND | FUNCTION | codegen builtin | Implemented |
| COMMAND$ | FUNCTION | Win32 | Implemented |
| CONTROL ADD | STATEMENT | PB/Win only | Implemented |
| CONTROL ADD ANIMATE | STATEMENT | PB/Win only | Implemented |
| CONTROL ADD BUTTON | STATEMENT | PB/Win only | Implemented |
| CONTROL ADD CHECK3STATE | STATEMENT | PB/Win only | Implemented |
| CONTROL ADD CHECKBOX | STATEMENT | PB/Win only | Implemented |
| CONTROL ADD COMBOBOX | STATEMENT | PB/Win only | Implemented |
| CONTROL ADD FRAME | STATEMENT | PB/Win only | Implemented |
| CONTROL ADD GRAPHIC | STATEMENT | PB/Win only | Implemented |
| CONTROL ADD HEADER | STATEMENT | PB/Win only | Implemented |
| CONTROL ADD HOTKEY | STATEMENT | PB/Win only | Implemented |
| CONTROL ADD IMAGE | STATEMENT | PB/Win only | Implemented |
| CONTROL ADD IMAGEX | STATEMENT | PB/Win only | Implemented |
| CONTROL ADD IMGBUTTON | STATEMENT | PB/Win only | Implemented |
| CONTROL ADD IMGBUTTONX | STATEMENT | PB/Win only | Implemented |
| CONTROL ADD IPADDRESS | STATEMENT | PB/Win only | Implemented |
| CONTROL ADD LABEL | STATEMENT | PB/Win only | Implemented |
| CONTROL ADD LINE | STATEMENT | PB/Win only | Implemented |
| CONTROL ADD LISTBOX | STATEMENT | PB/Win only | Implemented |
| CONTROL ADD LISTVIEW | STATEMENT | PB/Win only | Implemented |
| CONTROL ADD MONTHCAL | STATEMENT | PB/Win only | Implemented |
| CONTROL ADD OPTION | STATEMENT | PB/Win only | Implemented |
| CONTROL ADD PROGRESSBAR | STATEMENT | PB/Win only | Implemented |
| CONTROL ADD RICHEDIT | STATEMENT | PB/Win only | Implemented |
| CONTROL ADD SCROLLBAR | STATEMENT | PB/Win only | Implemented |
| CONTROL ADD STATUSBAR | STATEMENT | PB/Win only | Implemented |
| CONTROL ADD TAB | STATEMENT | PB/Win only | Implemented |
| CONTROL ADD TEXTBOX | STATEMENT | PB/Win only | Implemented |
| CONTROL ADD TOOLBAR | STATEMENT | PB/Win only | Implemented |
| CONTROL ADD TRACKBAR | STATEMENT | PB/Win only | Implemented |
| CONTROL ADD TREEVIEW | STATEMENT | PB/Win only | Implemented |
| CONTROL ADD UPDOWN | STATEMENT | PB/Win only | Implemented |
| CONTROL DISABLE | STATEMENT | PB/Win only | Implemented
| CONTROL ENABLE | STATEMENT | PB/Win only | Implemented
| CONTROL GET CHECK | STATEMENT | PB/Win only | Implemented
| CONTROL GET CLIENT | STATEMENT | PB/Win only | Implemented |
| CONTROL GET LOC | STATEMENT | PB/Win only | Implemented |
| CONTROL GET SIZE | STATEMENT | PB/Win only | Implemented |
| CONTROL GET TEXT | STATEMENT | PB/Win only | Implemented
| CONTROL GET USER | STATEMENT | PB/Win only | Implemented |
| CONTROL HANDLE | STATEMENT | PB/Win only | Implemented |
| CONTROL HIDE | STATEMENT | PB/Win only | Implemented
| CONTROL KILL | STATEMENT | PB/Win only | Implemented
| CONTROL NORMALIZE | STATEMENT | PB/Win only | Implemented |
| CONTROL POST | STATEMENT | PB/Win only | Implemented |
| CONTROL REDRAW | STATEMENT | PB/Win only | Implemented |
| CONTROL SEND | STATEMENT | PB/Win only | Implemented |
| CONTROL SET CHECK | STATEMENT | PB/Win only | Implemented
| CONTROL SET CLIENT | STATEMENT | PB/Win only | Implemented |
| CONTROL SET COLOR | STATEMENT | PB/Win only | Implemented |
| CONTROL SET FOCUS | STATEMENT | PB/Win only | Implemented |
| CONTROL SET FONT | STATEMENT | PB/Win only | Implemented |
| CONTROL SET IMAGE | STATEMENT | PB/Win only | Implemented |
| CONTROL SET IMAGEX | STATEMENT | PB/Win only | Implemented |
| CONTROL SET IMGBUTTON | STATEMENT | PB/Win only | Implemented |
| CONTROL SET IMGBUTTONX | STATEMENT | PB/Win only | Implemented |
| CONTROL SET LOC | STATEMENT | PB/Win only | Implemented |
| CONTROL SET OPTION | STATEMENT | PB/Win only | Implemented |
| CONTROL SET SIZE | STATEMENT | PB/Win only | Implemented |
| CONTROL SET TEXT | STATEMENT | PB/Win only | Implemented
| CONTROL SET USER | STATEMENT | PB/Win only | Implemented |
| CONTROL SHOW STATE | STATEMENT | PB/Win only | Implemented |
| COS | FUNCTION | LLVM intrinsic | Implemented |
| COSH | FUNCTION | LLVM intrinsic | Implemented |
| COT | FUNCTION | C runtime (MSVCRT) | Implemented |
| COTH | FUNCTION | C runtime (MSVCRT) | Implemented |
| CQUAD | FUNCTION | codegen builtin | Implemented |
| CSC | FUNCTION | C runtime (MSVCRT) | Implemented |
| CSCH | FUNCTION | C runtime (MSVCRT) | Implemented |
| CSET | STATEMENT | PB/Win + PB/CC | Established |
| CSET$ | FUNCTION | Win32 | Implemented |
| CSNG | FUNCTION | codegen builtin | Implemented |
| CSTR | FUNCTION | codegen builtin | Implemented |
| CULNG | FUNCTION | codegen builtin | Implemented |
| CURDIR | FUNCTION | codegen builtin | Implemented |
| CURDIR$ | FUNCTION | Win32 | Implemented |
| CVBYT | FUNCTION | codegen builtin | Implemented |
| CVCUX | FUNCTION | codegen builtin | Implemented |
| CVDWD | FUNCTION | codegen builtin | Implemented |
| CVL | FUNCTION | codegen builtin | Implemented |
| CVQ | FUNCTION | codegen builtin | Implemented |
| CVW | FUNCTION | codegen builtin | Implemented |
| CWORD | FUNCTION | codegen builtin | Implemented |
| DATA | STATEMENT | PB/Win + PB/CC | Established |
| DATACOUNT | FUNCTION | codegen builtin | Implemented |
| DAYNAME | FUNCTION | codegen builtin | Implemented |
| DAYNAME$ | FUNCTION | Win32 | Implemented |
| DEC | FUNCTION | codegen builtin | Implemented |
| DEC$ | FUNCTION | Win32 | Implemented |
| DECLARE | STATEMENT | PB/Win + PB/CC | Implemented |
| DECR | STATEMENT | PB/Win + PB/CC | Established |
| DEF | OPERATOR | PB/Win + PB/CC | Implemented |
| DESKTOP GET CLIENT | STATEMENT | PB/Win + PB/CC | Established |
| DESKTOP GET LOC | STATEMENT | PB/Win + PB/CC | Established |
| DESKTOP GET PPI | STATEMENT | PB/Win + PB/CC | Implemented |
| DESKTOP GET SIZE | STATEMENT | PB/Win + PB/CC | Established |
| DIALOG CENTER | FORK EXTENSION | PB/Win only | Implemented |
| DIALOG DEFAULT FONT | STATEMENT | PB/Win only | Implemented |
| DIALOG DISABLE | STATEMENT | PB/Win only | Implemented |
| DIALOG DOEVENTS | STATEMENT | PB/Win only | Implemented
| DIALOG ENABLE | STATEMENT | PB/Win only | Implemented |
| DIALOG END | STATEMENT | PB/Win only | Implemented
| DIALOG GET CLIENT | STATEMENT | PB/Win only | Implemented |
| DIALOG GET LOC | STATEMENT | PB/Win only | Implemented |
| DIALOG GET SIZE | STATEMENT | PB/Win only | Implemented |
| DIALOG GET TEXT | STATEMENT | PB/Win only | Implemented
| DIALOG GET USER | STATEMENT | PB/Win only | Implemented |
| DIALOG HIDE | STATEMENT | PB/Win only | Implemented |
| DIALOG MAXIMIZE | STATEMENT | PB/Win only | Implemented |
| DIALOG MINIMIZE | STATEMENT | PB/Win only | Implemented |
| DIALOG NEW | STATEMENT | PB/Win only | Implemented
| DIALOG NONSTABLE | STATEMENT | PB/Win only | Implemented |
| DIALOG NORMALIZE | STATEMENT | PB/Win only | Implemented |
| DIALOG PIXELS | STATEMENT | PB/Win only | Implemented |
| DIALOG POST | STATEMENT | PB/Win only | Implemented |
| DIALOG REDRAW | STATEMENT | PB/Win only | Implemented |
| DIALOG SEND | STATEMENT | PB/Win only | Implemented |
| DIALOG SET CLIENT | STATEMENT | PB/Win only | Implemented |
| DIALOG SET COLOR | STATEMENT | PB/Win only | Implemented |
| DIALOG SET ICON | STATEMENT | PB/Win only | Implemented |
| DIALOG SET LOC | STATEMENT | PB/Win only | Implemented |
| DIALOG SET SIZE | STATEMENT | PB/Win only | Implemented |
| DIALOG SET TEXT | STATEMENT | PB/Win only | Implemented
| DIALOG SET USER | STATEMENT | PB/Win only | Implemented |
| DIALOG SHOW MODAL | STATEMENT | PB/Win only | Implemented
| DIALOG SHOW MODELESS | STATEMENT | PB/Win only | Implemented |
| DIALOG SHOW STATE | STATEMENT | PB/Win only | Implemented
| DIALOG STABILIZE | STATEMENT | PB/Win only | Implemented |
| DIALOG UNITS | STATEMENT | PB/Win only | Implemented |
| DIM | STATEMENT | PB/Win + PB/CC | Implemented |
| DIR | FUNCTION | codegen builtin | Implemented |
| DIR CLOSE | STATEMENT | PB/Win + PB/CC | Implemented |
| DIR$ | FUNCTION | PB/Win + PB/CC | Implemented |
| DISKFREE | FUNCTION | codegen builtin | Implemented |
| DISKSIZE | FUNCTION | codegen builtin | Implemented |
| DISPLAY BROWSE | STATEMENT | PB/Win only | Implemented |
| DISPLAY COLOR | STATEMENT | PB/Win only | Implemented |
| DISPLAY FONT | STATEMENT | PB/Win only | Implemented |
| DISPLAY OPENFILE | STATEMENT | PB/Win only | Implemented |
| DISPLAY SAVEFILE | STATEMENT | PB/Win only | Implemented |
| DOUBLE | FUNCTION | codegen builtin | Implemented |
| END | STATEMENT | PB/Win + PB/CC | Established |
| ENVIRON | STATEMENT | PB/Win + PB/CC | Established |
| ENVIRON$ | FUNCTION | Win32 | Implemented |
| EOF | FUNCTION | codegen builtin | Implemented |
| EQV | FUNCTION | PB/Win + PB/CC | Implemented |
| ERASE | STATEMENT | PB/Win + PB/CC | Established |
| ERF | FUNCTION | C runtime (MSVCRT) | Implemented |
| ERL | FUNCTION | codegen builtin | Implemented |
| ERL$ | FUNCTION | Win32 | Implemented |
| ERR | FUNCTION | codegen builtin | Implemented |
| ERRCLEAR | FUNCTION | codegen builtin | Implemented |
| ERROR | STATEMENT | PB/Win + PB/CC | Established |
| ERROR$ | FUNCTION | Win32 | Implemented |
| EVENT SOURCE | STATEMENT | PB/Win + PB/CC | Implemented |
| EVENTS | STATEMENT | PB/Win + PB/CC | Implemented |
| EXE | FUNCTION | codegen builtin | Implemented |
| EXIST | FUNCTION | codegen builtin | Implemented |
| EXIT | STATEMENT | PB/Win + PB/CC | Established |
| EXP | FUNCTION | LLVM intrinsic | Implemented |
| EXP10 | FUNCTION | LLVM intrinsic | Implemented |
| EXP2 | FUNCTION | LLVM intrinsic | Implemented |
| EXPM1 | FUNCTION | C runtime (MSVCRT) | Implemented |
| EXTRACT | FUNCTION | codegen builtin | Implemented |
| EXTRACT$ | FUNCTION | Win32 | Implemented |
| FIELD | STATEMENT | PB/Win + PB/CC | Established |
| FILEATTR | FUNCTION | codegen builtin | Implemented |
| FILECOPY | STATEMENT | PB/Win + PB/CC | Established |
| FILENAME | FUNCTION | codegen builtin | Implemented |
| FILENAME$ | FUNCTION | Win32 | Implemented |
| FILESCAN | STATEMENT | PB/Win + PB/CC | Established |
| FIX | FUNCTION | codegen builtin | Implemented |
| FLOOR | FUNCTION | LLVM intrinsic | Implemented |
| FLUSH | STATEMENT | PB/Win + PB/CC | Established |
| FONT END | STATEMENT | PB/Win + PB/CC | Implemented |
| FONT NEW | STATEMENT | PB/Win + PB/CC | Implemented |
| FOR / NEXT | STATEMENT | PB/Win + PB/CC | Established |
| FORMAT | FUNCTION | codegen builtin | Implemented |
| FORMAT$ | FUNCTION | Win32 | Implemented |
| FRAC | FUNCTION | codegen builtin | Implemented |
| FRE | FUNCTION | codegen builtin | Implemented |
| FREEFILE | FUNCTION | codegen builtin | Implemented |
| FUNCNAME | FUNCTION | PB/Win + PB/CC | Implemented |
| FUNCNAME$ | FUNCTION | codegen builtin (current_fn_name) | Implemented |
| FUNCTION / END FUNCTION | STATEMENT | PB/Win + PB/CC | Established |
| GET | STATEMENT | PB/Win + PB/CC | Implemented |
| GET$ | STATEMENT | PB/Win + PB/CC | Implemented |
| GET$$ | STATEMENT | PB/Win + PB/CC | Implemented |
| GETATTR | FUNCTION | codegen builtin | Implemented |
| GET_STR | FUNCTION | codegen builtin | Implemented |
| GET_WSTR | FUNCTION | codegen builtin | Implemented |
| GLOBAL | STATEMENT | PB/Win + PB/CC | Established |
| GLOBALMEM | STATEMENT | PB/Win + PB/CC | Established |
| GRAPHIC ARC | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC ATTACH | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC BITMAP CAPTURE | FORK EXTENSION | PB/Win + PB/CC | Implemented |
| GRAPHIC BITMAP END | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC BITMAP LOAD | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC BITMAP NEW | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC BOX | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC CELL | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC CELL SIZE | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC CHR SIZE | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC CLEAR | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC COLOR | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC COPY | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC DETACH | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC ELLIPSE | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC GET BITS | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC GET CANVAS | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC GET CAPTION | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC GET CLIENT | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC GET CLIP | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC GET DC | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC GET LINES | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC GET LOC | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC GET MIX | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC GET OVERLAP | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC GET PIXEL | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC GET POS | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC GET PPI | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC GET SCALE | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC GET SCROLLTEXT | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC GET SIZE | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC GET STRETCHMODE | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC GET TEXTALIGN | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC GET VIEW | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC GET WORDWRAP | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC GET WRAP | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC IMAGELIST | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC INKEY$ | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC INPUT | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC INPUT FLUSH | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC INSTAT | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC LINE | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC LINE INPUT | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC PAINT | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC PIE | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC POLYGON | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC POLYLINE | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC PRINT | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC REDRAW | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC RENDER | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC SAVE | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC SCALE | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC SET AUTOSIZE | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC SET BITS | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC SET CAPTION | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC SET CLIENT | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC SET CLIP | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC SET FIXED | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC SET FOCUS | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC SET FONT | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC SET LOC | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC SET MIX | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC SET OVERLAP | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC SET PIXEL | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC SET POS | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC SET SCROLLTEXT | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC SET SIZE | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC SET STRETCHMODE | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC SET TEXTALIGN | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC SET VIEW | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC SET VIRTUAL | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC SET WORDWRAP | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC SET WRAP | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC SPLIT | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC STRETCH | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC STYLE | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC TEXT SIZE | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC WAITKEY$ | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC WIDTH | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC WINDOW | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC WINDOW CLICK | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC WINDOW END | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC WINDOW HIDE | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC WINDOW MINIMIZE | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC WINDOW NONSTABLE | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC WINDOW NORMALIZE | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC WINDOW STABILIZE | STATEMENT | PB/Win + PB/CC | Implemented |
| GRAPHIC_CIRCLE | FUNCTION | codegen builtin | Implemented |
| GRAPHIC_SCALE_PIXELS | FUNCTION | codegen builtin | Implemented |
| HEADER GET COUNT | STATEMENT | PB/Win only | Implemented |
| HEADER GET ITEM | STATEMENT | PB/Win only | Implemented |
| HEADER SEND | STATEMENT | PB/Win only | Implemented |
| HEADER SET ITEM | STATEMENT | PB/Win only | Implemented |
| HEX | FUNCTION | codegen builtin | Implemented |
| HEX$ | FUNCTION | Win32 | Implemented |
| HI | FUNCTION | Win32 | Implemented |
| HIWRD | FUNCTION | codegen builtin | Implemented |
| HOST ADDR | STATEMENT | PB/Win + PB/CC | Established |
| HOST NAME | STATEMENT | PB/Win + PB/CC | Established |
| HYPOT | FUNCTION | C runtime (MSVCRT) | Implemented |
| IF | STATEMENT | PB/Win + PB/CC | Established |
| IF/END IF | BLOCK | PB/Win + PB/CC | Established |
| IIF | FUNCTION | codegen builtin | Implemented |
| IMAGELIST ADD BITMAP | STATEMENT | PB/Win only | Implemented |
| IMAGELIST ADD ICON | STATEMENT | PB/Win only | Implemented |
| IMAGELIST ADD MASKED | STATEMENT | PB/Win only | Implemented |
| IMAGELIST GET COUNT | STATEMENT | PB/Win only | Implemented |
| IMAGELIST KILL | STATEMENT | PB/Win only | Implemented |
| IMAGELIST NEW BITMAP | STATEMENT | PB/Win only | Implemented |
| IMAGELIST NEW ICON | STATEMENT | PB/Win only | Implemented |
| IMAGELIST SET OVERLAY | STATEMENT | PB/Win only | Implemented |
| IMAGELIST_COUNT | FUNCTION | codegen builtin | Implemented |
| IMAGELIST_NEW | FUNCTION | codegen builtin | Implemented |
| IMP | FUNCTION | PB/Win + PB/CC | Implemented |
| IMPORT | STATEMENT | PB/Win + PB/CC | Established |
| INCR | STATEMENT | PB/Win + PB/CC | Established |
| INPUT FLUSH | STATEMENT | PB/CC only | Established |
| INPUT# | STATEMENT | PB/CC only | Implemented |
| INSTANCE | STATEMENT | PB/Win + PB/CC | Implemented |
| INSTR | FUNCTION | codegen builtin | Implemented |
| INT | FUNCTION | codegen builtin | Implemented |
| INTEGER | FUNCTION | codegen builtin | Implemented |
| INTERFACE / END INTERFACE (DIRECT) | BLOCK | PB/Win + PB/CC | Established |
| INTERFACE/END INTERFACE (IDBIND) | BLOCK | PB/Win + PB/CC | Established |
| ISEVEN | FUNCTION | codegen builtin | Implemented |
| ISFALSE | FUNCTION | codegen builtin | Implemented |
| ISFILE | FUNCTION | codegen builtin | Implemented |
| ISFOLDER | FUNCTION | codegen builtin | Implemented |
| ISINFINITE | STATEMENT | PB/Win + PB/CC | Implemented |
| ISNORMAL | STATEMENT | PB/Win + PB/CC | Implemented |
| ISODD | FUNCTION | codegen builtin | Implemented |
| ISTRUE | FUNCTION | codegen builtin | Implemented |
| ISWIN | FUNCTION | Win32 (IsWindow/GetDlgItem) | Implemented |
| ITERATE | STATEMENT | PB/Win + PB/CC | Established |
| JOIN$ | FUNCTION | Win32 | Implemented |
| KILL | STATEMENT | PB/Win + PB/CC | Established |
| LCASE | FUNCTION | codegen builtin | Implemented |
| LCASE$ | FUNCTION | Win32 | Implemented |
| LEFT | FUNCTION | codegen builtin | Implemented |
| LEFT$ | FUNCTION | Win32 | Implemented |
| LEN | FUNCTION | codegen builtin | Implemented |
| LET | STATEMENT | PB/Win + PB/CC | Established |
| LET *(WITH OBJECTS)* | STATEMENT | PB/Win + PB/CC | Implemented |
| LET *(WITH TYPES)* | STATEMENT | PB/Win + PB/CC | Established |
| LET *(WITH VARIANTS)* | STATEMENT | PB/Win + PB/CC | Implemented |
| LINE INPUT# | STATEMENT | PB/Win + PB/CC | Implemented |
| LISTBOX ADD | STATEMENT | PB/Win only | Implemented |
| LISTBOX DELETE | STATEMENT | PB/Win only | Implemented |
| LISTBOX FIND | STATEMENT | PB/Win only | Implemented |
| LISTBOX FIND EXACT | STATEMENT | PB/Win only | Implemented |
| LISTBOX GET COUNT | STATEMENT | PB/Win only | Implemented |
| LISTBOX GET SELCOUNT | STATEMENT | PB/Win only | Implemented |
| LISTBOX GET SELECT | STATEMENT | PB/Win only | Implemented |
| LISTBOX GET STATE | STATEMENT | PB/Win only | Implemented |
| LISTBOX GET TEXT | STATEMENT | PB/Win only | Implemented |
| LISTBOX GET USER | STATEMENT | PB/Win only | Implemented |
| LISTBOX INSERT | STATEMENT | PB/Win only | Implemented |
| LISTBOX RESET | STATEMENT | PB/Win only | Implemented |
| LISTBOX SELECT | STATEMENT | PB/Win only | Implemented |
| LISTBOX SET TEXT | STATEMENT | PB/Win only | Implemented |
| LISTBOX SET USER | STATEMENT | PB/Win only | Implemented |
| LISTBOX UNSELECT | STATEMENT | PB/Win only | Implemented |
| LISTVIEW DELETE COLUMN | STATEMENT | PB/Win only | Implemented |
| LISTVIEW DELETE ITEM | STATEMENT | PB/Win only | Implemented |
| LISTVIEW FIND | STATEMENT | PB/Win only | Implemented |
| LISTVIEW FIND EXACT | STATEMENT | PB/Win only | Implemented |
| LISTVIEW FIT CONTENT | STATEMENT | PB/Win only | Implemented |
| LISTVIEW FIT HEADER | STATEMENT | PB/Win only | Implemented |
| LISTVIEW GET COLUMN | STATEMENT | PB/Win only | Implemented |
| LISTVIEW GET COUNT | STATEMENT | PB/Win only | Implemented |
| LISTVIEW GET HEADER | STATEMENT | PB/Win only | Implemented |
| LISTVIEW GET HEADERID | STATEMENT | PB/Win only | Implemented |
| LISTVIEW GET MODE | STATEMENT | PB/Win only | Implemented |
| LISTVIEW GET SELCOUNT | STATEMENT | PB/Win only | Implemented |
| LISTVIEW GET SELECT | STATEMENT | PB/Win only | Implemented |
| LISTVIEW GET STATE | STATEMENT | PB/Win only | Implemented |
| LISTVIEW GET STYLEXX | STATEMENT | PB/Win only | Implemented |
| LISTVIEW GET TEXT | STATEMENT | PB/Win only | Implemented |
| LISTVIEW GET USER | STATEMENT | PB/Win only | Implemented |
| LISTVIEW INSERT COLUMN | STATEMENT | PB/Win only | Implemented |
| LISTVIEW INSERT ITEM | STATEMENT | PB/Win only | Implemented |
| LISTVIEW RESET | STATEMENT | PB/Win only | Implemented |
| LISTVIEW SELECT | STATEMENT | PB/Win only | Implemented |
| LISTVIEW SET COLUMN | STATEMENT | PB/Win only | Implemented |
| LISTVIEW SET HEADER | STATEMENT | PB/Win only | Implemented |
| LISTVIEW SET IMAGE | STATEMENT | PB/Win only | Implemented |
| LISTVIEW SET IMAGE2 | STATEMENT | PB/Win only | Implemented |
| LISTVIEW SET IMAGELIST | STATEMENT | PB/Win only | Implemented |
| LISTVIEW SET MODE | STATEMENT | PB/Win only | Implemented |
| LISTVIEW SET OVERLAY | STATEMENT | PB/Win only | Implemented |
| LISTVIEW SET STYLEXX | STATEMENT | PB/Win only | Implemented |
| LISTVIEW SET TEXT | STATEMENT | PB/Win only | Implemented |
| LISTVIEW SET USER | STATEMENT | PB/Win only | Implemented |
| LISTVIEW SORT | STATEMENT | PB/Win only | Implemented |
| LISTVIEW UNSELECT | STATEMENT | PB/Win only | Implemented |
| LISTVIEW VISIBLE | STATEMENT | PB/Win only | Implemented |
| LO | FUNCTION | codegen builtin | Implemented |
| LOC | FUNCTION | Win32 | Implemented |
| LOCAL | STATEMENT | PB/Win + PB/CC | Established |
| LOCK | STATEMENT | PB/Win + PB/CC | Established |
| LOF | FUNCTION | Win32 | Implemented |
| LOG | FUNCTION | LLVM intrinsic | Implemented |
| LOG10 | FUNCTION | LLVM intrinsic | Implemented |
| LOG1P | FUNCTION | C runtime (MSVCRT) | Implemented |
| LOG2 | FUNCTION | LLVM intrinsic | Implemented |
| LONG | FUNCTION | codegen builtin | Implemented |
| LOWRD | FUNCTION | codegen builtin | Implemented |
| LPRINT | STATEMENT | PB/Win + PB/CC | Established |
| LPRINT ATTACH | STATEMENT | PB/Win + PB/CC | Established |
| LPRINT CLOSE | STATEMENT | PB/Win + PB/CC | Established |
| LPRINT FLUSH | STATEMENT | PB/Win + PB/CC | Established |
| LPRINT FORMFEED | STATEMENT | PB/Win + PB/CC | Established |
| LPRINT$ | FUNCTION | Win32 | Implemented |
| LSET | STATEMENT | PB/Win + PB/CC | Established |
| LSET$ | FUNCTION | Win32 | Implemented |
| LTRIM | FUNCTION | codegen builtin | Implemented |
| LTRIM$ | FUNCTION | Win32 | Implemented |
| MACRO/END MACRO | BLOCK | PB/Win + PB/CC | Established |
| MAK | FUNCTION | codegen builtin | Implemented |
| MAT | STATEMENT | PB/Win + PB/CC | Established |
| MAX | FUNCTION | codegen builtin | Implemented |
| MCASE | FUNCTION | PB/Win + PB/CC | Implemented |
| MCASE$ | FUNCTION | runtime pb_mcase_string | Implemented |
| MEMORY | STATEMENT | PB/Win + PB/CC | Established |
| MEMORY_FILL | FUNCTION | codegen builtin | Implemented |
| MEMORY_FILLS | FUNCTION | codegen builtin | Implemented |
| MENU ADD POPUP | STATEMENT | PB/Win only | Implemented |
| MENU ADD STRING | STATEMENT | PB/Win only | Implemented |
| MENU ATTACH | STATEMENT | PB/Win only | Implemented |
| MENU CONTEXT | STATEMENT | PB/Win only | Implemented |
| MENU DELETE | STATEMENT | PB/Win only | Implemented |
| MENU DRAW BAR | STATEMENT | PB/Win only | Implemented |
| MENU GET STATE | STATEMENT | PB/Win only | Implemented |
| MENU GET TEXT | STATEMENT | PB/Win only | Implemented |
| MENU NEW BAR | STATEMENT | PB/Win only | Implemented |
| MENU NEW POPUP | STATEMENT | PB/Win only | Implemented |
| MENU SET STATE | STATEMENT | PB/Win only | Implemented |
| MENU SET TEXT | STATEMENT | PB/Win only | Implemented |
| METHOD / END METHOD | STATEMENT | PB/Win + PB/CC | Established |
| METRICS | FUNCTION | Win32 | Implemented |
| MID | FUNCTION | codegen builtin | Implemented |
| MID$ | STATEMENT | PB/Win + PB/CC | Established |
| MIN | FUNCTION | codegen builtin | Implemented |
| MKBYT | FUNCTION | codegen builtin | Implemented |
| MKBYT$ | STATEMENT | PB/Win + PB/CC | Established |
| MKCUR$ | STATEMENT | PB/Win + PB/CC | Established |
| MKCUX | FUNCTION | codegen builtin | Implemented |
| MKCUX$ | STATEMENT | PB/Win + PB/CC | Established |
| MKD$ | STATEMENT | PB/Win + PB/CC | Established |
| MKDIR | STATEMENT | PB/Win + PB/CC | Established |
| MKDWD | FUNCTION | codegen builtin | Implemented |
| MKDWD$ | STATEMENT | PB/Win + PB/CC | Established |
| MKE | FUNCTION | codegen builtin | Implemented |
| MKE$ | STATEMENT | PB/Win + PB/CC | Established |
| MKI$ | STATEMENT | PB/Win + PB/CC | Established |
| MKL$ | STATEMENT | PB/Win + PB/CC | Established |
| MKQ$ | STATEMENT | PB/Win + PB/CC | Established |
| MKS | FUNCTION | codegen builtin | Implemented |
| MKS$ | STATEMENT | PB/Win + PB/CC | Established |
| MKWRD | FUNCTION | codegen builtin | Implemented |
| MKWRD$ | STATEMENT | PB/Win + PB/CC | Established |
| MOD | FUNCTION | codegen builtin | Implemented |
| MONTHNAME | FUNCTION | codegen builtin | Implemented |
| MONTHNAME$ | FUNCTION | Win32 | Implemented |
| MOUSEPTR | STATEMENT | PB/CC only | Established |
| MSGBOX | STATEMENT | PB/Win only | Established |
| NAME | STATEMENT | PB/Win + PB/CC | Established |
| NOT | OPERATOR | PB/Win + PB/CC | Established |
| NUL | FUNCTION | PB/Win + PB/CC | Established |
| NUL$ | FUNCTION | Win32 | Implemented |
| OBJECT | STATEMENT | PB/Win + PB/CC | Established |
| OCT | FUNCTION | codegen builtin | Implemented |
| OCT$ | FUNCTION | Win32 | Implemented |
| OEMTOCHR | FUNCTION | codegen builtin | Implemented |
| OEMTOCHR$ | FUNCTION | Win32 | Implemented |
| ON CALL | STATEMENT | PB/Win + PB/CC | Implemented |
| ON ERROR | STATEMENT | PB/Win + PB/CC | Established |
| ON GOSUB | STATEMENT | PB/Win + PB/CC | Implemented |
| ON GOTO | STATEMENT | PB/Win + PB/CC | Implemented |
| OPEN | STATEMENT | PB/Win + PB/CC | Implemented |
| OPTION EXPLICIT | STATEMENT | PB/Win + PB/CC | Established |
| OR | OPERATOR | PB/Win + PB/CC | Established |
| PARSE | STATEMENT | PB/Win + PB/CC | Established |
| PARSE$ | FUNCTION | Win32 | Implemented |
| PARSECOUNT | FUNCTION | codegen builtin | Implemented |
| PATHNAME | FUNCTION | codegen builtin | Implemented |
| PATHNAME$ | FUNCTION | Win32 | Implemented |
| PATHSCAN | FUNCTION | codegen builtin | Implemented |
| PATHSCAN$ | FUNCTION | Win32 | Implemented |
| PEEK | FUNCTION | codegen builtin | Implemented |
| PLAY | FUNCTION | Win32 | Implemented |
| PLAY SOUND | STATEMENT | PB/Win + PB/CC | Implemented |
| PLAY WAVE | STATEMENT | PB/Win + PB/CC | Established |
| POKE | FUNCTION | codegen builtin | Implemented |
| PREFIX | BLOCK | PB/Win + PB/CC | Established |
| PRINT# | STATEMENT | PB/Win + PB/CC | Established |
| PRINTER$ | FUNCTION | Win32 | Implemented |
| PRINTERCOUNT | FUNCTION | codegen builtin | Implemented |
| PROCESS GET PRIORITY | STATEMENT | PB/Win + PB/CC | Established |
| PROCESS SET PRIORITY | STATEMENT | PB/Win + PB/CC | Established |
| PROFILE | STATEMENT | PB/Win + PB/CC | Established |
| PROGRESSBAR GET POS | STATEMENT | PB/Win only | Implemented |
| PROGRESSBAR GET RANGE | STATEMENT | PB/Win only | Implemented |
| PROGRESSBAR SET POS | STATEMENT | PB/Win only | Implemented |
| PROGRESSBAR SET RANGE | STATEMENT | PB/Win only | Implemented |
| PROGRESSBAR SET STEP | STATEMENT | PB/Win only | Implemented |
| PROGRESSBAR STEP | STATEMENT | PB/Win only | Implemented |
| PUT | STATEMENT | PB/Win + PB/CC | Implemented |
| PUT$ | STATEMENT | PB/Win + PB/CC | Implemented |
| PUT$$ | STATEMENT | PB/Win + PB/CC | Implemented |
| PUT_STR | FUNCTION | codegen builtin | Implemented |
| PUT_WSTR | FUNCTION | codegen builtin | Implemented |
| QUAD | FUNCTION | codegen builtin | Implemented |
| RAISEEVENT | STATEMENT | PB/Win + PB/CC | Implemented |
| RANDOMIZE | STATEMENT | PB/Win + PB/CC | Established |
| READ | FUNCTION | codegen builtin | Implemented |
| READ$ | FUNCTION | Win32 | Implemented |
| REDIM | STATEMENT | PB/Win + PB/CC | Implemented |
| REGEXPR | STATEMENT | PB/Win + PB/CC | Established |
| REGISTER | STATEMENT | PB/Win + PB/CC | Established |
| REGREPL | STATEMENT | PB/Win + PB/CC | Established |
| REM | STATEMENT | PB/Win + PB/CC | Established |
| REMAIN | FUNCTION | codegen builtin | Implemented |
| REMAIN$ | FUNCTION | Win32 | Implemented |
| REMOVE | FUNCTION | codegen builtin | Implemented |
| REMOVE$ | FUNCTION | Win32 | Implemented |
| REPEAT | FUNCTION | codegen builtin | Implemented |
| REPEAT$ | FUNCTION | Win32 | Implemented |
| REPLACE | STATEMENT | PB/Win + PB/CC | Established |
| RESET | STATEMENT | PB/Win + PB/CC | Established |
| RESOURCE SAVE FILE | STATEMENT | PB/Win + PB/CC | Implemented |
| RESOURCE$ | FUNCTION | Win32 | Implemented |
| RESUME | STATEMENT | PB/Win + PB/CC | Established |
| RETAIN | FUNCTION | codegen builtin | Implemented |
| RETAIN$ | FUNCTION | Win32 | Implemented |
| RETURN | STATEMENT | PB/Win + PB/CC | Established |
| RGB | FUNCTION | Win32 | Implemented |
| RIGHT | FUNCTION | codegen builtin | Implemented |
| RIGHT$ | FUNCTION | Win32 | Implemented |
| RMDIR | STATEMENT | PB/Win + PB/CC | Established |
| RND | FUNCTION | codegen builtin | Implemented |
| ROTATE | STATEMENT | PB/Win + PB/CC | Established |
| ROUND | FUNCTION | LLVM intrinsic | Implemented |
| RSET | STATEMENT | PB/Win + PB/CC | Established |
| RSET$ | FUNCTION | Win32 | Implemented |
| RTRIM | FUNCTION | codegen builtin | Implemented |
| RTRIM$ | FUNCTION | Win32 | Implemented |
| SCROLLBAR GET PAGESIZE | STATEMENT | PB/Win only | Implemented |
| SCROLLBAR GET POS | STATEMENT | PB/Win only | Implemented |
| SCROLLBAR GET RANGE | STATEMENT | PB/Win only | Implemented |
| SCROLLBAR GET TRACKPOS | STATEMENT | PB/Win only | Implemented |
| SCROLLBAR SET PAGESIZE | STATEMENT | PB/Win only | Implemented |
| SCROLLBAR SET POS | STATEMENT | PB/Win only | Implemented |
| SCROLLBAR SET RANGE | STATEMENT | PB/Win only | Implemented |
| SEC | FUNCTION | C runtime (MSVCRT) | Implemented |
| SECH | FUNCTION | C runtime (MSVCRT) | Implemented |
| SEEK | STATEMENT | PB/Win + PB/CC | Implemented |
| SELECT CASE/END SELECT | BLOCK | PB/Win + PB/CC | Established |
| SETATTR | STATEMENT | PB/Win + PB/CC | Established |
| SETEOF | STATEMENT | PB/Win + PB/CC | Implemented |
| SGN | FUNCTION | codegen builtin | Implemented |
| SHELL | STATEMENT | PB/Win + PB/CC | Established |
| SHIFT | STATEMENT | PB/Win + PB/CC | Established |
| SHRINK | FUNCTION | codegen builtin | Implemented |
| SHRINK$ | FUNCTION | Win32 | Implemented |
| SIN | FUNCTION | LLVM intrinsic | Implemented |
| SINGLE | FUNCTION | codegen builtin | Implemented |
| SINH | FUNCTION | LLVM intrinsic | Implemented |
| SIZEOF | FUNCTION | codegen builtin | Implemented |
| SLEEP | STATEMENT | PB/Win + PB/CC | Established |
| SPACE | FUNCTION | codegen builtin | Implemented |
| SPACE$ | FUNCTION | Win32 | Implemented |
| SPLIT | STATEMENT | PB/Win + PB/CC | Established |
| SQR | FUNCTION | LLVM intrinsic | Implemented |
| STATIC | STATEMENT | PB/Win + PB/CC | Established |
| STATUSBAR SET PARTS | STATEMENT | PB/Win + PB/CC | Implemented |
| STATUSBAR SET TEXT | STATEMENT | PB/Win + PB/CC | Implemented |
| STR | FUNCTION | codegen builtin | Implemented |
| STR$ | FUNCTION | Win32 | Implemented |
| STRDELETE | FUNCTION | codegen builtin | Implemented |
| STRDELETE$ | FUNCTION | Win32 | Implemented |
| STRING | FUNCTION | codegen builtin | Implemented |
| STRINSERT | FUNCTION | codegen builtin | Implemented |
| STRINSERT$ | FUNCTION | Win32 | Implemented |
| STRPTR | FUNCTION | codegen builtin | Implemented |
| STRREVERSE | FUNCTION | codegen builtin | Implemented |
| STRREVERSE$ | FUNCTION | Win32 | Implemented |
| SWAP | STATEMENT | PB/Win + PB/CC | Established |
| SWITCH | FUNCTION | Win32 | Implemented |
| SWITCH$ | FUNCTION | codegen builtin | Implemented |
| TAB DELETE | STATEMENT | PB/Win only | Implemented |
| TAB GET COUNT | STATEMENT | PB/Win only | Implemented |
| TAB GET DIALOG | STATEMENT | PB/Win only | Implemented |
| TAB GET IMAGE | STATEMENT | PB/Win only | Implemented |
| TAB GET PAGE | STATEMENT | PB/Win only | Implemented |
| TAB GET SELECT | STATEMENT | PB/Win only | Implemented |
| TAB GET TEXT | STATEMENT | PB/Win only | Implemented |
| TAB INSERT PAGE | STATEMENT | PB/Win only | Implemented |
| TAB RESET | STATEMENT | PB/Win only | Implemented |
| TAB SELECT | STATEMENT | PB/Win only | Implemented |
| TAB SET IMAGE | STATEMENT | PB/Win only | Implemented |
| TAB SET IMAGELIST | STATEMENT | PB/Win only | Implemented |
| TAB SET TEXT | STATEMENT | PB/Win only | Implemented |
| TAB$ | FUNCTION | Win32 | Implemented |
| TALLY | FUNCTION | codegen builtin | Implemented |
| TAN | FUNCTION | LLVM intrinsic | Implemented |
| TANH | FUNCTION | LLVM intrinsic | Implemented |
| TCP ACCEPT | STATEMENT | PB/Win + PB/CC | Established |
| TCP CLOSE | STATEMENT | PB/Win + PB/CC | Established |
| TCP LINE INPUT | STATEMENT | PB/Win + PB/CC | Established |
| TCP NOTIFY | STATEMENT | PB/Win + PB/CC | Implemented |
| TCP OPEN | STATEMENT | PB/Win + PB/CC | Established |
| TCP PRINT | STATEMENT | PB/Win + PB/CC | Established |
| TCP RECV | STATEMENT | PB/Win + PB/CC | Established |
| TCP SEND | STATEMENT | PB/Win + PB/CC | Established |
| THREAD CLOSE | STATEMENT | PB/Win + PB/CC | Established |
| THREAD CREATE | STATEMENT | PB/Win + PB/CC | Established |
| THREAD GET PRIORITY | STATEMENT | PB/Win + PB/CC | Established |
| THREAD RESUME | STATEMENT | PB/Win + PB/CC | Established |
| THREAD SET PRIORITY | STATEMENT | PB/Win + PB/CC | Established |
| THREAD STATUS | STATEMENT | PB/Win + PB/CC | Established |
| THREAD SUSPEND | STATEMENT | PB/Win + PB/CC | Established |
| THREADCOUNT | FUNCTION | codegen builtin | Implemented |
| THREADED | STATEMENT | PB/Win + PB/CC | Established |
| THREADID | FUNCTION | Win32 | Implemented |
| TIMER | FUNCTION | codegen builtin | Implemented |
| TIX | STATEMENT | PB/Win + PB/CC | Established |
| TOOLBAR ADD BUTTON | STATEMENT | PB/Win only | Implemented |
| TOOLBAR ADD SEPARATOR | STATEMENT | PB/Win only | Implemented |
| TOOLBAR DELETE BUTTON | STATEMENT | PB/Win only | Implemented |
| TOOLBAR GET COUNT | STATEMENT | PB/Win only | Implemented |
| TOOLBAR GET STATE | STATEMENT | PB/Win only | Implemented |
| TOOLBAR SET IMAGELIST | STATEMENT | PB/Win only | Implemented |
| TOOLBAR SET STATE | STATEMENT | PB/Win only | Implemented |
| TRACE | STATEMENT | PB/Win + PB/CC | Established |
| TREEVIEW DELETE | STATEMENT | PB/Win only | Implemented |
| TREEVIEW GET BOLD | STATEMENT | PB/Win only | Implemented |
| TREEVIEW GET CHECK | STATEMENT | PB/Win only | Implemented |
| TREEVIEW GET CHILD | STATEMENT | PB/Win only | Implemented |
| TREEVIEW GET COUNT | STATEMENT | PB/Win only | Implemented |
| TREEVIEW GET EXPANDED | STATEMENT | PB/Win only | Implemented |
| TREEVIEW GET NEXT | STATEMENT | PB/Win only | Implemented |
| TREEVIEW GET PARENT | STATEMENT | PB/Win only | Implemented |
| TREEVIEW GET PREVIOUS | STATEMENT | PB/Win only | Implemented |
| TREEVIEW GET ROOT | STATEMENT | PB/Win only | Implemented |
| TREEVIEW GET SELECT | STATEMENT | PB/Win only | Implemented |
| TREEVIEW GET TEXT | STATEMENT | PB/Win only | Implemented |
| TREEVIEW GET USER | STATEMENT | PB/Win only | Implemented |
| TREEVIEW INSERT ITEM | STATEMENT | PB/Win only | Implemented |
| TREEVIEW RESET | STATEMENT | PB/Win only | Implemented |
| TREEVIEW SELECT | STATEMENT | PB/Win only | Implemented |
| TREEVIEW SET BOLD | STATEMENT | PB/Win only | Implemented |
| TREEVIEW SET CHECK | STATEMENT | PB/Win only | Implemented |
| TREEVIEW SET EXPANDED | STATEMENT | PB/Win only | Implemented |
| TREEVIEW SET IMAGELIST | STATEMENT | PB/Win only | Implemented |
| TREEVIEW SET TEXT | STATEMENT | PB/Win only | Implemented |
| TREEVIEW SET USER | STATEMENT | PB/Win only | Implemented |
| TREEVIEW UNSELECT | STATEMENT | PB/Win only | Implemented |
| TRIM | FUNCTION | codegen builtin | Implemented |
| TRIM$ | FUNCTION | Win32 | Implemented |
| TRUNC | FUNCTION | LLVM intrinsic | Implemented |
| TRY/END TRY | BLOCK | PB/Win + PB/CC | Established |
| TYPE SET | STATEMENT | PB/Win + PB/CC | Established |
| TYPE/END TYPE | BLOCK | PB/Win + PB/CC | Established |
| UCASE | FUNCTION | codegen builtin | Implemented |
| UCASE$ | FUNCTION | Win32 | Implemented |
| UCODE$ | FUNCTION | Win32 | Implemented |
| UCODEPAGE | STATEMENT | PB/Win + PB/CC | Implemented |
| UDP CLOSE | STATEMENT | PB/Win + PB/CC | Established |
| UDP NOTIFY | STATEMENT | PB/Win + PB/CC | Implemented |
| UDP OPEN | STATEMENT | PB/Win + PB/CC | Established |
| UDP RECV | STATEMENT | PB/Win + PB/CC | Established |
| UDP SEND | STATEMENT | PB/Win + PB/CC | Established |
| UNLOCK | STATEMENT | PB/Win + PB/CC | Established |
| UNWRAP | FUNCTION | codegen builtin | Implemented |
| UNWRAP$ | FUNCTION | Win32 | Implemented |
| USING | FUNCTION | codegen builtin | Implemented |
| USING$ | FUNCTION | Win32 | Implemented |
| UTF8TOCHR | FUNCTION | codegen builtin | Implemented |
| UTF8TOCHR$ | FUNCTION | Win32 | Implemented |
| VAL | STATEMENT | PB/Win + PB/CC | Established |
| VARPTR | FUNCTION | codegen builtin | Implemented |
| VERIFY | FUNCTION | codegen builtin | Implemented |
| WAITKEY | FUNCTION | codegen builtin | Implemented |
| WAITKEY$ | FUNCTION | PB/CC + console | Implemented |
| WINDOW GET | STATEMENT | PB/Win only | Established |
| WINDOW SET | STATEMENT | PB/Win only | Established |
| WORD | FUNCTION | codegen builtin | Implemented |
| WRAP | FUNCTION | codegen builtin | Implemented |
| WRAP$ | FUNCTION | Win32 | Implemented |
| WRITE | FUNCTION | codegen builtin | Implemented |
| WRITE# | STATEMENT | PB/Win + PB/CC | Implemented |
| XOR | OPERATOR | PB/Win + PB/CC | Established |
| XPRINT ARC | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT ATTACH | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT BOX | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT CANCEL | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT CELL | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT CELL SIZE | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT CHR SIZE | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT CLOSE | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT COLOR | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT COPY | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT ELLIPSE | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT FORMFEED | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT GET ATTACH | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT GET CANVAS | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT GET CLIENT | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT GET CLIP | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT GET COLLATE | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT GET COLORMODE | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT GET COPIES | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT GET DC | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT GET DUPLEX | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT GET LINES | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT GET MARGIN | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT GET MIX | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT GET ORIENTATION | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT GET OVERLAP | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT GET PAGES | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT GET PAPER | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT GET PAPERS | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT GET PIXEL | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT GET POS | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT GET PPI | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT GET QUALITY | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT GET SCALE | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT GET SELECTION | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT GET SIZE | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT GET STRETCHMODE | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT GET TEXTALIGN | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT GET TRAY | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT GET TRAYS | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT GET WORDWRAP | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT GET WRAP | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT IMAGELIST | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT LINE | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT PIE | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT POLYGON | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT POLYLINE | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT PREVIEW | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT PRINT | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT RENDER | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT SCALE | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT SET CLIP | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT SET COLLATE | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT SET COLORMODE | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT SET COPIES | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT SET DUPLEX | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT SET FONT | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT SET MIX | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT SET ORIENTATION | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT SET OVERLAP | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT SET PAGES | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT SET PAPER | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT SET PIXEL | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT SET POS | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT SET QUALITY | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT SET STRETCHMODE | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT SET TEXTALIGN | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT SET TRAY | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT SET WORDWRAP | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT SET WRAP | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT SPLIT | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT STRETCH | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT STYLE | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT TEXT SIZE | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT WIDTH | STATEMENT | PB/Win + PB/CC | Implemented |
| XPRINT_GET_COLOR | FUNCTION | codegen builtin | Implemented |
| XPRINT_SET_COLOR | FUNCTION | codegen builtin | Implemented |
