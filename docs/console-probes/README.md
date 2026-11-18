# Console probes (batch 378)

These files replace the lost `b377_*` console probes. The original `b377_scroll4`,
`b377_newend` and `b377_stdin` sources/logs were never committed, so their claimed
passes were not reproducible. Each probe below is a standalone `.bas` whose result
is written next to it and archived here.

Compiler: the `batch378_lprint`, `batch378_redim` and `batch378_remarks` probes
were written against the modified `pbsrc_b378/target/release/PB64Basic.exe`.
Their parser/codegen changes were not in `v0.2.166`; they are included in
`v0.2.169` and later releases. The other probes were run with the `pbsrc_b378`
build as well; runtime objects are rebuilt from `pbcompiler/runtime/pb_runtime.c`
in `pbsrc_b378`.

## Files

| Probe | What it proves | Logs |
|---|---|---|
| `batch378_page_bounds.bas` | `CON.PAGE.ACTIVE` / `CON.PAGE.VISIBLE` accept 1..8; 0 and 9 raise `ERR=5` and leave the page unchanged; valid values clear `ERR=0`. | `batch378_page_bounds_x64.out`, `batch378_page_bounds_x86.out`, `batch378_page_bounds_console.out` |
| `batch378_scroll4.bas` | Independent assertions for `CON.SCROLL.DOWN`, `.LEFT`, `.RIGHT`, plus the 99-line full clear. | `batch378_scroll4_x64.out`, `batch378_scroll4_x86.out` |
| `batch378_handle.bas` | `CON.HANDLE` is non-zero only in a real console host. | `batch378_handle.txt` |
| `batch378_newend.bas` | `CON.END` releases a console; `CON.NEW` allocates a new one; `CON.END` again returns handle 0 with `ERR=0`. | `batch378_newend_x64.txt`, `batch378_newend_x86.txt` |
| `batch378_stdin.bas` | With redirected stdin/stdout: `CON.IN=0`, `CON.OUT=0`, `CON.STDIN.LINE` returns both lines, `CON.STDEOF=1`. | `batch378_stdin_x64.txt`, `batch378_stdin_x86.txt` |
| `batch384_stdin_edge.bas` | `CON.STDIN.LINE` accepts a 5000-byte line without truncation, preserves the next line, reports `CON.STDEOF=0` while another line remains and 1 after the final line; `0x1A` (Ctrl-Z) is treated as EOF. | `batch384_stdin_edge_x64.txt`, `batch384_stdin_edge_x86.txt` |
| `batch384_props.bas` | Real-console assertions for `CON.PAGE.ACTIVE/VISIBLE` 0/9 (`ERR=5`, value preserved), `CON.SIZE=0,0` (`ERR=5`), same-value `CON.SIZE`/`CON.LOC` (`ERR=0`), `CON.FOCUS` (`ERR=0`), `CON.GETSTDVID(0)` non-zero and `CON.GETSTDVID(9)` zero with `ERR=5`. | `batch384_props_x64.txt`, `batch384_props_x86.txt` |
| `batch384_stdin_console.bas` | Manual, source-only probe for one `CON.STDIN.LINE` from a real console keyboard; it has no automation result yet. | none |
| `batch378_exact.bas` | `CLS` and `CON.CLS` blank the active page; `LOCATE`/`CON.LOCATE` set `CURSORX/Y` exactly; `PCOPY`/`CON.PCOPY` copy page content; `CON.SIZE`/`CON.LOC` agree with `CONSOLE GET`, and same-value `CONSOLE SET SIZE/LOC` round-trips. | `batch378_exact_x64.txt`, `batch378_exact_x86.txt` |
| `batch378_view.bas` | Diagnostic only: records `CON.SCREEN`, `CON.VIRTUAL`, `CON.VIEW`, and `CONSOLE SET VIEW` on this host. It does not assert success because the host does not move the viewport. | `batch378_view.txt` |
| `batch378_input.bas` | Real keyboard: `CON.INPUT` parses `123,alpha`, `CON.LINE.INPUT` keeps `hello world`, `CON.WAITSTAT` wakes on a key, `CON.MOUSE.ON` + `CON.WAITSTAT` + `CON.WAITKEY$` return a 4-byte mouse key with event 4 / button 1, and `CON.MOUSE.ROW/COL` report the click cell. | `batch378_input_x64.txt`, `batch378_input_x86.txt` |
| `batch378_input_redirect.bas` | Piping `7,pipe` into the probe and then typing `9,manual` proves `CON.INPUT` ignores redirected stdin and reads the keyboard: the result is `n=9 s=[manual]`, not the piped `7,pipe`. | `batch378_input_redirect.txt`, `batch378_input_redirect_x86.txt` |
| `batch378_lprint.bas` | Exact `LPRINT` bytes: semicolon suppresses CRLF, comma advances to the 14-column zone, trailing semicolon suppresses the line break, and a trailing comma does not break the line. | `batch378_lprint_x64.txt`, `batch378_lprint_x86.txt`, `batch378_lprint1..4.txt`, `batch378_lprint_result.txt` |
| `batch378_redim.bas` | `ARRAY REDIM INCR/DECR` is a real heap realloc: values 1..4 survive, added slots are zeroed, live `UBOUND` follows the descriptor, shrink to one keeps only `a(1)`, and a second grow past the first capacity keeps the prefix. | `batch378_redim_x64.txt`, `batch378_redim_x86.txt`, `batch378_redim_result.txt`, `batch378_redim_result_x64.txt`, `batch378_redim_result_x86.txt` |
| `batch378_remarks.bas` | `RemarksData` returns a full-width pointer whose payload is `No remarks available.` (`lstrlenA=21` and `PEEK(BYTE,p)=78`), and the uppercase `HISCORE`/`REMARKSDATA` aliases link from PB identifiers. `HiScore` itself remains an intentional stub returning 1. | `batch378_remarks_x64.txt`, `batch378_remarks_x86.txt` |

## Observed results

- `batch378_page_bounds`: `batch378 page bounds, failures= 0` for x64 and i686, both
  in the Agent shell and in a real Git CMD console.
- `batch378_scroll4`: `batch378 scroll4, failures= 0` for x64 and i686 in a real
  Git CMD console. `CON.HANDLE` was `66680` there.
- `batch378_handle`: `handle= 66680`, `err= 0` in a real Git CMD console. In the
  Agent PowerShell/ConPTY host `CON.HANDLE` is `0`.
- `batch378_newend`: x64 `start=0`, `pre-end=0`, `new1=655390`, `end=0`, `new2=524458`,
  all `err=0`; i686 `new1=787298`, `new2=327922`, all `err=0`.
- `batch378_stdin`: x64 and i686 both `in= 0  out= 0`, `line1=[alpha ]`,
  `line2=[beta ]`, `eof= 1`. The trailing space is part of the piped line
  (`echo alpha` produced it); the probe deliberately preserves input bytes.
- `batch384_stdin_edge`: with `batch384_long_input.txt` (5000 `X` plus `tail`),
  x64 and i686 both report `len1= 5000`, `eof1= 0`, `len2= 4`, `eof2= 1`,
  `failures= 0`. With `batch384_ctrlz_input.txt` (`A`, `0x1A`, `B`), both report
  `len1= 1`, `eof1= 1`, `len2= 0`, `eof2= 1`, `failures= 0`. The long case
  would have been cut at 4095 bytes by the previous fixed-buffer implementation.
- `batch384_props`: in a real Git CMD console, x64 and i686 both report
  `page9 err= 5 page= 1`, `pagev0 err= 5 visible= 1`, `pagev9 err= 5`,
  `size0 err= 5 read= 960,480`, `size_same err= 0`, `loc_same err= 0`,
  `stdvid0` non-zero with `err= 0`, `stdvid9= 0 err= 5`, `focus err= 0`, and
  `failures= 0`. A host without a console returns `ERR=5` by design for these
  window-dependent calls; that path is not covered by this real-console log.
- `batch378_exact`: `batch378 exact, failures= 0` for both x64 and i686 in a real
  Git CMD console. The probe deliberately leaves `CON.VIEW` to
  `batch378_view.bas`, because this host accepts the setter but does not move the
  viewport.
- `batch378_view`: `screen= 10, 80 err= 0`, `virtual= 40, 120 err= 0`,
  `after CON.VIEW 5,7: row= 1 col= 1 err= 0`, and the same `1,1` after
  `CONSOLE SET VIEW 3,4`. This is an environment limitation, not a passing
  assertion: a direct Win32 probe using `CONOUT$` showed that after
  `SetConsoleScreenBufferSize(120,40)` followed by
  `SetConsoleWindowInfo(TRUE, {0,0,79,9})`, `GetConsoleScreenBufferInfo` reports
  the buffer resized to `80,10` instead of a `80x10` viewport inside a `120x40`
  buffer. Do not claim `CON.VIEW` is verified until it is rerun in a host that
  keeps a larger buffer and moves the viewport.
- `batch378_input`: x64 and i686 both produced `n= 123 s=[alpha]`,
  `line=[hello world]`, `key len= 1 hex=5A`, `mouse stat= 1 keylen= 4`,
  `mouse key=FFFF0401`, `mouse row= 10 col= 62`, `mouse buttons= 2`. The first
  mouse record is a `MOUSE_MOVED` event; `CON.INKEY$` consumed it and returned an
  empty string. `CON.WAITKEY$("", 2000)` then returned the press record
  (`event=4`, `button=1`). This is why the probe uses `WAITKEY$` for the mouse
  assertion instead of one `INKEY$` call.
- `batch378_input_redirect`: `n= 9 s=[manual]` after a `7,pipe` pipe, for both
  x64 and i686. The redirected bytes were not consumed; the typed keyboard line was. In the Agent
  PowerShell host the same piped run blocks because `CON.INPUT` waits on a
  console keyboard; it does not fall back to stdin.
- `batch378_lprint`: x64 and i686 both `batch378 lprint, failures= 0`; exact
  files are 4 B (`AB\r\n`), 17 B (`A` + 14-column zone + `B\r\n`), 17 B for the
  trailing-comma case, and 1 B (`A`, no CRLF) for the trailing-semicolon case.
- `batch378_redim`: x64 and i686 both `batch378 redim, failures= 0`, with stage
  logs `g1`..`g5`: `UBOUND` 8/6/10/1/3, values 11/22/33/44 preserved across
  grows and shrinks, and new slots zero. This is probe-level evidence; the full
  regression and release gates were not run in this batch.
- `batch378_remarks`: x64 and i686 both `batch378 remarks, failures= 0`,
  `hiscore= 1`, `p=No remarks available.`, `lstrlenA=21`, `PEEK(BYTE,p)=78`.
  The pointer return width is therefore full on x64; the old `int` return
  truncated it. `HiScore` is still the documented stub, not a real VARICHEK
  table implementation.

## Scroll direction semantics

Measured matrix, not inferred from the statement names:

| Statement | Content movement | Blank area |
|---|---|---|
| `CON.SCROLL.UP 1` | down | top |
| `CON.SCROLL.DOWN 1` | up | bottom |
| `CON.SCROLL.LEFT 1` | right | left |
| `CON.SCROLL.RIGHT 1` | left | right |

The first version of `batch378_scroll4.bas` asserted the opposite expectation and
reported 11 failures. The corrected probe reports 0 failures on both targets.

## Probe pitfall: variable file number (not a compiler defect)

`batch378_stdin.bas` originally used `LOCAL f AS LONG` followed by
`OPEN "file" FOR OUTPUT AS #f`. The emitted IR does not store the `pb_open_ex`
result back into `f`, but that is not a code-generation bug: CHW defines
`filenum&` as an input expression (a unique integer in 1..32767, normally from
`FREEFILE`), and `OPEN` never writes a file number back into the variable. An
uninitialized `LOCAL f AS LONG` is `0`, which is outside the valid range; the
runtime rejects it with Error 52, so `PRINT #f` has no file to write to.

Assign the number before the OPEN:

```powerbasic
LOCAL f AS LONG
f = FREEFILE          ' or f = 1
OPEN "file" FOR OUTPUT AS #f
PRINT #f, "text"
CLOSE #f
```

`examples/batch382_open_var.bas` locks this path: `FREEFILE` returns a valid
number, the same variable is reused for OUTPUT and INPUT, and the read-back
matches. Literal `AS #1` remains fine; the variable form just needs a valid
value first.

## How to run

Build one target at a time; a later build overwrites the same `.exe` name.

```bat
set PB=C:\path\to\pbsrc_b378\target\release\PB64Basic.exe
set WT=C:\path\to\pbsrc_b378

"%PB%" build "%WT%\examples\batch378_scroll4.bas" --exe --console ^
  --target x86_64-pc-windows-msvc --runtime-lib "%WT%\pb_runtime_x64.obj"
```

Run console-sensitive probes in a real console (Git CMD, Windows Terminal, or
`conhost`), not in the Agent shell: `CON.HANDLE` is 0 under ConPTY, and the
`has_con` gate in `batch377_test.bas` then skips every console assertion.

`batch378_newend.bas` must be launched with no inherited console. On Windows this
means `ProcessStartInfo.CreateNoWindow = true` with `UseShellExecute = false`. The
probe calls `CON.END` first so it starts from a known no-console state; calling
`CON.NEW` while a hidden console is still attached can fail with `ERR=5`.

`batch378_stdin.bas` is run with redirected stdin:

```bat
(echo alpha& echo beta)| batch378_stdin.exe
```

`batch384_stdin_edge.bas` is run with the matching input file on stdin:

```bat
batch384_stdin_edge.exe < batch384_long_input.txt
batch384_stdin_edge.exe < batch384_ctrlz_input.txt
```

`batch384_props.bas` needs a real console window and writes
`batch384_props.txt`; run it from `cmd.exe`, not from the Agent ConPTY shell.

## Not covered here

`CON.INPUT` without the optional prompt, `CON.INPUT` with a leading `;`, the
`no_newline` flag, `CON.STDIN.LINE` from a real console keyboard, Ctrl-Z typed
at a real console, and a second mouse button/double-click matrix still need
dedicated probes. `CON.INPUT`, `CON.LINE.INPUT`, `CON.WAITSTAT`, and a real
left-button click are covered by `batch378_input.bas` and
`batch378_input_redirect.bas`; `CON.STDIN.LINE` long-line and redirected
Ctrl-Z behavior is covered by `batch384_stdin_edge.bas`; `CON.FOCUS` and the
setter error paths are covered by `batch384_props.bas`.

## Related batch 379 file probes

`docs/batch379-probes/` holds the sequential WIDE `SEEK`, ANSI `0x1A` EOF,
`LINE INPUT #` array `RECORDS`/`TO`, and fixed `WSTRING` probes. The
`run_examples.py` classifier treats `CON.INPUT` as interactive because it reads
`CONIN$` directly; `batch378_input_redirect.bas` therefore must not be launched in
a headless loop. The file-only `batch379_*` probes are forced runnable through
`FORCE_RUN` because their `LINE INPUT #` targets a local file, not the keyboard.
