## v0.2.200 (batch 410) - 2026-11-26

This release closes the deterministic parser, structure-layout and runtime
defects found by the third Gary Beene GUI batch. The batch produced 42/50
samples where both compilers succeed, 25 identical, 2 near and 9 diff; there
are no PB64-only compile failures.

### Parser and preprocessor

- `DIALOG NEW` accepts an omitted style or exstyle slot: `...,, TO hDlg` and
  `...,,, TO hDlg` no longer try to parse the empty slot as an expression.
- `GRAPHIC CLEAR` treats a colon as a statement boundary, so
  `Graphic Clear : Graphic Redraw` is two statements.
- The unsuffixed `%LVN_GETDISPINFO` alias resolves to the ANSI notification
  `-150`, because PB64 creates ANSI common controls. `%LVN_GETDISPINFOW`
  keeps `-177`.

### Code generation

- `BYVAL ... AS DWORD/LONG` handle parameters in header `SUB` wrappers are
  widened to pointer size on x64, so `ListView_SetItemCountEx(hwndLV, ...)`
  no longer truncates the HWND.
- `LVITEM.iItem` / `iSubItem` stay 32-bit; removing the over-broad `IITEM`
  rule restored the Win64 `LVITEMW` layout and the `pszText` offset.
- `NMCUSTOMDRAW.dwItemSpec` and `lItemlParam` are pointer-sized; this restores
  the `clrText` / `clrTextBk` offsets used by ListView custom draw.
- `%RGB_PALETURQUOISE` is a built-in equate.

### Runtime

- `WM_NOTIFY` returns the dialog callback LRESULT. ListView/TreeView custom
  draw therefore receives `CDRF_NOTIFYITEMDRAW` and the per-item passes.
- RichEdit no longer selects the whole initial text when `ES_NOHIDESEL` is
  present; PBWin leaves the caret at the start.
- `LISTVIEW SET STYLEXX` passes the style as both mask and value, so
  `%LVS_EX_*` bits are applied instead of discarded by a zero mask.

### Validation

- `examples/batch410_test.bas` covers the two empty `DIALOG NEW` slots,
  `GRAPHIC CLEAR : GRAPHIC REDRAW`, `%RGB_PALETURQUOISE`, the ANSI
  `%LVN_GETDISPINFO` alias and the explicit W alias; `FAILURES= 0`.
- Gary Beene batch 3 report: `batch3_report.md`, `.csv`, `.json`, `.html`.
  Compile layer: PB64 46/50, PBWin10 42/50, both 42/50, PB64-only failures 0.
  Semantic layer: identical 25, near 2, diff 9. The remaining diffs are
  control-fidelity gaps (ListView checkbox state images, TreeView state
  images, EnumFontFamilies ListBox fill, Graphic Copy clipping and Graphic
  rounded-shape composition), not parser or codegen failures.
