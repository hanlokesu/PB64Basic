### v0.2.199 (2026-11-26)

- Fixed the José Roca WINAPI include-file failures reported in Discussion #8.
  `METHOD Xor` and `METHOD On` are now accepted as member names, so
  `CGdipRegion.inc` parses; `@pointer.@pointer_field` now dereferences a
  pointer-valued member, covering `@pImageCodecInfo.@MimeType` in
  `GDIPUTILS.INC` and `CGdiPlus.inc`; and a procedure declaration may reuse a
  macro name, so the `MAKEFOURCC` macro from `MMSystem.inc` no longer rewrites
  the `FUNCTION MAKEFOURCC` signature in `ddraw.inc`.
- Verified with the complete gbClientCapture v2.1 source from the PUMP thread:
  it parses and builds a native x86_64 executable against the José Roca
  `C:\PBWin10\WinAPI` headers.
- Added `examples/batch409_test.bas`, which exercises nested pointer-member
  dereference and the `Xor`/`On` method names at run time.
- Also includes the validated second-batch Gary Beene GUI fixes already in the
  tree: PBR rendering, status-bar layout, font charset handling, ScrollBar
  defaults, and GRAPHIC quadrant geometry.
