# batch 379 file-format probes

These probes cover the evidence boundaries tracked in the batch 379 work:
sequential WIDE `SEEK` byte offsets, ANSI `CHR$(26)` EOF, the array form of
`LINE INPUT #` with `RECORDS`/`TO`, and fixed `WSTRING` arrays / TYPE fields.

Build with the `pbsrc_b378` compiler and the runtime object for the target
under test. Each probe writes a small `.txt` result next to its `.exe`.

| probe | proves |
| --- | --- |
| `batch379_wide_seek.bas` | `SEEK` on a sequential WIDE file is a byte offset; lines resume at the requested byte. |
| `batch379_ansi_eof.bas` | ANSI `LINE INPUT #` stops at `CHR$(26)`, returns the prefix, and latches `EOF=-1`. |
| `batch379_lineinput_array.bas` | `LINE INPUT #f, Arr$() RECORDS n TO count` honours both limits; a missing `RECORDS` uses the array size; `TO` reports the actual count. |
| `batch379_wstring_array.bas` | Fixed `WSTRING * n` array elements and TYPE fields are space-padded and `LEN` reports `n`. |

Observed results (x86_64 and i686):

- `batch379 wide seek, failures= 0`
- `batch379 ansi eof, failures= 0` / `s=[A] e=-1`
- `batch379 lineinput array, failures= 0`
- `batch379 wstring array, failures= 0`

The `LINE INPUT #` array syntax has no comma between `Arr$()` and `RECORDS`.
`LINE INPUT #1, a(), RECORDS 3` is silently treated as a scalar read with the
clause dropped; use `LINE INPUT #1, a() RECORDS 3 TO n`.

`batch379_wide_seek.txt` / `batch379_ansi_eof.txt` and similar un-suffixed
result files are overwritten by the last run; the `_x64.txt` / `_x86.txt`
copies are the archived evidence.
