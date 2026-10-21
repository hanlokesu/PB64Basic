// sum.c — C equivalent of Erol's Poasm module (Win64: arg1=rcx, arg2=rdx).
// clang -c -O2 --target=x86_64-pc-windows-msvc sum.c -o sum.obj
__int64 sum(__int64 a, __int64 b) { return a + b; }
