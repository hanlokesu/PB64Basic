### v0.2.129 (2026-09-12) - Error 7 (Out of memory) malloc NULL check

- **Error 7 (Out of memory)**: Added `malloc_or_raise` helper in codegen.rs that checks for NULL after every `malloc()` call and raises runtime Error 7 (`pb_raise_error(7)`) if allocation fails. Covers all 9 malloc call sites: CHR$ range, CHR$ multi, CHR$$ multi, STR$, HEX$, num_to_string, STRING$, SPACE$, STRING$$. This completes the runtime error coverage for CHW error code 7.
- **#284 progress**: Runtime errors now landed: 5 (Illegal function call), 6 (Overflow), 7 (Out of memory), 9 (Subscript/Pointer out of range), 11 (Division by zero), 24 (Device timeout). Remaining: 51 (Internal error), 99 (Object error).
