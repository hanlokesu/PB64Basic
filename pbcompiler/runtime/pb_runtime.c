// =====================================================================
// Modified by the hanlokesu/PB64Basic fork (https://github.com/hanlokesu/PB64Basic).
// Derived from benstopics/powerbasilisk (https://github.com/benstopics/powerbasilisk), Apache-2.0.
// See LICENSE for the full license terms.
// =====================================================================
/*
 * pb_runtime.c � PowerBASIC runtime support for PB64Basic
 *
 * Implements FORMAT$, PARSE$, REPLACE, and other PB-specific functions
 * that don't have direct C library equivalents.
 *
 * Build: clang -c pb_runtime.c -o pb_runtime.obj --target=<triple> \
 *            -ffunction-sections -fdata-sections
 * Link:  clang program.ll pb_runtime.obj -o program.exe
 *          (/OPT:REF is on by default, so unreferenced runtime functions
 *           in their own COMDAT sections are discarded from the EXE.)
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>
#include <math.h>
#include <time.h>  /* clock for pb_tix fallback */
#include <io.h>  /* _locking / _fileno for LOCK/UNLOCK */
#include <conio.h>  /* _getch for WAITKEY$ */
#include <direct.h>  /* _chdrive for CHDRIVE */
#include <sys/locking.h>  /* _LK_LOCK / _LK_UNLCK */
#include <sys/stat.h>  /* _stat / _S_IFDIR for ISFOLDER */
#include <excpt.h>  /* GetExceptionCode for SEH filters */
#include <malloc.h>  /* _heapchk for heap integrity probes */
__declspec(dllimport) void* __stdcall GetProcessHeap(void);
__declspec(dllimport) int __stdcall HeapValidate(void*, unsigned long, const void*);

#ifdef _WIN32
/* Declare only what we need from oleaut32 � avoids pulling in all of windows.h */
__declspec(dllimport) char* __stdcall SysAllocStringByteLen(const char* psz, unsigned int len);
__declspec(dllimport) void __stdcall SysFreeString(char* bstrString);
__declspec(dllimport) wchar_t* __stdcall SysAllocStringLen(const wchar_t* psz, unsigned int len);
__declspec(dllimport) unsigned long __stdcall SysStringByteLen(const char* bstr);
__declspec(dllimport) unsigned long __stdcall GetFileAttributesA(const char* lpFileName);
__declspec(dllimport) int __stdcall MoveFileExA(const char* lpExisting, const char* lpNew, unsigned long dwFlags);
__declspec(dllimport) unsigned long __stdcall GetLastError(void);
__declspec(dllimport) int __stdcall CharToOemA(const char* lpszSrc, char* lpszDst);
__declspec(dllimport) int __stdcall OemToCharA(const char* lpszSrc, char* lpszDst);
__declspec(dllimport) unsigned long __stdcall CharLowerBuffA(char* lpsz, unsigned long cchLength);
__declspec(dllimport) unsigned long __stdcall CharUpperBuffA(char* lpsz, unsigned long cchLength);
__declspec(dllimport) int __stdcall IsDBCSLeadByte(unsigned char TestChar);
__declspec(dllimport) unsigned int __stdcall GetMenuState(void* h, unsigned int id, unsigned int f);
__declspec(dllimport) void* __stdcall GetModuleHandleA(const char* lpModuleName);
__declspec(dllimport) void* __stdcall FindResourceA(void* hModule, const char* lpName, const char* lpType);
__declspec(dllimport) unsigned long __stdcall SizeofResource(void* hModule, void* hResInfo);
__declspec(dllimport) void* __stdcall LoadResource(void* hModule, void* hResInfo);
__declspec(dllimport) void* __stdcall LockResource(void* hResData);
__declspec(dllimport) int __stdcall LoadStringA(void* hInstance, unsigned int uID, char* lpBuffer, int cchBufferMax);
#define RT_RCDATA MAKEINTRESOURCE(10)
#define MAKEINTRESOURCEA(i) ((const char*)((unsigned long long)(i)))
#define RT_RCDATA MAKEINTRESOURCEA(10)

/* === Common dialog API declarations (comdlg32.dll) === */
#define MAX_PATH 260

typedef struct {
    unsigned long lStructSize;
    void* hwndOwner;
    void* hInstance;
    const char* lpstrFilter;
    char* lpstrCustomFilter;
    unsigned long nMaxCustFilter;
    unsigned long nFilterIndex;
    char* lpstrFile;
    unsigned long nMaxFile;
    char* lpstrFileTitle;
    unsigned long nMaxFileTitle;
    const char* lpstrInitialDir;
    const char* lpstrTitle;
    unsigned long Flags;
    unsigned short nFileOffset;
    unsigned short nFileExtension;
    const char* lpstrDefExt;
    void* lCustData;
    void* lpfnHook;
    const char* lpTemplateName;
} OPENFILENAMEA;

typedef struct {
    unsigned long lStructSize;
    void* hwndOwner;
    void* hInstance;
    unsigned long rgbResult;
    unsigned char* lpCustColors;
    unsigned long Flags;
    void* lCustData;
    void* lpfnHook;
    const char* lpTemplateName;
} CHOOSECOLORA;

typedef struct {
    unsigned long lStructSize;
    void* hwndOwner;
    void* hDC;
    char* lpLogFont;
    int iPointSize;
    unsigned long Flags;
    unsigned long rgbColors;
    void* lCustData;
    void* lpfnHook;
    const char* lpTemplateName;
    void* hInstance;
    char* lpszStyle;
    unsigned short nFontType;
    int nSizeMin;
    int nSizeMax;
} CHOOSEFONTA;

__declspec(dllimport) int __stdcall GetSaveFileNameA(OPENFILENAMEA* lpofn);
__declspec(dllimport) int __stdcall GetOpenFileNameA(OPENFILENAMEA* lpofn);

/* SHBrowseForFolder (shell32) */
typedef struct {
    void* hwndOwner;
    void* pidlRoot;
    char* pszDisplayName;
    const char* lpszTitle;
    unsigned long ulFlags;
    void* lpfn;
    long long lParam;
    int iImage;
} BROWSEINFOA;

__declspec(dllimport) void* __stdcall SHBrowseForFolderA(BROWSEINFOA* lpbi);
__declspec(dllimport) int __stdcall SHGetPathFromIDListA(void* pidl, char* pszPath);
__declspec(dllimport) int __stdcall ChooseColorA(CHOOSECOLORA* lpcc);
__declspec(dllimport) int __stdcall ChooseFontA(CHOOSEFONTA* lpcf);

#define OFN_OVERWRITEPROMPT 0x00000002
#define OFN_FILEMUSTEXIST 0x00001000
#define OFN_PATHMUSTEXIST 0x00000800
#define CC_RGBINIT 0x00000001
#define CF_SCREENFONTS 0x00000001
#define CF_INITTOLOGFONTSTRUCT 0x00000040

__declspec(dllimport) int __stdcall EnableMenuItem(void* h, unsigned int id, unsigned int f);
__declspec(dllimport) int __stdcall CheckMenuItem(void* h, unsigned int id, unsigned int f);
__declspec(dllimport) int __stdcall GetMenuStringA(void* h, unsigned int id, char* buf, int max, unsigned int f);
__declspec(dllimport) int __stdcall ModifyMenuA(void* h, unsigned int id, unsigned int f, unsigned int newid, const char* txt);
__declspec(dllimport) int __stdcall GetDiskFreeSpaceExA(char* lpDir, unsigned long long* lpFreeAvail, unsigned long long* lpTotalBytes, unsigned long long* lpTotalFree);
/* Memory status for FRE() */
typedef struct _MEMORYSTATUSEX {
    unsigned long dwLength;
    unsigned long dwMemoryLoad;
    unsigned long long ullTotalPhys;
    unsigned long long ullAvailPhys;
    unsigned long long ullTotalPageFile;
    unsigned long long ullAvailPageFile;
    unsigned long long ullTotalVirtual;
    unsigned long long ullAvailVirtual;
    unsigned long long ullAvailExtendedVirtual;
} MEMORYSTATUSEX;
__declspec(dllimport) int __stdcall GlobalMemoryStatusEx(MEMORYSTATUSEX* lpBuffer);
/* Win32 API for crash handling */
__declspec(dllimport) void* __stdcall AddVectoredExceptionHandler(unsigned long First, void* Handler);
__declspec(dllimport) void __stdcall ExitProcess(unsigned int uExitCode);
__declspec(dllimport) int __stdcall MessageBoxA(void* hWnd, const char* lpText, const char* lpCaption, unsigned int uType);
__declspec(dllimport) int __stdcall CopyFileA(const char* lpExistingFileName, const char* lpNewFileName, int bFailIfExists);
__declspec(dllimport) int __stdcall DeleteFileA(const char* lpFileName);
/* Find-file family (forward declared so pb_kill can use them; the batch-24
   block later in this file is guarded by PB_B24_DEFS and will be skipped). */
#define PB_B24_DEFS
__declspec(dllimport) void* __stdcall FindFirstFileA(const char* lpFileName, char* lpFindFileData);
__declspec(dllimport) int __stdcall FindNextFileA(void* hFindFile, char* lpFindFileData);
__declspec(dllimport) int __stdcall FindClose(void* hFindFile);
__declspec(dllimport) unsigned long __stdcall GetLastError(void);
/* OBJRESULT$ descriptive text: measured PBWin returns the Windows system
 * message for the hResult (FormatMessage FROM_SYSTEM), e.g. 0 -> "The
 * operation completed successfully." */
__declspec(dllimport) unsigned long __stdcall FormatMessageA(
    unsigned long dwFlags, const void* lpSource, unsigned long dwMessageId,
    unsigned long dwLanguageId, char* lpBuffer, unsigned long nSize,
    const void* Arguments);
__declspec(dllimport) void __stdcall LocalFree(void* hMem);
__declspec(dllimport) unsigned long long __stdcall GetTickCount64(void);
__declspec(dllimport) int __stdcall SetFileAttributesA(const char* lpFileName, unsigned long dwFileAttributes);
__declspec(dllimport) void* __stdcall GetCurrentProcess(void);
__declspec(dllimport) unsigned long __stdcall GetPriorityClass(void* hProcess);
/* UTF-16 conversion for GET$$/PUT$$ (kernel32) */
__declspec(dllimport) int __stdcall MultiByteToWideChar(unsigned int CodePage, unsigned long dwFlags, const char* lpMultiByteStr, int cbMultiByte, short* lpWideCharStr, int cchWideChar);
__declspec(dllimport) int __stdcall WideCharToMultiByte(unsigned int CodePage, unsigned long dwFlags, const short* lpWideCharStr, int cchWideChar, char* lpMultiByteStr, int cbMultiByte, const char* lpDefaultChar, int* lpUsedDefaultChar);
#define PB_CP_ACP 0
__declspec(dllimport) int __stdcall SetPriorityClass(void* hProcess, unsigned long dwPriorityClass);
/* Win32 clipboard (user32) */
__declspec(dllimport) int __stdcall SystemParametersInfoA(unsigned int uiAction, unsigned int uiParam, void* pvParam, unsigned int fWinIni);
__declspec(dllimport) void* __stdcall GetDC(void* hWnd);
__declspec(dllimport) int __stdcall GetDeviceCaps(void* hdc, int nIndex);
__declspec(dllimport) int __stdcall ReleaseDC(void* hWnd, void* hDC);
__declspec(dllimport) int __stdcall MulDiv(int nNumber, int nNumerator, int nDenominator);
__declspec(dllimport) void* __stdcall CreateFontA(int cHeight, int cWidth, int cEscapement, int cOrientation, int cWeight, unsigned char bItalic, unsigned char bUnderline, unsigned char bStrikeOut, unsigned char iCharSet, unsigned char iOutPrecision, unsigned char iClipPrecision, unsigned char iQuality, unsigned char iPitchAndFamily, const char* pszFaceName);
__declspec(dllimport) int __stdcall DeleteObject(void* hObject);
__declspec(dllimport) void* __stdcall ImageList_Create(int cx, int cy, unsigned int flags, int cInitial, int cGrow);
__declspec(dllimport) int __stdcall ImageList_GetImageCount(void* himl);
__declspec(dllimport) int __stdcall ImageList_Destroy(void* himl);
__declspec(dllimport) int __stdcall ImageList_Add(void* himl, void* hbmImage, void* hbmMask);
__declspec(dllimport) int __stdcall ImageList_AddMasked(void* himl, void* hbmImage, unsigned long crMask);
__declspec(dllimport) int __stdcall ImageList_ReplaceIcon(void* himl, int i, void* hicon);
__declspec(dllimport) int __stdcall ImageList_SetOverlayImage(void* himl, int iImage, int iOverlay);
__declspec(dllimport) int __stdcall ImageList_GetIconSize(void* himl, int* cx, int* cy);
__declspec(dllimport) int __stdcall DestroyIcon(void* hIcon);
__declspec(dllimport) void* __stdcall GetStdHandle(unsigned int nStdHandle);
__declspec(dllimport) int __stdcall SetConsoleTextAttribute(void* hConsoleOutput, unsigned short wAttributes);
__declspec(dllimport) void* __stdcall CreateMenu(void);
__declspec(dllimport) void* __stdcall CreatePopupMenu(void);
__declspec(dllimport) int __stdcall AppendMenuA(void* hMenu, unsigned int uFlags, uintptr_t uIDNewItem, const char* lpNewItem);
__declspec(dllimport) int __stdcall DeleteMenu(void* hMenu, unsigned int uPosition, unsigned int uFlags);
__declspec(dllimport) int __stdcall DestroyMenu(void* hMenu);
typedef struct {
    unsigned int cbSize;
    unsigned int fMask;
    unsigned int fType;
    unsigned int fState;
    unsigned int wID;
    void* hSubMenu;
    void* hbmpChecked;
    void* hbmpUnchecked;
    uintptr_t dwItemData;
    char* dwTypeData;
    unsigned int cch;
    void* hbmpItem;
} PB_MENUITEMINFOA;
__declspec(dllimport) unsigned long __stdcall GetLastError(void);
__declspec(dllimport) int __stdcall GetMenuItemInfoA(void* hMenu, unsigned int uItem, int fByPosition, PB_MENUITEMINFOA* lpmii);
__declspec(dllimport) int __stdcall SetMenuDefaultItem(void* hMenu, unsigned int uItem, unsigned int fByPosition);
__declspec(dllimport) int __stdcall SetMenu(void* hWnd, void* hMenu);
__declspec(dllimport) int __stdcall DrawMenuBar(void* hWnd);
__declspec(dllimport) int __stdcall SetWindowPos(void* hWnd, void* hWndInsertAfter,
                                                 int x, int y, int cx, int cy, unsigned int uFlags);
__declspec(dllimport) void* __stdcall GetMenu(void* hWnd);
__declspec(dllimport) int __stdcall GetSystemMetrics(int nIndex);
__declspec(dllimport) int __stdcall TrackPopupMenu(void* hMenu, unsigned int uFlags, int x, int y, int nReserved, void* hWnd, const void* prcRect);
__declspec(dllimport) void* __stdcall GetActiveWindow(void);
__declspec(dllimport) int __stdcall InvalidateRect(void* hWnd, const void* lpRect, int bErase);
__declspec(dllimport) int __stdcall UpdateWindow(void* hWnd);
__declspec(dllimport) void* __stdcall CreateCompatibleDC(void* hdc);
__declspec(dllimport) void* __stdcall CreateDIBSection(void* hdc, const void* pbmi, unsigned int usage, void** ppvBits, void* hSection, unsigned long offset);
__declspec(dllimport) int __stdcall DeleteObject(void* hObject);
__declspec(dllimport) int __stdcall DeleteDC(void* hdc);
__declspec(dllimport) void* __stdcall CreateDCA(const char* lpszDriver, const char* lpszDevice, const char* lpszOutput, const void* lpInitData);
__declspec(dllimport) int __stdcall GetDefaultPrinterA(char* pszBuffer, unsigned int* pcchBuffer);
__declspec(dllimport) void* __stdcall SelectObject(void* hdc, void* hObject);
__declspec(dllimport) void* __stdcall CreateSolidBrush(unsigned long color);
__declspec(dllimport) int __stdcall FillRect(void* hdc, const void* lprc, void* hbr);
__declspec(dllimport) void* __stdcall CreatePen(int style, int width, unsigned long color);
__declspec(dllimport) void* __stdcall GetStockObject(int fnObject);
__declspec(dllimport) int __stdcall MoveToEx(void* hdc, int x, int y, void* lppt);
__declspec(dllimport) int __stdcall GetCurrentPositionEx(void* hdc, void* lppt);
__declspec(dllimport) int __stdcall GetStretchBltMode(void* hdc);
__declspec(dllimport) int __stdcall SetStretchBltMode(void* hdc, int mode);
__declspec(dllimport) int __stdcall GetROP2(void* hdc);
__declspec(dllimport) int __stdcall SetROP2(void* hdc, int rop2);
__declspec(dllimport) int __stdcall LineTo(void* hdc, int x, int y);
__declspec(dllimport) int __stdcall Rectangle(void* hdc, int left, int top, int right, int bottom);
__declspec(dllimport) int __stdcall Ellipse(void* hdc, int left, int top, int right, int bottom);
__declspec(dllimport) int __stdcall Arc(void* hdc, int x1, int y1, int x2, int y2, int x3, int y3, int x4, int y4);
__declspec(dllimport) int __stdcall Pie(void* hdc, int x1, int y1, int x2, int y2, int x3, int y3, int x4, int y4);
__declspec(dllimport) int __stdcall Polyline(void* hdc, const void* apt, int cpt);
__declspec(dllimport) int __stdcall FloodFill(void* hdc, int x, int y, unsigned long color);
__declspec(dllimport) unsigned long __stdcall GetPixel(void* hdc, int x, int y);
__declspec(dllimport) int __stdcall BitBlt(void* hdcDest, int xDest, int yDest, int w, int h, void* hdcSrc, int xSrc, int ySrc, unsigned long rop);
__declspec(dllimport) int __stdcall StretchBlt(void* hdcDest, int xDest, int yDest, int wDest, int hDest, void* hdcSrc, int xSrc, int ySrc, int wSrc, int hSrc, unsigned long rop);
__declspec(dllimport) int __stdcall Polygon(void* hdc, const long* pts, int count);
__declspec(dllimport) void* __stdcall LoadImageA(void* hinst, const char* name, unsigned int type, int cx, int cy, unsigned int fuLoad);
__declspec(dllimport) int __stdcall GetTextExtentPoint32A(void* hdc, const char* str, int count, void* size);
__declspec(dllimport) unsigned int __stdcall SetPixel(void* hdc, int x, int y, unsigned int color);
__declspec(dllimport) unsigned long __stdcall SetTextColor(void* hdc, unsigned long color);
__declspec(dllimport) int __stdcall SetBkMode(void* hdc, int mode);
__declspec(dllimport) unsigned int __stdcall SetTextAlign(void* hdc, unsigned int fmode);
__declspec(dllimport) int __stdcall TextOutA(void* hdc, int x, int y, const char* str, int count);
typedef struct { int x; int y; } pb_pt;
__declspec(dllimport) int __stdcall GetObjectA(void* hObject, int nCount, void* lpObject);
__declspec(dllimport) int __stdcall GetDIBits(void* hdc, void* hbm, unsigned int start, unsigned int cLines, void* lpvBits, void* lpbmi, unsigned int usage);
__declspec(dllimport) int __stdcall SetDIBits(void* hdc, void* hbm, unsigned int start, unsigned int cLines, const void* lpvBits, const void* lpbmi, unsigned int usage);
__declspec(dllimport) int __stdcall GetClipBox(void* hdc, void* lprc);
__declspec(dllimport) int __stdcall IntersectClipRect(void* hdc, int left, int top, int right, int bottom);
__declspec(dllimport) int __stdcall GetViewportOrgEx(void* hdc, void* lppt);
__declspec(dllimport) int __stdcall SetViewportOrgEx(void* hdc, int x, int y, void* lppt);
__declspec(dllimport) int __stdcall SetMapMode(void* hdc, int fnMapMode);
__declspec(dllimport) int __stdcall SetWindowExtEx(void* hdc, int x, int y, void* lpsz);
__declspec(dllimport) int __stdcall SetViewportExtEx(void* hdc, int x, int y, void* lpsz);







#define PB_FW_NORMAL 400
#define PB_FW_BOLD 700

__declspec(dllimport) void* __stdcall GlobalFree(void* hMem);
__declspec(dllimport) unsigned long long __stdcall GlobalSize(void* hMem);
__declspec(dllimport) void* __stdcall LoadCursorA(void* hInstance, const char* lpCursorName);
__declspec(dllimport) void* __stdcall SetCursor(void* hCursor);
__declspec(dllimport) int __stdcall ShowCursor(int bShow);
__declspec(dllimport) int __stdcall OpenClipboard(void* hWndNewOwner);
__declspec(dllimport) int __stdcall CloseClipboard(void);
__declspec(dllimport) int __stdcall EmptyClipboard(void);
__declspec(dllimport) void* __stdcall SetClipboardData(unsigned int uFormat, void* hMem);
__declspec(dllimport) void* __stdcall GetClipboardData(unsigned int uFormat);
__declspec(dllimport) int __stdcall CharToOemA(const char* src, char* dst);
__declspec(dllimport) int __stdcall OemToCharA(const char* src, char* dst);
__declspec(dllimport) void* __stdcall GlobalAlloc(unsigned int uFlags, unsigned long dwBytes);
__declspec(dllimport) void* __stdcall GlobalLock(void* hMem);
__declspec(dllimport) int __stdcall GlobalUnlock(void* hMem);
/* WinSock2 (HOST ADDR / HOST NAME) */
typedef struct _pb_hostent {
    char* h_name;
    char** h_aliases;
    int h_addrtype;
    int h_length;
    char** h_addr_list;
} pb_hostent;
typedef struct _pb_in_addr { unsigned long s_addr; } pb_in_addr;
__declspec(dllimport) pb_hostent* __stdcall gethostbyname(const char* name);
__declspec(dllimport) pb_hostent* __stdcall gethostbyaddr(const char* addr, int len, int type);
__declspec(dllimport) char* __stdcall inet_ntoa(pb_in_addr in);
__declspec(dllimport) int __stdcall gethostname(char* name, int namelen);
__declspec(dllimport) int __stdcall WSAStartup(unsigned short wVersionRequested, void* lpWSAData);
__declspec(dllimport) int __stdcall WSACleanup(void);
typedef unsigned long DWORD;
/* COMM serial + THREAD (batch 21) */
typedef void* HANDLE;
#define PB_INVALID_HANDLE ((void*)(long long)-1)
__declspec(dllimport) void* __stdcall CreateFileA(const char* lpFileName, unsigned long dwDesiredAccess, unsigned long dwShareMode, void* lpSecurityAttributes, unsigned long dwCreationDisposition, unsigned long dwFlagsAndAttributes, void* hTemplateFile);
__declspec(dllimport) int __stdcall GetCommState(void* hFile, void* lpDCB);
__declspec(dllimport) int __stdcall SetCommState(void* hFile, void* lpDCB);
__declspec(dllimport) int __stdcall SetCommTimeouts(void* hFile, void* lpCommTimeouts);
__declspec(dllimport) int __stdcall ReadFile(void* hFile, void* lpBuffer, unsigned long nNumberOfBytesToRead, unsigned long* lpNumberOfBytesRead, void* lpOverlapped);
__declspec(dllimport) int __stdcall WriteFile(void* hFile, const void* lpBuffer, unsigned long nNumberOfBytesToWrite, unsigned long* lpNumberOfBytesWritten, void* lpOverlapped);
__declspec(dllimport) int __stdcall CloseHandle(void* hObject);
__declspec(dllimport) int __stdcall FlushFileBuffers(void* hFile);
__declspec(dllimport) int __stdcall EscapeCommFunction(void* hFile, unsigned long dwFunc);
__declspec(dllimport) int __stdcall GetCommModemStatus(void* hFile, unsigned long* lpModemStat);
__declspec(dllimport) int __stdcall ClearCommError(void* hFile, unsigned long* lpErrors, void* lpStat);
__declspec(dllimport) int __stdcall GetCommProperties(void* hFile, void* lpCommProp);
__declspec(dllimport) int __stdcall SetupComm(void* hFile, unsigned long dwInQueue, unsigned long dwOutQueue);
__declspec(dllimport) void* __stdcall CreateThread(void* lpThreadAttributes, unsigned long dwStackSize, void* lpStartAddress, void* lpParameter, unsigned long dwCreationFlags, unsigned long* lpThreadId);
__declspec(dllimport) unsigned long __stdcall SuspendThread(void* hThread);
__declspec(dllimport) unsigned long __stdcall ResumeThread(void* hThread);
__declspec(dllimport) int __stdcall TerminateThread(void* hThread, unsigned long dwExitCode);
__declspec(dllimport) int __stdcall GetExitCodeThread(void* hThread, unsigned long* lpExitCode);
__declspec(dllimport) int __stdcall GetThreadPriority(void* hThread);
__declspec(dllimport) int __stdcall SetThreadPriority(void* hThread, int nPriority);
/* Slim Reader/Writer (SRW) locks for THREADSAFE procedures (kernel32).
   An SRWLOCK is one pointer-sized object that is zero-initialized
   (SRWLOCK_INIT == 0), so a global zero-initialized variable needs no
   explicit Initialize call.  THREADSAFE uses the exclusive mode. */
__declspec(dllimport) void __stdcall AcquireSRWLockExclusive(void* SRWLock);
__declspec(dllimport) void __stdcall ReleaseSRWLockExclusive(void* SRWLock);
#endif

/* ===== Debug/crash reporting ===== */
static const char* pb_debug_current_func = "unknown";
int pb_debug_line = 0; /* Current source line � updated by codegen */
static int pb_debug_enabled = 0;
static int pb_debug_verbose = 0; /* Set to 1 for verbose modal/function logging */
static FILE* pb_debug_log = NULL;

/* Circular buffer for recent call trace */
#define PB_TRACE_SIZE 50
static const char* pb_trace_buf[PB_TRACE_SIZE];
static int pb_trace_idx = 0;
static int pb_modal_count = 0;

/* Call this from codegen at the start of each function for debug mode */
void pb_debug_enter(const char* func_name) {
    pb_debug_current_func = func_name;
    pb_trace_buf[pb_trace_idx % PB_TRACE_SIZE] = func_name;
    pb_trace_idx++;
    if (pb_debug_log) {
        fprintf(pb_debug_log, "[ENTER] %s\n", func_name);
        fflush(pb_debug_log);
    }
}

void pb_debug_init(void) {
    pb_debug_enabled = 1;
    pb_debug_log = fopen("pb_debug.log", "w");
    if (pb_debug_log) {
        fprintf(pb_debug_log, "=== PB64Basic Debug Log ===\n");
        fflush(pb_debug_log);
    }
    /* Check environment variable for verbose debug mode */
    const char* verbose = getenv("PB_DEBUG_VERBOSE");
    if (verbose && (verbose[0] == '1' || verbose[0] == 'Y' || verbose[0] == 'y')) {
        pb_debug_verbose = 1;
        if (pb_debug_log) {
            fprintf(pb_debug_log, "[DEBUG] Verbose mode enabled via PB_DEBUG_VERBOSE\n");
            fflush(pb_debug_log);
        }
    }
}

/* ===== Modal/Dialog debug logging ===== */

/* Log modal dialog text for test debugging.
 * Called from PB code (ui_stubs or ui.inc) when debug mode is enabled.
 * Writes to pb_debug.log with modal type, title, and full text content.
 * When tests hang, this log shows the last modal that was displayed. */
void pb_debug_modal(const char* modal_type, const char* title, const char* text) {
    pb_modal_count++;
    if (pb_debug_log) {
        fprintf(pb_debug_log, "[MODAL #%d] type=%s title=\"%s\" text=\"%.500s\"\n",
                pb_modal_count,
                modal_type ? modal_type : "(null)",
                title ? title : "(no title)",
                text ? text : "(no text)");
        fflush(pb_debug_log);
    }
    /* Also write to stderr for immediate visibility in test output */
    fprintf(stderr, "[MODAL #%d] type=%s title=\"%s\"\n",
            pb_modal_count,
            modal_type ? modal_type : "(null)",
            title ? title : "(no title)");
}

/* Log a generic debug message from PB code */
void pb_debug_log_msg(const char* category, const char* message) {
    if (pb_debug_log) {
        fprintf(pb_debug_log, "[%s] in %s (line %d): %s\n",
                category ? category : "DEBUG",
                pb_debug_current_func, pb_debug_line,
                message ? message : "");
        fflush(pb_debug_log);
    }
}

#ifdef _WIN32
/* Vectored exception handler: logs fatal exceptions (0xC0000000 mask) to
   stderr.  Informational codes (0x4xxxxxxx, 0xE06D7363 C++ throw inside
   MessageBoxA and friends) are normal runtime noise and are ignored. */
static long __stdcall pb_crash_handler(void* exception_pointers) {
    /* VEH: only fatal exceptions log (informational/debug codes are normal
       runtime noise - e.g. 0xE06D7363 C++ throw inside MessageBoxA). */
    unsigned long long* pp = (unsigned long long*)exception_pointers;
    unsigned long long* er = pp ? (unsigned long long*)pp[0] : 0;
    unsigned long code = er ? (unsigned long)(er[0] & 0xFFFFFFFFu) : 0;
    if ((code & 0xF0000000) != 0xC0000000) return 0;
    unsigned long long addr = er ? er[2] : 0;
    char b[192];
    int n = sprintf(b, "CRASH code=%08X addr=%llX\n", code, addr);
    _write(2, b, n);
    return 0; /* EXCEPTION_CONTINUE_SEARCH */
}
#endif

/* Called from main() wrapper before PBMAIN */
void pb_install_crash_handler(void) {
#ifdef _WIN32
    AddVectoredExceptionHandler(1, pb_crash_handler);
    pb_debug_init();
#endif
}

int pb_str_cstr_len(const char* s) {
    return (int)strlen(s);
}

/* Forward declaration: BSTR byte length (NUL-safe), defined later. */
int pb_str_len(const char* s);

/* Public BSTR allocation wrapper — called from LLVM IR codegen (cdecl) */
char* pb_bstr_alloc(const char* src, unsigned int len) {
#ifdef _WIN32
    return SysAllocStringByteLen(src, len);
#else
    char* buf = (char*)malloc(len + 1);
    if (src) memcpy(buf, src, len);
    buf[len] = '\0';
    return buf;
#endif
}

/* UDT-to-string byte copy (CHW LET statement with Types): copy a UDT's raw
   bytes into a BSTR of exactly the struct size, preserving embedded NULs. */
char* pb_bstr_from_bytes(const void* data, unsigned int len) {
    return pb_bstr_alloc((const char*)data, len);
}

/* LEFT$ on a byte (ANSI) string.  CHW LEFT_function.htm: n is a long integer;
   n >= LEN returns the whole string, n == 0 returns empty, and a negative n is
   interpreted as LEN(s) - ABS(n).  Measured against PBWin10 at the overshoot
   point (ABS(n) > LEN): the effective count goes negative and PBWin returns the
   WHOLE string (e.g. n=-11 on a 10-char string), while n=-LEN (effective 0)
   still returns empty.  This effective-count rule reproduces both cases. */
char* pb_bstr_left(const char* src, long long n) {
    static const char empty[] = "";
    if (!src) src = empty;
    long long len = (long long)strlen(src);
    if (n < 0) {
        n = len + n;              /* LEN(s) - ABS(n) */
        if (n < 0) n = len;       /* overshoot: whole string */
    }
    if (n > len) n = len;
    if (n <= 0) return pb_bstr_alloc(empty, 0);
    return pb_bstr_alloc(src, (unsigned int)n);
}

/* RIGHT$ on a byte (ANSI) string.  CHW RIGHT_function.htm: the same effective-
   count rule as LEFT$; the selected bytes start at LEN(s)-n. */
char* pb_bstr_right(const char* src, long long n) {
    static const char empty[] = "";
    if (!src) src = empty;
    long long len = (long long)strlen(src);
    if (n < 0) {
        n = len + n;
        if (n < 0) n = len;
    }
    if (n > len) n = len;
    if (n <= 0) return pb_bstr_alloc(empty, 0);
    return pb_bstr_alloc(src + (len - n), (unsigned int)n);
}

/* LTRIM$ (CHW LTRIM_function.htm).  has_any=0 and a NULL/empty match: remove
   leading spaces (the one-argument form).  has_any=1: match is a list of single
   characters; remove leading characters that occur anywhere in the list.
   has_any=0 with a match: match is a whole substring; remove leading occurrences
   of that exact substring.  Matching is case sensitive. */
char* pb_ltrim(const char* src, int has_any, const char* match) {
    static const char empty[] = "";
    if (!src) src = empty;
    const char* p = src;
    if (!match || !*match) {
        while (*p == ' ') p++;
    } else if (has_any) {
        size_t mlen = strlen(match);
        while (*p) {
            int found = 0;
            for (size_t k = 0; k < mlen; k++) {
                if (match[k] == *p) { found = 1; break; }
            }
            if (!found) break;
            p++;
        }
    } else {
        size_t mlen = strlen(match);
        for (;;) {
            int starts = 1;
            for (size_t k = 0; k < mlen; k++) {
                if (p[k] != match[k]) { starts = 0; break; }
            }
            if (!starts) break;
            p += mlen;
        }
    }
    return pb_bstr_alloc(p, (unsigned int)strlen(p));
}

/* RTRIM$ (CHW RTRIM_function.htm): mirror of LTRIM$ from the right.  `end` is
   the exclusive kept boundary; trailing spaces / whole-substring occurrences /
   single-character set members are removed until the boundary no longer matches.
   Case sensitive. */
char* pb_rtrim(const char* src, int has_any, const char* match) {
    static const char empty[] = "";
    if (!src) src = empty;
    size_t end = strlen(src);
    if (!match || !*match) {
        while (end > 0 && src[end - 1] == ' ') end--;
    } else if (has_any) {
        size_t mlen = strlen(match);
        while (end > 0) {
            char c = src[end - 1];
            int found = 0;
            for (size_t k = 0; k < mlen; k++) {
                if (match[k] == c) { found = 1; break; }
            }
            if (!found) break;
            end--;
        }
    } else {
        size_t mlen = strlen(match);
        for (;;) {
            if (end < mlen) break;
            int ends = 1;
            for (size_t k = 0; k < mlen; k++) {
                if (src[end - mlen + k] != match[k]) { ends = 0; break; }
            }
            if (!ends) break;
            end -= mlen;
        }
    }
    return pb_bstr_alloc(src, (unsigned int)end);
}

/* COMMAND$(ArgNum) - batch258 C batch, CHW COMMAND$ function:
   n<=0 -> full trailer (program name excluded, quotes kept verbatim);
   n>0  -> nth parsed argument (blank-space delimited, double quotes
           stripped, zero-length quoted strings ("") ignored entirely);
   n out of range -> zero-length string. */
__declspec(dllimport) char* __stdcall GetCommandLineA(void);
char* pb_command_arg(long long n) {
    char* cmd = GetCommandLineA();
    if (!cmd) return pb_bstr_alloc("", 0);
    const char* p = cmd;
    /* skip the program name (may itself be quoted) */
    if (*p == '"') {
        p++;
        while (*p && *p != '"') p++;
        if (*p == '"') p++;
    } else {
        while (*p && *p != ' ') p++;
    }
    while (*p == ' ') p++;
    if (n <= 0) {
        /* full trailer, kept verbatim (leading blanks already skipped) */
        return pb_bstr_alloc(p, (unsigned int)strlen(p));
    }
    /* parse the nth argument */
    long long count = 0;
    int in_quote = 0;
    int cur_len = 0;
    char buf[4096];
    while (*p) {
        char c = *p;
        if (in_quote) {
            if (c == '"') in_quote = 0;
            else if (cur_len < 4095) buf[cur_len++] = c;
        } else if (c == '"') {
            in_quote = 1;
        } else if (c == ' ') {
            if (cur_len > 0) {
                count++;
                if (count == n) return pb_bstr_alloc(buf, (unsigned int)cur_len);
                cur_len = 0;
            }
        } else if (cur_len < 4095) {
            buf[cur_len++] = c;
        }
        p++;
    }
    if (cur_len > 0) {
        count++;
        if (count == n) return pb_bstr_alloc(buf, (unsigned int)cur_len);
    }
    return pb_bstr_alloc("", 0);
}


/* MKx family: numeric -> binary ANSI string (pb_mkbyt already exists) */
char* pb_mkint(int v) {
    short s = (short)v;
    return pb_bstr_alloc((const char*)&s, 2);      /* MKI$, MKWRD$ */
}
char* pb_mklong(int v) {
    long l = (long)v;
    return pb_bstr_alloc((const char*)&l, 4);      /* MKL$, MKDWD$ */
}
char* pb_mkquad(long long v) {
    return pb_bstr_alloc((const char*)&v, 8);      /* MKQ$, MKCUR$, MKCUX$ */
}
char* pb_mksingle(float v) {
    return pb_bstr_alloc((const char*)&v, 4);      /* MKS$ */
}
/* Batch 41: string utilities. */
char* pb_clip(char* mode, char* s, long long start, long long count) {
    size_t n = (size_t)pb_str_len(s);
    long long from = 0, to = (long long)n;
    if (strcmp(mode, "LEFT") == 0) { from = (count > (long long)n) ? (long long)n : count; }
    else if (strcmp(mode, "RIGHT") == 0) { to = n - ((count > (long long)n) ? (long long)n : count); if (to < 0) to = 0; }
    else { /* MID: remove from start (1-based), count chars */
        long long s0 = start - 1; if (s0 < 0) s0 = 0; if (s0 > (long long)n) s0 = n;
        long long e0 = s0 + (count < 0 ? 0 : count); if (e0 > (long long)n) e0 = n;
        size_t ln = (size_t)(s0 + (n - e0));
        char* buf = (char*)malloc(ln + 1);
        memcpy(buf, s, (size_t)s0);
        memcpy(buf + s0, s + e0, n - (size_t)e0);
        buf[ln] = 0;
        return pb_bstr_alloc(buf, (unsigned int)ln);
    }
    if (from >= to) return pb_bstr_alloc("", 0);
    return pb_bstr_alloc(s + from, (unsigned int)(to - from));
}
char* pb_wrap(char* s, char* l, char* r) {
    size_t nl = (size_t)pb_str_len(l), nr = (size_t)pb_str_len(r), ns = (size_t)pb_str_len(s);
    char* buf = (char*)malloc(nl + ns + nr + 1);
    memcpy(buf, l, nl); memcpy(buf + nl, s, ns); memcpy(buf + nl + ns, r, nr);
    buf[nl + ns + nr] = 0;
    return pb_bstr_alloc(buf, (unsigned int)(nl + ns + nr));
}
char* pb_unwrap(char* s, char* l, char* r) {
    size_t nl = (size_t)pb_str_len(l), nr = (size_t)pb_str_len(r), ns = (size_t)pb_str_len(s);
    size_t from = 0, to = ns;
    if (nl && ns >= nl && memcmp(s, l, nl) == 0) from = nl;
    if (nr && ns - from >= nr && memcmp(s + ns - nr, r, nr) == 0) to = ns - nr;
    if (to < from) to = from;
    return pb_bstr_alloc(s + from, (unsigned int)(to - from));
}
/* CHW SHRINK$ function (SHRINK$_function.htm).
 * First form (mask == NULL/empty): collapse runs of whitespace
 *   (space/tab/CR/LF) to a single space; trim leading/trailing whitespace.
 * Second form (mask != NULL): Mask$ defines one or more delimiter chars.
 *   All leading/trailing mask chars are removed; every run of one-or-more
 *   mask chars is collapsed to the first char of Mask$.  The old code only
 *   collapsed the hard-coded whitespace set even when a custom Mask$ was
 *   supplied, so SHRINK$("a,,b", ",") was returned unchanged. */
char* pb_shrink(char* s, char* mask) {
    size_t n = (size_t)pb_str_len(s);
    char* buf = (char*)malloc(n + 1);
    size_t o = 0;
    unsigned char delim[256];
    char sep;
    int first = 1;
    memset(delim, 0, sizeof(delim));
    if (mask && mask[0]) {
        size_t mi = 0;
        while (mask[mi]) delim[(unsigned char)mask[mi++]] = 1;
        sep = mask[0];
    } else {
        delim[(unsigned char)' '] = 1;
        delim[(unsigned char)'\t'] = 1;
        delim[(unsigned char)'\r'] = 1;
        delim[(unsigned char)'\n'] = 1;
        sep = ' ';
    }
    for (size_t i = 0; i < n; i++) {
        if (delim[(unsigned char)s[i]]) {
            while (i + 1 < n && delim[(unsigned char)s[i+1]]) i++;
            if (!first) buf[o++] = sep;
        } else {
            first = 0;
            buf[o++] = s[i];
        }
    }
    while (o > 0 && buf[o-1] == sep) o--;
    buf[o] = 0;
    return pb_bstr_alloc(buf, (unsigned int)o);
}
char* pb_remove_string(char* main, char* match, int any_flag) {
    size_t n = (size_t)pb_str_len(main);
    size_t mlen = match ? (size_t)pb_str_len(match) : 0;
    char* buf = (char*)malloc(n + 1);
    size_t o = 0;
    if (mlen == 0) {
        memcpy(buf, main, n + 1);
        return pb_bstr_alloc(buf, (unsigned int)n);
    }
    if (any_flag) {
        for (size_t i = 0; i < n; i++) {
            int found = 0;
            for (size_t j = 0; j < mlen; j++) {
                if (main[i] == match[j]) { found = 1; break; }
            }
            if (!found) buf[o++] = main[i];
        }
    } else {
        size_t i = 0;
        while (i < n) {
            if (i + mlen <= n && memcmp(main + i, match, mlen) == 0) {
                i += mlen;
            } else {
                buf[o++] = main[i++];
            }
        }
    }
    buf[o] = 0;
    return pb_bstr_alloc(buf, (unsigned int)o);
}
char* pb_retain_string(char* main, char* match, int any_flag) {
    size_t n = (size_t)pb_str_len(main);
    size_t mlen = match ? (size_t)pb_str_len(match) : 0;
    char* buf = (char*)malloc(n + 1);
    size_t o = 0;
    if (mlen == 0) {
        buf[0] = 0;
        return pb_bstr_alloc(buf, 0);
    }
    if (any_flag) {
        for (size_t i = 0; i < n; i++) {
            for (size_t j = 0; j < mlen; j++) {
                if (main[i] == match[j]) { buf[o++] = main[i]; break; }
            }
        }
    } else {
        size_t i = 0;
        while (i < n) {
            if (i + mlen <= n && memcmp(main + i, match, mlen) == 0) {
                memcpy(buf + o, match, mlen);
                o += mlen;
                i += mlen;
            } else {
                i++;
            }
        }
    }
    buf[o] = 0;
    return pb_bstr_alloc(buf, (unsigned int)o);
}
/* LCASE$ / UCASE$ with optional ANSI (mode=0) / OEM (mode=1).
 * ANSI: CharLowerBuffA / CharUpperBuffA handle DBCS correctly.
 * OEM: DBCS pairs are preserved untouched; only ASCII letters are folded.
 * Verified against PBWin 10 on a 936 DBCS system (lm_lcase* probes). */
char* pb_lcase(char* s, int mode) {
    size_t n = (size_t)pb_str_len(s);
    char* buf = (char*)malloc(n + 1);
    memcpy(buf, s, n);
    buf[n] = 0;
    if (mode == 0) {
        CharLowerBuffA(buf, (unsigned long)n);
    } else {
        size_t i = 0;
        while (i < n) {
            unsigned char c = (unsigned char)buf[i];
            if (IsDBCSLeadByte(c)) {
                i += 2; /* keep the whole DBCS pair untouched */
            } else {
                if (c >= 'A' && c <= 'Z') buf[i] = (char)(c + 32);
                i++;
            }
        }
    }
    char* r = pb_bstr_alloc(buf, (unsigned int)n);
    free(buf);
    return r;
}

char* pb_ucase(char* s, int mode) {
    size_t n = (size_t)pb_str_len(s);
    char* buf = (char*)malloc(n + 1);
    memcpy(buf, s, n);
    buf[n] = 0;
    if (mode == 0) {
        CharUpperBuffA(buf, (unsigned long)n);
    } else {
        size_t i = 0;
        while (i < n) {
            unsigned char c = (unsigned char)buf[i];
            if (IsDBCSLeadByte(c)) {
                i += 2;
            } else {
                if (c >= 'a' && c <= 'z') buf[i] = (char)(c - 32);
                i++;
            }
        }
    }
    char* r = pb_bstr_alloc(buf, (unsigned int)n);
    free(buf);
    return r;
}

char* pb_mcase_string(char* s, int mode) {
    // MCASE$: first letter of each word is capitalized, rest forced lowercase.
    // A DBCS pair is treated as one letter and then resets the word boundary
    // (next_upper=1). OEM mode keeps DBCS pairs untouched. Verified against
    // PBWin 10 on a 936 DBCS system (lm_mcase/lm_mcase2 probes).
    size_t n = (size_t)pb_str_len(s);
    char* buf = (char*)malloc(n + 1);
    int next_upper = 1;
    size_t i = 0;
    while (i < n) {
        unsigned char c = (unsigned char)s[i];
        if (IsDBCSLeadByte(c) && i + 1 < n) {
            if (mode == 0) {
                char pair[3];
                pair[0] = s[i];
                pair[1] = s[i + 1];
                pair[2] = 0;
                if (next_upper) {
                    CharUpperBuffA(pair, 2);
                } else {
                    CharLowerBuffA(pair, 2);
                }
                buf[i] = pair[0];
                buf[i + 1] = pair[1];
            } else {
                buf[i] = s[i];
                buf[i + 1] = s[i + 1];
            }
            next_upper = 1;
            i += 2;
        } else if ((c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z')) {
            if (next_upper) {
                buf[i] = (c >= 'a' && c <= 'z') ? (char)(c - 32) : (char)c;
            } else {
                buf[i] = (c >= 'A' && c <= 'Z') ? (char)(c + 32) : (char)c;
            }
            next_upper = 0;
            i++;
        } else {
            buf[i] = s[i];
            next_upper = 1;
            i++;
        }
    }
    buf[n] = 0;
    char* r = pb_bstr_alloc(buf, (unsigned int)n);
    free(buf);
    return r;
}
char* pb_remain_string(char* main, char* match, long long start, int any_flag) {
    size_t n = (size_t)pb_str_len(main);
    size_t mlen = match ? (size_t)pb_str_len(match) : 0;
    if (mlen == 0 || n == 0) {
        return pb_bstr_alloc("", 0);
    }
    // Convert start to 0-based index
    long long s;
    if (start == 0) {
        return pb_bstr_alloc("", 0);
    } else if (start > 0) {
        s = start - 1;
    } else {
        s = (long long)n + start; // -1 means last char
    }
    if (s < 0) s = 0;
    if ((size_t)s >= n) {
        return pb_bstr_alloc("", 0);
    }
    // Search for match
    size_t found = (size_t)-1;
    if (any_flag) {
        for (size_t i = (size_t)s; i < n; i++) {
            for (size_t j = 0; j < mlen; j++) {
                if (main[i] == match[j]) { found = i; break; }
            }
            if (found != (size_t)-1) break;
        }
    } else {
        for (size_t i = (size_t)s; i + mlen <= n; i++) {
            if (memcmp(main + i, match, mlen) == 0) { found = i; break; }
        }
    }
    if (found == (size_t)-1) {
        return pb_bstr_alloc("", 0);
    }
    // Return everything after the match (ANY mode matches 1 char)
    size_t match_len = any_flag ? 1 : mlen;
    size_t after = found + match_len;
    if (after >= n) {
        return pb_bstr_alloc("", 0);
    }
    size_t rlen = n - after;
    return pb_bstr_alloc(main + after, (unsigned int)rlen);
}
char* pb_build(char** arr, long long n) {
    size_t total = 0;
    for (long long i = 0; i < n; i++) total += (size_t)pb_str_len(arr[i]);
    char* buf = (char*)malloc(total + 1);
    size_t o = 0;
    for (long long i = 0; i < n; i++) { size_t l = (size_t)pb_str_len(arr[i]); memcpy(buf + o, arr[i], l); o += l; }
    buf[o] = 0;
    return pb_bstr_alloc(buf, (unsigned int)o);
}

/* Batch 40: OEM / UTF-8 code-page conversion. String args are payload pointers,
   string results are BSTRs. ACODE$ (wide-input) is not implemented: C-strlen
   cannot measure wide strings containing NUL bytes � honestly skipped. */
char* pb_chr_to_oem(char* s) {
    size_t n = strlen(s);
    char* buf = (char*)malloc(n + 1);
    CharToOemA(s, buf);
    buf[n] = 0;
    return pb_bstr_alloc(buf, (unsigned int)n);
}
char* pb_oem_to_chr(char* s) {
    size_t n = strlen(s);
    char* buf = (char*)malloc(n + 1);
    OemToCharA(s, buf);
    buf[n] = 0;
    return pb_bstr_alloc(buf, (unsigned int)n);
}
char* pb_chr_to_utf8(char* s) {
    int n = pb_str_len(s);
    int w = MultiByteToWideChar(0, 0, s, n, NULL, 0);
    if (w <= 0) return pb_bstr_alloc("", 0);
    short* ws = (short*)malloc(w * 2);
    MultiByteToWideChar(0, 0, s, n, ws, w);
    int u = WideCharToMultiByte(65001, 0, ws, w, NULL, 0, NULL, NULL);
    char* buf = (char*)malloc(u + 1);
    WideCharToMultiByte(65001, 0, ws, w, buf, u, NULL, NULL);
    buf[u] = 0;
    free(ws);
    return pb_bstr_alloc(buf, (unsigned int)u);
}
char* pb_utf8_to_chr(char* s) {
    int n = pb_str_len(s);
    int w = MultiByteToWideChar(65001, 0, s, n, NULL, 0);
    if (w <= 0) return pb_bstr_alloc("", 0);
    short* ws = (short*)malloc(w * 2);
    MultiByteToWideChar(65001, 0, s, n, ws, w);
    int a = WideCharToMultiByte(0, 0, ws, w, NULL, 0, NULL, NULL);
    char* buf = (char*)malloc(a + 1);
    WideCharToMultiByte(0, 0, ws, w, buf, a, NULL, NULL);
    buf[a] = 0;
    free(ws);
    return pb_bstr_alloc(buf, (unsigned int)a);
}

/* Batch 39: BIN$/OCT$/DEC$ (radix strings), VERIFY, GETATTR, DISKFREE/DISKSIZE.
   String args are payload pointers; string results are BSTRs. */
char* pb_bin(long long v) {
    unsigned long long u = (unsigned long long)v;
    char buf[66], tmp[66];
    int i = 0, t = 0;
    if (u == 0) { buf[i++] = '0'; }
    else {
        while (u) { tmp[t++] = (char)('0' + (int)(u & 1)); u >>= 1; }
        while (t) buf[i++] = tmp[--t];
    }
    buf[i] = 0;
    return pb_bstr_alloc(buf, (unsigned int)i);
}
char* pb_oct(long long v) {
    unsigned long long u = (unsigned long long)v;
    char buf[24], tmp[24];
    int i = 0, t = 0;
    if (u == 0) { buf[i++] = '0'; }
    else {
        while (u) { tmp[t++] = (char)('0' + (int)(u & 7)); u >>= 3; }
        while (t) buf[i++] = tmp[--t];
    }
    buf[i] = 0;
    return pb_bstr_alloc(buf, (unsigned int)i);
}
char* pb_dec(long long v) {
    /* PRINT integer formatting (PBWin10): leading space before every value,
       including negatives (" -2").  STR$ does NOT use pb_dec (it uses % d /
       % lld in codegen), so this only affects PRINT-family output. */
    char buf[24];
    int n = snprintf(buf, sizeof(buf), " %lld", v);
    return pb_bstr_alloc(buf, (unsigned int)n);
}

/* PBWin keeps fixed notation while %.15g switches to scientific earlier:
   decimal exponent -5 prints as "1e-05" via %g but PBWin prints " .00001".
   Rebuild the fixed form (also trims trailing zeros of the fraction). */
static void pb_expand_fixed(char* buf, int* n) {
    char* epos = strchr(buf, 'e');
    if (epos == NULL) return;
    int exp = atoi(epos + 1);
    if (exp < -15 || exp >= 15) return;     /* PBWin keeps scientific here */
    int sig = (buf[0] == ' ' || buf[0] == '-') ? 1 : 0;
    char m[40];
    int ml = 0;
    for (char* p = buf + sig; p < epos; p++) {
        if (*p != '.') m[ml++] = *p;
    }
    m[ml] = '\0';
    int point = 1 + exp;                    /* integer digits in fixed form */
    char out[40];
    int k = 0;
    for (int i = 0; i < sig; i++) out[k++] = buf[i];
    if (point <= 0) {
        out[k++] = '0';
        out[k++] = '.';
        for (int i = point; i < 0; i++) out[k++] = '0';
        for (int i = 0; i < ml; i++) out[k++] = m[i];
    } else if (point >= ml) {
        for (int i = 0; i < ml; i++) out[k++] = m[i];
        for (int i = ml; i < point; i++) out[k++] = '0';
    } else {
        for (int i = 0; i < point; i++) out[k++] = m[i];
        out[k++] = '.';
        for (int i = point; i < ml; i++) out[k++] = m[i];
    }
    out[k] = '\0';
    char* dot = strchr(out + sig, '.');
    if (dot != NULL) {
        char* q = out + k - 1;
        while (q > dot && *q == '0') { *q = '\0'; k--; q--; }
        if (q == dot) { *q = '\0'; k--; }
    }
    memcpy(buf, out, (size_t)k + 1);
    *n = k;
}

/* =====================================================================
 * PBWin10-faithful DOUBLE -> decimal conversion.
 *
 * The C library's %.15g does not reproduce PBWin's last-digit rounding:
 * the same double (e.g. LOG(10) = 0x40026BB1BBB55516) prints ...405 with
 * MSVC printf but ...404 with PBWin.  PBWin's runtime converts the double
 * to 80-bit extended precision, scales it by an exact power of ten, rounds
 * to an integer with round-to-nearest-even, and only then emits the top
 * digits with a second half-even step.  The hardware 80-bit long double is
 * not usable with the MSVC toolchain, so the 80-bit data path is emulated
 * here with portable 128-bit integer arithmetic.  Validated byte-for-byte
 * against PBWin10 output across subnormals, near-1.0 carries and the full
 * normal/scientific range.
 * ===================================================================== */

typedef struct {
    unsigned long long m;
    int e;
    int sign;
    int zero;
} pb_x87_t;

static const unsigned long long pb_dword_tbl[8] = {
    1ULL, 10ULL, 100ULL, 1000ULL, 10000ULL, 100000ULL, 1000000ULL,
    10000000ULL
};

/* exact 80-bit rounded powers 10^8, 10^16, 10^32, ... (sig, unbiased exp) */
static const unsigned long long pb_tbyte_sig[10] = {
    0xBEBC200000000000ULL, 0x8E1BC9BF04000000ULL, 0x9DC5ADA82B70B59EULL,
    0xC2781F49FFCFA6D5ULL, 0x93BA47C980E98CE0ULL, 0xAA7EEBFB9DF9DE8EULL,
    0xE319A0AEA60E91C7ULL, 0xC976758681750C17ULL, 0x9E8B3B5DC53D5DE5ULL,
    0xC46052028A20979BULL
};
static const int pb_tbyte_exp[10] = {
    26, 53, 106, 212, 425, 850, 1700, 3401, 6803, 20454
};

/* Portable unsigned 128-bit integer (works on 32-bit and 64-bit targets;
   clang's native 128-bit integer type is rejected on i686).  Stored as two
   64-bit words. */
typedef struct {
    unsigned long long lo;
    unsigned long long hi;
} pb_u128;

static pb_u128 pb_u128_from64(unsigned long long x) {
    pb_u128 r;
    r.lo = x;
    r.hi = 0;
    return r;
}

/* add x * 2^32 to the accumulator, tracking the carry explicitly */
static void pb_u128_add_shl32(pb_u128 *r, unsigned long long x) {
    unsigned long long oldlo = r->lo;
    unsigned long long newlo = oldlo + (x << 32);
    unsigned long long carry = (newlo < oldlo);
    r->lo = newlo;
    r->hi += (x >> 32) + carry;
}

/* 64 x 64 -> 128 multiply via 32-bit grade-school partials */
static pb_u128 pb_u128_mul64(unsigned long long a, unsigned long long b) {
    unsigned long long a0 = a & 0xffffffffULL, a1 = a >> 32;
    unsigned long long b0 = b & 0xffffffffULL, b1 = b >> 32;
    pb_u128 r = pb_u128_from64(a0 * b0);
    pb_u128_add_shl32(&r, a0 * b1);
    pb_u128_add_shl32(&r, a1 * b0);
    r.hi += a1 * b1;
    return r;
}

static int pb_u128_bitlen(pb_u128 x) {
    if (x.hi) return 64 + (64 - __builtin_clzll(x.hi));
    return 64 - __builtin_clzll(x.lo);
}

/* low n bits of x (n here is 63 or 64, so effectively the low word) */
static unsigned long long pb_u128_lowbits(pb_u128 x, int n) {
    if (n >= 64) return x.lo;
    return x.lo & ((1ULL << n) - 1ULL);
}

static int pb_u128_ge64(pb_u128 a, unsigned long long d) {
    if (a.hi) return 1;
    return a.lo >= d;
}

static pb_u128 pb_u128_sub64(pb_u128 a, unsigned long long d) {
    pb_u128 r;
    r.lo = a.lo - d;
    r.hi = a.hi - (r.lo > a.lo);   /* borrow when the low word wrapped */
    return r;
}

static pb_x87_t pb_x87_from_bits(unsigned long long bits) {
    pb_x87_t r;
    if (bits == 0) { r.m = 0; r.e = 0; r.sign = 0; r.zero = 1; return r; }
    r.sign = (int)((bits >> 63) & 1ULL);
    unsigned long long e = (bits >> 52) & 0x7ffULL;
    unsigned long long m = bits & ((1ULL << 52) - 1ULL);
    r.zero = 0;
    if (e == 0) {
        /* subnormal: value = m * 2^-1074, no implicit leading 1 */
        int bl = 64 - __builtin_clzll(m);
        int k = 64 - bl;
        r.m = m << k;
        r.e = -1011 - k;
    } else {
        r.m = (m | (1ULL << 52)) << 11;
        r.e = (int)e - 1023;
    }
    return r;
}

static pb_x87_t pb_x87_from_int(unsigned long long n) {
    pb_x87_t r;
    if (n == 0) { r.m = 0; r.e = 0; r.sign = 0; r.zero = 1; return r; }
    int bl = 64 - __builtin_clzll(n);
    int k = 63 - bl + 1;
    r.m = n << k;
    r.e = 63 - k;
    r.sign = 0;
    r.zero = 0;
    return r;
}

static pb_u128 pb_u128_add1(pb_u128 a) {
    a.lo++;
    if (a.lo == 0) a.hi++;
    return a;
}

/* 128-bit value right shifted by n (0..127), kept as 128 bits */
static pb_u128 pb_u128_shr128(pb_u128 x, int n) {
    if (n == 0) return x;
    if (n < 64) {
        x.lo = (x.lo >> n) | (x.hi << (64 - n));
        x.hi = x.hi >> n;
    } else if (n == 64) {
        x.lo = x.hi;
        x.hi = 0;
    } else {
        x.lo = x.hi >> (n - 64);
        x.hi = 0;
    }
    return x;
}

/* 128-bit value left shifted by n (0..127) */
static pb_u128 pb_u128_shl128(pb_u128 x, int n) {
    if (n == 0) return x;
    if (n < 64) {
        x.hi = (x.hi << n) | (x.lo >> (64 - n));
        x.lo = x.lo << n;
    } else if (n == 64) {
        x.hi = x.lo;
        x.lo = 0;
    } else {
        x.hi = x.lo << (n - 64);
        x.lo = 0;
    }
    return x;
}

/* divide (a << sh) by d without materializing the (possibly >128-bit)
   numerator, returning the full quotient via shift-subtract.  The running
   remainder stays below 2^65, so a 128-bit rem is sufficient. */
static pb_u128 pb_u128_div_shl64(unsigned long long a, int sh,
                                 unsigned long long d) {
    pb_u128 q;
    pb_u128 rem;
    q.lo = 0; q.hi = 0;
    rem.lo = 0; rem.hi = 0;
    int top = sh + 63;
    for (int i = top; i >= 0; i--) {
        rem.hi = (rem.hi << 1) | (rem.lo >> 63);
        rem.lo <<= 1;
        if (i >= sh) rem.lo |= (a >> (i - sh)) & 1ULL;
        if (pb_u128_ge64(rem, d)) {
            rem = pb_u128_sub64(rem, d);
            if (i >= 64) q.hi |= (1ULL << (i - 64));
            else q.lo |= (1ULL << i);
        }
    }
    return q;
}

/* round a 128-bit value at the given right shift, result kept at 128 bits so
   an overflow to 2^64 is preserved (handled by the reduce step) */
static pb_u128 pb_u128_round_shift(pb_u128 x, int shift) {
    if (shift <= 0) {
        if (shift == 0) return x;
        return pb_u128_shl128(x, -shift);
    }
    unsigned long long half = 1ULL << (shift - 1);
    unsigned long long low = pb_u128_lowbits(x, shift);
    pb_u128 q = pb_u128_shr128(x, shift);
    if (low > half) q = pb_u128_add1(q);
    else if (low == half && (q.lo & 1ULL)) q = pb_u128_add1(q);
    return q;
}

/* reduce a (possibly wider than 64-bit) mantissa to a normalized 64-bit
   pb_x87_t, mirroring the reference normalize: round down while >= 2^64,
   then shift up while < 2^63 */
static pb_x87_t pb_x87_reduce128(pb_u128 mant, int e, int sign) {
    pb_x87_t r;
    r.sign = sign;
    r.zero = 0;
    while (mant.hi != 0) {
        /* half-even at scale 1 (the low bit is the half position) */
        unsigned long long lb = mant.lo & 1ULL;
        pb_u128 q = pb_u128_shr128(mant, 1);
        if (lb && (q.lo & 1ULL)) q = pb_u128_add1(q);
        mant = q;
        e++;
    }
    r.m = mant.lo;
    while (r.m != 0 && (r.m & (1ULL << 63)) == 0) {
        r.m <<= 1;
        e--;
    }
    if (r.m == 0) {
        r.m = 0; r.e = 0; r.zero = 1;
    } else {
        r.e = e;
    }
    return r;
}

static pb_x87_t pb_x87_mul(pb_x87_t a, pb_x87_t b) {
    int sign = a.sign ^ b.sign;
    if (a.zero || b.zero) {
        pb_x87_t r;
        r.m = 0; r.e = 0; r.sign = sign; r.zero = 1;
        return r;
    }
    pb_u128 prod = pb_u128_mul64(a.m, b.m);
    int shift = pb_u128_bitlen(prod) - 64;
    pb_u128 q = pb_u128_round_shift(prod, shift);
    int e = a.e + b.e - 63 + shift;
    return pb_x87_reduce128(q, e, sign);
}

static pb_x87_t pb_x87_div(pb_x87_t a, pb_x87_t b) {
    int sign = a.sign ^ b.sign;
    pb_u128 q = pb_u128_div_shl64(a.m, 70, b.m);
    int e = a.e - b.e - 7;
    return pb_x87_reduce128(q, e, sign);
}

/* frndint: round to integer, keeping the natural exponent (RC = nearest) */
static pb_x87_t pb_x87_frndint(pb_x87_t a) {
    pb_x87_t r = a;
    if (a.e >= 63) return r;
    if (a.e < 0) { r.m = 0; r.e = 0; r.zero = 1; return r; }
    int drop = 63 - a.e;
    unsigned long long half = 1ULL << (drop - 1);
    unsigned long long low = a.m & ((1ULL << drop) - 1ULL);
    unsigned long long q = a.m >> drop;
    if (low > half) q++;
    else if (low == half && (q & 1ULL)) q++;
    r.m = q << drop;
    if (r.m == 0) { r.e = 0; r.zero = 1; }
    return r;
}

static unsigned long long pb_x87_to_int(pb_x87_t a) {
    pb_x87_t v = pb_x87_frndint(a);
    if (v.zero) return 0;
    if (v.e >= 63) return v.m << (v.e - 63);
    return v.m >> (63 - v.e);
}

static int pb_x87_cmp(pb_x87_t a, pb_x87_t b) {
    if (a.zero && b.zero) return 0;
    if (a.e != b.e) return a.e < b.e ? -1 : 1;
    if (a.m != b.m) return a.m < b.m ? -1 : 1;
    return 0;
}

/* high signed word of a 32-bit signed multiply (truncation toward zero) */
static int pb_imul_high(int a, int b) {
    unsigned int u = (unsigned int)(a * b);
    int h = (int)(u >> 16);
    if (h >= 0x8000) h -= 0x10000;
    return h;
}

static pb_x87_t pb_build_factor(unsigned long long mag) {
    pb_x87_t f;
    unsigned long long low = mag & 7ULL;
    f = low ? pb_x87_from_int(pb_dword_tbl[low]) : pb_x87_from_int(1);
    mag >>= 3;
    int i = 0;
    while (mag) {
        if (mag & 1ULL) {
            pb_x87_t t;
            t.m = pb_tbyte_sig[i];
            t.e = pb_tbyte_exp[i];
            t.sign = 0;
            t.zero = 0;
            f = pb_x87_mul(f, t);
        }
        mag >>= 1;
        i++;
    }
    return f;
}

static int pb_ull_digits(unsigned long long n) {
    if (n == 0) return 1;
    int d = 0;
    while (n) { n /= 10ULL; d++; }
    return d;
}

/* core scaling: returns the integer digits and the decimal exponent */
static unsigned long long pb_x87_core(unsigned long long bits, int digits,
                                      int *out_de, int *out_sign) {
    pb_x87_t x = pb_x87_from_bits(bits);
    int de_raw = pb_imul_high(x.e, 0x4d10);
    /* Total power of 10 by which the value is scaled (preserved across the
       0x1000 factor staging below). The rounded integer r = value*10^ax_total. */
    int ax_total = (digits + 2) - de_raw;
    int ax = ax_total;
    if (ax > 0x1000) {
        pb_x87_t t;
        t.m = pb_tbyte_sig[9]; t.e = pb_tbyte_exp[9]; t.sign = 0; t.zero = 0;
        x = pb_x87_mul(x, t);
        ax -= 0x1000;
    } else if (ax <= -0x1000) {
        pb_x87_t t;
        t.m = pb_tbyte_sig[9]; t.e = pb_tbyte_exp[9]; t.sign = 0; t.zero = 0;
        x = pb_x87_div(x, t);
        ax += 0x1000;
    }
    if (ax != 0) {
        pb_x87_t f = pb_build_factor((unsigned long long)(ax < 0 ? -ax : ax));
        x = ax > 0 ? pb_x87_mul(x, f) : pb_x87_div(x, f);
    }
    pb_x87_t r = pb_x87_frndint(x);
    pb_x87_t p18;
    p18.m = 0xDE0B6B3A76400000ULL; p18.e = 59; p18.sign = 0; p18.zero = 0;
    int extra_div = 0;
    if (pb_x87_cmp(r, p18) >= 0) {
        r = pb_x87_div(r, pb_x87_from_int(10));
        extra_div = 1;
    }
    unsigned long long ri = pb_x87_to_int(r);
    /* True decimal exponent derived from the actual integer. Because
       ri = value * 10^ax_total (one extra /10 when the p18 guard fires),
       floor(log10(value)) = digits(ri) - 1 - ax_total + extra_div. This
       includes the mantissa contribution that e*log10(2) alone omitted,
       which previously made every exponent one too small. */
    int de = pb_ull_digits(ri) - 1 - ax_total + extra_div;
    *out_de = de;
    *out_sign = r.sign;
    return ri;
}

static int pb_ull_str(unsigned long long n, char *out) {
    char tmp[24];
    int t = 0;
    do { tmp[t++] = (char)('0' + (n % 10ULL)); n /= 10ULL; } while (n);
    int len = 0;
    while (t) out[len++] = tmp[--t];
    out[len] = '\0';
    return len;
}

static int pb_signed_str(int v, char *out) {
    int neg = v < 0;
    unsigned long long n = neg ? (unsigned long long)(-(long long)v)
                               : (unsigned long long)v;
    char tmp[24];
    int t = 0;
    do { tmp[t++] = (char)('0' + (n % 10ULL)); n /= 10ULL; } while (n);
    int len = 0;
    if (neg) out[len++] = '-';
    while (t) out[len++] = tmp[--t];
    out[len] = '\0';
    return len;
}

/* shared renderer: digits significant digits; lead_space selects STR$ vs TRIM$ */
static char* pb_num_render(double v, int digits, int lead_space) {
    unsigned long long bits;
    memcpy(&bits, &v, sizeof(bits));
    if (bits == 0) {
        if (lead_space) return pb_bstr_alloc(" 0", 2);
        return pb_bstr_alloc("0", 1);
    }
    int sign = (int)((bits >> 63) & 1ULL);
    int de, rsign;
    unsigned long long n = pb_x87_core(bits, digits, &de, &rsign);
    if (n == 0) {
        if (lead_space) return pb_bstr_alloc(" 0", 2);
        return pb_bstr_alloc("0", 1);
    }
    char s[32];
    int sl = pb_ull_str(n, s);
    /* half-even: keep top `digits`, decide from the (digits+1)th digit */
    if (sl > digits) {
        char dnext = s[digits];
        s[digits] = '\0';
        unsigned long long head = strtoull(s, NULL, 10);
        if (dnext > '5') head++;
        else if (dnext == '5' && (head & 1ULL)) head++;
        sl = pb_ull_str(head, s);
        if (sl > digits) de++;   /* full carry past the first digit */
    }
    while (sl > 0 && s[sl - 1] == '0') s[--sl] = '\0';
    int ns = sl;
    /* CHW/PBWin scientific-notation rule (verified against PBWin):
     * - de >= 0: scientific when the integral part cannot fit, de >= digits
     *   (e.g. TRIM$(123.456,2)="1.2E+2").
     * - de < 0: fixed-point needs (-de-1) leading zeros plus ns significant
     *   digits; scientific only when that exceeds the budget, i.e.
     *   ns-de-1 > digits (e.g. STR$(0.001234)=" .001234" fixed at digits=15,
     *   but TRIM$(0.001234,2)="1.2E-3" scientific).
     * At digits=15 both branches are exactly the old hard-coded thresholds. */
    int scientific;
    if (de >= 0) {
        scientific = (de >= digits) ? 1 : 0;
    } else {
        scientific = (ns - de - 1 > digits) ? 1 : 0;
    }

    char out[80];
    int k = 0;
    if (lead_space) out[k++] = sign ? '-' : ' ';
    else if (sign) out[k++] = '-';

    if (scientific) {
        out[k++] = s[0];
        if (ns > 1) {
            out[k++] = '.';
            memcpy(out + k, s + 1, (size_t)(ns - 1));
            k += ns - 1;
        }
        out[k++] = 'E';
        if (de >= 0) out[k++] = '+';
        char des[16];
        int dl = pb_signed_str(de, des);
        memcpy(out + k, des, (size_t)dl);
        k += dl;
    } else if (de >= 0) {
        if (de + 1 >= ns) {
            memcpy(out + k, s, (size_t)ns);
            k += ns;
            int zeros = de + 1 - ns;
            memset(out + k, '0', (size_t)zeros);
            k += zeros;
        } else {
            memcpy(out + k, s, (size_t)(de + 1));
            k += de + 1;
            out[k++] = '.';
            memcpy(out + k, s + de + 1, (size_t)(ns - (de + 1)));
            k += ns - (de + 1);
        }
    } else {
        out[k++] = '.';
        int gap = -de;
        memset(out + k, '0', (size_t)(gap - 1));
        k += gap - 1;
        memcpy(out + k, s, (size_t)ns);
        k += ns;
    }
    out[k] = '\0';
    return pb_bstr_alloc(out, (unsigned int)k);
}

/* STR$(double) - PBWin10 measured formatting: leading space for positive
   values, '-' for negative; 15 significant digits, trailing zeros removed. */
char* pb_str_dbl(double v) {
    return pb_num_render(v, 15, 1);
}

/* STR$(double, digits) - CHW STR_function.htm: "digits specifies the maximum
   number of significant digits (1 to 18)".  digits<=0 defaults to 15; values
   above 18 clamp to 18.  Leading space for positive, '-' for negative. */
char* pb_str_dbl_d(double v, int digits) {
    if (digits <= 0) digits = 15;
    if (digits > 18) digits = 18;
    return pb_num_render(v, digits, 1);
}

/* TRIM$ numeric form (CHW TRIM_function.htm): no leading/trailing spaces.
   digits<=0 selects the default 15 significant digits, otherwise 1..18. */
char* pb_trim_num(double v, int digits) {
    if (digits <= 0) digits = 15;
    if (digits > 18) digits = 18;
    return pb_num_render(v, digits, 0);
}

/* BIN$/OCT$/DEC$(IntVal[, Digits, LeadSpaces, TrailSpaces]) (CHW, batch 256B).
   base 2/8/10; Digits > 0 = leading zeros to that width, < 0 = leading spaces
   to |Digits|, 0 or absent = none; LeadSpaces/TrailSpaces add extra spaces.
   base 10 is signed (DEC$(-5)="-5"); bases 2/8 use the unsigned bit pattern
   (BIN$(-1) = 64 ones, PBWin-measured). */
char* pb_radix_fmt(long long v, long long base, long long digits,
                   long long lead, long long trail) {
    char tmp[66];
    int numlen;
    if (base == 10) {
        numlen = snprintf(tmp, sizeof(tmp), "%lld", v);
    } else {
        unsigned long long u = (unsigned long long)v;
        int t = 0;
        if (u == 0) { tmp[t++] = '0'; }
        else {
            while (u) {
                int d = (int)(u % (unsigned long long)base);
                tmp[t++] = (char)('0' + d);
                u /= (unsigned long long)base;
            }
        }
        for (int i = 0; i < t / 2; i++) {
            char c = tmp[i]; tmp[i] = tmp[t - 1 - i]; tmp[t - 1 - i] = c;
        }
        numlen = t;
    }
    long long total = numlen;
    if (digits > 0 && digits > numlen) total = digits;
    else if (digits < 0 && -digits > numlen) total = -digits;
    if (lead > 0) total += lead;
    if (trail > 0) total += trail;
    if (total > 4096) total = 4096;
    char* buf = (char*)malloc((size_t)total + 1);
    int i = 0;
    if (lead > 0) { long long k; for (k = 0; k < lead; k++) buf[i++] = ' '; }
    if (digits < 0) { long long k; for (k = 0; k < (-digits - numlen); k++) buf[i++] = ' '; }
    if (digits > 0) { long long k; for (k = 0; k < (digits - numlen); k++) buf[i++] = '0'; }
    memcpy(buf + i, tmp, (size_t)numlen);
    i += numlen;
    if (trail > 0) { long long k; for (k = 0; k < trail; k++) buf[i++] = ' '; }
    buf[i] = 0;
    return pb_bstr_alloc(buf, (unsigned int)i);
}
long long pb_verify(char* s, char* m, long long start) {
    long long sl = (long long)strlen(s);
    long long i;
    /* CHW VERIFY_function.htm: a start position outside the string on either
     * side (including start=0) returns zero. */
    if (start < 1 || start > sl) return 0;
    for (i = start; i <= sl; i++) {
        if (!strchr(m, s[i - 1])) return i;
    }
    return 0;
}
long long pb_getattr(char* path) {
    unsigned long attrs = GetFileAttributesA(path);
    /* PBWin10 measured (2026-10-14 probe39): a missing path returns 0, not
     * GetFileAttributes' 0xFFFFFFFF.  Map INVALID_FILE_ATTRIBUTES -> 0. */
    if (attrs == 0xFFFFFFFFUL) return 0;
    return (long long)attrs;
}
static void pb_root_of(char* root, char* path) {
    if (path && path[0]) {
        /* drive:\  or  \server\share\  (UNC share root) */
        int i = 0;
        while (path[i] && path[i] != '\\') i++;
        if (i >= 2 && path[1] == ':') {
            root[0] = path[0]; root[1] = ':'; root[2] = '\\'; root[3] = 0;
            return;
        }
    }
    root[0] = (char)('A' + _getdrive() - 1);
    root[1] = ':'; root[2] = '\\'; root[3] = 0;
}
long long pb_diskfree(char* path) {
    char root[5];
    unsigned long long freeb = 0, total = 0, totfree = 0;
    pb_root_of(root, path);
    if (!GetDiskFreeSpaceExA(root, &freeb, &total, &totfree)) return -1;
    return (long long)freeb;
}
long long pb_disksize(char* path) {
    char root[5];
    unsigned long long freeb = 0, total = 0, totfree = 0;
    pb_root_of(root, path);
    if (!GetDiskFreeSpaceExA(root, &freeb, &total, &totfree)) return -1;
    return (long long)total;
}

/* Batch 38: string / math helpers for TALLY, STRREVERSE$, STRINSERT$, STRDELETE$,
   REPEAT$, FRAC, ISFOLDER, EXP2/EXP10/LOG2/LOG10. String args are payload pointers;
   string results are BSTRs (pb_bstr_alloc), matching the existing builtin pattern. */
long long pb_tally(char* s, char* m) {
    size_t n = strlen(m);
    if (n == 0) return 0;
    long long c = 0;
    char* p = s;
    while ((p = strstr(p, m)) != NULL) { c++; p += n; }
    return c;
}
/* TALLY ANY: count each byte of s that is a member of the single-character
   set m. The inner break dedups a repeated char in m (CHW: TALLY("ABCD",
   ANY "BDB")=2). Case-sensitive and BSTR-safe via byte lengths. */
long long pb_tally_any(char* s, char* m) {
    size_t sn = SysStringByteLen(s);
    size_t mn = SysStringByteLen(m);
    long long c = 0;
    size_t i, j;
    for (i = 0; i < sn; i++) {
        for (j = 0; j < mn; j++) {
            if (m[j] == s[i]) { c++; break; }
        }
    }
    return c;
}
/* TAB$(strtotab, tabstop) - expand every tab (CHR$9 / $TAB) into spaces so
   the next character lands one column past the next tab-stop boundary.
   CHW TAB_function.htm: a tab stop below 1 or above 256 returns the original
   string unchanged. BSTR-safe (uses the byte length, preserves embedded NULs). */
char* pb_tab_expand(char* s, int tabstop) {
    if (!s) return pb_bstr_alloc("", 0);
    size_t n = SysStringByteLen(s);
    if (tabstop < 1 || tabstop > 256) return pb_bstr_alloc(s, (unsigned int)n);
    size_t cap = n * (size_t)tabstop + 1;
    char* buf = (char*)malloc(cap);
    if (!buf) return pb_bstr_alloc(s, (unsigned int)n);
    size_t col = 0, out = 0, i;
    for (i = 0; i < n; i++) {
        unsigned char c = (unsigned char)s[i];
        if (c == 9u) {
            int spaces = tabstop - (int)(col % tabstop);
            int k;
            for (k = 0; k < spaces; k++) { buf[out++] = ' '; col++; }
        } else {
            buf[out++] = (char)c;
            col++;
        }
    }
    char* r = pb_bstr_alloc(buf, (unsigned int)out);
    free(buf);
    return r;
}
char* pb_strreverse(char* s) {
    /* BSTR-safe: use the byte length, not strlen — CHR$(0 TO 255) starts with
       NUL and strlen would report 0 (batch272: STRREVERSE$ of a NUL-leading
       string silently returned ""). */
    size_t n = SysStringByteLen(s);
    char* buf = (char*)malloc(n + 1);
    size_t i;
    for (i = 0; i < n; i++) buf[i] = s[n - 1 - i];
    buf[n] = 0;
    return pb_bstr_alloc(buf, (unsigned int)n);
}
char* pb_strinsert(char* s, char* n, long long pos) {
    size_t sl = SysStringByteLen(s), nl = SysStringByteLen(n);
    if (pos < 1) pos = 1;
    if ((size_t)pos > sl + 1) pos = (long long)sl + 1;
    size_t b = (size_t)pos - 1;
    size_t total = sl + nl;
    char* buf = (char*)malloc(total + 1);
    memcpy(buf, s, b);
    memcpy(buf + b, n, nl);
    memcpy(buf + b + nl, s + b, sl - b);
    buf[total] = 0;
    return pb_bstr_alloc(buf, (unsigned int)total);
}
char* pb_strdelete(char* s, long long start, long long count) {
    size_t sl = SysStringByteLen(s);
    if (start < 1) start = 1;
    if (count < 0) count = 0;
    size_t b = (size_t)start - 1;
    if (b > sl) b = sl;
    size_t del = (size_t)count;
    if (del > sl - b) del = sl - b;
    size_t total = sl - del;
    char* buf = (char*)malloc(total + 1);
    memcpy(buf, s, b);
    memcpy(buf + b, s + b + del, sl - b - del);
    buf[total] = 0;
    return pb_bstr_alloc(buf, (unsigned int)total);
}
char* pb_repeat(long long n, char* s) {
    size_t sl = SysStringByteLen(s);
    if (n < 0) n = 0;
    size_t total = (size_t)n * sl;
    char* buf = (char*)malloc(total + 1);
    long long i;
    for (i = 0; i < n; i++) memcpy(buf + i * sl, s, sl);
    buf[total] = 0;
    return pb_bstr_alloc(buf, (unsigned int)total);
}
char* pb_nul_str(long long n) {
    // NUL$(n) — a BSTR of n ASCII-0 bytes (batch 306).  WindowsX.inc /
    // ListBoxCtrl.inc use NUL$(4096) to preallocate a receive buffer.
    if (n < 0) n = 0;
    char* buf = (char*)malloc((size_t)n + 1);
    if (buf) memset(buf, 0, (size_t)n + 1);
    return pb_bstr_alloc(buf, (unsigned int)n);
}
double pb_frac(double x) {
    double i;
    return modf(x, &i);
}
long long pb_isfolder(char* name) {
    struct _stat st;
    if (_stat(name, &st) == 0 && (st.st_mode & _S_IFDIR)) return -1;
    return 0;
}
/* ISFILE: true only for an existing non-directory file, in any attribute
   (hidden/system/read-only). Wildcards and directories return 0, matching
   CHW ISFILE and the measured PBWin behavior. */
long long pb_isfile(const char* name) {
    if (!name) return 0;
    if (strchr(name, '*') || strchr(name, '?')) return 0;
    struct _stat st;
    if (_stat(name, &st) != 0) return 0;
    if (st.st_mode & _S_IFDIR) return 0;
    return -1;
}
double pb_exp2(double x) { return exp2(x); }
double pb_exp10(double x) { return pow(10.0, x); }
double pb_log2(double x) { return log2(x); }
double pb_log10(double x) { return log10(x); }

/* Batch 37: CVx family � read little-endian binary strings (PB payload pointer).
   off is 1-based character position, default 1.
   mode 0 = zero-extend (BYTE/WORD/DWORD), 1 = sign-extend (LONG/QUAD). */
long long pb_cv_int(char* s, long long off, long long n, long long mode) {
    unsigned char* p = (unsigned char*)s;
    unsigned long long v = 0;
    long long i;
    if (off < 1) off = 1;
    for (i = 0; i < n; i++) v |= ((unsigned long long)p[off - 1 + i]) << (8 * i);
    if (mode == 1) {
        if (n == 1) return (long long)(signed char)v;
        if (n == 2) return (long long)(short)v;   /* CVI: sign-extend 16-bit */
        if (n == 4) return (long long)(long)(int)v;
        return (long long)v;
    }
    if (n == 1) return (long long)(v & 0xFFu);
    if (n == 2) return (long long)(v & 0xFFFFu);
    if (n == 4) return (long long)(v & 0xFFFFFFFFu);
    return (long long)v;
}
double pb_cv_dbl(char* s, long long off, long long n) {
    unsigned char* p = (unsigned char*)s;
    if (off < 1) off = 1;
    if (n == 4) {
        float f;
        memcpy(&f, p + off - 1, 4);
        return (double)f;
    }
    double d;
    memcpy(&d, p + off - 1, 8);
    return d;
}

char* pb_mkdouble(double v) {
    return pb_bstr_alloc((const char*)&v, 8);      /* MKD$ */
}
/* MKCUR$ / MKCUX$ (batch283): CURRENCY fixed-point pack.  PBWin stores
   CURRENCY as a signed 64-bit int scaled by 10000 (CUR) / 100 (CUX);
   MKCUR$(123.45) yields the bytes of int64 1234500. */
char* pb_mkcur(double x, long long scale) {
    long long v = (long long)llround(x * (double)scale);
    return pb_bstr_alloc((const char*)&v, 8);
}
/* CVCUR$ / CVCUX$: read the 8-byte fixed-point int back, as a double. */
double pb_cv_cur(char* s, long long off, long long scale) {
    unsigned char* p = (unsigned char*)s;
    unsigned long long uv = 0;
    int i;
    if (off < 1) off = 1;
    for (i = 0; i < 8; i++) uv |= ((unsigned long long)p[off - 1 + i]) << (8 * i);
    return (double)(long long)uv / (double)scale;
}
/* MKE$ (batch283b): pack a double as the official PB 10-byte 80-bit extended
   (x87) format.  Byte layout verified against PBWin10 probe: MKE$(1.5) =
   00 00 00 00 00 00 00 C0 FF 3F; MKE$(-1.5) = ... C0 FF BF; MKE$(0) = 10 zeros.
   Sign(1) | exponent(15, bias 16383) | explicit-integer significand(64). */
char* pb_mkext(double v) {
    unsigned char buf[10];
    union { double d; unsigned long long u; } du;
    unsigned long long m;
    int e, sign, i;
    du.d = v;
    sign = (int)(du.u >> 63);
    {
        int dexp = (int)((du.u >> 52) & 0x7FF);
        unsigned long long dfrac = du.u & 0xFFFFFFFFFFFFFULL;
        if (dexp == 0x7FF) {                     /* inf / nan */
            m = (1ULL << 63) | (dfrac << 11);
            e = 0x7FFF;
        } else if (dexp == 0) {
            if (dfrac == 0) { m = 0; e = 0; }    /* +/- 0 */
            else {                               /* double denormal -> normalise */
                int shift = 0;
                unsigned long long f = dfrac;
                while (!(f & (1ULL << 52))) { f <<= 1; shift++; }
                m = (1ULL << 63) | ((f << 11) & 0x7FFFFFFFFFFFFFFFULL);
                e = 16383 - 1022 - shift;
            }
        } else {                                 /* normal double */
            m = (1ULL << 63) | (dfrac << 11);
            e = dexp - 1023 + 16383;
        }
    }
    for (i = 0; i < 8; i++) buf[i] = (unsigned char)(m >> (8 * i));
    buf[8] = (unsigned char)(e & 0xFF);
    buf[9] = (unsigned char)(((e >> 8) & 0x7F) | (sign ? 0x80 : 0));
    return pb_bstr_alloc((const char*)buf, 10);
}
/* CVE$: read a 10-byte 80-bit extended back as a double. */
double pb_cv_ext(char* s, long long off) {
    unsigned char* p = (unsigned char*)s;
    unsigned long long m = 0;
    int i;
    if (off < 1) off = 1;
    for (i = 0; i < 8; i++) m |= ((unsigned long long)p[off - 1 + i]) << (8 * i);
    {
        /* off is 1-based; the 10 bytes start at p[off-1].  m reads p[off-1..off+6];
           exponent/sign live at p[off+7] and p[off+8]. */
        int e = p[off - 1 + 8] | ((p[off - 1 + 9] & 0x7F) << 8);
        int sign = (p[off - 1 + 9] >> 7) & 1;
        union { double d; unsigned long long u; } du;
        int dexp;
        unsigned long long dfrac;
        if (e == 0x7FFF) {
            dexp = 0x7FF;
            dfrac = (m & 0x7FFFFFFFFFFFFFFFULL) >> 11;
        } else if (e == 0) {
            if (m == 0) { dexp = 0; dfrac = 0; }
            else {                               /* extended denormal: round-to-zero */
                dexp = 0; dfrac = 0;             /* below double range (honest) */
            }
        } else {
            dfrac = (m & 0x7FFFFFFFFFFFFFFFULL) >> 11;
            dexp = e - 16383 + 1023;
        }
        du.u = ((unsigned long long)sign << 63)
             | ((unsigned long long)dexp << 52) | dfrac;
        return du.d;
    }
}

/* DESKTOP GET CLIENT / LOC / PPI (work area = screen minus taskbar) */
typedef struct { long left, top, right, bottom; } PB_RECT;
#define PB_SPI_GETWORKAREA 0x0030
void pb_desktop_get_client(long* w, long* h) {
    PB_RECT rc = {0, 0, 0, 0};
    SystemParametersInfoA(PB_SPI_GETWORKAREA, 0, &rc, 0);
    *w = rc.right - rc.left;
    *h = rc.bottom - rc.top;
}
void pb_desktop_get_loc(long* x, long* y) {
    PB_RECT rc = {0, 0, 0, 0};
    SystemParametersInfoA(PB_SPI_GETWORKAREA, 0, &rc, 0);
    *x = rc.left;
    *y = rc.top;
}
void pb_desktop_get_ppi(long* x, long* y) {
    void* dc = GetDC(NULL);
    *x = GetDeviceCaps(dc, 88);   /* LOGPIXELSX */
    *y = GetDeviceCaps(dc, 90);   /* LOGPIXELSY */
    ReleaseDC(NULL, dc);
}

/* LEN() � BSTR byte length (unlike strlen, handles embedded NUL bytes) */
int pb_str_len(const char* s) {
    if (!s) return 0;
    const unsigned int* p = (const unsigned int*)s - 1;  /* BSTR length prefix */
    return (int)*p;
}

/* VAL: convert a text string to a number (CHW VAL_function.htm).
 * offset is 1-based; pass 0/1 to start at the first character.
 * Supports leading whitespace skip, &H/&B/&O/&Q/& base-prefixed integers
 * (leading zero -> unsigned, otherwise 32-bit signed), decimal floating point
 * with e/d exponent, comma and other non-numeric characters terminate. */
double pb_val(const char* s, int offset) {
    if (!s) return 0.0;
    int n = pb_str_len(s);
    int p = offset - 1;
    if (p < 0) p = 0;
    if (p >= n) return 0.0;
    while (p < n) {
        char c = s[p];
        if (c == ' ' || c == '\t' || c == '\r' || c == '\n') p++;
        else break;
    }
    if (p >= n) return 0.0;
    /* base-prefixed integer */
    if (s[p] == '&' && p + 1 < n) {
        char bc = s[p + 1];
        int base = 8;
        if (bc == 'H' || bc == 'h') base = 16;
        else if (bc == 'B' || bc == 'b') base = 2;
        else if (bc == 'O' || bc == 'o' || bc == 'Q' || bc == 'q') base = 8;
        int q = p + 2;
        unsigned long long acc = 0;
        int any = 0;
        while (q < n) {
            char c = s[q];
            int d = -1;
            if (c >= '0' && c <= '9') d = c - '0';
            else if (base == 16 && c >= 'A' && c <= 'F') d = c - 'A' + 10;
            else if (base == 16 && c >= 'a' && c <= 'f') d = c - 'a' + 10;
            if (d < 0 || d >= base) break;
            acc = acc * (unsigned)base + (unsigned)d;
            any = 1;
            q++;
        }
        if (!any) return 0.0;
        if (s[p + 2] == '0') {
            return (double)acc;  /* leading zero -> unsigned */
        }
        /* signed: smallest signed width is 16-bit (no 8-bit signed integer
         * type), then 32/64-bit. Sign-extend at the boundary whose top bit
         * is set. Verified against PBWin (CHW VAL_function.htm). */
        if (acc < 0x8000ull) return (double)acc;            /* < 2^15 positive */
        if (acc < 0x10000ull) return -(double)(0x10000ull - acc);     /* 16-bit neg */
        if (acc < 0x80000000ull) return (double)acc;        /* < 2^31 positive */
        if (acc < 0x100000000ull) return -(double)(0x100000000ull - acc); /* 32-bit neg */
        if (acc < 0x8000000000000000ull) return (double)acc; /* < 2^63 positive */
        return -(double)(0xFFFFFFFFFFFFFFFFull - acc + 1);  /* 64-bit neg */
    }
    /* decimal floating point: build a NUL-terminated working copy, map d/D
     * exponent to e/E, stop at the first non-numeric character (comma etc). */
    char* work = (char*)malloc((size_t)(n - p) + 1);
    if (!work) return 0.0;
    int w = 0;
    int seen_digit = 0;
    while (p < n) {
        char c = s[p];
        if (c >= '0' && c <= '9') { work[w++] = c; seen_digit = 1; }
        else if (c == '.') work[w++] = c;
        else if (c == 'd' || c == 'D') work[w++] = 'e';
        else if (c == 'e' || c == 'E') work[w++] = c;
        else if (c == '+' || c == '-') work[w++] = c;
        else break;  /* comma and any other non-numeric character */
        p++;
    }
    work[w] = 0;
    double r = seen_digit ? strtod(work, NULL) : 0.0;
    free(work);
    return r;
}

/* VAL statement backend: same conversion as pb_val, but also report the
 * mantissa digit count and the count of unused trailing characters (CHW
 * Val_statement.htm). Results are written through the output pointers.
 * unused skips whitespace immediately following the number, then counts
 * everything to the end (trailing whitespace is retained); a dangling
 * exponent marker with no digits is consumed and does not count as
 * unused. */
void pb_val_full(const char* s, int offset, double* pv, int* pd, int* pu) {
    *pv = 0.0;
    *pd = 0;
    *pu = 0;
    if (!s) return;
    int n = pb_str_len(s);
    int p = offset - 1;
    if (p < 0) p = 0;
    if (p >= n) return;
    while (p < n) {
        char c = s[p];
        if (c == ' ' || c == '\t' || c == '\r' || c == '\n') p++;
        else break;
    }
    if (p >= n) return;  /* all whitespace */
    /* base-prefixed integer */
    if (s[p] == '&' && p + 1 < n) {
        char bc = s[p + 1];
        int base = 8;
        if (bc == 'H' || bc == 'h') base = 16;
        else if (bc == 'B' || bc == 'b') base = 2;
        else if (bc == 'O' || bc == 'o' || bc == 'Q' || bc == 'q') base = 8;
        int q = p + 2;
        unsigned long long acc = 0;
        int any = 0;
        int digits = 0;
        while (q < n) {
            char c = s[q];
            int d = -1;
            if (c >= '0' && c <= '9') d = c - '0';
            else if (base == 16 && c >= 'A' && c <= 'F') d = c - 'A' + 10;
            else if (base == 16 && c >= 'a' && c <= 'f') d = c - 'a' + 10;
            if (d < 0 || d >= base) break;
            acc = acc * (unsigned)base + (unsigned)d;
            any = 1;
            digits++;
            q++;
        }
        if (!any) return;  /* bare prefix: no digits, no unused */
        double val;
        if (s[p + 2] == '0') {
            val = (double)acc;
        } else if (acc < 0x8000ull) {
            val = (double)acc;
        } else if (acc < 0x10000ull) {
            val = -(double)(0x10000ull - acc);
        } else if (acc < 0x80000000ull) {
            val = (double)acc;
        } else if (acc < 0x100000000ull) {
            val = -(double)(0x100000000ull - acc);
        } else if (acc < 0x8000000000000000ull) {
            val = (double)acc;
        } else {
            val = -(double)(0xFFFFFFFFFFFFFFFFull - acc + 1);
        }
        *pv = val;
        *pd = digits;
        /* unused: skip whitespace immediately after the number, then count
         * everything to the end (trailing whitespace is retained). */
        int st = q;
        while (st < n && (s[st] == ' ' || s[st] == '\t' ||
                          s[st] == '\r' || s[st] == '\n')) st++;
        *pu = n - st;
        return;
    }
    /* decimal floating point */
    int q = p;
    int mdigits = 0;
    if (q < n && (s[q] == '+' || s[q] == '-')) q++;
    while (q < n) {
        char c = s[q];
        if (c >= '0' && c <= '9') { mdigits++; q++; }
        else if (c == '.') q++;
        else break;
    }
    int scan_end = q;  /* unused starts here unless an exponent is consumed */
    if (q < n && (s[q] == 'e' || s[q] == 'E' || s[q] == 'd' || s[q] == 'D')) {
        int eq = q + 1;
        if (eq < n && (s[eq] == '+' || s[eq] == '-')) eq++;
        if (eq < n && s[eq] >= '0' && s[eq] <= '9') {
            q = eq;
            while (q < n && s[q] >= '0' && s[q] <= '9') q++;
            scan_end = q;  /* full exponent consumed */
        } else {
            scan_end = q + 1;  /* dangling marker: consume it, no unused */
        }
    }
    if (mdigits > 0) {
        char* work = (char*)malloc((size_t)(scan_end - p) + 1);
        if (work) {
            int w = 0;
            for (int i = p; i < scan_end; i++) {
                char c = s[i];
                work[w++] = (c == 'd' || c == 'D') ? 'e' : c;
            }
            work[w] = 0;
            *pv = strtod(work, NULL);
            free(work);
        }
    }
    *pd = mdigits;
    /* unused: skip whitespace immediately after the number, then count
     * everything to the end (trailing whitespace is retained). */
    int st = scan_end;
    while (st < n && (s[st] == ' ' || s[st] == '\t' ||
                      s[st] == '\r' || s[st] == '\n')) st++;
    *pu = n - st;
}

/* GLOBALMEM: allocate moveable global memory, return 1-based slot id so the
 * handle fits a PB LONG/DWORD on 64-bit. Slot table avoids truncation. */
#define GMEM_MOVEABLE 0x0002u
#define GMEM_ZEROINIT 0x0040u
static void* gm_table[256];
static int gm_alloc_slot(void) {
    for (int i = 1; i < 256; i++) {
        if (!gm_table[i]) return i;
    }
    return 0;
}
long pb_globalmem_alloc(long count) {
    void* h = GlobalAlloc(GMEM_MOVEABLE | GMEM_ZEROINIT, (unsigned long long)count);
    if (!h) return 0;
    int slot = gm_alloc_slot();
    if (!slot) { GlobalFree(h); return 0; }
    gm_table[slot] = h;
    return slot;
}
long pb_globalmem_free(long slot) {
    if (slot <= 0 || slot >= 256) return slot;
    void* h = gm_table[slot];
    if (!h) return slot;
    void* r = GlobalFree(h);
    gm_table[slot] = NULL;
    return r ? slot : 0; /* success -> 0, failure -> original handle value */
}
void* pb_globalmem_lock(long slot) {
    if (slot <= 0 || slot >= 256) return NULL;
    return GlobalLock(gm_table[slot]);
}
long pb_globalmem_size(long slot) {
    if (slot <= 0 || slot >= 256) return 0;
    return (long)GlobalSize(gm_table[slot]);
}
long pb_globalmem_unlock(long slot) {
    if (slot <= 0 || slot >= 256) return 0;
    return GlobalUnlock(gm_table[slot]) ? 1 : 0; /* non-zero = still locked */
}

/* MOUSEPTR: style 0 = hide, 1..13 = stock cursors, other = fail closed.
 * Returns the previous HCURSOR handle in var& per CHW; 0 on failure. */
long pb_mouseptr(long style) {
    unsigned short idc = 0;
    switch (style) {
        case 0: ShowCursor(0); return 0;
        case 1: case 4: idc = 32512; break; /* IDC_ARROW */
        case 2: idc = 32515; break;         /* IDC_CROSS */
        case 3: idc = 32513; break;         /* IDC_IBEAM */
        case 5: idc = 32648; break;         /* IDC_SIZEALL */
        case 6: idc = 32645; break;         /* IDC_SIZENESW */
        case 7: idc = 32647; break;         /* IDC_SIZENS */
        case 8: idc = 32644; break;         /* IDC_SIZENWSE */
        case 9: idc = 32646; break;         /* IDC_SIZEWE */
        case 10: idc = 32516; break;        /* IDC_UPARROW */
        case 11: idc = 32514; break;        /* IDC_WAIT */
        case 12: ShowCursor(0); return 0;
        case 13: idc = 32651; break;        /* IDC_APPSTARTING */
        default: return 0;                  /* opaque cursor handle: fail closed */
    }
    void* h = LoadCursorA(NULL, (const char*)(size_t)idc);
    if (!h) return 0;
    /* SetCursor returns the HCURSOR of the previous cursor (NULL on failure),
     * which is exactly the CHW-documented value for TO var&. */
    return (long)(intptr_t)SetCursor(h);
}

/* UCODEPAGE: record codepage for future ANSI<->UNICODE conversions.
 * ANSI = CP_ACP(0), OEM = CP_OEMCP(1), numeric = explicit codepage. */
static long pb_ucodepage_cur = 0;
long pb_ucodepage(long cp) {
    long old = pb_ucodepage_cur;
    pb_ucodepage_cur = cp;
    return old;
}

/* UCODE$ / ACODE$ - the legacy byte-string ANSI <-> UNICODE pair.
 * UCODE$ takes ANSI bytes and returns the WIDE UNICODE bytes inside a byte
 * string, so the byte count doubles while the character count is unchanged.
 * ACODE$ is the exact inverse.  A codepage of -1 means "use the codepage
 * recorded by UCODEPAGE"; CP_ACP(0) and CP_OEMCP(1) are already valid Win32
 * codepage constants, so the recorded value needs no translation.
 * Lengths come from the BSTR prefix, not strlen: UTF-16LE ASCII has a 0x00
 * second byte, so strlen would stop after the first character. */
char* pb_ucode(const char* ansi, int codepage) {
    int cp = codepage;
    int alen, need, got;
    short* wbuf;
    char* out;
    if (!ansi) ansi = "";
    if (cp < 0) cp = (int)pb_ucodepage_cur;
    alen = pb_str_len(ansi);
    if (alen <= 0) return pb_bstr_alloc("", 0);
    need = MultiByteToWideChar((unsigned int)cp, 0, ansi, alen, NULL, 0);
    if (need <= 0) return pb_bstr_alloc("", 0);
    wbuf = (short*)malloc(sizeof(short) * need);
    if (!wbuf) return pb_bstr_alloc("", 0);
    got = MultiByteToWideChar((unsigned int)cp, 0, ansi, alen, wbuf, need);
    if (got <= 0) { free(wbuf); return pb_bstr_alloc("", 0); }
    out = pb_bstr_alloc((const char*)wbuf,
                        (unsigned int)(sizeof(short) * got));
    free(wbuf);
    return out;
}

char* pb_acode(const char* wide, int codepage) {
    int cp = codepage;
    int wchars, need, got;
    char* buf;
    char* out;
    if (!wide) wide = "";
    if (cp < 0) cp = (int)pb_ucodepage_cur;
    /* two bytes per wide character */
    wchars = pb_str_len(wide) / 2;
    if (wchars <= 0) return pb_bstr_alloc("", 0);
    need = WideCharToMultiByte((unsigned int)cp, 0, (const short*)wide, wchars,
                               NULL, 0, NULL, NULL);
    if (need <= 0) return pb_bstr_alloc("", 0);
    buf = (char*)malloc(need + 1);
    if (!buf) return pb_bstr_alloc("", 0);
    got = WideCharToMultiByte((unsigned int)cp, 0, (const short*)wide, wchars,
                              buf, need, NULL, NULL);
    if (got < 0) got = 0;
    out = pb_bstr_alloc(buf, (unsigned int)got);
    free(buf);
    return out;
}

/* Public BSTR free wrapper � called from LLVM IR codegen (cdecl) */
void pb_bstr_free(char* bstr) {
#ifdef _WIN32
    SysFreeString(bstr);
#else
    free(bstr);
#endif
}

/* Helper: allocate a BSTR from a C string buffer, then free the buffer */
static char* bstr_from_buf(char* buf) {
    int len = (int)strlen(buf);
    char* bstr = pb_bstr_alloc(buf, len);
    free(buf);
    return bstr;
}

char* pb_format_digits(double val, int digits);   /* forward decl */

/* ============================================================
 * FORMAT$  -  full CHW mask engine (batch 308)
 *
 * Two entry forms (selected in codegen by the 2nd argument type):
 *   pb_format_digits(val, digits)   digits& = 1..18 significant digits
 *   pb_format(val, fmt)             fmt$   = full mask system
 *
 * Mask characters (CHW FORMAT$ function):
 *   0        mandatory digit placeholder (zero-fill)
 *   #        optional digit placeholder (omitted, or filled by *x)
 *   *x       fill spec: two '#' fields; fill char x when no digit
 *   .        decimal placeholder
 *   ,        thousands separator (must follow a digit placeholder)
 *   %        multiply by 100, trailing percent
 *   E-/e-/E+/e+  scientific notation (between two digit placeholders)
 *   "..."    quoted literal
 *   \x       escaped literal
 *   $()+-    space: pass-through literals (no escape needed)
 *   ;        mask delimiter (1/2/3 masks: positive/negative/zero)
 *
 * Rounding: banker's rounding (IEEE ties-to-even) obtained from snprintf
 * under the default FE_TONEAREST rounding mode.
 *
 * Mask rule set (measured against PBWin10, batch308f probe, 40+ cases):
 *   digits form:  scientific when int-digit-count > digits, or |v|<0.1;
 *                 fixed otherwise, rounded to `digits` significant digits;
 *                 leading integer zero omitted, trailing fractional zeros
 *                 stripped; exponent has no leading zeros, non-negative
 *                 exponent carries '+'.
 *   mask form:    integer digits right-aligned into integer slots; '0' slots
 *                 zero-fill, '#' slots fill (when a *x spec exists) or
 *                 omit; fraction digits left-aligned, trailing '#' zeros
 *                 omitted/filled; overflow is displayed without truncation;
 *                 negative sign is prepended to the number when the mask
 *                 carries no literal '-'.
 * ============================================================ */

#define FMT_MAX_SLOTS 128

typedef struct {
    char  tok[FMT_MAX_SLOTS];   /* '0' '#' 'F' 'L' 'D' 'C' 'P' 'S' 'M' 'H' 'T' */
    char  lit[FMT_MAX_SLOTS];   /* literal / fill / sci-case / tail-sign char */
    int   n;
    int   int_slots, frac_slots;
    int   has_dot, comma, percent, sci;
    char  sci_case;             /* 'E' or 'e' */
    char  sci_sign;             /* '+' or '-' */
    int   exp_slots;
    int   has_fill;
    char  fillch;
    int   fill_specs;           /* count of '*x' fill specifiers in the mask */
    char  first_fillch;         /* fill char of the leftmost (first) fill spec */
    int   has_neg_lit;          /* mask contains a literal '-' (FORMAT$) */
    /* USING$ (um=1) only fields */
    int   has_dollar;           /* '$$' floating dollar present */
    int   head_sign;            /* leading '+' floating sign present */
    int   tail_sign;            /* trailing '+'/'-' fixed sign present */
    char  tail_sign_ch;         /* '+' or '-' for the trailing sign slot */
} fmt_mask_t;

/* Scan one mask (up to ';' or NUL). um=0 FORMAT$ rules, um=1 USING$ rules
 * (recognises '$$', leading/trailing sign slots and '^' scientific). Returns
 * chars consumed. */
static int fmt_scan_mask(const char* m, fmt_mask_t* fm, int um) {
    memset(fm, 0, sizeof(*fm));
    int i = 0, n = 0;
    int seen_digit = 0;          /* any digit-bearing slot encountered yet */
    fm->fillch = ' ';
    while (m[i] && m[i] != ';') {
        char c = m[i];
        if (c == '"') {
            i++;
            while (m[i] && m[i] != '"' && n < FMT_MAX_SLOTS) {
                fm->tok[n] = 'L'; fm->lit[n] = m[i]; n++; i++;
            }
            if (m[i] == '"') i++;
            continue;
        }
        if (c == '\\') {
            if (m[i + 1] && n < FMT_MAX_SLOTS) {
                fm->tok[n] = 'L'; fm->lit[n] = m[i + 1]; n++; i += 2;
            } else i++;
            continue;
        }
        if (c == '0' || c == '#') {
            if (n < FMT_MAX_SLOTS) {
                fm->tok[n] = c; n++;
                if (fm->has_dot) fm->frac_slots++; else fm->int_slots++;
            }
            seen_digit = 1;
            i++;
            continue;
        }
        if (c == '*') {
            char f = m[i + 1] ? m[i + 1] : ' ';
            if (n + 1 < FMT_MAX_SLOTS) {
                fm->tok[n] = 'F'; fm->lit[n] = f; n++;
                fm->tok[n] = 'F'; fm->lit[n] = f; n++;
            }
            if (fm->fill_specs == 0) fm->first_fillch = f;
            fm->fill_specs++;
            fm->has_fill = 1; fm->fillch = f;
            if (fm->has_dot) fm->frac_slots += 2; else fm->int_slots += 2;
            seen_digit = 1;
            i += 2;
            continue;
        }
        if (c == '.') {
            if (n < FMT_MAX_SLOTS) { fm->tok[n] = 'D'; n++; }
            fm->has_dot = 1; i++;
            continue;
        }
        if (c == ',') {
            if (n < FMT_MAX_SLOTS) { fm->tok[n] = 'C'; n++; }
            fm->comma = 1; i++;
            continue;
        }
        if (c == '%') {
            if (n < FMT_MAX_SLOTS) { fm->tok[n] = 'P'; n++; }
            fm->percent = 1; i++;
            continue;
        }
        if ((c == 'E' || c == 'e') && (m[i + 1] == '+' || m[i + 1] == '-')) {
            if (n < FMT_MAX_SLOTS) { fm->tok[n] = 'S'; fm->lit[n] = c; n++; }
            fm->sci = 1; fm->sci_case = c; fm->sci_sign = m[i + 1];
            i += 2;
            continue;
        }
        /* ---- USING$ numeric mask rules (um=1) ---- */
        if (um) {
            if (c == '$' && m[i + 1] == '$') {
                /* floating dollar: two integer slots; the dollar renders next
                 * to the first significant digit at apply time */
                if (n + 1 < FMT_MAX_SLOTS) { fm->tok[n] = 'M'; n++; fm->tok[n] = 'M'; n++; }
                fm->has_dollar = 1; fm->int_slots += 2; seen_digit = 1;
                i += 2;
                continue;
            }
            if (c == '^') {
                /* run of 3-6 carets: first two map to E + sign, the rest to
                 * exponent digit slots */
                int nc = 0;
                while (m[i + nc] == '^' && nc < 6) nc++;
                if (nc < 3) nc = 3;
                fm->sci = 1; fm->sci_case = 'E'; fm->sci_sign = '+';
                if (n < FMT_MAX_SLOTS) { fm->tok[n] = 'S'; fm->lit[n] = 'E'; n++; }
                fm->exp_slots = nc - 2;
                for (int z = 0; z < nc - 2 && n < FMT_MAX_SLOTS; z++) { fm->tok[n] = '#'; n++; }
                i += nc;
                continue;
            }
            if (c == '+') {
                if (!seen_digit) {
                    /* leading floating sign: sign renders next to the digits */
                    if (n < FMT_MAX_SLOTS) { fm->tok[n] = 'H'; n++; }
                    fm->head_sign = 1; fm->int_slots++; seen_digit = 1;
                } else {
                    /* trailing fixed sign slot */
                    if (n < FMT_MAX_SLOTS) { fm->tok[n] = 'T'; fm->lit[n] = '+'; n++; }
                    fm->tail_sign = 1; fm->tail_sign_ch = '+';
                }
                i++;
                continue;
            }
            if (c == '-') {
                /* trailing fixed sign: '-' for negative, space for positive */
                if (n < FMT_MAX_SLOTS) { fm->tok[n] = 'T'; fm->lit[n] = '-'; n++; }
                fm->tail_sign = 1; fm->tail_sign_ch = '-';
                i++;
                continue;
            }
            if (c == '$') {
                /* lone '$' (no pair): pass through as a literal */
                if (n < FMT_MAX_SLOTS) { fm->tok[n] = 'L'; fm->lit[n] = '$'; n++; }
                i++;
                continue;
            }
        }
        if (c == '$' || c == '(' || c == ')' || c == '+' || c == '-' || c == ' ') {
            if (n < FMT_MAX_SLOTS) { fm->tok[n] = 'L'; fm->lit[n] = c; n++; }
            if (c == '-') fm->has_neg_lit = 1;
            i++;
            continue;
        }
        /* unknown literal: pass through (CHW requires escaping; tolerated) */
        if (n < FMT_MAX_SLOTS) { fm->tok[n] = 'L'; fm->lit[n] = c; n++; }
        i++;
    }
    fm->n = n;
    return i;
}

/* Append a slot char: digit, '0' zero-fill, '#/F' fill, '#' omit. */
static void fmt_slot_char(char kind, const fmt_mask_t* fm, int has_digit,
                          char d, char* out, int* oi, int cap) {
    if (has_digit) {
        if (*oi < cap - 1) out[(*oi)++] = d;
        return;
    }
    if (kind == '0') {
        if (*oi < cap - 1) out[(*oi)++] = '0';
        return;
    }
    if (kind == 'F' || fm->has_fill) {
        if (*oi < cap - 1) out[(*oi)++] = fm->fillch;
        return;
    }
    /* '#' without fill: omit */
}

/* Insert thousands separators into a plain digit string (right-to-left 3s). */
static void fmt_thousands(const char* in, char* out, int cap) {
    int len = (int)strlen(in);
    int ncom = (len - 1) / 3;
    int olen = len + ncom;
    if (olen >= cap) olen = cap - 1;
    int oi = olen;
    out[oi] = 0;
    int cnt = 0;
    for (int i = len - 1; i >= 0; i--) {
        if (cnt == 3 && oi > 0) {
            out[--oi] = ',';
            cnt = 0;
        }
        if (oi > 0) out[--oi] = in[i];
        cnt++;
    }
    if (oi > 0) memmove(out, out + oi, olen - oi + 1);
}

/* Fraction digits with trailing '#'-slot zeros trimmed ('0' slots keep). */
static void fmt_frac_trim(char* frac, const fmt_mask_t* fm) {
    int i = fm->frac_slots - 1;
    while (i >= 0 && frac[i] == '0') {
        /* find kind of frac slot i */
        int slotidx = 0, seen_dot = 0;
        for (int t = 0; t < fm->n; t++) {
            char k = fm->tok[t];
            if (k == 'D') { seen_dot = 1; continue; }
            if (!seen_dot || (k != '0' && k != '#' && k != 'F')) continue;
            if (slotidx == i) {
                if (k != '0' && k != 'F') frac[i] = 0;  /* trim '#' zero */
                break;
            }
            slotidx++;
        }
        i--;
    }
}

/* Return the mask kind of the idx'th integer-region digit slot. */
static char fmt_int_kind(const fmt_mask_t* fm, int idx) {
    int c = 0;
    for (int t = 0; t < fm->n; t++) {
        char k = fm->tok[t];
        if (k == 'D' || k == 'S') break;
        if (k == '0' || k == '#' || k == 'F' || k == 'M' || k == 'H') {
            if (c == idx) return k;
            c++;
        }
    }
    return '#';
}

/* Build per-slot display chars for the integer region (digits right-aligned,
 * '0' zero-fill, '#' omit, 'F'/has-fill fill).  Returns the overflow digit
 * count (leading digits that do not fit in the slots); the caller emits the
 * overflow run (digits[0..ovf) + slot chars) as one grouped string. */
static int fmt_int_slots(const fmt_mask_t* fm, const char* digits, char* slots, int cap) {
    int n = fm->int_slots;
    int j = (int)strlen(digits) - 1;
    for (int e = n - 1; e >= 0; e--) {
        if (j >= 0) {
            slots[e] = digits[j--];
        } else {
            char k = fmt_int_kind(fm, e);
            if (k == '0') slots[e] = '0';
            else if (k == 'F') slots[e] = fm->fillch;
            else if (k == '#' && fm->has_fill) {
                /* #-region empty slot: with 2+ fill specs the char that floats
                 * in front is the first (leftmost) fill char; a single fill
                 * spec uses its own fill char. */
                slots[e] = fm->fill_specs >= 2 ? fm->first_fillch : fm->fillch;
            }
            else slots[e] = 0;
        }
    }
    int ovf = j + 1;
    if (ovf > 0 && ovf + n > cap) ovf = cap - n;
    return ovf;
}

/* Emit the integer region as one run: overflow digits first, then the
 * non-fill slot chars.  With comma=1 the whole digit run is thousands-
 * grouped (fill characters never participate in grouping). */
static void fmt_emit_int_run(const fmt_mask_t* fm, const char* digits, int ovf,
                             const char* islots, int ilen,
                             char* out, int* oi) {
    char run[FMT_MAX_SLOTS * 2 + 64];
    int rn = 0;
    for (int i = 0; i < ovf; i++) run[rn++] = digits[i];
    for (int e = 0; e < ilen; e++) {
        char c = islots[e];
        if (!c) continue;
        char kk = fmt_int_kind(fm, e);
        if (kk == 'F' || (kk == '#' && fm->has_fill)) {
            if (c == fm->fillch) continue;   /* fill stays out of the digit run */
        }
        run[rn++] = c;
    }
    run[rn] = 0;
    if (fm->comma) {
        char gbuf[600];
        fmt_thousands(run, gbuf, (int)sizeof(gbuf));
        for (int i = 0; gbuf[i]; i++) out[(*oi)++] = gbuf[i];
    } else {
        for (int i = 0; i < rn; i++) out[(*oi)++] = run[i];
    }
}

/* Build per-slot display chars for the fraction region (left-aligned). */
static int fmt_frac_slots(const fmt_mask_t* fm, const char* frac_digits,
                          char* slots, int cap) {
    int n = fm->frac_slots;
    int flen = (int)strlen(frac_digits);
    int fi = 0;
    for (int e = 0; e < n; e++) {
        if (fi < flen) {
            slots[e] = frac_digits[fi++];
        } else {
            /* find kind of frac slot e */
            int idx = 0, seen_dot = 0;
            int done = 0;
            for (int t = 0; t < fm->n; t++) {
                char k = fm->tok[t];
                if (k == 'D') { seen_dot = 1; continue; }
                if (k == 'S') break;
                if (!seen_dot || (k != '0' && k != '#' && k != 'F')) continue;
                if (idx == e) {
                    if (k == '0') { slots[e] = '0'; }
                    else if (k == 'F' || fm->has_fill) { slots[e] = fm->fillch; }
                    else slots[e] = 0;
                    done = 1;
                    break;
                }
                idx++;
            }
            if (!done) slots[e] = 0;
        }
    }
    return n;
}

/* build the formatted string for one mask.
 * um=0 FORMAT$ semantics (leading '#' slots omitted, trailing fraction '#'
 * zeros trimmed); um=1 USING$ fixed-width semantics (leading integer '#'
 * slots render as spaces, empty fraction '#' slots render as zeros). */
static char* fmt_apply(const fmt_mask_t* fm, double val, int um) {
    char out[FMT_MAX_SLOTS * 2 + 64];
    int oi = 0;
    int neg = val < 0;
    double av = val < 0 ? -val : val;
    if (fm->percent) av *= 100.0;

    char fixed[1024];
    if (fm->sci) {
        /* Scientific mantissa always has one integer digit plus frac_slots
         * fraction digits, so the %e precision is frac_slots. This lets the
         * library round the last kept digit (e.g. 123456 -> 1.2346E+5). */
        int prec = fm->frac_slots;
        if (prec < 0) prec = 0;
        char ebuf[160];
        snprintf(ebuf, sizeof(ebuf), "%.*e", prec, av);
        char mant[96]; char esign; int expo;
        sscanf(ebuf, "%95[^e]e%c%d", mant, &esign, &expo);
        char mdig[96]; int md = 0;
        for (int k = 0; mant[k]; k++) if (mant[k] != '.') mdig[md++] = mant[k];
        mdig[md] = 0;
        /* integer slots: mantissa int digit (1) right-aligned */
        char islots[FMT_MAX_SLOTS];
        char idig[2] = { mdig[0], 0 };
        int ovf = fmt_int_slots(fm, idig, islots, (int)sizeof(islots));
        int ilen = fm->int_slots;
        /* fraction slots: mantissa digits mdig[1..] */
        char fslots[FMT_MAX_SLOTS];
        int flen = 0;
        {
            int mj = 1;
            int e = 0, idx = 0, seen_dot = 0;
            for (int t = 0; t < fm->n; t++) {
                char k = fm->tok[t];
                if (k == 'D') { seen_dot = 1; continue; }
                if (k == 'S') break;
                if (!seen_dot || (k != '0' && k != '#' && k != 'F')) continue;
                if (mj < md) { fslots[e] = mdig[mj++]; flen++; }
                else {
                    if (k == '0') { fslots[e] = '0'; flen++; }
                    else if (k == 'F' || fm->has_fill) { fslots[e] = fm->fillch; flen++; }
                    else fslots[e] = 0;
                }
                e++;
            }
        }
        /* exponent slots: right-aligned */
        char expo_str[32];
        int ea = expo < 0 ? -expo : expo;
        snprintf(expo_str, sizeof(expo_str), "%d", ea);
        char eslots[FMT_MAX_SLOTS];
        int elen = fm->exp_slots;   /* right-aligned: count ALL slots (omit slots included) */
        {
            int ej = (int)strlen(expo_str) - 1;
            int e = fm->exp_slots - 1;
            while (e >= 0) {
                if (ej >= 0) { eslots[e] = expo_str[ej--]; }
                else {
                    /* exp slots are '#'-like (omit) or '0' (fill): find kind */
                    int idx = 0, seen_s = 0;
                    for (int t = 0; t < fm->n; t++) {
                        char k = fm->tok[t];
                        if (k == 'S') { seen_s = 1; continue; }
                        if (!seen_s || (k != '0' && k != '#' && k != 'F')) continue;
                        if (idx == e) {
                            if (k == '0') { eslots[e] = '0'; }
                            else if (k == 'F' || fm->has_fill) { eslots[e] = fm->fillch; }
                            else eslots[e] = 0;
                            break;
                        }
                        idx++;
                    }
                }
                e--;
            }
        }
        /* assemble */
        int neg_out = 0;
        int ii = 0, ff = 0, ee = 0, phase = 0;   /* 0 int, 1 frac, 2 exp */
        for (int t = 0; t < fm->n; t++) {
            char k = fm->tok[t];
            if (k == 'L') out[oi++] = fm->lit[t];
            else if (k == 'D') {
                if (neg && !neg_out && !fm->has_neg_lit) { out[oi++] = '-'; neg_out = 1; }
                out[oi++] = '.'; phase = 1;
            }
            else if (k == 'S') {
                if (neg && !neg_out && !fm->has_neg_lit) { out[oi++] = '-'; neg_out = 1; }
                phase = 2;
                out[oi++] = fm->sci_case;
                if (esign == '-') out[oi++] = '-';      /* %e sign char, exponent value non-negative */
                else if (fm->sci_sign == '+') out[oi++] = '+';  /* E+/e+ adds '+' on non-negative */
            }
            else if (k == 'P') out[oi++] = '%';
            else if (k == '0' || k == '#' || k == 'F') {
                if (neg && !neg_out && !fm->has_neg_lit) { out[oi++] = '-'; neg_out = 1; }
                if (phase == 0) {
                    if (ii < ilen) {
                        char c = islots[ii];
                        ii++;
                        if (c) out[oi++] = c;
                        else if (um) out[oi++] = ' ';
                    }
                }
                else if (phase == 1) {
                    if (ff < flen) {
                        if (fslots[ff]) out[oi++] = fslots[ff];
                        else if (um) out[oi++] = '0';
                        ff++;
                    }
                }
                else if (phase == 2) {
                    if (ee < elen && eslots[ee]) out[oi++] = eslots[ee++];
                    else if (um && ee < elen) { out[oi++] = '0'; ee++; }
                    else ee++;
                }
            }
        }
        out[oi] = 0;
        {
            char* r = (char*)malloc(oi + 1);
            memcpy(r, out, oi + 1);
            return bstr_from_buf(r);
        }
    }

    /* ---- fixed point ---- */
    int nfrac = fm->frac_slots;
    snprintf(fixed, sizeof(fixed), "%.*f", nfrac, av);
    char int_raw[512], frac_raw[512];
    {
        char* dot = strchr(fixed, '.');
        if (dot) {
            size_t il = (size_t)(dot - fixed);
            memcpy(int_raw, fixed, il); int_raw[il] = 0;
            strcpy(frac_raw, dot + 1);
        } else {
            strcpy(int_raw, fixed);
            frac_raw[0] = 0;
        }
    }
    if (fm->frac_slots > 0) fmt_frac_trim(frac_raw, fm);

    char islots[FMT_MAX_SLOTS];
    int ovf = fmt_int_slots(fm, int_raw, islots, (int)sizeof(islots));
    int ilen = fm->int_slots;
    char fslots[FMT_MAX_SLOTS];
    int flen = fmt_frac_slots(fm, frac_raw, fslots, (int)sizeof(fslots));

    /* Floating prefix slots (USING$ '$$' and/or leading '+'). The prefix is
     * rendered immediately in front of the digits; dollar is closest. When
     * digits + prefix exceed the integer width the whole prefix is prepended
     * to the overflow run instead. */
    int dlen_f = (int)strlen(int_raw);
    int pref_dollar = fm->has_dollar ? 1 : 0;
    int pref_sign = fm->head_sign ? 1 : 0;
    int pref_neg = (neg && !fm->head_sign && fm->has_dollar) ? 1 : 0;
    int pref_total = pref_dollar + pref_sign + pref_neg;
    int pref_overflow = (dlen_f + pref_total) > ilen;
    int dollar_slot = -1, sign_slot = -1, neg_slot = -1;
    if (!pref_overflow) {
        int cursor = ilen - dlen_f - 1;
        if (pref_dollar) { dollar_slot = cursor; cursor--; }
        if (pref_sign) { sign_slot = cursor; cursor--; }
        if (pref_neg) { neg_slot = cursor; cursor--; }
    }

    /* Fill-spec folding. With 2+ '*x' specifiers the leading fill slots that
     * fall before the right-aligned digit run collapse (no output), and the
     * empty #-region slots render with the first fill char. With a single fill
     * spec the field stays full width (fill slots render). */
    int fcnt = fm->fill_specs * 2;             /* total 'F' integer slots */
    int hash_w = ilen - fcnt;                  /* true #/0 integer slots */
    int fill_fold = um && fm->has_fill && fm->fill_specs >= 2;
    int int_start = 0;
    if (fill_fold) {
        int ds = ilen - dlen_f;
        if (ds < 0) ds = 0;
        int ds2 = ds < fcnt ? ds : fcnt;
        int_start = ds2;
    }

    int ii = 0, ff = 0, phase = 0;   /* 0 int, 1 frac */
    char dbuf[FMT_MAX_SLOTS * 2 + 64];
    int dn = 0;
    int ovf_emitted = 0;
    int neg_out = 0;
    for (int t = 0; t < fm->n; t++) {
        char k = fm->tok[t];
        if (k == 'L') out[oi++] = fm->lit[t];
        else if (k == 'D') {
            if (neg && !neg_out && !fm->has_neg_lit) { out[oi++] = '-'; neg_out = 1; }
            if (fm->comma && dn > 0) {
                dbuf[dn] = 0;
                char gbuf[600];
                fmt_thousands(dbuf, gbuf, (int)sizeof(gbuf));
                for (int i = 0; gbuf[i]; i++) out[oi++] = gbuf[i];
                dn = 0;
            }
            out[oi++] = '.'; phase = 1;
        }
        else if (k == 'P') out[oi++] = '%';
        else if (k == 'C') {
            if (!ovf_emitted && ovf > 0) {
                fmt_emit_int_run(fm, int_raw, ovf, islots, ilen, out, &oi);
                ovf_emitted = 1;
                ii = ilen;
            }
        }
        else if (k == 'M' || k == 'H') {
            /* Floating dollar ('M') or leading floating sign ('H'). In the
             * non-overflow case the prefix slot may sit on this token or on a
             * later digit slot; only an exact match here renders the prefix. */
            if (phase == 0 && !pref_overflow) {
                if (ii == dollar_slot) out[oi++] = '$';
                else if (ii == neg_slot) out[oi++] = '-';
                else if (ii == sign_slot) out[oi++] = neg ? '-' : '+';
                else out[oi++] = ' ';
            } else if (phase == 0) {
                /* overflow prefix is emitted before the overflow run */
                ii++;
                continue;
            }
            ii++;
        }
        else if (k == 'T') {
            /* Trailing fixed sign slot: '+' shows +/- , '-' shows '-' or space. */
            if (fm->tail_sign_ch == '+') out[oi++] = neg ? '-' : '+';
            else out[oi++] = neg ? '-' : ' ';
        }
        else if (k == '0' || k == '#' || k == 'F') {
            if (neg && !neg_out && !fm->has_neg_lit && !pref_dollar && !pref_sign &&
                !fm->tail_sign) {
                out[oi++] = '-'; neg_out = 1;
            }
            if (phase == 0) {
                if (pref_overflow && !ovf_emitted) {
                    if (pref_neg) out[oi++] = '-';
                    if (pref_sign) out[oi++] = neg ? '-' : '+';
                    if (pref_dollar) out[oi++] = '$';
                    fmt_emit_int_run(fm, int_raw, ovf, islots, ilen, out, &oi);
                    ovf_emitted = 1;
                    ii = ilen;
                } else if (!ovf_emitted && ovf > 0) {
                    fmt_emit_int_run(fm, int_raw, ovf, islots, ilen, out, &oi);
                    ovf_emitted = 1;
                    ii = ilen;
                } else if (ii < ilen) {
                    if (ii < int_start) { ii++; }        /* folded leading fill slot */
                    else if (ii == dollar_slot) { out[oi++] = '$'; ii++; }
                    else if (ii == neg_slot) { out[oi++] = '-'; ii++; }
                    else if (ii == sign_slot) { out[oi++] = neg ? '-' : '+'; ii++; }
                    else {
                        char c = islots[ii];
                        char kk = fmt_int_kind(fm, ii);
                        int slot = ii;
                        ii++;
                        if (!c) {
                            /* empty integer slot: in the # region with fill
                             * folding use the first fill char, else space */
                            if (fill_fold && slot >= fcnt) out[oi++] = fm->first_fillch;
                            else if (um) out[oi++] = ' ';
                        }
                        else if ((kk == 'F' || (kk == '#' && fm->has_fill)) &&
                                 (c == fm->fillch || c == fm->first_fillch)) {
                            out[oi++] = c;              /* fill stays out of the digit run */
                        }
                        else if (fm->comma) dbuf[dn++] = c;
                        else out[oi++] = c;
                    }
                }
            } else {
                if (ff < flen) {
                    if (fslots[ff]) out[oi++] = fslots[ff];
                    else if (um) out[oi++] = '0';  /* USING: empty fraction '#' = zero */
                    ff++;
                }
            }
        }
    }
    if (fm->comma && dn > 0) {
        dbuf[dn] = 0;
        char gbuf[600];
        fmt_thousands(dbuf, gbuf, (int)sizeof(gbuf));
        for (int i = 0; gbuf[i]; i++) out[oi++] = gbuf[i];
    }
    out[oi] = 0;
    {
        char* r = (char*)malloc(oi + 1);
        memcpy(r, out, oi + 1);
        return bstr_from_buf(r);
    }
}

/* Shared mask split/select for FORMAT$ (um=0) and USING$ numeric fields
 * (um=1 fixed-width). The mask rules are identical except for how empty
 * integer/fraction '#' slots render (see fmt_apply). */
static char* format_with_mode(double val, const char* fmt, int um) {
    if (fmt == NULL || fmt[0] == '\0') {
        return pb_format_digits(val, 0);   /* default significant digits */
    }
    /* split masks at top-level ';' (quotes/escapes protected) */
    const char* m[3] = { fmt, NULL, NULL };
    int nm = 1;
    {
        int inq = 0;
        for (const char* p = fmt; *p; p++) {
            if (*p == '\\' && p[1]) { p++; continue; }
            if (*p == '"') { inq = !inq; continue; }
            if (*p == ';' && !inq && nm < 3) { m[nm++] = p + 1; }
        }
    }
    fmt_mask_t fm[3];
    for (int k = 0; k < nm; k++) {
        fmt_scan_mask(m[k], &fm[k], um);
        fm[k].exp_slots = 0;
        int seen_s = 0;
        for (int t = 0; t < fm[k].n; t++) {
            if (fm[k].tok[t] == 'S') seen_s = 1;
            else if (seen_s && (fm[k].tok[t] == '0' || fm[k].tok[t] == '#' || fm[k].tok[t] == 'F')) {
                fm[k].exp_slots++;
            }
        }
    }
    const fmt_mask_t* use = &fm[0];
    if (val < 0 && nm >= 2) use = &fm[1];
    if (val == 0 && nm >= 3) use = &fm[2];
    return fmt_apply(use, val, um);
}

char* pb_format(double val, const char* fmt) {
    return format_with_mode(val, fmt, 0);
}

/* USING$ numeric field: fixed-width, leading '#' spaces / trailing zeros. */
char* pb_format_using(double val, const char* fmt) {
    return format_with_mode(val, fmt, 1);
}

/* digits form: 1..18 significant digits (default 16 for fork's double) */
char* pb_format_digits(double val, int digits) {
    if (digits < 1) digits = 16;      /* CHW default 7 single / 16 precise */
    if (digits > 18) digits = 18;
    if (val == 0) {
        char* r = (char*)malloc(2);
        r[0] = '0'; r[1] = 0;
        return bstr_from_buf(r);
    }
    int neg = val < 0;
    double av = val < 0 ? -val : val;
    /* scientific threshold (PBWin measured):
     *   large: integer digit count > digits
     *   small: at least one leading zero after the decimal point, and
     *          the decimal exponent e satisfies e <= 1-digits
     *     e.g. 0.0123 digits=3 -> e=-2 <= -2 -> sci "1.2E-2"
     *          0.0123 digits=4 -> e=-2 <= -3 false -> fixed ".0123"
     *          0.123  digits=3 -> z=0 -> fixed ".123" */
    int sci = 0;
    int ip = (int)floor(log10(av)) + 1;   /* integer digit count (>=1) */
    if (ip > digits) {
        sci = 1;
    } else if (av > 0 && av < 1.0) {
        int e = (int)floor(log10(av));
        /* at least one leading zero (e <= -2) and e <= 1-digits */
        if (e <= -2 && e <= 1 - digits) sci = 1;
    }
    char buf[512];
    if (sci) {
        snprintf(buf, sizeof(buf), "%.*e", digits - 1, av);
        char mant[64], esign;
        int expo;
        sscanf(buf, "%63[^e]e%c%d", mant, &esign, &expo);
        /* strip trailing zeros of mantissa */
        int ml = (int)strlen(mant);
        while (ml > 0 && mant[ml - 1] == '0') ml--;
        if (ml > 0 && mant[ml - 1] == '.') ml--;
        mant[ml] = 0;
        char ebuf[24];
        snprintf(ebuf, sizeof(ebuf), "%c%d", esign, expo);   /* keep %e's sign char */
        char* r = (char*)malloc(2 + strlen(mant) + 1 + strlen(ebuf) + 2);
        char* o = r;
        if (neg) *o++ = '-';
        memcpy(o, mant, strlen(mant)); o += strlen(mant);
        *o++ = 'E';
        memcpy(o, ebuf, strlen(ebuf)); o += strlen(ebuf);
        *o = 0;
        return bstr_from_buf(r);
    }
    /* fixed: digits significant digits (C %g; sci already handled above) */
    int n = snprintf(buf, sizeof(buf), "%.*g", digits, av);
    if (strchr(buf, 'e') != NULL || strchr(buf, 'E') != NULL) {
        pb_expand_fixed(buf, &n);           /* "1e+06" -> "1000000" (boundary) */
        buf[n] = 0;
    }
    /* strip trailing zeros in the fraction only (integers keep theirs) */
    char* dot2 = strchr(buf, '.');
    if (dot2) {
        char* p = buf + strlen(buf) - 1;
        while (p > dot2 && *p == '0') { *p = 0; p--; }
        if (p == dot2) *p = 0;   /* drop the dot too */
    }
    /* strip leading integer zero: 0.xx -> .xx (CHW: no leading space) */
    if (buf[0] == '0' && (buf[1] == '.' || buf[1] == 0)) {
        memmove(buf, buf + 1, strlen(buf));
    }
    char* r = (char*)malloc(strlen(buf) + 2);
    char* o = r;
    if (neg) *o++ = '-';
    memcpy(o, buf, strlen(buf) + 1);
    return bstr_from_buf(r);
}
/* ============================================================
 * PARSE$ � Split string by delimiter, extract Nth field (1-based)
 *
 * PARSE$(string, delimiter, index)  � returns the index'th field
 * PARSE$(string, delimiter)          � returns field count
 *
 * When index <= 0, returns the count of fields as a string-encoded integer.
 * When called with 3 args from PB: index is the field number (1-based).
 * ============================================================ */
/* ============================================================
 * PARSE$ / PARSECOUNT shared field walker
 *
 * Walk the fields of str per mode and record the processed field ranges
 * (pointers into str) in fstart[]/fend[], up to cap. Returns the total
 * number of fields (which may exceed cap). Modes:
 *   0 exact delimiter (empty/NULL delim -> default comma, mode 3)
 *   1 ANY (any single char in delim is a separator)
 *   2 BINARY (packed WORD/DWORD length-prefixed fields)
 *   3 default comma (optionally quoted; unquoted trimmed)
 * ============================================================ */
#define PB_PARSE_MAX_FIELDS 1024
static int pb_split_fields(const char* str, const char* delim, int mode,
                           const char** fstart, const char** fend, int cap) {
    if (str == NULL) str = "";
    if (delim == NULL) delim = "";
    int dlen = (int)strlen(delim);
    if (mode == 0 && dlen == 0) mode = 3;

    int n = 0;

    /* ---- BINARY: each field is length-prefixed ---- */
    if (mode == 2) {
        size_t total_bytes = pb_str_len(str);
        const char* p = str;
        const char* end = str + total_bytes;
        while (p < end) {
            unsigned int flen;
            unsigned short wlen;
            memcpy(&wlen, p, sizeof(wlen));
            p += sizeof(wlen);
            if (wlen == 65535) {
                unsigned long dword;
                memcpy(&dword, p, sizeof(dword));
                p += sizeof(dword);
                flen = dword;
            } else {
                flen = wlen;
            }
            if (n < cap) { fstart[n] = p; fend[n] = p + flen; }
            n++;
            p += flen;
        }
        return n;
    }

    /* ---- text modes ---- */
    const char* p = str;
    const char* start = str;
    while (1) {
        int at_delim = 0;
        int adv = 0;
        if (mode == 1) {
            if (dlen > 0 && memchr(delim, *p, dlen) != NULL) { at_delim = 1; adv = 1; }
        } else if (mode == 0) {
            if (dlen > 0 && strncmp(p, delim, dlen) == 0) { at_delim = 1; adv = dlen; }
        } else if (mode == 3) {
            if (*p == ',') { at_delim = 1; adv = 1; }
        }
        int at_end = (*p == '\0');

        if (at_delim || at_end) {
            const char* fs = start;
            const char* fe = p;
            if (mode == 3) {
                /* default comma: quoted field -> inner content; else trim */
                if (fs < fe && *fs == '"') {
                    const char* q = fs + 1;
                    while (q < fe && *q != '"') q++;
                    fs = fs + 1;
                    fe = q;                      /* chars after quote discarded */
                } else {
                    while (fs < fe && (*fs == ' ' || *fs == '"')) fs++;
                    while (fe > fs && (*(fe - 1) == ' ' || *(fe - 1) == '"')) fe--;
                }
            }
            if (n < cap) { fstart[n] = fs; fend[n] = fe; }
            n++;
            if (at_end) break;
            p += adv;
            start = p;
        } else {
            p++;
        }
    }
    return n;
}

/* Helper: build a BSTR from a field range [fs, fe). */
static char* pb_field_bstr(const char* fs, const char* fe) {
    return pb_bstr_alloc(fs, (unsigned int)(fe - fs));
}

char* pb_parse(const char* str, const char* delim, int mode, int index) {
    const char* fstart[PB_PARSE_MAX_FIELDS];
    const char* fend[PB_PARSE_MAX_FIELDS];
    int total = pb_split_fields(str, delim, mode, fstart, fend, PB_PARSE_MAX_FIELDS);

    /* CHW: index 0 always returns empty; negative counts from the right. */
    if (index == 0) return pb_bstr_alloc("", 0);
    int wanted = index;
    if (index < 0) wanted = total + index + 1;
    if (wanted < 1 || wanted > total) return pb_bstr_alloc("", 0);
    return pb_field_bstr(fstart[wanted - 1], fend[wanted - 1]);
}

/* ============================================================
 * PARSECOUNT - Return number of fields in a delimited string
 * ============================================================ */
int pb_parsecount(const char* str, const char* delim, int mode) {
    const char* fstart[PB_PARSE_MAX_FIELDS];
    const char* fend[PB_PARSE_MAX_FIELDS];
    return pb_split_fields(str, delim, mode, fstart, fend, PB_PARSE_MAX_FIELDS);
}

/* ============================================================
 * PARSE statement - split a whole string into successive array elements
 *
 *   mode 0: exact delimiter (empty/NULL delim -> default comma mode)
 *   mode 1: ANY (any single char in delim is a separator)
 *   mode 2: BINARY (packed WORD/DWORD length-prefixed fields)
 *   mode 3: default comma (optionally quoted; unquoted trimmed)
 *
 * target is the address of the first destination BSTR slot; capacity is the
 * number of slots available. Fields beyond capacity are ignored.
 * ============================================================ */
void pb_parse_array(const char* str, const char* delim, int mode,
                    char** target, int capacity) {
    if (target == NULL || capacity <= 0) return;
    if (str == NULL) str = "";
    if (delim == NULL) delim = "";

    int filled = 0;

    /* Store one field given its raw bytes [fs, fe). The codegen's string model
       never frees old slots (a slot may hold a static literal, which
       SysFreeString would corrupt), so overwrite the pointer like pb_replace. */
    #define PB_STORE_FIELD(fs, fe) do {                                   \
        if (filled < capacity) {                                          \
            target[filled] = pb_bstr_alloc((fs),                          \
                (unsigned int)((fe) - (fs)));                             \
            filled++;                                                     \
        }                                                                \
    } while (0)

    if (mode == 2) {
        /* BINARY: packed length-prefixed fields. */
        const char* p = str;
        while (*p != '\0') {
            const unsigned char* q = (const unsigned char*)p;
            unsigned int flen = (unsigned int)(
                (unsigned char)q[0] | ((unsigned char)q[1] << 8));
            q += 2;
            if (flen == 65535) {
                flen = (unsigned int)(
                    (unsigned char)q[0] | ((unsigned char)q[1] << 8)
                    | ((unsigned char)q[2] << 16) | ((unsigned char)q[3] << 24));
                q += 4;
            }
            PB_STORE_FIELD((const char*)q, (const char*)q + flen);
            p = (const char*)(q + flen);
            if (filled >= capacity) break;
        }
        return;
    }

    if (mode == 3 || (mode == 0 && delim[0] == '\0')) {
        /* Default comma-delimited, optionally quoted. */
        const char* p = str;
        while (1) {
            const char* q = p;
            while (*q == ' ') q++;
            if (*q == '"') {
                q++;  /* opening quote */
                char* buf = (char*)malloc(strlen(q) + 1);
                int n = 0;
                while (*q != '\0') {
                    if (*q == '"') {
                        if (*(q + 1) == '"') { buf[n++] = '"'; q += 2; continue; }
                        q++;  /* closing quote */
                        break;
                    }
                    buf[n++] = *q++;
                }
                PB_STORE_FIELD(buf, buf + n);
                free(buf);
                /* Discard any characters until the next comma / end. */
                while (*q != '\0' && *q != ',') q++;
                if (*q == ',') {
                    p = q + 1;
                    if (filled >= capacity) break;
                } else {
                    break;
                }
            } else {
                /* Unquoted: collect to comma, then trim surrounding spaces. */
                const char* start = p;
                const char* e = p;
                while (*e != '\0' && *e != ',') e++;
                const char* ts = start;
                const char* te = e;
                while (ts < e && *ts == ' ') ts++;
                while (te > ts && *(te - 1) == ' ') te--;
                PB_STORE_FIELD(ts, te);
                if (*e == ',') {
                    p = e + 1;
                    if (filled >= capacity) break;
                } else {
                    break;
                }
            }
        }
        return;
    }

    /* Exact (mode 0, non-empty delim) or ANY (mode 1). */
    int delim_len = (int)strlen(delim);
    const char* p = str;
    while (1) {
        const char* fs = p;
        int found = 0;
        while (*p != '\0') {
            int hit = 0;
            if (mode == 1) {
                for (int i = 0; i < delim_len; i++) {
                    if (*p == delim[i]) { hit = 1; break; }
                }
            } else if (strncmp(p, delim, delim_len) == 0) {
                hit = 1;
            }
            if (hit) { found = 1; break; }
            p++;
        }
        PB_STORE_FIELD(fs, p);
        if (!found) break;
        p += (mode == 1) ? 1 : delim_len;
        if (filled >= capacity) break;
        /* A trailing delimiter yields one final empty field. */
        if (*p == '\0') {
            PB_STORE_FIELD(p, p);
            break;
        }
    }
}
#undef PB_STORE_FIELD

/* ============================================================
 * REPLACE � Replace all occurrences of old_str with new_str in target
 *
 * Modifies the target string pointer in-place (PB semantics).
 * ============================================================ */
void pb_replace(char** target, const char* old_str, const char* new_str) {
    if (target == NULL || *target == NULL || old_str == NULL || new_str == NULL) return;

    int old_len = (int)strlen(old_str);
    int new_len = (int)strlen(new_str);
    if (old_len == 0) return;

    /* Count occurrences */
    int count = 0;
    const char* p = *target;
    while ((p = strstr(p, old_str)) != NULL) {
        count++;
        p += old_len;
    }
    if (count == 0) return;

    /* Build result */
    int orig_len = (int)strlen(*target);
    int result_len = orig_len + count * (new_len - old_len);
    char* result = (char*)malloc(result_len + 1);
    char* dst = result;
    const char* src = *target;

    while (*src) {
        if (strncmp(src, old_str, old_len) == 0) {
            memcpy(dst, new_str, new_len);
            dst += new_len;
            src += old_len;
        } else {
            *dst++ = *src++;
        }
    }
    *dst = '\0';

    *target = bstr_from_buf(result);
}

/* REPLACE ANY "oldset" WITH "newset" IN main$ (CHW REPLACE_statement.htm).
 * For each character in main that appears in oldset, replace it with the
 * character at the same position in newset. oldset and newset must be the
 * same length (one-to-one correspondence). */
void pb_replace_any(char** target, const char* old_set, const char* new_set) {
    if (target == NULL || *target == NULL || old_set == NULL || new_set == NULL) return;
    int old_len = (int)strlen(old_set);
    int new_len = (int)strlen(new_set);
    if (old_len == 0 || new_len == 0) return;
    int map_len = old_len < new_len ? old_len : new_len;
    size_t orig_len = strlen(*target);
    char* result = (char*)malloc(orig_len + 1);
    if (!result) return;
    for (size_t i = 0; i < orig_len; i++) {
        char c = (*target)[i];
        int slot = -1;
        for (int k = 0; k < map_len; k++) {
            if (old_set[k] == c) { slot = k; break; }
        }
        result[i] = (slot >= 0) ? new_set[slot] : c;
    }
    result[orig_len] = 0;
    *target = bstr_from_buf(result);
}

/* ============================================================
 * USING$ - Format one or more string/numeric expressions with a
 * PRINT USING-style mask. Backing arrays are aligned by original
 * argument index; output is hard capped at 1024 bytes (CHW).
 * ============================================================ */
typedef struct {
    char* data;
    size_t len;
    size_t cap;
} using_buf;

static void ub_init(using_buf* b) {
    b->cap = 256;
    b->len = 0;
    b->data = (char*)malloc(b->cap);
}
static void ub_ensure(using_buf* b, size_t extra) {
    if (b->len + extra + 1 > b->cap) {
        while (b->len + extra + 1 > b->cap) b->cap *= 2;
        b->data = (char*)realloc(b->data, b->cap);
    }
}
static void ub_put(using_buf* b, const char* p, size_t n) {
    if (n == 0) return;
    ub_ensure(b, n);
    memcpy(b->data + b->len, p, n);
    b->len += n;
}

static int pb_using_is_num_char(char c) {
    return c == '#' || c == '.' || c == ',' || c == '$' ||
           c == '*' || c == '+' || c == '-' || c == '^';
}

char* pb_using_multi(const char* fmt, const int* kinds, const double* nums,
                     const char* const* strs, int n) {
    using_buf b;
    ub_init(&b);
    int ai = 0;                 /* next argument index (mask order) */
    const char* p = fmt;
    while (*p) {
        if (*p == '_' && p[1]) {
            ub_put(&b, p + 1, 1);
            p += 2;
        } else if (*p == '!') {
            if (ai < n && kinds[ai] == 1 && strs[ai]) {
                int sl = pb_str_len(strs[ai]);
                if (sl > 0) ub_put(&b, strs[ai], 1);
            }
            ai++;
            p++;
        } else if (*p == '&') {
            if (ai < n && kinds[ai] == 1 && strs[ai]) {
                int sl = pb_str_len(strs[ai]);
                if (sl > 0) ub_put(&b, strs[ai], (size_t)sl);
            }
            ai++;
            p++;
        } else if (*p == '\\') {
            const char* q = p + 1;
            int width;
            if (*q == '\\') {
                width = 2;
                q++;
            } else {
                int spaces = 0;
                while (*q == ' ') { spaces++; q++; }
                if (*q == '\\') q++;
                width = spaces + 2;
            }
            if (ai < n && kinds[ai] == 1 && strs[ai]) {
                int sl = pb_str_len(strs[ai]);
                int take = sl < width ? sl : width;
                if (take > 0) ub_put(&b, strs[ai], (size_t)take);
            }
            ai++;
            p = q;
        } else if (pb_using_is_num_char(*p)) {
            /* A numeric field may contain '*' followed by ANY fill character
             * (including letters, which are not in the num-char set), so the
             * segment scan consumes the fill char explicitly after each '*'. */
            const char* q = p;
            while (*q) {
                if (*q == '*') {
                    q++;
                    if (*q) q++;
                    continue;
                }
                if (pb_using_is_num_char(*q)) { q++; continue; }
                break;
            }
            size_t mlen = (size_t)(q - p);
            char* msk = (char*)malloc(mlen + 1);
            memcpy(msk, p, mlen);
            msk[mlen] = '\0';
            if (ai < n && kinds[ai] == 0) {
                char* piece = pb_format_using(nums[ai], msk);
                if (piece) ub_put(&b, piece, (size_t)pb_str_len(piece));
            }
            free(msk);
            ai++;
            p = q;
        } else {
            ub_put(&b, p, 1);
            p++;
        }
    }
    size_t out_len = b.len < 1024 ? b.len : 1024;
    char* result = pb_bstr_alloc(b.data, (unsigned int)out_len);
    free(b.data);
    return result;
}

/* ============================================================
 * REMOVE$ � Remove all occurrences of characters in chars from str
 * ============================================================ */
char* pb_remove(const char* str, const char* chars) {
    if (str == NULL) {
        char* empty = (char*)malloc(1);
        empty[0] = '\0';
        return bstr_from_buf(empty);
    }
    if (chars == NULL || chars[0] == '\0') {
        char* result = (char*)malloc(strlen(str) + 1);
        strcpy(result, str);
        return bstr_from_buf(result);
    }

    int len = (int)strlen(str);
    char* result = (char*)malloc(len + 1);
    char* dst = result;

    for (const char* p = str; *p; p++) {
        if (strchr(chars, *p) == NULL) {
            *dst++ = *p;
        }
    }
    *dst = '\0';
    return bstr_from_buf(result);
}

/* ============================================================
 * File I/O Runtime Functions
 * ============================================================ */

#define MAX_FILE_HANDLES 256
static FILE* file_handles[MAX_FILE_HANDLES] = {0};
static int file_modes[MAX_FILE_HANDLES] = {0};
static char file_names[MAX_FILE_HANDLES][260]; /* 0=INPUT 1=OUTPUT 2=APPEND 3=BINARY 4=RANDOM */
/* RANDOM-mode record state: record length, record buffer, current record (0-based) */
static unsigned rec_len[MAX_FILE_HANDLES] = {0};
static char* rec_buf[MAX_FILE_HANDLES] = {0};
static long long rec_pos[MAX_FILE_HANDLES] = {0};

/* Forward declaration: socket state array is defined in the Winsock section
   (pb_sock_state). pb_freefile must account for occupied sockets. */
static int pb_sock_state[256];

int pb_freefile(void) {
    for (int i = 1; i < MAX_FILE_HANDLES; i++) {
        // A number is free only when neither a regular file nor a socket
        // (TCP/UDP) occupies it. pb_sock_state is defined later in this file,
        // so a forward declaration is provided below.
        if (file_handles[i] == NULL && pb_sock_state[i] == 0) return i;
    }
    return 0;
}

/* Batch 92: FRE() � free memory in bytes (uses GlobalMemoryStatusEx) */
long long pb_fre(void) {
    MEMORYSTATUSEX msx;
    memset(&msx, 0, sizeof(msx));
    msx.dwLength = sizeof(MEMORYSTATUSEX);
    if (GlobalMemoryStatusEx(&msx)) {
        return (long long)msx.ullAvailPhys;
    }
    return 0;
}

int pb_open(const char* path, int mode, int filenum) {
    if (filenum < 1 || filenum >= MAX_FILE_HANDLES) return -1;
    const char* fmode;
    switch (mode) {
        case 0: fmode = "rb"; break;  /* INPUT (binary: PBWin10 writes CRLF raw; a
                                         text-mode fopen would translate \r\n and
                                         corrupt LOF/byte counts - measured) */
        case 1: fmode = "wb"; break;  /* OUTPUT */
        case 2: fmode = "ab"; break;  /* APPEND */
        case 3:
            /* BINARY: open existing file read/write without truncating;
               create it if it does not exist (PB semantics). */
            file_handles[filenum] = fopen(path, "r+b");
            if (!file_handles[filenum]) {
                file_handles[filenum] = fopen(path, "w+b");
            }
            file_modes[filenum] = mode;
            if (file_handles[filenum]) {
                strncpy(file_names[filenum], path, 259);
                file_names[filenum][259] = 0;
            }
            return (file_handles[filenum] != NULL) ? 0 : -1;
        default: fmode = "r"; break;
    }
    file_handles[filenum] = fopen(path, fmode);
    if (file_handles[filenum]) {
        file_modes[filenum] = mode;
        strncpy(file_names[filenum], path, 259);
        file_names[filenum][259] = 0;
    }
    return (file_handles[filenum] != NULL) ? 0 : -1;
}

void pb_close(int filenum) {
    if (filenum >= 1 && filenum < MAX_FILE_HANDLES && file_handles[filenum]) {
        fclose(file_handles[filenum]);
        file_handles[filenum] = NULL;
        file_modes[filenum] = 0;
        file_names[filenum][0] = 0;
        if (rec_buf[filenum]) {
            free(rec_buf[filenum]);
            rec_buf[filenum] = NULL;
        }
        rec_len[filenum] = 0;
        rec_pos[filenum] = 0;
    }
}

/* FILEATTR([#] fnum&, fattr) � PB attribute query (see official FILEATTR function page). */
long long pb_fileattr(int filenum, int attr) {
    if (filenum < 1 || filenum >= MAX_FILE_HANDLES) return 0;
    FILE* f = file_handles[filenum];
    switch (attr) {
        case -3: return f ? 1 : 0;                 /* 1 = file (COMM/TCP/UDP channels are devices; not in this table) */
        case -2: return 1;                         /* logical first-byte position (BASE= not implemented -> 1) */
        case -1:
            if (!f) return 0;
            if (file_modes[filenum] == 4) return rec_len[filenum];   /* RANDOM: record length */
            if (file_modes[filenum] == 0) return 128;                /* INPUT: LEN= buffer default */
            return 1;                                               /* BINARY/OUTPUT/APPEND: unbuffered */
        case 0: return f ? -1 : 0;                 /* open state (non-zero = open) */
        case 1:                                    /* file mode bits (PB codes) */
            if (!f) return 0;
            switch (file_modes[filenum]) {
                case 0: return 1;    /* Input */
                case 1: return 2;    /* Output */
                case 4: return 4;    /* Random */
                case 2: return 10;   /* Append(8) + Output(2) */
                case 3: return 32;   /* Binary */
            }
            return 0;
        case 2: return f ? (long long)_fileno(f) : 0;   /* OS file handle */
        case 3: {                                    /* enumerate: nth open file number */
            int cnt = 0;
            for (int i = 1; i < MAX_FILE_HANDLES; i++) {
                if (file_handles[i]) {
                    cnt++;
                    if (cnt == filenum) return i;
                }
            }
            return -1;
        }
    }
    return 0;
}

/* FILENAME$([#] fnum&) � file-system name of an open file. */
char* pb_filename(int filenum) {
    if (filenum < 1 || filenum >= MAX_FILE_HANDLES || !file_handles[filenum]) {
        return pb_bstr_alloc("", 0);
    }
    return pb_bstr_alloc(file_names[filenum], (int)strlen(file_names[filenum]));
}

/* FILESCAN [#] fnum&, RECORDS TO y& [, WIDTH TO x&]
   INPUT mode: count CRLF/EOF-terminated records; WIDTH = longest record.
   BINARY mode: PB packed strings (2-byte length prefix, 0xFFFF marker + 4-byte
   length for strings > 65535 bytes); WIDTH = longest string length. */
void pb_filescan(int filenum, long long* records, long long* width) {
    FILE* f = (filenum >= 1 && filenum < MAX_FILE_HANDLES) ? file_handles[filenum] : NULL;
    if (!f) { *records = 0; *width = 0; return; }
    long pos = ftell(f);
    rewind(f);
    long long rec = 0, w = 0;
    if (file_modes[filenum] == 3) {
        /* BINARY: PB packed strings */
        for (;;) {
            unsigned char h[2];
            size_t n = fread(h, 1, 2, f);
            if (n < 2) break;
            unsigned int len = h[0] | (h[1] << 8);
            if (len == 0xFFFF) {
                unsigned char l4[4];
                if (fread(l4, 1, 4, f) < 4) break;
                len = l4[0] | (l4[1] << 8) | (l4[2] << 16) | ((unsigned int)l4[3] << 24);
            }
            if (fseek(f, (long)len, SEEK_CUR) != 0) break;
            rec++;
            if ((long long)len > w) w = len;
        }
    } else {
        /* INPUT: CRLF-delimited records */
        int ch;
        long long cur = 0;
        int in_rec = 0;
        while ((ch = fgetc(f)) != EOF) {
            if (ch == '\r' || ch == '\n') {
                if (in_rec) { rec++; if (cur > w) w = cur; cur = 0; in_rec = 0; }
            } else {
                in_rec = 1;
                cur++;
            }
        }
        if (in_rec) { rec++; if (cur > w) w = cur; }
    }
    fseek(f, pos, SEEK_SET);
    *records = rec;
    *width = w;
}

void pb_print_file(int filenum, const char* text) {
    if (filenum >= 1 && filenum < MAX_FILE_HANDLES && file_handles[filenum]) {
        /* batch 262: PRINT # must write the FULL string bytes, not strlen --
           embedded NULs (e.g. CLIPBOARD GET UNICODE UTF-16LE) would be
           truncated by fputs.  text is a BSTR data pointer (SysStringByteLen
           reads the real byte count). */
        unsigned long n = SysStringByteLen(text);
        fwrite(text, 1, n, file_handles[filenum]);
    }
}

void pb_print_file_item(int filenum, const char* text) {
    /* PRINT # item writer (batch284): PBWin appends one space AFTER every
       item (strings and numbers alike), including the last one before CR/LF.
       The number itself carries no leading space (pb_dec_file), so
       `PRINT #1, "x="; r` yields "x= -1 " exactly as PBWin. */
    if (filenum >= 1 && filenum < MAX_FILE_HANDLES && file_handles[filenum]) {
        unsigned long n = SysStringByteLen(text);
        fwrite(text, 1, n, file_handles[filenum]);
        fputc(' ', file_handles[filenum]);
    }
}

void pb_print_file_newline(int filenum) {
    if (filenum >= 1 && filenum < MAX_FILE_HANDLES && file_handles[filenum]) {
        /* batch 270: PBWin writes CRLF (0d 0a) to PRINT # files; fork wrote
           bare LF because the file handle is opened in binary mode ("wb",
           "ab", "r+b") where fputc('\n') stays LF.  Write both bytes. */
        fputc('\r', file_handles[filenum]);
        fputc('\n', file_handles[filenum]);
    }
}

char* pb_line_input(int filenum) {
    if (filenum < 1 || filenum >= MAX_FILE_HANDLES || !file_handles[filenum]) {
        char* empty = (char*)malloc(1);
        empty[0] = '\0';
        return bstr_from_buf(empty);
    }
    char buf[4096];
    if (fgets(buf, sizeof(buf), file_handles[filenum]) == NULL) {
        char* empty = (char*)malloc(1);
        empty[0] = '\0';
        return bstr_from_buf(empty);
    }
    /* Strip trailing newline */
    int len = (int)strlen(buf);
    if (len > 0 && buf[len-1] == '\n') {
        buf[len-1] = '\0';
        len--;
    }
    if (len > 0 && buf[len-1] == '\r') {
        buf[len-1] = '\0';
        len--;
    }
    char* result = (char*)malloc(len + 1);
    memcpy(result, buf, len);
    result[len] = '\0';
    return bstr_from_buf(result);
}

/* COMM LINE timeout EOF query (batch 260); defined in the COMM section. */
int pb_comm_is_eof(int channel);
/* TCP/UDP socket EOF probe; defined in the socket section. */
int pb_socket_eof_state(int filenum);

int pb_eof(int filenum) {
    /* Batch 260: EOF(channel) on an open COMM port reflects the COMM LINE
       timeout state (CHW COMM_LINE_statement.htm: on timeout the accumulated
       partial line is returned and EOF becomes TRUE).  Non-COMM file numbers
       fall through to the file check below. */
    if (pb_comm_is_eof(filenum)) return -1;
    int se = pb_socket_eof_state(filenum);
    if (se != -2) return se;
    if (filenum < 1 || filenum >= MAX_FILE_HANDLES || !file_handles[filenum]) return -1;
    /* PB EOF() returns true when no more data to read � need to peek ahead */
    int ch = fgetc(file_handles[filenum]);
    if (ch == EOF) return -1;  /* PB: -1 = true (EOF) */
    ungetc(ch, file_handles[filenum]);
    return 0;  /* PB: 0 = false (not EOF) */
}

void pb_input_file_str(int filenum, char** dest) {
    if (filenum < 1 || filenum >= MAX_FILE_HANDLES || !file_handles[filenum]) return;
    char buf[4096];
    int i = 0;
    int ch;
    while ((ch = fgetc(file_handles[filenum])) != EOF) {
        if (ch == ',' || ch == '\n') break;
        if (ch == '\r') continue;
        /* batch 308 (I): a string item ignores leading spaces (PBWin INPUT#
           semantics).  Only the first char is subject to the skip; spaces
           after the first char are content ("hello world" stays intact). */
        if (i == 0 && ch == ' ') continue;
        if (i < (int)sizeof(buf) - 1) buf[i++] = (char)ch;
    }
    buf[i] = '\0';
    /* Strip surrounding double quotes written by WRITE# (CSV quoting) */
    if (i >= 2 && buf[0] == '"' && buf[i - 1] == '"') {
        char* unquoted = (char*)malloc(i - 1);
        memcpy(unquoted, buf + 1, i - 2);
        unquoted[i - 2] = '\0';
        *dest = bstr_from_buf(unquoted);
        return;
    }
    char* result = (char*)malloc(i + 1);
    memcpy(result, buf, i + 1);
    *dest = bstr_from_buf(result);
}

/* FixedString target for INPUT# / INPUT (file): read one comma/newline
   delimited token, copy it into an inline [N x i8] buffer truncated to
   N-1 chars with a trailing NUL (PBWin semantics), honouring WRITE#
   double-quote CSV quoting.  This is the fixed-buffer sibling of
   pb_input_file_str: the old path wrote a BSTR pointer into the inline
   buffer, corrupting the struct (legacy #7). */
/* FixedString inline [N x i8] buffer -> fresh COM BSTR (legacy #7 read
   side).  Runtime functions such as pb_str_concat use SysStringByteLen on
   Windows, which reads the 4-byte length header of a COM BSTR; an inline
   buffer has no header, so passing it straight through reads garbage as the
   length.  Materialise a real BSTR once at the read site instead. */
char* pb_fixed_to_bstr(const char* buf) {
    if (!buf) return pb_bstr_alloc("", 0);
    return pb_bstr_alloc(buf, (unsigned int)strlen(buf));
}

void pb_input_file_fixed(int filenum, char* buf, long long n) {
    if (filenum < 1 || filenum >= MAX_FILE_HANDLES || !file_handles[filenum]) return;
    if (!buf || n <= 0) return;
    char tmp[4096];
    int i = 0, ch;
    while ((ch = fgetc(file_handles[filenum])) != EOF) {
        if (ch == ',' || ch == '\n') break;
        if (ch == '\r') continue;
        if (i < (int)sizeof(tmp) - 1) tmp[i++] = (char)ch;
    }
    tmp[i] = '\0';
    if (i >= 2 && tmp[0] == '"' && tmp[i - 1] == '"') {
        char* unquoted = (char*)malloc(i - 1);
        memcpy(unquoted, tmp + 1, i - 2);
        unquoted[i - 2] = '\0';
        strncpy(buf, unquoted, (size_t)n - 1);
        free(unquoted);
    } else {
        strncpy(buf, tmp, (size_t)n - 1);
    }
    buf[n - 1] = '\0';
}

/* FixedString target for console INPUT / LINE INPUT / LINE INPUT#:
   copy a freshly read BSTR into an inline [N x i8] buffer truncated to
   N-1 chars with a trailing NUL (legacy #7).  The BSTR is NOT released
   here: pb_line_input allocates with bstr_from_buf (malloc, not COM) so
   pb_bstr_free (SysFreeString) would crash; this matches the existing
   fork INPUT semantics, which never free the freshly read BSTR. */
void pb_copy_bstr_to_fixed(char* buf, const char* src, long long n) {
    if (!buf || n <= 0) return;
    if (src) strncpy(buf, src, (size_t)n - 1);
    buf[n - 1] = '\0';
}

void pb_input_file_int(int filenum, int* dest) {
    if (filenum < 1 || filenum >= MAX_FILE_HANDLES || !file_handles[filenum]) return;
    char buf[64];
    int i = 0;
    int ch;
    while ((ch = fgetc(file_handles[filenum])) != EOF) {
        /* batch 308 (I): a numeric item also terminates on a space, not just
           ',' or '\n' - "42 hello" reads 42 and leaves "hello" for the next
           item (PBWin INPUT# semantics).  atoi/atof skip leading blanks, so
           the extra space before the next item is harmless. */
        if (ch == ',' || ch == '\n' || ch == ' ') break;
        if (ch == '\r') continue;
        if (i < (int)sizeof(buf) - 1) buf[i++] = (char)ch;
    }
    buf[i] = '\0';
    *dest = atoi(buf);
}

void pb_input_file_dbl(int filenum, double* dest) {
    if (filenum < 1 || filenum >= MAX_FILE_HANDLES || !file_handles[filenum]) return;
    char buf[64];
    int i = 0;
    int ch;
    while ((ch = fgetc(file_handles[filenum])) != EOF) {
        if (ch == ',' || ch == '\n' || ch == ' ') break;
        if (ch == '\r') continue;
        if (i < (int)sizeof(buf) - 1) buf[i++] = (char)ch;
    }
    buf[i] = '\0';
    *dest = atof(buf);
}

/* pb_err / pb_err_var are defined later in this file (global ERR variables). */
extern int pb_err;
extern int pb_err_var;   /* ERR system variable - keeps the LAST error code */

/* KILL filespec - delete file(s). Supports paths and wildcard characters.
   CHW KILL semantics (the only standard):
     - no matching file          -> ERR 53 (File not found)
     - read-only / access denied -> ERR 70 (Permission denied)
     - HIDDEN / SYSTEM files     -> silently ignored, no error
     - directories               -> never deleted (use RMDIR)
   Returns 0 on success, -1 on failure; the ERR code is set here directly, so
   the codegen only needs a void call. WIN32_FIND_DATAA is 592 bytes; the
   directory attribute is 0x10, HIDDEN 0x2, SYSTEM 0x4 (hand-rolled, no
   windows.h). */
int pb_kill(const char* path) {
    if (!path || !*path) { pb_err_var = pb_err = 53; return -1; }

    /* split directory part from the file pattern, to rebuild full paths */
    const char* last_sep = NULL;
    for (const char* q = path; *q; q++) {
        if (*q == '\\' || *q == '/') last_sep = q;
    }
    char dirbuf[512];
    if (last_sep) {
        size_t dlen = (size_t)(last_sep - path) + 1;
        if (dlen >= sizeof(dirbuf)) dlen = sizeof(dirbuf) - 1;
        memcpy(dirbuf, path, dlen);
        dirbuf[dlen] = '\0';
    } else {
        dirbuf[0] = '\0';
    }

    char find_data[592];
    void* h = FindFirstFileA(path, find_data);
    if (h == PB_INVALID_HANDLE) {
        /* FindFirstFile: ERROR_FILE_NOT_FOUND (2) / ERROR_PATH_NOT_FOUND (3) */
        pb_err_var = pb_err = 53;
        return -1;
    }

    int matched = 0;       /* entries enumerated, excluding "." and ".." */
    int fail_code = 0;     /* first hard failure (0 = none yet) */
    do {
        const char* name = find_data + 44;   /* cFileName offset */
        if ((name[0] == '.' && name[1] == '\0') ||
            (name[0] == '.' && name[1] == '.' && name[2] == '\0')) {
            continue;
        }
        unsigned long fa;
        memcpy(&fa, find_data, sizeof(fa));
        matched++;

        /* KILL never deletes directories. */
        if (fa & 0x10UL) continue;

        /* HIDDEN / SYSTEM: the attempt is ignored with no error. */
        if (fa & (0x2UL | 0x4UL)) continue;

        char full[768];
        int n = snprintf(full, sizeof(full), "%s%s", dirbuf, name);
        if (n < 0 || (size_t)n >= sizeof(full)) {
            if (!fail_code) fail_code = 76;
            continue;
        }

        if (DeleteFileA(full)) continue;

        unsigned long e = GetLastError();
        int code;
        if (e == 5)                code = 70;  /* ERROR_ACCESS_DENIED */
        else if (e == 2 || e == 3) code = 53;
        else                       code = 75;
        if (!fail_code) fail_code = code;
    } while (FindNextFileA(h, find_data));
    FindClose(h);

    if (matched == 0) {
        pb_err_var = pb_err = 53;
        return -1;
    }
    if (fail_code) {
        pb_err_var = pb_err = fail_code;
        return -1;
    }
    pb_err = 0;
    return 0;
}

/* CLS: clear the console screen (PB/CC) */
void pb_cls(void) {
    system("cls");
}

/* ENVIRON "VAR=value" (statement form): set / remove an environment variable.
   With '=' ? set. Without '=' ? remove the variable. */
void pb_environ_set(const char* s) {
    if (!s) return;
    if (strchr(s, '=')) {
        _putenv(s);
    } else {
        size_t n = strlen(s);
        char* buf = (char*)malloc(n + 2);
        if (!buf) return;
        memcpy(buf, s, n);
        buf[n] = '=';
        buf[n + 1] = '\0';
        _putenv(buf);
        free(buf);
    }
}

/* FILECOPY src$, dst$ � copy a file. Sets ERR on failure (PB semantics). */
int pb_filecopy(const char* src, const char* dst) {
    if (!src || !dst) { pb_err_var = pb_err = 76; return -1; }
    if (!CopyFileA(src, dst, 0)) {
        unsigned long e = GetLastError();
        if (e == 2)       pb_err_var = pb_err = 53;  /* ERROR_FILE_NOT_FOUND */
        else if (e == 5)  pb_err_var = pb_err = 70;  /* ERROR_ACCESS_DENIED */
        else              pb_err_var = pb_err = 76;  /* path / other */
        return -1;
    }
    pb_err = 0;
    return 0;
}

/* SETATTR "path", attr& � set file attributes. Sets ERR on failure. */
void pb_setattr(const char* path, int attr) {
    if (!path) { pb_err_var = pb_err = 76; return; }
    if (!SetFileAttributesA(path, (unsigned long)attr)) {
        unsigned long e = GetLastError();
        pb_err_var = pb_err = (e == 2) ? 53 : 76;
    } else {
        pb_err = 0;
    }
}

/* ===== ERR system variable (PB-compatible error code) ===== */
int pb_err = 0;   /* trap flag: per-statement; set by failing runtime calls,
                     cleared by emit_stmt_enter before each checked statement */
int pb_err_var = 0; /* ERR system variable: keeps the LAST error code after
                       RESUME and after successful statements (PBWin semantics
                       measured: BINARY PUT after an ERR 54 trap reads ERR=54) */

/* ===== ON ERROR GOTO runtime state ===== */
int pb_err_stmt_id = 0;  /* id of the statement that triggered the error */
int pb_err_active = 0;   /* 1 while executing the ON ERROR handler (trapping suspended) */

/* ===== ERROR$ � PB error message lookup ===== */
static const char* pb_error_msgs[] = {
    [0] = "No Error",
    [5] = "Illegal function call",
    [6] = "Overflow",
    [7] = "Out of memory",
    [9] = "Subscript/Pointer out of range",
    [11] = "Division by zero",
    [24] = "Device timeout",
    [51] = "Internal error",
    [52] = "Bad file name or number",
    [53] = "File not found",
    [54] = "Bad file mode",
    [55] = "File is open",
    [57] = "Device I/O error",
    [58] = "File already exists",
    [61] = "Disk full",
    [62] = "Input past end",
    [63] = "Bad record number",
    [64] = "Bad file name",
    [67] = "Too many files",
    [70] = "Permission denied",
    [76] = "Path not found",
    [99] = "Object error",
    [999] = "No Error",
};
#define PB_ERROR_MSG_COUNT (sizeof(pb_error_msgs)/sizeof(pb_error_msgs[0]))

/* Returns BSTR with error message for code n (or current pb_err if n<0).
   Caller must SysFreeString the result. */
char* pb_error_message(int n) {
    int code = (n < 0) ? pb_err : n;
    const char* msg = NULL;
    if (code >= 0 && code < (int)PB_ERROR_MSG_COUNT) {
        msg = pb_error_msgs[code];
    }
    if (!msg) {
        /* PBWin10 returns an empty string for error codes that have no
           table entry (measured 2026-10-13: 1-4, 8, 10, 12-23, 25-50, 56,
           59-60, 65-66, 68-98, 100-500+ all empty). */
        static const char empty[] = "";
        msg = empty;
    }

    return pb_bstr_alloc(msg, (unsigned int)strlen(msg));
}

/* Raise a runtime error: set pb_err, print the message, and exit.
   Called from compiler-inserted checks (division by zero, subscript out of
   range, overflow).  noreturn so LLVM knows the following sdiv/srem/load
   is only reached on the safe path. */
__declspec(noreturn) void pb_raise_error(int code) {
    pb_err = code;
    pb_err_var = code;
    const char* msg = NULL;
    if (code >= 0 && code < (int)PB_ERROR_MSG_COUNT) {
        msg = pb_error_msgs[code];
    }
    if (!msg) msg = "Runtime error";
    fprintf(stderr, "Error %d: %s\n", code, msg);
    fflush(stderr);
    exit(code);
}

/* ===== System builtins ===== */

#ifdef _WIN32
__declspec(dllimport) unsigned long __stdcall GetEnvironmentVariableA(const char* lpName, char* lpBuffer, unsigned long nSize);
__declspec(dllimport) unsigned long __stdcall GetModuleFileNameA(void* hModule, char* lpFilename, unsigned long nSize);
#endif

/* ENVIRON$("VARNAME") � returns environment variable value */
char* pb_environ(const char* var_name) {
    if (!var_name || !var_name[0]) {
        char* empty = (char*)malloc(1);
        empty[0] = '\0';
        return bstr_from_buf(empty);
    }
#ifdef _WIN32
    char buf[4096];
    unsigned long len = GetEnvironmentVariableA(var_name, buf, sizeof(buf));
    if (len == 0 || len >= sizeof(buf)) {
        char* empty = (char*)malloc(1);
        empty[0] = '\0';
        return bstr_from_buf(empty);
    }
    char* result = (char*)malloc(len + 1);
    memcpy(result, buf, len);
    result[len] = '\0';
    return bstr_from_buf(result);
#else
    const char* val = getenv(var_name);
    if (!val) val = "";
    int len = (int)strlen(val);
    char* result = (char*)malloc(len + 1);
    memcpy(result, val, len);
    result[len] = '\0';
    return bstr_from_buf(result);
#endif
}

/* EXE.PATH$ � returns directory of current executable */
char* pb_exe_path(void) {
#ifdef _WIN32
    char buf[4096];
    unsigned long len = GetModuleFileNameA(0, buf, sizeof(buf));
    if (len == 0) {
        char* empty = (char*)malloc(1);
        empty[0] = '\0';
        return bstr_from_buf(empty);
    }
    /* Strip filename, keep directory with trailing backslash */
    int i = (int)len - 1;
    while (i >= 0 && buf[i] != '\\' && buf[i] != '/') i--;
    if (i >= 0) buf[i + 1] = '\0';
    int plen = (int)strlen(buf);
    char* result = (char*)malloc(plen + 1);
    memcpy(result, buf, plen);
    result[plen] = '\0';
    return bstr_from_buf(result);
#else
    char* empty = (char*)malloc(1);
    empty[0] = '\0';
    return bstr_from_buf(empty);
#endif
}

/* EXE.NAME$ � returns filename of current executable */
char* pb_exe_name(void) {
#ifdef _WIN32
    char buf[4096];
    unsigned long len = GetModuleFileNameA(0, buf, sizeof(buf));
    if (len == 0) {
        char* empty = (char*)malloc(1);
        empty[0] = '\0';
        return bstr_from_buf(empty);
    }
    /* Find last path separator */
    int i = (int)len - 1;
    while (i >= 0 && buf[i] != '\\' && buf[i] != '/') i--;
    const char* name = buf + i + 1;
    int nlen = (int)strlen(name);
    char* result = (char*)malloc(nlen + 1);
    memcpy(result, name, nlen);
    result[nlen] = '\0';
    return bstr_from_buf(result);
#else
    char* empty = (char*)malloc(1);
    empty[0] = '\0';
    return bstr_from_buf(empty);
#endif
}

/* ===== Null-safe string concatenation ===== */

/* pb_str_concat � concatenates two strings, treating null as empty.
   Strings are real BSTRs (SysAllocStringByteLen) on Windows, so strlen would
   truncate embedded NULs (CHR$(0)): use SysStringByteLen for the true length. */
char* pb_str_concat(const char* a, const char* b) {
    static const char empty[] = "";
    if (!a) a = empty;
    if (!b) b = empty;
#ifdef _WIN32
    unsigned int len_a = (unsigned int)SysStringByteLen(a);
    unsigned int len_b = (unsigned int)SysStringByteLen(b);
#else
    unsigned int len_a = (unsigned int)strlen(a);
    unsigned int len_b = (unsigned int)strlen(b);
#endif
    int total = (int)(len_a + len_b);
    char* tmp = (char*)malloc(total + 1);
    memcpy(tmp, a, len_a);
    memcpy(tmp + len_a, b, len_b);
    tmp[total] = '\0';
    char* result = pb_bstr_alloc(tmp, total);
    free(tmp);
    return result;
}

/* ===== DATE$ and TIME$ builtins ===== */

#ifdef _WIN32
typedef struct {
    unsigned short wYear;
    unsigned short wMonth;
    unsigned short wDay;
    unsigned short wDayOfWeek;
    unsigned short wHour;
    unsigned short wMinute;
    unsigned short wSecond;
    unsigned short wMilliseconds;
} PB_SYSTEMTIME;

__declspec(dllimport) int __stdcall SetLocalTime(PB_SYSTEMTIME* lpSystemTime);
__declspec(dllimport) void __stdcall GetLocalTime(PB_SYSTEMTIME* lpSystemTime);
#endif

/* DATE$ � returns "MM-DD-YYYY" */
char* pb_date(void) {
    char buf[12];
#ifdef _WIN32
    PB_SYSTEMTIME st;
    GetLocalTime(&st);
    sprintf(buf, "%02d-%02d-%04d", st.wMonth, st.wDay, st.wYear);
#else
    strcpy(buf, "01-01-2026");
#endif
    return pb_bstr_alloc(buf, (int)strlen(buf));
}

/* TIME$ � returns "HH:MM:SS" */
char* pb_time(void) {
    char buf[10];
#ifdef _WIN32
    PB_SYSTEMTIME st;
    GetLocalTime(&st);
    sprintf(buf, "%02d:%02d:%02d", st.wHour, st.wMinute, st.wSecond);
#else
    strcpy(buf, "00:00:00");
#endif
    return pb_bstr_alloc(buf, (int)strlen(buf));
}

/* TIMER (CHW TIMER_function.htm): seconds since midnight as a Single,
 * range 0..86400. The fork previously returned time(NULL) (Unix epoch
 * seconds), which broke bare RANDOMIZE seeding and CHW parity. */
double pb_timer(void) {
#ifdef _WIN32
    PB_SYSTEMTIME st;
    GetLocalTime(&st);
    return (double)st.wHour * 3600.0 + (double)st.wMinute * 60.0 +
           (double)st.wSecond + (double)st.wMilliseconds / 1000.0;
#else
    return 0.0;
#endif
}

/* Per-thread PB-style PRNG. CHW RANDOMIZE: "each thread has its own,
 * independent random number seed." The fork previously used the process-wide
 * libc srand/rand. The state below is thread-local; RANDOMIZE n makes the
 * stream reproducible, and RND reads this same state (pb_rnd_value is the
 * shared entry point the R2 shard builds on). */
static __declspec(thread) unsigned long long pb_prng_state = 0;
static __declspec(thread) int pb_prng_seeded = 0;

void pb_randomize(unsigned int seed) {
    /* SplitMix64-style mixer so any seed (including 0 / 0xFFFFFFFF) lands in
     * a good xorshift state; guarantees RANDOMIZE n -> reproducible stream. */
    unsigned long long x = (unsigned long long)seed + 0x9E3779B97F4A7C15ULL;
    x = (x ^ (x >> 30)) * 0xBF58476D1CE4E5B9ULL;
    x = (x ^ (x >> 27)) * 0x94D049BB133111EBULL;
    pb_prng_state = x ^ (x >> 31);
    if (pb_prng_state == 0) pb_prng_state = 1;
    pb_prng_seeded = 1;
}

/* Bare RANDOMIZE: seed from TIMER (CHW: "If number is not specified, the
 * value returned by the TIMER function is used"). The float->integer mapping
 * is fork-internal (CHW does not specify it). */
void pb_randomize_timer(void) {
    pb_randomize((unsigned int)(pb_timer() * 1000.0));
}

/* R2 RND: cached last value so RND(0) can repeat it without advancing.
 * CHW RND_function.htm: "With an argument of 0, RND repeats the last number
 * generated." pb_rnd_value() stores every produced sample here; pb_rnd_last()
 * returns it without stepping the generator. */
static __declspec(thread) double pb_prng_last = 0.0;

double pb_rnd_value(void) {
    if (!pb_prng_seeded) {
        /* CHW default seed: RANDOMIZE CVS(CHR$(255,255,255,255)) */
        pb_randomize(0xFFFFFFFFu);
    }
    unsigned long long x = pb_prng_state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    pb_prng_state = x;
    double v = (double)(x >> 11) * (1.0 / 9007199254740992.0);
    pb_prng_last = v;
    return v;
}

/* RND(0): repeat the most recently generated value without advancing. */
double pb_rnd_last(void) {
    if (!pb_prng_seeded) {
        pb_randomize(0xFFFFFFFFu);
    }
    return pb_prng_last;
}

/* RND(n) with n<0: re-seed the per-thread PRNG from the signed argument
 * (CHW: "A negative argument causes the random number generator to be
 * re-seeded"). The caller then issues a normal next-value call to consume
 * the first sample of the new stream. */
void pb_rnd_reseed(int seed) {
    pb_randomize((unsigned int)seed);
}

/* R2 RND 1-arg CHW dispatch (CHW RND_function.htm):
 *   n == 0  -> repeat the last value (no advance)
 *   n <  0  -> reseed from n, then produce the next value
 *   n >  0  -> produce the next value (same stream as bare RND)
 * The old code treated RND(n) as "random int in [0,n)" which is not CHW. */
double pb_rnd_1arg(double n) {
    if (n == 0.0) return pb_rnd_last();
    if (n < 0.0) pb_rnd_reseed((int)n);
    return pb_rnd_value();
}

/* DATE$ = s$ - CHW: set the local system date.  Accepted formats:
   MM-DD-YY / MM-DD-YYYY / MM/DD/YY / MM/DD/YYYY (2-digit years mapped
   with the standard Windows rule: 00-69 -> 2000-2069, 70-99 -> 1970-1999).
   Invalid format or a failed SetLocalTime -> ERR = 5, clock untouched.
   CAUTION: side effect on the real system clock; never run behavior probes. */
void pb_set_date(const char* s) {
#ifdef _WIN32
    int m = 0, d = 0, y = 0, n = 0;
    char c1 = 0, c2 = 0;
    n = sscanf(s, "%d%c%d%c%d", &m, &c1, &d, &c2, &y);
    if (n == 5 && (c1 == '-' || c1 == '/') && (c2 == '-' || c2 == '/')) {
        if (y < 100) { if (y < 70) y += 2000; else y += 1900; }
        if (m >= 1 && m <= 12 && d >= 1 && d <= 31 && y >= 1601 && y <= 30827) {
            PB_SYSTEMTIME st;
            GetLocalTime(&st);
            st.wYear = (unsigned short)y;
            st.wMonth = (unsigned short)m;
            st.wDay = (unsigned short)d;
            if (SetLocalTime(&st)) return;
        }
    }
    pb_err_var = pb_err = 5;
#else
    pb_err_var = pb_err = 5;
#endif
}

/* TIME$ = s$ - CHW: set the local system time.  Accepted: HH:MM[:SS].
   Invalid format or a failed SetLocalTime -> ERR = 5. */
void pb_set_time(const char* s) {
#ifdef _WIN32
    int h = 0, m = 0, sec = 0, n = 0;
    n = sscanf(s, "%d:%d:%d", &h, &m, &sec);
    /* CHW: minutes and seconds may be omitted, so n=1 (hour only), 2 or 3.
     * Unparsed m/sec stay 0, e.g. TIME$="12" means 12:00:00. */
    if (n >= 1 && n <= 3) {
        if (h >= 0 && h <= 23 && m >= 0 && m <= 59 && sec >= 0 && sec <= 59) {
            PB_SYSTEMTIME st;
            GetLocalTime(&st);
            st.wHour = (unsigned short)h;
            st.wMinute = (unsigned short)m;
            st.wSecond = (unsigned short)sec;
            st.wMilliseconds = 0;
            if (SetLocalTime(&st)) return;
        }
    }
    pb_err_var = pb_err = 5;
#else
    pb_err_var = pb_err = 5;
#endif
}

/* WAITKEY$ — waits for one key press (console), returns the key as a 1-char string.
   Interactive console: _getch (immediate, no Enter needed).
   Redirected stdin (pipes / CI): getchar so automated tests can feed a key.
   GUI app that got a console via AllocConsole (double-clicked EXE): CRT stdin is
   not a tty after AllocConsole, so the Win32 console input handle is used to
   wait for a key - otherwise the program exits instantly and the console
   window disappears before the user can read the output. */
char* pb_waitkey(void) {
    int c;
    if (_isatty(_fileno(stdin))) {
        c = _getch();
        if (c == 0 || c == 0xE0) {
            /* extended key: read the scan code and return an empty string */
            _getch();
            return pb_bstr_alloc("", 0);
        }
    } else {
#ifdef _WIN32
        void* hIn = GetStdHandle((unsigned int)-10); /* STD_INPUT_HANDLE */
        if (hIn != NULL && hIn != PB_INVALID_HANDLE) {
            char ch = 0;
            unsigned long n = 0;
            if (ReadFile(hIn, &ch, 1, &n, NULL)) {
                if (n == 1) {
                    char buf[2];
                    buf[0] = ch;
                    buf[1] = '\0';
                    return pb_bstr_alloc(buf, 1);
                }
                return pb_bstr_alloc("", 0); /* EOF (DEVNULL) - return at once */
            }
        }
#endif
        c = getchar();
        if (c == EOF) c = ' ';
    }
    char buf[2];
    buf[0] = (char)c;
    buf[1] = '\0';
    return pb_bstr_alloc(buf, 1);
}

/* ===== TIX / MKBYT$ / ISINFINITE / ISNORMAL / CHDRIVE / SETEOF / PLAY WAVE ===== */

#ifdef _WIN32
typedef union {
    long long QuadPart;
    struct { unsigned long LowPart; long HighPart; } u;
} PB_LARGE_INTEGER;

__declspec(dllimport) int __stdcall QueryPerformanceCounter(PB_LARGE_INTEGER* lp);
__declspec(dllimport) int __stdcall SetEndOfFile(void* hFile);
__declspec(dllimport) int __stdcall PlaySoundA(const char* pszSound, void* hmod, unsigned long fdwSound);
#endif

#define PB_SND_FILENAME 0x00020000

/* TIX � high-resolution performance counter (QUAD) */
/* TIX - read the CPU time-stamp counter (RDTSC). CHW TIX_statement.htm:
 * "Measures elapsed CPU cycles", the smallest timing increment.  PBWin10 emits
 * the raw rdtsc opcode (verified by disassembly), so we match that instead of
 * QueryPerformanceCounter, whose coarse 100 ns ticks can read 0 for short code. */
static __inline long long pb_rdtsc_counter(void) {
    unsigned int lo, hi;
    /* "memory" clobber makes rdtsc a compiler barrier: the code being timed
     * must not be reordered or folded across the counter reads. */
    __asm__ __volatile__("rdtsc" : "=a"(lo), "=d"(hi) : : "memory");
    return ((long long)hi << 32) | (long long)lo;
}

long long pb_tix(void) {
    return pb_rdtsc_counter();
}

/* MKBYT$ (n) � one byte as a single-character string */
char* pb_mkbyt(int n) {
    char buf[2];
    buf[0] = (char)(n & 0xFF);
    buf[1] = '\0';
    return pb_bstr_alloc(buf, 1);
}

/* ISINFINITE (x) / ISNORMAL (x) � IEEE-754 classification, PB -1/0 */
int pb_isinfinite(double v) { return isinf(v) ? -1 : 0; }
int pb_isnormal(double v) { return isnormal(v) ? -1 : 0; }

/* CHDRIVE drv$ � change current drive (PB semantics: drive letter only) */
int pb_chdrive(const char* drv) {
#ifdef _WIN32
    if (drv && drv[0] != '\0') {
        return _chdrive(toupper((unsigned char)drv[0]) - 'A' + 1);
    }
#endif
    pb_err_var = pb_err = 76; return -1;
}

/* SETEOF #f � truncate file at current position */
int pb_seteof(int f) {
    if (f < 1 || f >= MAX_FILE_HANDLES || file_handles[f] == NULL) { pb_err_var = pb_err = 52; return -1; }
#ifdef _WIN32
    int fd = _fileno(file_handles[f]);
    void* h = (void*)_get_osfhandle(fd);
    if (h == (void*)-1) { pb_err_var = pb_err = 57; return -1; }
    return SetEndOfFile(h) ? 0 : -1;
#else
    long cur = ftell(file_handles[f]);
    return ftruncate(fileno(file_handles[f]), cur) == 0 ? 0 : -1;
#endif
}

/* PLAY WAVE "file.wav" � play a .wav synchronously */
int pb_play_wave(const char* path) {
#ifdef _WIN32
    return PlaySoundA(path, NULL, PB_SND_FILENAME) ? 0 : -1;
#else
    pb_err_var = pb_err = 68; return -1;
#endif
}

/* PLAY WAVE END - stop any currently playing waveform. A NULL pszSound tells
PlaySound to stop playback; it is safe (no error) when nothing is playing. */
void pb_play_wave_end(void) {
#ifdef _WIN32
    PlaySoundA(NULL, NULL, 0);
#endif
}

/* ===== Batch 2: ARRAY REVERSE / PUT$ / SHIFT / ROTATE ===== */

/* ARRAY REVERSE � reverse elements in place */
void pb_array_reverse(void* base, int elem_size, int total) {
    if (!base || total <= 1) return;
    char* p = (char*)base;
    for (int i = 0; i < total / 2; i++) {
        char* a = p + (size_t)i * elem_size;
        char* b = p + (size_t)(total - 1 - i) * elem_size;
        for (int j = 0; j < elem_size; j++) {
            char t = a[j];
            a[j] = b[j];
            b[j] = t;
        }
    }
}

/* ============ FIELD variables + RANDOM record buffer ============ */
/* PB field variable storage: 24 bytes =
   { char* data; unsigned offset; unsigned len; unsigned kind; }
   kind 0 = direct buffer (RANDOM record buffer / private copy)
   kind 1 = string variable slot (char**): resolve *data + offset at use time */
typedef struct { char* data; unsigned offset; unsigned len; unsigned kind; } pb_field_t;

/* OPEN ... FOR RANDOM AS #n LEN=reclen � open r+b (keep existing) else w+b, alloc record buffer */
int pb_open_random(const char* path, int filenum, unsigned reclen) {
    if (filenum < 1 || filenum >= MAX_FILE_HANDLES) { pb_err_var = pb_err = 52; return -1; }
    if (reclen == 0) reclen = 128;
    FILE* f = fopen(path, "r+b");
    if (!f) f = fopen(path, "w+b");
    if (!f) { pb_err_var = pb_err = 53; return -1; }
    file_handles[filenum] = f;
    file_modes[filenum] = 4;
    if (rec_buf[filenum]) free(rec_buf[filenum]);
    rec_buf[filenum] = (char*)calloc(reclen, 1);
    rec_len[filenum] = reclen;
    rec_pos[filenum] = 0;
    return 0;
}

/* PUT #f [,recnum] � write current record buffer at current record position */
void pb_put_record(int filenum) {
    if (filenum < 1 || filenum >= MAX_FILE_HANDLES || !file_handles[filenum]) return;
    /* CHW error 54: PUT on a file opened in sequential mode -> ERR 54 */
    if (file_modes[filenum] <= 2) { pb_err_var = pb_err = 54; return; }
    FILE* f = file_handles[filenum];
    long long pos = rec_pos[filenum] * (long long)rec_len[filenum];
    _fseeki64(f, pos, SEEK_SET);
    fwrite(rec_buf[filenum], 1, rec_len[filenum], f);
    fflush(f);
}

/* GET #f [,recnum] � read record into buffer (short read zero-fills) */
void pb_get_record(int filenum) {
    if (filenum < 1 || filenum >= MAX_FILE_HANDLES || !file_handles[filenum]) return;
    /* CHW error 54: GET on a file opened in sequential mode -> ERR 54 */
    if (file_modes[filenum] <= 2) { pb_err_var = pb_err = 54; return; }
    FILE* f = file_handles[filenum];
    long long pos = rec_pos[filenum] * (long long)rec_len[filenum];
    _fseeki64(f, pos, SEEK_SET);
    size_t got = fread(rec_buf[filenum], 1, rec_len[filenum], f);
    if (got < rec_len[filenum]) memset(rec_buf[filenum] + got, 0, rec_len[filenum] - got);
}

/* FIELD #n, FROM off TO ... � set current record position (1-based recnum) */
void pb_seek_record(int filenum, long long recnum) {
    if (filenum >= 1 && filenum < MAX_FILE_HANDLES) {
        rec_pos[filenum] = recnum > 0 ? recnum - 1 : 0;
    }
}

/* FIELD #n, size AS var � bind field var to file record buffer sub-section */
int pb_field_bind_file(int filenum, long long offset, unsigned len, pb_field_t* fv) {
    if (!fv) { pb_err_var = pb_err = 5; return -1; }
    if (filenum < 1 || filenum >= MAX_FILE_HANDLES || !rec_buf[filenum]) {
        fv->data = NULL; fv->offset = 0; fv->len = 0; fv->kind = 0;
        pb_err_var = pb_err = 5; return -1;
    }
    if (offset + (long long)len > (long long)rec_len[filenum]) {
        len = (unsigned)((long long)rec_len[filenum] - offset > 0 ? (long long)rec_len[filenum] - offset : 0);
    }
    fv->data = rec_buf[filenum];
    fv->offset = (unsigned)(offset > 0 ? offset : 0);
    fv->len = len;
    fv->kind = 0;
    return 0;
}

/* FIELD dyn$, size AS var � bind field var BY REFERENCE to a string variable
   slot (char**). The payload is resolved at every get/set, so reassigning the
   string variable follows automatically. */
int pb_field_bind_str(char** slot, long long offset, unsigned len, pb_field_t* fv) {
    if (!fv) { pb_err_var = pb_err = 5; return -1; }
    fv->data = (char*)slot;
    fv->offset = (unsigned)(offset > 0 ? offset : 0);
    fv->len = len;
    fv->kind = 1;
    return 0;
}

/* Resolve the base payload pointer of a field variable (file buffer / slot / private) */
static char* pb_field_base(pb_field_t* fv) {
    if (!fv || !fv->data) return NULL;
    if (fv->kind == 1) return *(char**)fv->data;
    return fv->data;
}

/* fieldvar = expr � copy into bound sub-section, pad blanks (file semantics) */
void pb_field_set(pb_field_t* fv, const char* s, unsigned slen) {
    char* base = pb_field_base(fv);
    if (!fv || !base || fv->len == 0) return;
    char* dst = base + fv->offset;
    if (slen >= fv->len) {
        memcpy(dst, s, fv->len);
    } else {
        memcpy(dst, s, slen);
        memset(dst + slen, ' ', fv->len - slen);
    }
}

/* expr = fieldvar / PRINT fieldvar � return a fresh NUL-terminated copy of the
   sub-section (payload pointer, no BSTR prefix � the compiler's string convention) */
char* pb_field_get(pb_field_t* fv) {
    char* base = pb_field_base(fv);
    if (!fv || !base || fv->len == 0) {
        char* e = (char*)malloc(1);
        e[0] = 0;
        return e;
    }
    unsigned len = fv->len;
    char* out = (char*)malloc(len + 1);
    memcpy(out, base + fv->offset, len);
    out[len] = 0;
    return out;
}

/* ===== Batch 33+34: CALLSTK call-stack tracing + PROFILE =====
   codegen pushes the procedure name at every function entry and pops at every
   exit. CALLSTKCOUNT = current depth (1 = PBMAIN); CALLSTK$(n) is 1-based
   (1 = innermost). Parameter VALUES are not captured (names only).
   PROFILE: when enabled, the same stack records call counts and elapsed
   milliseconds per procedure; PROFILE filename$ dumps
   "<Name>, <Call Count>, <Time mSec>" lines (PB-compatible format). */
#define PB_CALLSTK_MAX 256
typedef struct {
    const char* name;
    char* args; /* parameter snapshot: "7,\"hello\"" (NULL = none captured) */
    long long enter_tick; /* 0 when profiling is off */
} pb_callstk_frame_t;
static pb_callstk_frame_t pb_callstk_frames[PB_CALLSTK_MAX];
static int pb_callstk_depth = 0;
static int pb_profile_enabled = 0;
#define PB_PROFILE_MAX 256
static const char* pb_profile_names[PB_PROFILE_MAX];
static long long pb_profile_calls[PB_PROFILE_MAX];
static long long pb_profile_total_ms[PB_PROFILE_MAX];
static int pb_profile_count = 0;

static long long pb_now_ms(void) {
    return (long long)GetTickCount64();
}

/* 0 = new entry added at pb_profile_count-1; -1 = table full (not tracked) */
static int pb_profile_find_or_add(const char* name) {
    for (int i = 0; i < pb_profile_count; i++) {
        if (pb_profile_names[i] == name) return i;
    }
    if (pb_profile_count < PB_PROFILE_MAX) {
        int idx = pb_profile_count++;
        pb_profile_names[idx] = name;
        pb_profile_calls[idx] = 0;
        pb_profile_total_ms[idx] = 0;
        return idx;
    }
    return -1;
}

void pb_profile_enable(void) {
    pb_profile_enabled = 1;
}

void pb_callstk_push(const char* name) {
    if (pb_callstk_depth < PB_CALLSTK_MAX) {
        pb_callstk_frames[pb_callstk_depth].name = name;
        pb_callstk_frames[pb_callstk_depth].args = NULL;
        if (pb_profile_enabled) {
            int idx = pb_profile_find_or_add(name);
            pb_callstk_frames[pb_callstk_depth].enter_tick = pb_now_ms();
            if (idx >= 0) pb_profile_calls[idx]++;
        } else {
            pb_callstk_frames[pb_callstk_depth].enter_tick = 0;
        }
        pb_callstk_depth++;
    }
}

void pb_callstk_pop(void) {
    if (pb_callstk_depth > 0) {
        pb_callstk_depth--;
        free(pb_callstk_frames[pb_callstk_depth].args);
        pb_callstk_frames[pb_callstk_depth].args = NULL;
        if (pb_profile_enabled && pb_callstk_frames[pb_callstk_depth].enter_tick != 0) {
            long long el = pb_now_ms() - pb_callstk_frames[pb_callstk_depth].enter_tick;
            int idx = pb_profile_find_or_add(pb_callstk_frames[pb_callstk_depth].name);
            if (idx >= 0) pb_profile_total_ms[idx] += el;
        }
    }
}

long long pb_callstk_count(void) {
    return (long long)pb_callstk_depth;
}

/* CALLSTK$(n): 1-based from the innermost frame; empty string when out of range.
   PBWin format: NAME(arg1,arg2,...) with trailing "()" added here (the pushed
   name is stored WITHOUT parentheses; see codegen). */
char* pb_callstk_get(long long n) {
    if (n < 1 || n > pb_callstk_depth) return pb_bstr_alloc("", 0);
    int idx = pb_callstk_depth - (int)n;
    const char* nm = pb_callstk_frames[idx].name;
    if (!nm) nm = "?";
    const char* ar = pb_callstk_frames[idx].args;
    size_t nl = strlen(nm), al = ar ? strlen(ar) : 0;
    char* b = (char*)malloc(nl + al + 3);
    memcpy(b, nm, nl);
    b[nl] = '(';
    if (al) memcpy(b + nl + 1, ar, al);
    b[nl + 1 + al] = ')';
    b[nl + al + 2] = 0;
    char* out = pb_bstr_alloc(b, (unsigned int)(nl + al + 2));
    free(b);
    return out;
}

/* codegen: capture this procedure's parameter snapshot ("7,\"hello\"") right
   after push; replaces the innermost frame's args. */
void pb_callstk_setargs(const char* s) {
    if (pb_callstk_depth <= 0) return;
    pb_callstk_frame_t* f = &pb_callstk_frames[pb_callstk_depth - 1];
    free(f->args);
    if (s && s[0]) {
        size_t n = strlen(s);
        char* c = (char*)malloc(n + 1);
        memcpy(c, s, n);
        c[n] = 0;
        f->args = c;
    } else {
        f->args = NULL;
    }
}

/* String parameter -> "..." (double quotes, PBWin CALLSTK$ format) */
char* pb_callstk_quote(const char* s) {
    if (!s) s = "";
    size_t n = strlen(s);
    char* b = (char*)malloc(n + 3);
    b[0] = '"';
    memcpy(b + 1, s, n);
    b[n + 1] = '"';
    b[n + 2] = 0;
    char* out = pb_bstr_alloc(b, (unsigned int)(n + 2));
    free(b);
    return out;
}

/* CALLSTK filename$: write every frame, OUTERMOST first (PBWin order), one per
   line: NAME(args) — see CHW CALLSTK_statement.htm example (PBMAIN() first). */
void pb_callstk_dump(const char* filename) {
    if (!filename || !filename[0]) return;
    FILE* f = fopen(filename, "w");
    if (!f) return;
    for (int i = 0; i < pb_callstk_depth; i++) {
        const char* nm = pb_callstk_frames[i].name;
        if (!nm) nm = "?";
        const char* ar = pb_callstk_frames[i].args;
        fprintf(f, "%s(%s)\n", nm, ar ? ar : "");
    }
    fclose(f);
}

/* PROFILE filename$: "<Name>, <Call Count>, <Time mSec>" per line */

/* ===== Batch 35: REGEXPR / REGREPL (documented subset) =====
   REGEXPR mask$ IN target$ [AT start&] TO iPos& [, iLen&]
   REGREPL mask$ IN target$ WITH repl$ [AT start&] TO iPos&, newtarget$
   Documented subset: literals (case-insensitive by default), '.', '*', '+',
   '?', '^', '$', '|', '[class] / [^class] (incl. a-z ranges), '\' escapes
   (\b word boundary, \n \r \t \e \f \q, \c case-sensitive toggle, any other
   escaped char = literal), '(' ')' groups for alternation precedence.
   Longest match at the leftmost start position. Tags (\01..\99) and the
   shortest-match '\s' operator are NOT implemented (documented as subset). */

typedef struct { const char* pat; const char* s; const char* send; int cs; int depth; } pb_rex_t;

static int pb_re_is_word_char(unsigned char c) {
    return (c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z') ||
           (c >= '0' && c <= '9') || c == '_';
}

static const char* pb_rex_alt_split(const char* p) {
    int d = 0;
    while (*p) {
        if (*p == '\\') { p += 2; continue; }
        if (*p == '[') { while (*p && *p != ']') p++; p++; continue; }
        if (*p == '(') d++;
        else if (*p == ')') d--;
        else if (*p == '|' && d == 0) return p;
        p++;
    }
    return NULL;
}

static const char* pb_rex_alt_end(const char* p) {
    /* If p points at '(', return its matching ')' (the group-arm / atom_end
     * call sites). Otherwise return the first ')' at depth 0 (closes an outer
     * group). The previous version started depth at 0 and only returned on a
     * ')' seen while d==0, so a pattern that *started* with '(' never matched
     * its own closing paren and the group arm bailed out with NULL. */
    int d = 0;
    if (*p == '(') { d = 1; p++; }
    while (*p) {
        if (*p == '\\') { p += 2; continue; }
        if (*p == '[') { while (*p && *p != ']') p++; p++; continue; }
        if (*p == '(') d++;
        else if (*p == ')') {
            if (d <= 0) return p;   /* outer close (non-group-anchored call) */
            d--;
            if (d == 0) return p;   /* matched the opening group anchor */
        }
        p++;
    }
    return NULL;
}

/* Count unescaped '(' outside character classes in p. Used to advance the
 * capture-group counter past every group opened inside the current group's
 * body, so sibling groups after a group that contained nested groups still
 * receive the correct CHW left-to-right group number. */
static int pb_rex_count_groups(const char* p) {
    int n = 0;
    while (*p) {
        if (*p == '\\') { p += 2; continue; }
        if (*p == '[') { while (*p && *p != ']') p++; p++; continue; }
        if (*p == '(') n++;
        p++;
    }
    return n;
}

static int pb_rex_char_eq(int a, int b, int cs) {
    if (!cs) {
        if (a >= 'A' && a <= 'Z') a += 32;
        if (b >= 'A' && b <= 'Z') b += 32;
    }
    return a == b;
}

static const char* pb_rex_match_one(const char* pat, const char* s, const char* send, int cs) {
    if (s >= send) return NULL;
    unsigned char c = (unsigned char)*s;
    if (pat[0] == '\\') {
        switch (pat[1]) {
            case 'n': return (c == 10) ? s + 1 : NULL;
            case 'r': return (c == 13) ? s + 1 : NULL;
            case 't': return (c == 9) ? s + 1 : NULL;
            case 'e': return (c == 27) ? s + 1 : NULL;
            case 'f': return (c == 12) ? s + 1 : NULL;
            case 'q': return (c == '"') ? s + 1 : NULL;
            case 'c': return NULL; /* directive handled in match_here */
            default:
                return (pb_rex_char_eq(c, (unsigned char)pat[1], cs)) ? s + 1 : NULL;
        }
    }
    if (pat[0] == '.') return s + 1;
    if (pat[0] == '[') {
        const char* q = pat + 1;
        int negate = 0;
        if (*q == '^') { negate = 1; q++; }
        int matched = 0;
        while (*q && *q != ']') {
            if (q[1] == '-' && q[2] && q[2] != ']') {
                int lo = (unsigned char)q[0], hi = (unsigned char)q[2], cc = c;
                if (!cs) {
                    if (lo >= 'A' && lo <= 'Z') lo += 32;
                    if (hi >= 'A' && hi <= 'Z') hi += 32;
                    if (cc >= 'A' && cc <= 'Z') cc += 32;
                }
                if (cc >= lo && cc <= hi) matched = 1;
                q += 3;
            } else {
                if (pb_rex_char_eq(c, (unsigned char)q[0], cs)) matched = 1;
                q++;
            }
        }
        if (negate) matched = !matched;
        return matched ? s + 1 : NULL;
    }
    return pb_rex_char_eq(c, (unsigned char)pat[0], cs) ? s + 1 : NULL;
}

static const char* pb_rex_atom_end(const char* p) {
    if (*p == '\\') return p + 2;
    if (*p == '[') {
        const char* q = p + 1;
        while (*q && *q != ']') q++;
        return (*q == ']') ? q + 1 : q;
    }
    if (*p == '(') {
        const char* q = pb_rex_alt_end(p);
        return q ? q + 1 : p + 1;
    }
    return p + 1;
}

/* match whole pattern at s; anchors recomputed from the CURRENT s every call.
 * gstart[gdepth+1..], gend[gdepth+1..] receive (start,end) pointers for each
 * (..) capture group entered at this recursion level (CHW REGREPL \01..\99).
 * gdepth points to the caller's current group depth; this function treats
 * *gdepth as read-only except when entering a new group (it uses a local). */
#define PB_REX_MAX_GROUPS 100
static const char* pb_rex_match_here(const char* pat, const char* s, const char* send,
                                     const char* tbegin, int cs,
                                     const char** gstart, const char** gend, int gdepth) {
    if (!*pat) return s;
    int at_line_start = (s == tbegin) ||
        (s > tbegin && s[-1] == '\n') ||
        (s >= tbegin + 2 && s[-2] == '\r' && s[-1] == '\n');
    int at_line_end = (s == send) || (s < send && (s[0] == '\n' || (s[0] == '\r' && s + 1 < send && s[1] == '\n')));

    {
        const char* bar = pb_rex_alt_split(pat);
        if (bar) {
            char tmp[4096];
            size_t l = (size_t)(bar - pat);
            if (l >= sizeof(tmp)) return NULL;
            memcpy(tmp, pat, l); tmp[l] = 0;
            const char* e = pb_rex_match_here(tmp, s, send, tbegin, cs, gstart, gend, gdepth);
            if (e) return e;
            return pb_rex_match_here(bar + 1, s, send, tbegin, cs, gstart, gend, gdepth);
        }
    }
    if (*pat == '^') {
        if (!at_line_start) return NULL;
        return pb_rex_match_here(pat + 1, s, send, tbegin, cs, gstart, gend, gdepth);
    }
    if (*pat == '$') {
        if (!at_line_end) return NULL;
        return pb_rex_match_here(pat + 1, s, send, tbegin, cs, gstart, gend, gdepth);
    }
    if (pat[0] == '\\' && pat[1] == 'c') {
        return pb_rex_match_here(pat + 2, s, send, tbegin, cs ? 0 : 1, gstart, gend, gdepth);
    }
    if (pat[0] == '\\' && pat[1] == 'b') {
        /* word boundary: previous/next char class change */
        int prev_word = (s > tbegin) && pb_re_is_word_char((unsigned char)s[-1]);
        int next_word = (s < send) && pb_re_is_word_char((unsigned char)s[0]);
        if (prev_word == next_word) return NULL;
        return pb_rex_match_here(pat + 2, s, send, tbegin, cs, gstart, gend, gdepth);
    }
    {
        const char* aend = pb_rex_atom_end(pat);
        char quant = *aend;
        if (quant == '*' || quant == '+' || quant == '?') {
            int min = (quant == '+') ? 1 : 0;
            int max = (quant == '?') ? 1 : -1;
            char atom[4096];
            size_t alen = (size_t)(aend - pat);
            if (alen >= sizeof(atom)) return NULL;
            memcpy(atom, pat, alen); atom[alen] = 0;
            const char* rest = aend + 1;
            const char* cur = s;
            int n = 0;
            while ((max < 0 || n < max) && cur < send) {
                const char* e = pb_rex_match_here(atom, cur, send, tbegin, cs, gstart, gend, gdepth);
                if (!e || e == cur) break;
                cur = e; n++;
            }
            while (n >= min) {
                const char* e = pb_rex_match_here(rest, cur, send, tbegin, cs, gstart, gend, gdepth);
                if (e) return e;
                if (n == 0) break;
                const char* prev = s;
                for (int k = 1; k < n; k++) {
                    const char* e2 = pb_rex_match_here(atom, prev, send, tbegin, cs, gstart, gend, gdepth);
                    if (!e2) break;
                    prev = e2;
                }
                cur = prev;
                n--;
            }
            return NULL;
        }
    }
    if (*pat == '(') {
        const char* close = pb_rex_alt_end(pat);
        if (!close) return NULL;
        char inner[4096];
        size_t ilen = (size_t)(close - pat - 1);
        if (ilen >= sizeof(inner)) return NULL;
        memcpy(inner, pat + 1, ilen); inner[ilen] = 0;
        int child = gdepth + 1;
        if (child < PB_REX_MAX_GROUPS) gstart[child] = s;
        const char* e = pb_rex_match_here(inner, s, send, tbegin, cs, gstart, gend, child);
        if (!e) return NULL;
        if (child < PB_REX_MAX_GROUPS) gend[child] = e;
        /* After this group, the next sibling group must be numbered after this
         * group AND every group nested inside its body (CHW left-to-right
         * capture numbering). Passing the parent gdepth here reused index 1
         * for the second sibling group, so \01 grabbed the wrong capture. */
        int after = child + pb_rex_count_groups(inner);
        return pb_rex_match_here(close + 1, e, send, tbegin, cs, gstart, gend, after);
    }
    {
        const char* e = pb_rex_match_one(pat, s, send, cs);
        if (!e) return NULL;
        return pb_rex_match_here(pb_rex_atom_end(pat), e, send, tbegin, cs, gstart, gend, gdepth);
    }
}

/* Internal scan with out-group array. gstart/gend may be NULL. */
static int pb_regex_scan_ex(const char* mask, const char* target, long long start,
                            long long* opos, long long* olen,
                            const char** gstart, const char** gend) {
    *opos = 0; *olen = 0;
    size_t tlen = strlen(target);
    if (!mask || !mask[0]) return 0;
    if (start < 1) start = 1;
    if ((size_t)start > tlen + 1) return 0;
    const char* begin = target + (size_t)(start - 1);
    const char* send = target + tlen;
    for (const char* s = begin; s <= send; s++) {
        const char* e = pb_rex_match_here(mask, s, send, target, 0,
                                          gstart, gend, 0);
        if (e) {
            *opos = (long long)(s - target) + 1;
            *olen = (long long)(e - s);
            return 1;
        }
    }
    return 0;
}

int pb_regex_scan(const char* mask, const char* target, long long start,
                  long long* opos, long long* olen) {
    return pb_regex_scan_ex(mask, target, start, opos, olen, NULL, NULL);
}

char* pb_regex_replace(const char* mask, const char* target, const char* repl,
                       long long start, long long* opos) {
    *opos = 0;
    long long pos = 0, len = 0;
    const char* gstart[PB_REX_MAX_GROUPS] = {0};
    const char* gend[PB_REX_MAX_GROUPS] = {0};
    if (!pb_regex_scan_ex(mask, target, start, &pos, &len, gstart, gend)) {
        return pb_bstr_alloc(target, (int)strlen(target));
    }
    size_t tlen = strlen(target), rlen = strlen(repl);
    /* worst case: every repl char is a tag referencing a group of length <= tlen */
    size_t outsz = tlen + rlen * (tlen + 1) + 16;
    char* out = (char*)malloc(outsz ? outsz : 1);
    if (!out) return pb_bstr_alloc("", 0);
    size_t pre = (size_t)(pos - 1);
    memcpy(out, target, pre);
    size_t o = pre;
    for (size_t i = 0; i < rlen; i++) {
        if (repl[i] == '\\' && i + 1 < rlen) {
            if (repl[i+1] == '0' && i + 2 < rlen && repl[i+2] == '0') {
                /* \00 = whole match */
                memcpy(out + o, target + pre, (size_t)len);
                o += (size_t)len; i += 2;
            } else if (repl[i+1] >= '0' && repl[i+1] <= '9') {
                /* \N or \NN capture-group reference (CHW \01..\99) */
                int gnum = repl[i+1] - '0';
                i++;
                if (i + 1 < rlen && repl[i+1] >= '0' && repl[i+1] <= '9') {
                    gnum = gnum * 10 + (repl[i+1] - '0');
                    i++;
                }
                if (gnum >= 1 && gnum < PB_REX_MAX_GROUPS && gstart[gnum] && gend[gnum]) {
                    size_t gl = (size_t)(gend[gnum] - gstart[gnum]);
                    memcpy(out + o, gstart[gnum], gl);
                    o += gl;
                }
                /* unmatched group: emit nothing (CHW: non-match presumed) */
            } else {
                out[o++] = repl[i];
            }
        } else {
            out[o++] = repl[i];
        }
    }
    memcpy(out + o, target + pre + (size_t)len, tlen - pre - (size_t)len);
    o += tlen - pre - (size_t)len;
    out[o] = 0;
    /* iPos& = 1-based position immediately following the replacement in newtarget$ */
    *opos = (long long)(o + 1 - (tlen - pre - (size_t)len));
    return out;
}

void pb_profile_dump(const char* filename) {
    if (!filename || !filename[0]) return;
    FILE* f = fopen(filename, "w");
    if (!f) return;
    for (int i = 0; i < pb_profile_count; i++) {
        const char* nm = pb_profile_names[i];
        if (!nm) nm = "?";
        fprintf(f, "%s, %lld, %lld\n", nm, pb_profile_calls[i], pb_profile_total_ms[i]);
    }
    fclose(f);
}

/* ===== Batch 32: MAT matrix algebra =====
   Arrays are flat element buffers; elem_size is 1/2/4/8; is_float selects
   SINGLE/DOUBLE element decoding (ints are sign-extended, floats IEEE).
   Arithmetic is performed in double and stored back with element-type truncation.
   No bounds checking (PB semantics). */

static double pb_mat_read(const char* base, int es, int is_float, long long idx) {
    if (is_float) {
        if (es == 4) { float t; memcpy(&t, base + idx * 4, 4); return t; }
        double t; memcpy(&t, base + idx * 8, 8); return t;
    }
    double v = 0.0;
    if (es == 1) { signed char t; memcpy(&t, base + idx, 1); v = t; }
    else if (es == 2) { short t; memcpy(&t, base + idx * 2, 2); v = t; }
    else if (es == 4) { int t; memcpy(&t, base + idx * 4, 4); v = t; }
    else { long long t; memcpy(&t, base + idx * 8, 8); v = (double)t; }
    return v;
}

static void pb_mat_write(char* base, int es, int is_float, long long idx, double v) {
    if (is_float) {
        if (es == 4) { float t = (float)v; memcpy(base + idx * 4, &t, 4); return; }
        memcpy(base + idx * 8, &v, 8); return;
    }
    if (es == 1) { signed char t = (signed char)v; memcpy(base + idx, &t, 1); }
    else if (es == 2) { short t = (short)v; memcpy(base + idx * 2, &t, 2); }
    else if (es == 4) { int t = (int)v; memcpy(base + idx * 4, &t, 4); }
    else { memcpy(base + idx * 8, &v, 8); }
}

/* MAT a() = CON / CON(expr) / ZER � fill all elements */
void pb_mat_fill(char* base, int es, int is_float, long long total, double val) {
    for (long long i = 0; i < total; i++) pb_mat_write(base, es, is_float, i, val);
}

/* MAT a() = b() � element copy */
void pb_mat_copy(char* dst, const char* src, int es, long long total) {
    memcpy(dst, src, (size_t)(total * es));
}

/* MAT a() = b() + c() / b() - c() � elementwise, same size */
void pb_mat_add(char* dst, const char* a, const char* b, int es, int is_float, long long total, int sub) {
    for (long long i = 0; i < total; i++) {
        double av = pb_mat_read(a, es, is_float, i);
        double bv = pb_mat_read(b, es, is_float, i);
        pb_mat_write(dst, es, is_float, i, sub ? av - bv : av + bv);
    }
}

/* MAT a() = (expr) * b() � scalar multiplication */
void pb_mat_scale(char* dst, int es, int is_float, long long total, double s, const char* a) {
    for (long long i = 0; i < total; i++) pb_mat_write(dst, es, is_float, i, s * pb_mat_read(a, es, is_float, i));
}

/* MAT a() = IDN � 2-D square identity (rows == cols) */
void pb_mat_identity(char* dst, int es, int is_float, int rows, int cols) {
    for (int i = 0; i < rows; i++)
        for (int j = 0; j < cols; j++)
            pb_mat_write(dst, es, is_float, (long long)i * cols + j, i == j ? 1.0 : 0.0);
}

/* MAT a() = TRN(b()) � dst(rows x cols) = src(cols x rows); dst dims must be swapped */
void pb_mat_trn(char* dst, const char* src, int es, int is_float, int src_rows, int src_cols) {
    for (int i = 0; i < src_rows; i++)
        for (int j = 0; j < src_cols; j++)
            pb_mat_write(dst, es, is_float, (long long)j * src_rows + i, pb_mat_read(src, es, is_float, (long long)i * src_cols + j));
}

/* MAT a() = b() * c() � 2-D multiply: dst(l x n) = a(l x m) * b(m x n) */
void pb_mat_mul(char* dst, const char* a, const char* b, int es, int is_float, int l, int m, int n) {
    for (int i = 0; i < l; i++) {
        for (int j = 0; j < n; j++) {
            double acc = 0.0;
            for (int k = 0; k < m; k++)
                acc += pb_mat_read(a, es, is_float, (long long)i * m + k) * pb_mat_read(b, es, is_float, (long long)k * n + j);
            pb_mat_write(dst, es, is_float, (long long)i * n + j, acc);
        }
    }
}

/* MAT a() = INV(b()) � 2-D square inverse via Gauss-Jordan on the augmented
   matrix [A | I]. Returns 0 on success, -1 if singular. */
int pb_mat_inv(char* dst, const char* src, int es, int is_float, int n) {
    if (n <= 0) return -1;
    double* aug = (double*)malloc((size_t)n * n * 2 * sizeof(double));
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < n; j++) aug[(size_t)i * n * 2 + j] = pb_mat_read(src, es, is_float, (long long)i * n + j);
        for (int j = 0; j < n; j++) aug[(size_t)i * n * 2 + n + j] = (i == j) ? 1.0 : 0.0;
    }
    int singular = 0;
    for (int col = 0; col < n && !singular; col++) {
        int pivot = col;
        for (int r = col + 1; r < n; r++)
            if (fabs(aug[(size_t)r * n * 2 + col]) > fabs(aug[(size_t)pivot * n * 2 + col])) pivot = r;
        if (fabs(aug[(size_t)pivot * n * 2 + col]) < 1e-12) { singular = 1; break; }
        if (pivot != col) {
            for (int j = 0; j < n * 2; j++) {
                double t = aug[(size_t)col * n * 2 + j];
                aug[(size_t)col * n * 2 + j] = aug[(size_t)pivot * n * 2 + j];
                aug[(size_t)pivot * n * 2 + j] = t;
            }
        }
        double d = aug[(size_t)col * n * 2 + col];
        for (int j = 0; j < n * 2; j++) aug[(size_t)col * n * 2 + j] /= d;
        for (int r = 0; r < n; r++) {
            if (r == col) continue;
            double f = aug[(size_t)r * n * 2 + col];
            for (int j = 0; j < n * 2; j++) aug[(size_t)r * n * 2 + j] -= f * aug[(size_t)col * n * 2 + j];
        }
    }
    for (int i = 0; i < n; i++)
        for (int j = 0; j < n; j++)
            pb_mat_write(dst, es, is_float, (long long)i * n + j, aug[(size_t)i * n * 2 + n + j]);
    free(aug);
    return singular ? -1 : 0;
}

/* x$ = expr for FIELD dyn$-bound strings � copy into a fresh mutable buffer
   (string constants live in read-only memory and must never be written through) */
/* Copy a string into a FRESH buffer and rebind slot.  The buffer carries the
   runtime's BSTR layout (4-byte little-endian byte length immediately before
   the data pointer), so the result is a valid BSTR: SysStringByteLen-style
   readers (-4 header) and strlen/NUL consumers both work, and STRPTR is the
   data pointer.  Allocating without the header broke downstream BSTR readers
   (batch 259: MID$ statement crashed 0xC0000005 in the very next string
   concatenation after pb_mid_assign replaced the slot). */
void pb_str_assign_copy(char** slot, const char* src, unsigned len) {
    if (!slot) return;
    char* buf = (char*)malloc(len + 5);   /* 4-byte length header + data + NUL */
    *(unsigned*)buf = len;
    char* data = buf + 4;
    if (src && len) memcpy(data, src, len);
    data[len] = 0;
    *slot = data;
}

/* ASC(s$, pos&) = code&  (CHW ASC_statement.htm, batch 256A)
   Replace one character in a dynamic string by its character code.
   pos& is 1-based; negative counts from the end (-1 = last char).
   Null string, pos==0 or out of range: the operation is ignored.
   ANSI strings: char code must be in 0..255 (outside is ignored). */
void pb_asc_stmt(char** slot, long long pos, long long code) {
    if (!slot) return;
    char* s = *slot;
    if (!s) return;                          /* null pointer: ignored */
    /* batch (A-family probe): index by the BSTR byte-prefixed length, NOT
       strlen - binary strings (MK*$, CHR$ 0) embed NUL bytes, and strlen
       would truncate so positions past the first NUL were wrongly ignored.
       PBWin writes those positions (NUL-safe), matching pb_asc_get. */
#ifdef _WIN32
    long long len = (long long)SysStringByteLen(s);
#else
    long long len = (long long)strlen(s);
#endif
    if (len == 0) return;                    /* zero-length string: ignored */
    if (pos == 0 || pos > len) return;       /* zero or beyond length: ignored */
    long long idx = (pos > 0) ? (pos - 1) : (len + pos);
    if (idx < 0 || idx >= len) return;       /* negative out of range: ignored */
    if (code < 0 || code > 255) return;      /* ANSI range guard */
    /* the buffer may be a read-only string constant: copy before writing */
    pb_str_assign_copy(slot, s, (unsigned)len);
    (*slot)[idx] = (char)code;
}

/* ASC statement on a fixed-length string (inline [N x i8] buffer). */
void pb_asc_stmt_fixed(char* buf, long long buflen, long long pos, long long code) {
    if (!buf || buflen <= 0 || pos == 0) return;
    long long idx = (pos > 0) ? (pos - 1) : (buflen + pos);
    if (idx < 0 || idx >= buflen) return;
    if (code < 0 || code > 255) return;
    buf[idx] = (char)code;
}

/* MID$(svar$, start& [, length&]) = repl$  (CHW MID$ statement, batch 259)
   Overwrite characters of svar$ in place, starting at 1-based start&.
   Replaces min(length(repl$), length& (if given)) characters; svar$ never
   grows or shrinks.  start&==0, negative start, or out-of-range: ignored. */
/* BSTR byte length: 4-byte little-endian length prefix immediately before the
   data pointer (same layout SysStringByteLen reads).  Read it directly instead
   of calling the oleaut32 dllimport: that import crashed (0xC0000005) on the
   compiler's static string constants under -O2, and the layout is fixed here. */
static __inline long long pb_bstr_byte_len(const char* p) {
    if (!p) return 0;
    const unsigned char* h = (const unsigned char*)p - 4;
    unsigned l = (unsigned)h[0] | ((unsigned)h[1] << 8) |
                 ((unsigned)h[2] << 16) | ((unsigned)h[3] << 24);
    return (long long)l;
}
void pb_mid_assign(char** slot, long long start, long long length, char* repl) {
    if (!slot) return;
    char* s = *slot;
    if (!s || !repl) return;
    long long slen = pb_bstr_byte_len(s);
    long long rlen = pb_bstr_byte_len(repl);
    if (start <= 0 || start > slen) return;          /* 0 / negative / past end: ignored */
    long long idx = start - 1;
    long long n = rlen;
    if (length > 0 && length < n) n = length;       /* explicit length cap */
    if (n > slen - idx) n = slen - idx;             /* never write past the end */
    if (n <= 0) return;
    pb_str_assign_copy(slot, s, (unsigned)slen);    /* buffer may be read-only: copy first */
    memcpy((*slot) + idx, repl, (size_t)n);
}

/* CHW MID$ statement shared resolution (MID_statement.htm).
   Given a target buffer of length L (1-based positions 1..L), the raw
   start/third arguments, and whether `third` is a Count (is_range=0) or a
   TO-End position (is_range=1), compute the 0-based write index and the number
   of bytes to copy.  Returns 1 on success, 0 for a no-op.

   Semantics, probed against PBWin10 on a STRING*10 buffer:
     - start < 0 counts backward from the end (-1 = last char): pos = L+start+1
     - start == 0 or pos outside 1..L                 -> no-op
     - is_range=0 (Count): third<0 -> L-ABS(third); third==0 (omitted) -> to end
     - is_range=1 (End): end<0 counts backward; count = end-pos+1
     - write min(count, len(repl), L-pos+1) bytes; never extends past L. */
static int pb_mid_resolve(long long L, long long start, long long third,
                          long long is_range, long long rlen,
                          long long *out_idx, long long *out_n) {
    long long pos = (start < 0) ? (L + start + 1) : start;
    if (pos < 1 || pos > L) return 0;
    long long avail = L - pos + 1;
    long long count;
    if (is_range) {
        long long end = (third < 0) ? (L + third + 1) : third;
        count = end - pos + 1;
    } else if (third < 0) {
        count = L - (-third);
    } else if (third == 0) {
        count = avail;
    } else {
        count = third;
    }
    long long n = count;
    if (n > rlen) n = rlen;
    if (n > avail) n = avail;
    if (n <= 0) return 0;
    *out_idx = pos - 1;
    *out_n = n;
    return 1;
}

/* MID$ statement on a fixed-length STRING*n inline [N x i8] buffer (batch 316).
   buflen is the declared N; the trailing NUL at buf[N-1] is never touched, so
   the buffer keeps its length.  third is a Count (is_range=0) or an End
   position (is_range=1). */
void pb_mid_assign_fixed(char* buf, long long buflen, long long start,
                         long long third, long long is_range, const char* repl) {
    if (!buf || buflen <= 0 || !repl) return;
    long long rlen = pb_bstr_byte_len(repl);
    long long idx, n;
    if (!pb_mid_resolve(buflen, start, third, is_range, rlen, &idx, &n)) return;
    memcpy(buf + idx, repl, (size_t)n);
}

/* MID$ statement on a dynamic string slot, full CHW start/count/End resolution
   (the TO-End form; the comma form keeps the original pb_mid_assign).  The slot
   may point at a read-only string constant, so rebind a fresh mutable buffer
   first. */
void pb_mid_assign_dynamic(char** slot, long long start, long long third,
                           long long is_range, const char* repl) {
    if (!slot || !repl) return;
    char* s = *slot;
    long long L = s ? pb_bstr_byte_len(s) : 0;
    long long rlen = pb_bstr_byte_len(repl);
    long long idx, n;
    if (!pb_mid_resolve(L, start, third, is_range, rlen, &idx, &n)) return;
    pb_str_assign_copy(slot, s, (unsigned)L);
    memcpy((*slot) + idx, repl, (size_t)n);
}

/* MID$ FUNCTION form (MID_function.htm): return a slice of s using the same
   CHW start/count/End resolution as the assignment form.  third is a Count
   (is_range=0; 0 = omitted -> to end) or a TO-End position (is_range=1).
   Negative start/third count backward from the end; zero/out-of-range yields
   an empty string.  Pass L as the "replacement length" cap so a slice is only
   bounded by the requested count and the available characters. */
char* pb_mid_func(const char* s, long long start, long long third,
                  long long is_range) {
    static const char empty[] = "";
    if (!s) s = empty;
    long long L = (long long)strlen(s);
    long long idx, n;
    if (!pb_mid_resolve(L, start, third, is_range, L, &idx, &n)) {
        return pb_bstr_alloc("", 0);
    }
    return pb_bstr_alloc(s + idx, (unsigned int)n);
}

/* FIELD RESET var � unbind, becomes empty */
void pb_field_reset(pb_field_t* fv) {
    if (!fv) return;
    fv->data = NULL;
    fv->offset = 0;
    fv->len = 0;
    fv->kind = 0;
}

/* FIELD STRING var � copy current sub-section into a private buffer, unbind */
void pb_field_tostr(pb_field_t* fv) {
    char* base = pb_field_base(fv);
    if (!fv) return;
    unsigned len = fv->len;
    if (len == 0 || !base) { pb_field_reset(fv); return; }
    char* copy = (char*)malloc(len);
    memcpy(copy, base + fv->offset, len);
    fv->data = copy;
    fv->offset = 0;
    fv->len = len;
    fv->kind = 0; /* private buffer */
}

/* PUT$ #f, str$ � write ANSI string at current file position */
int pb_put_string(int f, const char* s) {
    if (f < 1 || f >= MAX_FILE_HANDLES || file_handles[f] == NULL) { pb_err_var = pb_err = 52; return -1; }
    /* CHW error 54: PUT$ on a file opened in sequential mode -> ERR 54 */
    if (file_modes[f] <= 2) { pb_err_var = pb_err = 54; return -1; }
    if (!s) { pb_err_var = pb_err = 5; return -1; }
    size_t n = strlen(s);
    if (n == 0) return 0;
    return fwrite(s, 1, n, file_handles[f]) == n ? 0 : -1;
}

/* GET$ #f, count, dest$: read count bytes from a binary file into a BSTR */
int pb_get_string(int f, long long count, char** dest) {
    if (f < 1 || f >= MAX_FILE_HANDLES || file_handles[f] == NULL) { pb_err_var = pb_err = 52; return -1; }
    /* CHW error 54: GET$ on a file opened in sequential mode -> ERR 54 */
    if (file_modes[f] <= 2) { pb_err_var = pb_err = 54; return -1; }
    if (count < 0 || count > 0x7FFFFFFFLL) { pb_err_var = pb_err = 5; return -1; }
    char* buf = (char*)malloc((size_t)count + 1);
    if (!buf) { pb_err_var = pb_err = 5; return -1; }
    size_t got = fread(buf, 1, (size_t)count, file_handles[f]);
    *dest = pb_bstr_alloc(buf, (unsigned int)got);
    free(buf);
    /* NB: do NOT SysFreeString the previous *dest � the variable is often
       initialized to a codegen string constant (not a BSTR), and freeing it
       crashes. Old BSTRs leak instead; acceptable for a compiler runtime. */
    return (got == (size_t)count) ? 0 : -1;
}

/* PUT$$ #f, StrgExpr: write a WIDE (UTF-16LE) string at the file position */
int pb_put_wstring(int f, const char* s) {
    if (f < 1 || f >= MAX_FILE_HANDLES || file_handles[f] == NULL) { pb_err_var = pb_err = 52; return -1; }
    /* CHW error 54: PUT$$ on a file opened in sequential mode -> ERR 54 */
    if (file_modes[f] <= 2) { pb_err_var = pb_err = 54; return -1; }
    if (!s) { pb_err_var = pb_err = 5; return -1; }
    size_t n = strlen(s);
    if (n == 0) return 0;
    int wlen = MultiByteToWideChar(PB_CP_ACP, 0, s, (int)n, NULL, 0);
    if (wlen <= 0) { pb_err_var = pb_err = 5; return -1; }
    short* wbuf = (short*)malloc((size_t)wlen * 2);
    if (!wbuf) { pb_err_var = pb_err = 5; return -1; }
    MultiByteToWideChar(PB_CP_ACP, 0, s, (int)n, wbuf, wlen);
    size_t written = fwrite(wbuf, 2, (size_t)wlen, file_handles[f]);
    free(wbuf);
    return written == (size_t)wlen ? 0 : -1;
}

/* GET\$\$ #f, Count&, StrgVar: read Count WIDE chars (Count*2 bytes), convert to ANSI */
int pb_get_wstring(int f, long long count, char** dest) {
    if (f < 1 || f >= MAX_FILE_HANDLES || file_handles[f] == NULL) { pb_err_var = pb_err = 52; return -1; }
    /* CHW error 54: GET$$ on a file opened in sequential mode -> ERR 54 */
    if (file_modes[f] <= 2) { pb_err_var = pb_err = 54; return -1; }
    if (count < 0 || count > 0x3FFFFFFFLL) { pb_err_var = pb_err = 5; return -1; }
    size_t bytes = (size_t)count * 2;
    char* buf = (char*)malloc(bytes + 2);
    if (!buf) { pb_err_var = pb_err = 5; return -1; }
    size_t got = fread(buf, 1, bytes, file_handles[f]);
    size_t wchars = got / 2;
    int ansi_len = WideCharToMultiByte(PB_CP_ACP, 0, (const short*)buf, (int)wchars,
                                       NULL, 0, NULL, NULL);
    if (ansi_len < 0) ansi_len = 0;
    char* out = (char*)malloc((size_t)ansi_len + 1);
    if (!out) { free(buf); pb_err_var = pb_err = 5; return -1; }
    if (ansi_len > 0) {
        WideCharToMultiByte(PB_CP_ACP, 0, (const short*)buf, (int)wchars,
                            out, ansi_len, NULL, NULL);
    }
    out[ansi_len] = '\0';
    *dest = pb_bstr_alloc(out, (unsigned int)ansi_len);
    free(out);
    free(buf);
    /* same NB as pb_get_string: do not free the previous *dest */
    return (got == bytes) ? 0 : -1;
}

/* SHIFT LEFT � logical left shift on 64-bit */
long long pb_shift_left(long long v, int n) { return v << (n & 63); }

/* SHIFT RIGHT � keep_sign=1 is arithmetic (SIGNED), 0 is logical */
long long pb_shift_right(long long v, int n, int keep_sign) {
    int k = n & 63;
    if (keep_sign) return v >> k;
    return (long long)((unsigned long long)v >> k);
}

/* SHIFT RIGHT width-aware (CHW SHIFT_statement.htm).
 * The SIGNED option "does not allow the sign (positive or negative) of the
 * value to change", where sign is the MSB of ivar's DECLARED width, even for
 * nominally-unsigned BYTE/WORD.  The fork previously zero-extended ivar to
 * 64 bits before the arithmetic shift, so BYTE 0xDD (bit 7 set) SIGNED RIGHT
 * 1 yielded 0x6E (110) instead of the documented 0xEE (238).  Sign-extend
 * from the declared width (8/16/32/64) to 64 bits first; the logical case is
 * masked to the width as well.  Count is taken modulo the width. */
long long pb_shift_right_w(long long v, int n, int width, int keep_sign) {
    if (width <= 0 || width >= 64) {
        int k = n & 63;
        if (keep_sign) return v >> k;
        return (long long)((unsigned long long)v >> k);
    }
    unsigned long long mask = (1ULL << (unsigned)width) - 1ULL;
    unsigned long long x = ((unsigned long long)v) & mask;
    unsigned k = ((unsigned)n) % (unsigned)width;
    if (keep_sign) {
        unsigned long long sign_bit = 1ULL << ((unsigned)width - 1);
        if (x & sign_bit) x |= ~mask;
        return (long long)(x >> k);
    }
    return (long long)(x >> k);
}

/* ROTATE LEFT/RIGHT � 64-bit circular shift */
long long pb_rotate_left(long long v, int n) {
    int k = n & 63;
    if (k == 0) return v;
    return (v << k) | ((unsigned long long)v >> (64 - k));
}
long long pb_rotate_right(long long v, int n) {
    int k = n & 63;
    if (k == 0) return v;
    return ((unsigned long long)v >> k) | (v << (64 - k));
}

/* R2 ROTATE width-aware (CHW ROTATE_statement.htm): ivar may be Byte (8),
 * Word/Integer (16), Dword/Long (32), or Quad (64). The fork previously
 * widened to i64, rotated across all 64 bits, then truncated - so the wrap
 * bit travelled from bit 0 to bit 63 and was dropped, turning BYTE 221
 * ROR 1 into 110 instead of the documented 238. Mask to the operand width
 * and take the count modulo that width. CHW: "ROTATE rotates all the bits
 * in ivar without special regard to the sign bit of a signed integral-class
 * variable" - i.e. treat the narrow value as unsigned bits. */
static unsigned long long pb_rotate_mask(int width) {
    if (width >= 64) return ~0ULL;
    return (1ULL << (unsigned)width) - 1ULL;
}
long long pb_rotate_left_w(long long v, int n, int width) {
    if (width <= 0) width = 64;
    unsigned long long mask = pb_rotate_mask(width);
    unsigned long long x = ((unsigned long long)v) & mask;
    unsigned k = ((unsigned)n) % (unsigned)width;
    if (k == 0) return (long long)x;
    unsigned long long r = (x << k) | (x >> ((unsigned)width - k));
    return (long long)(r & mask);
}
long long pb_rotate_right_w(long long v, int n, int width) {
    if (width <= 0) width = 64;
    unsigned long long mask = pb_rotate_mask(width);
    unsigned long long x = ((unsigned long long)v) & mask;
    unsigned k = ((unsigned)n) % (unsigned)width;
    if (k == 0) return (long long)x;
    unsigned long long r = (x >> k) | (x << ((unsigned)width - k));
    return (long long)(r & mask);
}

/* ===== Batch 3: PLAY SOUND / SPLIT / ARRAY SHUFFLE ===== */

#ifdef _WIN32
__declspec(dllimport) int __stdcall Beep(unsigned long dwFreq, unsigned long dwDuration);
__declspec(dllimport) int __stdcall MessageBeep(unsigned long uType);
#endif

/* PLAY SOUND freq&, duration& � speaker beep via Beep() */
int pb_play_sound(long freq, long dur) {
#ifdef _WIN32
    return Beep((unsigned long)freq, (unsigned long)dur) ? 0 : -1;
#else
    pb_err_var = pb_err = 68; return -1;
#endif
}

/* BEEP (CHW BEEP_statement.htm): play the default Windows waveform sound,
   i.e. the configurable "Default Beep" (Control Panel -> Sounds). */
void pb_beep(void) {
#ifdef _WIN32
    MessageBeep(0xFFFFFFFF);
#else
    pb_err_var = pb_err = 68;
#endif
}

/* SPLIT [WORD] MainStr, Part1Len TO Part1Var, Part2Var.
   word != 0 (CHW SPLIT_statement.htm, PBWin10 measured): "Part1 will not end
   on a partial word".  PBWin behaviour: (1) drop any trailing space that would
   end up in Part1; (2) if the cut then still divides a word (Part1 ends mid-word
   AND Part2 starts mid-word), back the cut off to the preceding space, leaving
   that space as the leading space of Part2 (the caller may LTRIM$ it). */
void pb_split(const char* src, int n, char** out1, char** out2, int word) {
    if (!src) src = "";
    size_t len = strlen(src);
    if (n < 0) n = 0;
    if ((size_t)n > len) n = (int)len;
    if (word) {
        /* (1) trailing space in Part1 -> back off. */
        while (n > 0 && src[n - 1] == ' ') n--;
        /* (2) cut still divides a word -> back to the preceding space. */
        if (n > 0 && (size_t)n < len && src[n - 1] != ' ' && src[n] != ' ') {
            int s = n - 1;
            while (s > 0 && src[s] != ' ') s--;
            n = (src[s] == ' ') ? s : 0;
        }
    }
    *out1 = pb_bstr_alloc(src, n);
    *out2 = pb_bstr_alloc(src + n, (int)(len - n));
}

/* ARRAY SHUFFLE � Fisher-Yates shuffle in place */
void pb_array_shuffle(void* base, int elem_size, int total) {
    if (!base || total <= 1) return;
    char* p = (char*)base;
    for (int i = total - 1; i > 0; i--) {
        int j = rand() % (i + 1);
        if (j == i) continue;
        char* a = p + (size_t)i * elem_size;
        char* b = p + (size_t)j * elem_size;
        for (int k = 0; k < elem_size; k++) {
            char t = a[k];
            a[k] = b[k];
            b[k] = t;
        }
    }
}

/* ===== Batch 4: DATA / READ / RESTORE =====
   CHW READ_function.htm: "There is a limit of 64 Kilobytes and 16384 separate
   data items per Sub, Function, Method, or Property, and it is not possible
   to read DATA from outside of the scope of current procedure." The fork used
   one global pool; now each procedure owns its own list, keyed by name, with
   an activate/deactivate stack so nested calls do not cross-pollute. */

#define MAX_DATA_ITEMS 16384
#define MAX_DATA_POOLS 64
#define DATA_STACK 32

typedef struct {
    char name[48];
    const char* items[MAX_DATA_ITEMS];
    int count;
    int cursor;
    int in_use;
} DataPool;

static DataPool data_pools[MAX_DATA_POOLS];
static DataPool* data_active = NULL;
static DataPool* data_stack[DATA_STACK];
static int data_sp = 0;

void pb_data_activate(const char* procname) {
    DataPool* found = NULL;
    for (int i = 0; i < MAX_DATA_POOLS; i++) {
        if (data_pools[i].in_use && strcmp(data_pools[i].name, procname) == 0) {
            found = &data_pools[i];
            break;
        }
    }
    if (!found) {
        for (int i = 0; i < MAX_DATA_POOLS; i++) {
            if (!data_pools[i].in_use) {
                found = &data_pools[i];
                strncpy(found->name, procname, sizeof(found->name) - 1);
                found->name[sizeof(found->name) - 1] = 0;
                found->count = 0;
                found->cursor = 0;
                found->in_use = 1;
                break;
            }
        }
    }
    if (data_sp < DATA_STACK) {
        data_stack[data_sp++] = data_active;
    }
    data_active = found;
    /* On each entry the local list is (re)built from the inline DATA items. */
    if (data_active) {
        data_active->count = 0;
        data_active->cursor = 0;
    }
}

void pb_data_deactivate(void) {
    if (data_sp > 0) {
        data_sp--;
        data_active = data_stack[data_sp];
    }
}

void pb_data_append(const char* s) {
    if (!data_active || !s) return;
    if (data_active->count >= MAX_DATA_ITEMS) return;
    data_active->items[data_active->count++] = s;
}

char* pb_read_data_str(void) {
    if (!data_active || data_active->cursor >= data_active->count) {
        return pb_bstr_alloc("", 0);
    }
    const char* item = data_active->items[data_active->cursor];
    data_active->cursor++;
    return pb_bstr_alloc(item, (int)strlen(item));
}

double pb_read_data_num(void) {
    if (!data_active || data_active->cursor >= data_active->count) return 0.0;
    return atof(data_active->items[data_active->cursor++]);
}

/* READ$(n%) — CHW READ_function.htm: 1-based random access to the local DATA
   list; n > DATACOUNT returns an empty string with no runtime error. */
char* pb_read_data_idx(long long n) {
    if (!data_active || n < 1 || n > data_active->count) {
        return pb_bstr_alloc("", 0);
    }
    const char* item = data_active->items[n - 1];
    return pb_bstr_alloc(item, (int)strlen(item));
}

void pb_data_reset(void) {
    if (data_active) data_active->cursor = 0;
}

// PROCESS GET/SET PRIORITY - current process priority class
unsigned long pb_process_get_priority(void) {
    return GetPriorityClass(GetCurrentProcess());
}

int pb_process_set_priority(unsigned long pri) {
    return SetPriorityClass(GetCurrentProcess(), pri) ? 0 : -1;
}

/* CHW SHELL statement (SHELL_statement.htm): run a child process SYNCHRONOUSLY.
 * "The SHELLing thread ... is suspended until the SHELLed program ends."
 * mode&: 0=hide window, 1=normal with focus (default), 2=minimized, 3=maximized,
 * 4=normal-no-focus, 6=minimized-no-focus.  exit_out (may be NULL) receives the
 * child process exit code (GetExitCodeProcess).  Returns the child process id. */
#ifdef _WIN32
__declspec(dllimport) int __stdcall CreateProcessA(const char*, char*, void*, void*, int,
    unsigned long, void*, const char*, void*, void*);
__declspec(dllimport) unsigned long __stdcall WaitForSingleObject(void* h, unsigned long ms);
__declspec(dllimport) int __stdcall GetExitCodeProcess(void* h, unsigned long* code);
__declspec(dllimport) int __stdcall CloseHandle(void* h);
#endif
long pb_shell_sync(char* cmd, int mode, long* exit_out) {
    if (exit_out) *exit_out = 0;
    if (!cmd || !*cmd) return 0;
#ifdef _WIN32
    struct { unsigned long cb; void* reserved; char* desktop; char* title;
             unsigned long x, y, xsz, ysz; unsigned long xcount, ycount;
             unsigned long fillattr; unsigned long flags; unsigned short show;
             unsigned short reserved2; void* reserved3; void* stdin_, *stdout_, *stderr_; } si;
    struct { void* hProcess; void* hThread; unsigned long pid, tid; } pi;
    memset(&si, 0, sizeof(si)); si.cb = sizeof(si);
    memset(&pi, 0, sizeof(pi));
    unsigned long create_flags = (mode == 0) ? 0x08000000UL /* CREATE_NO_WINDOW */ : 0;
    si.flags = 0x00000001UL /* STARTF_USESHOWWINDOW */;
    si.show = (unsigned short)(mode < 0 ? 1 : (mode > 10 ? 1 : mode));
    char* cmdmut = (char*)malloc(strlen(cmd) + 1);
    if (!cmdmut) return 0;
    strcpy(cmdmut, cmd);
    int ok = CreateProcessA(NULL, cmdmut, NULL, NULL, 0/*FALSE*/,
                            create_flags, NULL, NULL, &si, &pi);
    free(cmdmut);
    if (!ok) { if (exit_out) *exit_out = -1; return 0; }
    WaitForSingleObject(pi.hProcess, 0xFFFFFFFFUL /* INFINITE */);
    unsigned long code = 0;
    GetExitCodeProcess(pi.hProcess, &code);
    CloseHandle(pi.hThread);
    CloseHandle(pi.hProcess);
    if (exit_out) *exit_out = (long)code;
    return (long)pi.pid;
#else
    (void)mode;
    if (exit_out) *exit_out = -1;
    return 0;
#endif
}

/* ARRAY SCAN arr([idx]) [FOR count], OP expr, TO var& � first matching relative index, 0 = none.
   op: 0='=', 1='<>', 2='<', 3='>', 4='<=', 5='>=' */
long long pb_array_scan_num(char* base, int elem_size, long long total, long long index,
                            long long count, long long value, int op) {
    if (index < 1) index = 1;
    if (index > total) return 0;
    if (count <= 0) count = total - index + 1;
    if (index + count - 1 > total) count = total - index + 1;
    for (long long i = 0; i < count; i++) {
        long long v;
        if (elem_size == 1) {
            signed char t;
            memcpy(&t, base + (index - 1 + i) * 1, 1);
            v = t;
        } else if (elem_size == 2) {
            short t;
            memcpy(&t, base + (index - 1 + i) * 2, 2);
            v = t;
        } else if (elem_size == 4) {
            int t;
            memcpy(&t, base + (index - 1 + i) * 4, 4);
            v = t;
        } else {
            memcpy(&v, base + (index - 1 + i) * 8, 8);
        }
        int hit = 0;
        switch (op) {
            case 0: hit = (v == value); break;
            case 1: hit = (v != value); break;
            case 2: hit = (v < value); break;
            case 3: hit = (v > value); break;
            case 4: hit = (v <= value); break;
            default: hit = (v >= value); break;
        }
        if (hit) return index + i;
    }
    return 0;
}

long long pb_array_scan_str(char* base, long long total, long long index, long long count,
                            char* strval) {
    if (index < 1) index = 1;
    if (index > total) return 0;
    if (count <= 0) count = total - index + 1;
    if (index + count - 1 > total) count = total - index + 1;
    for (long long i = 0; i < count; i++) {
        char** sp = (char**)(base + (index - 1 + i) * 8);
        if (*sp && strval && strcmp(*sp, strval) == 0) return index + i;
        if (!*sp && !strval) return index + i;
    }
    return 0;
}

/* ARRAY INSERT arr(index) [FOR count] [, value] — insert element; with count,
   only count elements shift up to make room (PBWin-measured; count 0/absent =
   shift to the end). */
void pb_array_insert_num(char* base, int elem_size, long long total, long long index,
                         long long count, long long value) {
    if (index < 1) index = 1;
    if (index > total) return;
    if (count <= 0) count = total - index + 1;
    long long shift = count;
    if (index - 1 + shift > total) shift = total - index + 1;
    memmove(base + (index - 1 + 1) * elem_size, base + (index - 1) * elem_size,
            (size_t)((shift - 1) * elem_size));
    memcpy(base + (index - 1) * elem_size, &value, (size_t)elem_size);
}

void pb_array_insert_str(char* base, long long total, long long index,
                         long long count, char* bstr) {
    if (index < 1) index = 1;
    if (index > total) return;
    if (count <= 0) count = total - index + 1;
    long long shift = count;
    if (index - 1 + shift > total) shift = total - index + 1;
    memmove(base + index * 8, base + (index - 1) * 8, (size_t)((shift - 1) * 8));
    memcpy(base + (index - 1) * 8, &bstr, 8);
}

/* ARRAY DELETE arr(index) [FOR count] [, fill] — remove element(s), shift up;
   the vacated tail slot is filled with `fill` (0 / empty string when absent). */
void pb_array_delete(char* base, int elem_size, long long total, long long index,
                     long long count, int is_string, long long fill_num, char* fill_str) {
    if (index < 1) index = 1;
    if (count <= 0) count = 1;
    if (index > total) return;
    if (index + count > total) count = total - index + 1;
    long long from = index - 1;  /* 0-based */
    if (count == 1) {
        memmove(base + from * elem_size, base + (from + 1) * elem_size,
                (size_t)((total - from - 1) * elem_size));
        if (is_string) {
            char** sp = (char**)(base + (total - 1) * elem_size);
            if (*sp) { SysFreeString(*sp); *sp = NULL; }
            if (fill_str) *sp = pb_bstr_alloc(fill_str, (unsigned int)strlen(fill_str));
        } else {
            memcpy(base + (total - 1) * elem_size, &fill_num, (size_t)elem_size);
        }
    } else {
        memmove(base + from * elem_size, base + (from + 1) * elem_size,
                (size_t)((count - 1) * elem_size));
        if (is_string) {
            char** sp = (char**)(base + (from + count - 1) * elem_size);
            if (*sp) { SysFreeString(*sp); *sp = NULL; }
            if (fill_str) *sp = pb_bstr_alloc(fill_str, (unsigned int)strlen(fill_str));
        } else {
            memcpy(base + (from + count - 1) * elem_size, &fill_num, (size_t)elem_size);
        }
    }
}


/* ARRAY ARRAYIX arr() � set each element to its element index (1-based).
   Numeric arrays: value = index (widened to element size).
   String arrays: element = decimal text of index (BSTR). */
void pb_array_arrayix(char* base, int elem_size, long long total, int type) {
    for (long long i = 1; i <= total; i++) {
        char* p = base + (i - 1) * elem_size;
        if (type == 0) {
            if (elem_size == 1) {
                signed char t = (signed char)i;
                memcpy(p, &t, 1);
            } else if (elem_size == 2) {
                short t = (short)i;
                memcpy(p, &t, 2);
            } else if (elem_size == 4) {
                int t = (int)i;
                memcpy(p, &t, 4);
            } else {
                long long t = i;
                memcpy(p, &t, 8);
            }
        } else {
            char buf[24];
            _i64toa_s(i, buf, 24, 10);
            char** sp = (char**)p;
            if (*sp) SysFreeString(*sp);
            *sp = (char*)SysAllocStringByteLen(buf, (unsigned int)strlen(buf));
        }
    }
}

/* CLIPBOARD SET TEXT / GET TEXT / RESET (Win32) */
#define CF_TEXT 1
#define CF_BITMAP 2
#define CF_OEMTEXT 7
#define CF_UNICODETEXT 13
#define GMEM_MOVEABLE 0x0042

/* CCUR / CCUX — round to fixed decimals (currency semantics, PBWin10). */
double pb_ccur(double x) { return round(x * 10000.0) / 10000.0; }
double pb_ccux(double x) { return round(x * 100.0) / 100.0; }

/* CSET$(s$, len& [, pad]) — center s$ into len bytes, pad with first char
   of pad or space.  Left padding = (len - slen)/2 (PBWin behavior). */
char* pb_cset_str(const char* s, long long len, const char* pad) {
    unsigned long slen = (unsigned long)strlen(s);
    char padc = (pad && pad[0]) ? pad[0] : ' ';
    if (len < 0) len = 0;
    unsigned long n = (unsigned long)len;
    char* r = pb_bstr_alloc(NULL, n);
    if (n == 0) return r;
    unsigned long left = (slen >= n) ? 0 : (unsigned long)((n - slen) / 2);
    unsigned long take = (slen < n) ? slen : n;
    for (unsigned long i = 0; i < n; i++) r[i] = padc;
    memcpy(r + left, s, take);
    return r;
}

/* LSET$(s, len [USING u]): left-justify s into a len result, padding the
   right with the first char of u (or spaces).  If s is longer than len, only
   the left-most len bytes are returned.  (CHW LSET$_function.htm) */
char* pb_lset_str(const char* s, long long len, const char* pad) {
    unsigned long slen = (unsigned long)strlen(s);
    char padc = (pad && pad[0]) ? pad[0] : ' ';
    if (len < 0) len = 0;
    unsigned long n = (unsigned long)len;
    char* r = pb_bstr_alloc(NULL, n);
    if (n == 0) return r;
    for (unsigned long i = 0; i < n; i++) r[i] = padc;
    unsigned long take = (slen < n) ? slen : n;
    memcpy(r, s, take);
    return r;
}

/* RSET$(s, len [USING u]): right-justify s into a len result, padding the
   left with the first char of u (or spaces).  If s is longer than len, only
   the left-most len bytes are returned.  (CHW RSET$_function.htm) */
char* pb_rset_str(const char* s, long long len, const char* pad) {
    unsigned long slen = (unsigned long)strlen(s);
    char padc = (pad && pad[0]) ? pad[0] : ' ';
    if (len < 0) len = 0;
    unsigned long n = (unsigned long)len;
    char* r = pb_bstr_alloc(NULL, n);
    if (n == 0) return r;
    for (unsigned long i = 0; i < n; i++) r[i] = padc;
    unsigned long take = (slen < n) ? slen : n;
    memcpy(r + (n - take), s, take);
    return r;
}

int pb_clipboard_set_text(const char* text) {
    if (!OpenClipboard(0)) return 0;   /* FALSE on failure */
    EmptyClipboard();
    unsigned long len = (unsigned long)strlen(text);
    void* h = GlobalAlloc(GMEM_MOVEABLE, len + 1);
    int ok = 0;
    if (h) {
        char* p = (char*)GlobalLock(h);
        if (p) {
            memcpy(p, text, len + 1);
            GlobalUnlock(h);
        }
        if (SetClipboardData(CF_TEXT, h)) ok = -1;  /* TRUE on success (PBWin10) */
    }
    CloseClipboard();
    return ok;
}


/* batch 262: GET ClipResult must reflect real success/failure (PBWin10 writes
   -1=TRUE on success, 0 on failure).  GET helpers return the string, so the
   result code is reported through this global + getter. */
static int pb_clip_ok = 0;
int pb_clip_last_ok(void) { return pb_clip_ok; }


char* pb_clipboard_get_text(void) {
    char* empty = pb_bstr_alloc("", 0);
    pb_clip_ok = 0;
    if (!OpenClipboard(0)) return empty;
    void* h = GetClipboardData(CF_TEXT);
    if (!h) { CloseClipboard(); return empty; }
    char* p = (char*)GlobalLock(h);
    if (!p) { CloseClipboard(); return empty; }
    unsigned long len = (unsigned long)strlen(p);
    char* r = pb_bstr_alloc(p, len);
    GlobalUnlock(h);
    CloseClipboard();
    pb_clip_ok = -1;
    return r;
}

int pb_clipboard_reset(void) {
    pb_clip_ok = 0;
    if (!OpenClipboard(0)) return 0;   /* FALSE on failure */
    EmptyClipboard();
    CloseClipboard();
    pb_clip_ok = -1;
    return -1;                          /* TRUE on success (PBWin10) */
}

/* CLIPBOARD SET/GET OEMTEXT / UNICODE / BITMAP — CHW 9 forms (batch 258e).
   OEMTEXT converts ANSI<->OEM (console charset).  UNICODE (legacy PB form)
   stores/reads UTF-16LE bytes inside an ANSI-typed string; the byte count
   comes from SysStringByteLen because embedded NULs break strlen.  BITMAP
   moves a GRAPHIC BITMAP (HBITMAP) handle in/out of the clipboard. */
int pb_clipboard_set_oemtext(const char* text) {
    if (!OpenClipboard(0)) return 0;
    EmptyClipboard();
    unsigned long len = (unsigned long)strlen(text);
    void* h = GlobalAlloc(GMEM_MOVEABLE, len + 1);
    int ok = 0;
    if (h) {
        char* p = (char*)GlobalLock(h);
        if (p) {
            CharToOemA(text, p);
            p[len] = 0;
            GlobalUnlock(h);
        }
        if (SetClipboardData(CF_OEMTEXT, h)) ok = -1;
    }
    CloseClipboard();
    return ok;
}

char* pb_clipboard_get_oemtext(void) {
    char* empty = pb_bstr_alloc("", 0);
    pb_clip_ok = 0;
    if (!OpenClipboard(0)) return empty;
    void* h = GetClipboardData(CF_OEMTEXT);
    if (!h) {
        /* batch 262: fall back to CF_TEXT (ANSI) - CHW CLIPBOARD_statement.htm
           "Windows automatically converts string data between TEXT, OEMTEXT,
           and UNICODE"; PBWin10 GET OEMTEXT on a TEXT-only clipboard returns
           the text (probe: SET TEXT "abc" -> GET OEMTEXT = "abc", ClipResult -1). */
        h = GetClipboardData(CF_TEXT);
    }
    if (!h) { CloseClipboard(); return empty; }
    char* p = (char*)GlobalLock(h);
    if (!p) { CloseClipboard(); return empty; }
    unsigned long len = (unsigned long)strlen(p);
    char* tmp = (char*)malloc(len + 1);
    if (!tmp) { GlobalUnlock(h); CloseClipboard(); return empty; }
    OemToCharA(p, tmp);
    tmp[len] = 0;
    char* r = pb_bstr_alloc(tmp, (int)strlen(tmp));
    free(tmp);
    GlobalUnlock(h);
    CloseClipboard();
    pb_clip_ok = -1;
    return r;
}

int pb_clipboard_set_unicode(const char* bytes) {
    if (!OpenClipboard(0)) return 0;
    EmptyClipboard();
    unsigned long len = (unsigned long)SysStringByteLen(bytes); /* UTF-16LE byte count */
    void* h = GlobalAlloc(GMEM_MOVEABLE, len + 2);
    int ok = 0;
    if (h) {
        char* p = (char*)GlobalLock(h);
        if (p) {
            memcpy(p, bytes, len);
            p[len] = 0; p[len + 1] = 0;   /* wide nul */
            GlobalUnlock(h);
        }
        if (SetClipboardData(CF_UNICODETEXT, h)) ok = -1;
    }
    CloseClipboard();
    return ok;
}

char* pb_clipboard_get_unicode(void) {
    char* empty = pb_bstr_alloc("", 0);
    pb_clip_ok = 0;
    if (!OpenClipboard(0)) return empty;
    int is_wide = 1;
    void* h = GetClipboardData(CF_UNICODETEXT);
    if (!h) {
        /* batch 262: fall back to CF_TEXT (ANSI) then widen to UTF-16LE -
           CHW: Windows automatically converts string data between TEXT,
           OEMTEXT, and UNICODE.  PBWin10 GET UNICODE on TEXT-only clipboard
           returns the text (probe: SET TEXT "abc" -> GET UNICODE = "abc",
           LEN 6 = UTF-16LE bytes, ClipResult -1). */
        is_wide = 0;
        h = GetClipboardData(CF_TEXT);
    }
    if (!h) { CloseClipboard(); return empty; }
    char* p = (char*)GlobalLock(h);
    if (!p) { CloseClipboard(); return empty; }
    char* r;
    if (is_wide) {
        unsigned long chars = 0;
        while (((const unsigned short*)p)[chars] != 0) chars++;   /* wide strlen */
        unsigned long len = chars * 2;
        r = pb_bstr_alloc(NULL, len);
        if (len) memcpy(r, p, len);
    } else {
        unsigned long n = (unsigned long)strlen(p);
        r = pb_bstr_alloc(NULL, n * 2);
        for (unsigned long i = 0; i < n; i++) {
            r[i * 2] = p[i];
            r[i * 2 + 1] = 0;
        }
    }
    GlobalUnlock(h);
    CloseClipboard();
    pb_clip_ok = -1;
    return r;
}

int pb_clipboard_set_bitmap(long long hbitmap) {
    pb_clip_ok = 0;
    if (!OpenClipboard(0)) return 0;
    EmptyClipboard();
    int ok = SetClipboardData(CF_BITMAP, (void*)(intptr_t)hbitmap) ? -1 : 0;
    CloseClipboard();
    pb_clip_ok = ok;
    return ok;
}

long long pb_clipboard_get_bitmap(void) {
    pb_clip_ok = 0;
    if (!OpenClipboard(0)) return 0;
    void* h = GetClipboardData(CF_BITMAP);
    long long r = h ? (long long)(intptr_t)h : 0;
    CloseClipboard();
    if (h) pb_clip_ok = -1;
    return r;
}
static int pb_winsock_inited = 0;
static int pb_winsock_init(void) {
    if (pb_winsock_inited) return 0;
    unsigned short ver = 0x0202; /* 2.2 */
    char data[408] = {0};
    if (WSAStartup(ver, data) != 0) return -1;
    pb_winsock_inited = 1;
    return 0;
}

void pb_host_addr(const char* host, unsigned long* out) {
    *out = 0;
    if (pb_winsock_init() != 0) return;
    char local[256] = {0};
    const char* name = host;
    if (!name || !name[0]) {
        if (gethostname(local, 255) == 0) {
            name = local;
        } else {
            return;
        }
    }
    pb_hostent* he = gethostbyname(name);
    if (!he || !he->h_addr_list || !he->h_addr_list[0]) return;
    unsigned long ip = 0;
    memcpy(&ip, he->h_addr_list[0], 4);
    *out = ip;
}

char* pb_host_name(unsigned long ip) {
    if (pb_winsock_init() != 0) return pb_bstr_alloc("", 0);
    if (ip == 0) {
        char buf[256] = {0};
        if (gethostname(buf, 255) != 0) return pb_bstr_alloc("", 0);
        return pb_bstr_alloc(buf, (unsigned int)strlen(buf));
    }
    pb_in_addr ia;
    ia.s_addr = ip;
    char* dotted = inet_ntoa(ia);
    if (!dotted) return pb_bstr_alloc("", 0);
    pb_hostent* he = gethostbyaddr((char*)&ia, 4, 2 /* AF_INET */);
    if (!he || !he->h_name) return pb_bstr_alloc("", 0);
    return pb_bstr_alloc(he->h_name, (unsigned int)strlen(he->h_name));
}

void pb_input_flush(void) {
    fflush(stdin);
}

/* ===== Batch 81: INPUT / LINE INPUT (console) ===== */

static char* pb_read_line_console(void) {
    static char buf[65536];
    buf[0] = 0;
    if (fgets(buf, sizeof(buf), stdin)) {
        size_t n = strlen(buf);
        while (n > 0 && (buf[n-1] == '\n' || buf[n-1] == '\r')) {
            buf[--n] = 0;
        }
    }
    return pb_bstr_alloc(buf, strlen(buf));
}

char* pb_input_console(char* prompt, int has_prompt, int no_newline) {
    if (has_prompt && prompt) {
        fputs(prompt, stdout);
        fputs("? ", stdout);
        if (!no_newline) fputc('\n', stdout);
        fflush(stdout);
    }
    return pb_read_line_console();
}

char* pb_line_input_console(char* prompt, int has_prompt) {
    if (has_prompt && prompt) {
        fputs(prompt, stdout);
        fflush(stdout);
    }
    return pb_read_line_console();
}

/* ===== Batch 5: PEEK / POKE ===== */

int pb_peek8(void* a) { return *(unsigned char*)a; }
int pb_peek16(void* a) { return *(unsigned short*)a; }
int pb_peek16s(void* a) { return *(short*)a; }
long pb_peek32(void* a) { return *(long*)a; }
unsigned long pb_peek32u(void* a) { return *(unsigned long*)a; }
long long pb_peek64(void* a) { return *(long long*)a; }
float pb_peekf(void* a) { float f; memcpy(&f, a, 4); return f; }
double pb_peekd(void* a) { double d; memcpy(&d, a, 8); return d; }

void pb_poke8(void* a, int v) { *(unsigned char*)a = (unsigned char)v; }
void pb_poke16(void* a, int v) { *(unsigned short*)a = (unsigned short)v; }
void pb_poke32(void* a, long v) { *(long*)a = v; }
void pb_poke64(void* a, long long v) { *(long long*)a = v; }
void pb_pokef(void* a, float v) { memcpy(a, &v, 4); }
void pb_poked(void* a, double v) { memcpy(a, &v, 8); }

/* PEEK$ / POKE$ - raw byte memory access (CHW PEEK/POKE pages).
 * PEEK$ returns a BSTR holding exactly `count` bytes; the STRINGZ form scans
 * for a NUL and returns at most count-1 bytes.  POKE$ writes the BSTR's raw
 * bytes; STRINGZ additionally appends a NUL terminator. */
char* pb_peek_str(void* addr, int count) {
    if (count <= 0) return pb_bstr_alloc("", 0);
    return pb_bstr_alloc((const char*)addr, (unsigned int)count);
}
char* pb_peek_str_stringz(void* addr, int count) {
    if (count <= 0) return pb_bstr_alloc("", 0);
    int n = 0;
    const unsigned char* p = (const unsigned char*)addr;
    while (n < count - 1 && p[n] != 0) n++;
    return pb_bstr_alloc((const char*)addr, (unsigned int)n);
}
void pb_poke_str(void* addr, const char* s) {
    if (s == NULL) return;
    unsigned int n = (unsigned int)pb_str_len(s);
    memcpy(addr, s, n);
}
void pb_poke_str_stringz(void* addr, const char* s) {
    unsigned int n = (s == NULL) ? 0 : (unsigned int)pb_str_len(s);
    if (n) memcpy(addr, s, n);
    ((unsigned char*)addr)[n] = 0;
}

/* PEEK$$ / POKE$$ - raw wide-character memory access.  `count` is a number of
 * wide characters; the BSTR byte length is count*2.  WSTRINGZ scans for a
 * NUL wide char and returns at most count-1 wide chars. */
char* pb_peek_wstr(void* addr, int count) {
    if (count <= 0) return pb_bstr_alloc("", 0);
    return pb_bstr_alloc((const char*)addr, (unsigned int)(count * 2));
}
char* pb_peek_wstr_wstringz(void* addr, int count) {
    if (count <= 0) return pb_bstr_alloc("", 0);
    int n = 0;
    const unsigned short* p = (const unsigned short*)addr;
    while (n < count - 1 && p[n] != 0) n++;
    return pb_bstr_alloc((const char*)addr, (unsigned int)(n * 2));
}
void pb_poke_wstr(void* addr, const char* s) {
    if (s == NULL) return;
    unsigned int n = (unsigned int)pb_str_len(s);
    memcpy(addr, s, n);
}
void pb_poke_wstr_wstringz(void* addr, const char* s) {
    unsigned int n = (s == NULL) ? 0 : (unsigned int)pb_str_len(s);
    if (n) memcpy(addr, s, n);
    ((unsigned short*)addr)[n / 2] = 0;
}

/* ===== Stubs for external DLL functions not yet available ===== */

#ifdef _WIN32
#define STDCALL __stdcall
#else
#define STDCALL
#endif

/* VARICHEK.DLL � HiScore: checks/updates high score table
 * Returns: 2 if highest score, else 1; x=-1 for check-only
 * Stubbed: linked directly into exe (cdecl, not dllimport) */
int HiScore(int x, const char* Winner, const char* PathSpec) {
    (void)x; (void)Winner; (void)PathSpec;
    return 1;  /* Not highest score */
}

/* Example DLL stub � RemarksData: returns STRPTR to a string
 * Stubbed: linked directly into exe (cdecl, not dllimport) */
int RemarksData(int x) {
    static char humor[] = "No remarks available.";
    (void)x;
    return (int)(size_t)humor;  /* Return pointer as integer (32-bit) */
}

/* EZLIB functions are now nooped in the compiler (should_noop_function).
 * No runtime stubs needed. */

/* LSET (BSTR target): left-justify src into a fixed-length PB string, pad spaces */
void pb_lset(char** dest, const char* src, int len) {
    char* old = *dest;
    char* buf = (char*)malloc((size_t)len + 1);
    int src_len = src ? (int)strlen(src) : 0;
    int i;
    for (i = 0; i < len; i++) {
        buf[i] = (i < src_len) ? src[i] : 32;
    }
    buf[len] = (char)0;
    *dest = pb_bstr_alloc(buf, (unsigned int)len);
    free(buf);
    if (old) SysFreeString(old);
}

/* CSET (BSTR target): center src in a fixed-length PB string, pad spaces */
void pb_cset(char** dest, const char* src, int len) {
    char* old = *dest;
    char* buf = (char*)malloc((size_t)len + 1);
    int src_len = src ? (int)strlen(src) : 0;
    int pad_l = (src_len < len) ? (len - src_len) / 2 : 0;
    int i;
    for (i = 0; i < len; i++) {
        if (i < pad_l) {
            buf[i] = 32;
        } else if (i - pad_l < src_len) {
            buf[i] = src[i - pad_l];
        } else {
            buf[i] = 32;
        }
    }
    buf[len] = (char)0;
    *dest = pb_bstr_alloc(buf, (unsigned int)len);
    free(buf);
    /* NB: do NOT SysFreeString the previous *dest (may be a codegen constant) */
}

/* CSET (buffer target, e.g. STRING * N): center src, pad spaces */
void pb_cset_buf(char* dest, const char* src, int len) {
    int src_len = src ? (int)strlen(src) : 0;
    int pad_l = (src_len < len) ? (len - src_len) / 2 : 0;
    int i;
    for (i = 0; i < len; i++) {
        if (i < pad_l) {
            dest[i] = 32;
        } else if (i - pad_l < src_len) {
            dest[i] = src[i - pad_l];
        } else {
            dest[i] = 32;
        }
    }
    dest[len] = (char)0;
}

/* RSET (BSTR target): right-justify src into a fixed-length PB string, pad spaces */
void pb_rset(char** dest, const char* src, int len) {
    char* old = *dest;
    char* buf = (char*)malloc((size_t)len + 1);
    int src_len = src ? (int)strlen(src) : 0;
    int pad = (src_len < len) ? (len - src_len) : 0;
    int i;
    for (i = 0; i < len; i++) {
        buf[i] = (i < pad) ? 32 : ((i - pad) < src_len ? src[i - pad] : 32);
    }
    buf[len] = (char)0;
    *dest = pb_bstr_alloc(buf, (unsigned int)len);
    free(buf);
    if (old) SysFreeString(old);
}

/* R2 RSET on a dynamic BSTR target (CHW RSET_statement.htm example:
 * `a$ = SPACE$(20): RSET a$ = "Right-align"` -> 9 spaces + 11 chars = 20).
 * The "space of the string variable" for a dynamic BSTR is its current LEN.
 * pad=NULL means pad with spaces; otherwise the first character of pad is
 * used (CHW USING). If src is longer than the current width, truncate src
 * from the right (take the leftmost `width` bytes). */
void pb_rset_dynamic(char** dest, const char* src, const char* pad) {
    if (!dest) return;
    char* old = *dest;
    int width = old ? (int)strlen(old) : 0;
    if (width < 0) width = 0;
    const char* s = src ? src : "";
    int src_len = (int)strlen(s);
    int take = src_len < width ? src_len : width;
    char padc = (pad && pad[0]) ? pad[0] : ' ';
    char* buf = (char*)malloc((size_t)width + 1);
    int padn = width - take;
    int i;
    for (i = 0; i < padn; i++) buf[i] = (char)padc;
    if (take > 0) memcpy(buf + padn, s, (size_t)take);
    buf[width] = (char)0;
    *dest = pb_bstr_alloc(buf, (unsigned)width);
    free(buf);
    if (old) SysFreeString(old);
}

/* ERASE: zero-out a static array (non-string) or NULL-out string elements */
void pb_erase_array(char* base, int elem_size, int count, int is_string) {
    if (is_string) {
        char** p = (char**)base;
        int i;
        for (i = 0; i < count; i++) p[i] = NULL;
    } else {
        memset(base, 0, (size_t)elem_size * (size_t)count);
    }
}

/* LSET (buffer target, e.g. STRING * N fixed-length var): left-justify src, pad spaces */
void pb_lset_buf(char* dest, const char* src, int len) {
    int src_len = src ? (int)strlen(src) : 0;
    int i;
    for (i = 0; i < len; i++) {
        dest[i] = (i < src_len) ? src[i] : 32;
    }
    dest[len] = (char)0;
}

/* RSET (buffer target): right-justify src, pad spaces */
void pb_rset_buf(char* dest, const char* src, int len) {
    int src_len = src ? (int)strlen(src) : 0;
    int pad = (src_len < len) ? (len - src_len) : 0;
    int i;
    for (i = 0; i < len; i++) {
        dest[i] = (i < pad) ? 32 : ((i - pad) < src_len ? src[i - pad] : 32);
    }
    dest[len] = (char)0;
}

/* LSET ABS (buffer target): left-justify src, leave padding positions
   unchanged from their original content (CHW LSET_statement.htm). */
void pb_lset_abs_buf(char* dest, const char* src, int len) {
    int src_len = src ? (int)strlen(src) : 0;
    int take = src_len < len ? src_len : len;
    if (take > 0) memmove(dest, src, (size_t)take);
    dest[len] = (char)0;
}

/* RSET ABS (buffer target): right-justify src, leave padding positions
   unchanged from their original content. */
void pb_rset_abs_buf(char* dest, const char* src, int len) {
    int src_len = src ? (int)strlen(src) : 0;
    int take = src_len < len ? src_len : len;
    int pad = len - take;
    if (take > 0) memmove(dest + pad, src, (size_t)take);
    dest[len] = (char)0;
}

/* LSET USING (buffer target): left-justify src, pad with first char of pad. */
void pb_lset_using_buf(char* dest, const char* src, int len, const char* pad) {
    char padc = (pad && pad[0]) ? pad[0] : ' ';
    int src_len = src ? (int)strlen(src) : 0;
    int i;
    for (i = 0; i < len; i++) {
        dest[i] = (i < src_len) ? src[i] : padc;
    }
    dest[len] = (char)0;
}

/* RSET USING (buffer target): right-justify src, pad with first char of pad. */
void pb_rset_using_buf(char* dest, const char* src, int len, const char* pad) {
    char padc = (pad && pad[0]) ? pad[0] : ' ';
    int src_len = src ? (int)strlen(src) : 0;
    int p = (src_len < len) ? (len - src_len) : 0;
    int i;
    for (i = 0; i < len; i++) {
        dest[i] = (i < p) ? padc : ((i - p) < src_len ? src[i - p] : padc);
    }
    dest[len] = (char)0;
}

/* RESET: close all open files */
/* RESET: close all open files */
void pb_reset(void) {
    for (int i = 1; i < MAX_FILE_HANDLES; i++) {
        if (file_handles[i]) {
            fclose(file_handles[i]);
            file_handles[i] = NULL;
        }
    }
}

/* FLUSH: flush file buffers to disk */
void pb_flush(int filenum) {
    if (filenum >= 1 && filenum < MAX_FILE_HANDLES && file_handles[filenum]) {
        fflush(file_handles[filenum]);
    }
}

/* NAME: rename a file - CHW-specified errors:
 *   53 = source does not exist
 *   58 = destination already exists
 *   51 = source open/locked by the caller (ACCESS_DENIED on MoveFileEx)
 */
int pb_name(const char* old_path, const char* new_path) {
    if (!old_path || !new_path || !*old_path || !*new_path) { pb_err_var = pb_err = 76; return -1; }
    /* source must exist (CHW: Error 53) */
    if (GetFileAttributesA(old_path) == 0xFFFFFFFF) { pb_err_var = pb_err = 53; return -1; }
    /* destination must not already exist (CHW: Error 58) */
    if (GetFileAttributesA(new_path) != 0xFFFFFFFF) { pb_err_var = pb_err = 58; return -1; }
    if (!MoveFileExA(old_path, new_path, 0)) {
        unsigned long e = GetLastError();
        /* ERROR_ACCESS_DENIED(5): source open/locked by caller (CHW: Error 51) */
        if (e == 5) { pb_err_var = pb_err = 51; }
        else { pb_err_var = pb_err = 53; }
        return -1;
    }
    pb_err = 0;
    return 0;
}

/* WRITE # support: per-file "first field" tracking */
static int write_first_field[MAX_FILE_HANDLES] = {0};
/* Set when a WRITE# ends with a trailing comma/semicolon: the record is not
   closed, so the next WRITE# on this file continues it instead of resetting. */
static int write_record_open[MAX_FILE_HANDLES] = {0};

/* Convert a C %g-style numeric string into PowerBASIC WRITE# numeric style.
 * CHW WRITE# outputs: uppercase E in scientific form, no leading zero on the
 * exponent (e.g. "1.234E-5", "1E+8"), and no integer zero before a leading
 * decimal point (e.g. ".5", "-.5", ".0001234"). The C library %g emits
 * lowercase e, a padded two-digit exponent and a leading "0."; fix both. */
static void pb_style_num(char* s) {
    char* e = strchr(s, 'e');
    if (e) {
        *e = 'E';
        char* mp = e + 1; /* exponent start: optional sign then digits */
        char* dp = mp;
        char* wp = mp; /* where normalized digits are written (after sign) */
        if (*dp == '+' || *dp == '-') {
            dp++;
            wp++;
        }
        while (*dp == '0' && isdigit((unsigned char)dp[1])) dp++;
        size_t k = 0;
        while (dp[k]) {
            wp[k] = dp[k];
            k++;
        }
        wp[k] = 0;
    }
    if (s[0] == '0' && s[1] == '.') {
        memmove(s, s + 1, strlen(s));
    } else if (s[0] == '-' && s[1] == '0' && s[2] == '.') {
        memmove(s + 1, s + 2, strlen(s + 2) + 1);
    }
}

void pb_write_file_begin(int filenum) {
    if (filenum >= 1 && filenum < MAX_FILE_HANDLES) {
        if (write_record_open[filenum]) {
            /* Continue the record: keep write_first_field cleared so the next
               field emits the separating comma. */
            write_record_open[filenum] = 0;
            write_first_field[filenum] = 0;
        } else {
            write_first_field[filenum] = 1;
        }
    }
}
/* Trailing comma/semicolon: keep the record open for the next WRITE#. */
void pb_write_file_keep(int filenum) {
    if (filenum >= 1 && filenum < MAX_FILE_HANDLES) {
        write_record_open[filenum] = 1;
        write_first_field[filenum] = 0;
    }
}
void pb_write_file_str(int filenum, const char* s) {
    if (filenum >= 1 && filenum < MAX_FILE_HANDLES && file_handles[filenum]) {
        if (!write_first_field[filenum]) fputc(',', file_handles[filenum]);
        write_first_field[filenum] = 0;
        fputc(34, file_handles[filenum]); /* double quote */
        fputs(s ? s : "", file_handles[filenum]);
        fputc(34, file_handles[filenum]);
    }
}
void pb_write_file_int(int filenum, long long v) {
    if (filenum >= 1 && filenum < MAX_FILE_HANDLES && file_handles[filenum]) {
        if (!write_first_field[filenum]) fputc(',', file_handles[filenum]);
        write_first_field[filenum] = 0;
        fprintf(file_handles[filenum], "%lld", v);
    }
}
/* Numeric literal path: emit the pre-normalized source-form text verbatim
   (field separator handled, but no surrounding quotes). */
void pb_write_file_raw(int filenum, const char* s) {
    if (filenum >= 1 && filenum < MAX_FILE_HANDLES && file_handles[filenum]) {
        if (!write_first_field[filenum]) fputc(',', file_handles[filenum]);
        write_first_field[filenum] = 0;
        fputs(s ? s : "", file_handles[filenum]);
    }
}
void pb_write_file_sng(int filenum, float v) {
    if (filenum >= 1 && filenum < MAX_FILE_HANDLES && file_handles[filenum]) {
        if (!write_first_field[filenum]) fputc(',', file_handles[filenum]);
        write_first_field[filenum] = 0;
        char buf[64];
        snprintf(buf, sizeof(buf), "%.7g", (double)v);
        pb_style_num(buf);
        fputs(buf, file_handles[filenum]);
    }
}
void pb_write_file_dbl(int filenum, double v) {
    if (filenum >= 1 && filenum < MAX_FILE_HANDLES && file_handles[filenum]) {
        if (!write_first_field[filenum]) fputc(',', file_handles[filenum]);
        write_first_field[filenum] = 0;
        char buf[64];
        snprintf(buf, sizeof(buf), "%.15g", v);
        pb_style_num(buf);
        fputs(buf, file_handles[filenum]);
    }
}
void pb_write_file_newline(int filenum) {
    if (filenum >= 1 && filenum < MAX_FILE_HANDLES && file_handles[filenum]) {
        fputs("\r\n", file_handles[filenum]);
        write_first_field[filenum] = 1;
        write_record_open[filenum] = 0;
        fflush(file_handles[filenum]);
    }
}

/* SEEK: position file pointer (PB is 1-based) */
long long pb_lof(int filenum) {
    if (filenum >= 1 && filenum < MAX_FILE_HANDLES && file_handles[filenum]) {
        FILE* fp = file_handles[filenum];
        long cur = ftell(fp);
        fseek(fp, 0, SEEK_END);
        long long len = ftell(fp);
        fseek(fp, cur, SEEK_SET);
        return len;
    }
    return 0;
}

long long pb_loc(int filenum) {
    if (filenum >= 1 && filenum < MAX_FILE_HANDLES && file_handles[filenum]) {
        /* PB semantics by mode (measured on PBWin10):
           RANDOM  -> current record number
           BINARY  -> 1-based byte position (SEEK #f,4 then LOC = 4)
           sequential INPUT/OUTPUT/APPEND -> byte position / 128 (fresh INPUT = 0) */
        int m = file_modes[filenum];
        if (m == 4) return rec_pos[filenum];
        if (m == 3) return ftell(file_handles[filenum]) + 1;
        return ftell(file_handles[filenum]) / 128;
    }
    return 0;
}

/* SEEK function (CHW SEEK_function.htm): return the 1-based position where
 * the next I/O will occur.  RANDOM -> next record number; every other mode
 * -> next byte position.  This is distinct from the legacy LOC function,
 * which on sequential files reports ftell/128 (128-byte block convention).
 * The fork previously aliased SEEK(f) to pb_loc. */
long long pb_seek_pos(int filenum) {
    if (filenum >= 1 && filenum < MAX_FILE_HANDLES && file_handles[filenum]) {
        int m = file_modes[filenum];
        if (m == 4) return rec_pos[filenum] + 1;
        return (long long)ftell(file_handles[filenum]) + 1;
    }
    return 0;
}

void pb_seek(int filenum, long long pos) {
    if (filenum >= 1 && filenum < MAX_FILE_HANDLES && file_handles[filenum]) {
        _fseeki64(file_handles[filenum], pos - 1, SEEK_SET);
    }
}

/* GET #f [, pos], var: read raw bytes from a binary file into a variable.
   pos <= 0 means "current position" (caller passes -1 when omitted). */
void pb_get(int filenum, long long pos, char* buf, long long size) {
    if (filenum >= 1 && filenum < MAX_FILE_HANDLES && file_handles[filenum]) {
        /* CHW error 54: GET on a file opened in sequential mode -> ERR 54 */
        if (file_modes[filenum] <= 2) { pb_err_var = pb_err = 54; return; }
        /* PBWin: on a RANDOM file, pos is the 1-based RECORD number ->
           byte offset (pos-1)*rec_len; on a BINARY file it is a 1-based
           byte offset (PBWin-measured record layout, batch 269 probe).
           An omitted position (pos <= 0) reads at the CURRENT file pointer,
           so no seek is issued. */
        if (pos > 0) {
            long long off = pos - 1;
            if (file_modes[filenum] == 4) off *= (long long)rec_len[filenum];
            _fseeki64(file_handles[filenum], off, SEEK_SET);
        }
        fread(buf, 1, (size_t)size, file_handles[filenum]);
    }
}

/* PUT #f [, pos], var: write raw bytes from a variable to a binary file. */
void pb_put(int filenum, long long pos, char* buf, long long size) {
    if (filenum >= 1 && filenum < MAX_FILE_HANDLES && file_handles[filenum]) {
        /* CHW error 54: PUT on a file opened in sequential mode -> ERR 54 */
        if (file_modes[filenum] <= 2) { pb_err_var = pb_err = 54; return; }
        /* Omitted position (pos <= 0) writes at the CURRENT file pointer. */
        if (pos > 0) {
            long long off = pos - 1;
            if (file_modes[filenum] == 4) off *= (long long)rec_len[filenum];
            _fseeki64(file_handles[filenum], off, SEEK_SET);
        }
        fwrite(buf, 1, (size_t)size, file_handles[filenum]);
        fflush(file_handles[filenum]);
    }
}

/* PUT #f [, pos], strvar: PBWin writes a 2-byte LE length prefix + bytes at
   the record offset (RANDOM: (pos-1)*rec_len; BINARY: pos-1 byte offset).
   The record area is NOT padded to rec_len (PBWin-measured, batch 269). */
void pb_put_str(int filenum, long long pos, const char* s) {
    if (filenum >= 1 && filenum < MAX_FILE_HANDLES && file_handles[filenum]) {
        if (file_modes[filenum] <= 2) { pb_err_var = pb_err = 54; return; }
        /* Omitted position (pos <= 0) writes at the CURRENT file pointer. */
        if (pos > 0) {
            long long off = pos - 1;
            if (file_modes[filenum] == 4) off *= (long long)rec_len[filenum];
            _fseeki64(file_handles[filenum], off, SEEK_SET);
        }
        unsigned long n = (s != NULL) ? SysStringByteLen(s) : 0;
        unsigned char hdr[2] = { (unsigned char)(n & 0xFF), (unsigned char)((n >> 8) & 0xFF) };
        fwrite(hdr, 1, 2, file_handles[filenum]);
        if (n > 0) fwrite(s, 1, n, file_handles[filenum]);
        fflush(file_handles[filenum]);
    }
}

/* GET #f [, pos], strvar: read the 2-byte LE length prefix, then n bytes.
   Short read (prefix past EOF) yields an empty string, err stays 0.
   NOTE: like pb_str_assign_copy, the old slot value is NOT freed (the fork's
   string slots are static/compiler-managed; freeing a stack/static slot was
   a heap-corruption crash, batch 269). */
void pb_get_str(int filenum, long long pos, char** dest) {
    if (filenum >= 1 && filenum < MAX_FILE_HANDLES && file_handles[filenum] && dest) {
        if (file_modes[filenum] <= 2) { pb_err_var = pb_err = 54; return; }
        /* Omitted position (pos <= 0) reads at the CURRENT file pointer. */
        if (pos > 0) {
            long long off = pos - 1;
            if (file_modes[filenum] == 4) off *= (long long)rec_len[filenum];
            _fseeki64(file_handles[filenum], off, SEEK_SET);
        }
        unsigned char hdr[2] = { 0, 0 };
        size_t got = fread(hdr, 1, 2, file_handles[filenum]);
        unsigned long n = 0;
        if (got == 2) {
            n = (unsigned long)hdr[0] | ((unsigned long)hdr[1] << 8);
            if (n > 0) {
                char* tmp = (char*)malloc(n);
                if (tmp) {
                    size_t rb = fread(tmp, 1, n, file_handles[filenum]);
                    char* nb = pb_bstr_alloc(tmp, (long)rb);
                    free(tmp);
                    if (nb) *dest = nb;
                    return;
                }
            }
        }
        *dest = pb_bstr_alloc("", 0);
    }
}

/* ARRAY SORT helpers */
static int cmp_i32(const void* a, const void* b) {
    long x = *(const long*)a, y = *(const long*)b;
    return (x > y) - (x < y);
}
static int cmp_i64(const void* a, const void* b) {
    long long x = *(const long long*)a, y = *(const long long*)b;
    return (x > y) - (x < y);
}
static int cmp_dbl(const void* a, const void* b) {
    double x = *(const double*)a, y = *(const double*)b;
    return (x > y) - (x < y);
}
static int cmp_f32(const void* a, const void* b) {
    float x = *(const float*)a, y = *(const float*)b;
    return (x > y) - (x < y);
}
static int cmp_str(const void* a, const void* b) {
    const char* x = *(char* const*)a;
    const char* y = *(char* const*)b;
    if (!x) x = "";
    if (!y) y = "";
    return strcmp(x, y);
}
/* ARRAY SORT arr(): ascending sort of a PB array.
   type: 0=LONG(4) 1=QUAD(8) 2=DOUBLE(8) 3=STRING(ptr,8) 4=SINGLE(4) */
void pb_array_sort(char* base, int elem_size, int count, int type) {
    if (!base || count <= 1) return;
    switch (type) {
        case 0: qsort(base, (size_t)count, (size_t)elem_size, cmp_i32); break;
        case 1: qsort(base, (size_t)count, (size_t)elem_size, cmp_i64); break;
        case 2: qsort(base, (size_t)count, (size_t)elem_size, cmp_dbl); break;
        case 3: qsort(base, (size_t)count, (size_t)elem_size, cmp_str); break;
        case 4: qsort(base, (size_t)count, (size_t)elem_size, cmp_f32); break;
    }
}

static int arr_cmp_e(const void* a, const void* b, int type) {
    switch (type) {
        case 0: return cmp_i32(a, b);
        case 1: return cmp_i64(a, b);
        case 2: return cmp_dbl(a, b);
        case 3: return cmp_str(a, b);
        case 4: return cmp_f32(a, b);
        default: return 0;
    }
}

/* CHW collate weight table: position i (ASCII code i) holds the sort weight.
   UCASE == weight table with a-z mapped onto A-Z (CHW statement).  A custom
   COLLATE string is a 256-byte table built by the program.  NULL = ASCII. */
static unsigned char ucase_w[256];
static int ucase_w_init = 0;
static void ucase_w_build(void) {
    int i;
    for (i = 0; i < 256; i++) ucase_w[i] = (unsigned char)i;
    for (i = 97; i <= 122; i++) ucase_w[i] = (unsigned char)(i - 32);
    ucase_w_init = 1;
}

/* String compare for ARRAY SORT: both elements use the same 1-based char
   window [from..to] (0/absent = whole string, out-of-range = CHR$(0)).
   Optional 256-byte collate weight table.  Returns -1/0/1. */
static int str_cmp_sort(const char* x, const char* y, long long from, long long to,
                        const unsigned char* tbl) {
    if (!x) x = "";
    if (!y) y = "";
    long long xl = (long long)SysStringByteLen(x);
    long long yl = (long long)SysStringByteLen(y);
    long long lo = (from > 0) ? from : 1;
    long long hi = (to > 0) ? to : (xl > yl ? xl : yl);
    long long n = (hi >= lo) ? (hi - lo + 1) : 0;
    long long k;
    for (k = 0; k < n; k++) {
        unsigned char a = (lo - 1 + k < xl) ? (unsigned char)x[lo - 1 + k] : 0;
        unsigned char b = (lo - 1 + k < yl) ? (unsigned char)y[lo - 1 + k] : 0;
        if (tbl) { a = tbl[a]; b = tbl[b]; }
        if (a != b) return (a > b) ? 1 : -1;
    }
    return 0;
}

static void swap_elem(char* a, char* b, int es) {
    char tmp[16];
    memcpy(tmp, a, (size_t)es);
    memcpy(a, b, (size_t)es);
    memcpy(b, tmp, (size_t)es);
}

/* ARRAY SORT arr([index]) [FOR count] [,TAGARRAY t()] [,DESCEND] — CHW range
   sort with optional tag-along array.  index/count are 1-based; count 0/absent
   = index .. end.  Tag array swaps in lockstep with the data array. */
void pb_array_sort2(char* base, int elem_size, long long total, long long index,
                    long long count, int type, int descend,
                    char* tag_base, int tag_elem_size,
                    long long from, long long to, int collate_mode, char* collate,
                    long long cmp_fn) {
    if (!base || total <= 1) return;
    if (index < 1) index = 1;
    if (count <= 0) count = total - index + 1;
    long long end = index + count - 1;
    if (end > total) end = total;
    long long n = end - index + 1;
    if (n <= 1) return;
    const unsigned char* tbl = NULL;
    if (collate_mode == 1) {
        if (!ucase_w_init) ucase_w_build();
        tbl = ucase_w;
    } else if (collate_mode == 2 && collate) {
        tbl = (const unsigned char*)collate;
    }
    /* custom CALL sort: cmp_fn is the address of a user comparison function
       taking two BYREF element pointers and returning -1/0/1 (CHW
       ARRAY_SORT_statement.htm CALL form).  Elements are passed by address
       exactly like a BYREF variable slot, so a fixed-string element (a BSTR
       pointer slot) is passed as its own address. */
    int (*cmp_user)(const void*, const void*) = (int (*)(const void*, const void*))cmp_fn;
    long long i, j;
    for (i = 0; i < n - 1; i++) {
        long long best = i;
        for (j = i + 1; j < n; j++) {
            int c;
            if (cmp_fn) {
                c = cmp_user(base + (index - 1 + best) * elem_size,
                             base + (index - 1 + j) * elem_size);
            } else if (type == 3) {
                const char* x = *(char* const*)(base + (index - 1 + best) * elem_size);
                const char* y = *(char* const*)(base + (index - 1 + j) * elem_size);
                c = str_cmp_sort(x, y, from, to, tbl);
            } else {
                c = arr_cmp_e(base + (index - 1 + best) * elem_size,
                              base + (index - 1 + j) * elem_size, type);
            }
            if (descend) c = -c;
            if (c > 0) best = j;
        }
        if (best != i) {
            swap_elem(base + (index - 1 + i) * elem_size,
                      base + (index - 1 + best) * elem_size, elem_size);
            if (tag_base && tag_elem_size > 0) {
                swap_elem(tag_base + (index - 1 + i) * tag_elem_size,
                          tag_base + (index - 1 + best) * tag_elem_size, tag_elem_size);
            }
        }
    }
}

/* compare helper: returns -1/0/1 */
static int cmp_longlong(long long a, long long b) { return (a > b) - (a < b); }

/* apply relational op (0 =, 1 <>, 2 <, 3 >, 4 <=, 5 >=) to compare result c */
static int apply_op(int c, int op) {
    switch (op) {
        case 0: return c == 0;
        case 1: return c != 0;
        case 2: return c < 0;
        case 3: return c > 0;
        case 4: return c <= 0;
        default: return c >= 0;
    }
}

/* ARRAY SCAN arr([index]) [FOR count], op expr, TO lvar — CHW relative-index
   scan.  Returns 1..count for the first matching element (relative to the
   scan start), 0 = no match.  Numeric arrays compare i64-widened values;
   string arrays compare BSTR contents with optional 1-based FROM..TO window
   (0 = whole string), COLLATE UCASE (case-insensitive) and a custom 256-byte
   COLLATE weight string. */
long long pb_array_scan(char* base, int elem_size, long long total, long long index,
                        long long count, int op, long long val, int is_string,
                        long long from, long long to, int collate_ucase,
                        char* collate) {
    if (!base || total <= 0) return 0;
    if (index < 1) index = 1;
    if (count <= 0) count = total - index + 1;
    long long end = index + count - 1;
    if (end > total) end = total;
    const unsigned char* tbl = NULL;
    if (collate) {
        tbl = (const unsigned char*)collate;
    } else if (collate_ucase) {
        if (!ucase_w_init) ucase_w_build();
        tbl = ucase_w;
    }
    long long i;
    for (i = index; i <= end; i++) {
        char* p = base + (i - 1) * elem_size;
        int match;
        if (is_string) {
            char* s = *(char**)p;
            if (!s) s = "";
            const char* t = (const char*)val;
            if (!t) t = "";
            long long sl = (long long)SysStringByteLen(s);
            long long tl = (long long)SysStringByteLen(t);
            long long lo = (from > 0) ? from : 1;
            long long hi = (to > 0) ? to : sl;
            if (lo < 1) lo = 1;
            if (hi > sl) hi = sl;
            long long wl = (hi >= lo) ? (hi - lo + 1) : 0;
            int c = 0;
            long long n = (wl < tl) ? wl : tl;
            long long k;
            for (k = 0; k < n; k++) {
                unsigned char a = (unsigned char)s[lo - 1 + k];
                unsigned char b = (unsigned char)t[k];
                if (tbl) { a = tbl[a]; b = tbl[b]; }
                if (a != b) { c = (a > b) ? 1 : -1; break; }
            }
            if (c == 0) {
                if (wl > tl) c = 1;
                else if (wl < tl) c = -1;
            }
            match = apply_op(c, op);
        } else {
            long long e;
            if (elem_size == 1) e = *(signed char*)p;
            else if (elem_size == 2) e = *(short*)p;
            else if (elem_size == 4) e = *(int*)p;
            else e = *(long long*)p;
            match = apply_op(cmp_longlong(e, val), op);
        }
        if (match) return i - index + 1;
    }
    return 0;
}

/* PBWin-style string compare for = / <> / < / > / <= / >= (batch 283b).
   strcmp truncates at the first NUL, so binary strings (every MK*$ result,
   any BSTR with embedded NUL) compared equal wrongly.  Compare by the BSTR
   byte-prefixed length (Windows) or strlen (elsewhere): memcmp over the
   shorter length, then length decides.  Returns -1/0/1 like strcmp. */
int pb_str_cmp(const char* a, const char* b) {
    if (!a && !b) return 0;
    if (!a) return -1;
    if (!b) return 1;
#ifdef _WIN32
    unsigned int la = (unsigned int)SysStringByteLen(a);
    unsigned int lb = (unsigned int)SysStringByteLen(b);
#else
    unsigned int la = (unsigned int)strlen(a);
    unsigned int lb = (unsigned int)strlen(b);
#endif
    unsigned int n = la < lb ? la : lb;
    if (n) {
        int c = memcmp(a, b, n);
        if (c) return c < 0 ? -1 : 1;
    }
    return la == lb ? 0 : (la < lb ? -1 : 1);
}

/* MAX$/MIN$ (MAX_function.htm / MIN_function.htm): return the lexicographically
   larger (is_max != 0) or smaller string.  The compiler folds the variadic
   argument list pairwise; this picks one of the two input pointers, so no
   allocation is needed. */
const char* pb_minmax_str(const char* a, const char* b, long long is_max) {
    int c = pb_str_cmp(a, b); /* <0: a<b, 0: equal, >0: a>b */
    int pick_a;
    if (is_max) { pick_a = (c >= 0); }
    else        { pick_a = (c <= 0); }
    return pick_a ? a : b;
}

/* ASC(s$ [, pos]) — character code at 1-based pos; negative pos counts from
   the end; null string / pos 0 / out-of-range returns -1 (CHW + PBWin10). */
long long pb_asc_get(char* s, long long pos) {
    if (!s) return -1;
    /* batch283e probe: ASC must index a BSTR by its byte-prefixed length, NOT
       strlen - MK*$ / binary strings contain NUL bytes (strlen truncates and
       every byte after the first NUL reads as out-of-range -> -1). */
#ifdef _WIN32
    long long len = (long long)SysStringByteLen(s);
#else
    long long len = (long long)strlen(s);
#endif
    if (len == 0) return -1;
    if (pos < 0) pos = len + 1 + pos;
    if (pos < 1 || pos > len) return -1;
    return (long long)(unsigned char)s[pos - 1];
}

/* ===== WSTRING (UTF-16 BSTR) helpers - batch 281 =====
   PBWin semantics measured by probe (batch281_wstr_probe / probe2):
   - LEN(wstr) = UTF-16 code-unit count (BSTR byte prefix / 2)
   - ASC(wstr, n) = UTF-16 code at 1-based n (0..65535); negative counts
     from the end; null / 0 / out-of-range -> -1 (same shape as pb_asc_get)
   - INSTR over a wide haystack/needle searches by code unit, 1-based
   - Mixed STRING+WSTRING concatenation promotes to WSTRING. */

#define PB_CP_ACP 0

int pb_wstr_len(const char* s) {
    if (!s) return 0;
#ifdef _WIN32
    return (int)((unsigned long)SysStringByteLen(s) / 2);
#else
    { const unsigned char* p = (const unsigned char*)s; int n = 0;
      while (p[0] || p[1]) { n++; p += 2; } return n; }
#endif
}

long long pb_wstr_asc(char* s, long long pos) {
    if (!s) return -1;
    long long len = (long long)pb_wstr_len(s);
    if (len == 0) return -1;
    if (pos < 0) pos = len + 1 + pos;
    if (pos < 1 || pos > len) return -1;
    const unsigned char* p = (const unsigned char*)s + (long long)(pos - 1) * 2;
    return (long long)((unsigned short)((unsigned short)p[0] | ((unsigned short)p[1] << 8)));
}

/* ANSI BSTR payload -> UTF-16 BSTR (freshly BSTR-alloc'd). */
char* pb_ansi_to_wide(const char* ansi) {
    static const char empty[] = "";
    if (!ansi) ansi = empty;
#ifdef _WIN32
    unsigned int alen = (unsigned int)SysStringByteLen(ansi);
    int wlen = MultiByteToWideChar(PB_CP_ACP, 0, ansi, (int)alen, NULL, 0);
    if (wlen < 0) wlen = 0;
    char* tmp = (char*)malloc((size_t)wlen * 2 + 2);
    if (wlen > 0) MultiByteToWideChar(PB_CP_ACP, 0, ansi, (int)alen, (short*)tmp, wlen);
    char* result = pb_bstr_alloc(tmp, (unsigned int)(wlen * 2));
    free(tmp);
    return result;
#else
    { size_t n = strlen(ansi); size_t i;
      char* tmp = (char*)malloc(n * 2 + 2);
      for (i = 0; i < n; i++) { tmp[i*2] = ansi[i]; tmp[i*2+1] = 0; }
      tmp[n*2] = 0; tmp[n*2+1] = 0;
      char* result = pb_bstr_alloc(tmp, (unsigned int)(n * 2));
      free(tmp); return result; }
#endif
}

/* INSTR over UTF-16 payloads: 1-based char position, 0 = not found.
   start>=1 scans left->right; start<=-1 scans right->left and the returned
   position is still absolute (left-to-right, CHW INSTR function). */
long long pb_wide_instr(const char* main_s, const char* match_s, long long start) {
    static const char empty[] = "";
    if (!main_s) main_s = empty;
    if (!match_s) match_s = empty;
    long long mlen = (long long)pb_wstr_len(main_s);
    long long nlen = (long long)pb_wstr_len(match_s);
    if (start == 0 || mlen == 0) return 0;
    if (nlen == 0) {
        if (start > 0) return start;
        if (start < 0) return mlen + start + 1;
        return 1;
    }
    long long from;
    int reverse = 0;
    if (start < 0) { reverse = 1; from = mlen + start; }
    else { from = start - 1; }
    if (from < 0) from = 0;
    if (reverse) {
        for (long long i = from; i >= 0; i--) {
            if (i + nlen > mlen) continue;
            long long j;
            for (j = 0; j < nlen; j++) {
                unsigned short a = (unsigned short)((unsigned char)main_s[(i+j)*2] | ((unsigned short)(unsigned char)main_s[(i+j)*2+1] << 8));
                unsigned short b = (unsigned short)((unsigned char)match_s[j*2] | ((unsigned short)(unsigned char)match_s[j*2+1] << 8));
                if (a != b) break;
            }
            if (j == nlen) return i + 1;
        }
        return 0;
    }
    for (long long i = from; i + nlen <= mlen; i++) {
        long long j;
        for (j = 0; j < nlen; j++) {
            unsigned short a = (unsigned short)((unsigned char)main_s[(i+j)*2] | ((unsigned short)(unsigned char)main_s[(i+j)*2+1] << 8));
            unsigned short b = (unsigned short)((unsigned char)match_s[j*2] | ((unsigned short)(unsigned char)match_s[j*2+1] << 8));
            if (a != b) break;
        }
        if (j == nlen) return i + 1;
    }
    return 0;
}

/* LEFT$ on a wide payload: first n code units (n<0 -> empty; n>len -> all). */
char* pb_wstr_left(const char* s, long long n) {
    static const char empty[] = "";
    if (!s) s = empty;
    long long len = (long long)pb_wstr_len(s);
    if (n < 0) n = 0;
    if (n > len) n = len;
    return pb_bstr_alloc(s, (unsigned int)(n * 2));
}

/* RIGHT$ on a wide payload: last n code units. */
char* pb_wstr_right(const char* s, long long n) {
    static const char empty[] = "";
    if (!s) s = empty;
    long long len = (long long)pb_wstr_len(s);
    if (n < 0) n = 0;
    if (n > len) n = len;
    return pb_bstr_alloc(s + (len - n) * 2, (unsigned int)(n * 2));
}

/* MID$ on a wide payload (PB semantics): 1-based start; start<0 counts from
   the end; count<0 = len-|count|; out-of-range -> empty; has_count=0 -> rest. */
char* pb_wstr_mid(const char* s, long long start, long long count, int has_count) {
    static const char empty[] = "";
    if (!s) s = empty;
    long long len = (long long)pb_wstr_len(s);
    if (start < 0) start = len + start + 1;
    if (start < 1 || start > len) return pb_bstr_alloc(empty, 0);
    long long n;
    if (has_count) {
        if (count < 0) count = len + count;  /* len - |count| */
        if (count < 0) count = 0;
        long long avail = len - start + 1;
        n = count < avail ? count : avail;
    } else {
        n = len - start + 1;
    }
    if (n <= 0) return pb_bstr_alloc(empty, 0);
    return pb_bstr_alloc(s + (start - 1) * 2, (unsigned int)(n * 2));
}

/* ARRAY ADD: add elements of src array into dst array (in-place), batch 67 */
void pb_array_add(void* dst, const void* src, int elem_size, int is_float, long long total) {
    if (total <= 0) return;
    if (is_float) {
        if (elem_size == 4) {
            float* d = (float*)dst; const float* s = (const float*)src;
            for (long long i = 0; i < total; i++) d[i] += s[i];
        } else {
            double* d = (double*)dst; const double* s = (const double*)src;
            for (long long i = 0; i < total; i++) d[i] += s[i];
        }
    } else {
        switch (elem_size) {
            case 1: { unsigned char* d = (unsigned char*)dst; const unsigned char* s = (const unsigned char*)src; for (long long i = 0; i < total; i++) d[i] += s[i]; break; }
            case 2: { unsigned short* d = (unsigned short*)dst; const unsigned short* s = (const unsigned short*)src; for (long long i = 0; i < total; i++) d[i] += s[i]; break; }
            case 4: { unsigned int* d = (unsigned int*)dst; const unsigned int* s = (const unsigned int*)src; for (long long i = 0; i < total; i++) d[i] += s[i]; break; }
            case 8: { unsigned long long* d = (unsigned long long*)dst; const unsigned long long* s = (const unsigned long long*)src; for (long long i = 0; i < total; i++) d[i] += s[i]; break; }
        }
    }
}
void pb_array_copy(char* dest, char* src, int elem_size, long long total) {
    if (!dest || !src || total <= 0) return;
    memcpy(dest, src, (size_t)(elem_size * total));
}

void pb_array_swap(char* a, char* b, int elem_size, long long total) {
    if (!a || !b || a == b || total <= 0) return;
    size_t bytes = (size_t)(elem_size * total);
    char* tmp = (char*)malloc(bytes);
    if (!tmp) return;
    memcpy(tmp, a, bytes);
    memcpy(a, b, bytes);
    memcpy(b, tmp, bytes);
    free(tmp);
}

long long pb_array_unique(char* base, int elem_size, long long total, int is_string) {
    if (!base || total <= 1) return total;
    long long write = 0;
    for (long long i = 0; i < total; i++) {
        int dup = 0;
        for (long long j = 0; j < write; j++) {
            int eq;
            if (is_string) {
                char* a = *(char**)(base + j * elem_size);
                char* b = *(char**)(base + i * elem_size);
                eq = (a && b && strcmp(a, b) == 0) || (!a && !b);
            } else {
                eq = memcmp(base + j * elem_size, base + i * elem_size, elem_size) == 0;
            }
            if (eq) { dup = 1; break; }
        }
        if (!dup) {
            if (write != i) memmove(base + write * elem_size, base + i * elem_size, elem_size);
            write++;
        }
    }
    /* zero the tail � fixed-size array model (PB decrements UBOUND; we cannot resize) */
    for (long long k = write; k < total; k++) {
        if (is_string) {
            *(char**)(base + k * elem_size) = NULL;
        } else {
            memset(base + k * elem_size, 0, elem_size);
        }
    }
    return write;
}

/* LOCK/UNLOCK: lock a byte range of the file (C runtime _locking) */
void pb_lock(int filenum, long long record, long long length) {
    if (filenum >= 1 && filenum < MAX_FILE_HANDLES && file_handles[filenum]) {
        if (record > 0) fseek(file_handles[filenum], (long)(record - 1), SEEK_SET);
        long nb = (length > 0) ? (long)length : 1;
        _locking(_fileno(file_handles[filenum]), _LK_LOCK, nb);
    }
}
void pb_unlock(int filenum, long long record, long long length) {
    if (filenum >= 1 && filenum < MAX_FILE_HANDLES && file_handles[filenum]) {
        if (record > 0) fseek(file_handles[filenum], (long)(record - 1), SEEK_SET);
        long nb = (length > 0) ? (long)length : 1;
        _locking(_fileno(file_handles[filenum]), _LK_UNLCK, nb);
    }
}

/* ===== TCP / UDP sockets (batch 19) ===== */
#ifdef _WIN64
typedef unsigned long long pb_sock_t;   /* SOCKET is 64-bit on x64 */
#else
typedef unsigned int pb_sock_t;         /* SOCKET is 32-bit on x86 (stdcall @N decoration must match) */
#endif
struct pb_sockaddr_in {
    short sin_family;
    unsigned short sin_port;
    unsigned long sin_addr;
    char sin_zero[8];
};

__declspec(dllimport) pb_sock_t __stdcall socket(int af, int type, int protocol);
__declspec(dllimport) int __stdcall connect(pb_sock_t s, const void* name, int namelen);
__declspec(dllimport) int __stdcall bind(pb_sock_t s, const void* name, int namelen);
__declspec(dllimport) int __stdcall listen(pb_sock_t s, int backlog);
__declspec(dllimport) pb_sock_t __stdcall accept(pb_sock_t s, void* addr, int* addrlen);
__declspec(dllimport) int __stdcall send(pb_sock_t s, const char* buf, int len, int flags);
__declspec(dllimport) int __stdcall recv(pb_sock_t s, char* buf, int len, int flags);
__declspec(dllimport) int __stdcall sendto(pb_sock_t s, const char* buf, int len, int flags, const void* to, int tolen);
__declspec(dllimport) int __stdcall recvfrom(pb_sock_t s, char* buf, int len, int flags, void* from, int* fromlen);
__declspec(dllimport) int __stdcall closesocket(pb_sock_t s);
__declspec(dllimport) unsigned short __stdcall htons(unsigned short hostshort);
__declspec(dllimport) unsigned long __stdcall htonl(unsigned long hostlong);
__declspec(dllimport) unsigned long __stdcall ntohl(unsigned long netlong);
__declspec(dllimport) unsigned long __stdcall inet_addr(const char* cp);
__declspec(dllimport) int __stdcall setsockopt(pb_sock_t s, int level, int optname, const char* optval, int optlen);
__declspec(dllimport) int __stdcall ioctlsocket(pb_sock_t s, long cmd, unsigned long* argp);
__declspec(dllimport) int __stdcall WSAGetLastError(void);
__declspec(dllimport) unsigned short __stdcall ntohs(unsigned short netshort);

/* servent layout returned by getservbyname (winsock.h struct servent). Only
   s_port is read, but the full field order is declared so the offset is correct.
   winsock.h swaps s_proto/s_port under _WIN64 for pointer alignment: on 64-bit
   s_proto precedes s_port, on 32-bit the classic order is used. A fixed order
   put s_port at the wrong offset and yielded garbage port numbers. */
struct pb_servent {
    char* s_name;
    char** s_aliases;
#ifdef _WIN64
    char* s_proto;
    short s_port;
#else
    short s_port;
    char* s_proto;
#endif
};
__declspec(dllimport) struct pb_servent* __stdcall getservbyname(const char* name, const char* proto);

#define PB_MAX_SOCK 256
/* FIONREAD: ioctlsocket command returning the number of bytes available to read
   (winsock2.h _IOR('f',127,u_long) = 0x4004667F). */
#define PB_FIONREAD 0x4004667Fu
/* WSAETIMEDOUT: blocking socket send/recv timeout (winsock2.h error 10060). */
#define PB_WSAETIMEDOUT 10060
/* PBWin default TCP timeout is 60000 ms (CHW TCP_OPEN_statement.htm). */
#define PB_TCP_DEFAULT_TIMEOUT 60000L
static pb_sock_t pb_sock[PB_MAX_SOCK];
/* pb_sock_state is defined earlier (file/socket section) so pb_freefile can
   reference it: 0=free 1=tcp 2=udp 3=tcp-server. */
static long pb_sock_timeout[PB_MAX_SOCK];  /* per-socket SEND/RECV timeout in ms */
#define PB_SOCK_INVALID ((pb_sock_t)-1)

/* Apply the SEND/RECV timeout to one socket (SO_RCVTIMEO/SO_SNDTIMEO). A
   non-positive value falls back to the PBWin 60000 ms default. */
static void pb_apply_socket_timeout(pb_sock_t s, long ms) {
    if (ms <= 0) ms = PB_TCP_DEFAULT_TIMEOUT;
    int t = (int)ms;
    setsockopt(s, 0xFFFF, 0x1006, (const char*)&t, 4); /* SO_RCVTIMEO */
    setsockopt(s, 0xFFFF, 0x1005, (const char*)&t, 4); /* SO_SNDTIMEO */
}

/* Map a failed blocking call to the PB error code: a Winsock timeout is PBWin
   Error 24 (Device timeout); every other device failure stays Error 57. */
static int pb_wsa_err_code(void) {
    return (WSAGetLastError() == PB_WSAETIMEDOUT) ? 24 : 57;
}

static unsigned long pb_net_ip(const char* host) {
    unsigned long ip = inet_addr(host);
    if (ip == (unsigned long)-1) {
        pb_hostent* he = gethostbyname(host);
        if (he && he->h_addr_list && he->h_addr_list[0]) {
            memcpy(&ip, he->h_addr_list[0], 4);
        } else {
            return (unsigned long)-1;
        }
    }
    return ip;
}

static void pb_net_fill_addr(struct pb_sockaddr_in* a, unsigned long ip, int port) {
    memset(a, 0, sizeof(*a));
    a->sin_family = 2; /* AF_INET */
    a->sin_port = htons((unsigned short)port);
    /* PB's numeric AT/ipaddr& operand is NETWORK order, exactly like PBWin10:
     * &H0100007F means 127.0.0.1 (bytes 7F 00 00 01 on little-endian memory).
     * Measured on PBWin10 2026-10-15 (SEND numeric is stored as-is; UDP RECV
     * returns the same network-order value).  So we store the value raw - no
     * htonl - to be 100% compatible.  String paths (inet_addr/gethostbyname)
     * already yield network order, so callers pass them through unchanged. */
    a->sin_addr = ip;
}

/* Resolve a TCP service-name string (srvc$) to a numeric port number. CHW TCP
   OPEN accepts a numeric port expression or a service-name string. Measured on
   PBWin10: a string port is ALWAYS resolved via getservbyname, with no
   numeric-string atoi - "48731" is not a registered service name and fails.
   The returned s_port is network order and must be converted. Returns -1 and
   sets Error 57 when the string is empty or the name is not found. */
int pb_resolve_service(const char* name) {
    if (pb_winsock_init() != 0) { pb_err_var = pb_err = 57; return -1; }
    if (!name) { pb_err_var = pb_err = 57; return -1; }
    struct pb_servent* se = getservbyname(name, "tcp");
    if (!se) { pb_err_var = pb_err = 57; return -1; }
    return (int)ntohs((unsigned short)se->s_port);
}

int pb_tcp_open(int mode, int port, const char* addr, int filenum, long timeout, unsigned long bind_addr) {
    if (pb_winsock_init() != 0) { pb_err_var = pb_err = 57; return -1; }
    if (filenum < 1 || filenum >= PB_MAX_SOCK) { pb_err_var = pb_err = 52; return -1; }
    if (pb_sock_state[filenum] != 0) { pb_err_var = pb_err = 52; return -1; }
    /* A service-name string is resolved by pb_resolve_service before this call;
       it returns -1 and sets Error 57 when the name is unknown. Reject the
       invalid port here so no socket is created (PBWin leaves the file closed on
       a bad service name), instead of binding/connecting to htons(-1). */
    if (port < 0 || port > 65535) { pb_err_var = pb_err = 57; return -1; }
    if (timeout <= 0) timeout = PB_TCP_DEFAULT_TIMEOUT; /* CHW default 60s */
    pb_sock_t s = socket(2, 1, 6); /* AF_INET, SOCK_STREAM, TCP */
    if (s == PB_SOCK_INVALID) { pb_err_var = pb_err = 57; return -1; }
    struct pb_sockaddr_in a;
    if (mode == 1) {
        /* server: bind + listen.  bind_addr is the numeric network-order address
           (PBWin ADDR semantics: &H0100007F = 127.0.0.1); 0 = INADDR_ANY. */
        pb_net_fill_addr(&a, bind_addr, port);
        if (bind(s, &a, sizeof(a)) != 0 || listen(s, 8) != 0) { closesocket(s); pb_err_var = pb_err = 57; return -1; }
        pb_sock[filenum] = s;
        pb_sock_state[filenum] = 3;
        pb_sock_timeout[filenum] = timeout;
        return 0;
    }
    /* connect */
    unsigned long ip = pb_net_ip(addr);
    if (ip == (unsigned long)-1) { closesocket(s); pb_err_var = pb_err = 57; return -1; }
    pb_apply_socket_timeout(s, timeout);
    pb_net_fill_addr(&a, ip, port);   /* pb_net_ip returns network order; fill_addr stores raw */
    if (connect(s, &a, sizeof(a)) != 0) {
        int e = pb_wsa_err_code();
        closesocket(s); pb_err_var = pb_err = e; return -1;
    }
    pb_sock[filenum] = s;
    pb_sock_state[filenum] = 1;
    pb_sock_timeout[filenum] = timeout;
    return 0;
}

int pb_tcp_accept(int srv, int* out_newf) {
    /* batch205: `TCP ACCEPT #srv AS newf` - newf is an OUTPUT variable whose
       value is 0 before the call, so it must not be validated as an input
       file number.  Accept, then allocate the next free file number and write
       it back (PBWin semantics: the runtime assigns the file number). */
    if (srv < 1 || srv >= PB_MAX_SOCK || pb_sock_state[srv] != 3) { pb_err_var = pb_err = 52; return -1; }
    struct pb_sockaddr_in a;
    int alen = sizeof(a);
    pb_sock_t s = accept(pb_sock[srv], &a, &alen);
    if (s == PB_SOCK_INVALID) { pb_err_var = pb_err = 57; return -1; }
    int f = 1;
    while (f < PB_MAX_SOCK && pb_sock_state[f] != 0) f++;
    if (f >= PB_MAX_SOCK) { closesocket(s); pb_err_var = pb_err = 52; return -1; }
    pb_sock[f] = s;
    pb_sock_state[f] = 1;
    /* Inherit the listening server's SEND/RECV timeout so the accepted socket
       also times out (CHW TIMEOUT applies to SEND/RECV/PRINT/LINE). */
    long t = pb_sock_timeout[srv];
    pb_sock_timeout[f] = t;
    pb_apply_socket_timeout(s, t);
    if (out_newf) *out_newf = f;
    return 0;
}

int pb_tcp_send(int f, const char* data, int len) {
    if (f < 1 || f >= PB_MAX_SOCK || (pb_sock_state[f] != 1 && pb_sock_state[f] != 3)) { pb_err_var = pb_err = 52; return -1; }
    if (!data) { data = ""; len = 0; }
    if (len < 0) len = 0;
    int off = 0;
    while (off < len) {
        int n = send(pb_sock[f], data + off, len - off, 0);
        if (n <= 0) { pb_err_var = pb_err = pb_wsa_err_code(); return -1; }
        off += n;
    }
    return 0;
}

int pb_tcp_recv(int f, long count, char** out) {
    if (f < 1 || f >= PB_MAX_SOCK || (pb_sock_state[f] != 1 && pb_sock_state[f] != 3)) { pb_err_var = pb_err = 52; return -1; }
    if (count < 0) count = 0;
    /* Receive into a scratch buffer, then build a BSTR of the ACTUAL byte
       count. A BSTR preallocated at the requested count would report LEN=count
       even on a timeout or partial read. CHW: the result string length equals
       the bytes actually received; a timeout returns an empty string. */
    char* tmp = (char*)malloc((size_t)count + 1);
    if (!tmp) { pb_err_var = pb_err = 7; return -1; }
    int n = recv(pb_sock[f], tmp, (int)count, 0);
    if (n == 0) {
        /* graceful close: empty result, no error. */
        n = 0;
    } else if (n < 0) {
        pb_err_var = pb_err = pb_wsa_err_code(); /* timeout=24, other=57 */
        n = 0;
    }
    char* result = pb_bstr_alloc(n > 0 ? tmp : NULL, (unsigned int)n);
    free(tmp);
    *out = result;
    return n;
}

int pb_tcp_line_input(int f, char** out) {
    if (f < 1 || f >= PB_MAX_SOCK || (pb_sock_state[f] != 1 && pb_sock_state[f] != 3)) {
        pb_err_var = pb_err = 52;
        /* Measured on PBWin10: a failed TCP LINE INPUT clears the target string
           instead of leaving its previous value in place. */
        if (out) *out = pb_bstr_alloc(NULL, 0);
        return -1;
    }
    /* Dynamic buffer: the old fixed 4096-byte stack buffer silently truncated
       lines longer than 4095 bytes. Grow on demand so a line of any length is
       returned in full; the blocking byte-at-a-time recv, timeout/error and
       CRLF handling below are unchanged. */
    long cap = 256;
    char* tmp = (char*)malloc((size_t)cap);
    if (!tmp) { pb_err_var = pb_err = 7; return -1; }
    int n = 0;
    for (;;) {
        char c = 0;
        int r = recv(pb_sock[f], &c, 1, 0);
        if (r == 0) break;                 /* graceful close: no error */
        if (r < 0) {                       /* timeout=24, other device error=57 */
            pb_err_var = pb_err = pb_wsa_err_code();
            break;
        }
        if (c == '\n') break;
        if (c == '\r') continue;
        if (n == cap) {
            long ncap = cap * 2;
            char* nb = (char*)realloc(tmp, (size_t)ncap);
            if (!nb) { free(tmp); pb_err_var = pb_err = 7; return -1; }
            tmp = nb; cap = ncap;
        }
        tmp[n++] = c;
    }
    char* buf = pb_bstr_alloc(NULL, (unsigned int)n);
    memcpy(buf, tmp, n);
    buf[n] = 0;
    free(tmp);
    *out = buf;
    return n;
}

int pb_tcp_print(int f, const char* data, int newline) {
    if (!data) data = "";
    int dlen = pb_str_len(data);            /* BSTR byte length, NUL-safe */
    if (pb_tcp_send(f, data, dlen) != 0) return -1; /* error code already set */
    if (newline) return pb_tcp_send(f, "\r\n", 2);
    return 0;
}

int pb_tcp_close(int f) {
    if (f < 1 || f >= PB_MAX_SOCK || pb_sock_state[f] == 0) { pb_err_var = pb_err = 52; return -1; }
    closesocket(pb_sock[f]);
    pb_sock_state[f] = 0;
    pb_sock_timeout[f] = 0;
    return 0;
}

/* EOF state for a TCP/UDP socket. PBWin EOF on a socket is a live, non-blocking
   probe: ioctlsocket FIONREAD reports the bytes currently available. >0 -> 0
   (FALSE), 0 -> -1 (TRUE). Returns -2 when filenum is not an open socket so
   pb_eof can fall through to the file check. */
int pb_socket_eof_state(int filenum) {
    if (filenum < 1 || filenum >= PB_MAX_SOCK || pb_sock_state[filenum] == 0) return -2;
    unsigned long avail = 0;
    if (ioctlsocket(pb_sock[filenum], PB_FIONREAD, &avail) != 0) return -1;
    return avail > 0 ? 0 : -1;
}

int pb_udp_open(int port, int filenum, long timeout) {
    if (pb_winsock_init() != 0) { pb_err_var = pb_err = 57; return -1; }
    if (filenum < 1 || filenum >= PB_MAX_SOCK) { pb_err_var = pb_err = 52; return -1; }
    if (pb_sock_state[filenum] != 0) { pb_err_var = pb_err = 52; return -1; }
    pb_sock_t s = socket(2, 2, 17); /* AF_INET, SOCK_DGRAM, UDP */
    if (s == PB_SOCK_INVALID) { pb_err_var = pb_err = 57; return -1; }
    struct pb_sockaddr_in a;
    pb_net_fill_addr(&a, 0, port);
    // CHW: default UDP timeout is 60000 ms; guard against a non-positive value.
    if (timeout <= 0) timeout = 60000;
    {
        int t = (int)timeout;
        setsockopt(s, 0xFFFF, 0x1006, (const char*)&t, 4);
    }
    if (bind(s, &a, sizeof(a)) != 0) { closesocket(s); pb_err_var = pb_err = 57; return -1; }
    pb_sock[filenum] = s;
    pb_sock_state[filenum] = 2;
    return 0;
}

int pb_udp_send(int f, unsigned long ip, int port, const char* data, int len);

int pb_udp_send_str(int f, const char* ipstr, int port, const char* data, int len) {
    /* inet_addr yields network order, same as PBWin's ipaddr& semantics;
     * pb_udp_send expects network order, so pass through unchanged. */
    return pb_udp_send(f, inet_addr(ipstr ? ipstr : "0.0.0.0"), port, data, len);
}

int pb_udp_send(int f, unsigned long ip, int port, const char* data, int len) {
    if (f < 1 || f >= PB_MAX_SOCK || pb_sock_state[f] != 2) { pb_err_var = pb_err = 52; return -1; }
    if (!data) { data = ""; len = 0; }
    if (len < 0) len = 0;
    struct pb_sockaddr_in a;
    pb_net_fill_addr(&a, ip, port);
    int n = sendto(pb_sock[f], data, len, 0, &a, sizeof(a));
    if (n < 0) { pb_err_var = pb_err = pb_wsa_err_code(); return -1; } return 0;
}


static unsigned short ntohs_s(unsigned short v) {
    return (unsigned short)((v >> 8) | (v << 8));
}

int pb_udp_recv(int f, unsigned long* ip, int* port, char** out) {
    if (f < 1 || f >= PB_MAX_SOCK || pb_sock_state[f] != 2) { pb_err_var = pb_err = 52; return -1; }
    char tmp[65536];
    struct pb_sockaddr_in a;
    int alen = sizeof(a);
    int n = recvfrom(pb_sock[f], tmp, 65535, 0, &a, &alen);
    if (n <= 0) {
        // CHW: a timed-out/failed UDP RECV returns an empty string and sets
        // ERR (24 device timeout, otherwise 57).
        pb_err_var = pb_err = pb_wsa_err_code();
        n = 0;
    }
    char* buf = pb_bstr_alloc(NULL, (unsigned int)n);
    memcpy(buf, tmp, n);
    buf[n] = 0;
    if (ip) *ip = a.sin_addr;   /* network order back, same as PBWin10 (measured &H0100007F for 127.0.0.1) */
    if (port) *port = ntohs_s(a.sin_port);
    *out = buf;
    return n;
}

int pb_udp_close(int f) {
    if (f < 1 || f >= PB_MAX_SOCK || pb_sock_state[f] == 0) { pb_err_var = pb_err = 52; return -1; }
    closesocket(pb_sock[f]);
    pb_sock_state[f] = 0;
    return 0;
}



/* ============================================================
 * COMM serial port statements (batch 21)
 * ============================================================ */
typedef struct _pb_dcb {
    unsigned long DCBlength;
    unsigned long BaudRate;
    unsigned long fBits;
    unsigned short wReserved;
    unsigned short XonLim;
    unsigned short XoffLim;
    unsigned char ByteSize;
    unsigned char Parity;
    unsigned char StopBits;
    char XonChar;
    char XoffChar;
    char ErrorChar;
    char EofChar;
    char EvtChar;
    unsigned short wReserved1;
} pb_dcb;

typedef struct _pb_commtimeouts {
    unsigned long ReadIntervalTimeout;
    unsigned long ReadTotalTimeoutMultiplier;
    unsigned long ReadTotalTimeoutConstant;
    unsigned long WriteTotalTimeoutMultiplier;
    unsigned long WriteTotalTimeoutConstant;
} pb_commtimeouts;

#define PB_MAXDWORD 0xFFFFFFFFUL
#define PB_SETDTR 5
#define PB_CLRDTR 6
#define PB_SETRTS 3
#define PB_CLRRTS 4
#define PB_SETBREAK 8
#define PB_CLRBREAK 9
#define PB_GENERIC_READ_WRITE 0xC0000000UL
#define PB_OPEN_EXISTING 3UL

static HANDLE pb_comm_h[256];
static int pb_comm_ok[256];
static int pb_comm_wide[256];   /* CHR=WIDE mode from COMM OPEN (batch 260) */
static int pb_comm_eof[256];    /* COMM LINE timed out -> EOF(channel) TRUE */

/* DCB fFlags bit masks (batch 260) - the pb_dcb layout already matches the
   real Windows DCB: offset 8 is the fFlags DWORD. */
#define PB_F_BINARY       0x00000001UL
#define PB_F_PARITY       0x00000002UL
#define PB_F_OUTXCTSFLOW  0x00000004UL
#define PB_F_OUTXDSRFLOW  0x00000008UL
#define PB_F_DTRCONTROL   0x00000030UL
#define PB_F_DSRSENSITIVITY 0x00000040UL
#define PB_F_OUTX         0x00000100UL
#define PB_F_INX          0x00000200UL
#define PB_F_ERRORCHAR    0x00000400UL
#define PB_F_NULL         0x00000800UL
#define PB_F_RTSCONTROL   0x00003000UL

/* COMM function keyword -> option string compare helper (uppercased copy) */
static void pb_comm_opt(const char* in, char* out, int cap) {
    int i = 0;
    if (!in) { out[0] = '\0'; return; }
    while (in[i] && i < cap - 1) {
        out[i] = (char)toupper((unsigned char)in[i]);
        i++;
    }
    out[i] = '\0';
}

/* TRUE if `channel` is an open COMM port whose COMM LINE has hit a timeout. */
int pb_comm_is_eof(int channel) {
    if (channel < 0 || channel > 255 || !pb_comm_ok[channel]) return 0;
    return pb_comm_eof[channel];
}

/* Batch 206: COMM failures used to be completely silent.  A port that does not
   exist (or is already in use) gave the program no signal whatsoever - the return
   value was discarded by the compiler and nothing was printed.  Every COMM failure
   now reports here, at most once per channel so that a loop cannot flood the
   console; a successful COMM OPEN re-arms the message for that channel. */
static int pb_comm_reported[256] = {0};

static void pb_comm_report(const char* what, int channel) {
    if (channel < 0 || channel > 255) return;
    if (pb_comm_reported[channel]) return;
    pb_comm_reported[channel] = 1;
    fprintf(stderr, "Runtime error: COMM %s on channel %d - the port is not open "
                    "(COMM OPEN failed or was never called)\n", what, channel);
    fflush(stderr);
}

int pb_comm_open(const char* port, int channel, int baud, const char* parity, int data, int stop, int wide) {
    HANDLE h;
    pb_dcb dcb;
    pb_commtimeouts to;
    if (channel < 0 || channel > 255 || port == NULL) { pb_err_var = pb_err = 5; return -1; }
    /* PBWin probe: OPEN of a channel already in use (COMM or file #) -> ERR 55 */
    if (pb_comm_ok[channel]
        || (channel >= 1 && channel < MAX_FILE_HANDLES && file_handles[channel] != NULL)) {
        pb_err_var = pb_err = 55; return -1;
    }
    h = CreateFileA(port, PB_GENERIC_READ_WRITE, 0, NULL, PB_OPEN_EXISTING, 0, NULL);
    if (h == PB_INVALID_HANDLE) {
        pb_comm_h[channel] = NULL;
        pb_comm_ok[channel] = 0;
        fprintf(stderr, "Runtime error: COMM OPEN \"%s\" failed on channel %d - the port "
                        "does not exist, is already in use, or the name is wrong\n",
                port, channel);
        fflush(stderr);
        pb_comm_reported[channel] = 1;
        pb_err_var = pb_err = 69; return -1;   /* PBWin probe: port open failure -> ERR 69 */
    }
    memset(&dcb, 0, sizeof(dcb));
    dcb.DCBlength = sizeof(dcb);
    GetCommState(h, &dcb);
    dcb.BaudRate = (baud > 0) ? (unsigned long)baud : 9600UL;
    dcb.ByteSize = (unsigned char)((data > 0) ? data : 8);
    dcb.StopBits = (unsigned char)((stop > 1) ? 2 : 1);
    dcb.Parity = (unsigned char)((parity && parity[0] == 'E') ? 2 : ((parity && parity[0] == 'O') ? 1 : 0));
    dcb.fBits = 1; /* fBinary = 1 */
    SetCommState(h, &dcb);
    memset(&to, 0, sizeof(to));
    to.ReadIntervalTimeout = PB_MAXDWORD;
    to.ReadTotalTimeoutMultiplier = 0;
    to.ReadTotalTimeoutConstant = 0;
    SetCommTimeouts(h, &to);
    pb_comm_h[channel] = h;
    pb_comm_ok[channel] = 1;
    pb_comm_wide[channel] = (wide != 0);
    pb_comm_eof[channel] = 0;
    pb_comm_reported[channel] = 0;   /* re-arm: this channel now works */
    return 0;
}

int pb_comm_close(int channel) {
    if (channel < 0 || channel > 255 || !pb_comm_ok[channel]) {
        pb_comm_report("CLOSE", channel);
        pb_err_var = pb_err = 52; return -1;
    }
    CloseHandle(pb_comm_h[channel]);
    pb_comm_h[channel] = NULL;
    pb_comm_ok[channel] = 0;
    pb_comm_wide[channel] = 0;
    pb_comm_eof[channel] = 0;
    return 0;
}

int pb_comm_reset(int channel) {
    pb_dcb dcb;
    if (channel < 0 || channel > 255 || !pb_comm_ok[channel]) {
        pb_comm_report("RESET", channel);
        pb_err_var = pb_err = 52; return -1;
    }
    /* CHW COMM_RESET_statement.htm: switch off ALL flow control on the port. */
    memset(&dcb, 0, sizeof(dcb));
    dcb.DCBlength = sizeof(dcb);
    if (!GetCommState(pb_comm_h[channel], &dcb)) { pb_err_var = pb_err = 57; return -1; }
    dcb.fBits &= ~(PB_F_OUTXCTSFLOW | PB_F_OUTXDSRFLOW | PB_F_DSRSENSITIVITY
                   | PB_F_OUTX | PB_F_INX | PB_F_DTRCONTROL | PB_F_RTSCONTROL);
    if (!SetCommState(pb_comm_h[channel], &dcb)) { pb_err_var = pb_err = 57; return -1; }
    return 0;
}

int pb_comm_get(int channel, const char* option) {
    char opt[16];
    unsigned long ms = 0, errs = 0;
    if (channel < 0 || channel > 255 || !pb_comm_ok[channel] || option == NULL) {
        pb_comm_report("GET", channel);
        pb_err_var = pb_err = 5; return 0;   /* PBWin probe: COMM() on unopened channel -> ERR 5 */
    }
    pb_comm_opt(option, opt, (int)sizeof(opt));
    {
        pb_dcb dcb;
        memset(&dcb, 0, sizeof(dcb));
        dcb.DCBlength = sizeof(dcb);
        GetCommState(pb_comm_h[channel], &dcb);
        if (strcmp(opt, "BAUD") == 0) return (int)dcb.BaudRate;
        if (strcmp(opt, "BYTE") == 0) return dcb.ByteSize;
        if (strcmp(opt, "PARITY") == 0) return (dcb.fBits & PB_F_PARITY) ? -1 : 0;
        if (strcmp(opt, "PARITYTYPE") == 0) return dcb.Parity;
        if (strcmp(opt, "PARITYREPL") == 0) return (dcb.fBits & PB_F_ERRORCHAR) ? -1 : 0;
        if (strcmp(opt, "PARITYCHAR") == 0) return (unsigned char)dcb.ErrorChar;
        if (strcmp(opt, "STOP") == 0) return dcb.StopBits; /* 0=1, 1=1.5, 2=2 (CHW) */
        if (strcmp(opt, "CTSFLOW") == 0) return (dcb.fBits & PB_F_OUTXCTSFLOW) ? -1 : 0;
        if (strcmp(opt, "DSRFLOW") == 0) return (dcb.fBits & PB_F_OUTXDSRFLOW) ? -1 : 0;
        if (strcmp(opt, "DSRSENS") == 0) return (dcb.fBits & PB_F_DSRSENSITIVITY) ? -1 : 0;
        if (strcmp(opt, "DTRFLOW") == 0) return (((dcb.fBits & PB_F_DTRCONTROL) >> 4) == 2) ? -1 : 0;
        if (strcmp(opt, "DTRLINE") == 0) return (((dcb.fBits & PB_F_DTRCONTROL) >> 4) == 1) ? -1 : 0;
        if (strcmp(opt, "RTSFLOW") == 0) return (int)((dcb.fBits & PB_F_RTSCONTROL) >> 12);
        if (strcmp(opt, "XINPFLOW") == 0) return (dcb.fBits & PB_F_INX) ? -1 : 0;
        if (strcmp(opt, "XOUTFLOW") == 0) return (dcb.fBits & PB_F_OUTX) ? -1 : 0;
        if (strcmp(opt, "NULL") == 0) return (dcb.fBits & PB_F_NULL) ? -1 : 0;
        if (strcmp(opt, "BREAK") == 0) return 0; /* not queryable via Win32 API */
    }
    /* modem status lines (READ-ONLY) */
    if (GetCommModemStatus(pb_comm_h[channel], &ms)) {
        if (strcmp(opt, "CD") == 0 || strcmp(opt, "RLSD") == 0) return (ms & 0x80UL) ? -1 : 0; /* MS_RLSD_ON */
        if (strcmp(opt, "CTS") == 0) return (ms & 0x10UL) ? -1 : 0;                            /* MS_CTS_ON */
        if (strcmp(opt, "DSR") == 0) return (ms & 0x20UL) ? -1 : 0;                            /* MS_DSR_ON */
        if (strcmp(opt, "RING") == 0) return (ms & 0x40UL) ? -1 : 0;                           /* MS_RING_ON */
    }
    if (strcmp(opt, "RXQUE") == 0 || strcmp(opt, "TXQUE") == 0) {
        /* COMSTAT: DWORD fCtsHold.. (4 bytes) then cbInQue (offset 4), cbOutQue (offset 8) */
        unsigned char stat[12];
        memset(stat, 0, sizeof(stat));
        if (ClearCommError(pb_comm_h[channel], &errs, stat)) {
            unsigned long inque = *(unsigned long*)(stat + 4);
            unsigned long outque = *(unsigned long*)(stat + 8);
            if (strcmp(opt, "RXQUE") == 0) return (int)inque;
            return (int)outque;
        }
    }
    if (strcmp(opt, "RXBUFFER") == 0 || strcmp(opt, "TXBUFFER") == 0) {
        /* COMMPROP: dwPacketLength(0) dwPacketVersion(4) dwServiceMask(8)
           dwReserved1(12) dwMaxTxQueue(16) dwMaxRxQueue(20) */
        unsigned char cp[28];
        memset(cp, 0, sizeof(cp));
        if (GetCommProperties(pb_comm_h[channel], cp)) {
            unsigned long tx = *(unsigned long*)(cp + 16);
            unsigned long rx = *(unsigned long*)(cp + 20);
            if (strcmp(opt, "RXBUFFER") == 0) return (int)rx;
            return (int)tx;
        }
    }
    pb_err_var = pb_err = 5;
    return 0;
}

int pb_comm_send(int channel, const char* s) {
    unsigned long written = 0;
    /* channel number already used by an OPEN FILE -> Bad file mode (PBWin probe) */
    if (channel >= 0 && channel <= 255 && file_modes[channel] != 0) {
        pb_err_var = pb_err = 54; return -1;
    }
    if (channel < 0 || channel > 255 || !pb_comm_ok[channel] || s == NULL) {
        pb_comm_report("SEND", channel);
        pb_err_var = pb_err = 52; return -1;
    }
    WriteFile(pb_comm_h[channel], s, (unsigned long)strlen(s), &written, NULL);
    FlushFileBuffers(pb_comm_h[channel]);
    return (int)written;
}

int pb_comm_recv(int channel, long long bytes, char** dest, int want_wide) {
    unsigned long got = 0;
    char* buf;
    long long n = (bytes > 0) ? bytes : 1;
    if (channel >= 0 && channel <= 255 && file_modes[channel] != 0) {
        pb_err_var = pb_err = 54; return -1;
    }
    if (channel < 0 || channel > 255 || !pb_comm_ok[channel] || dest == NULL) {
        pb_comm_report("RECV", channel);
        pb_err_var = pb_err = 52; return -1;
    }
    /* CHW COMM_RECV_statement.htm: string_var must match the channel's
       character mode (CHR=ANSI -> STRING, CHR=WIDE -> WSTRING); a mismatch
       is ERR 5 (Illegal function call). */
    if ((pb_comm_wide[channel] ? 1 : 0) != (want_wide ? 1 : 0)) {
        pb_err_var = pb_err = 5; return -1;
    }
    buf = (char*)malloc((size_t)n + 1);
    if (buf == NULL) { pb_err_var = pb_err = 5; return -1; }
    if (!ReadFile(pb_comm_h[channel], buf, (unsigned long)n, &got, NULL)) {
        free(buf);
        pb_err_var = pb_err = 57; return -1;
    }
    buf[got] = '\0';
    {
        char* nb = pb_bstr_alloc(buf, (int)got);
        if (*dest) free(*dest);
        *dest = nb;
    }
    free(buf);
    return (int)got;
}

int pb_comm_line_input(int channel, char** dest) {
    int n = 0, cap = 64, c;
    char* buf;
    if (channel >= 0 && channel <= 255 && file_modes[channel] != 0) {
        pb_err_var = pb_err = 54; return -1;
    }
    if (channel < 0 || channel > 255 || !pb_comm_ok[channel] || dest == NULL) {
        pb_comm_report("LINE INPUT", channel);
        pb_err_var = pb_err = 52; return -1;
    }
    buf = (char*)malloc((size_t)cap);
    if (buf == NULL) { pb_err_var = pb_err = 5; return -1; }
    pb_comm_eof[channel] = 0;
    while (1) {
        unsigned long got = 0;
        char ch;
        if (!ReadFile(pb_comm_h[channel], &ch, 1, &got, NULL) || got == 0) {
            /* timeout (COMM TIMEOUT) or device error -> partial line + EOF TRUE */
            pb_comm_eof[channel] = 1;
            break;
        }
        c = (unsigned char)ch;
        if (c == '\r') continue;
        if (c == '\n') break;
        if (n + 1 >= cap) {
            cap *= 2;
            buf = (char*)realloc(buf, (size_t)cap);
            if (buf == NULL) { pb_err_var = pb_err = 5; return -1; }
        }
        buf[n++] = (char)c;
    }
    buf[n] = '\0';
    {
        char* nb = pb_bstr_alloc(buf, n);
        if (*dest) free(*dest);
        *dest = nb;
    }
    free(buf);
    return n;
}

int pb_comm_print_str(int channel, const char* s) {
    unsigned long written = 0;
    if (channel >= 0 && channel <= 255 && file_modes[channel] != 0) {
        pb_err_var = pb_err = 54; return 0;
    }
    if (channel >= 0 && channel <= 255 && pb_comm_ok[channel] && s != NULL) {
        WriteFile(pb_comm_h[channel], s, (unsigned long)strlen(s), &written, NULL);
        FlushFileBuffers(pb_comm_h[channel]);
        return (int)written;
    }
    pb_comm_report("PRINT", channel);
    pb_err_var = pb_err = 52;   /* PBWin probe: COMM PRINT on unopened channel -> ERR 52 */
    return 0;
}

int pb_comm_print_line(int channel, const char* s) {
    /* COMM PRINT: CR/LF appended unless a trailing ';' suppresses it.
       The count returned includes the CR/LF (CHW COMM_PRINT_statement.htm). */
    int n = 0;
    char crlf[2] = { '\r', '\n' };
    unsigned long written = 0;
    if (channel >= 0 && channel <= 255 && file_modes[channel] != 0) {
        pb_err_var = pb_err = 54; return 0;
    }
    if (channel >= 0 && channel <= 255 && pb_comm_ok[channel] && s != NULL) {
        WriteFile(pb_comm_h[channel], s, (unsigned long)strlen(s), &written, NULL);
        n = (int)written;
        if (WriteFile(pb_comm_h[channel], crlf, 2, &written, NULL)) n += (int)written;
        FlushFileBuffers(pb_comm_h[channel]);
        return n;
    }
    pb_comm_report("PRINT", channel);
    pb_err_var = pb_err = 52;
    return 0;
}

void pb_comm_print_int(int channel, long long v) {
    char buf[32];
    snprintf(buf, sizeof(buf), "%lld", v);
    pb_comm_print_str(channel, buf);
}

void pb_comm_print_dbl(int channel, double v) {
    char buf[64];
    snprintf(buf, sizeof(buf), "%g", v);
    pb_comm_print_str(channel, buf);
}

int pb_comm_set(int channel, const char* option, int on) {
    unsigned long fn;
    char opt[16];
    pb_dcb dcb;
    if (channel < 0 || channel > 255 || !pb_comm_ok[channel] || option == NULL) {
        pb_comm_report("SET", channel);
        pb_err_var = pb_err = 52; return -1;
    }
    pb_comm_opt(option, opt, (int)sizeof(opt));
    /* EscapeCommFunction family: no DCB round-trip needed */
    if (strcmp(opt, "DTR") == 0) fn = on ? PB_SETDTR : PB_CLRDTR;
    else if (strcmp(opt, "RTS") == 0) fn = on ? PB_SETRTS : PB_CLRRTS;
    else if (strcmp(opt, "BREAK") == 0) fn = on ? PB_SETBREAK : PB_CLRBREAK;
    else {
        /* Everything else goes through DCB (or SetupComm for the buffers) */
        memset(&dcb, 0, sizeof(dcb));
        dcb.DCBlength = sizeof(dcb);
        if (!GetCommState(pb_comm_h[channel], &dcb)) { pb_err_var = pb_err = 57; return -1; }
        if (strcmp(opt, "BAUD") == 0) {
            dcb.BaudRate = (unsigned long)(on > 0 ? on : 9600);
        } else if (strcmp(opt, "BYTE") == 0) {
            dcb.ByteSize = (unsigned char)(on > 0 ? on : 8);
        } else if (strcmp(opt, "PARITY") == 0) {
            if (on) dcb.fBits |= PB_F_PARITY; else dcb.fBits &= ~PB_F_PARITY;
        } else if (strcmp(opt, "PARITYTYPE") == 0) {
            dcb.Parity = (unsigned char)((on >= 0 && on <= 4) ? on : 0);
        } else if (strcmp(opt, "PARITYREPL") == 0) {
            if (on) dcb.fBits |= PB_F_ERRORCHAR; else dcb.fBits &= ~PB_F_ERRORCHAR;
        } else if (strcmp(opt, "PARITYCHAR") == 0) {
            dcb.ErrorChar = (char)(on & 0xFF);
        } else if (strcmp(opt, "STOP") == 0) {
            dcb.StopBits = (unsigned char)((on >= 0 && on <= 2) ? on : 0);
        } else if (strcmp(opt, "CTSFLOW") == 0) {
            if (on) dcb.fBits |= PB_F_OUTXCTSFLOW; else dcb.fBits &= ~PB_F_OUTXCTSFLOW;
        } else if (strcmp(opt, "DSRFLOW") == 0) {
            if (on) dcb.fBits |= PB_F_OUTXDSRFLOW; else dcb.fBits &= ~PB_F_OUTXDSRFLOW;
        } else if (strcmp(opt, "DSRSENS") == 0) {
            if (on) dcb.fBits |= PB_F_DSRSENSITIVITY; else dcb.fBits &= ~PB_F_DSRSENSITIVITY;
        } else if (strcmp(opt, "DTRFLOW") == 0) {
            dcb.fBits = (dcb.fBits & ~PB_F_DTRCONTROL) | ((on ? 2UL : 0UL) << 4);
        } else if (strcmp(opt, "DTRLINE") == 0) {
            dcb.fBits = (dcb.fBits & ~PB_F_DTRCONTROL) | ((on ? 1UL : 0UL) << 4);
        } else if (strcmp(opt, "RTSFLOW") == 0) {
            dcb.fBits = (dcb.fBits & ~PB_F_RTSCONTROL) | (((unsigned long)(on & 3)) << 12);
        } else if (strcmp(opt, "XINPFLOW") == 0) {
            if (on) dcb.fBits |= PB_F_INX; else dcb.fBits &= ~PB_F_INX;
        } else if (strcmp(opt, "XOUTFLOW") == 0) {
            if (on) dcb.fBits |= PB_F_OUTX; else dcb.fBits &= ~PB_F_OUTX;
        } else if (strcmp(opt, "NULL") == 0) {
            if (on) dcb.fBits |= PB_F_NULL; else dcb.fBits &= ~PB_F_NULL;
        } else if (strcmp(opt, "TXBUFFER") == 0) {
            return SetupComm(pb_comm_h[channel], 0, (unsigned long)(on > 0 ? on : 0)) ? 0 : -1;
        } else if (strcmp(opt, "RXBUFFER") == 0) {
            return SetupComm(pb_comm_h[channel], (unsigned long)(on > 0 ? on : 0), 0) ? 0 : -1;
        } else {
            pb_err_var = pb_err = 5;
            return -1;
        }
        if (!SetCommState(pb_comm_h[channel], &dcb)) { pb_err_var = pb_err = 57; return -1; }
    }
    return 0;
}

int pb_comm_timeout(int channel, long long ms) {
    pb_commtimeouts to;
    if (channel < 0 || channel > 255 || !pb_comm_ok[channel]) {
        pb_comm_report("TIMEOUT", channel);
        pb_err_var = pb_err = 52; return -1;
    }
    memset(&to, 0, sizeof(to));
    if (ms > 0) {
        to.ReadIntervalTimeout = PB_MAXDWORD;
        to.ReadTotalTimeoutConstant = (unsigned long)ms;
        to.WriteTotalTimeoutConstant = (unsigned long)ms;
    }
    return SetCommTimeouts(pb_comm_h[channel], &to) ? 0 : -1;
}

/* ============================================================
 * THREAD statements (batch 21)
 *   thread id passed to PB code is a slot number (0..255),
 *   real HANDLE is stored in pb_thr[] (same pattern as GLOBALMEM).
 * ============================================================ */
typedef struct _pb_thread_slot {
    HANDLE h;
    unsigned long tid;
    int state; /* 0 free, 1 running, 2 suspended, 3 finished */
} pb_thread_slot;

static pb_thread_slot pb_thr[256];

int pb_thread_create(void* func, unsigned long param,
                     unsigned long stack_size, unsigned long create_flags,
                     unsigned long* out_id) {
    int i;
    unsigned long tid = 0;
    if (func == NULL) { pb_err_var = pb_err = 5; return -1; }
#ifdef _WIN64
    for (i = 0; i < 256; i++) {
        if (!pb_thr[i].h) break;
    }
    if (i >= 256) { pb_err_var = pb_err = 5; return -1; }
    /* CHW THREAD CREATE: the thread function takes one BYVAL LONG/DWORD passed
       by value.  CreateThread hands it over as lpParameter; on x64 the PB
       function reads its single i32 argument from the low 32 bits of RCX. */
    /* dwStackSize is the optional StackSize (0 = main thread size);
       dwCreationFlags carries CREATE_SUSPENDED (0x4) when SUSPEND is used. */
    pb_thr[i].h = CreateThread(NULL, stack_size, (void* (__stdcall*)(void*))func,
                               (void*)(unsigned long long)param, create_flags, &tid);
    if (!pb_thr[i].h) { pb_err_var = pb_err = 7; return -1; }
    pb_thr[i].tid = tid;
    pb_thr[i].state = 1;
    /* CHW: hThread is the real Windows thread handle (a DWORD/LONG).  Real
       handle values are small and fit in 32 bits even on x64. */
    if (out_id) *out_id = (unsigned long)(uintptr_t)pb_thr[i].h;
    return 0;
#else
    /* 32-bit: PB functions use cdecl but CreateThread requires stdcall �
       calling a cdecl function through a stdcall pointer corrupts the stack. */
    (void)param;
    (void)stack_size;
    (void)create_flags;
    if (out_id) *out_id = 0;
    pb_err_var = pb_err = 7; return -1;
#endif
}

static pb_thread_slot* pb_thread_slot_of(unsigned long handle) {
    int i;
    /* THREAD CREATE hands back the real Windows handle, so locate the slot by
       matching that handle value rather than indexing the array. */
    for (i = 0; i < 256; i++) {
        if (pb_thr[i].h &&
            (unsigned long)(uintptr_t)pb_thr[i].h == handle) {
            return &pb_thr[i];
        }
    }
    return NULL;
}

int pb_thread_close(unsigned long id) {
    pb_thread_slot* t = pb_thread_slot_of(id);
    int ok;
    /* CHW THREAD CLOSE: release the handle only; it does NOT stop a running
       thread.  Returns PB TRUE (-1) on success, FALSE (0) on failure. */
    if (t == NULL) return 0;
    ok = CloseHandle(t->h);
    memset(t, 0, sizeof(*t));
    return ok ? -1 : 0;
}

int pb_thread_suspend(unsigned long id) {
    pb_thread_slot* t = pb_thread_slot_of(id);
    unsigned long prev;
    if (t == NULL) return -1;
    /* CHW: lResult is the thread's previous suspend count (-1 on failure). */
    prev = SuspendThread(t->h);
    if (prev == (unsigned long)-1) return -1;
    t->state = 2;
    return (int)prev;
}

int pb_thread_resume(unsigned long id) {
    pb_thread_slot* t = pb_thread_slot_of(id);
    unsigned long prev;
    if (t == NULL) return -1;
    /* CHW: lResult is the thread's previous suspend count (-1 on failure). */
    prev = ResumeThread(t->h);
    if (prev == (unsigned long)-1) return -1;
    t->state = 1;
    return (int)prev;
}

int pb_thread_status(unsigned long id) {
    pb_thread_slot* t = pb_thread_slot_of(id);
    unsigned long code = 0;
    /* CHW THREAD STATUS: failure -> 0; still running -> &H103; once
       terminated (and the handle is open) -> the thread function's return
       value, which GetExitCodeThread reports as the exit code. */
    if (t == NULL) return 0;
    if (!GetExitCodeThread(t->h, &code)) return 0;
    if (code == 0x00000103UL) { /* STILL_ACTIVE */
        return 0x00000103;
    }
    t->state = 3;
    return (int)code;
}

int pb_thread_get_priority(unsigned long id) {
    pb_thread_slot* t = pb_thread_slot_of(id);
    if (t == NULL) return 0;
    return GetThreadPriority(t->h);
}

int pb_thread_set_priority(unsigned long id, int prio) {
    pb_thread_slot* t = pb_thread_slot_of(id);
    if (t == NULL) { pb_err_var = pb_err = 5; return -1; }
    return SetThreadPriority(t->h, prio) ? 0 : -1;
}

/* THREADSAFE procedure guard (CHW THREADSAFE).  The code generator emits one
   global zero-initialized SRWLOCK per THREADSAFE procedure and passes its
   address; exclusive acquisition lets only one thread execute the procedure
   at a time, and the paired release runs on every return path. */
void pb_threadsafe_enter(void* lock) {
    AcquireSRWLockExclusive(lock);
}

void pb_threadsafe_leave(void* lock) {
    ReleaseSRWLockExclusive(lock);
}

/* THREADED array backing storage (CHW THREADED).  Each thread owns an
   independent buffer; the code generator keeps a thread-local pointer slot and
   calls this on DIM to obtain a zero-initialized heap block.  calloc yields
   zeroed numeric elements and NULL BSTR slots for string arrays.  A later
   DIM/REDIM frees the calling thread's previous block first. */
/* D4: THREADED array blocks are additionally registered per thread, so a
   thread exit frees every block the exiting thread allocated instead of
   leaving them for process teardown.  FLS (Fiber Local Storage) callbacks run
   on thread detach, and the head of the per-thread list lives IN the FLS slot
   itself, so the callback never depends on static TLS that the loader may
   already have reclaimed.  FlsAlloc failure falls back to the old behaviour
   (process-exit reclaim): the allocation still succeeds, it just is not
   registered. */
#ifndef PB_THREADED_FLS_DEFS
#define PB_THREADED_FLS_DEFS
#define PB_FLS_OUT_OF_INDEXES 0xFFFFFFFFUL
__declspec(dllimport) unsigned long __stdcall FlsAlloc(void(__stdcall*)(void*));
__declspec(dllimport) int __stdcall FlsSetValue(unsigned long, void*);
__declspec(dllimport) void* __stdcall FlsGetValue(unsigned long);
#endif

typedef struct pb_tls_block_node {
    void* block;
    struct pb_tls_block_node* next;
} pb_tls_block_node;

static unsigned long pb_tls_fls_index = PB_FLS_OUT_OF_INDEXES;
static int pb_tls_fls_state = 0; /* 0 uninitialised, 1 ready, 2 unavailable */

static void __stdcall pb_threaded_fls_callback(void* value) {
    pb_tls_block_node* node = (pb_tls_block_node*)value;
    while (node != NULL) {
        pb_tls_block_node* next = node->next;
        if (node->block != NULL) { free(node->block); }
        free(node);
        node = next;
    }
}

static void pb_threaded_fls_ensure(void) {
    if (pb_tls_fls_state == 0) {
        pb_tls_fls_index = FlsAlloc(pb_threaded_fls_callback);
        pb_tls_fls_state = (pb_tls_fls_index == PB_FLS_OUT_OF_INDEXES) ? 2 : 1;
    }
}

void* pb_threaded_array_alloc(long long nbytes) {
    void* block;
    if (nbytes <= 0) { return NULL; }
    block = calloc((size_t)nbytes, 1);
    if (block == NULL) { return NULL; }
    pb_threaded_fls_ensure();
    if (pb_tls_fls_state == 1) {
        pb_tls_block_node* node = (pb_tls_block_node*)malloc(sizeof(pb_tls_block_node));
        if (node != NULL) {
            node->block = block;
            node->next = (pb_tls_block_node*)FlsGetValue(pb_tls_fls_index);
            FlsSetValue(pb_tls_fls_index, (void*)node);
        }
    }
    return block;
}

void pb_threaded_array_free(void* block) {
    if (block == NULL) { return; }
    if (pb_tls_fls_state == 1) {
        pb_tls_block_node* node = (pb_tls_block_node*)FlsGetValue(pb_tls_fls_index);
        pb_tls_block_node* prev = NULL;
        while (node != NULL) {
            if (node->block == block) {
                if (prev == NULL) {
                    FlsSetValue(pb_tls_fls_index, (void*)node->next);
                } else {
                    prev->next = node->next;
                }
                free(node);
                break;
            }
            prev = node;
            node = node->next;
        }
    }
    free(block);
}

/* ================= Batch 22: LPRINT / TRACE / IMPORT ================= */

/* Win32 constants / imports used below (file uses hand-rolled dllimport, no windows.h) */
#ifndef PB_B22_DEFS
#define PB_B22_DEFS
#define PB_GENERIC_WRITE     0x40000000UL
#define PB_FILE_SHARE_WRITE  0x00000002UL
#define PB_OPEN_ALWAYS       4UL
#define PB_FORMFEED_CHAR     12
typedef void* HMODULE_PB;
typedef void* FARPROC_PB;
__declspec(dllimport) void* __stdcall LoadLibraryA(const char* lpFileName);
__declspec(dllimport) void* __stdcall GetProcAddress(void* hModule, const char* lpProcName);
__declspec(dllimport) int __stdcall FreeLibrary(void* hLibModule);
typedef void* HKEY_PB;
__declspec(dllimport) long __stdcall RegOpenKeyExA(HKEY_PB, const char*, unsigned long, unsigned long, HKEY_PB*);
__declspec(dllimport) long __stdcall RegQueryInfoKeyA(HKEY_PB, char*, unsigned long*, unsigned long*, unsigned long*, unsigned long*, unsigned long*, unsigned long*, unsigned long*, unsigned long*, unsigned long*, void*);
__declspec(dllimport) long __stdcall RegQueryValueExA(HKEY_PB, const char*, unsigned long*, unsigned long*, unsigned char*, unsigned long*);
__declspec(dllimport) long __stdcall RegCloseKey(HKEY_PB);;
__declspec(dllimport) long __stdcall RegEnumKeyExA(HKEY_PB, unsigned long, char*, unsigned long*, unsigned long*, unsigned long*, void*, void*);

/* CLSID$(ProgramID$) - batch259 C batch, CHW CLSID$ function:
   Look up HKCR\<progid>\CLSID default value, parse the GUID string
   "{XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX}" into 16 binary bytes:
   Data1/Data2/Data3 stored little-endian, Data4 raw order.  Returns a
   heap BSTR of exactly 16 payload bytes, or an empty BSTR when the
   ProgID is not found / any error occurs.  CHW: CLSID$ does NOT set
   ERR on lookup failure -- it returns "". */
static int pb_hexv(char c) {
    if (c >= '0' && c <= '9') return c - '0';
    if (c >= 'a' && c <= 'f') return c - 'a' + 10;
    if (c >= 'A' && c <= 'F') return c - 'A' + 10;
    return -1;
}
char* pb_clsid_get(const char* progid) {
    char keybuf[512];
    unsigned char valbuf[128];
    unsigned long type = 1;                 /* REG_SZ */
    unsigned long size = sizeof(valbuf);
    HKEY_PB hkey = 0;
    if (!progid || !progid[0]) return pb_bstr_alloc("", 0);
    {
        int n = 0;
        while (progid[n] && n < (int)sizeof(keybuf) - 8) { keybuf[n] = progid[n]; n++; }
        memcpy(keybuf + n, "\\CLSID", 7);   /* includes trailing NUL */
    }
    if (RegOpenKeyExA((HKEY_PB)0x80000000L, keybuf, 0, 0x20019L, &hkey) != 0)
        return pb_bstr_alloc("", 0);
    if (RegQueryValueExA(hkey, "", 0, &type, valbuf, &size) != 0 || type != 1) {
        RegCloseKey(hkey);
        return pb_bstr_alloc("", 0);
    }
    RegCloseKey(hkey);
    valbuf[127] = '\0';
    {
        unsigned char raw[16];
        int idx = 0, half = 0;
        unsigned char cur = 0;
        const unsigned char* p = valbuf;
        for (; *p && idx < 16; p++) {
            int hv;
            if (*p == '-' || *p == '{' || *p == '}') continue;
            hv = pb_hexv((char)*p);
            if (hv < 0) return pb_bstr_alloc("", 0);
            if (half == 0) { cur = (unsigned char)(hv << 4); half = 1; }
            else { cur |= (unsigned char)hv; raw[idx++] = cur; half = 0; }
        }
        if (idx != 16 || half != 0) return pb_bstr_alloc("", 0);
        {
            /* reorder to GUID binary layout (Data1..3 LE, Data4 raw) */
            char bin[16];
            bin[0] = (char)raw[3]; bin[1] = (char)raw[2];
            bin[2] = (char)raw[1]; bin[3] = (char)raw[0];
            bin[4] = (char)raw[5]; bin[5] = (char)raw[4];
            bin[6] = (char)raw[7]; bin[7] = (char)raw[6];
            memcpy(bin + 8, raw + 8, 8);
            return pb_bstr_alloc(bin, 16);
        }
    }
}

/* PROGID$(ClassID$) - batch292 B batch, CHW PROGID$ function (the complement
   of CLSID$): takes the 16-byte binary class ID (typically a CLSID$ output
   BSTR, or a fixed string / GUID of >= 16 bytes), formats it as
   "{XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX}", looks up HKCR\CLSID\{...}\ProgID
   default value and returns the alphanumeric ProgID string.  Returns an empty
   string (and does NOT set ERR) when the CLSID is not found or any error
   occurs.  The 4-byte BSTR length prefix is used so embedded NULs in the
   16-byte GUID are preserved; inputs shorter than 16 bytes return "". */
char* pb_progid_get(const char* clsid) {
    unsigned char b[16];
    char guid[40], key[80], valbuf[256];
    unsigned long type = 1;                 /* REG_SZ */
    unsigned long size = sizeof(valbuf);
    HKEY_PB hkey = 0, sub = 0;
    if (!clsid) return pb_bstr_alloc("", 0);
    {
        unsigned int n = (unsigned int)((unsigned char)clsid[0] |
                          ((unsigned char)clsid[1] << 8) |
                          ((unsigned char)clsid[2] << 16) |
                          ((unsigned char)clsid[3] << 24));
        if (n < 16) return pb_bstr_alloc("", 0);
    }
    memcpy(b, clsid, 16);
    sprintf(guid, "{%08X-%04X-%04X-%02X%02X-%02X%02X%02X%02X%02X%02X}",
            ((unsigned)b[3] << 24) | ((unsigned)b[2] << 16) | ((unsigned)b[1] << 8) | b[0],
            ((unsigned)b[5] << 8) | b[4],
            ((unsigned)b[7] << 8) | b[6],
            b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15]);
    memcpy(key, "CLSID\\", 6);
    {
        int i = 0;
        while (guid[i] && i < (int)sizeof(key) - 7) { key[6 + i] = guid[i]; i++; }
        key[6 + i] = '\0';
    }
    if (RegOpenKeyExA((HKEY_PB)0x80000000L, key, 0, 0x20019L, &hkey) != 0)
        return pb_bstr_alloc("", 0);
    /* The ProgID lives in a SUBKEY named "ProgID" (its default value), not as
       a named value of the CLSID key — open the subkey, then read "" default. */
    if (RegOpenKeyExA(hkey, "ProgID", 0, 0x20019L, &sub) != 0) {
        RegCloseKey(hkey);
        return pb_bstr_alloc("", 0);
    }
    if (RegQueryValueExA(sub, "", 0, &type, (unsigned char*)valbuf, &size) != 0
        || type != 1) {
        RegCloseKey(sub);
        RegCloseKey(hkey);
        return pb_bstr_alloc("", 0);
    }
    RegCloseKey(sub);
    RegCloseKey(hkey);
    valbuf[255] = '\0';
    return pb_bstr_alloc(valbuf, (unsigned int)strlen(valbuf));
}

/* GUID$() / GUIDTXT$() - CHW GUID_function.htm / GUIDTXT_function.htm
   (batch 299).  GUID$ with no argument (or a null string) returns a NEW
   unique 16-byte string GUID via CoCreateGuid; GUID$(guidtext$) parses a
   38-byte "{XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX}" text form back to the
   same 16 binary bytes.  GUIDTXT$(guid16$) is the complement: 16 bytes ->
   38-byte text.  The 16-byte layout is the COM GUID binary layout
   (Data1..3 little-endian, Data4 raw), identical to the CLSID$/PROGID$
   helpers above. */
__declspec(dllimport) long __stdcall CoCreateGuid(void* pguid);

char* pb_guid_new(void) {
    char g[16];
    if (CoCreateGuid(g) != 0) return pb_bstr_alloc("", 0);
    return pb_bstr_alloc(g, 16);
}

char* pb_guid_fromtxt(const char* txt) {
    unsigned char raw[16];
    int idx = 0, half = 0;
    unsigned char cur = 0;
    const unsigned char* p;
    if (!txt || !txt[0]) return pb_guid_new();   /* null/empty -> new GUID (CHW) */
    p = (const unsigned char*)txt;
    for (; *p && idx < 16; p++) {
        int hv;
        if (*p == '-' || *p == '{' || *p == '}') continue;
        hv = pb_hexv((char)*p);
        if (hv < 0) return pb_bstr_alloc("", 0);
        if (half == 0) { cur = (unsigned char)(hv << 4); half = 1; }
        else { cur |= (unsigned char)hv; raw[idx++] = cur; half = 0; }
    }
    if (idx != 16 || half != 0) return pb_bstr_alloc("", 0);
    {
        /* reorder to GUID binary layout (Data1..3 LE, Data4 raw) */
        char bin[16];
        bin[0] = (char)raw[3]; bin[1] = (char)raw[2];
        bin[2] = (char)raw[1]; bin[3] = (char)raw[0];
        bin[4] = (char)raw[5]; bin[5] = (char)raw[4];
        bin[6] = (char)raw[7]; bin[7] = (char)raw[6];
        memcpy(bin + 8, raw + 8, 8);
        return pb_bstr_alloc(bin, 16);
    }
}

char* pb_guid_txt(const char* guid16) {
    unsigned char b[16];
    char guid[40];
    if (!guid16) return pb_bstr_alloc("", 0);
    {
        unsigned int n = (unsigned int)((unsigned char)guid16[0] |
                          ((unsigned char)guid16[1] << 8) |
                          ((unsigned char)guid16[2] << 16) |
                          ((unsigned char)guid16[3] << 24));
        if (n < 16) return pb_bstr_alloc("", 0);
    }
    memcpy(b, guid16, 16);
    sprintf(guid, "{%08X-%04X-%04X-%02X%02X-%02X%02X%02X%02X%02X%02X}",
            ((unsigned)b[3] << 24) | ((unsigned)b[2] << 16) | ((unsigned)b[1] << 8) | b[0],
            ((unsigned)b[5] << 8) | b[4],
            ((unsigned)b[7] << 8) | b[6],
            b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15]);
    return pb_bstr_alloc(guid, 38);
}

/* VARIANT$(vrnt) / VARIANT$$(vrnt) - CHW VARIANT_function1.htm (batch 299):
   extract the dynamic string (VT_BSTR, code 8) held in a 16-byte PB Variant
   variable.  VARIANT$ returns the ANSI (narrow) form, VARIANT$$ the WSTRING
   (UTF-16 BSTR) form — both share the BSTR value union slot at +8.  Any other
   VT type returns an empty string (ANSI "") / null wide pointer. */
char* pb_variant_str(const void* v) {
    unsigned short vt;
    const char* s;
    if (!v) return pb_bstr_alloc("", 0);
    vt = (unsigned short)(((const unsigned char*)v)[0] | (((const unsigned char*)v)[1] << 8));
    if (vt != 8) return pb_bstr_alloc("", 0);
    memcpy(&s, (const char*)v + 8, sizeof(s));
    if (!s) return pb_bstr_alloc("", 0);
    /* The fork's Variant stores the ANSI string pointer in the +8 union slot
       (batch293 emit_variant_store: IrType::Ptr -> VT_BSTR).  The pointer still
       points at the fork's internal BSTR layout (4-byte length prefix + ANSI
       payload), so the byte length is read from that prefix (NUL-safe) rather
       than strlen, which would stop at the first embedded NUL. */
    return pb_bstr_alloc(s, (unsigned int)pb_str_len(s));
}

/* VARIANT$$ returns the WSTRING form: the fork's WSTRING variables are UTF-16
   BSTRs (batch 281), so convert the stored ANSI string to UTF-16 and hand back
   a BSTR that the assignment stores directly.  Any other VT returns NULL. */
const void* pb_variant_wstr(const void* v) {
    unsigned short vt;
    const char* s;
    int nb, need, got;
    wchar_t* w;
    char* out;
    if (!v) return NULL;
    vt = (unsigned short)(((const unsigned char*)v)[0] | (((const unsigned char*)v)[1] << 8));
    if (vt != 8) return NULL;
    memcpy(&s, (const char*)v + 8, sizeof(s));
    if (!s) return NULL;
    nb = pb_str_len(s);                       /* ANSI payload bytes, NUL-safe */
    if (nb <= 0) return NULL;
    need = MultiByteToWideChar(0 /* CP_ACP */, 0, s, nb, NULL, 0);
    if (need <= 0) return NULL;
    w = (wchar_t*)malloc((size_t)need * sizeof(wchar_t));
    if (!w) return NULL;
    got = MultiByteToWideChar(0 /* CP_ACP */, 0, s, nb, w, need);
    if (got <= 0) { free(w); return NULL; }
    out = (char*)pb_bstr_alloc((const char*)w, (unsigned int)got * 2);
    free(w);
    return out;
}

/* VARIANTVT(vrntvar) - CHW VARIANTVT_function.htm: returns the internal VT
   data type code stored in a 16-byte PB Variant variable (equates: 0
   VT_EMPTY, 1 VT_NULL, 2 VT_I2, 3 VT_I4, 5 VT_R8, 8 VT_BSTR, ...).  Layout:
   word VT code at +0, reserved words +2..+7, 8-byte value union at +8. */
long pb_variant_vt(const void* v) {
    if (!v) return 0;
    return ((const unsigned char*)v)[0] | (((const unsigned char*)v)[1] << 8);
}

/* VARIANT#(vrntvar) - CHW VARIANT_function.htm: returns the numeric value
   contained in a Variant, whatever its internal numeric representation
   (BYTE..QUAD / CURRENCY / DOUBLE).  Non-numeric contents (BSTR, objects,
   arrays) and empty/null variants return 0.  The result is a DOUBLE, so the
   caller's destination type performs the final conversion (CHW itself warns
   that QUAD/CURRENCY can exceed DOUBLE precision). */
double pb_variant_num(const void* v) {
    unsigned short vt;
    double d;
    long long i;
    unsigned long long u;
    if (!v) return 0.0;
    vt = (unsigned short)(((const unsigned char*)v)[0] | (((const unsigned char*)v)[1] << 8));
    switch (vt) {
        case 2:  /* VT_I2  */ { short x; memcpy(&x, (const char*)v + 8, 2); return (double)x; }
        case 3:  /* VT_I4  */ { int x; memcpy(&x, (const char*)v + 8, 4); return (double)x; }
        case 4:  /* VT_R4  */ { float x; memcpy(&x, (const char*)v + 8, 4); return (double)x; }
        case 5:  /* VT_R8  */ { double x; memcpy(&x, (const char*)v + 8, 8); return x; }
        case 6:  /* VT_CY  */ { long long x; memcpy(&x, (const char*)v + 8, 8); return (double)x; }
        case 7:  /* VT_DATE*/ { double x; memcpy(&x, (const char*)v + 8, 8); return x; }
        case 11: /* VT_BOOL*/ { short x; memcpy(&x, (const char*)v + 8, 2); return x ? -1.0 : 0.0; }
        case 17: /* VT_UI1 */ { unsigned char x = ((const unsigned char*)v)[8]; return (double)x; }
        case 18: /* VT_UI2 */ { unsigned short x; memcpy(&x, (const char*)v + 8, 2); return (double)x; }
        case 19: /* VT_UI4 */ { unsigned long x; memcpy(&x, (const char*)v + 8, 4); return (double)x; }
        case 20: /* VT_I8  */ { long long x; memcpy(&x, (const char*)v + 8, 8); return (double)x; }
        case 21: /* VT_UI8 */ { unsigned long long x; memcpy(&x, (const char*)v + 8, 8); return (double)x; }
        default: return 0.0;   /* VT_EMPTY / VT_NULL / VT_BSTR / arrays / objects */
    }
}

/* batch304 - ISMISSING optional-parameter support (CHW ISMISSING_function.htm):
   a VARIANT parameter is considered missing when it has a type of %VT_ERROR
   (10) and an error value of %DISP_E_PARAMNOTFOUND (0x80020004).  A NULL
   pointer is treated as missing too (defensive - a caller that skipped a
   BYREF variant argument passes the marker below, never NULL). */
long pb_variant_missing(const void* v) {
    unsigned short vt;
    unsigned int sc;
    if (!v) return -1;
    vt = (unsigned short)(((const unsigned char*)v)[0] | (((const unsigned char*)v)[1] << 8));
    if (vt != 10) return 0;   /* not VT_ERROR */
    memcpy(&sc, (const char*)v + 8, 4);
    return (sc == 0x80020004u) ? -1 : 0;
}

/* Initialise a 16-byte PB Variant container to the "missing" marker:
   VT_ERROR at +0, DISP_E_PARAMNOTFOUND in the +8 union slot.  The call side
   uses this for an omitted optional VARIANT argument. */
void pb_variant_missing_init(void* v) {
    unsigned char* b = (unsigned char*)v;
    unsigned int sc = 0x80020004u;
    b[0] = 10; b[1] = 0;              /* VT_ERROR */
    b[2] = 0; b[3] = 0;               /* reserved */
    b[4] = 0; b[5] = 0; b[6] = 0; b[7] = 0;
    memcpy(b + 8, &sc, 4);            /* scode DISP_E_PARAMNOTFOUND */
    b[12] = 0; b[13] = 0; b[14] = 0; b[15] = 0;
}
__declspec(dllimport) int __stdcall SetConsoleTitleA(const char* lpConsoleTitle);
__declspec(dllimport) unsigned long __stdcall GetConsoleTitleA(char* lpConsoleTitle, unsigned long nSize);
#endif

/* ---- BSTR helpers (4-byte LE length prefix + payload) ---- */
static unsigned int pb_bstr_len(const char* s) {
    if (!s) return 0;
    return (unsigned int)((unsigned char)s[0] | ((unsigned char)s[1] << 8) |
                          ((unsigned char)s[2] << 16) | ((unsigned char)s[3] << 24));
}
static void pb_bstr_cpy(char* dst, size_t cap, const char* s) {
    /* copy payload of BSTR s into dst (capped, NUL-terminated) */
    if (!dst) return;
    if (!s) { dst[0] = '\0'; return; }
    unsigned int n = pb_bstr_len(s);
    if (n >= cap) n = (unsigned int)(cap - 1);
    memcpy(dst, s + 4, n);
    dst[n] = '\0';
}

/* ---- LPRINT: direct line-printer device output ---- */
static HANDLE pb_lpt_handle = NULL;
static int pb_lpt_col = 0; /* current column for comma (14-column) handling */
static char pb_lpt_device[64]; /* name of the attached device, for LPRINT$ */

/* LPRINT ATTACH device$  (open device/file for direct writes; "" or NULL = close) */
int pb_lprint_attach(char* device) {
    if (pb_lpt_handle) { CloseHandle(pb_lpt_handle); pb_lpt_handle = NULL; }
    pb_lpt_col = 0;
    pb_lpt_device[0] = '\0';
    if (!device || strlen(device) == 0) return 0; /* close-only */
    char path[512];
    strncpy(path, device, sizeof(path) - 1);
    path[sizeof(path) - 1] = '\0';
    HANDLE h = CreateFileA(path, PB_GENERIC_WRITE, PB_FILE_SHARE_WRITE, NULL,
                           PB_OPEN_EXISTING, 0, NULL);
    if (h == PB_INVALID_HANDLE) { pb_err_var = pb_err = 68; return -1; }
    pb_lpt_handle = h;
    /* retain the device name for LPRINT$ (CHW: max 32 chars; buffer holds 63) */
    strncpy(pb_lpt_device, device, sizeof(pb_lpt_device) - 1);
    pb_lpt_device[sizeof(pb_lpt_device) - 1] = '\0';
    return 0;
}
/* LPRINT$: BSTR of the attached device name, or an empty BSTR if none */
char* pb_lprint_device(void) {
    return pb_bstr_alloc(pb_lpt_device, (unsigned int)strlen(pb_lpt_device));
}
/* LPRINT CLOSE */
void pb_lprint_close(void) {
    if (pb_lpt_handle) { CloseHandle(pb_lpt_handle); pb_lpt_handle = NULL; }
    pb_lpt_device[0] = '\0';
}
/* LPRINT FLUSH */
void pb_lprint_flush(void) {
    if (pb_lpt_handle) FlushFileBuffers(pb_lpt_handle);
}
/* LPRINT FORMFEED */
void pb_lprint_formfeed(void) {
    if (pb_lpt_handle) {
        char c = 12; DWORD w;
        WriteFile(pb_lpt_handle, &c, 1, &w, NULL);
        pb_lpt_col = 0;
    }
}
/* raw bytes to device */
static void pb_lpt_write(const char* buf, unsigned int len) {
    if (!pb_lpt_handle || !buf) return;
    DWORD w;
    WriteFile(pb_lpt_handle, buf, len, &w, NULL);
    pb_lpt_col += (int)len;
}
/* LPRINT "..." � BSTR payload, no CRLF */
void pb_lprint_bstr(char* s) {
    if (s) pb_lpt_write(s, (unsigned int)strlen(s));
}
void pb_lprint_int(int n) {
    char b[32]; int l = sprintf(b, "%d", n);
    /* PB numeric field: leading space reserves the sign column for non-negatives,
       and a trailing space always follows the number (CHW PRINT/LPRINT semantics). */
    if (n >= 0) pb_lpt_write(" ", 1);
    pb_lpt_write(b, (unsigned int)l);
    pb_lpt_write(" ", 1);
}
void pb_lprint_i64(long long n) {
    char b[32]; int l = sprintf(b, "%lld", n);
    if (n >= 0) pb_lpt_write(" ", 1);
    pb_lpt_write(b, (unsigned int)l);
    pb_lpt_write(" ", 1);
}
void pb_lprint_dbl(double d) {
    /* PBWin LPRINT prints the full 15-significant-digit field (same renderer as
       STR$), including the leading sign space, then a trailing space. */
    char* b = pb_num_render(d, 15, 1);
    if (b) {
        pb_lpt_write(b, (unsigned int)strlen(b));
        pb_lpt_write(" ", 1);
    }
}
/* LPRINT statement terminator: CRLF unless trailing semicolon (simplified: always CRLF) */
void pb_lprint_crlf(void) {
    pb_lpt_write("\r\n", 2);
    pb_lpt_col = 0;
}
/* LPRINT comma: advance to next 14-column position */
void pb_lprint_tab(void) {
    if (!pb_lpt_handle) return;
    int pad = (14 - (pb_lpt_col % 14));
    if (pad == 0) pad = 14;
    char buf[14]; memset(buf, ' ', 14);
    pb_lpt_write(buf, (unsigned int)pad);
}

/* ---- TRACE: explicit trace file logging ---- */
static FILE* pb_trace_fp = NULL;
static int pb_trace_enabled = 0;

/* TRACE NEW fname$ */
int pb_trace_new(char* fname) {
    if (pb_trace_fp) { fclose(pb_trace_fp); pb_trace_fp = NULL; }
    pb_trace_fp = fopen(fname ? fname : "", "w");
    pb_trace_enabled = (pb_trace_fp != NULL);
    /* CHW TRACE: TRACE NEW always emits the banner line, even before TRACE ON.
       Text mode turns the '\n' into CRLF, matching PBWin byte-for-byte. */
    if (pb_trace_fp) {
        fputs("Trace Begins...\n", pb_trace_fp);
        fflush(pb_trace_fp);
    }
    return pb_trace_fp ? 0 : -1;
}
void pb_trace_on(void) { pb_trace_enabled = 1; }
void pb_trace_off(void) { pb_trace_enabled = 0; }
void pb_trace_print(char* s) {
    if (!pb_trace_enabled || !pb_trace_fp || !s) return;
    fputs(s, pb_trace_fp);
    fputc('\n', pb_trace_fp);
    fflush(pb_trace_fp);
}
void pb_trace_close(void) {
    if (pb_trace_fp) { fclose(pb_trace_fp); pb_trace_fp = NULL; }
    pb_trace_enabled = 0;
}

/* ---- IMPORT: explicit DLL loading ---- */
/* IMPORT ADDR ProcName$, LibName$ TO AddrVar& [,HndlVar&] */
int pb_import_addr(char* procname, char* libname, void** out_addr, void** out_hndl) {
    /* procname/libname arrive as C-string payloads. Out slots must be 8-byte
       (PB QUAD variables) because x64 system-DLL addresses exceed 32 bits. */
    char pname[512], lname[1024];
    strncpy(pname, procname ? procname : "", sizeof(pname) - 1);
    pname[sizeof(pname) - 1] = '\0';
    strncpy(lname, libname ? libname : "", sizeof(lname) - 1);
    lname[sizeof(lname) - 1] = '\0';
    void* m = LoadLibraryA(lname);
    if (!m) { pb_err_var = pb_err = 53; return -1; }
    void* fp = GetProcAddress(m, pname);
    if (!fp) { FreeLibrary(m); pb_err_var = pb_err = 53; return -1; }
    if (out_addr) *out_addr = fp;
    if (out_hndl) *out_hndl = m;
    return 0;
}
/* IMPORT CLOSE HndlVar */
void pb_import_close(void* hndl) {
    if (hndl) FreeLibrary(hndl);
}

/* Batch 43: PRINTERCOUNT (placed mid-file next to pb_import_addr � file-tail
   placement of fresh Win32 dllimport calls crashed inside PB-linked exes). */
#define PB_PRINTER_ENUM_LOCAL 0x00000002
#define PB_PRINTER_ENUM_CONNECTIONS 0x00000004
#define PB_PRINTER_INFO_LEVEL2 2
typedef int (__stdcall* PB_EnumPrintersW_t)(unsigned long, const char*, unsigned long, char*, unsigned long, unsigned long*, unsigned long*);
__declspec(dllimport) int __stdcall EnumPrintersW(unsigned long Flags, const char* Name, unsigned long Level, char* pPrinterEnum, unsigned long cbBuf, unsigned long* pcbNeeded, unsigned long* pcReturned);
long long pb_printer_count(void) {
    /* PRINTERCOUNT: count installed printers via the registry (winspool's
       EnumPrintersW crashed inside PB-linked exes for reasons not yet found;
       registry path is stable and PB-exe-safe). */
    HKEY_PB hk = 0;
    unsigned long count = 0;
    if (RegOpenKeyExA((HKEY_PB)0x80000002L, "SYSTEM\\CurrentControlSet\\Control\\Print\\Printers",
                      0, 0x20019L, &hk)) return 0;
    RegQueryInfoKeyA(hk, 0, 0, 0, &count, 0, 0, 0, 0, 0, 0, 0);
    RegCloseKey(hk);
    return (long long)count;
}

/* PRINTER$: PRINTER$([NAME|PORT], printernum&).  Enumerate the printer subkeys
 * under the same registry path pb_printer_count uses (1-based).  NAME/default
 * returns the printer (subkey) name; PORT reads that subkey's "Port" value.
 * An out-of-range index (or any failure) returns an empty BSTR. */
char* pb_printer(const char* director, long long printernum) {
    HKEY_PB hk = 0;
    if (RegOpenKeyExA((HKEY_PB)0x80000002L,
                      "SYSTEM\\CurrentControlSet\\Control\\Print\\Printers",
                      0, 0x20019L, &hk))
        return pb_bstr_alloc("", 0);
    unsigned long total = 0;
    RegQueryInfoKeyA(hk, 0, 0, 0, &total, 0, 0, 0, 0, 0, 0, 0);
    char* result = pb_bstr_alloc("", 0);
    if (printernum >= 1 && (unsigned long long)printernum <= (unsigned long long)total) {
        char namebuf[256];
        unsigned long nlen = sizeof(namebuf);
        if (RegEnumKeyExA(hk, (unsigned long)(printernum - 1), namebuf, &nlen,
                          0, 0, 0, 0) == 0) {
            int want_port = director && strcmp(director, "PORT") == 0;
            if (!want_port) {
                result = pb_bstr_alloc(namebuf, nlen);
            } else {
                HKEY_PB sub = 0;
                char portbuf[256];
                unsigned long plen = sizeof(portbuf);
                unsigned long ptype = 0;
                if (RegOpenKeyExA(hk, namebuf, 0, 0x20019L, &sub) == 0
                    && RegQueryValueExA(sub, "Port", 0, &ptype,
                                        (unsigned char*)portbuf, &plen) == 0) {
                    unsigned long blen = plen;
                    if (blen && portbuf[blen - 1] == 0) blen--;
                    result = pb_bstr_alloc(portbuf, blen);
                }
                if (sub) RegCloseKey(sub);
            }
        }
    }
    RegCloseKey(hk);
    return result;
}

/* ---- Batch 23: WINDOW SET/GET TEXT (console title) + TYPE SET ---- */
void pb_console_set_title(const char* s) {
    SetConsoleTitleA(s ? s : "");
}

char* pb_console_get_title(void) {
    char buf[1024];
    unsigned long n = GetConsoleTitleA(buf, 1024);
    if (n == 0) return pb_bstr_alloc("", 0);
    if (n >= sizeof(buf)) n = sizeof(buf) - 1;
    buf[n] = '\0';
    return pb_bstr_alloc(buf, (unsigned int)n);
}

void pb_type_set(void* dest, const void* src, unsigned int size) {
    if (!dest || !src || size == 0) return;
    memcpy(dest, src, size);
}

void pb_type_set_str(void* dest, const char* src, unsigned int size) {
    if (!dest || size == 0) return;
    unsigned int n = src ? (unsigned int)strlen(src) : 0;
    if (n > size) n = size;
    if (n) memcpy(dest, src, n);
    if (n < size) memset((char*)dest + n, 0, size - n);
}

/* TYPE SET with USING: bytes after the source are filled with the first
   character of the USING string; a NULL/empty pad falls back to binary zeros. */
void pb_type_set_str_pad(void* dest, const char* src, unsigned int size, const char* pad_str) {
    if (!dest || size == 0) return;
    unsigned char pad = (pad_str && pad_str[0]) ? (unsigned char)pad_str[0] : 0;
    unsigned int n = src ? (unsigned int)strlen(src) : 0;
    if (n > size) n = size;
    if (n) memcpy(dest, src, n);
    if (n < size) memset((char*)dest + n, pad, size - n);
}

/* ================= Batch 24: DIR$ / DIR statement (FindFirstFileA family) ================= */

/* WIN32_FIND_DATAA layout (hand-rolled, no windows.h):
   0  DWORD  dwFileAttributes
   4  FILETIME ftCreationTime (8)
   12 FILETIME ftLastAccessTime (8)
   20 FILETIME ftLastWriteTime (8)
   28 DWORD  nFileSizeHigh
   32 DWORD  nFileSizeLow
   36 DWORD  dwReserved0
   40 DWORD  dwReserved1
   44 CHAR   cFileName[260]
   304 CHAR  cAlternateFileName[14]
   total >= 592 bytes
*/
#define PB_FIND_DATA_SIZE 592
#define PB_FILE_ATTRIBUTE_DIRECTORY 0x10UL

#ifndef PB_B24_DEFS
#define PB_B24_DEFS
typedef void* HANDLE_PB24;
#define PB_INVALID_HANDLE ((void*)(intptr_t)-1)
__declspec(dllimport) void* __stdcall FindFirstFileA(const char* lpFileName, char* lpFindFileData);
__declspec(dllimport) int __stdcall FindNextFileA(void* hFindFile, char* lpFindFileData);
__declspec(dllimport) int __stdcall FindClose(void* hFindFile);
#endif

static void* g_dir_handle = PB_INVALID_HANDLE;
static char g_dir_find_data[PB_FIND_DATA_SIZE];
static int g_dir_only = 0;
static unsigned long g_dir_attr = 0;

static const char* g_dir_name(void) { return g_dir_find_data + 44; }
static unsigned long g_dir_attrs(void) {
    unsigned long a;
    memcpy(&a, g_dir_find_data, sizeof(a));
    return a;
}
static int g_dir_is_dot(void) {
    const char* n = g_dir_name();
    return (n[0] == '.' && n[1] == '\0') || (n[0] == '.' && n[1] == '.' && n[2] == '\0');
}
static int g_dir_match(void) {
    if (g_dir_is_dot()) return 0;
    unsigned long fa = g_dir_attrs();
    if (g_dir_only) return (fa & g_dir_attr) == g_dir_attr;
    /* non-ONLY, no attribute: normal files only (not directory/hidden/system/label) */
    if (g_dir_attr == 0) {
        if (fa & (PB_FILE_ATTRIBUTE_DIRECTORY | 0x2UL | 0x4UL | 0x8UL)) return 0;
        return 1;
    }
    /* non-ONLY with attribute: inclusive - normal files plus attribute matches */
    if (!(fa & (PB_FILE_ATTRIBUTE_DIRECTORY | 0x2UL | 0x4UL | 0x8UL))) return 1;
    return (fa & g_dir_attr) == g_dir_attr;
}
static char* g_dir_make_bstr(void) {
    const char* n = g_dir_name();
    return pb_bstr_alloc(n, (unsigned int)strlen(n));
}

/* DIR$ mask [, ONLY attr]  -> first match; returns "" when none */
char* pb_dir_first(char* mask, int only, unsigned long attr) {
    if (g_dir_handle != PB_INVALID_HANDLE) {
        FindClose(g_dir_handle);
        g_dir_handle = PB_INVALID_HANDLE;
    }
    g_dir_only = only;
    g_dir_attr = attr;
    if (!mask || !*mask) return pb_bstr_alloc("", 0);
    g_dir_handle = FindFirstFileA(mask, g_dir_find_data);
    if (g_dir_handle == PB_INVALID_HANDLE) return pb_bstr_alloc("", 0);
    for (;;) {
        if (g_dir_match()) return g_dir_make_bstr();
        if (!FindNextFileA(g_dir_handle, g_dir_find_data)) {
            FindClose(g_dir_handle);
            g_dir_handle = PB_INVALID_HANDLE;
            return pb_bstr_alloc("", 0);
        }
    }
}

/* DIR$ (NEXT) -> next match; "" when exhausted */
char* pb_dir_next(void) {
    if (g_dir_handle == PB_INVALID_HANDLE) return pb_bstr_alloc("", 0);
    for (;;) {
        if (!FindNextFileA(g_dir_handle, g_dir_find_data)) {
            FindClose(g_dir_handle);
            g_dir_handle = PB_INVALID_HANDLE;
            return pb_bstr_alloc("", 0);
        }
        if (g_dir_match()) return g_dir_make_bstr();
    }
}

void pb_dir_close(void) {
    if (g_dir_handle != PB_INVALID_HANDLE) {
        FindClose(g_dir_handle);
        g_dir_handle = PB_INVALID_HANDLE;
    }
}
/* PATHSCAN$(director, filespec$ [, pathspec$]) � find a file on disk and return
   FULL/PATH/NAME/EXTN/NAMEX part (semantics per official PATHSCAN$ function page).
   pathspec$ is a semicolon-separated list of directories to search. */
char* pb_pathscan(const char* director, const char* filespec, const char* pathspec) {
    char buf[1024];
    const char* dirs = (pathspec && pathspec[0]) ? pathspec : "";
    const char* p = dirs;
    for (;;) {
        const char* semi = strchr(p, ';');
        int dlen = semi ? (int)(semi - p) : (int)strlen(p);
        int total = dlen + 1 + (int)strlen(filespec ? filespec : "");
        if (total >= 1024) return pb_bstr_alloc("", 0);
        if (dlen > 0) {
            memcpy(buf, p, dlen);
            buf[dlen] = '\\';
            strcpy(buf + dlen + 1, filespec ? filespec : "");
        } else {
            strcpy(buf, filespec ? filespec : "");
        }
        /* FindFirstFileA: 0xFFFFFFFF = INVALID_HANDLE_VALUE */
        char find_data[592];
        void* h = FindFirstFileA(buf, find_data);
        if (h != PB_INVALID_HANDLE) {
            /* file exists � now resolve the part on the FULL found path */
            char found[1024];
            strncpy(found, buf, 1023); found[1023] = 0;
            /* remove the filespec tail from buf to get the directory, then
               reconstruct: we simply parse buf itself (path + filespec). */
            const char* slash = strrchr(buf, '\\');
            const char* colon = strrchr(buf, ':');
            const char* last = slash > colon ? slash : colon;
            const char* dot = strrchr(buf, '.');
            char out[1024];
            int len = 0;
            const char* dr = director ? director : "FULL";
            if (strcmp(dr, "FULL") == 0) {
                len = (int)strlen(buf); if (len > 1023) len = 1023; memcpy(out, buf, len);
            } else if (strcmp(dr, "PATH") == 0) {
                if (last) { len = (int)(last - buf) + 1; if (len > 1023) len = 1023; memcpy(out, buf, len); }
            } else if (strcmp(dr, "EXTN") == 0) {
                if (dot) { len = (int)strlen(dot); if (len > 1023) len = 1023; memcpy(out, dot, len); }
            } else { /* NAME / NAMEX */
                const char* start = last ? last + 1 : buf;
                int slen = (int)strlen(start);
                if (strcmp(dr, "NAME") == 0 && dot && dot > start) slen = (int)(dot - start);
                len = slen; if (len > 1023) len = 1023;
                memcpy(out, start, len);
            }
            out[len] = 0;
            FindClose(h);
            return pb_bstr_alloc(out, len);
        }
        if (!semi) break;
        p = semi + 1;
    }
    return pb_bstr_alloc("", 0);
}


static void* g_gr_dc = 0;
static void* g_gr_dc_win = 0;   /* set when g_gr_dc came from GetDC(control) */
static void pb_graphic_release_dc(void);
static void pb_gw_touch(void);   /* batch 180 - invalidate a graph window */
static void* pb_pb_hwnd(void* hDlg, long long id);   /* defined with the CONTROL helpers below */
__declspec(dllimport) int __stdcall GetClientRect(void*, void*);
static void* g_gr_bmp = 0;
/* GRAPHIC WIDTH/STYLE/SAVE (batch 54) */
static int g_gr_width = 1;
static int g_gr_style = 0;
/* GRAPHIC SET/GET OVERLAP (batch 181).  Off by default - that is Windows' own
   RECT convention, where the right and bottom edges are exclusive.  When it is
   on the RECT-based drawing statements grow their rectangle by one pixel so the
   coordinates the program wrote are themselves painted. */
static int g_gr_overlap = 0;
int pb_graphic_set_overlap(int on) {
    if (!g_gr_dc) return 0;
    g_gr_overlap = on ? 1 : 0;
    return 1;
}
int pb_graphic_get_overlap(void) {
    return g_gr_overlap;
}

/* ==================== batch 182: ScrollText / ImageList / Render / Stretch ====
   This file deliberately does not include windows.h: every import is declared
   here in the same style the rest of the file uses, with plain C types only. */

#define PB_B182_IMAGE_BITMAP   0u
#define PB_B182_IMAGE_ICON     1u
#define PB_B182_LR_LOADFROMFILE 0x0010u
#define PB_B182_LR_SHARED      0x8000u
#define PB_B182_SRCCOPY        0x00CC0020u

__declspec(dllimport) int __stdcall ImageList_GetImageCount(void* himl);
__declspec(dllimport) int __stdcall ImageList_Draw(void* himl, int i, void* hdc,
                                                   int x, int y, unsigned int f);
__declspec(dllimport) void* __stdcall LoadImageA(void* hinst, const char* name,
                                                 unsigned int type, int cx, int cy,
                                                 unsigned int fu);
__declspec(dllimport) int __stdcall StretchBlt(void* hdcDst, int x, int y, int w, int h,
                                               void* hdcSrc, int sx, int sy, int sw,
                                               int sh, unsigned long rop);
__declspec(dllimport) int __stdcall SetStretchBltMode(void* hdc, int mode);
__declspec(dllimport) void* __stdcall CreateCompatibleDC(void* hdc);
__declspec(dllimport) int __stdcall DrawIconEx(void* hdc, int x, int y, void* hicon,
                                               int cx, int cy, unsigned int istep,
                                               void* hbr, unsigned int fu);
__declspec(dllimport) int __stdcall DeleteDC(void* hdc);
__declspec(dllimport) void* __stdcall SelectObject(void* hdc, void* h);
__declspec(dllimport) int __stdcall DeleteObject(void* h);

static int g_gr_scrolltext = 0;

int pb_graphic_set_scrolltext(int on) {
    if (!g_gr_dc) return 0;
    g_gr_scrolltext = on ? 1 : 0;
    return 1;
}

int pb_graphic_get_scrolltext(void) {
    return g_gr_scrolltext;
}

int pb_graphic_imagelist(int x, int y, long long hl, int index, int overlay, int style) {
    if (!g_gr_dc) return 0;
    if (!hl) return 0;
    int n = ImageList_GetImageCount((void*)(long long)hl);
    if (index < 1 || index > n) return 0;
    if (!ImageList_Draw((void*)(long long)hl, index - 1, g_gr_dc, x, y, (unsigned int)style))
        return 0;
    if (overlay >= 1 && overlay <= n)
        ImageList_Draw((void*)(long long)hl, overlay - 1, g_gr_dc, x, y, (unsigned int)style);
    return 1;
}

/* A period in the name means a disk file; otherwise try the resource first and
   fall back to the disk.  want_icon picks LoadImageA's type. */
int pb_graphic_render_common(char* name, int x1, int y1, int x2, int y2, int want_icon) {
    if (!g_gr_dc || !name) return 0;
    unsigned int type = want_icon ? PB_B182_IMAGE_ICON : PB_B182_IMAGE_BITMAP;
    void* img = 0;
    if (strchr(name, '.')) {
        img = LoadImageA(0, name, type, 0, 0, PB_B182_LR_LOADFROMFILE);
    } else {
        img = LoadImageA(GetModuleHandleA(0), name, type, 0, 0, PB_B182_LR_SHARED);
        if (!img) img = LoadImageA(0, name, type, 0, 0, PB_B182_LR_LOADFROMFILE);
    }
    if (!img) return 0;

    int w = x2 - x1;
    int h = y2 - y1;
    if (!want_icon && (w <= 0 || h <= 0)) {
        unsigned char bm[40];
        for (int i = 0; i < 40; i++) bm[i] = 0;
        if (GetObjectA(img, 40, (void*)bm)) {
            w = bm[4] | (bm[5] << 8) | (bm[6] << 16) | (bm[7] << 24);
            h = bm[8] | (bm[9] << 8) | (bm[10] << 16) | (bm[11] << 24);
        }
    }
    if (w <= 0 || h <= 0) {
        if (type == PB_B182_IMAGE_BITMAP || !type) return 0;
        return 0;   /* an icon with no target rectangle: nothing sane to draw */
    }

    void* mdc = CreateCompatibleDC(g_gr_dc);
    if (!mdc) return 0;
    void* old = SelectObject(mdc, img);
    int ok;
    if (want_icon) {
        ok = DrawIconEx(g_gr_dc, x1, y1, img, w, h, 0, 0, 3 /* DI_NORMAL */);
    } else {
        ok = StretchBlt(g_gr_dc, x1, y1, w, h, mdc, 0, 0, w, h, PB_B182_SRCCOPY);
    }
    SelectObject(mdc, old);
    DeleteDC(mdc);
    if (!want_icon) DeleteObject(img);
    return ok ? 1 : 0;
}

int pb_graphic_render(char* name, int x1, int y1, int x2, int y2) {
    return pb_graphic_render_common(name, x1, y1, x2, y2, 0);
}

int pb_graphic_render_icon(char* name, int x1, int y1, int x2, int y2) {
    return pb_graphic_render_common(name, x1, y1, x2, y2, 1);
}

static void pb_gr_size(int* w, int* h) {
    unsigned char bm[40];
    *w = 0; *h = 0;
    if (!g_gr_bmp) return;
    for (int i = 0; i < 40; i++) bm[i] = 0;
    if (!GetObjectA(g_gr_bmp, 40, (void*)bm)) return;
    *w = bm[4] | (bm[5] << 8) | (bm[6] << 16) | (bm[7] << 24);
    *h = bm[8] | (bm[9] << 8) | (bm[10] << 16) | (bm[11] << 24);
}

int pb_graphic_stretch(long long hb, int id, int x1, int y1, int x2, int y2,
                       int x3, int y3, int x4, int y4, int mix, int stretch) {
    (void)id;
    if (!g_gr_dc || !hb) return 0;
    /* hb is an HBITMAP, not a DC: StretchBlt's source must be a DC with the
       bitmap selected into it - passing the bitmap handle itself painted
       nothing at all. */
    void* sdc = CreateCompatibleDC(g_gr_dc);
    if (!sdc) return 0;
    void* old = SelectObject(sdc, (void*)(long long)hb);
    if (stretch) SetStretchBltMode(g_gr_dc, stretch);
    unsigned long rop = mix ? (unsigned long)mix : PB_B182_SRCCOPY;
    int ok = StretchBlt(g_gr_dc, x3, y3, x4 - x3, y4 - y3, sdc, x1, y1, x2 - x1, y2 - y1, rop);
    SelectObject(sdc, old);
    DeleteDC(sdc);
    return ok ? 1 : 0;
}

int pb_graphic_stretch_page(long long hb, int id, int mix, int stretch) {
    (void)id;
    if (!g_gr_dc || !hb) return 0;
    int tw, th;
    pb_gr_size(&tw, &th);
    if (tw <= 0 || th <= 0) return 0;
    void* sdc = CreateCompatibleDC(g_gr_dc);
    if (!sdc) return 0;
    void* old = SelectObject(sdc, (void*)(long long)hb);
    if (stretch) SetStretchBltMode(g_gr_dc, stretch);
    unsigned long rop = mix ? (unsigned long)mix : PB_B182_SRCCOPY;
    int ok = StretchBlt(g_gr_dc, 0, 0, tw, th, sdc, 0, 0, tw, th, rop);
    SelectObject(sdc, old);
    DeleteDC(sdc);
    return ok ? 1 : 0;
}

/* ---- batch 182: GRAPHIC BITMAP CAPTURE (fork extension) ------------------ */

__declspec(dllimport) void* __stdcall CreateCompatibleBitmap(void* hdc, int cx, int cy);
__declspec(dllimport) int __stdcall BitBlt(void* hdcDst, int x, int y, int w, int h,
                                           void* hdcSrc, int sx, int sy, unsigned long rop);

long long pb_graphic_bitmap_capture(void) {
    if (!g_gr_dc || !g_gr_bmp) return 0;
    int w, h;
    pb_gr_size(&w, &h);
    if (w <= 0 || h <= 0) return 0;
    void* nb = CreateCompatibleBitmap(g_gr_dc, w, h);
    if (!nb) return 0;
    void* mdc = CreateCompatibleDC(g_gr_dc);
    if (!mdc) { DeleteObject(nb); return 0; }
    void* old = SelectObject(mdc, nb);
    int ok = BitBlt(mdc, 0, 0, w, h, g_gr_dc, 0, 0, PB_B182_SRCCOPY);
    SelectObject(mdc, old);
    DeleteDC(mdc);
    if (!ok) { DeleteObject(nb); return 0; }
    return (long long)(size_t)nb;
}
int pb_graphic_width(int w) {
    g_gr_width = w > 0 ? w : 1;
    return 1;
}
int pb_graphic_style(int st) {
    g_gr_style = st;
    return 1;
}
int pb_graphic_save(char* fname) {
    if (!g_gr_dc || !g_gr_bmp) return 0;
    unsigned char bm[40]; /* BITMAP (64-bit: 24 bytes + 8-byte bmBits pointer) */
    for (int i = 0; i < 40; i++) bm[i] = 0;
    if (!GetObjectA(g_gr_bmp, 40, (void*)bm)) return 0;
    int w = bm[4] | (bm[5] << 8) | (bm[6] << 16) | (bm[7] << 24);
    int h = bm[8] | (bm[9] << 8) | (bm[10] << 16) | (bm[11] << 24);
    if (w <= 0 || h <= 0) return 0;
    if (w > 100000 || h > 100000) return 0;
    int bpp = 32;
    int row = ((w * bpp + 31) / 32) * 4;
    int size = row * h;
    unsigned char* bits = (unsigned char*)malloc(size);
    if (!bits) return 0;
    unsigned char bi[40];
    for (int i = 0; i < 40; i++) bi[i] = 0;
    bi[0] = 40;
    bi[4] = (unsigned char)(w & 255); bi[5] = (unsigned char)((w >> 8) & 255);
    bi[6] = (unsigned char)((w >> 16) & 255); bi[7] = (unsigned char)((w >> 24) & 255);
    bi[8] = (unsigned char)(h & 255); bi[9] = (unsigned char)((h >> 8) & 255);
    bi[10] = (unsigned char)((h >> 16) & 255); bi[11] = (unsigned char)((h >> 24) & 255);
    bi[12] = 1; bi[14] = (unsigned char)bpp;
    if (!GetDIBits(g_gr_dc, g_gr_bmp, 0, (unsigned int)h, (void*)bits, (void*)bi, 0)) {
        free(bits); return 0;
    }
    FILE* f = fopen(fname, "wb");
    if (!f) { free(bits); return 0; }
    int bfSize = 14 + 40 + size;
    unsigned char bfh[14];
    for (int i = 0; i < 14; i++) bfh[i] = 0;
    bfh[0] = 'B'; bfh[1] = 'M';
    bfh[2] = (unsigned char)(bfSize & 255); bfh[3] = (unsigned char)((bfSize >> 8) & 255);
    bfh[4] = (unsigned char)((bfSize >> 16) & 255); bfh[5] = (unsigned char)((bfSize >> 24) & 255);
    bfh[10] = 14 + 40;
    fwrite(bfh, 1, 14, f);
    fwrite(bi, 1, 40, f);
    fwrite(bits, 1, size, f);
    fclose(f);
    free(bits);
    return 1;
}

static int gr_bmp_dim(void* bmp, int* w, int* h) {
    unsigned char bm[32];
    for (int i = 0; i < 32; i++) bm[i] = 0;
    if (!GetObjectA(bmp, 32, (void*)bm)) return 0;
    *w = bm[4] | (bm[5] << 8) | (bm[6] << 16) | ((int)bm[7] << 24);
    *h = bm[8] | (bm[9] << 8) | (bm[10] << 16) | ((int)bm[11] << 24);
    if (*h < 0) *h = -*h;
    return 1;
}
static unsigned char* gr_read_bits(void* dc, void* bmp, int w, int h, void* bi) {
    int stride = ((w * 32 + 31) / 32) * 4;
    unsigned char* buf = (unsigned char*)malloc(stride * h);
    if (!buf) return 0;
    int ok = GetDIBits(dc, bmp, 0, (unsigned int)h, buf, bi, 0);
    if (!ok) { free(buf); return 0; }
    return buf;
}
static void gr_write_bits(void* dc, void* bmp, int w, int h, void* bi, unsigned char* buf) {
    SetDIBits(dc, bmp, 0, (unsigned int)h, buf, bi, 0);
}

/* GRAPHIC SET PIXEL / GET SIZE / SET+GET TEXTALIGN (batch 59) */
static long g_gr_textalign = 0;
int pb_graphic_set_pixel(long x, long y, long color) {
    if (!g_gr_dc || !g_gr_bmp) return 0;
    int w = 0, h = 0;
    if (!gr_bmp_dim(g_gr_bmp, &w, &h)) return 0;
    if (x < 0 || y < 0 || x >= w || y >= h) return 0;
    unsigned char bi[40];
    for (int i = 0; i < 40; i++) bi[i] = 0;
    bi[0] = 40;
    bi[4] = (unsigned char)(w & 255); bi[5] = (unsigned char)((w >> 8) & 255);
    bi[6] = (unsigned char)((w >> 16) & 255); bi[7] = (unsigned char)((w >> 24) & 255);
    int nh = -h;
    bi[8] = (unsigned char)(nh & 255); bi[9] = (unsigned char)((nh >> 8) & 255);
    bi[10] = (unsigned char)((nh >> 16) & 255); bi[11] = (unsigned char)((nh >> 24) & 255);
    bi[12] = 1; bi[14] = 32;
    int stride = ((w * 32 + 31) / 32) * 4;
    unsigned char* buf = gr_read_bits(g_gr_dc, g_gr_bmp, w, h, (void*)bi);
    if (!buf) return 0;
    unsigned char* px = buf + (unsigned long)y * stride + (unsigned long)x * 4;
    px[0] = (unsigned char)((color >> 16) & 255);
    px[1] = (unsigned char)((color >> 8) & 255);
    px[2] = (unsigned char)(color & 255);
    px[3] = 0;
    gr_write_bits(g_gr_dc, g_gr_bmp, w, h, (void*)bi, buf);
    free(buf);
    return 1;
}
int pb_graphic_get_size(long* w, long* h) {
    if (!g_gr_bmp) return 0;
    unsigned char bm[32];
    for (int i = 0; i < 32; i++) bm[i] = 0;
    GetObjectA(g_gr_bmp, 32, (void*)bm);
    *w = bm[4] | (bm[5] << 8) | (bm[6] << 16) | ((long)bm[7] << 24);
    *h = bm[8] | (bm[9] << 8) | (bm[10] << 16) | ((long)bm[11] << 24);
    return 1;
}
int pb_graphic_set_textalign(long align) {
    g_gr_textalign = align;
    return 1;
}
int pb_graphic_get_textalign(long* align) {
    *align = g_gr_textalign;
    return 1;
}


/* GRAPHIC GET BITS / SET BITS / GET SCALE / SCALE / SET AUTOSIZE (batch 64) */
static long g_gr_scale_x1 = 0, g_gr_scale_y1 = 0, g_gr_scale_x2 = 0, g_gr_scale_y2 = 0;
static long g_gr_autosize_w = 0, g_gr_autosize_h = 0;
#define PB_MM_TEXT 1
#define PB_MM_ANISOTROPIC 10

int pb_graphic_get_bits(char** dest) {
    if (!g_gr_bmp) return 0;
    int w = 0, h = 0;
    if (!gr_bmp_dim(g_gr_bmp, &w, &h)) return 0;
    unsigned char bi[40];
    for (int i = 0; i < 40; i++) bi[i] = 0;
    int stride = ((w * 32 + 31) / 32) * 4;
    long size = (long)stride * h;
    bi[0] = 40;
    bi[4] = (unsigned char)(w & 255); bi[5] = (unsigned char)((w >> 8) & 255);
    bi[6] = (unsigned char)((w >> 16) & 255); bi[7] = (unsigned char)((w >> 24) & 255);
    bi[8] = (unsigned char)(h & 255); bi[9] = (unsigned char)((h >> 8) & 255);
    bi[10] = (unsigned char)((h >> 16) & 255); bi[11] = (unsigned char)((h >> 24) & 255);
    bi[12] = 1; bi[14] = 32;
    bi[20] = (unsigned char)(size & 255); bi[21] = (unsigned char)((size >> 8) & 255);
    bi[22] = (unsigned char)((size >> 16) & 255); bi[23] = (unsigned char)((size >> 24) & 255);
    unsigned char* buf = gr_read_bits(g_gr_dc, g_gr_bmp, w, h, (void*)bi);
    if (!buf) return 0;
    unsigned char* dib = (unsigned char*)malloc((size_t)40 + (size_t)size);
    if (!dib) { free(buf); return 0; }
    memcpy(dib, bi, 40);
    memcpy(dib + 40, buf, (size_t)size);
    free(buf);
    *dest = pb_bstr_alloc((const char*)dib, (unsigned int)(40 + size));
    free(dib);
    return 1;
}

int pb_graphic_set_bits(char* src) {
    if (!src || !g_gr_dc) return 0;
    int w = src[4] | (src[5] << 8) | (src[6] << 16) | ((int)src[7] << 24);
    int h = src[8] | (src[9] << 8) | (src[10] << 16) | ((int)src[11] << 24);
    int bpp = src[14] | (src[15] << 8);
    if (w <= 0 || w > 100000 || h == 0) return 0;
    if (bpp != 32 && bpp != 24 && bpp != 8 && bpp != 1) return 0;
    int ah = (h < 0) ? -h : h;
    if (ah <= 0 || ah > 100000) return 0;
    long size = src[20] | (src[21] << 8) | (src[22] << 16) | ((long)src[23] << 24);
    if (size <= 0) { int stride = ((w * bpp + 31) / 32) * 4; size = (long)stride * ah; }
    /* src IS a 40-byte BITMAPINFOHEADER (produced by pb_graphic_get_bits) */
    void* bits = NULL;
    void* nb = CreateDIBSection(g_gr_dc, (void*)src, 0, &bits, NULL, 0);
    if (!nb) return 0;
    SetDIBits(g_gr_dc, nb, 0, (unsigned int)ah, src + 40, (void*)src, 0);
    if (g_gr_bmp) {
        SelectObject(g_gr_dc, GetStockObject(5)); /* NULL_BRUSH: detach old bitmap */
        DeleteObject(g_gr_bmp);
    }
    g_gr_bmp = nb;
    SelectObject(g_gr_dc, g_gr_bmp);
    return 1;
}

int pb_graphic_get_scale(float* x1, float* y1, float* x2, float* y2) {
    if (!g_gr_bmp) return 0;
    int w = 0, h = 0;
    gr_bmp_dim(g_gr_bmp, &w, &h);
    *x1 = (float)g_gr_scale_x1;
    *y1 = (float)g_gr_scale_y1;
    long x2l = g_gr_scale_x2, y2l = g_gr_scale_y2;
    if (x2l == 0 && y2l == 0) { x2l = w; y2l = h; }
    *x2 = (float)x2l;
    *y2 = (float)y2l;
    return 1;
}

int pb_graphic_scale(float x1, float y1, float x2, float y2) {
    if (!g_gr_dc) return 0;
    g_gr_scale_x1 = (long)x1; g_gr_scale_y1 = (long)y1;
    g_gr_scale_x2 = (long)x2; g_gr_scale_y2 = (long)y2;
    int w = 0, h = 0;
    if (g_gr_bmp) gr_bmp_dim(g_gr_bmp, &w, &h);
    SetMapMode(g_gr_dc, PB_MM_ANISOTROPIC);
    SetWindowExtEx(g_gr_dc, w, h, NULL);
    SetViewportExtEx(g_gr_dc, (int)(x2 - x1), (int)(y2 - y1), NULL);
    SetViewportOrgEx(g_gr_dc, (int)x1, (int)y1, NULL);
    return 1;
}

int pb_graphic_scale_pixels(void) {
    if (!g_gr_dc) return 0;
    SetMapMode(g_gr_dc, PB_MM_TEXT);
    g_gr_scale_x1 = 0; g_gr_scale_y1 = 0; g_gr_scale_x2 = 0; g_gr_scale_y2 = 0;
    return 1;
}

int pb_graphic_set_autosize(long w, long h) {
    g_gr_autosize_w = w;
    g_gr_autosize_h = h;
    return 1;
}

/* GRAPHIC GET PPI / GET POS / SET POS / TEXT SIZE / STRETCHMODE / CAPTION (batch 61) */
#define PB_LOGPIXELSX 88
#define PB_LOGPIXELSY 90
/* GRAPHIC SET SIZE / SET CLIP / SET VIRTUAL / WORDWRAP (batch 65) */
__declspec(dllimport) int __stdcall IntersectClipRect(void* hdc, int l, int t, int r, int b);
int pb_graphic_set_size(int w, int h) {
    if (!g_gr_dc || !g_gr_bmp) return 0;
    if (w <= 0 || h <= 0 || w > 100000 || h > 100000) return 0;
    unsigned char bi[40];
    for (int i = 0; i < 40; i++) bi[i] = 0;
    bi[0] = 40;
    bi[4] = (unsigned char)(w & 0xFF); bi[5] = (unsigned char)((w >> 8) & 0xFF);
    bi[6] = (unsigned char)((w >> 16) & 0xFF); bi[7] = (unsigned char)((w >> 24) & 0xFF);
    int hh = -h; /* top-down, matches pb_gdi_bitmap_new */
    bi[8] = (unsigned char)(hh & 0xFF); bi[9] = (unsigned char)((hh >> 8) & 0xFF);
    bi[10] = (unsigned char)((hh >> 16) & 0xFF); bi[11] = (unsigned char)((hh >> 24) & 0xFF);
    bi[12] = 1; /* biPlanes */
    bi[14] = 32; /* biBitCount */
    long stride = ((w * 32 + 31) / 32) * 4;
    long size = stride * h;
    bi[20] = (unsigned char)(size & 0xFF); bi[21] = (unsigned char)((size >> 8) & 0xFF);
    bi[22] = (unsigned char)((size >> 16) & 0xFF); bi[23] = (unsigned char)((size >> 24) & 0xFF);
    void* bits = NULL;
    void* nb = CreateDIBSection(g_gr_dc, (void*)bi, 0, &bits, NULL, 0);
    if (!nb) return 0;
    if (g_gr_bmp) {
        SelectObject(g_gr_dc, GetStockObject(5)); /* NULL_BRUSH: detach old bitmap */
        DeleteObject(g_gr_bmp);
    }
    g_gr_bmp = nb;
    SelectObject(g_gr_dc, g_gr_bmp);
    return 1;
}
static float g_gr_clip_l = 0, g_gr_clip_t = 0, g_gr_clip_r = 0, g_gr_clip_b = 0;
static int g_gr_clip_set = 0;
int pb_graphic_set_clip(float l, float t, float r, float b) {
    g_gr_clip_l = l; g_gr_clip_t = t; g_gr_clip_r = r; g_gr_clip_b = b;
    g_gr_clip_set = 1;
    if (g_gr_dc) IntersectClipRect(g_gr_dc, (int)l, (int)t, (int)r, (int)b);
    return 1;
}
static long g_gr_virtual_w = 0, g_gr_virtual_h = 0;
static long g_gr_wordwrap = 1; /* GRAPHIC SET/GET WORDWRAP state (batch 65) */
int pb_graphic_set_virtual(int w, int h) { g_gr_virtual_w = w; g_gr_virtual_h = h; return 1; }
static long g_gr_fixed = 0;
static void* g_gr_font = 0;
int pb_graphic_set_fixed(void) {
    g_gr_fixed = 1;
    return 1;
}
int pb_graphic_set_font(long long hFont) {
    g_gr_font = (void*)(intptr_t)hFont;
    if (g_gr_dc && g_gr_font) SelectObject(g_gr_dc, g_gr_font);
    return 1;
}
int pb_graphic_set_wordwrap(int n) { g_gr_wordwrap = n ? 1 : 0; return 1; }
int pb_graphic_get_wordwrap(int* out) { if (out) *out = g_gr_wordwrap; return 1; }

int pb_graphic_get_ppi(long* x, long* y) {
    if (!g_gr_dc) return 0;
    *x = GetDeviceCaps(g_gr_dc, PB_LOGPIXELSX);
    *y = GetDeviceCaps(g_gr_dc, PB_LOGPIXELSY);
    return 1;
}
int pb_graphic_get_pos(float* x, float* y) {
    if (!g_gr_dc) return 0;
    pb_pt pt;
    pt.x = 0; pt.y = 0;
    if (!GetCurrentPositionEx(g_gr_dc, (void*)&pt)) return 0;
    *x = (float)pt.x;
    *y = (float)pt.y;
    return 1;
}
int pb_graphic_set_pos(float x, float y) {
    if (!g_gr_dc) return 0;
    MoveToEx(g_gr_dc, (int)x, (int)y, 0);
    return 1;
}
int pb_graphic_text_size(char* txt, float* w, float* h) {
    if (!g_gr_dc) return 0;
    long sz[2];
    sz[0] = 0; sz[1] = 0;
    if (!GetTextExtentPoint32A(g_gr_dc, txt, (int)strlen(txt), (void*)sz)) return 0;
    *w = (float)sz[0];
    *h = (float)sz[1];
    return 1;
}
int pb_graphic_get_stretchmode(long* m) {
    if (!g_gr_dc) return 0;
    *m = GetStretchBltMode(g_gr_dc);
    return 1;
}
int pb_graphic_set_stretchmode(long m) {
    if (!g_gr_dc) return 0;
    SetStretchBltMode(g_gr_dc, (int)m);
    return 1;
}
int pb_graphic_get_caption(char** dest) {
    if (!dest) return 0;
    char buf[1024];
    buf[0] = '\0';
    GetConsoleTitleA(buf, 1024);
    *dest = pb_bstr_alloc(buf, (unsigned int)strlen(buf));
    return 1;
}
int pb_graphic_set_caption(char* s) {
    SetConsoleTitleA(s);
    return 1;
}

/* GRAPHIC GET CLIP / GET VIEW / SET VIEW / GET LINES / GET+SET WRAP (batch 63) */
static long g_gr_wrap = 1; /* default: text wraps */
int pb_graphic_get_clip(float* w, float* h) {
    if (g_gr_clip_set) {
        *w = g_gr_clip_r - g_gr_clip_l;
        *h = g_gr_clip_b - g_gr_clip_t;
        return 1;
    }
    if (!g_gr_dc) return 0;
    int rc[4];
    rc[0] = 0; rc[1] = 0; rc[2] = 0; rc[3] = 0;
    if (!GetClipBox(g_gr_dc, (void*)rc)) return 0;
    *w = (float)(rc[2] - rc[0]);
    *h = (float)(rc[3] - rc[1]);
    return 1;
}
int pb_graphic_get_view(float* x, float* y) {
    if (!g_gr_dc) return 0;
    pb_pt pt;
    pt.x = 0; pt.y = 0;
    if (!GetViewportOrgEx(g_gr_dc, (void*)&pt)) return 0;
    *x = (float)pt.x;
    *y = (float)pt.y;
    return 1;
}
int pb_graphic_set_view(float x, float y) {
    if (!g_gr_dc) return 0;
    SetViewportOrgEx(g_gr_dc, (int)x, (int)y, 0);
    return 1;
}
int pb_graphic_get_lines(long* n) {
    if (!g_gr_bmp) return 0;
    unsigned char bm[32];
    for (int i = 0; i < 32; i++) bm[i] = 0;
    GetObjectA(g_gr_bmp, 32, (void*)bm);
    *n = bm[8] | (bm[9] << 8) | (bm[10] << 16) | ((long)bm[11] << 24);
    return 1;
}
int pb_graphic_get_wrap(long* w) {
    *w = g_gr_wrap;
    return 1;
}
int pb_graphic_set_wrap(long w) {
    g_gr_wrap = w ? 1 : 0;
    return 1;
}

/* GRAPHIC GET CANVAS / GET DC / GET MIX / SET MIX (batch 58) */
static long g_gr_mix = 13; /* R2_COPYPEN default */
void* pb_graphic_get_canvas(void) {
    return g_gr_bmp;
}
void* pb_graphic_get_dc(void) {
    return g_gr_dc;
}
int pb_graphic_set_mix(long mix) {
    g_gr_mix = mix;
    return 1;
}
int pb_graphic_get_mix(long* mix) {
    *mix = g_gr_mix;
    return 1;
}

/* GRAPHIC BITMAP LOAD / CHR SIZE / CELL / CELL SIZE (batch 57) */
void* pb_graphic_bitmap_load(char* fname) {
    if (!fname) return 0;
    return LoadImageA(0, fname, 0, 0, 0, 0x10); /* IMAGE_BITMAP, LR_LOADFROMFILE */
}
static int gr_char_w(void) {
    unsigned char sz[8];
    for (int i = 0; i < 8; i++) sz[i] = 0;
    void* old = SelectObject(g_gr_dc, GetStockObject(17)); /* DEFAULT_GUI_FONT */
    GetTextExtentPoint32A(g_gr_dc, "W", 1, (void*)sz);
    SelectObject(g_gr_dc, old);
    int w = sz[0] | (sz[1] << 8) | (sz[2] << 16) | (sz[3] << 24);
    return w > 0 ? w : 8;
}
static int gr_char_h(void) {
    unsigned char sz[8];
    for (int i = 0; i < 8; i++) sz[i] = 0;
    void* old = SelectObject(g_gr_dc, GetStockObject(17));
    GetTextExtentPoint32A(g_gr_dc, "W", 1, (void*)sz);
    SelectObject(g_gr_dc, old);
    int h = sz[4] | (sz[5] << 8) | (sz[6] << 16) | (sz[7] << 24);
    return h > 0 ? h : 16;
}
int pb_graphic_chr_size(char* text, long* w, long* h) {
    if (!text) text = "";
    int len = 0;
    while (text[len]) len++;
    unsigned char sz[8];
    for (int i = 0; i < 8; i++) sz[i] = 0;
    void* old = SelectObject(g_gr_dc, GetStockObject(17));
    GetTextExtentPoint32A(g_gr_dc, text, len, (void*)sz);
    SelectObject(g_gr_dc, old);
    *w = sz[0] | (sz[1] << 8) | (sz[2] << 16) | (sz[3] << 24);
    *h = sz[4] | (sz[5] << 8) | (sz[6] << 16) | (sz[7] << 24);
    return 1;
}
int pb_graphic_cell_size(long rows, long cols, long* w, long* h) {
    int cw = gr_char_w();
    int ch = gr_char_h();
    *w = cw * cols;
    *h = ch * rows;
    return 1;
}
int pb_graphic_cell(long row, long col, long* x, long* y) {
    int cw = gr_char_w();
    int ch = gr_char_h();
    *x = cw * col;
    *y = ch * row;
    return 1;
}

/* GRAPHIC CIRCLE / POLYGON / GET CLIENT / GET LOC (batch 56) */
int pb_graphic_circle(int x, int y, int r, unsigned long col) {
    if (!g_gr_dc) return 0;
    void* pen = CreatePen(g_gr_style, g_gr_width, (unsigned long)(unsigned int)col);
    SelectObject(g_gr_dc, pen);
    int rc = Ellipse(g_gr_dc, x - r, y - r, x + r, y + r);
    SelectObject(g_gr_dc, GetStockObject(0)); /* NULL_PEN */
    DeleteObject(pen);
    return rc;
}
int pb_graphic_polygon(long* pts, int count, unsigned long col) {
    if (!g_gr_dc || count < 3) return 0;
    void* pen = CreatePen(g_gr_style, g_gr_width, (unsigned long)(unsigned int)col);
    SelectObject(g_gr_dc, pen);
    int rc = Polygon(g_gr_dc, (const long*)pts, count);
    SelectObject(g_gr_dc, GetStockObject(0));
    DeleteObject(pen);
    return rc;
}
int pb_graphic_get_client(long* w, long* h) {
    if (!g_gr_dc || !g_gr_bmp) return 0;
    unsigned char bm[40];
    for (int i = 0; i < 40; i++) bm[i] = 0;
    if (!GetObjectA(g_gr_bmp, 40, (void*)bm)) return 0;
    *w = bm[4] | (bm[5] << 8) | (bm[6] << 16) | (bm[7] << 24);
    *h = bm[8] | (bm[9] << 8) | (bm[10] << 16) | (bm[11] << 24);
    return 1;
}
static int pb_graphic_win_rect(long* x, long* y);   /* batch 181 */
int pb_graphic_get_loc(long* x, long* y) {
    /* The help page: "If no graphic object is selected, or it is not a Graphic
       Window, 0,0 is returned.  The location is specified in pixels, relative
       to the upper left corner of the screen."  This used to answer 0,0 always,
       which happened to be right only for the not-a-window case. */
    *x = 0;
    *y = 0;
    return pb_graphic_win_rect(x, y);
}

/* GRAPHIC COLOR / GET PIXEL / COPY (batch 55) */
static unsigned long g_gr_fore = 0;
static unsigned long g_gr_back = 0;
int pb_graphic_color(unsigned long fore, unsigned long back) {
    g_gr_fore = fore;
    g_gr_back = back;
    return 1;
}

#ifndef PB_GR_LINECAP
#define PB_GR_LINECAP 260
#endif
/* GRAPHIC SPLIT / SPLIT_WORD: how much of MainStr fits in a print field that is
   Part1Len page units wide.  Width comes from the graphic DC's current font; with
   no attached graphic target there is nothing to measure, so a nominal 8 units
   per character is used and that fallback is what the headless example sees. */
static int pb_gr_measure_ok(const char* buf, int len) {
    long sz[2];
    sz[0] = 0;
    sz[1] = 0;
    if (!GetTextExtentPoint32A(g_gr_dc, buf, len, (void*)sz)) return -1;
    return (int)sz[0];
}

static void pb_graphic_split_core(const char* src, int fieldw,
                                  char** out1, char** out2, int word) {
    int n = (int)strlen(src);
    int best = 0, i, k;
    char buf[PB_GR_LINECAP];
    if (n >= PB_GR_LINECAP) n = PB_GR_LINECAP - 1;
    if (g_gr_dc) {
        for (i = 1; i <= n; i++) {
            int w;
            for (k = 0; k < i; k++) buf[k] = src[k];
            buf[i] = '\0';
            w = pb_gr_measure_ok(buf, i);
            if (w < 0 || w > fieldw) break;
            best = i;
        }
    } else {
        best = fieldw > 0 ? fieldw / 8 : 0;
        if (best > n) best = n;
    }
    if (word && best > 0 && best < n) {
        k = best;
        while (k > 0 && src[k - 1] != ' ') k--;
        if (k > 0) best = k;
    }
    if (out1) *out1 = pb_bstr_alloc(src, (unsigned int)best);
    if (out2) *out2 = pb_bstr_alloc(src + best, (unsigned int)(n - best));
}
int pb_graphic_split(const char* src, int fieldw, char** out1, char** out2) {
    pb_graphic_split_core(src ? src : "", fieldw, out1, out2, 0);
    return 1;
}
int pb_graphic_split_word(const char* src, int fieldw, char** out1, char** out2) {
    pb_graphic_split_core(src ? src : "", fieldw, out1, out2, 1);
    return 1;
}

int pb_graphic_print_str(const char* s) {
    /* GRAPHIC PRINT - draw text on the attached graphic bitmap DC (batch 118) */
    if (!g_gr_dc) return 0;
    pb_gw_touch();
    if (!s) s = "";
    int len = (int)strlen(s);
    SetTextColor(g_gr_dc, (unsigned long)(unsigned int)g_gr_fore);
    SetBkMode(g_gr_dc, 1); /* TRANSPARENT */
    if (g_gr_font) SelectObject(g_gr_dc, g_gr_font);
    pb_pt pt; pt.x = 0; pt.y = 0;
    GetCurrentPositionEx(g_gr_dc, (void*)&pt);
    int ok = TextOutA(g_gr_dc, pt.x, pt.y, s, len);
    long sz[2]; sz[0] = 0; sz[1] = 0;
    if (GetTextExtentPoint32A(g_gr_dc, s, len, (void*)sz)) MoveToEx(g_gr_dc, pt.x + sz[0], pt.y, 0);
    return ok ? 1 : 0;
}

int pb_graphic_get_pixel(int x, int y, unsigned long* out) {
    if (!g_gr_dc || !g_gr_bmp) return 0;
    int w = 0, h = 0;
    if (!gr_bmp_dim(g_gr_bmp, &w, &h)) return 0;
    if (x < 0 || y < 0 || x >= w || y >= h) return 0;
    unsigned char bi[40];
    for (int i = 0; i < 40; i++) bi[i] = 0;
    bi[0] = 40;
    bi[4] = (unsigned char)(w & 255); bi[5] = (unsigned char)((w >> 8) & 255);
    bi[6] = (unsigned char)((w >> 16) & 255); bi[7] = (unsigned char)((w >> 24) & 255);
    int nh = -h;
    bi[8] = (unsigned char)(nh & 255); bi[9] = (unsigned char)((nh >> 8) & 255);
    bi[10] = (unsigned char)((nh >> 16) & 255); bi[11] = (unsigned char)((nh >> 24) & 255);
    bi[12] = 1; bi[14] = 32;
    int stride = ((w * 32 + 31) / 32) * 4;
    unsigned char* buf = gr_read_bits(g_gr_dc, g_gr_bmp, w, h, (void*)bi);
    if (!buf) return 0;
    unsigned char* px = buf + (unsigned long)y * stride + (unsigned long)x * 4;
    *out = ((unsigned long)px[0] << 16) | ((unsigned long)px[1] << 8) | px[2];
    free(buf);
    return 1;
}
int pb_graphic_copy(int x1, int y1, int x2, int y2, int x3, int y3) {
    if (!g_gr_dc) return 0;
    int w = x2 - x1 + 1;
    int h = y2 - y1 + 1;
    if (w <= 0 || h <= 0) return 0;
    return BitBlt(g_gr_dc, x3, y3, w, h, g_gr_dc, x1, y1, 0x00CC0020); /* SRCCOPY */
}

/* GRAPHIC LINE/BOX/ELLIPSE � drawing on attached target (batch 53) */
int pb_graphic_line(int x1, int y1, int x2, int y2, int color) {
    if (!g_gr_dc) return 0;
    pb_gw_touch();
    void* pen = CreatePen(g_gr_style, g_gr_width, (unsigned long)(unsigned int)color);
    void* old = SelectObject(g_gr_dc, pen);
    MoveToEx(g_gr_dc, x1, y1, 0);
    int ok = LineTo(g_gr_dc, x2, y2);
    SelectObject(g_gr_dc, old);
    if (pen) DeleteObject(pen);
    return ok ? 1 : 0;
}
int pb_graphic_box(int x1, int y1, int x2, int y2, int color, int fillcolor, int fillstyle) {
    if (!g_gr_dc) return 0;
    if (g_gr_overlap) { x2 += 1; y2 += 1; }   /* batch 181 */
    pb_gw_touch();
    void* pen = CreatePen(g_gr_style, g_gr_width, (unsigned long)(unsigned int)color);
    void* oldp = SelectObject(g_gr_dc, pen);
    void* br = fillstyle ? CreateSolidBrush((unsigned long)(unsigned int)fillcolor) : GetStockObject(5); /* NULL_BRUSH */
    void* oldb = SelectObject(g_gr_dc, br);
    int ok = Rectangle(g_gr_dc, x1, y1, x2, y2);
    SelectObject(g_gr_dc, oldp);
    SelectObject(g_gr_dc, oldb);
    if (pen) DeleteObject(pen);
    if (fillstyle && br) DeleteObject(br);
    return ok ? 1 : 0;
}
int pb_graphic_ellipse(int x1, int y1, int x2, int y2, int color, int fillcolor, int fillstyle) {
    if (!g_gr_dc) return 0;
    if (g_gr_overlap) { x2 += 1; y2 += 1; }   /* batch 181 */
    pb_gw_touch();
    void* pen = CreatePen(g_gr_style, g_gr_width, (unsigned long)(unsigned int)color);
    void* oldp = SelectObject(g_gr_dc, pen);
    void* br = fillstyle ? CreateSolidBrush((unsigned long)(unsigned int)fillcolor) : GetStockObject(5);
    void* oldb = SelectObject(g_gr_dc, br);
    int ok = Ellipse(g_gr_dc, x1, y1, x2, y2);
    SelectObject(g_gr_dc, oldp);
    SelectObject(g_gr_dc, oldb);
    if (pen) DeleteObject(pen);
    if (fillstyle && br) DeleteObject(br);
    return ok ? 1 : 0;
}

/* GRAPHIC ARC / PIE / POLYLINE / PAINT (batch 60) */
static void pb_angle_point(int cx, int cy, int rx, int ry, int deg, int* px, int* py) {
    double rad = (double)deg * 3.141592653589793 / 180.0;
    *px = (int)(cx + (double)rx * cos(rad));
    *py = (int)(cy - (double)ry * sin(rad));
}
int pb_graphic_arc(int x1, int y1, int x2, int y2, int start, int end, int color) {
    if (!g_gr_dc) return 0;
    int cx = (x1 + x2) / 2, cy = (y1 + y2) / 2, rx = (x2 - x1) / 2, ry = (y2 - y1) / 2;
    int sx = 0, sy = 0, ex = 0, ey = 0;
    pb_angle_point(cx, cy, rx, ry, start, &sx, &sy);
    pb_angle_point(cx, cy, rx, ry, end, &ex, &ey);
    void* pen = CreatePen(g_gr_style, g_gr_width, (unsigned long)(unsigned int)color);
    void* old = SelectObject(g_gr_dc, pen);
    int ok = Arc(g_gr_dc, x1, y1, x2, y2, sx, sy, ex, ey);
    SelectObject(g_gr_dc, old);
    if (pen) DeleteObject(pen);
    return ok ? 1 : 0;
}
int pb_graphic_pie(int x1, int y1, int x2, int y2, int start, int end, int color, int fillcolor, int fillstyle) {
    if (!g_gr_dc) return 0;
    int cx = (x1 + x2) / 2, cy = (y1 + y2) / 2, rx = (x2 - x1) / 2, ry = (y2 - y1) / 2;
    int sx = 0, sy = 0, ex = 0, ey = 0;
    pb_angle_point(cx, cy, rx, ry, start, &sx, &sy);
    pb_angle_point(cx, cy, rx, ry, end, &ex, &ey);
    void* pen = CreatePen(g_gr_style, g_gr_width, (unsigned long)(unsigned int)color);
    void* oldp = SelectObject(g_gr_dc, pen);
    void* br = fillstyle ? CreateSolidBrush((unsigned long)(unsigned int)fillcolor) : GetStockObject(5);
    void* oldb = SelectObject(g_gr_dc, br);
    int ok = Pie(g_gr_dc, x1, y1, x2, y2, sx, sy, ex, ey);
    SelectObject(g_gr_dc, oldp);
    SelectObject(g_gr_dc, oldb);
    if (pen) DeleteObject(pen);
    if (fillstyle && br) DeleteObject(br);
    return ok ? 1 : 0;
}
int pb_graphic_polyline(long* pts, int npts, int color) {
    if (!g_gr_dc || npts < 2) return 0;
    void* pen = CreatePen(g_gr_style, g_gr_width, (unsigned long)(unsigned int)color);
    void* old = SelectObject(g_gr_dc, pen);
    int ok = Polyline(g_gr_dc, (const void*)pts, npts);
    SelectObject(g_gr_dc, old);
    if (pen) DeleteObject(pen);
    return ok ? 1 : 0;
}
int pb_graphic_paint(int x, int y, int fillcolor, int border, int fillstyle) {
    if (!g_gr_dc) return 0;
    void* br = CreateSolidBrush((unsigned long)(unsigned int)fillcolor);
    void* oldb = SelectObject(g_gr_dc, br);
    int ok = FloodFill(g_gr_dc, x, y, (unsigned long)(unsigned int)border);
    SelectObject(g_gr_dc, oldb);
    if (br) DeleteObject(br);
    return ok ? 1 : 0;
}

/* GRAPHIC ATTACH/DETACH/CLEAR � bitmap graphic target (batch 52) */
int pb_graphic_attach(long long h) {
    pb_graphic_release_dc();
    g_gr_bmp = (void*)(intptr_t)h;
    if (!g_gr_bmp) return 0;
    g_gr_dc = CreateCompatibleDC(0);
    if (!g_gr_dc) return 0;
    SelectObject(g_gr_dc, g_gr_bmp);
    return 1;
}
int pb_graphic_detach(void) {
    pb_graphic_release_dc();
    g_gr_bmp = 0;
    return 1;
}
/* Batch 179 - a GRAPHIC target may be a control, not only a memory bitmap.
   GRAPHIC ATTACH hDlg, id& was parsed and then the id operand was dropped,
   so that form attached to nothing at all; pb_graphic_attach_ctl is what it
   calls now.  CreateCompatibleDC + SelectObject is the right recipe for a
   bitmap and the wrong one for a window, so the two targets are told apart
   here and the DC is released the matching way on detach.
   The DIB-section statements (GET BITS / SET PIXEL / COPY / GET PIXEL ...)
   still require g_gr_bmp and so decline on a control target; the plain GDI
   drawing statements (CLEAR / LINE / BOX / CIRCLE / COLOR / SET ...) work. */
__declspec(dllimport) void* __stdcall GetDC(void* hWnd);
__declspec(dllimport) int __stdcall ReleaseDC(void* hWnd, void* hDC);
__declspec(dllimport) int __stdcall IsWindow(void* hWnd);

static void pb_graphic_release_dc(void) {
    if (!g_gr_dc) return;
    if (g_gr_dc_win) ReleaseDC(g_gr_dc_win, g_gr_dc);
    else DeleteDC(g_gr_dc);
    g_gr_dc = 0;
    g_gr_dc_win = 0;
}

int pb_graphic_attach_ctl(void* hDlg, long long id) {
    void* h = pb_pb_hwnd(hDlg, id);
    if (!h || !IsWindow(h)) return 0;
    pb_graphic_release_dc();
    g_gr_bmp = 0;
    g_gr_dc = GetDC(h);
    if (!g_gr_dc) return 0;
    g_gr_dc_win = h;
    return 1;
}

int pb_graphic_clear(int color) {
    if (!g_gr_dc) return 0;
    pb_gw_touch();
    unsigned char rc[16];
    for (int i = 0; i < 16; i++) rc[i] = 0;
    int cw = 0, ch = 0;
    if (g_gr_dc_win) {
        /* a window or control DC: the target is its client area */
        if (!GetClientRect(g_gr_dc_win, (void*)rc)) return 0;
    } else if (g_gr_bmp && gr_bmp_dim(g_gr_bmp, &cw, &ch)) {
        /* a memory DC over a DIB: the target is the bitmap's own size */
        rc[8]  = (unsigned char)(cw & 255);         rc[9]  = (unsigned char)((cw >> 8) & 255);
        rc[10] = (unsigned char)((cw >> 16) & 255); rc[11] = (unsigned char)((cw >> 24) & 255);
        rc[12] = (unsigned char)(ch & 255);         rc[13] = (unsigned char)((ch >> 8) & 255);
        rc[14] = (unsigned char)((ch >> 16) & 255); rc[15] = (unsigned char)((ch >> 24) & 255);
    } else {
        return 0;                     /* nothing to clear: no known target */
    }
    void* br = CreateSolidBrush((unsigned long)(unsigned int)color);
    int ok = FillRect(g_gr_dc, (const void*)rc, br);
    if (br) DeleteObject(br);
    return ok ? 1 : 0;
}

/* XPRINT � host-based printer GDI (batch 68) */
static void* g_xp_dc = 0;
#define PB_LOGPIXELSX 88
#define PB_LOGPIXELSY 90
#define PB_PHYSICALWIDTH 110
#define PB_PHYSICALHEIGHT 111

int pb_xprint_attach(char* printer, char* job) {
    (void)job;
    if (g_xp_dc) { DeleteDC(g_xp_dc); g_xp_dc = 0; }
    void* dc = 0;
    /* Always use screen DC for now � printer DC requires winspool and may not exist in CI */
    dc = CreateDCA("DISPLAY", 0, 0, 0);
    g_xp_dc = dc;
    return dc ? 1 : 0;
}

int pb_xprint_close(void) {
    if (g_xp_dc) { DeleteDC(g_xp_dc); g_xp_dc = 0; }
    return 1;
}

int pb_xprint_get_ppi(long long* x, long long* y) {
    if (!g_xp_dc) { *x = 0; *y = 0; return 0; }
    *x = GetDeviceCaps(g_xp_dc, PB_LOGPIXELSX);
    *y = GetDeviceCaps(g_xp_dc, PB_LOGPIXELSY);
    return 1;
}

int pb_xprint_get_size(long long* w, long long* h) {
    if (!g_xp_dc) { *w = 0; *h = 0; return 0; }
    *w = GetDeviceCaps(g_xp_dc, PB_PHYSICALWIDTH);
    *h = GetDeviceCaps(g_xp_dc, PB_PHYSICALHEIGHT);
    /* Screen DC fallback: PHYSICALWIDTH may be 0, use HORZRES/VERTRES */
    if (*w == 0) *w = GetDeviceCaps(g_xp_dc, 8);   /* HORZRES */
    if (*h == 0) *h = GetDeviceCaps(g_xp_dc, 10);  /* VERTRES */
    return 1;
}

int pb_xprint_get_dc(long long* hdc) {
    *hdc = (long long)(uintptr_t)g_xp_dc;
    return g_xp_dc ? 1 : 0;
}

/* XPRINT drawing + text + attributes (batch 69) */
static void* g_xp_pen = 0;
static void* g_xp_brush = 0;
static void* g_xp_font = 0;
static long g_xp_color = 0x000000;
static long g_xp_pos_x = 0, g_xp_pos_y = 0;
static long g_xp_textalign = 0;
static long g_xp_wrap = 0;
static long g_xp_wordwrap = 0;
static long g_xp_overlap = 0;
static long g_xp_scale_w = 0;
static long g_xp_scale_h = 0;
static long g_xp_scale_x1 = 0, g_xp_scale_y1 = 0, g_xp_scale_x2 = 0, g_xp_scale_y2 = 0;
static long g_xp_arc_x1 = 0, g_xp_arc_y1 = 0, g_xp_arc_x2 = 0, g_xp_arc_y2 = 0;
static double g_xp_arc_a = 0.0, g_xp_arc_b = 0.0;
/* === Batch 74: XPRINT printer property globals === */
static long g_xp_copies = 1;
static long g_xp_orientation = 1; /* 1=portrait, 2=landscape */
static long g_xp_quality = 0;     /* 0=draft, 1=low, 2=medium, 3=high */
static long g_xp_duplex = 0;      /* 0=simplex, 1=vertical, 2=horizontal */
static long g_xp_collate = 0;     /* 0=off, 1=on */
static long g_xp_colormode = 2;   /* 1=mono, 2=color */
static long g_xp_pages = 0;       /* 0=all */
static long g_xp_paper = 1;        /* DMPAPER_LETTER */
static long g_xp_tray = 0;         /* DMBIN_DEFAULT */
static long g_xp_cell_x = 0;
static long g_xp_cell_y = 0;
static int g_xp_pen_width = 1;
static int g_xp_pen_style = 0; /* PS_SOLID */

static void xp_ensure_pen(void) {
    if (g_xp_pen) { DeleteObject(g_xp_pen); }
    g_xp_pen = (void*)((uintptr_t)CreatePen(g_xp_pen_style, g_xp_pen_width, (unsigned long)(unsigned int)g_xp_color));
    if (g_xp_dc && g_xp_pen) SelectObject(g_xp_dc, g_xp_pen);
}

int pb_xprint_cancel(void) {
    /* AbortDoc on printer DC; noop on screen DC */
    return 1;
}

int pb_xprint_formfeed(void) {
    /* EndPage + StartPage on printer DC; noop on screen DC */
    g_xp_pos_x = 0;
    g_xp_pos_y = 0;
    return 1;
}

int pb_xprint_line(long x1, long y1, long x2, long y2) {
    if (!g_xp_dc) return 0;
    xp_ensure_pen();
    MoveToEx(g_xp_dc, x1, y1, 0);
    LineTo(g_xp_dc, x2, y2);
    g_xp_pos_x = x2;
    g_xp_pos_y = y2;
    return 1;
}

int pb_xprint_box(long x1, long y1, long x2, long y2) {
    if (!g_xp_dc) return 0;
    xp_ensure_pen();
    if (!g_xp_brush) {
        g_xp_brush = (void*)((uintptr_t)GetStockObject(5)); /* NULL_BRUSH */
        SelectObject(g_xp_dc, g_xp_brush);
    }
    Rectangle(g_xp_dc, x1, y1, x2, y2);
    return 1;
}

int pb_xprint_width(long w) {
    g_xp_pen_width = w > 0 ? w : 1;
    xp_ensure_pen();
    return 1;
}

int pb_xprint_style(long st) {
    g_xp_pen_style = st;
    xp_ensure_pen();
    return 1;
}

int pb_xprint_set_color(long c) {
    g_xp_color = c;
    xp_ensure_pen();
    if (g_xp_dc) SetTextColor(g_xp_dc, (unsigned long)(unsigned int)c);
    return 1;
}

/* XPRINT COLOR r[, g[, b]] - RGB component form (batch 118) */
int pb_xprint_set_color_rgb(int rv, int gv, int bv) {
    if (rv < 0) rv = 0; if (gv < 0) gv = 0; if (bv < 0) bv = 0;
    if (rv > 255) rv = 255; if (gv > 255) gv = 255; if (bv > 255) bv = 255;
    long c = (long)(((unsigned)rv & 0xFF) | (((unsigned)gv & 0xFF) << 8) | (((unsigned)bv & 0xFF) << 16));
    return pb_xprint_set_color(c);
}

int pb_xprint_get_color(long* c) {
    *c = g_xp_color;
    return 1;
}

int pb_xprint_set_pos(long x, long y) {
    g_xp_pos_x = x;
    g_xp_pos_y = y;
    return 1;
}

int pb_xprint_get_pos(long* x, long* y) {
    *x = g_xp_pos_x;
    *y = g_xp_pos_y;
    return 1;
}

int pb_xprint_set_pixel(long x, long y, long c) {
    if (!g_xp_dc) return 0;
    SetPixel(g_xp_dc, x, y, (unsigned long)(unsigned int)c);
    return 1;
}

int pb_xprint_get_pixel(long x, long y, long* c) {
    if (!g_xp_dc) { *c = 0; return 0; }
    *c = (long)GetPixel(g_xp_dc, x, y);
    return 1;
}

int pb_xprint_set_textalign(long a) {
    g_xp_textalign = a;
    if (g_xp_dc) SetTextAlign(g_xp_dc, (unsigned int)a);
    return 1;
}

int pb_xprint_get_textalign(long* a) {
    *a = g_xp_textalign;
    return 1;
}

int pb_xprint_get_attach(char** v) {
    /* target is a PB STRING variable (BSTR slot); writing a raw LONG here
       corrupts the 8-byte slot and crashes the next dereference (0xC0000005
       on batch069/070) - write "1"/"0" as a proper BSTR instead */
    if (!v) return 0;
    *v = (char*)SysAllocStringByteLen(g_xp_dc ? "1" : "0", 1);
    return 1;
}

int pb_xprint_print_str(char* s) {
    if (!g_xp_dc || !s) return 0;
    int len = 0;
    while (s[len]) len++;
    TextOutA(g_xp_dc, g_xp_pos_x, g_xp_pos_y, s, len);
    /* advance position by approximate text width */
    int sz[2] = {0, 0};
    if (GetTextExtentPoint32A(g_xp_dc, s, len, (void*)sz)) {
        g_xp_pos_x += sz[0];
    }
    return 1;
}
/* === Batch 70: XPRINT ARC/ELLIPSE/PIE/SET FONT/MIX/STRETCHMODE === */
int pb_xprint_arc(int x1, int y1, int x2, int y2, double start, double end) {
    g_xp_arc_x1 = x1; g_xp_arc_y1 = y1; g_xp_arc_x2 = x2; g_xp_arc_y2 = y2;
    g_xp_arc_a = start; g_xp_arc_b = end;
    if (!g_xp_dc) return 0;
    xp_ensure_pen();
    /* batch 234: official form is (x1,y1)-(x2,y2), startRadians, endRadians.
       No printer attached (g_xp_dc==0) => record-only, as before. */
    return Arc(g_xp_dc, x1, y1, x2, y2, x1, y1, x2, y2);
}
int pb_xprint_ellipse(int x1, int y1, int x2, int y2) {
    if (!g_xp_dc) return 0;
    xp_ensure_pen();
    SelectObject(g_xp_dc, GetStockObject(5)); /* NULL_BRUSH */
    return Ellipse(g_xp_dc, x1, y1, x2, y2);
}
int pb_xprint_pie(int x1, int y1, int x2, int y2, double start, double end) {
    g_xp_arc_x1 = x1; g_xp_arc_y1 = y1; g_xp_arc_x2 = x2; g_xp_arc_y2 = y2;
    g_xp_arc_a = start; g_xp_arc_b = end;
    if (!g_xp_dc) return 0;
    xp_ensure_pen();
    SelectObject(g_xp_dc, GetStockObject(5)); /* NULL_BRUSH */
    return Pie(g_xp_dc, x1, y1, x2, y2, x1, y1, x2, y2);
}
int pb_xprint_set_font(long hFont) {
    if (!hFont) return 0;
    if (g_xp_font && g_xp_font != (void*)((uintptr_t)GetStockObject(17)))
        DeleteObject(g_xp_font);
    g_xp_font = (void*)(uintptr_t)hFont;
    if (g_xp_dc) SelectObject(g_xp_dc, g_xp_font);
    return 1;
}
int pb_xprint_set_mix(int mode) {
    if (!g_xp_dc) return 0;
    return SetROP2(g_xp_dc, mode);
}
int pb_xprint_get_mix(long* out) {
    if (!g_xp_dc || !out) return 0;
    *out = GetROP2(g_xp_dc);
    return 1;
}
int pb_xprint_set_stretchmode(int mode) {
    if (!g_xp_dc) return 0;
    return SetStretchBltMode(g_xp_dc, mode);
}
int pb_xprint_get_stretchmode(long* out) {
    if (!g_xp_dc || !out) return 0;
    *out = GetStretchBltMode(g_xp_dc);
    return 1;
}
/* === Batch 71: XPRINT TEXT SIZE / GET CLIENT / GET CANVAS / WRAP / WORDWRAP / OVERLAP === */
int pb_xprint_text_size(char* s, long* w, long* h) {
    if (!g_xp_dc || !s || !w || !h) return 0;
    int len = 0;
    while (s[len]) len++;
    int sz[2] = {0, 0};
    if (!GetTextExtentPoint32A(g_xp_dc, s, len, (void*)sz)) return 0;
    *w = sz[0];
    *h = sz[1];
    return 1;
}
int pb_xprint_get_client(long* w, long* h) {
    if (!g_xp_dc || !w || !h) return 0;
    *w = GetDeviceCaps(g_xp_dc, 8);  /* HORZRES */
    *h = GetDeviceCaps(g_xp_dc, 10); /* VERTRES */
    return 1;
}
int pb_xprint_get_canvas(long* w, long* h) {
    return pb_xprint_get_client(w, h);
}
int pb_xprint_set_wrap(long v) { g_xp_wrap = v; return 1; }
int pb_xprint_get_wrap(long* out) { if (!out) return 0; *out = g_xp_wrap; return 1; }
int pb_xprint_set_wordwrap(long v) { g_xp_wordwrap = v; return 1; }
int pb_xprint_get_wordwrap(long* out) { if (!out) return 0; *out = g_xp_wordwrap; return 1; }
int pb_xprint_set_overlap(long v) { g_xp_overlap = v; return 1; }
int pb_xprint_get_overlap(long* out) { if (!out) return 0; *out = g_xp_overlap; return 1; }
/* === Batch 72: XPRINT CLIP/SCALE/LINES/CELL SIZE/CHR SIZE/COPY === */
int pb_xprint_set_clip(int x1, int y1, int x2, int y2) {
    if (!g_xp_dc) return 0;
    return IntersectClipRect(g_xp_dc, x1, y1, x2, y2);
}
int pb_xprint_get_clip(long* w, long* h) {
    if (!g_xp_dc || !w || !h) return 0;
    int rc[4] = {0, 0, 0, 0};
    int r = GetClipBox(g_xp_dc, (void*)rc);
    *w = rc[2] - rc[0]; *h = rc[3] - rc[1];
    return r;
}
int pb_xprint_scale(int x1, int y1, int x2, int y2) {
    g_xp_scale_x1 = x1; g_xp_scale_y1 = y1; g_xp_scale_x2 = x2; g_xp_scale_y2 = y2;
    g_xp_scale_w = x2 - x1;
    g_xp_scale_h = y2 - y1;
    if (!g_xp_dc) return 0;
    SetMapMode(g_xp_dc, 8); /* MM_ANISOTROPIC */
    SetWindowExtEx(g_xp_dc, x2 - x1, y2 - y1, 0);
    int vw = GetDeviceCaps(g_xp_dc, 8);  /* HORZRES */
    int vh = GetDeviceCaps(g_xp_dc, 10); /* VERTRES */
    SetViewportExtEx(g_xp_dc, vw, vh, 0);
    return 1;
}
int pb_xprint_get_scale(long* x1, long* y1, long* x2, long* y2) {
    if (!x1 || !y1 || !x2 || !y2) return 0;
    *x1 = g_xp_scale_x1;
    *y1 = g_xp_scale_y1;
    *x2 = g_xp_scale_x2;
    *y2 = g_xp_scale_y2;
    return 1;
}
int pb_xprint_get_lines(long* out) {
    if (!g_xp_dc || !out) return 0;
    int ch = GetDeviceCaps(g_xp_dc, 10); /* VERTRES */
    int sz[2] = {0, 0};
    GetTextExtentPoint32A(g_xp_dc, "W", 1, (void*)sz);
    *out = (sz[1] > 0) ? ch / sz[1] : 0;
    return 1;
}
int pb_xprint_cell_size(long* w, long* h) {
    if (!g_xp_dc || !w || !h) return 0;
    int sz[2] = {0, 0};
    if (!GetTextExtentPoint32A(g_xp_dc, "W", 1, (void*)sz)) return 0;
    *w = sz[0]; *h = sz[1];
    return 1;
}
int pb_xprint_chr_size(long* w, long* h) {
    return pb_xprint_cell_size(w, h);
}
int pb_xprint_copy(int dx, int dy, int w, int h, int sx, int sy) {
    if (!g_xp_dc) return 0;
    return BitBlt(g_xp_dc, dx, dy, w, h, g_xp_dc, sx, sy, 0x00CC0020); /* SRCCOPY */
}
/* === Batch 73: XPRINT POLYGON / POLYLINE === */
int pb_xprint_polygon(int* pts, int count, unsigned long col) {
    if (!g_xp_dc || count < 3 || !pts) return 0;
    xp_ensure_pen();
    SelectObject(g_xp_dc, GetStockObject(5)); /* NULL_BRUSH */
    return Polygon(g_xp_dc, (const long*)pts, count);
}
int pb_xprint_polyline(int* pts, int count, unsigned long col) {
    if (!g_xp_dc || count < 2 || !pts) return 0;
    xp_ensure_pen();
    return Polyline(g_xp_dc, (const void*)pts, count);
}
/* === Batch 74: XPRINT printer property SET/GET === */
int pb_xprint_set_copies(int v) { g_xp_copies = v; return 1; }
int pb_xprint_get_copies(long* out) { if (!out) return 0; *out = g_xp_copies; return 1; }
int pb_xprint_set_orientation(int v) { g_xp_orientation = v; return 1; }
int pb_xprint_get_orientation(long* out) { if (!out) return 0; *out = g_xp_orientation; return 1; }
int pb_xprint_set_quality(int v) { g_xp_quality = v; return 1; }
int pb_xprint_get_quality(long* out) { if (!out) return 0; *out = g_xp_quality; return 1; }
int pb_xprint_set_duplex(int v) { g_xp_duplex = v; return 1; }
int pb_xprint_get_duplex(long* out) { if (!out) return 0; *out = g_xp_duplex; return 1; }
int pb_xprint_set_collate(int v) { g_xp_collate = v; return 1; }
int pb_xprint_get_collate(long* out) { if (!out) return 0; *out = g_xp_collate; return 1; }
int pb_xprint_set_colormode(int v) { g_xp_colormode = v; return 1; }
int pb_xprint_get_colormode(long* out) { if (!out) return 0; *out = g_xp_colormode; return 1; }
int pb_xprint_set_pages(int v) { g_xp_pages = v; return 1; }
int pb_xprint_get_pages(long* out) { if (!out) return 0; *out = g_xp_pages; return 1; }
/* === Batch 75: XPRINT CELL/SELECTION/PAPER/TRAY + RESOURCE SAVE FILE === */
int pb_xprint_set_cell(int x, int y) { g_xp_cell_x = x; g_xp_cell_y = y; return 1; }
int pb_xprint_get_cell(long* x, long* y) { if (!x || !y) return 0; *x = g_xp_cell_x; *y = g_xp_cell_y; return 1; }
int pb_xprint_get_selection(char** out) { if (!out) return 0; *out = (char*)SysAllocStringByteLen("", 0); return 1; }
int pb_xprint_set_paper(int v) { g_xp_paper = v; return 1; }
int pb_xprint_get_paper(long* out) { if (!out) return 0; *out = g_xp_paper; return 1; }
int pb_xprint_set_tray(int v) { g_xp_tray = v; return 1; }
int pb_xprint_get_tray(long* out) { if (!out) return 0; *out = g_xp_tray; return 1; }
int pb_resource_save_file(const char* resname, const char* filename) {
    if (!resname || !filename) return 0;

    /* Real resource extraction via Win32 API */
    void* hMod = GetModuleHandleA(NULL);
    if (!hMod) return 0;

    /* Find the resource (RCDATA type = user-defined resource) */
    void* hRes = FindResourceA(hMod, resname, RT_RCDATA);
    if (!hRes) return 0;

    unsigned long size = SizeofResource(hMod, hRes);
    if (size == 0) return 0;

    void* hData = LoadResource(hMod, hRes);
    if (!hData) return 0;

    void* pData = LockResource(hData);
    if (!pData) return 0;

    /* Write to file */
    FILE* f = fopen(filename, "wb");
    if (!f) return 0;
    fwrite(pData, 1, size, f);
    fclose(f);

    return 1;
}

/* R2 RESOURCE$ (CHW RESOURCE$_function.htm): return embedded resource data as
 * a BSTR. type_str selects RCDATA (raw bytes) or STRING (string table,
 * wide->ANSI via LoadStringA); anything else defaults to RCDATA. resid may
 * be a numeric 0..65535 ID (all-digit string) or an alphanumeric label.
 * CHW: "If the ID you request is not present, a nul (zero-length) string
 * is returned." */
static int pb_resource_type_code(const char* t) {
    if (!t) return 0;
    /* case-insensitive compare, short strings */
    char a = (char)toupper((unsigned char)t[0]);
    if (a == 'S') return 1; /* STRING */
    return 0;               /* RCDATA / default */
}
static const char* pb_res_make_id(const char* s) {
    if (!s || !*s) return (const char*)0;
    int all_digits = 1;
    for (const char* p = s; *p; p++) {
        if (*p < '0' || *p > '9') { all_digits = 0; break; }
    }
    if (all_digits) {
        unsigned long id = strtoul(s, NULL, 10);
        return MAKEINTRESOURCEA(id);
    }
    return s;
}
char* pb_resource_str(const char* type_str, const char* resid) {
    void* hMod = GetModuleHandleA(NULL);
    if (!hMod || !resid || !*resid) return pb_bstr_alloc("", 0);
    int type = pb_resource_type_code(type_str);
    if (type == 1) {
        /* STRING table: CHW says only integral numeric IDs are accepted. */
        unsigned long id = 0;
        int digits = 1;
        for (const char* p = resid; *p; p++) {
            if (*p < '0' || *p > '9') { digits = 0; break; }
        }
        if (!digits) return pb_bstr_alloc("", 0);
        id = strtoul(resid, NULL, 10);
        char buf[4096];
        int n = LoadStringA(hMod, (unsigned int)id, buf, (int)sizeof(buf));
        if (n <= 0) return pb_bstr_alloc("", 0);
        return pb_bstr_alloc(buf, (unsigned)n);
    }
    /* RCDATA: raw bytes, byte-for-byte as compiled. */
    const char* name = pb_res_make_id(resid);
    void* hRes = FindResourceA(hMod, name, RT_RCDATA);
    if (!hRes) return pb_bstr_alloc("", 0);
    unsigned long sz = SizeofResource(hMod, hRes);
    void* hData = LoadResource(hMod, hRes);
    if (!hData || sz == 0) return pb_bstr_alloc("", 0);
    void* p = LockResource(hData);
    if (!p) return pb_bstr_alloc("", 0);
    return pb_bstr_alloc((const char*)p, sz);
}
/* Numeric-id entry point: RESOURCE$(RCDATA, 999) / RESOURCE$(STRING, id%)
 * where the id is a numeric expression, not a string literal. Avoids the
 * crash where an i32 id was reinterpreted as a const char*. */
char* pb_resource_str_num(const char* type_str, int resid_num) {
    void* hMod = GetModuleHandleA(NULL);
    if (!hMod) return pb_bstr_alloc("", 0);
    if (resid_num < 0) return pb_bstr_alloc("", 0);
    int type = pb_resource_type_code(type_str);
    if (type == 1) {
        char buf[4096];
        int n = LoadStringA(hMod, (unsigned int)resid_num, buf, (int)sizeof(buf));
        if (n <= 0) return pb_bstr_alloc("", 0);
        return pb_bstr_alloc(buf, (unsigned)n);
    }
    void* hRes = FindResourceA(hMod, MAKEINTRESOURCEA((unsigned int)resid_num), RT_RCDATA);
    if (!hRes) return pb_bstr_alloc("", 0);
    unsigned long sz = SizeofResource(hMod, hRes);
    void* hData = LoadResource(hMod, hRes);
    if (!hData || sz == 0) return pb_bstr_alloc("", 0);
    void* p = LockResource(hData);
    if (!p) return pb_bstr_alloc("", 0);
    return pb_bstr_alloc((const char*)p, sz);
}
/* === Batch 76: remaining XPRINT statements (completes XPRINT family) === */
/* XPRINT papers/trays: targets are PB STRING variables (BSTR slots), same
   crash hazard as pb_xprint_get_attach - write BSTR strings, not raw LONGs */
int pb_xprint_get_papers(char** out) { if (!out) return 0; *out = (char*)SysAllocStringByteLen("1", 1); return 1; } /* A4 default */
int pb_xprint_get_trays(char** out) { if (!out) return 0; *out = (char*)SysAllocStringByteLen("1", 1); return 1; }  /* one tray default */

/* XPRINT preview/render: track preview mode state */
static int g_xp_preview_mode = 0;
int pb_xprint_preview(int mode) { g_xp_preview_mode = mode; return 1; }
int pb_xprint_preview_close(void) { g_xp_preview_mode = 0; return 1; }

int pb_xprint_render(void) {
    /* In screen DC mode, render = flush to screen (already done per op) */
    g_xp_preview_mode = 0;
    return 1;
}

/* XPRINT split: track split region */
static int g_xp_split_x = 0, g_xp_split_y = 0, g_xp_split_w = 0, g_xp_split_h = 0;
int pb_xprint_split(char* src, int len, char** p1, char** p2) {
    if (!src || !p1 || !p2) return 0;
    int total = 0;
    while (src[total]) total++;
    int cut = (len < 0) ? 0 : (len > total ? total : len);
    *p1 = (char*)SysAllocStringByteLen(src, cut);
    *p2 = (char*)SysAllocStringByteLen(src + cut, total - cut);
    return 1;
}
int pb_xprint_stretch(int dx, int dy, int dw, int dh, int sx, int sy, int sw, int sh) {
    if (!g_xp_dc) return 0;
    return StretchBlt(g_xp_dc, dx, dy, dw, dh, g_xp_dc, sx, sy, sw, sh, 0x00CC0020); /* SRCCOPY */
}
/* XPRINT imagelist: track imagelist handle */
static void* g_xp_imagelist = NULL;
int pb_xprint_imagelist(int op, int arg1, int arg2) {
    /* op: 0=attach, 1=detach */
    if (op == 0) {
        g_xp_imagelist = (void*)(long long)arg1;
    } else if (op == 1) {
        g_xp_imagelist = NULL;
    }
    return 1;
}
/* === Batch 77: TCP/UDP NOTIFY + PROGRESSBAR + HEADER + ARRAY SELECT/TAGARRAY === */
/* TCP/UDP NOTIFY: save event mask per socket (async mode flag) */
static int g_socket_event_mask[256] = {0};

int pb_tcp_notify(int socket, int eventmask) {
    if (socket < 0 || socket >= 256) return 0;
    g_socket_event_mask[socket] = eventmask;
    return 1;
}

int pb_udp_notify(int socket, int eventmask) {
    if (socket < 0 || socket >= 256) return 0;
    g_socket_event_mask[socket] = eventmask;
    return 1;
}

/* PROGRESSBAR: Win32 progress bar control (comctl32) */
/* lParam and wParam are pointer-width on x64 and 32-bit on i686.  On i686 the
   __stdcall decorated name (_SendMessageA@16) counts 4+4+4+4 bytes, so
   pb_wparam_t has to stay 'unsigned int' there: widening it unconditionally
   would ask the linker for _SendMessageA@20 and break the 32-bit build. */
#if defined(_WIN64)
typedef unsigned long long pb_lparam_t;
typedef unsigned long long pb_wparam_t;
#else
typedef unsigned int pb_lparam_t;
typedef unsigned int pb_wparam_t;
#endif
__declspec(dllimport) void* __stdcall SendMessageA(void* hWnd, unsigned int Msg, pb_wparam_t wParam, pb_lparam_t lParam);
__declspec(dllimport) void* __stdcall GetDlgItem(void* hDlg, int nId);
__declspec(dllimport) unsigned long __stdcall RegisterClassExA(const void* lpwcx);
__declspec(dllimport) void* __stdcall CreateWindowExA(unsigned long dwExStyle, const char* lpClassName, const char* lpWindowName, unsigned long dwStyle, int x, int y, int nWidth, int nHeight, void* hWndParent, void* hMenu, void* hInstance, void* lpParam);
__declspec(dllimport) int __stdcall ShowWindow(void* hWnd, int nCmdShow);
__declspec(dllimport) int __stdcall UpdateWindow(void* hWnd);
#ifdef _WIN64
__declspec(dllimport) long long __stdcall DefWindowProcA(void* hWnd, unsigned int Msg, unsigned long long wParam, unsigned long long lParam);
#else
__declspec(dllimport) long long __stdcall DefWindowProcA(void* hWnd, unsigned int Msg, unsigned int wParam, unsigned int lParam);
#endif
__declspec(dllimport) int __stdcall GetMessageA(void* lpMsg, void* hWnd, unsigned int wMsgFilterMin, unsigned int wMsgFilterMax);
__declspec(dllimport) int __stdcall TranslateMessage(const void* lpMsg);
__declspec(dllimport) unsigned long __stdcall DispatchMessageA(const void* lpMsg);
__declspec(dllimport) int __stdcall DestroyWindow(void* hWnd);
__declspec(dllimport) void __stdcall PostQuitMessage(int nExitCode);
__declspec(dllimport) int __stdcall IsDialogMessageA(void* hDlg, const void* lpMsg);

#define PBM_SETPOS   0x0402
#define PBM_SETRANGE32 0x0406

/* HEADER: Win32 header control.
 * The 4-arg pb_header() helper (and its HD_ITEMA typedef) was removed in batch
 * 163: its only caller was the unreachable codegen arm "HEADER_CTRL", and it
 * treated the dialog handle as the control HWND.  The official PB syntax is
 * served by pb_header_send / _get_count / _get_item / _set_item below. */
#define HDM_FIRST 0x1200
/* PROGRESSBAR / HEADER statements: official PB syntax addresses the control by
 * (owner window, control id), so resolve the real HWND with GetDlgItem.  All
 * integer parameters are long long to match the I64 LLVM declarations. */
#define PBM_SETSTEP   0x0404
#define PBM_STEPIT    0x0405
#define PBM_DELTAPOS  0x0403
#define PBM_GETPOS    0x0408
#define PBM_GETRANGE  0x0407
#define HDM_GETITEMCOUNT (HDM_FIRST + 0)
#define HDM_GETITEMA     (HDM_FIRST + 3)
#define HDM_SETITEMA     (HDM_FIRST + 4)

typedef struct { int iLow; int iHigh; } PB_PBRANGE;

static void* pb_pb_hwnd(void* hDlg, long long id) {
    if (!hDlg) return 0;
    return GetDlgItem(hDlg, (int)id);
}

long long pb_progressbar_set_range(void* hDlg, long long id, long long lo, long long hi) {
    void* h = pb_pb_hwnd(hDlg, id);
    if (!h) return 0;
    SendMessageA(h, PBM_SETRANGE32, (unsigned int)lo, (unsigned long long)hi);
    return 1;
}

long long pb_progressbar_set_pos(void* hDlg, long long id, long long pos) {
    void* h = pb_pb_hwnd(hDlg, id);
    if (!h) return 0;
    SendMessageA(h, PBM_SETPOS, (unsigned int)pos, 0);
    return 1;
}

long long pb_progressbar_set_step(void* hDlg, long long id, long long step) {
    void* h = pb_pb_hwnd(hDlg, id);
    if (!h) return 0;
    SendMessageA(h, PBM_SETSTEP, (unsigned int)step, 0);
    return 1;
}

long long pb_progressbar_step(void* hDlg, long long id, long long inc) {
    void* h = pb_pb_hwnd(hDlg, id);
    if (!h) return 0;
    if (inc) SendMessageA(h, PBM_DELTAPOS, (unsigned int)inc, 0);
    else     SendMessageA(h, PBM_STEPIT, 0, 0);
    return 1;
}

long long pb_progressbar_get_pos(void* hDlg, long long id) {
    void* h = pb_pb_hwnd(hDlg, id);
    if (!h) return -1;
    return (long long)SendMessageA(h, PBM_GETPOS, 0, 0);
}

static long long pb_progressbar_range_part(void* hDlg, long long id, int want_high) {
    PB_PBRANGE r;
    void* h = pb_pb_hwnd(hDlg, id);
    if (!h) return -1;
    r.iLow = 0; r.iHigh = 0;
    SendMessageA(h, PBM_GETRANGE, 1, (unsigned long long)&r);
    return want_high ? (long long)r.iHigh : (long long)r.iLow;
}

long long pb_progressbar_get_lo(void* hDlg, long long id) {
    return pb_progressbar_range_part(hDlg, id, 0);
}

long long pb_progressbar_get_hi(void* hDlg, long long id) {
    return pb_progressbar_range_part(hDlg, id, 1);
}

/* HEADER GET ITEM / SET ITEM take a 1-based Index& per the official help
   ("1=first, 2=second, ..."); the underlying HDM_* messages are 0-based. */
static int pb_hdr_idx(long long pb_index) {
    long long i = pb_index - 1;
    return (int)(i < 0 ? 0 : i);
}

long long pb_header_send(void* hWin, long long id, long long msg, long long wparam, long long lparam) {
    void* h = pb_pb_hwnd(hWin, id);
    if (!h) return 0;
    return (long long)SendMessageA(h, (unsigned int)msg, (pb_wparam_t)wparam, (pb_lparam_t)lparam);
}

long long pb_header_get_count(void* hWin, long long id) {
    void* h = pb_pb_hwnd(hWin, id);
    if (!h) return -1;
    return (long long)SendMessageA(h, HDM_GETITEMCOUNT, 0, 0);
}

long long pb_header_get_item(void* hWin, long long id, long long index, long long itemPtr) {
    void* h = pb_pb_hwnd(hWin, id);
    if (!h || !itemPtr) return 0;
    /* The official HEADER help says Index& is 1-based ("1=first,")
       while HDM_GETITEMA is 0-based, so convert (batch 171). */
    return (long long)SendMessageA(h, HDM_GETITEMA,
                                   (unsigned int)pb_hdr_idx(index), (pb_lparam_t)itemPtr);
}

long long pb_header_set_item(void* hWin, long long id, long long index, long long itemPtr) {
    void* h = pb_pb_hwnd(hWin, id);
    if (!h || !itemPtr) return 0;
    return (long long)SendMessageA(h, HDM_SETITEMA,
                                   (unsigned int)pb_hdr_idx(index), (pb_lparam_t)itemPtr);
}

/* ===================================================================
   TOOLBAR / STATUSBAR  (batch 164)
   -------------------------------------------------------------------
   Constants below are spelled out from commctrl.h and the PowerBASIC
   WINAPI headers; this runtime never includes windows.h.  Cross-checked
   against C:\PBWin10\WINAPI\CommCtrl.inc and compiler-built-in.inc, so
   the %BTNS_* / %TBSTATE_* / %SBT_* / %CCS_* values a PB program passes
   in are exactly the Win32 values used here.
   =================================================================== */

#define PB_TOOLBARCLASSNAME   "ToolbarWindow32"
#define PB_STATUSCLASSNAME    "msctls_statusbar32"

/* Toolbar messages - WM_USER + n */
#define PB_TB_ENABLED         (0x0400 + 16)
#define PB_TB_SETSTATE        (0x0400 + 17)
#define PB_TB_GETSTATE        (0x0400 + 18)
#define PB_TB_ADDBUTTONSA     (0x0400 + 20)
#define PB_TB_INSERTBUTTONA   (0x0400 + 21)
#define PB_TB_DELETEBUTTON    (0x0400 + 22)
#define PB_TB_GETBUTTON       (0x0400 + 23)
#define PB_TB_BUTTONCOUNT     (0x0400 + 24)
#define PB_TB_COMMANDTOINDEX  (0x0400 + 25)
#define PB_TB_ADDSTRINGA      (0x0400 + 28)
#define PB_TB_SETIMAGELIST    (0x0400 + 48)
#define PB_TB_BUTTONSTRUCTSIZE (0x0400 + 30)

/* TBSTYLE_* / BTNS_* (identical values in CommCtrl.inc) */
#define PB_TBSTYLE_SEP        0x0001
#define PB_TBSTYLE_ENABLED    0x0004

/* TBSTATE_* */
#define PB_TBSTATE_ENABLED    0x0004

/* Statusbar messages - WM_USER + n */
#define PB_SB_SETTEXTA        (0x0400 + 1)
#define PB_SB_SETPARTS        (0x0400 + 5)

/* Common-control styles */
#define PB_CCS_TOP            0x00000001
#define PB_CCS_BOTTOM         0x00000003

/* TBBUTTON - the two reserved BYTE slots of the 32-bit layout become six
   on x64 so that the pointer-sized dwData stays 8-byte aligned. */
#ifdef _WIN64
typedef struct {
    int                iBitmap;
    int                idCommand;
    unsigned char      fsState;
    unsigned char      fsStyle;
    unsigned char      bReserved[6];
    unsigned long long dwData;
    long long          iString;
} PB_TBBUTTON;
#else
typedef struct {
    int                iBitmap;
    int                idCommand;
    unsigned char      fsState;
    unsigned char      fsStyle;
    unsigned char      bReserved[2];
    unsigned long      dwData;
    long               iString;
} PB_TBBUTTON;
#endif

/* TB_BUTTONSTRUCTSIZE must be sent exactly once per control, before the
   first TB_ADDBUTTONS.  Sending it again makes comctl32 re-create its
   button array, which leaves the earlier entries unaddressable: the
   control still reports the old TB_BUTTONCOUNT but TB_GETSTATE starts
   returning -1 for every index above the last insert. */
static void* pb_toolbar_inited[64];

static void pb_toolbar_init(void* h) {
    int i;
    for (i = 0; i < 64; i++) {
        if (pb_toolbar_inited[i] == h) return;
    }
    SendMessageA(h, (unsigned int)PB_TB_BUTTONSTRUCTSIZE,
                 (unsigned int)sizeof(PB_TBBUTTON), 0);
    for (i = 0; i < 64; i++) {
        if (pb_toolbar_inited[i] == 0) {
            pb_toolbar_inited[i] = h;
            return;
        }
    }
}

/* Shared tail for ADD BUTTON / ADD SEPARATOR: `at` is a 1-based position
   (0 = append). */
static long long pb_toolbar_insert(void* h, const PB_TBBUTTON* tb, long long at) {
    if (at > 0) {
        return (long long)SendMessageA(h, (unsigned int)PB_TB_INSERTBUTTONA,
                                       (unsigned int)(at - 1), (pb_lparam_t)tb);
    }
    return (long long)SendMessageA(h, (unsigned int)PB_TB_ADDBUTTONSA, 1, (pb_lparam_t)tb);
}

/* TOOLBAR ADD BUTTON hDlg, ID, image&, cmd&, style&, text$ [AT item&] */
long long pb_toolbar_add_button(void* hDlg, long long id, long long image,
                                long long cmd, long long style,
                                const char* text, long long at) {
    void* h = pb_pb_hwnd(hDlg, id);
    PB_TBBUTTON tb;
    long long stridx = -1;      /* -1 = "this button has no text" */
    if (!h) return 0;
    pb_toolbar_init(h);
    memset(&tb, 0, sizeof(tb));
    tb.iBitmap   = (int)image;
    tb.idCommand = (int)cmd;
    tb.fsStyle   = (unsigned char)(style & 0xFF);
    tb.fsState   = (unsigned char)PB_TBSTATE_ENABLED;
    if (text && *text) {
        /* TB_ADDSTRING copies the text into the control's own string pool;
           iString must be that index, not a caller-owned pointer. */
        stridx = (long long)SendMessageA(h, (unsigned int)PB_TB_ADDSTRINGA,
                                         0, (pb_lparam_t)text);
        /* A failed add leaves the pool empty, and an iString of 0 against an
           empty pool makes comctl32 treat the button as unaddressable
           (TB_GETSTATE returns -1 for it).  Fall back to -1 = "no text". */
        if (stridx < 0) stridx = -1;
    }
    tb.iString = (long long)stridx;
    return pb_toolbar_insert(h, &tb, at);
}

/* TOOLBAR ADD SEPARATOR hDlg, ID, size& [,cmd&] [AT item&]
   For separators iBitmap carries the width in pixels. */
long long pb_toolbar_add_separator(void* hDlg, long long id, long long size,
                                   long long cmd, long long at) {
    void* h = pb_pb_hwnd(hDlg, id);
    PB_TBBUTTON tb;
    if (!h) return 0;
    pb_toolbar_init(h);
    memset(&tb, 0, sizeof(tb));
    tb.iBitmap   = (int)size;
    tb.idCommand = (int)cmd;
    tb.fsStyle   = (unsigned char)PB_TBSTYLE_SEP;
    tb.fsState   = (unsigned char)PB_TBSTATE_ENABLED;
    return pb_toolbar_insert(h, &tb, at);
}

/* Resolve a 1-based position or a command id into a 0-based index. */
static long long pb_toolbar_index(void* h, long long item, long long bycmd) {
    if (bycmd) {
        long long idx = (long long)SendMessageA(h, (unsigned int)PB_TB_COMMANDTOINDEX,
                                                (unsigned int)item, 0);
        return idx;
    }
    return item - 1;
}

/* TOOLBAR GET/SET STATE address the button by COMMAND ID, not by index:
   comctl32's TB_GETSTATE / TB_SETSTATE take a command identifier in wParam,
   while TB_GETBUTTON / TB_DELETEBUTTON take an index.  Resolve a 1-based
   position to the button's idCommand through TB_GETBUTTON. */
static long long pb_toolbar_cmdid(void* h, long long item, long long bycmd) {
    PB_TBBUTTON tb;
    long long idx;
    if (bycmd) return item;
    idx = item - 1;
    if (idx < 0) return -1;
    memset(&tb, 0, sizeof(tb));
    if (!SendMessageA(h, (unsigned int)PB_TB_GETBUTTON, (unsigned int)idx,
                      (pb_lparam_t)&tb)) {
        return -1;
    }
    return (long long)tb.idCommand;
}

/* TOOLBAR DELETE BUTTON hDlg, id&, [BYCMD] item& */
long long pb_toolbar_delete_button(void* hDlg, long long id, long long item,
                                   long long bycmd) {
    void* h = pb_pb_hwnd(hDlg, id);
    long long idx;
    if (!h) return 0;
    idx = pb_toolbar_index(h, item, bycmd);
    if (idx < 0) return 0;
    return (long long)SendMessageA(h, (unsigned int)PB_TB_DELETEBUTTON,
                                   (unsigned int)idx, 0);
}

/* TOOLBAR GET STATE hDlg, ID, [BYCMD] item& TO datav& */
long long pb_toolbar_get_state(void* hDlg, long long id, long long item,
                               long long bycmd) {
    void* h = pb_pb_hwnd(hDlg, id);
    long long cmd;
    if (!h) return 0;
    cmd = pb_toolbar_cmdid(h, item, bycmd);
    if (cmd < 0) return 0;
    return (long long)SendMessageA(h, (unsigned int)PB_TB_GETSTATE,
                                   (unsigned int)cmd, 0);
}

/* TOOLBAR GET COUNT hDlg, ID TO datav& */
long long pb_toolbar_get_count(void* hDlg, long long id) {
    void* h = pb_pb_hwnd(hDlg, id);
    if (!h) return 0;
    return (long long)SendMessageA(h, (unsigned int)PB_TB_BUTTONCOUNT, 0, 0);
}

/* TOOLBAR SET STATE hDlg, ID, [BYCMD] item&, state& */
long long pb_toolbar_set_state(void* hDlg, long long id, long long item,
                               long long state, long long bycmd) {
    void* h = pb_pb_hwnd(hDlg, id);
    long long cmd;
    if (!h) return 0;
    cmd = pb_toolbar_cmdid(h, item, bycmd);
    if (cmd < 0) return 0;
    return (long long)SendMessageA(h, (unsigned int)PB_TB_SETSTATE,
                                   (unsigned int)cmd, (pb_lparam_t)state);
}

/* TOOLBAR SET IMAGELIST hDlg, ID, hLst, ListType&
   PB ListType: 1 = default images, 2 = disabled, 3 = hot.
   TB_SETIMAGELIST wParam: 0 = normal, 1 = hot, 2 = disabled. */
long long pb_toolbar_set_imagelist(void* hDlg, long long id, long long hLst,
                                   long long listtype) {
    void* h = pb_pb_hwnd(hDlg, id);
    unsigned int which;
    if (!h) return 0;
    if (listtype == 2) {
        which = 2;
    } else if (listtype == 3) {
        which = 1;
    } else {
        which = 0;
    }
    return (long long)SendMessageA(h, (unsigned int)PB_TB_SETIMAGELIST,
                                   which, (pb_lparam_t)hLst);
}

/* STATUSBAR SET PARTS hDlg, id&, x& [,x&...]  (max 32 parts) */
long long pb_statusbar_set_parts(void* hDlg, long long id, int* widths,
                                 long long count) {
    void* h = pb_pb_hwnd(hDlg, id);
    if (!h || !widths || count <= 0) return 0;
    if (count > 32) count = 32;
    return (long long)SendMessageA(h, (unsigned int)PB_SB_SETPARTS,
                                   (unsigned int)count, (pb_lparam_t)widths);
}

/* STATUSBAR SET TEXT hDlg, id&, item&, style&, text$
   SB_SETTEXT wParam: low byte = 0-based part index, high byte = style. */
long long pb_statusbar_set_text(void* hDlg, long long id, long long item,
                                long long style, const char* text) {
    void* h = pb_pb_hwnd(hDlg, id);
    unsigned int wp;
    if (!h) return 0;
    wp = (unsigned int)((item > 0 ? item - 1 : 0) & 0xFF)
       | (unsigned int)((style & 0xFF) << 8);
    return (long long)SendMessageA(h, (unsigned int)PB_SB_SETTEXTA,
                                   wp, (pb_lparam_t)text);
}
/* ARRAY SELECT state: tracks selected range for subsequent array operations */
static int g_array_sel_start = 0;
static int g_array_sel_end = 0;
static int g_array_sel_active = 0;

int pb_array_select(int* arr, int count, int start, int end) {
    (void)arr; (void)count;
    if (start < 1 || end < start) return 0;
    g_array_sel_start = start;
    g_array_sel_end = end;
    g_array_sel_active = 1;
    return 1;
}

/* ARRAY TAGARRAY state: stores tag array pointer */
static int* g_tag_arrays[256] = {0}; /* slot per array index */

int pb_array_tagarray(int* arr, int count, int* tag) {
    (void)count;
    /* Use array pointer as a simple hash into tag slot table */
    int slot = ((unsigned long long)arr / 8) % 256;
    g_tag_arrays[slot] = tag;
    return 1;
}

int pb_array_tagarray_erase(int* arr, int count) {
    (void)count;
    int slot = ((unsigned long long)arr / 8) % 256;
    g_tag_arrays[slot] = NULL;
    return 1;
}
/* === Batch 78: XPRINT GET MARGIN + DISPLAY common dialogs (6 statements) === */
static long g_xp_margin_left = 0, g_xp_margin_top = 0, g_xp_margin_right = 0, g_xp_margin_bottom = 0;
int pb_xprint_get_margin(long* left, long* top, long* right, long* bottom) {
    if (left) *left = g_xp_margin_left;
    if (top) *top = g_xp_margin_top;
    if (right) *right = g_xp_margin_right;
    if (bottom) *bottom = g_xp_margin_bottom;
    return 1;
}
int pb_display_openfile(const char* title, const char* filter, const char* initialdir, char** out) {
    if (!out) return 0;
    char file_buf[MAX_PATH] = {0};
    OPENFILENAMEA ofn = {0};
    ofn.lStructSize = sizeof(OPENFILENAMEA);
    ofn.lpstrFilter = filter ? filter : "All Files\0*.*\0";
    ofn.lpstrFile = file_buf;
    ofn.nMaxFile = MAX_PATH;
    ofn.lpstrInitialDir = initialdir;
    ofn.lpstrTitle = title;
    ofn.Flags = OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST;
    if (GetOpenFileNameA(&ofn)) {
        *out = SysAllocStringByteLen(file_buf, (unsigned int)strlen(file_buf));
        return 1;
    }
    *out = SysAllocStringByteLen("", 0); /* user cancelled */
    return 0;
}
int pb_display_savefile(const char* title, const char* filter, const char* initialdir, char** out) {
    if (!out) return 0;
    char file_buf[MAX_PATH] = {0};
    OPENFILENAMEA ofn = {0};
    ofn.lStructSize = sizeof(OPENFILENAMEA);
    ofn.lpstrFilter = filter ? filter : "All Files\0*.*\0";
    ofn.lpstrFile = file_buf;
    ofn.nMaxFile = MAX_PATH;
    ofn.lpstrInitialDir = initialdir;
    ofn.lpstrTitle = title;
    ofn.Flags = OFN_OVERWRITEPROMPT;
    if (GetSaveFileNameA(&ofn)) {
        *out = SysAllocStringByteLen(file_buf, (unsigned int)strlen(file_buf));
        return 1;
    }
    *out = SysAllocStringByteLen("", 0);
    return 0;
}
int pb_display_color(long* out_color) {
    if (!out_color) return 0;
    static unsigned char cust_colors[16] = {0};
    CHOOSECOLORA cc = {0};
    cc.lStructSize = sizeof(CHOOSECOLORA);
    cc.rgbResult = *out_color;
    cc.lpCustColors = cust_colors;
    cc.Flags = CC_RGBINIT;
    if (ChooseColorA(&cc)) {
        *out_color = cc.rgbResult;
        return 1;
    }
    return 0;
}
int pb_display_font(char** out_font) {
    if (!out_font) {
        *out_font = SysAllocStringByteLen("", 0);
        return 0;
    }
    static char logfont[256] = {0};
    CHOOSEFONTA cf = {0};
    cf.lStructSize = sizeof(CHOOSEFONTA);
    cf.lpLogFont = logfont;
    cf.Flags = CF_SCREENFONTS | CF_INITTOLOGFONTSTRUCT;
    if (ChooseFontA(&cf)) {
        /* Return face name from LOGFONTA (lfFaceName at offset 28) */
        char* face = logfont + 28;
        *out_font = SysAllocStringByteLen(face, (unsigned int)strlen(face));
        return 1;
    }
    *out_font = SysAllocStringByteLen("", 0);
    return 0;
}
int pb_display_browse(const char* title, const char* initialdir, char** out) {
    if (!out) return 0;
    char display_name[MAX_PATH] = {0};
    char path[MAX_PATH] = {0};
    BROWSEINFOA bi = {0};
    bi.hwndOwner = 0;
    bi.pidlRoot = 0;
    bi.pszDisplayName = display_name;
    bi.lpszTitle = title ? title : "Select Folder";
    bi.ulFlags = 0x00000001; /* BIF_RETURNONLYFSDIRS */
    void* pidl = SHBrowseForFolderA(&bi);
    if (pidl) {
        if (SHGetPathFromIDListA(pidl, path)) {
            *out = SysAllocStringByteLen(path, (unsigned int)strlen(path));
            return 1;
        }
    }
    *out = SysAllocStringByteLen("", 0); /* user cancelled */
    return 0;
}
/* === Batch 79: OOP foundation (CLASS/METHOD/OBJECT/INSTANCE) + ARRAY REDIM === */
/* OOP simplified model: objects are opaque pointers (void*), methods are regular functions
   with a hidden 'this' pointer as first arg. CLASS/END CLASS is a namespace block. */
static void* g_oop_last_instance = 0;
void* pb_class_create(const char* classname) {
    /* allocate a minimal object stub (16 bytes: vtable ptr + refcount + classname hash) */
    void* obj = calloc(1, 16);
    g_oop_last_instance = obj;
    return obj;
}
void pb_class_destroy(void* obj) { if (obj) free(obj); }
int pb_method_call(void* obj, const char* methodname) { return 1; } /* noop dispatch stub */
int pb_array_redim_incr(void* arr_ptr, int elem_size, int old_count, int increment) {
    /* simplified: report requested new size; actual realloc needs dynamic array model */
    return old_count + increment;
}
int pb_array_redim_decr(void* arr_ptr, int elem_size, int old_count, int decrement) {
    int n = old_count - decrement;
    return n < 0 ? 0 : n;
}
/* === Batch 80: OOP remaining 9 items (INTERFACE/EVENTS/RAISEEVENT/INSTANCE/OBJECT/LET) === */
/* INSTANCE var AS ClassName � create an object instance (simplified: allocate stub) */
void* pb_instance_create(const char* classname) {
    return pb_class_create(classname);
}
/* EVENTS / RAISEEVENT / EVENT SOURCE � simplified noop event model */
static int g_event_enabled = 1;
void pb_events_enable(int enable) { g_event_enabled = enable; }
/* --- batch 222: class-wide event bus (integer method ids) ---
 * A handler table is built once per EVENTS FROM site and attached to the
 * subscribing client object.  RAISEEVENT fires by method id across every
 * attached client.  Handlers are called as fn(client, arg); a zero-argument
 * handler simply ignores the second register (x64 calling convention). */
typedef int (*pb_event_fn)(void* client, int arg);
typedef struct pb_event_ent { int mname; pb_event_fn fn; } pb_event_ent;
typedef struct pb_event_tbl { int count; int cap; pb_event_ent* ents; } pb_event_tbl;
typedef struct pb_sub { void* client; pb_event_tbl* tbl; } pb_sub;
static pb_sub g_ev_subs[64];
static int g_ev_sub_count = 0;

void* pb_events_new_table(void) {
    pb_event_tbl* t = (pb_event_tbl*)calloc(1, sizeof(pb_event_tbl));
    return (void*)t;
}

/* pb_obj_new - allocate a zeroed heap block for a CLASS object (batch 264).
 * The size is the LLVM layout size of the CLASS's INSTANCE struct, computed
 * by the compiler with a GEP-on-null trick.  Returns NULL when the allocation
 * fails; PBWin semantics then report the object variable as NOTHING
 * (ISOBJECT() = 0).  Objects are not reference-counted in this fork; they
 * live for the process lifetime, matching the object model's simplified
 * <Class>_<Method>(this, ...) lowering. */
void* pb_obj_new(long long size) {
    // A CLASS with no INSTANCE members still produces a valid object
    // (size 0 layout): allocate at least one byte so the pointer is never
    // NULL (batch 264 -- ISOBJECT must see a real object).
    if (size < 1) size = 1;
    return calloc(1, (size_t)size);
}

/* ---- batch271/284: NEWCOM / GETCOM / ANYCOM — real COM instantiation ------
 * CHW How_do_you_create_an_object_:
 *   LET objvar = NEWCOM "ProgID"                  (CoCreateInstance, new)
 *   LET objvar = GETCOM "ProgID"                  (GetActiveObject, running)
 *   LET objvar = ANYCOM "ProgID"                  (active, else new)
 *   LET objvar = NEWCOM CLSID "{...}"             (GUID string, no ProgID)
 *   LET objvar = NEWCOM CLSID "{...}" LIB "dll"   (registry-free load)
 * The object variable holds an IDispatch*; ISOBJECT()/OBJPTR() work on the
 * raw pointer.  OBJRESULT() returns the HRESULT of the most recent COM
 * operation (0 = S_OK, negative = failure code).
 */
typedef struct _PB_GUID {
    unsigned long  Data1;
    unsigned short Data2;
    unsigned short Data3;
    unsigned char  Data4[8];
} PB_GUID;

__declspec(dllimport) int  __stdcall CoInitializeEx(void* pvReserved, unsigned long dwCoInit);
__declspec(dllimport) int  __stdcall CLSIDFromProgID(const wchar_t* lpszProgID, PB_GUID* lpclsid);
__declspec(dllimport) int  __stdcall CLSIDFromString(const wchar_t* lpsz, PB_GUID* lpclsid);
__declspec(dllimport) int  __stdcall CoCreateInstance(const PB_GUID* rclsid, void* pUnkOuter,
                                                      unsigned long dwClsContext, const PB_GUID* riid,
                                                      void** ppv);
__declspec(dllimport) int  __stdcall GetActiveObject(const PB_GUID* rclsid, void* pUnkReserved,
                                                     void** ppunk);
__declspec(dllimport) void* __stdcall LoadLibraryA(const char* lpFileName);
__declspec(dllimport) void* __stdcall GetProcAddress(void* hModule, const char* lpProcName);

static const PB_GUID PB_IID_IDISPATCH =
    {0x00020400, 0x0000, 0x0000, {0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46}};
static const PB_GUID PB_IID_ICLASSFACTORY =
    {0x00000001, 0x0000, 0x0000, {0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46}};

static int  pb_com_inited = 0;
static long pb_com_hresult = 0;

void pb_com_init(void) {
    if (!pb_com_inited) {
        CoInitializeEx(NULL, 0x2 /*COINIT_APARTMENTTHREADED*/);
        pb_com_inited = 1;
    }
}

/* OBJACTIVE(progid$) - CHW OBJACTIVE_function.htm: TRUE (-1) if an EXE-based
   COM object for the registered program ID is currently running (an active
   instance can be obtained via GetActiveObject), FALSE (0) otherwise.
   DLL/OCX-based servers always return FALSE because GetActiveObject has no
   running EXE instance to find.  The acquired active-object reference is
   released before returning. */
long pb_objactive(const char* progid) {
    PB_GUID cg;
    wchar_t wbuf[256];
    void* obj = NULL;
    if (!progid) return 0;
    pb_com_init();
    MultiByteToWideChar(0 /*CP_ACP*/, 0, progid, -1, (short*)wbuf, 256);
    if (CLSIDFromProgID(wbuf, &cg) != 0) return 0;
    if (GetActiveObject(&cg, NULL, &obj) != 0 || !obj) return 0;
    /* release the active-object reference we acquired */
    {
        typedef unsigned long (__stdcall* RELEASE_FN)(void*);
        RELEASE_FN rel = (RELEASE_FN)((void***)obj)[0][2];
        rel(obj);
    }
    return -1;  /* TRUE */
}

/* Registry-free COM: load the DLL, resolve DllGetClassObject, ask it for the
 * class factory, CreateInstance(IID_IDispatch).  Returns the IDispatch* or
 * NULL.  (CHW: "no need for the server to be registered at all.") */
static void* pb_com_new_from_lib(const PB_GUID* cg, const char* lib) {
    void* h = LoadLibraryA(lib);
    if (!h) return NULL;
    typedef int (__stdcall* DGCO_FN)(const PB_GUID*, const PB_GUID*, void**);
    DGCO_FN dgco = (DGCO_FN)GetProcAddress(h, "DllGetClassObject");
    if (!dgco) return NULL;
    void* fac = NULL;
    if (dgco(cg, &PB_IID_ICLASSFACTORY, &fac) != 0 || !fac) return NULL;
    /* IClassFactory vtable: 0 QI, 1 AddRef, 2 Release, 3 CreateInstance */
    typedef int (__stdcall* CREATEINST_FN)(void*, void*, const PB_GUID*, void**);
    typedef unsigned long (__stdcall* RELEASE_FN)(void*);
    CREATEINST_FN ci = (CREATEINST_FN)((void***)fac)[0][3];
    RELEASE_FN rel = (RELEASE_FN)((void***)fac)[0][2];
    void* obj = NULL;
    long hr = ci(fac, NULL, &PB_IID_IDISPATCH, &obj);
    rel(fac);
    return hr == 0 ? obj : NULL;
}

void* pb_com_new(int mode, const char* progid, const char* clsid, const char* lib) {
    PB_GUID cg;
    void* obj = NULL;
    long hr;
    pb_com_init();
    if (clsid && clsid[0]) {
        /* ClassID$ is a 16-byte binary GUID (CLSID$() result), or a "{...}"
         * string form.  PB BSTRs carry their own length, so the binary form
         * survives the NUL bytes. */
        size_t clen = SysStringByteLen(clsid);
        if (clen == 16) {
            memcpy(&cg, clsid, 16);
        } else {
            wchar_t wc[96];
            MultiByteToWideChar(0 /*CP_ACP*/, 0, clsid, -1, (short*)wc, 96);
            hr = CLSIDFromString(wc, &cg);
            if (hr != 0) { pb_com_hresult = 0; return NULL; }
        }
    } else {
        wchar_t wbuf[256];
        MultiByteToWideChar(0 /*CP_ACP*/, 0, progid, -1, (short*)wbuf, 256);
        hr = CLSIDFromProgID(wbuf, &cg);
        if (hr != 0) { pb_com_hresult = 0; return NULL; }
    }
    if (lib && lib[0]) {
        obj = pb_com_new_from_lib(&cg, lib);
    } else if (mode == 1) {
        /* GETCOM: open an existing running instance */
        hr = GetActiveObject(&cg, NULL, &obj);
        if (hr != 0) obj = NULL;
    } else {
        hr = CoCreateInstance(&cg, NULL, 0x7 /*CLSCTX_ALL*/, &PB_IID_IDISPATCH, &obj);
        if (hr != 0 && mode == 2) {
            /* ANYCOM: prefer an existing running instance */
            if (GetActiveObject(&cg, NULL, &obj) != 0) obj = NULL;
        } else if (hr != 0) {
            obj = NULL;
        }
    }
    /* batch271 measured: PBWin's OBJRESULT stays 0 both on NEWCOM success AND
     * failure (it reflects OBJECT-statement hResults, not NEWCOM).  Keep the
     * result slot at 0 so OBJRESULT matches PBWin on both paths. */
    pb_com_hresult = 0;
    return obj;
}

long pb_com_result(void) {
    return pb_com_hresult;
}

/* OBJRESULT$([nexp&]) — CHW OBJRESULT$_function.htm: "Returns a string which
 * describes an OBJRESULT (hResult) code."  Measured against PBWin10 (probe
 * objres_out.txt): the descriptive text is the Windows system message for the
 * hResult, produced by FormatMessageA(FROM_SYSTEM), e.g.
 *   hr=0x00000000 -> "The operation completed successfully."
 *   hr=0x00000001 -> "Incorrect function."
 *   hr=0x80004005 -> "Unspecified error"
 * Trailing CR/LF/space from FormatMessage are stripped (PBWin output has none).
 * If the system has no message for the code, fall back to a hex rendering
 * (CHW does not specify an English message table, so none is invented). */
char* pb_objresult_str(long hr_signed) {
    unsigned long hr = (unsigned long)hr_signed;
#define PB_FORMAT_MESSAGE_ALLOCATE_BUFFER 0x00000100UL
#define PB_FORMAT_MESSAGE_FROM_SYSTEM     0x00001000UL
#define PB_FORMAT_MESSAGE_IGNORE_INSERTS  0x00000200UL
#define PB_MAKELANGID(p, s)               (((unsigned long)(s) << 10) | (unsigned long)(p))
    char* buf = NULL;
    DWORD n = FormatMessageA(
        PB_FORMAT_MESSAGE_ALLOCATE_BUFFER | PB_FORMAT_MESSAGE_FROM_SYSTEM |
            PB_FORMAT_MESSAGE_IGNORE_INSERTS,
        NULL, hr, PB_MAKELANGID(0x00 /*LANG_NEUTRAL*/, 0x01 /*SUBLANG_DEFAULT*/),
        (char*)&buf, 0, NULL);
    if (n == 0 || buf == NULL) {
        char tmp[24];
        _snprintf(tmp, sizeof(tmp) - 1, "0x%08lX", hr);
        tmp[sizeof(tmp) - 1] = 0;
        return pb_bstr_alloc(tmp, (unsigned int)strlen(tmp));
    }
    while (n > 0 && (buf[n - 1] == '\r' || buf[n - 1] == '\n' || buf[n - 1] == ' ')) {
        n--;
    }
    char* out = pb_bstr_alloc(buf, (unsigned int)n);
    LocalFree(buf);
    return out;
}

/* ---- batch278-74: OBJECT CALL / OBJECT GET — DISPATCH invocation with
 * VARIANT marshalling (CHW OBJECT_statement / How_do_you_call_a_DISPATCH_METHOD).
 * Type codes shared with codegen:
 *   0 LONG(I4) 1 DOUBLE(R8) 2 STRING(ANSI PB BSTR -> VT_BSTR) 3 SINGLE(R4)
 *   4 DWORD(UI4) 5 WORD(UI2) 6 BYTE(UI1) 7 QUAD(I8) 8 CURRENCY 9 BOOL(VT_BOOL)
 *   10 VARIANT (caller passes a 16-byte PB_VARIANT slot)
 * wflags: 1 DISPATCH_METHOD (OBJECT CALL), 2 DISPATCH_PROPERTYGET (OBJECT GET),
 *         4 DISPATCH_PROPERTYPUT (OBJECT LET).
 * Returns the HRESULT (also stored for OBJRESULT). */
typedef struct _PB_IDISPATCHVtbl {
    long (__stdcall *QueryInterface)(void*, const PB_GUID*, void**);
    unsigned long (__stdcall *AddRef)(void*);
    unsigned long (__stdcall *Release)(void*);
    long (__stdcall *GetTypeInfoCount)(void*, unsigned int*);
    long (__stdcall *GetTypeInfo)(void*, unsigned int, unsigned long, void**);
    long (__stdcall *GetIDsOfNames)(void*, const PB_GUID*, wchar_t**,
                                    unsigned int, unsigned long, long*);
    long (__stdcall *Invoke)(void*, long, const PB_GUID*, unsigned long,
                             unsigned short, void*, void*, void*, unsigned int*);
} PB_IDISPATCHVtbl;

typedef struct _PB_IDISPATCH {
    PB_IDISPATCHVtbl* lpVtbl;
} PB_IDISPATCH;

typedef struct _PB_VARIANT {
    unsigned short vt;
    unsigned short wReserved1;
    unsigned short wReserved2;
    unsigned short wReserved3;
    union {
        long long llVal;          /* VT_I8    */
        double dblVal;            /* VT_R8    */
        float fltVal;             /* VT_R4    */
        long lVal;                /* VT_I4    */
        unsigned long ulVal;      /* VT_UI4   */
        unsigned short uiVal;     /* VT_UI2   */
        unsigned char bVal;       /* VT_UI1   */
        short boolVal;            /* VT_BOOL  */
        long long cyVal;          /* VT_CY    */
        void* pVal;               /* VT_BSTR / VT_DISPATCH / VT_UNKNOWN */
    } v;
} PB_VARIANT;

typedef struct _PB_DISPPARAMS {
    PB_VARIANT* rgvarg;           /* last-to-first! */
    long* rgdispidNamedArgs;
    unsigned int cArgs;
    unsigned int cNamedArgs;
} PB_DISPPARAMS;

static const PB_GUID PB_IID_NULL = {0, 0, 0, {0, 0, 0, 0, 0, 0, 0, 0}};

static unsigned short pb_com_vt_for(int tc) {
    switch (tc) {
        case 0: return 3;      /* VT_I4   */
        case 1: return 5;      /* VT_R8   */
        case 2: return 8;      /* VT_BSTR */
        case 3: return 4;      /* VT_R4   */
        case 4: return 19;     /* VT_UI4  */
        case 5: return 18;     /* VT_UI2  */
        case 6: return 17;     /* VT_UI1  */
        case 7: return 20;     /* VT_I8   */
        case 8: return 6;      /* VT_CY   */
        case 9: return 11;     /* VT_BOOL */
        case 10: return 12;    /* VT_VARIANT */
        case 12: return 8;     /* VT_BSTR — WSTRING slot already holds a
                                  UTF-16 BSTR (batch280) */
        default: return 0;     /* VT_EMPTY */
    }
}

/* Convert the result VARIANT into the PB-typed result slot. */
static void pb_com_coerce_result(PB_VARIANT* v, int rtype, void* slot) {
    switch (rtype) {
        case 0: /* LONG */
            switch (v->vt) {
                case 3:  *(long*)slot = v->v.lVal; break;
                case 11: *(long*)slot = v->v.boolVal ? -1 : 0; break;
                case 19: *(long*)slot = (long)v->v.ulVal; break;
                case 20: *(long*)slot = (long)v->v.llVal; break;
                case 5:  *(long*)slot = (long)v->v.dblVal; break;
                case 4:  *(long*)slot = (long)v->v.fltVal; break;
                default: *(long*)slot = 0; break;
            }
            break;
        case 1: /* DOUBLE */
            switch (v->vt) {
                case 5:  *(double*)slot = v->v.dblVal; break;
                case 3:  *(double*)slot = (double)v->v.lVal; break;
                case 4:  *(double*)slot = (double)v->v.fltVal; break;
                case 11: *(double*)slot = v->v.boolVal ? 1.0 : 0.0; break;
                case 19: *(double*)slot = (double)v->v.ulVal; break;
                default: *(double*)slot = 0.0; break;
            }
            break;
        case 2: { /* STRING: COM BSTR (UTF-16) -> ANSI PB string */
            char* src = NULL;
            int slen = 0;
            if (v->vt == 8 && v->v.pVal) {
                int n = WideCharToMultiByte(0, 0, (wchar_t*)v->v.pVal, -1, NULL, 0, NULL, NULL);
                if (n > 0) {
                    src = (char*)malloc((size_t)n);
                    WideCharToMultiByte(0, 0, (wchar_t*)v->v.pVal, -1, src, n, NULL, NULL);
                    slen = n - 1;
                }
            }
            *(char**)slot = pb_bstr_alloc(src ? src : "", src ? (unsigned int)slen : 0);
            if (src) free(src);
            break;
        }
        case 3:  *(float*)slot = (v->vt == 4) ? v->v.fltVal : 0.0f; break;
        case 4:  *(unsigned long*)slot = (v->vt == 19) ? v->v.ulVal : (unsigned long)0; break;
        case 5:  *(unsigned short*)slot = (v->vt == 18) ? v->v.uiVal : (unsigned short)0; break;
        case 6:  *(unsigned char*)slot = (v->vt == 17) ? v->v.bVal : (unsigned char)0; break;
        case 7:  *(long long*)slot = (v->vt == 20) ? v->v.llVal : (long long)0; break;
        case 8:  *(long long*)slot = (v->vt == 6) ? v->v.cyVal : (long long)0; break;
        case 9:  *(long*)slot = (v->vt == 11 && v->v.boolVal) ? -1 : 0; break;
        case 10: *(PB_VARIANT*)slot = *v; break;
        case 11: /* IDISPATCH object: keep the interface pointer (compound
                    member chains walk it for the next segment).  A non-object
                    result is a marshalling error -> NULL + DISP_E_BADVARTYPE. */
            if (v->vt == 9 && v->v.pVal) {
                *(void**)slot = v->v.pVal;
            } else if (v->vt == 9) {
                *(void**)slot = NULL;
            } else {
                *(void**)slot = NULL;
                pb_com_hresult = (long)0x80020005L; /* DISP_E_BADVARTYPE */
            }
            break;
        default: break;
    }
}

long pb_com_invoke(void* disp, const char* mname, void** pargs, const int* ptypes,
                   int nargs, void* presult, int rtype, int wflags) {
    PB_IDISPATCH* d = (PB_IDISPATCH*)disp;
    PB_VARIANT* vargs = NULL;
    PB_VARIANT vresult;
    wchar_t wname[512];
    long dispid = 0;
    long hr;
    unsigned int argerr = 0;
    int i;

    if (!d) {
        pb_com_hresult = (long)0x80004003L; /* E_POINTER */
        return pb_com_hresult;
    }
    pb_com_init();
    if (mname && mname[0]) {
        wchar_t* pname = wname; /* rgszNames is wchar_t*[] — a pointer array */
        MultiByteToWideChar(0, 0, mname, -1, (short*)wname, 512);
        hr = d->lpVtbl->GetIDsOfNames(d, &PB_IID_NULL, &pname, 1,
                                      0x400 /*LOCALE_USER_DEFAULT*/, &dispid);
        if (hr != 0) {
            pb_com_hresult = hr;
            return hr;
        }
    } else {
        dispid = 0; /* DISPID_VALUE */
    }
    if (nargs > 0) {
        vargs = (PB_VARIANT*)calloc((size_t)nargs, sizeof(PB_VARIANT));
        if (!vargs) {
            pb_com_hresult = (long)0x8007000EL; /* E_OUTOFMEMORY */
            return pb_com_hresult;
        }
        for (i = 0; i < nargs; i++) {
            PB_VARIANT* v = &vargs[nargs - 1 - i]; /* DISPPARAMS is last-to-first */
            v->vt = pb_com_vt_for(ptypes[i]);
            switch (ptypes[i]) {
                case 0: v->v.lVal = *(long*)pargs[i]; break;
                case 1: v->v.dblVal = *(double*)pargs[i]; break;
                case 2: { /* ANSI PB string (literal-constant pointer or BSTR) -> UTF-16 COM BSTR.
                           * Fork string slots may point at interned C-string constants (no
                           * BSTR length prefix) or at real BSTRs; read length via strlen so
                           * constants cannot make us read before the string (A/V risk).
                           * NUL-containing BSTRs passed to COM are a recorded limitation. */
                    const char* s = *(const char**)pargs[i];
                    unsigned int blen = s ? (unsigned int)strlen(s) : 0;
                    int n = s ? MultiByteToWideChar(0, 0, s, (int)blen, NULL, 0) : 0;
                    wchar_t* wb = n > 0 ? (wchar_t*)SysAllocStringLen(NULL, n) : NULL;
                    if (wb && s && n > 0) {
                        MultiByteToWideChar(0, 0, s, (int)blen, (short*)wb, n);
                    }
                    v->v.pVal = wb;
                    break;
                }
                case 12: { /* WSTRING: slot already points at a UTF-16 BSTR
                               (length prefix + UTF-16LE + NUL) — pass as-is. */
                    v->v.pVal = *(void**)pargs[i];
                    break;
                }
                case 3: v->v.fltVal = *(float*)pargs[i]; break;
                case 4: v->v.ulVal = *(unsigned long*)pargs[i]; break;
                case 5: v->v.uiVal = *(unsigned short*)pargs[i]; break;
                case 6: v->v.bVal = *(unsigned char*)pargs[i]; break;
                case 7: v->v.llVal = *(long long*)pargs[i]; break;
                case 8: v->v.cyVal = *(long long*)pargs[i]; break;
                case 9: v->v.boolVal = (*(long*)pargs[i]) ? -1 : 0; break;
                case 10: *v = *(PB_VARIANT*)pargs[i]; break;
                default: v->vt = 0; break;
            }
        }
    }
    memset(&vresult, 0, sizeof(vresult));
    {
        PB_DISPPARAMS dp;
        dp.rgvarg = vargs;
        dp.rgdispidNamedArgs = NULL;
        dp.cArgs = (unsigned int)nargs;
        dp.cNamedArgs = 0;
        hr = d->lpVtbl->Invoke(d, dispid, &PB_IID_NULL, 0x400,
                               (unsigned short)wflags, &dp, &vresult, NULL, &argerr);
    }
    pb_com_hresult = hr;
    if (hr == 0 && presult) {
        pb_com_coerce_result(&vresult, rtype, presult);
    }
    if (vargs) {
        for (i = 0; i < nargs; i++) {
            if (ptypes[i] == 2 && vargs[nargs - 1 - i].v.pVal) {
                SysFreeString((char*)vargs[nargs - 1 - i].v.pVal);
            }
            /* ptypes[i] == 12 (WSTRING) passes the slot pointer as-is —
               it belongs to the PB variable, never freed here. */
        }
        free(vargs);
    }
    return hr;
}

void pb_events_add(void* tblp, int mname, pb_event_fn fn) {
    pb_event_tbl* t = (pb_event_tbl*)tblp;
    if (!t) return;
    if (t->count >= t->cap) {
        t->cap = t->cap ? t->cap * 2 : 4;
        t->ents = (pb_event_ent*)realloc(t->ents, t->cap * sizeof(pb_event_ent));
    }
    t->ents[t->count].mname = mname;
    t->ents[t->count].fn = fn;
    t->count++;
}
int pb_events_attach(void* client, void* tblp) {
    if (g_ev_sub_count >= 64) return 0;
    g_ev_subs[g_ev_sub_count].client = client;
    g_ev_subs[g_ev_sub_count].tbl = (pb_event_tbl*)tblp;
    g_ev_sub_count++;
    return 1;
}
int pb_events_detach(void* client) {
    int i;
    for (i = 0; i < g_ev_sub_count; i++) {
        if (g_ev_subs[i].client == client) {
            free(g_ev_subs[i].tbl->ents);
            free(g_ev_subs[i].tbl);
            g_ev_sub_count--;
            for (; i < g_ev_sub_count; i++) g_ev_subs[i] = g_ev_subs[i + 1];
            return 1;
        }
    }
    return 0;
}
int pb_raise_event(int mname, int arg) {
    int called = 0, i, j;
    if (!g_event_enabled) return 0;
    for (i = 0; i < g_ev_sub_count; i++) {
        pb_event_tbl* t = g_ev_subs[i].tbl;
        for (j = 0; j < t->count; j++) {
            if (t->ents[j].mname == mname) {
                t->ents[j].fn(g_ev_subs[i].client, arg);
                called++;
            }
        }
    }
    return called;
}
void pb_event_source_set(void* obj, int source_id) { /* noop */ }
/* LET with OBJECTS � object reference assignment (simplified: pointer copy) */
void pb_let_object(void** dst, void* src) { if (dst) *dst = src; }
/* LET with VARIANTS � variant assignment (simplified: store as pointer+tag) */
static void* g_variant_last_ptr = 0;
static int g_variant_last_tag = 0;
void pb_let_variant(void** dst_ptr, int* dst_tag, void* src_ptr, int src_tag) {
    if (dst_ptr) *dst_ptr = src_ptr;
    if (dst_tag) *dst_tag = src_tag;
    g_variant_last_ptr = src_ptr;
    g_variant_last_tag = src_tag;
}

/* GRAPHIC BITMAP � memory DIB bitmaps (batch 51) */
static long long g_last_bmp = 0;
long long pb_gdi_bitmap_new(int w, int h) {
    void* hdc = CreateCompatibleDC(0);
    unsigned char bi[40];
    for (int i = 0; i < 40; i++) bi[i] = 0;
    bi[0] = 40;                   /* biSize */
    bi[4] = (unsigned char)(w & 255); bi[5] = (unsigned char)((w >> 8) & 255);
    bi[6] = (unsigned char)((w >> 16) & 255); bi[7] = (unsigned char)((w >> 24) & 255);
    int hh = -h;                  /* top-down */
    bi[8] = (unsigned char)(hh & 255); bi[9] = (unsigned char)((hh >> 8) & 255);
    bi[10] = (unsigned char)((hh >> 16) & 255); bi[11] = (unsigned char)((hh >> 24) & 255);
    bi[12] = 1;                   /* biPlanes */
    bi[14] = 32;                  /* biBitCount */
    void* bits = 0;
    void* hbm = CreateDIBSection(hdc, (const void*)bi, 0, &bits, 0, 0);
    DeleteDC(hdc);
    if (hbm) g_last_bmp = (long long)(intptr_t)hbm;
    return (long long)(intptr_t)hbm;
}
int pb_gdi_bitmap_end(long long h) {
    void* target = (void*)(intptr_t)(h == -1 ? g_last_bmp : h);
    int ok = target ? DeleteObject(target) : 0;
    if (target == (void*)(intptr_t)g_last_bmp) g_last_bmp = 0;
    return ok ? 1 : 0;
}

/* MENU � user32 menu objects (batch 50) */
long long pb_menu_new_bar(void) {
    return (long long)(intptr_t)CreateMenu();
}
long long pb_menu_new_popup(void) {
    return (long long)(intptr_t)CreatePopupMenu();
}
int pb_menu_add_string(long long h, char* txt, int id, int state) {
    /* CHW separator convention: txt$="-", id&=0, state&=0 creates a real
     * Win32 separator (MF_SEPARATOR), not a string item displaying "-". */
    if (txt && txt[0] == '-' && txt[1] == '\0' && id == 0 && state == 0) {
        return AppendMenuA((void*)(intptr_t)h, 0x0800, 0, 0) ? 1 : 0; /* MF_SEPARATOR */
    }
    unsigned int flags = 0x0000; /* MF_STRING */
    flags |= (unsigned int)state;
    int ok = AppendMenuA((void*)(intptr_t)h, flags, (unsigned long long)id, txt) ? 1 : 0;
    /* AppendMenuA with MF_DEFAULT does not reliably mark the item as the
     * default; call SetMenuDefaultItem to make it stick (so GetMenuItemInfoA
     * returns MFS_DEFAULT and PBWin's readback matches). */
    if (ok && (state & 0x1000)) {
        SetMenuDefaultItem((void*)(intptr_t)h, (unsigned int)id, 0 /*MF_BYCOMMAND*/);
    }
    return ok;
}
int pb_menu_add_popup(long long h, char* txt, long long hSub, int state) {
    /* MF_POPUP | MF_STRING: attach the submenu handle as uIDNewItem and pass
     * the label as lpNewItem.  state is ORed in so MFS_CHECKED etc. on a popup
     * entry are honored. */
    unsigned int flags = 0x0010u /*MF_POPUP*/ | 0x0000u /*MF_STRING*/;
    flags |= (unsigned int)state;
    return AppendMenuA((void*)(intptr_t)h, flags,
                       (unsigned long long)(intptr_t)hSub, txt) ? 1 : 0;
}
int pb_menu_delete(long long h, int bycmd, int pos) {
    /* PB MENU DELETE uses 1-based physical indexing; Win32 MF_BYPOSITION is
     * 0-based.  BYCMD switches to MF_BYCOMMAND (pos is the item id, no offset). */
    unsigned int flags = bycmd ? 0x0000u /*MF_BYCOMMAND*/ : 0x0400u /*MF_BYPOSITION*/;
    unsigned int m = bycmd ? (unsigned int)pos : (unsigned int)(pos - 1);
    return DeleteMenu((void*)(intptr_t)h, m, flags) ? 1 : 0;
}
int pb_dialog_menu(void* hDlg, void* hMenu) {
    SetMenu(hDlg, hMenu);
    /* SetMenu alone does not reliably resize the non-client area; the menu bar
       must be accounted for in GetClientRect right away (batch 174 probe: the
       PBWin client height shrank by the menu bar, the fork's did not).  A
       SWP_FRAMECHANGED pass forces Windows to recompute the window frame and
       client area, which is what makes DIALOG GET CLIENT report the smaller
       height with the bar attached. */
    SetWindowPos(hDlg, 0, 0, 0, 0, 0,
                 0x0001 | 0x0002 | 0x0004 | 0x0020); /* SWP_NOSIZE|SWP_NOMOVE|SWP_NOZORDER|SWP_FRAMECHANGED */
    DrawMenuBar(hDlg);
    InvalidateRect(hDlg, 0, 1);
    UpdateWindow(hDlg);
    return 1;
}

/* MENU ATTACH / MENU CONTEXT / MENU DRAW BAR (batch 174) */
int pb_menu_attach(long long hMenu, long long hDlg) {
    /* Official page: "Attaches a menu to a dialog, replacing any existing
       menu.  The dialog is redrawn to accommodate the new menu." - that is
       exactly what the DIALOG SET MENU path already does, so share it. */
    return pb_dialog_menu((void*)(intptr_t)hDlg, (void*)(intptr_t)hMenu);
}
int pb_menu_draw_bar(long long hDlg) {
    /* Official page: redraw a dialog's menu bar after it was altered
       dynamically.  DrawMenuBar returns non-zero on success. */
    return DrawMenuBar((void*)(intptr_t)hDlg) ? 1 : 0;
}
int pb_menu_context(long long hMenu, int x, int y, unsigned int flags, long* cmd) {
    if (!cmd) return 0;
    *cmd = 0;
    /* The official statement has no owner-window parameter - PowerBASIC
       tracks the current dialog itself - so the active window is the
       closest runtime equivalent. */
    void* owner = GetActiveWindow();
    /* TPM_RETURNCMD returns the chosen id instead of posting WM_COMMAND, and
       TPM_NONOTIFY suppresses the notifications.  Together they reproduce the
       documented MENU CONTEXT behaviour: an item callback is ignored and the id
       comes back in the target variable.  0 means dismissed with no choice. */
    int id = TrackPopupMenu((void*)(intptr_t)hMenu,
                            flags | 0x0100u /* TPM_RETURNCMD */ | 0x0080u /* TPM_NONOTIFY */,
                            x, y, 0, owner, 0);
    *cmd = (long)id;
    return 1;
}

/* MENU GET STATE / SET STATE / GET TEXT / SET TEXT (batch 62) */
int pb_menu_get_state(long long h, int bycmd, long pos, long* state) {
    if (!state) return 0;
    /* Use GetMenuState as the base.  PBWin normalizes out MF_SEPARATOR
     * (0x800) and returns 0 for a missing item.  MF_DEFAULT (0x1000) is
     * exposed via GetMenuItemInfoA best-effort. */
    unsigned int f = GetMenuState((void*)(intptr_t)h,
                                  (unsigned int)(bycmd ? pos : pos - 1),
                                  bycmd ? 0 : 0x0400);
    if (f == (unsigned int)-1) { *state = 0; return 0; }
    f &= ~0x0800u;
    {
        PB_MENUITEMINFOA mii;
        memset(&mii, 0, sizeof(mii));
        mii.cbSize = sizeof(mii);
        mii.fMask = 0x0001; /* MIIM_STATE */
        int ok = GetMenuItemInfoA((void*)(intptr_t)h,
                                  (unsigned int)(bycmd ? pos : pos - 1),
                                  bycmd ? 0 : 1, &mii);
        if (ok && (mii.fState & 0x1000u)) {
            f |= 0x1000u; /* MFS_DEFAULT */
        }
    }
    *state = (long)(f & 0xFFFF);
    return 1;
}
int pb_menu_set_state(long long h, int bycmd, long pos, long state) {
    unsigned int base = bycmd ? 0 : 0x0400; /* MF_BYCOMMAND=0, MF_BYPOSITION=0x400 */
    int ok = 1;
    unsigned int en = 0;
    if (state & 1) en = 0x0001;      /* MF_GRAYED */
    else if (state & 2) en = 0x0002; /* MF_DISABLED */
    else en = 0x0000;                /* MF_ENABLED */
    ok = ok && EnableMenuItem((void*)(intptr_t)h, (unsigned int)(bycmd ? pos : pos - 1), base | en) != (unsigned int)-1;
    if (state & 8) ok = ok && CheckMenuItem((void*)(intptr_t)h, (unsigned int)(bycmd ? pos : pos - 1), base | 0x0008) != (unsigned int)-1;
    else if (state & 0x10) ok = ok && CheckMenuItem((void*)(intptr_t)h, (unsigned int)(bycmd ? pos : pos - 1), base | 0x0000) != (unsigned int)-1;
    if (state & 0x80) ok = ok && EnableMenuItem((void*)(intptr_t)h, (unsigned int)(bycmd ? pos : pos - 1), base | 0x0080) != (unsigned int)-1; /* MF_HILITE */
    return ok ? 1 : 0;
}
int pb_menu_get_text(long long h, int bycmd, long pos, char** dest) {
    if (!dest) return 0;
    char buf[1024];
    buf[0] = '\0';
    int n = GetMenuStringA((void*)(intptr_t)h, (unsigned int)(bycmd ? pos : pos - 1), buf, 1024, bycmd ? 0 : 0x0400);
    if (n == 0) { *dest = pb_bstr_alloc("", 0); return 0; }
    *dest = pb_bstr_alloc(buf, (unsigned int)strlen(buf));
    return 1;
}
int pb_menu_set_text(long long h, int bycmd, long pos, char* txt) {
    unsigned int base = bycmd ? 0 : 0x0400;
    return ModifyMenuA((void*)(intptr_t)h, (unsigned int)(bycmd ? pos : pos - 1), base | 0x0040 | 0x0000, (unsigned int)pos, txt) ? 1 : 0; /* MF_STRING|MF_ENABLED */
}
int pb_menu_destroy(long long h) {
    return DestroyMenu((void*)(intptr_t)h) ? 1 : 0;
}

/* COLOR � console text attribute (PB/CC, batch 49) */
void pb_color(int fore, int back) {
    void* h = GetStdHandle((unsigned int)-11); /* STD_OUTPUT_HANDLE */
    if (h == 0 || h == (void*)-1) return;
    unsigned short attr = 7;
    if (fore >= 0 && fore <= 15) attr = (unsigned short)(fore & 15);
    if (back >= 0 && back <= 15) attr |= (unsigned short)((back & 15) << 4);
    SetConsoleTextAttribute(h, attr);
}

/* IMAGELIST � comctl32 image list objects (batch 48) */
long long pb_imagelist_new(int width, int height, int depth, int initial, int is_icon) {
    unsigned int flags = 0;
    switch (depth) {
        case 0: flags = 1 | 0; break;          /* ILC_MASK | ILC_COLOR */
        case 4: flags = 1 | 0x4; break;        /* ILC_COLOR4 */
        case 8: flags = 1 | 0x8; break;        /* ILC_COLOR8 */
        case 16: flags = 1 | 0x10; break;      /* ILC_COLOR16 */
        case 32: flags = 1 | 0x20; break;      /* ILC_COLOR32 */
        default: flags = 1 | 0x18; break;      /* ILC_COLOR24 */
    }
    /* ILC_MASK (bit 0) is set for both forms.  comctl32 has no separate
       "bitmap list" mode: the IMAGELIST NEW BITMAP / NEW ICON words select
       which ADD forms the program intends, and the ImageList object itself is
       built the same way either way.  `is_icon` is carried through so the two
       documented forms stay distinct in the IR and in the coverage CSV - a
       documented equivalence, not a discarded keyword. */
    (void)is_icon;
    void* h = ImageList_Create(width, height, flags, initial, 4);
    return (long long)(intptr_t)h;
}
int pb_imagelist_count(long long h) {
    return ImageList_GetImageCount((void*)(intptr_t)h);
}
int pb_imagelist_kill(long long h) {
    return ImageList_Destroy((void*)(intptr_t)h) ? 1 : 0;
}

/* IMAGELIST ADD / SET OVERLAY - batch 173.
   The Win32 calls return the 0-based index of the first image added, or -1
   when the operation fails.  The official PB page documents the value the TO
   clause receives as "the index position of the first added bitmap (starting
   with 1)", and 0 on failure - so the wrappers below shift by one and fold
   every failure into 0, which keeps the two ranges disjoint. */

static const char* pb_il_res_id(const char* name) {
    /* Mirrors pb_dialog_set_icon(): a name that begins with '#' is an
       integral resource id rather than a resource name. */
    if (name && name[0] == '#') {
        long id = 0;
        const char* q = name + 1;
        while (*q >= '0' && *q <= '9') { id = id * 10 + (*q - '0'); q++; }
        return (const char*)(long long)id;
    }
    return name ? name : "";
}

/* Official rule for the Bmp$ / Icn$ / Msk$ forms: a name containing a period
   is a disk file; otherwise the resource is tried first and the disk file is
   the fallback.  LR_SHARED (0x8000) keeps the resource handle owned by the
   system, so it must never be freed; a handle loaded with LR_LOADFROMFILE
   (0x10) is a private copy, and *owned reports that. */
static void* pb_il_load_name(const char* name, unsigned int type, int cx, int cy, int* owned) {
    if (owned) *owned = 0;
    if (!name || !name[0]) return 0;
    if (strchr(name, '.')) {
        void* h = LoadImageA(0, name, type, cx, cy, 0x10);
        if (h && owned) *owned = 1;
        return h;
    }
    /* LoadImageA consults the calling module's resources only when the module
       handle is passed; with a NULL instance it looks at the system image, so a
       resource compiled into this very EXE (the documented `#id` form) would
       never be found.  The NULL retry preserves the previous behaviour; a
       handle obtained with LR_SHARED stays owned by the system. */
    void* h = LoadImageA(GetModuleHandleA(0), pb_il_res_id(name), type, cx, cy, 0x8000);
    if (!h) h = LoadImageA(0, pb_il_res_id(name), type, cx, cy, 0x8000);
    if (h) return h;
    h = LoadImageA(0, name, type, cx, cy, 0x10);
    if (h && owned) *owned = 1;
    return h;
}

static void pb_il_free(void* h, unsigned int type, int owned) {
    if (!h || !owned) return;
    if (type == 1) DestroyIcon(h);   /* IMAGE_ICON */
    else DeleteObject(h);            /* IMAGE_BITMAP */
}

int pb_imagelist_add_bitmap(long long h, long long hbmpImage, long long hbmpMask) {
    void* himl = (void*)(intptr_t)h;
    if (!himl || !hbmpImage) return 0;
    int idx = ImageList_Add(himl, (void*)(intptr_t)hbmpImage,
                            hbmpMask ? (void*)(intptr_t)hbmpMask : 0);
    return idx < 0 ? 0 : idx + 1;
}

int pb_imagelist_add_bitmap_file(long long h, const char* bmp, const char* msk) {
    void* himl = (void*)(intptr_t)h;
    if (!himl || !bmp || !bmp[0]) return 0;
    int own_bmp = 0, own_msk = 0;
    void* hb = pb_il_load_name(bmp, 0, 0, 0, &own_bmp);
    if (!hb) return 0;
    void* hm = 0;
    if (msk && msk[0]) hm = pb_il_load_name(msk, 0, 0, 0, &own_msk);
    int idx = ImageList_Add(himl, hb, hm);
    pb_il_free(hm, 0, own_msk);
    pb_il_free(hb, 0, own_bmp);
    return idx < 0 ? 0 : idx + 1;
}

int pb_imagelist_add_icon(long long h, long long hicon) {
    void* himl = (void*)(intptr_t)h;
    if (!himl || !hicon) return 0;
    /* ImageList_ReplaceIcon(himl, -1, hicon) appends an icon, and is the
       documented way to do it: the legacy ImageList_AddIcon macro
       (ImageList_Add with an icon handle) returns -1 for a modern icon
       handle - measured on this machine, batch 173. */
    int idx = ImageList_ReplaceIcon(himl, -1, (void*)(intptr_t)hicon);
    return idx < 0 ? 0 : idx + 1;
}

int pb_imagelist_add_icon_file(long long h, const char* icn) {
    void* himl = (void*)(intptr_t)h;
    if (!himl || !icn || !icn[0]) return 0;
    /* Load the icon at the size the list stores, so nothing needs scaling. */
    int cx = 0, cy = 0;
    ImageList_GetIconSize(himl, &cx, &cy);
    int owned = 0;
    void* hi = pb_il_load_name(icn, 1, cx, cy, &owned);
    if (!hi) return 0;
    int idx = ImageList_ReplaceIcon(himl, -1, hi);
    pb_il_free(hi, 1, owned);
    return idx < 0 ? 0 : idx + 1;
}

int pb_imagelist_add_masked(long long h, long long hbmpImage, unsigned long rgb) {
    void* himl = (void*)(intptr_t)h;
    if (!himl || !hbmpImage) return 0;
    int idx = ImageList_AddMasked(himl, (void*)(intptr_t)hbmpImage, rgb);
    return idx < 0 ? 0 : idx + 1;
}

int pb_imagelist_add_masked_file(long long h, const char* bmp, unsigned long rgb) {
    void* himl = (void*)(intptr_t)h;
    if (!himl || !bmp || !bmp[0]) return 0;
    int owned = 0;
    void* hb = pb_il_load_name(bmp, 0, 0, 0, &owned);
    if (!hb) return 0;
    int idx = ImageList_AddMasked(himl, hb, rgb);
    pb_il_free(hb, 0, owned);
    return idx < 0 ? 0 : idx + 1;
}

int pb_imagelist_set_overlay(long long h, int image, int overlay) {
    void* himl = (void*)(intptr_t)h;
    if (!himl) { pb_err_var = pb_err = 5; return 0; }
    /* Overlay indexes are 1..15; anything else is reported as a failure. */
    if (ImageList_SetOverlayImage(himl, image, overlay)) return 1;
    pb_err_var = pb_err = 5;
    return 0;
}

/* FONT NEW / FONT END � GDI logical font objects (batch 47) */
int pb_font_new(const char* name, float points, int style, int charset, int pitch, int escapement) {
    void* hdc = GetDC(NULL);
    int height = 0;
    if (hdc) {
        height = -MulDiv((int)(points + 0.5f), GetDeviceCaps(hdc, 90), 72);
        ReleaseDC(NULL, hdc);
    }
    int weight = PB_FW_NORMAL;
    unsigned char italic = 0, underline = 0, strikeout = 0;
    if (style & 1) weight = PB_FW_BOLD;
    if (style & 2) italic = 1;
    if (style & 4) underline = 1;
    if (style & 8) strikeout = 1;
    void* hf = CreateFontA(height, 0, 0, 0, weight, italic, underline, strikeout,
                           (unsigned char)charset, 0, 0,
                           0, 0, name);
    return (int)(intptr_t)hf;
}
int pb_font_end(int h) {
    return DeleteObject((void*)(intptr_t)h) ? 1 : 0;
}

/* MEMORY COPY src, dst, count � byte block copy (memmove, overlap-safe). */
void pb_mem_copy(long long src, long long dst, long long count) {
    if (count > 0) memmove((void*)(intptr_t)dst, (void*)(intptr_t)src, (size_t)count);
}

/* MEMORY SWAP src, dst, count � byte-by-byte exchange of two blocks. */
void pb_mem_swap(long long src, long long dst, long long count) {
    unsigned char* a = (unsigned char*)(intptr_t)src;
    unsigned char* b = (unsigned char*)(intptr_t)dst;
    for (long long i = 0; i < count; i++) {
        unsigned char t = a[i];
        a[i] = b[i];
        b[i] = t;
    }
}

/* MEMORY FILL dst, Count_bytes, BYTE|WORD|DWORD val.  CHW MEMORY_statement.htm:
   Count is always a number of BYTES; the BYTE/WORD/DWORD keyword only chooses
   the width of the repeating little-endian pattern.  Fill `count` bytes,
   cycling through the pattern bytes. */
void pb_mem_fill(long long dst, long long count, long long val, long long width) {
    unsigned char* p = (unsigned char*)(intptr_t)dst;
    if (width < 1) width = 1;
    for (long long i = 0; i < count; i++) {
        p[i] = (unsigned char)((val >> (8 * (i % width))) & 0xFF);
    }
}

/* MEMORY FILL dst, count, str$ � repeat the string pattern over count bytes. */
void pb_mem_fill_str(long long dst, long long count, const char* s) {
    unsigned char* p = (unsigned char*)(intptr_t)dst;
    size_t slen = s ? (size_t)pb_str_len(s) : 0;
    if (slen == 0) return;
    for (long long i = 0; i < count; i++) p[i] = (unsigned char)s[i % slen];
}

/* ERL$ � most recent error checkpoint id, as a string (numeric approximation of
   the official label/line-name semantics; limited to the checkpoint id stored by
   the ON ERROR trapping machinery). */
char* pb_erl_str(void) {
    char buf[32];
    sprintf(buf, "%d", pb_err_stmt_id);
    return pb_bstr_alloc(buf, (int)strlen(buf));
}

/* EXTRACT$([start,] MainStr, [ANY] MatchStr) � returns MainStr from start up to
   (not including) the first occurrence of MatchStr; ANY = any single character
   of MatchStr ends the extraction. start <= 0 or beyond length -> empty string;
   MatchStr absent -> whole remainder. */
char* pb_extract(long long start, const char* main_str, const char* match_str, int any_mode) {
    long long len = main_str ? (long long)pb_str_len(main_str) : 0;
    if (start <= 0 || start > len) return pb_bstr_alloc("", 0);
    long long i = start - 1;
    long long end = len;
    if (match_str && match_str[0]) {
        if (any_mode) {
            for (long long k = i; k < len; k++) {
                if (strchr(match_str, main_str[k])) { end = k; break; }
            }
        } else {
            const char* p = strstr(main_str + i, match_str);
            if (p) end = (long long)(p - main_str);
        }
    }
    return pb_bstr_alloc(main_str + i, (unsigned int)(end - i));
}

/* RGB(r, g, b) � pack into &H00BBGGRR (byte1 red, byte2 green, byte3 blue). */
long long pb_rgb3(long long r, long long g, long long b) {
    return (r & 0xFF) | ((g & 0xFF) << 8) | ((b & 0xFF) << 16);
}

/* BGR(r, g, b) � pack into &H00RRGGBB (byte1 blue, byte2 green, byte3 red). */
long long pb_bgr3(long long r, long long g, long long b) {
    return (b & 0xFF) | ((g & 0xFF) << 8) | ((r & 0xFF) << 16);
}

/* RGB(bgrval) / BGR(rgbval) � single-argument byte swap (identical op). */
long long pb_rgb_swap(long long x) {
    return ((x & 0xFF) << 16) | (x & 0xFF00) | ((x >> 16) & 0xFF);
}

/* Batch 43: BITS$ / PATHNAME$ / PRINTERCOUNT
   LoadLibraryA-based EnumPrintersW probe; kept near the other dllimport
   consumers (pb_import_addr) rather than the file tail. */
#define PB_PRINTER_ENUM_LOCAL 0x00000002
#define PB_PRINTER_ENUM_CONNECTIONS 0x00000004
#define PB_PRINTER_INFO_LEVEL2 2
__declspec(dllimport) int __stdcall EnumPrintersW(unsigned long Flags, const char* Name, unsigned long Level, char* pPrinterEnum, unsigned long cbBuf, unsigned long* pcbNeeded, unsigned long* pcReturned);

char* pb_bits_str(const char* director, const char* s) {
    (void)director; /* STRING / WSTRING � this build is ANSI-only */
    if (!s) return pb_bstr_alloc("", 0);
    return pb_bstr_alloc(s, (int)strlen(s));
}
char* pb_pathname(const char* director, const char* spec) {
    if (!spec) spec = "";
    const char* slash = strrchr(spec, '\\');
    const char* colon = strrchr(spec, ':');
    const char* last = slash > colon ? slash : colon;
    const char* dot = strrchr(spec, '.');
    char buf[1024];
    int len = 0;
    if (!director) director = "FULL";
    if (strcmp(director, "FULL") == 0) {
        len = (int)strlen(spec);
        if (len > 1023) len = 1023;
        memcpy(buf, spec, len);
    } else if (strcmp(director, "PATH") == 0) {
        if (last) { len = (int)(last - spec) + 1; if (len > 1023) len = 1023; memcpy(buf, spec, len); }
    } else if (strcmp(director, "EXTN") == 0) {
        if (dot) { const char* q = dot; len = (int)strlen(q); if (len > 1023) len = 1023; memcpy(buf, q, len); }
    } else { /* NAME or NAMEX */
        const char* start = last ? last + 1 : spec;
        int slen = (int)strlen(start);
        if (strcmp(director, "NAME") == 0 && dot && dot > start) slen = (int)(dot - start);
        len = slen;
        if (len > 1023) len = 1023;
        memcpy(buf, start, len);
    }
    buf[len] = 0;
    return pb_bstr_alloc(buf, len);
}

/* Batch 42: DAYNAME$ / MONTHNAME$ / DATACOUNT / THREADCOUNT (definitions at file end:
   they reference data_count and pb_thr[] declared earlier). */
char* pb_dayname(long long n) {
    static const char* names[7] = {"Sunday","Monday","Tuesday","Wednesday","Thursday","Friday","Saturday"};
    if (n < 0 || n > 6) n = 0;
    return pb_bstr_alloc((char*)names[n], (unsigned int)strlen(names[n]));
}
char* pb_monthname(long long n) {
    static const char* names[12] = {"January","February","March","April","May","June","July","August","September","October","November","December"};
    if (n < 1 || n > 12) n = 1;
    return pb_bstr_alloc((char*)names[n-1], (unsigned int)strlen(names[n-1]));
}
long long pb_data_count(void) {
    return data_active ? (long long)data_active->count : 0;
}
long long pb_thread_count(void) {
    long long n = 1; /* primary thread */
    for (int i = 0; i < 256; i++) {
        if (pb_thr[i].state) n++;
    }
    return n;
}


/* === JOIN$: concatenate string array with delimiter ===
   CHW: JOIN$(array(), delim$ | BINARY).  base is the fork [N x ptr] array
   block, which has NO NULL sentinel, so count is the exact element count.
   PBWin-measured: when the delimiter content is the 3 characters
   quote-comma-quote (source code """,""") each section is wrapped in one
   double quote at each end (quotes/commas inside the data are NOT escaped)
   and the sections are joined with a single comma.  Empty sections become
   a pair of quotes.  Any other delimiter is copied verbatim between
   sections, and an empty delimiter inserts nothing. */
char* pb_join(char* base, long long count, const char* delim) {
    if (count <= 0) return SysAllocStringByteLen("", 0);
    char** arr = (char**)base;
    size_t dlen = delim ? strlen(delim) : 0;
    int quote_mode = (dlen == 3 && delim[0] == '"' && delim[1] == ',' && delim[2] == '"');

    /* First pass: calculate total length */
    size_t total = 0;
    for (long long i = 0; i < count; i++) {
        char* s = arr[i];
        size_t slen = (size_t)pb_str_len(s);
        if (quote_mode) {
            total += slen + 2;                 /* wrapping quotes */
            if (i < count - 1) total += 1;    /* single comma */
        } else {
            total += slen;
            if (i < count - 1) total += dlen;
        }
    }

    char* result = SysAllocStringByteLen(NULL, (unsigned int)total);
    if (!result) return SysAllocStringByteLen("", 0);

    /* Second pass: build */
    char* p = result;
    for (long long i = 0; i < count; i++) {
        char* s = arr[i];
        size_t slen = (size_t)pb_str_len(s);
        if (quote_mode) {
            *p++ = '"';
            if (slen) { memcpy(p, s, slen); p += slen; }
            *p++ = '"';
            if (i < count - 1) *p++ = ',';
        } else {
            if (slen) { memcpy(p, s, slen); p += slen; }
            if (i < count - 1 && dlen) { memcpy(p, delim, dlen); p += dlen; }
        }
    }

    return result;
}

/* Tier-3 DDT GUI #1: WINDOW title$, x, y, w, h TO hWnd& */
/* Callback table: control HWND -> C function pointer (PB SUB address) */
#define PB_MAX_CALLBACKS 64
static void* pb_cb_hwnd[PB_MAX_CALLBACKS];
static void* pb_cb_fns[PB_MAX_CALLBACKS];
static int    pb_cb_count = 0;

void pb_register_callback_hwnd(void* hwnd, void* fn) {
    for (int i = 0; i < pb_cb_count; i++) {
        if (pb_cb_hwnd[i] == hwnd) { pb_cb_fns[i] = fn; return; }
    }
    if (pb_cb_count < PB_MAX_CALLBACKS) {
        pb_cb_hwnd[pb_cb_count] = hwnd;
        pb_cb_fns[pb_cb_count] = fn;
        pb_cb_count++;
    }
}

/* batch 169: look up the per-window callback registered above.  Until now the
   table was written but never read, so a `... CALL cb` operand recorded the
   address and then dropped it.  TAB page dialogs are the first consumer. */
void* pb_lookup_callback_hwnd(void* hwnd) {
    for (int i = 0; i < pb_cb_count; i++) {
        if (pb_cb_hwnd[i] == hwnd) return pb_cb_fns[i];
    }
    return 0;
}



/* Forward declaration */

/* The dialog-unit base every CONTROL coordinate is expressed in.
   pb_dlu_to_px() below converts units -> pixels when a control is created,
   and the CONTROL GET/SET geometry statements convert back with the same two
   numbers: a control written at "12, 12" must read back as 12, 12 and SET LOC
   must put it where ADD would have.  They used to disagree - the geometry
   went through GetDialogBaseUnits() (8x16 on this machine), so a button added
   at 10,10 read back 8,8 (see the batch 175 block further down).

   batch 239: the base is the dialog font MS Shell Dlg 8pt, whose real
   metrics on this machine are aveCharWidth=6, tmHeight=13 (the comment below
   already said ~6/~13; the constants were 7x14, which made every control
   ~17% too wide and ~8% too tall).  Verified against PBWin's HelloDDT: a
   40x14 DLU button must come out 60x23 px (40*6/4=60, 14*13/8=22.75) and a
   134x12 DLU textbox 201x19 px. */
#define PB_CTL_DLU_X 6
#define PB_CTL_DLU_Y 13

/* DLU <-> px conversion, matching PBWin's measured behaviour:
   - X axis (position and width): truncate (14 DLU -> 21 px, 40 -> 60,
     134 -> 201).
   - Y axis (position and height): round to nearest half-pixel upward
     (12 -> 20, 32 -> 52, 14 -> 23).  The +6 is dy/2.
   - The DIALOG NEW client size is truncated (160x50 -> 240x81), which is
     why pb_window_new uses plain multiplication, not these macros. */
#define PB_DLU_X_TO_PX(v) (((v) * PB_CTL_DLU_X) / 4)
#define PB_DLU_Y_TO_PX(v) (((v) * PB_CTL_DLU_Y + 6) / 8)
#define PB_PX_X_TO_DLU(v) (((v) * 4) / PB_CTL_DLU_X)
#define PB_PX_Y_TO_DLU(v) (((v) * 8 + 6) / PB_CTL_DLU_Y)

/* Convert dialog units to pixels (matching PBWin DDT) */
static void pb_dlu_to_px(int *x, int *y, int *w, int *h) {
    /* 8pt MS Shell Dlg: avg char width 6px, height 13px (measured) */
    *x = PB_DLU_X_TO_PX(*x);
    *y = PB_DLU_Y_TO_PX(*y);
    *w = PB_DLU_X_TO_PX(*w);
    *h = PB_DLU_Y_TO_PX(*h);
}

void* pb_control_add_button(void* parent, long id, const char* text, int x, int y, int w, int h);

/* === DIALOG model: CB.* context variables === */
static unsigned int cb_msg = 0;
static void*        cb_hwnd = 0;
static unsigned int cb_ctl = 0;
static unsigned int cb_ctlmsg = 0;
static unsigned long long cb_wparam = 0;
static unsigned long long cb_lparam = 0;
/* Dialog callback function (CB.MSG etc. are visible inside it) */
static void* pb_dialog_cb = 0;
/* Modal dialog: when DIALOG END is called, set this */
static int pb_modal_result = 0;
static int pb_modal_done = 0;

unsigned int pb_get_cb_msg(void) { return cb_msg; }
void*        pb_get_cb_hwnd(void) { return cb_hwnd; }
unsigned int pb_get_cb_ctl(void) { return cb_ctl; }
unsigned int pb_get_cb_ctlmsg(void) { return cb_ctlmsg; }
unsigned long long pb_get_cb_wparam(void) { return cb_wparam; }
unsigned long long pb_get_cb_lparam(void) { return cb_lparam; }

/* CB.NM* (CHW CB_Callback_functions.htm).  On WM_NOTIFY the lParam points at
   the Win64 NMHDR (hwndFrom UINT_PTR, idFrom UINT_PTR, code LONG -> 20 bytes
   padded to 24).  We snapshot it as a 12-byte PB-layout block (hwndFrom DWord,
   idFrom DWord, code Long) so CB.NMHDR$, CB.NMCODE, CB.NMHWND and CB.NMID
   return PBWin-equivalent values on both 32-bit PBWin and 64-bit fork. */
static unsigned char cb_nm_buf[12] = {0,0,0,0, 0,0,0,0, 0,0,0,0};
static int cb_nm_valid = 0;

void* pb_get_cb_nmhdr(void) { return cb_nm_valid ? (void*)cb_nm_buf : 0; }
char* pb_get_cb_nmhdr_str(int size) {
    int n = (size > 0) ? size : 12;
    return pb_bstr_alloc((const char*)cb_nm_buf, n);
}
unsigned int pb_get_cb_nmcode(void) {
    if (!cb_nm_valid) return 0;
    unsigned char* p = cb_nm_buf + 8;
    return (unsigned int)(p[0] | (p[1] << 8) | ((unsigned int)p[2] << 16) | ((unsigned int)p[3] << 24));
}
void* pb_get_cb_nmhwnd(void) {
    if (!cb_nm_valid) return 0;
    unsigned char* p = cb_nm_buf;
    unsigned long lo = (unsigned long)(p[0] | (p[1] << 8) | ((unsigned long)p[2] << 16) | ((unsigned long)p[3] << 24));
    return (void*)(size_t)lo;
}
unsigned int pb_get_cb_nmid(void) {
    if (!cb_nm_valid) return 0;
    unsigned char* p = cb_nm_buf + 4;
    return (unsigned int)(p[0] | (p[1] << 8) | ((unsigned int)p[2] << 16) | ((unsigned int)p[3] << 24));
}

void pb_register_dialog_cb(void* fn) { pb_dialog_cb = fn; }
void pb_dialog_end(void* hWnd, int result) {
    pb_modal_result = result;
    pb_modal_done = 1;
    DestroyWindow(hWnd);
}

/* CONTROL GET TEXT by dialog handle + control ID */
void pb_control_get_text_by_id(void* hDlg, long id, void* buf, long bufsize) {
    void* hCtrl = GetDlgItem(hDlg, id);
    if (hCtrl) {
        SendMessageA(hCtrl, 0x000D /* WM_GETTEXT */, (unsigned long long)bufsize, (unsigned long long)buf);
    }
}

/* CONTROL ADD BUTTON with CALL callback - register BN_CLICKED callback */
void pb_control_add_button_with_cb(void* parent, long id, const char* text, int x, int y, int w, int h, void* fn) {
    void* hBtn = pb_control_add_button(parent, id, text, x, y, w, h);
    if (fn) pb_register_callback_hwnd(hBtn, fn);
}

/* -------------------------------------------------------------------
   Batch 165 forward declarations.  pb_wndproc (immediately below) paints
   a dialog's custom background colour, and the helpers that colour table
   lives further down the file.
   ------------------------------------------------------------------- */
/* batch 169: TAB page switching.  TCN_SELCHANGE = TCN_FIRST(-550) - 1
   (commctrl.inc / CommCtrl.h); pb_wndproc forwards it to the TAB family. */
#define PB_TCN_SELCHANGE 0xFFFFFDD9u
static void pb_tab_apply_selection(void* hTab);

#define PB_DLG_BG_SLOTS 64
static void* pb_dlg_bg_h[PB_DLG_BG_SLOTS];
static long  pb_dlg_bg_v[PB_DLG_BG_SLOTS];
static int   pb_dlg_bg_set[PB_DLG_BG_SLOTS];

/* ------------------------------------------------------------------
   Batch 179 - CONTROL SET COLOR hDlg, id&, foreclr&, backclr&
   ------------------------------------------------------------------
   The DDT engine answers the colour question from inside the dialog's
   own window procedure: Windows sends %WM_CTLCOLORSTATIC (and the rest
   of the %WM_CTLCOLOR* family) to the control's PARENT before a control
   paints, and the brush handed back is what paints that control's
   background.  pb_wndproc below is that parent procedure, so the table
   and its lookup live here, next to the dialog-background table the
   same procedure already consults for %WM_ERASEBKGND.

   Slots are keyed by the CONTROL's window handle, and the parent window
   is remembered too so the whole group can be dropped when the dialog
   goes away: Windows recycles window handles, and a stale entry would
   otherwise colour an unrelated control created later.

   backclr&:  -1 = the control's default background (per class: an EDIT
                     or LISTBOX paints on COLOR_WINDOW, a STATIC on the
                     dialog face COLOR_BTNFACE),
              -2 = the text background is not painted at all
                     (SetBkMode TRANSPARENT + the stock NULL_BRUSH),
             >=0 = that solid colour.
   foreclr&:  -1 = the default text colour, otherwise that colour.
   ------------------------------------------------------------------ */
#define PB_CTL_COLOR_SLOTS 64
static void* pb_ctl_clr_ctl[PB_CTL_COLOR_SLOTS];
static void* pb_ctl_clr_par[PB_CTL_COLOR_SLOTS];
static long  pb_ctl_clr_fg[PB_CTL_COLOR_SLOTS];
static long  pb_ctl_clr_bg[PB_CTL_COLOR_SLOTS];
static void* pb_ctl_clr_br[PB_CTL_COLOR_SLOTS];
static int   pb_ctl_clr_set[PB_CTL_COLOR_SLOTS];
static void* pb_ctlcolor_brush(void* hCtl, void* hdc);
static void  pb_ctl_color_forget(void* hWnd);
static int pb_dlg_bg_lookup(void* hWnd, unsigned long* rgb);
/* forward decl (full decl later in file at the batch-181 block) */
__declspec(dllimport) int __stdcall PostMessageA(void* hWnd, unsigned long Msg, pb_wparam_t wParam, pb_lparam_t lParam);
/* DIALOG DEFAULT FONT installs this into every dialog created afterwards */
static void* pb_default_font = 0;
__declspec(dllimport) int __stdcall GetClientRect(void*, void*);
__declspec(dllimport) void* __stdcall CreateSolidBrush(unsigned long);
__declspec(dllimport) int __stdcall FillRect(void*, const void*, void*);
__declspec(dllimport) int __stdcall DeleteObject(void*);
__declspec(dllimport) int __stdcall ScreenToClient(void*, void*);
__declspec(dllimport) void* __stdcall ChildWindowFromPoint(void*, long long);
typedef struct { long x, y; } pb_point_t;
#define PB_LOWORD(l)  ((unsigned int)((unsigned long long)(l) & 0xFFFF))
#define PB_HIWORD(l)  ((unsigned int)(((unsigned long long)(l) >> 16) & 0xFFFF))

static long long __stdcall pb_wndproc(void* hWnd, unsigned int Msg, unsigned long long wParam, unsigned long long lParam) {
    if (Msg == 0x0083) /* WM_NCCALCSIZE - dialog-style frame */ {
        long* r = (long*)lParam;   /* RECT: left, top, right, bottom */
        /* A dialog with a menu bar attached (MENU ATTACH / DIALOG SET MENU)
           must reserve the menu height on top, or GetClientRect keeps
           reporting the menu-less client height (batch 174 probe: PBWin
           client shrank by the bar, the fork's did not).  The frame here is
           hand-computed (3px borders + 26px caption), so the menu bar has
           to be added explicitly. */
        int menuh = 0;
        if (GetMenu(hWnd)) menuh = GetSystemMetrics(15 /* SM_CYMENU */);
        r[0] += 3;
        r[1] += 26 + menuh;
        r[2] -= 3;
        r[3] -= 3;
        return 0;
    }
    if (Msg == 0x0002) /* WM_DESTROY */ {
        pb_ctl_color_forget(hWnd);
        PostQuitMessage(0);
        return 0;
    }
    if (Msg == 0x0111) /* WM_COMMAND */ {
        cb_msg = Msg;
        cb_hwnd = hWnd;
        cb_ctl = (unsigned int)(wParam & 0xFFFF);
        cb_ctlmsg = (unsigned int)(wParam >> 16);
        cb_wparam = wParam;
        cb_lparam = lParam;
        /* batch 243: official callbacks.htm semantics - a control callback
           (registered by `CONTROL ADD ... CALL cb`, lParam is the control
           handle for control notifications) receives the control's
           WM_COMMAND; if it returns TRUE the message is consumed.  Otherwise
           the dialog callback is tried next; TRUE there also consumes it.
           Previously only pb_dialog_cb was consulted, so a BUTTON's CALL
           callback never fired. */
        void* fn = pb_lookup_callback_hwnd((void*)lParam);
        if (fn) {
            long (*cf)(void) = (long(*)(void))fn;
            if (cf()) return 0;
        }
        if (pb_dialog_cb) {
            long (*df)(void) = (long(*)(void))pb_dialog_cb;
            if (df()) return 0;
        }
        return DefWindowProcA(hWnd, Msg, wParam, lParam);
    }
    if (Msg == 0x004E) /* WM_NOTIFY */ {
        /* NMHDR is { HWND hwndFrom; UINT_PTR idFrom; UINT code; } (WinUser.h) */
        void** nh = (void**)lParam;
        if (nh) {
            void* from = nh[0];
            unsigned int idfrom = (unsigned int)(size_t)nh[1];
            unsigned int code = (unsigned int)(size_t)nh[2];
            {
                /* snapshot PB-layout NMHDR (12 bytes) for CB.NM* */
                unsigned long hwlo = (unsigned long)(size_t)from;
                unsigned long idlo = idfrom;
                long codel = (long)(int)code;
                cb_nm_buf[0] = (unsigned char)hwlo;
                cb_nm_buf[1] = (unsigned char)(hwlo >> 8);
                cb_nm_buf[2] = (unsigned char)(hwlo >> 16);
                cb_nm_buf[3] = (unsigned char)(hwlo >> 24);
                cb_nm_buf[4] = (unsigned char)idlo;
                cb_nm_buf[5] = (unsigned char)(idlo >> 8);
                cb_nm_buf[6] = (unsigned char)(idlo >> 16);
                cb_nm_buf[7] = (unsigned char)(idlo >> 24);
                cb_nm_buf[8] = (unsigned char)codel;
                cb_nm_buf[9] = (unsigned char)(codel >> 8);
                cb_nm_buf[10] = (unsigned char)(codel >> 16);
                cb_nm_buf[11] = (unsigned char)(codel >> 24);
                cb_nm_valid = 1;
            }
            if (code == PB_TCN_SELCHANGE) pb_tab_apply_selection(from);
            cb_msg = Msg;
            cb_hwnd = hWnd;
            cb_ctl = idfrom;
            cb_ctlmsg = code;
            cb_wparam = wParam;
            cb_lparam = lParam;
            if (pb_dialog_cb) {
                void (*fn)(void) = (void(*)(void))pb_dialog_cb;
                fn();
            }
        }
        return 0;
    }
    /* PB_CTLCOLOR branch - batch 179.  0x0132..0x0138 is the whole
       %WM_CTLCOLOR* family (MSGBOX, EDIT, LISTBOX, BTN, DLG, SCROLLBAR,
       STATIC).  lParam is the control those colours belong to and wParam
       is the DC it is about to paint with; the brush returned from here is
       what Windows fills that control's background with.  Zero means "no
       colour is registered for that control", and the message then falls
       through to the default handling below, exactly as before.
       batch 245c: even with no registered colour, the dialog callback must
       still see WM_CTLCOLOR* (SYNCLIST.BAS synchronises two ListBoxes
       inside its WM_CTLCOLORLISTBOX case - the message is Windows' "a
       ListBox is about to repaint" hook).  Set the CB pseudo-variables and
       call pb_dialog_cb; a non-zero return from the callback is the brush
       handle to use, zero means default. */
    if (Msg >= 0x0132 && Msg <= 0x0138) {
        void* br = pb_ctlcolor_brush((void*)lParam, (void*)wParam);
        if (br) return (long long)(size_t)br;
        if (pb_dialog_cb) {
            cb_msg = Msg;
            cb_hwnd = hWnd;
            cb_ctl = 0;
            cb_ctlmsg = 0;
            cb_wparam = wParam;
            cb_lparam = lParam;
            long (*df)(void) = (long(*)(void))pb_dialog_cb;
            long r = df();
            if (r) return r;
        }
    }
    if (Msg == 0x020A) /* WM_MOUSEWHEEL */ {
        /* A standard #32770 dialog (DefDlgProc) forwards the mouse wheel
           to the child control under the cursor; our own dialog class
           goes through DefWindowProcA, which does not, so the wheel is
           dropped and a ListBox never scrolls.  Replicate the dialog
           behaviour: find the child under the wheel position and hand
           the message to it.  lParam holds the event's screen coords,
           wParam the wheel delta. */
        pb_point_t pt;
        void* child;
        pt.x = (short)PB_LOWORD(lParam);
        pt.y = (short)PB_HIWORD(lParam);
        ScreenToClient(hWnd, &pt);
        child = ChildWindowFromPoint(hWnd, (long long)pt.x | ((long long)pt.y << 32));
        if (child && child != hWnd) {
            SendMessageA(child, Msg, wParam, lParam);
            return 0;
        }
    }
    if (Msg == 0x0014) /* WM_ERASEBKGND */ {
        unsigned long rgb;
        if (pb_dlg_bg_lookup(hWnd, &rgb)) {
            void* dc = (void*)wParam;
            void* br;
            long rcs[4];   /* RECT = four LONGs */
            rcs[0] = 0; rcs[1] = 0; rcs[2] = 0; rcs[3] = 0;
            GetClientRect(hWnd, rcs);
            br = CreateSolidBrush(rgb);
            if (br) {
                FillRect(dc, rcs, br);
                DeleteObject(br);
            }
            return 1;
        }
    }
    __try {
        return DefWindowProcA(hWnd, Msg, wParam, lParam);
    } __except(1) {
        return 0;
    }
}

/* x64 WNDCLASSEXA layout (80 bytes) */
typedef struct {
    unsigned int cbSize;
    unsigned int style;
    void* lpfnWndProc;
    int cbClsExtra;
    int cbWndExtra;
    void* hInstance;
    void* hIcon;
    void* hCursor;
    void* hbrBackground;
    const char* lpszMenuName;
    const char* lpszClassName;
    void* hIconSm;
} pb_wndclassex_t;

/* x64 MSG layout (48 bytes) */
typedef struct {
    void* hwnd;
    unsigned int message;
    unsigned int _pad0;
    unsigned long long wParam;
    unsigned long long lParam;
    unsigned long time;
    long pt_x;
    long pt_y;
} pb_msg_t;

/* batch 239: pb_window_new needs the non-client geometry.  pb_rect_t and
   AdjustWindowRectEx are declared later in the file (the batch 181 block);
   the typedef is moved up here so pb_window_new can use them, and the
   later typedef is removed (redefinition would be an error). */
typedef struct { int left; int top; int right; int bottom; } pb_rect_t;
__declspec(dllimport) int __stdcall AdjustWindowRectEx(pb_rect_t*, unsigned long, int, unsigned long);

/* batch 239: the dialog-unit base.  Official PBWin converts DIALOG NEW and
   every geometry statement with the dialog template font - the DIALOG
   DEFAULT FONT if the program set one, else the default dialog font
   (MS Shell Dlg 8pt, base units 6 x 13 as PBWin measures them on this
   system).  One DLU x = baseunitX/4 px, one DLU y = baseunitY/8 px.
   Everything that converts DLU <-> px goes through the same two numbers so
   ADD and GET round-trip.  (The old code used GetDialogBaseUnits() - the
   *system* font 8x16 @96dpi - which made dialogs and controls ~33% wider
   and ~25% taller than PBWin's, and 13/8 is not an integer divisor, so the
   round-trip now rounds like PBWin does.) */
static void pb_dialog_base_units(int* dlu_x, int* dlu_y) {
    int dx = 6, dy = 13;
    if (pb_default_font) {
        typedef struct {
            long tmHeight, tmAscent, tmDescent, tmInternalLeading, tmExternalLeading;
            long tmAveCharWidth, tmMaxCharWidth, tmWeight, tmOverhang;
            long tmDigitizedAspectX, tmDigitizedAspectY;
            unsigned short tmFirstChar, tmLastChar, tmDefaultChar, tmBreakChar;
            unsigned char tmItalic, tmUnderlined, tmStruckOut, tmPitchAndFamily, tmCharSet;
        } pb_textmetrica_t;
        __declspec(dllimport) int __stdcall GetTextMetricsA(void*, void*);
        void* dc = GetDC(0);
        if (dc) {
            void* old = SelectObject(dc, pb_default_font);
            pb_textmetrica_t tm;
            memset(&tm, 0, sizeof(tm));
            if (GetTextMetricsA(dc, &tm)) {
                if (tm.tmAveCharWidth > 0) dx = tm.tmAveCharWidth;
                if (tm.tmHeight > 0) dy = tm.tmHeight;
            }
            if (old) SelectObject(dc, old);
            ReleaseDC(0, dc);
        }
    }
    if (dlu_x) *dlu_x = dx;
    if (dlu_y) *dlu_y = dy;
}

/* batch 241: the dialog font every DIALOG/control uses.  A program-set
   DIALOG DEFAULT FONT wins; otherwise the default dialog font - MS Shell
   Dlg 8pt, the same template font PBWin's dialogs use (tmHeight 13 on this
   system, which is the baseY the DLU conversions use).  The handle is
   cached so every control shares one HFONT. */
static void* pb_dialog_font(void) {
    if (pb_default_font) return pb_default_font;
    static void* f = 0;
    if (!f) {
        /* PBWin's DIALOG NEW has no FONT clause by default, and the
           compiled dialog then carries MS Sans Serif at -11 (measured on
           the original HelloDDT: height -11, weight 400, face
           "MS Sans Serif").  The old CreateFontA(-10, "MS Shell Dlg")
           was 8pt Tahoma-class, which is why the fork text looked smaller
           and different; GetStockObject(DEFAULT_GUI_FONT) resolves to the
           Win11 UI font (Segoe UI) which is also not a match. */
        f = CreateFontA(-11, 0, 0, 0, 400, 0, 0, 0, 1,
                        0, 0, 0, 0, "MS Sans Serif");
        if (!f) f = (void*)1;   /* never return NULL (WM_SETFONT requires a font) */
    }
    return f;
}

void* pb_window_new(const char* title, int x, int y, int w, int h) {
    __declspec(dllimport) void __stdcall InitCommonControls(void); InitCommonControls();
    __declspec(dllimport) int __stdcall SetWindowPos(void*, void*, int, int, int, int, unsigned int);
    static int registered = 0;
    if (!registered) {
        pb_wndclassex_t wc;
        memset(&wc, 0, sizeof(wc));
        wc.cbSize = sizeof(wc);
        wc.lpfnWndProc = (void*)pb_wndproc;
        wc.hInstance = GetModuleHandleA(0);
        wc.hCursor = LoadCursorA(0, (const char*)32512);
        wc.hbrBackground = (void*)5;
        wc.lpszClassName = "PBWIN_CLASS";
        unsigned long rc = RegisterClassExA(&wc);
        registered = 1;
    }
    /* batch 239: DIALOG NEW dimensions are dialog units (DLU) - resolved
       through pb_dialog_base_units() above (default dialog font 6x13,
       or the DIALOG DEFAULT FONT's real metrics when the program set one).
       The created window is the client rect plus the non-client area. */
    int dluX = 6, dluY = 13;
    pb_dialog_base_units(&dluX, &dluY);
    /* PBWin truncates: 160x50 DLU -> 240x81 px client rect */
    int px = (x * dluX) / 4;
    int py = (y * dluY) / 8;
    int cw = (w * dluX) / 4;
    int ch = (h * dluY) / 8;
    /* outer window = client rect + non-client (title bar, borders) */
    pb_rect_t rc;
    rc.left = 0; rc.top = 0; rc.right = cw; rc.bottom = ch;
    /* batch 241: the DIALOG NEW style is the standard dialog style -
       WS_CAPTION | WS_SYSMENU.  No WS_THICKFRAME (a dialog is not
       resizable), no WS_MINIMIZEBOX / WS_MAXIMIZEBOX (PBWin dialogs have
       only the close button).  The old WS_OVERLAPPEDWINDOW style drew the
       minimize/maximize buttons and a thick resizable border, which is why
       the fork window looked unlike the PBWin original.  The extended
       style is the dialog one (WS_EX_DLGMODALFRAME | WS_EX_WINDOWEDGE =
       0x101).
       The outer size is client + (6, 29): the dialog non-client frame
       (3px borders + 26px caption, see the WM_NCCALCSIZE handler), which
       is exactly what a PBWin dialog measures.  Creating the window at
       this size makes its client rect come out as the requested cw x ch. */
    void* hwnd = CreateWindowExA(0x101u, "PBWIN_CLASS", title,
        0x00C80000u | 0x10000000u,
        px, py, cw + 6, ch + 29, 0, 0, GetModuleHandleA(0), 0);
    if (hwnd) {
        ShowWindow(hwnd, 5);
        UpdateWindow(hwnd);
        /* the dialog font (DIALOG DEFAULT FONT if set, else MS Sans Serif
           -11, matching a PBWin dialog) - batch 241: controls inherit it,
           and without it the window and its children render in the system
           default font (different glyphs, clipped text). */
        SendMessageA(hwnd, 0x0030 /* WM_SETFONT */,
                     (unsigned long long)(size_t)pb_dialog_font(), 1);
    }
    return hwnd;
}

/* batch 242: DIALOG NEW full official syntax (PBWin.chw).
   DIALOG NEW [PIXELS, | UNITS,] hParent, title$, [x&], [y&], xx&, yy&
             [, [style&] [, [exstyle&]]] [,] TO hDlg
   - units: 0 = UNITS (dialog units, default), 1 = PIXELS (plain pixels)
   - style:  0 -> official default &H084C000D4& (DS_3DLOOK | DS_SETFONT |
             DS_MODALFRAME | DS_NOFAILCREATE | WS_BORDER | WS_CLIPSIBLINGS |
             WS_DLGFRAME | WS_POPUP); non-zero -> caller's value used as-is
             (PBWin semantics: custom style replaces the default).
   - exstyle: 0 -> 0x101 (WS_EX_DLGMODALFRAME | WS_EX_WINDOWEDGE);
             non-zero -> caller's value.
   - Background: COLOR_BTNFACE (3D dialog grey, what a PBWin dialog shows),
             via its own window class PBWIN_DLG_CLASS (WINDOW keeps
             PBWIN_CLASS / COLOR_WINDOW, unchanged).
   - NOTE: like pb_window_new, the dialog is shown immediately (WS_VISIBLE).
             PBWin semantics create the dialog hidden and DIALOG SHOW MODAL /
             MODELESS reveal it; aligning that requires reworking
             pb_message_loop (TODO, see SKILL). */
void* pb_window_new_dlg(const char* title, int x, int y, int w, int h,
                        unsigned long style, unsigned long exstyle, int units) {
    __declspec(dllimport) void __stdcall InitCommonControls(void); InitCommonControls();
    static int dlg_registered = 0;
    if (!dlg_registered) {
        pb_wndclassex_t wc;
        memset(&wc, 0, sizeof(wc));
        wc.cbSize = sizeof(wc);
        wc.lpfnWndProc = (void*)pb_wndproc;
        wc.hInstance = GetModuleHandleA(0);
        wc.hCursor = LoadCursorA(0, (const char*)32512);
        wc.hbrBackground = (void*)16;   /* COLOR_BTNFACE + 1 (3D dialog grey) */
        wc.lpszClassName = "PBWIN_DLG_CLASS";
        unsigned long rc = RegisterClassExA(&wc);
        dlg_registered = 1;
    }
    int dluX = 6, dluY = 13;
    pb_dialog_base_units(&dluX, &dluY);
    int px, py, cw, ch;
    if (units) {
        /* PIXELS: x&/y&/xx&/yy& are plain pixels (client size). */
        px = x; py = y; cw = w; ch = h;
    } else {
        /* UNITS (default): dialog units -> pixels, PBWin truncation rules. */
        px = (x * dluX) / 4;
        py = (y * dluY) / 8;
        cw = (w * dluX) / 4;
        ch = (h * dluY) / 8;
    }
    unsigned long dwStyle = style ? style : 0x84C000D4u; /* official default */
    dwStyle |= 0x10000000u; /* WS_VISIBLE (see NOTE above) */
    unsigned long dwEx = exstyle ? exstyle : 0x101u;
    void* hwnd = CreateWindowExA(dwEx, "PBWIN_DLG_CLASS", title, dwStyle,
        px, py, cw + 6, ch + 29, 0, 0, GetModuleHandleA(0), 0);
    if (hwnd) {
        ShowWindow(hwnd, 5);
        UpdateWindow(hwnd);
        SendMessageA(hwnd, 0x0030 /* WM_SETFONT */,
                     (unsigned long long)(size_t)pb_dialog_font(), 1);
    }
    return hwnd;
}
/* ==================================================================
   Batch 180 - the GRAPHIC WINDOW family
   ------------------------------------------------------------------
   GRAPHIC WINDOW NEW/TEXT....... creates a standalone graphic window
   GRAPHIC WINDOW CLICK.......... 1 single / 2 double / 0 none + x!/y!
   GRAPHIC WINDOW END............ close and destroy
   GRAPHIC WINDOW HIDE........... ShowWindow SW_HIDE
   GRAPHIC WINDOW NORMALIZE...... ShowWindow SW_RESTORE (clears HIDE and
                                 MINIMIZE both, which is what the help
                                 page promises)
   GRAPHIC WINDOW MINIMIZE....... ShowWindow SW_MINIMIZE
   GRAPHIC WINDOW STABILIZE...... close box greyed, WM_CLOSE and the
                                 ALT-F4 path (SC_CLOSE) refused
   GRAPHIC WINDOW NONSTABLE...... the default again

   Each window keeps its own content buffer: a memory DC over a
   compatible bitmap the size of its client area.  That buffer is what
   the GRAPHIC drawing statements write into while the window is the
   selected target, and WM_PAINT blits it back, which is what makes the
   display persistent after the window is uncovered or restored
   ("All PowerBASIC graphical displays are persistent", GRAPHIC WINDOW).

   A window is addressed by its handle; when the handle is omitted or
   zero the rule is the documented one - use the graphic window that was
   created (and therefore selected) last, else a target attached with
   GRAPHIC ATTACH.  */
#define PB_GW_SLOTS 8
#define PB_GW_CLASS "PBGRAPHIC_CLASS"
#define PB_SC_CLOSE 0xF060u
#define PB_GWLP_USERDATA (-21)
#define PB_GW_STABLE 1
#define PB_GW_SRCCOPY 0x00CC0020u

static void* pb_gw_win[PB_GW_SLOTS];
static void* pb_gw_dc[PB_GW_SLOTS];
static void* pb_gw_bmp[PB_GW_SLOTS];
static long long pb_gw_click[PB_GW_SLOTS];
static int   pb_gw_cx[PB_GW_SLOTS];
static int   pb_gw_cy[PB_GW_SLOTS];
static void* pb_gw_cur = 0;              /* window created (selected) last */

__declspec(dllimport) void* __stdcall BeginPaint(void* hWnd, void* lpPaint);
__declspec(dllimport) int   __stdcall EndPaint(void* hWnd, const void* lpPaint);
__declspec(dllimport) void* __stdcall CreateCompatibleBitmap(void* hdc, int cx, int cy);
__declspec(dllimport) int   __stdcall IsIconic(void* hWnd);
__declspec(dllimport) int   __stdcall IsWindowVisible(void* hWnd);
__declspec(dllimport) void* __stdcall GetSystemMenu(void* hWnd, int bRevert);
/* GetWindowLongPtrA / SetWindowLongPtrA exist in 64-bit user32 only - in the
   32-bit API they are macros over GetWindowLongA / SetWindowLongA.  Declaring
   the *Ptr names directly linked for x64 and broke the 32-bit build
   (_GetWindowLongPtrA@8 undefined), so the width-portable helpers defined
   further down are used instead. */
static long long pb_get_winlong(void* h, int idx);
static long long pb_set_winlong(void* h, int idx, long long v);
/* Do NOT declare GetWindowRect, EnableMenuItem or LoadCursorA here:
   GetWindowRect is declared further down with a pb_rect_t* parameter and a
   second declaration with void* is a conflicting type, while the other two
   are already declared above.  This block only calls GetClientRect. */

/* the four ints of a RECT out of a 16-byte buffer written by Win32 */
static void pb_gw_rect(void* buf, int* x, int* y, int* w, int* h) {
    unsigned char* b = (unsigned char*)buf;
    int l = b[0] | (b[1] << 8) | (b[2] << 16) | (b[3] << 24);
    int t = b[4] | (b[5] << 8) | (b[6] << 16) | (b[7] << 24);
    int r = b[8] | (b[9] << 8) | (b[10] << 16) | (b[11] << 24);
    int bo = b[12] | (b[13] << 8) | (b[14] << 16) | (b[15] << 24);
    *x = l; *y = t; *w = r - l; *h = bo - t;
}

static int pb_gw_find(void* h) {
    for (int i = 0; i < PB_GW_SLOTS; i++) if (pb_gw_win[i] == h) return i;
    return -1;
}

static void pb_gw_free_slot(int i) {
    if (pb_gw_dc[i] && (g_gr_dc == pb_gw_dc[i])) { g_gr_dc = 0; g_gr_bmp = 0; }
    if (pb_gw_bmp[i] && (g_gr_bmp == pb_gw_bmp[i])) { g_gr_dc = 0; g_gr_bmp = 0; }
    if (pb_gw_dc[i]) { SelectObject(pb_gw_dc[i], GetStockObject(5)); DeleteDC(pb_gw_dc[i]); }
    if (pb_gw_bmp[i]) DeleteObject(pb_gw_bmp[i]);
    pb_gw_win[i] = 0; pb_gw_dc[i] = 0; pb_gw_bmp[i] = 0;
    pb_gw_click[i] = 0; pb_gw_cx[i] = 0; pb_gw_cy[i] = 0;
}

static int pb_gw_alloc(int i) {
    unsigned char rc[16];
    for (int k = 0; k < 16; k++) rc[k] = 0;
    if (!GetClientRect(pb_gw_win[i], (void*)rc)) return 0;
    int x = 0, y = 0, w = 0, h = 0;
    pb_gw_rect(rc, &x, &y, &w, &h);
    if (w <= 0 || h <= 0) return 0;
    void* sdc = GetDC(0);
    void* dc = CreateCompatibleDC(sdc);
    void* bmp = CreateCompatibleBitmap(sdc, w, h);
    if (sdc) ReleaseDC(0, sdc);
    if (!dc || !bmp) {
        if (dc) DeleteDC(dc);
        if (bmp) DeleteObject(bmp);
        return 0;
    }
    SelectObject(dc, bmp);
    /* start from white: an unpainted window should not show garbage */
    unsigned char fr[16];
    for (int k = 0; k < 16; k++) fr[k] = 0;
    fr[8] = (unsigned char)(w & 255);         fr[9] = (unsigned char)((w >> 8) & 255);
    fr[10] = (unsigned char)((w >> 16) & 255); fr[11] = (unsigned char)((w >> 24) & 255);
    fr[12] = (unsigned char)(h & 255);        fr[13] = (unsigned char)((h >> 8) & 255);
    fr[14] = (unsigned char)((h >> 16) & 255); fr[15] = (unsigned char)((h >> 24) & 255);
    void* br = CreateSolidBrush(0x00FFFFFFu);
    if (br) { FillRect(dc, (const void*)fr, br); DeleteObject(br); }
    pb_gw_dc[i] = dc;
    pb_gw_bmp[i] = bmp;
    return 1;
}

/* ==== batch 185: the GRAPHIC keyboard family ==============================
   GRAPHIC INKEY$ is specified to return a string of 0, 1 or 2 characters: an
   empty string when the buffer is empty, one ASCII byte for a normal key (1..31
   are control codes), and a leading NUL plus the scan code for an extended key.
   The queue below stores exactly that byte stream, so INSTAT is simply "is
   anything queued" and INKEY$ is a single pop.

   The read statements pump this thread's message queue first: the graphic
   window in these programs has no explicit message loop of its own. */
#define PB_GR_KEYCAP 256
static unsigned char g_gr_keys[PB_GR_KEYCAP];
static int g_gr_kn = 0;

/* forward declarations; these must match the definitions already in this file */
__declspec(dllimport) int __stdcall TranslateMessage(const void* lpMsg);
__declspec(dllimport) unsigned long __stdcall DispatchMessageA(const void* lpMsg);
__declspec(dllimport) long __stdcall PeekMessageA(void*, void*, unsigned long,
                                                  unsigned long, unsigned long);
/* not declared elsewhere in this file */
__declspec(dllimport) void __stdcall Sleep(unsigned long dwMilliseconds);

static void pb_gr_key_push(unsigned char c) {
    if (g_gr_kn < PB_GR_KEYCAP) g_gr_keys[g_gr_kn++] = c;
}
static int pb_gr_is_ext_key(unsigned int vk) {
    if (vk >= 0x21 && vk <= 0x28) return 1;   /* PgUp PgDn End Home  arrows */
    if (vk == 0x2D || vk == 0x2E) return 1;   /* Insert  Delete */
    if (vk >= 0x70 && vk <= 0x7B) return 1;   /* F1 .. F12 */
    return 0;
}
static void pb_gr_pump(void) {
    unsigned char msg[64];
    int guard = 0;
    while (PeekMessageA((void*)msg, 0, 0, 0, 1u) && guard++ < 256) {
        TranslateMessage((void*)msg);
        DispatchMessageA((void*)msg);
    }
}

/* GRAPHIC INSTAT - non-zero when a character is ready; consumes nothing. */
int pb_graphic_instat(void) {
    pb_gr_pump();
    return g_gr_kn > 0 ? 1 : 0;
}
/* GRAPHIC INPUT FLUSH - remove all buffered keyboard data. */
int pb_graphic_input_flush(void) {
    pb_gr_pump();
    g_gr_kn = 0;
    return 0;
}

/* ---- batch 185 part 2: key reads, line reads, text splitting ------------- */

/* Pop one key: 0, 1 or 2 bytes, extended keys keeping the leading NUL. */
static int pb_gr_key_pop(unsigned char* out) {
    int i;
    if (g_gr_kn <= 0) return 0;
    out[0] = g_gr_keys[0];
    if (g_gr_keys[0] == 0 && g_gr_kn >= 2) {
        out[1] = g_gr_keys[1];
        for (i = 2; i < g_gr_kn; i++) g_gr_keys[i - 2] = g_gr_keys[i];
        g_gr_kn -= 2;
        return 2;
    }
    for (i = 1; i < g_gr_kn; i++) g_gr_keys[i - 1] = g_gr_keys[i];
    g_gr_kn -= 1;
    return 1;
}

/* GRAPHIC INKEY$ - reads a key if one is ready, otherwise the null string. */
char* pb_graphic_inkey(void) {
    unsigned char b[2];
    int n;
    pb_gr_pump();
    n = pb_gr_key_pop(b);
    return pb_bstr_alloc((const char*)b, (unsigned int)n);
}

static int pb_gr_mask_ok(const unsigned char* mask, unsigned char c) {
    int i;
    if (!mask || mask[0] == '\0') return 1;   /* empty mask: any key */
    for (i = 0; mask[i]; i++) if (mask[i] == c) return 1;
    return 0;
}

/* GRAPHIC WAITKEY$ (KeyMask$, TimeOut&) - timeout_ms < 0 waits forever, which
   is what the bare syntax asks for.  Keys the mask rejects are discarded, as
   the official page states, and the slice is released while waiting. */
char* pb_graphic_waitkey(const char* mask, int timeout_ms) {
    unsigned char b[2];
    int waited = 0, n;
    for (;;) {
        pb_gr_pump();
        if (g_gr_kn > 0) {
            unsigned char ch = (g_gr_keys[0] == 0 && g_gr_kn >= 2)
                             ? g_gr_keys[1] : g_gr_keys[0];
            n = pb_gr_key_pop(b);
            if (pb_gr_mask_ok((const unsigned char*)mask, ch)) {
                return pb_bstr_alloc((const char*)b, (unsigned int)n);
            }
            continue;
        }
        if (timeout_ms >= 0 && waited >= timeout_ms) return pb_bstr_alloc("", 0);
        Sleep(1);
        waited += 1;
    }
}

/* ---- line input ----------------------------------------------------------
   GRAPHIC LINE INPUT and GRAPHIC INPUT both read a whole line; INPUT then
   splits it on commas, one field per destination variable. */
#define PB_GR_LINECAP 260
#define PB_GR_MAXFIELD 32
static char g_gr_line[PB_GR_LINECAP];
static int  g_gr_field_start[PB_GR_MAXFIELD];
static int  g_gr_field_len[PB_GR_MAXFIELD];
static int  g_gr_field_count = 0;

static void pb_gr_echo_char(char c) {
    char buf[2];
    buf[0] = c;
    buf[1] = '\0';
    pb_graphic_print_str(buf);
}

/* Blocking, like the statement it backs.  ENTER ends the line, backspace
   edits it, extended keys are ignored. */
static int pb_gr_read_line(int echo) {
    unsigned char b[2];
    int n, len = 0;
    for (;;) {
        pb_gr_pump();
        if (g_gr_kn <= 0) { Sleep(1); continue; }
        n = pb_gr_key_pop(b);
        if (n != 1) continue;
        if (b[0] == 13 || b[0] == 10) break;
        if (b[0] == 8 || b[0] == 127) {
            if (len > 0) len--;
            continue;
        }
        if (b[0] < 32) continue;
        if (len < PB_GR_LINECAP - 1) {
            g_gr_line[len++] = (char)b[0];
            if (echo) pb_gr_echo_char((char)b[0]);
        }
    }
    g_gr_line[len] = '\0';
    return len;
}

static void pb_gr_split_fields(void) {
    int i = 0, f = 0, start = 0;
    for (;;) {
        if (g_gr_line[i] == ',' || g_gr_line[i] == '\0') {
            if (f < PB_GR_MAXFIELD) {
                g_gr_field_start[f] = start;
                g_gr_field_len[f] = i - start;
                f++;
            }
            if (g_gr_line[i] == '\0') break;
            start = i + 1;
        }
        i++;
    }
    g_gr_field_count = f;
}

/* GRAPHIC LINE INPUT ["prompt"] var$ - the whole line, up to 255 characters. */
char* pb_graphic_line_input(const char* prompt) {
    if (prompt && prompt[0]) pb_graphic_print_str(prompt);
    pb_gr_read_line(1);
    return pb_bstr_alloc(g_gr_line, (unsigned int)strlen(g_gr_line));
}

/* GRAPHIC INPUT [prompt,] varlist - reads the line, then hands out one field
   per call so each destination keeps its own type. */
int pb_graphic_input_begin(const char* prompt) {
    if (prompt && prompt[0]) pb_graphic_print_str(prompt);
    pb_gr_read_line(1);
    pb_gr_split_fields();
    return g_gr_field_count;
}
char* pb_graphic_input_field(int idx) {
    if (idx < 0 || idx >= g_gr_field_count) return pb_bstr_alloc("", 0);
    return pb_bstr_alloc(g_gr_line + g_gr_field_start[idx],
                         (unsigned int)g_gr_field_len[idx]);
}
double pb_graphic_input_field_num(int idx) {
    char buf[64];
    int k, n;
    if (idx < 0 || idx >= g_gr_field_count) return 0.0;
    n = g_gr_field_len[idx];
    if (n > 63) n = 63;
    for (k = 0; k < n; k++) buf[k] = g_gr_line[g_gr_field_start[idx] + k];
    buf[n] = '\0';
    return atof(buf);
}

static long long __stdcall pb_graphic_wndproc(void* hWnd, unsigned int Msg,
                                              unsigned long long wParam,
                                              unsigned long long lParam) {
    if (Msg == 0x000F) {                       /* WM_PAINT */
        unsigned char ps[64];
        for (int k = 0; k < 64; k++) ps[k] = 0;
        void* dc = BeginPaint(hWnd, (void*)ps);
        int i = pb_gw_find(hWnd);
        if (dc && i >= 0 && pb_gw_dc[i]) {
            unsigned char rc[16];
            for (int k = 0; k < 16; k++) rc[k] = 0;
            if (GetClientRect(hWnd, (void*)rc)) {
                int x = 0, y = 0, w = 0, h = 0;
                pb_gw_rect(rc, &x, &y, &w, &h);
                BitBlt(dc, x, y, w, h, pb_gw_dc[i], x, y, PB_GW_SRCCOPY);
            }
        }
        EndPaint(hWnd, (void*)ps);
        return 0;
    }
    if (Msg == 0x0002) {                       /* WM_DESTROY */
        int i = pb_gw_find(hWnd);
        if (i >= 0) pb_gw_free_slot(i);
        if (pb_gw_cur == hWnd) pb_gw_cur = 0;
        if (g_gr_dc_win == hWnd) pb_graphic_release_dc();
        return 0;
    }
    if (Msg == 0x0010) {                       /* WM_CLOSE */
        if (pb_get_winlong(hWnd, PB_GWLP_USERDATA) == PB_GW_STABLE) return 0;
        DestroyWindow(hWnd);
        return 0;
    }
    if (Msg == 0x0112) {                       /* WM_SYSCOMMAND */
        if ((wParam & 0xFFF0) == (unsigned long long)PB_SC_CLOSE
            && pb_get_winlong(hWnd, PB_GWLP_USERDATA) == PB_GW_STABLE) return 0;
        return DefWindowProcA(hWnd, Msg, wParam, lParam);
    }
    if (Msg == 0x0102) {                       /* WM_CHAR (batch 185) */
        unsigned int ch = (unsigned int)(wParam & 0xFFFF);
        if (ch >= 0x20 || (ch >= 0x01 && ch <= 0x1F)) pb_gr_key_push((unsigned char)ch);
        return 0;                              /* no echo, no system beep */
    }
    if (Msg == 0x0100) {                       /* WM_KEYDOWN (batch 185) */
        unsigned int vk = (unsigned int)(wParam & 0xFFFF);
        if (pb_gr_is_ext_key(vk)) {
            pb_gr_key_push(0);                 /* extended key: NUL + scan code */
            pb_gr_key_push((unsigned char)vk);
            return 0;
        }
        return DefWindowProcA(hWnd, Msg, wParam, lParam);
    }
    if (Msg == 0x0201 || Msg == 0x0203) {      /* WM_LBUTTONDOWN / WM_LBUTTONDBLCLK */
        int i = pb_gw_find(hWnd);
        if (i >= 0) {
            if (Msg == 0x0203) pb_gw_click[i] = 2;
            else if (pb_gw_click[i] != 2) pb_gw_click[i] = 1;
            pb_gw_cx[i] = (int)(short)(lParam & 0xFFFF);
            pb_gw_cy[i] = (int)(short)((lParam >> 16) & 0xFFFF);
        }
        return 0;
    }
    return DefWindowProcA(hWnd, Msg, wParam, lParam);
}

static void pb_gw_register(void) {
    static int done = 0;
    if (done) return;
    pb_wndclassex_t wc;
    memset(&wc, 0, sizeof(wc));
    wc.cbSize = sizeof(wc);
    wc.style = 0x0008;                 /* CS_DBLCLKS: GRAPHIC WINDOW CLICK's double */
    wc.lpfnWndProc = (void*)pb_graphic_wndproc;
    wc.hInstance = GetModuleHandleA(0);
    wc.hCursor = LoadCursorA(0, (const char*)32512);
    wc.lpszClassName = PB_GW_CLASS;
    RegisterClassExA(&wc);
    done = 1;
}

static void* pb_gw_create(const char* cap, int x, int y, int w, int h,
                          void* font, int show) {
    pb_gw_register();
    int i = -1;
    for (int k = 0; k < PB_GW_SLOTS; k++) if (!pb_gw_win[k]) { i = k; break; }
    if (i < 0) return 0;
    unsigned long style = 0x00CF0000u | (show ? 0x10000000u : 0u);
    void* hwnd = CreateWindowExA(0, PB_GW_CLASS, cap, style, x, y, w, h,
                                 0, 0, GetModuleHandleA(0), 0);
    if (!hwnd) return 0;
    pb_gw_win[i] = hwnd;
    if (font) SendMessageA(hwnd, 0x0030 /* WM_SETFONT */,
                           (pb_wparam_t)(size_t)font, 1);
    if (!pb_gw_alloc(i)) { DestroyWindow(hwnd); return 0; }
    if (show) { ShowWindow(hwnd, 5); UpdateWindow(hwnd); }
    /* "if there is no selected graphic target at the time of creation, the
       new Graphic Window is automatically attached and selected" */
    g_gr_dc = pb_gw_dc[i];
    g_gr_bmp = pb_gw_bmp[i];
    g_gr_dc_win = 0;
    pb_gw_cur = hwnd;
    return hwnd;
}

void* pb_graphic_window_new(const char* cap, int x, int y, int w, int h,
                            void* font, int show) {
    return pb_gw_create(cap, x, y, w, h, font, show);
}

void* pb_graphic_window_text(const char* cap, int x, int y, int rows, int cols,
                             void* font, int show) {
    /* The TEXT form sizes the window in rows and columns.  The help page does
       not name a cell size; 8x16 (the system fixed-pitch cell) is used so the
       mapping is explicit rather than accidental. */
    if (rows <= 0) rows = 1;
    if (cols <= 0) cols = 1;
    return pb_gw_create(cap, x, y, cols * 8, rows * 16, font, show);
}

static void* pb_gw_target(void* h) {
    if (h) return h;
    if (pb_gw_cur) return pb_gw_cur;
    return g_gr_dc_win;                 /* a window attached with GRAPHIC ATTACH */
}

int pb_graphic_window_end(void* h) {
    void* w = pb_gw_target(h);
    if (!w) return 0;
    if (g_gr_dc_win == w) pb_graphic_release_dc();
    if (pb_gw_cur == w) { pb_gw_cur = 0; g_gr_dc = 0; g_gr_bmp = 0; }
    return DestroyWindow(w) ? 1 : 0;
}

int pb_graphic_window_hide(void* h) {
    void* w = pb_gw_target(h);
    if (!w) return 0;
    ShowWindow(w, 0);                   /* SW_HIDE */
    return 1;
}

int pb_graphic_window_normalize(void* h) {
    void* w = pb_gw_target(h);
    if (!w) return 0;
    ShowWindow(w, 9);                   /* SW_RESTORE: clears HIDE and MINIMIZE */
    return 1;
}

int pb_graphic_window_minimize(void* h) {
    void* w = pb_gw_target(h);
    if (!w) return 0;
    ShowWindow(w, 6);                   /* SW_MINIMIZE */
    return 1;
}

int pb_graphic_window_stabilize(void* h, int on) {
    void* w = pb_gw_target(h);
    if (!w) return 0;
    pb_set_winlong(w, PB_GWLP_USERDATA, on ? (long long)PB_GW_STABLE : 0);
    void* menu = GetSystemMenu(w, 0);
    /* MF_BYCOMMAND = 0, MF_GRAYED = 1, MF_ENABLED = 0 */
    if (menu) EnableMenuItem(menu, PB_SC_CLOSE, on ? 0x00000001u : 0x00000000u);
    return 1;
}

int pb_graphic_window_click(void* h, int* click, float* px, float* py) {
    void* w = pb_gw_target(h);
    if (!w) return 0;
    int i = pb_gw_find(w);
    long long c = 0;
    int cx = 0, cy = 0;
    if (i >= 0) {
        c = pb_gw_click[i];
        cx = pb_gw_cx[i];
        cy = pb_gw_cy[i];
        pb_gw_click[i] = 0;             /* "since the last time this statement ran" */
    }
    /* the help page types the three results as click& (LONG) and x!/y! (SINGLE);
       each is written through its own width so no slot is over-written */
    if (click) *click = (int)c;
    if (px) *px = (float)cx;
    if (py) *py = (float)cy;
    return 1;
}

static void pb_gw_touch(void) {
    /* A drawing statement has written into a graph window's own buffer, so the
       window has to be marked dirty: nothing else in this runtime invalidates a
       window, and without this the screen keeps showing what the last WM_PAINT
       blitted (the sample caught exactly that - GRAPHIC CLEAR changed the
       buffer while the window stayed white until it was uncovered).
       The call sites are the GDI drawing statements, right after their DC
       guard, so an invalidate also happens before the pixels are written -
       harmless, the paint itself only runs on the next UpdateWindow or
       message-loop turn, by which time the drawing is done. */
    if (!pb_gw_cur) return;
    int i = pb_gw_find(pb_gw_cur);
    if (i < 0 || g_gr_dc != pb_gw_dc[i]) return;
    InvalidateRect(pb_gw_cur, 0, 0);
}

/* ---------------------------------------------------------------------------
   ACCEL ATTACH (batch 211)
   The first of the five parked statements (ACCEL_ATTACH / EVENT_SOURCE /
   EVENTS / RAISEEVENT / INSTANCE - one shared empty arm until batch 209 made
   the drop visible) to get a real runtime.

   Record layout: the program passes an array of QUAD records, one per key:
       bits  0-15   fVirt   Win32 FVIRTKEY / FSHIFT / FCONTROL / FALT
       bits 16-31   key     virtual key code
       bits 32-47   cmd     command id, reported as WM_COMMAND's low word
   The compiler passes the array's declared element count (ArrayInfo::
   total_elements); the official form has no count argument because a compiler
   is expected to know the size of a constant array.  NOTE: the machine's
   official help carries no ACCEL_ATTACH page, so this layout is the fork's own
   documented shape, not a transcription of a PowerBASIC page.

   Attached tables are remembered as hDlg -> hAccel and offered to every message
   the two pumps see, which is what makes a key actually fire; a table nobody
   consults would be the same no-op the statement used to be.
   ------------------------------------------------------------------------- */
typedef struct {
    unsigned short fVirt;
    unsigned short key;
    unsigned short cmd;
} pb_accel_rec;

__declspec(dllimport) void* __stdcall CreateAcceleratorTableA(void* lpaccel, int cAccel);
__declspec(dllimport) int   __stdcall DestroyAcceleratorTable(void* hAccel);
__declspec(dllimport) int   __stdcall TranslateAcceleratorA(void* hWnd, void* hAccTable, void* lpMsg);

#define PB_ACCEL_MAX 8
static void* pb_accel_tbl[PB_ACCEL_MAX];
static void* pb_accel_dlg[PB_ACCEL_MAX];
static int   pb_accel_n = 0;

/* Build a table from the program's records; returns the handle, or 0 when the
   array is unusable (null pointer, no elements, or the Win32 call failed). */
long long pb_accel_attach(void* hDlg, unsigned long long* keys, long count) {
    pb_accel_rec recs[32];
    void* hAccel;
    long i, n;
    if (!keys || count <= 0) return 0;
    n = count > 32 ? 32 : count;
    for (i = 0; i < n; i++) {
        unsigned long long v = keys[i];
        recs[i].fVirt = (unsigned short)(v & 0xFFFF);
        recs[i].key   = (unsigned short)((v >> 16) & 0xFFFF);
        recs[i].cmd   = (unsigned short)((v >> 32) & 0xFFFF);
    }
    hAccel = CreateAcceleratorTableA(recs, (int)n);
    if (!hAccel) return 0;
    if (pb_accel_n < PB_ACCEL_MAX) {
        pb_accel_dlg[pb_accel_n] = hDlg;
        pb_accel_tbl[pb_accel_n] = hAccel;
        pb_accel_n++;
    }
    return (long long)hAccel;
}

/* ACCEL DETACH hAccel - destroy one table and forget it. */
long long pb_accel_detach(void* hAccel) {
    int i;
    if (!hAccel) return 0;
    for (i = 0; i < pb_accel_n; i++) {
        if (pb_accel_tbl[i] == hAccel) {
            DestroyAcceleratorTable(hAccel);
            pb_accel_n--;
            pb_accel_tbl[i] = pb_accel_tbl[pb_accel_n];
            pb_accel_dlg[i] = pb_accel_dlg[pb_accel_n];
            pb_accel_tbl[pb_accel_n] = 0;
            pb_accel_dlg[pb_accel_n] = 0;
            return 1;
        }
    }
    return 0;
}

void pb_accel_kill_all(void) {
    int i;
    for (i = 0; i < pb_accel_n; i++) {
        if (pb_accel_tbl[i]) DestroyAcceleratorTable(pb_accel_tbl[i]);
        pb_accel_tbl[i] = 0;
        pb_accel_dlg[i] = 0;
    }
    pb_accel_n = 0;
}

/* Offer the message to every attached table.  Returns 1 when one consumed it -
   the caller must then skip TranslateMessage/DispatchMessage, exactly like the
   accelerator handling in a hand-written Win32 message loop. */
static int pb_accel_translate(pb_msg_t* msg) {
    int i, hit = 0;
    for (i = 0; i < pb_accel_n; i++) {
        if (pb_accel_tbl[i] && TranslateAcceleratorA(pb_accel_dlg[i], pb_accel_tbl[i], msg))
            hit = 1;
    }
    return hit;
}

void pb_message_loop(void) {
    pb_msg_t msg;
    while (GetMessageA(&msg, 0, 0, 0) > 0) {
        if (pb_accel_translate(&msg)) continue;
        TranslateMessage(&msg);
        DispatchMessageA(&msg);
    }
}

int pb_msgbox_impl(void* text, void* title, unsigned int style) {
    return MessageBoxA(0, (const char*)text, (const char*)title, style);
}

int pb_show_modal_to(void* hDlg) {
    /* DIALOG SHOW MODAL hDlg TO result - official PB form (batch 238:
       HelloDDT from C:\PBWin10\samples\DDT).  Runs the same message loop;
       when DIALOG END records pb_modal_result and destroys the dialog
       (WM_DESTROY -> PostQuitMessage), GetMessage returns 0 and the
       recorded modal result is handed back to the caller.
       batch 243: IsDialogMessage routes Enter/Tab to the dialog so a
       BS_DEFPUSHBUTTON control fires like a native default button. */
    pb_msg_t msg;
    pb_modal_done = 0;
    pb_modal_result = 0;
    int gm = 0;
    while ((gm = GetMessageA(&msg, 0, 0, 0)) > 0) {
        if (pb_accel_translate(&msg)) continue;
        if (hDlg && IsDialogMessageA(hDlg, &msg)) continue;
        TranslateMessage(&msg);
        DispatchMessageA(&msg);
    }
    return pb_modal_result;
}

/* Tier-3 DDT GUI #2: CONTROL ADD BUTTON */
__declspec(dllimport) void* __stdcall CreateWindowExA(
    unsigned long, const char*, const char*, unsigned long,
    int, int, int, int, void*, void*, void*, void*);

void* pb_control_add_button(void* parent, long id, const char* text,
                            int x, int y, int w, int h) {
    pb_dlu_to_px(&x, &y, &w, &h);
    /* batch 243/244: PBWin default button style measured from the original
       HelloDDT.EXE = %WS_TABSTOP (0x10000) only (Cancel showed 0x50010000 =
       WS_CHILD|WS_VISIBLE|WS_TABSTOP, no BS_* bits; single-line button text
       is centred by the control itself, so BS_CENTER/BS_VCENTER are not
       needed and PBWin does not set them).  NOTE: BS_VCENTER is 0x0C00, NOT
       0x0C (0x0C = BS_SPLITBUTTON, which draws a drop-down arrow).  The old
       hard-coded BS_DEFPUSHBUTTON (0x01) made every button a default button;
       PBWin only gives that look to buttons whose style includes %BS_DEFAULT. */
    unsigned long style = 0x10000 | 0x40000000 | 0x10000000;
    void* hw = CreateWindowExA(0, "BUTTON", text, style,
                            x, y, w, h, parent,
                            (void*)(long long)id, GetModuleHandleA(0), 0);
    if (hw) SendMessageA(hw, 0x0030 /* WM_SETFONT */,
                        (unsigned long long)(size_t)pb_dialog_font(), 1);
    return hw;
}

/* CONTROL ADD BUTTON full official syntax (batch 243):
   style&/exstyle& passthrough + optional CALL callback.
   style==0 -> PBWin default (see above); otherwise the user value is used
   directly (PBWin semantics: custom style replaces the default, and the
   runtime adds the WS_CHILD|WS_VISIBLE a control always needs).
   exstyle==0 -> 0; else passed straight to CreateWindowExA. */
void* pb_control_add_button_ex(void* parent, long id, const char* text,
                               int x, int y, int w, int h,
                               long style, long exstyle, void* fn) {
    pb_dlu_to_px(&x, &y, &w, &h);
    unsigned long s = style ? (unsigned long)style : 0x10000;
    s |= 0x40000000u | 0x10000000u; /* WS_CHILD | WS_VISIBLE */
    void* hw = CreateWindowExA((unsigned long)exstyle, "BUTTON", text, s,
                            x, y, w, h, parent,
                            (void*)(long long)id, GetModuleHandleA(0), 0);
    if (hw) {
        SendMessageA(hw, 0x0030 /* WM_SETFONT */,
                    (unsigned long long)(size_t)pb_dialog_font(), 1);
        if (fn) pb_register_callback_hwnd(hw, fn);
    }
    return hw;
}


void* pb_control_add_editbox(void* parent, long id, const char* text,
                              int x, int y, int w, int h) {
    pb_dlu_to_px(&x, &y, &w, &h);
    unsigned long style = 0x80 | 0x40000000 | 0x10000000 | 0x800000 | 0x10000;
    void* hEdit = CreateWindowExA(0x200, "EDIT", text, style,
                            x, y, w, h, parent,
                            (void*)(long long)id, GetModuleHandleA(0), 0);
    if (hEdit) SendMessageA(hEdit, 0x0030 /* WM_SETFONT */,
                            (unsigned long long)(size_t)pb_dialog_font(), 1);
    return hEdit;
}

/* Tier-3 DDT: CONTROL GET/SET TEXT */
__declspec(dllimport) int __stdcall GetWindowTextA(void*, char*, int);
__declspec(dllimport) int __stdcall GetWindowTextLengthA(void*);
__declspec(dllimport) int __stdcall SetWindowTextA(void*, const char*);
__declspec(dllimport) int __stdcall EnableWindow(void*, int);
__declspec(dllimport) void* __stdcall SetFocus(void*);

void pb_dialog_set_text(void* hDlg, const char* text) {
    SetWindowTextA(hDlg, text);
}
/* pb_rect_t is typedef'd above pb_window_new (batch 239) */
__declspec(dllimport) int __stdcall GetWindowRect(void* hWnd, pb_rect_t* lpRect);
/* batch 181 - GRAPHIC REDRAW / SET FOCUS / SET LOC / SET CLIENT.
   Everything these need is declared by this point in the file (pb_rect_t just
   above, the width-portable window-long helpers earlier), so the block lives
   here rather than beside the other GRAPHIC WINDOW statements. */
__declspec(dllimport) int __stdcall SetForegroundWindow(void* hWnd);
__declspec(dllimport) int __stdcall AdjustWindowRectEx(pb_rect_t*, unsigned long, int, unsigned long);
__declspec(dllimport) int __stdcall SetWindowPos(void*, void*, int, int, int, int, unsigned int);

/* The statements say "the selected graphic window".  A GRAPHIC WINDOW window is
   the usual target; failing that, a window/control attached with GRAPHIC ATTACH,
   which is what g_gr_dc_win records.  A buffered bitmap has no window, so these
   statements are the documented no-operation for it. */
static void* pb_graphic_win_target(void) {
    if (pb_gw_cur) return pb_gw_cur;
    if (g_gr_dc_win) return g_gr_dc_win;
    return 0;
}

static int pb_graphic_win_rect(long* x, long* y) {
    void* w = pb_graphic_win_target();
    pb_rect_t rc;
    if (!w) return 0;
    if (!GetWindowRect(w, &rc)) return 0;
    *x = rc.left;
    *y = rc.top;
    return 1;
}

int pb_graphic_redraw(void) {
    void* w = pb_graphic_win_target();
    if (!w) return 0;                   /* nothing buffered: already on screen */
    InvalidateRect(w, 0, 0);
    UpdateWindow(w);
    return 1;
}

int pb_graphic_set_focus(void) {
    void* w = pb_graphic_win_target();
    if (!w) return 0;
    SetForegroundWindow(w);
    SetFocus(w);
    return 1;
}

int pb_graphic_set_loc(long long x, long long y) {
    void* w = pb_graphic_win_target();
    if (!w) return 0;
    SetWindowPos(w, 0, (int)x, (int)y, 0, 0,
                 0x0001u | 0x0004u /* SWP_NOSIZE | SWP_NOZORDER */);
    return 1;
}

int pb_graphic_set_client(long long w, long long h) {
    void* win = pb_graphic_win_target();
    pb_rect_t rc;
    unsigned long style, exstyle;
    if (!win || w <= 0 || h <= 0) return 0;
    rc.left = 0; rc.top = 0;
    rc.right = (long)w; rc.bottom = (long)h;
    style = (unsigned long)pb_get_winlong(win, -16 /* GWL_STYLE */);
    exstyle = (unsigned long)pb_get_winlong(win, -20 /* GWL_EXSTYLE */);
    AdjustWindowRectEx(&rc, style, 0, exstyle);
    SetWindowPos(win, 0, 0, 0, (int)(rc.right - rc.left), (int)(rc.bottom - rc.top),
                 0x0002u | 0x0004u /* SWP_NOMOVE | SWP_NOZORDER */);
    /* The drawing buffer *is* the client area here, and GRAPHIC GET CLIENT
       reports the buffer's dimensions, so it has to follow the window. */
    if (g_gr_bmp) pb_graphic_set_size((int)w, (int)h);
    return 1;
}
__declspec(dllimport) int __stdcall GetSystemMetrics(int nIndex);
__declspec(dllimport) int __stdcall MoveWindow(void* hWnd, int X, int Y, int nWidth, int nHeight, int bRepaint);
void pb_dialog_center(void* hDlg) {
    pb_rect_t rc;
    GetWindowRect(hDlg, &rc);
    int w = rc.right - rc.left;
    int h = rc.bottom - rc.top;
    int sw = GetSystemMetrics(0);
    int sh = GetSystemMetrics(1);
    MoveWindow(hDlg, (sw - w) / 2, (sh - h) / 2, w, h, 1);
}


void pb_combobox_add(void* hCombo, const char* text) {
    SendMessageA(hCombo, 0x143, 0, (pb_lparam_t)text);
}
void pb_listbox_add(void* hList, const char* text) {
    SendMessageA(hList, 0x180, 0, (pb_lparam_t)text);
}

void pb_control_get_text(void* hctrl, char* out, int outlen) {
    int n = GetWindowTextLengthA(hctrl);
    if (n < outlen - 1) {
        GetWindowTextA(hctrl, out, outlen);
    } else {
        out[0] = 0;
    }
}

void pb_control_set_text(void* hctrl, const char* text) {
    SetWindowTextA(hctrl, text);
}

void pb_control_show(void* hctrl) { ShowWindow(hctrl, 5); }
void pb_control_hide(void* hctrl) { ShowWindow(hctrl, 0); }
void pb_control_enable(void* hctrl) { EnableWindow(hctrl, 1); }
void pb_control_disable(void* hctrl) { EnableWindow(hctrl, 0); }
void pb_control_focus(void* hctrl) { SetFocus(hctrl); }

/* CONTROL CHECK / UNCHECK / GET CHECK (uses existing SendMessageA decl at line 5324) */
#define BM_SETCHECK 0x00F1
#define BM_GETCHECK 0x00F0
#define BST_CHECKED 1

void pb_control_check(void* hctrl) { SendMessageA(hctrl, BM_SETCHECK, BST_CHECKED, 0); }
void pb_control_uncheck(void* hctrl) { SendMessageA(hctrl, BM_SETCHECK, 0, 0); }
long long pb_control_get_check(void* hctrl) { return (long long)SendMessageA(hctrl, BM_GETCHECK, 0, 0); }

void* pb_control_add_combobox(void* parent, long id, int x, int y, int w, int h) {
    /* CHW control_add_combobox.htm: default style = %CBS_DROPDOWN(0x2) OR
       %CBS_SORT(0x100) OR %WS_TABSTOP(0x10000); default extended style =
       %WS_EX_LEFT OR %WS_EX_CLIENTEDGE(0x200).  (CBS_DROPDOWN is the editable
       combo; 0x3 would be CBS_DROPDOWNLIST - read-only - and our old code
       also added WS_BORDER/WS_VSCROLL which PBWin does not.) */
    unsigned long style = 0x2 | 0x100 | 0x10000 | 0x40000000 | 0x10000000;
    unsigned long ex = 0x200;
    void* hw = CreateWindowExA(ex, "COMBOBOX", "", style,
                            x, y, w, h, parent,
                            (void*)(long long)id, GetModuleHandleA(0), 0);
    if (hw) SendMessageA(hw, 0x0030, (unsigned long long)(size_t)pb_dialog_font(), 1);
    return hw;
}

void* pb_control_add_listbox(void* parent, long id, int x, int y, int w, int h) {
    /* CHW control_add_listbox.htm: default style = %LBS_SORT(0x2) OR
       %LBS_NOTIFY(0x1) OR %WS_TABSTOP(0x10000) OR %WS_VSCROLL(0x200000);
       default extended style = %WS_EX_CLIENTEDGE(0x200) OR %WS_EX_LEFT. */
    unsigned long style = 0x1 | 0x2 | 0x200000 | 0x10000 | 0x40000000 | 0x10000000;
    unsigned long ex = 0x200;
    void* hw = CreateWindowExA(ex, "LISTBOX", "", style,
                            x, y, w, h, parent,
                            (void*)(long long)id, GetModuleHandleA(0), 0);
    if (hw) SendMessageA(hw, 0x0030, (unsigned long long)(size_t)pb_dialog_font(), 1);
    return hw;
}

/* CONTROL ADD LISTBOX full official syntax (batch 245):
   CONTROL ADD LISTBOX, hDlg, id&, [items$() | txt$ | <empty>], x, y, xx, yy
     [, [style&] [, [exstyle&]]] [[,] CALL callback]
   items is a pointer (array ref / string / 0); filling WSTRING arrays is
   deferred (see SKILL batch 245 note).  x/y/w/h are DLU (converted like
   BUTTON/TEXTBOX).  Height: measured original SYNCLIST 65 DLU -> 95 px
   (PB_DLU_Y_TO_PX gives 106; the listbox CLIENTEDGE/inset crops ~11 px).
   style==0 && exstyle==0 -> CHW default style (LBS_NOTIFY|LBS_SORT|
   WS_TABSTOP|WS_VSCROLL) with default ex WS_EX_CLIENTEDGE(0x200)|LEFT.
   style!=0 -> user value used directly, but PBWin always adds LBS_NOTIFY
   (measured original SYNCLIST: user WS_VSCROLL|WS_TABSTOP = 0x210000 became
   0x50210001 with LBS_NOTIFY set).  exstyle is passed straight. */
void* pb_control_add_listbox_ex(void* parent, long id, void* items, long long count,
                                int x, int y, int w, int h,
                                long style, long exstyle, void* fn) {
    x = PB_DLU_X_TO_PX(x);
    y = PB_DLU_Y_TO_PX(y);
    w = PB_DLU_X_TO_PX(w);
    int hpx = PB_DLU_Y_TO_PX(h) - 11;
    unsigned long ex = (style == 0 && exstyle == 0) ? 0x200u
                                                    : (unsigned long)exstyle;
    unsigned long s = style ? ((unsigned long)style | 0x1u)  /* + LBS_NOTIFY */
                            : (0x1u | 0x2u | 0x200000u | 0x10000u);
    s |= 0x40000000u | 0x10000000u; /* WS_CHILD | WS_VISIBLE */
    void* hw = CreateWindowExA(ex, "LISTBOX", "", s,
                           x, y, w, hpx, parent,
                           (void*)(long long)id, GetModuleHandleA(0), 0);
    if (hw) {
        SendMessageA(hw, 0x0030, (unsigned long long)(size_t)pb_dialog_font(), 1);
        /* CreateWindowExA auto-adds WS_CLIPSIBLINGS, and the listbox (without
           LBS_NOINTEGRALHEIGHT) snaps height to integral rows and WM_SETFONT
           even drops WS_VSCROLL; PBWin keeps the requested style and DLU size
           exactly (measured original SYNCLIST: style 0x50210001 with WS_VSCROLL,
           225x95).  Restore size first, then style - SetWindowLong must be
           last or the listbox undoes WS_VSCROLL again. */
        SetWindowPos(hw, 0, x, y, w, hpx, 0x0004u | 0x0010u);
        pb_set_winlong(hw, -16 /* GWL_STYLE */, (long long)s);
        /* items$() full-array reference: with batch 248's DIM type-inheritance
           fix the array is [N x Ptr], each element a full 8-byte WSTRING
           pointer.  Add every item with LB_ADDSTRING (0x0180). */
        if (count > 0 && items) {
            long long i;
            for (i = 0; i < count; i++) {
                void* p = ((void**)items)[i];
                if (!p) continue;
                SendMessageA(hw, 0x0180 /* LB_ADDSTRING */, 0,
                             (long long)(size_t)p);
            }
        }
        if (fn) pb_register_callback_hwnd(hw, fn);
    }
    return hw;
}

/* CONTROL ADD COMBOBOX full official syntax (batch 248):
   CONTROL ADD COMBOBOX, hDlg, id&, [items$()], x, y, xx, yy
     [, [style&] [, [exstyle&]]] [[,] CALL callback]
   Mirrors pb_control_add_listbox_ex: DLU conversion, items$() array fill
   (CB_ADDSTRING), style/exstyle passthrough, callback registration.
   CHW control_add_combobox.htm default style = %CBS_DROPDOWN(0x2) OR
   %CBS_SORT(0x100) OR %WS_TABSTOP(0x10000), default ex = %WS_EX_LEFT OR
   %WS_EX_CLIENTEDGE(0x200).  style!=0: user style used directly; a CBS type
   bit (0x3 mask) is forced to DROPDOWN when absent so the combo stays
   editable (listbox batch 245 forces LBS_NOTIFY analogously). */
void* pb_control_add_combobox_ex(void* parent, long id, void* items, long long count,
                                 int x, int y, int w, int h,
                                 long style, long exstyle, void* fn) {
    x = PB_DLU_X_TO_PX(x);
    y = PB_DLU_Y_TO_PX(y);
    w = PB_DLU_X_TO_PX(w);
    int hpx = PB_DLU_Y_TO_PX(h);
    unsigned long ex = (style == 0 && exstyle == 0) ? 0x200u
                                                    : (unsigned long)exstyle;
    unsigned long s;
    if (style) {
        s = (unsigned long)style;
        if ((s & 0x3u) == 0) s |= 0x2u; /* CBS_DROPDOWN if no CBS type bit */
    } else {
        s = 0x2u | 0x100u | 0x10000u;    /* CBS_DROPDOWN|CBS_SORT|WS_TABSTOP */
    }
    s |= 0x40000000u | 0x10000000u; /* WS_CHILD | WS_VISIBLE */
    void* hw = CreateWindowExA(ex, "COMBOBOX", "", s,
                           x, y, w, hpx, parent,
                           (void*)(long long)id, GetModuleHandleA(0), 0);
    if (hw) {
        SendMessageA(hw, 0x0030, (unsigned long long)(size_t)pb_dialog_font(), 1);
        /* items$() full-array reference: with batch 248's DIM type-inheritance
           fix the array is [N x Ptr], each element a full 8-byte WSTRING
           pointer.  Add every item with CB_ADDSTRING (0x0143). */
        if (count > 0 && items) {
            long long i;
            for (i = 0; i < count; i++) {
                void* p = ((void**)items)[i];
                if (!p) continue;
                SendMessageA(hw, 0x0143 /* CB_ADDSTRING */, 0,
                             (long long)(size_t)p);
            }
        }
        if (fn) pb_register_callback_hwnd(hw, fn);
    }
    return hw;
}


void* pb_control_add_checkbox(void* parent, long id, const char* text,
                                int x, int y, int w, int h) {
    pb_dlu_to_px(&x, &y, &w, &h);
    /* CHW control_add_checkbox.htm: default style = %BS_LEFT(0x100) OR
       %BS_VCENTER(0xC00) OR %WS_TABSTOP(0x10000), plus the control type
       BS_AUTOCHECKBOX(0x3).  (BS_VCENTER is 0x0C00, not 0x0C.) */
    unsigned long style = 0x3 | 0x100 | 0xC00 | 0x40000000 | 0x10000000 | 0x10000;
    void* hw = CreateWindowExA(0, "BUTTON", text, style,
                            x, y, w, h, parent,
                            (void*)(long long)id, GetModuleHandleA(0), 0);
    if (hw) SendMessageA(hw, 0x0030, (unsigned long long)(size_t)pb_dialog_font(), 1);
    return hw;
}

void* pb_control_add_radiobutton(void* parent, long id, const char* text,
                                int x, int y, int w, int h) {
    pb_dlu_to_px(&x, &y, &w, &h);
    unsigned long style = 0x9 | 0x40000000 | 0x10000000 | 0x10000;
    void* hw = CreateWindowExA(0, "BUTTON", text, style,
                            x, y, w, h, parent,
                            (void*)(long long)id, GetModuleHandleA(0), 0);
    if (hw) SendMessageA(hw, 0x0030, (unsigned long long)(size_t)pb_dialog_font(), 1);
    return hw;
}

void* pb_control_add_groupbox(void* parent, long id, const char* text,
                               int x, int y, int w, int h) {
    pb_dlu_to_px(&x, &y, &w, &h);
    unsigned long style = 0x7 | 0x40000000 | 0x10000000;
    void* hw = CreateWindowExA(0, "BUTTON", text, style,
                            x, y, w, h, parent,
                            (void*)(long long)id, GetModuleHandleA(0), 0);
    if (hw) SendMessageA(hw, 0x0030, (unsigned long long)(size_t)pb_dialog_font(), 1);
    return hw;
}

/* === batch 177: the plain and static CONTROL family ==================
   Sources: control_add_option.htm, control_add_check3state.htm,
   control_add_frame.htm, control_add_textbox.htm, control_add_line.htm,
   control_set_option.htm.  Each helper spells out the documented default
   style set, and every one runs pb_dlu_to_px() so that creation and the
   CONTROL GET/SET geometry helpers (which divide by PB_CTL_DLU_X/Y) agree. */

/* CONTROL ADD OPTION - DDT's radio button ("just like a conventional
   'radio button' control").  Class BUTTON, type BS_AUTORADIOBUTTON=0x9;
   default style %WS_TABSTOP | %BS_LEFT(0x100) | %BS_VCENTER(0xC00). */
void* pb_control_add_option(void* parent, long id, const char* text,
                            int x, int y, int w, int h) {
    pb_dlu_to_px(&x, &y, &w, &h);
    unsigned long style = 0x9 | 0x100 | 0xC00 | 0x40000000 | 0x10000000 | 0x10000;
    void* hw = CreateWindowExA(0, "BUTTON", text, style,
                           x, y, w, h, parent,
                           (void*)(long long)id, GetModuleHandleA(0), 0);
    if (hw) SendMessageA(hw, 0x0030, (unsigned long long)(size_t)pb_dialog_font(), 1);
    return hw;
}

/* CONTROL ADD CHECK3STATE - auto 3-state checkbox (True / False /
   Indeterminate).  Class BUTTON, type BS_AUTO3STATE=0x6; default style
   %BS_LEFT | %BS_VCENTER | %WS_TABSTOP. */
void* pb_control_add_check3state(void* parent, long id, const char* text,
                                 int x, int y, int w, int h) {
    pb_dlu_to_px(&x, &y, &w, &h);
    unsigned long style = 0x6 | 0x100 | 0xC00 | 0x40000000 | 0x10000000 | 0x10000;
    void* hw = CreateWindowExA(0, "BUTTON", text, style,
                           x, y, w, h, parent,
                           (void*)(long long)id, GetModuleHandleA(0), 0);
    if (hw) SendMessageA(hw, 0x0030, (unsigned long long)(size_t)pb_dialog_font(), 1);
    return hw;
}

/* CONTROL ADD FRAME - the "group" box.  Class BUTTON, type
   BS_GROUPBOX=0x7; default style %BS_LEFT | %BS_TOP(0x400).  Two details
   straight from the help page: the syntax has NO CALL callback operand for
   FRAME, and %BS_TOP is persistent - FRAME does not support %BS_BOTTOM. */
void* pb_control_add_frame(void* parent, long id, const char* text,
                           int x, int y, int w, int h) {
    pb_dlu_to_px(&x, &y, &w, &h);
    unsigned long style = 0x7 | 0x100 | 0x400 | 0x40000000 | 0x10000000;
    void* hw = CreateWindowExA(0, "BUTTON", text, style,
                           x, y, w, h, parent,
                           (void*)(long long)id, GetModuleHandleA(0), 0);
    if (hw) SendMessageA(hw, 0x0030, (unsigned long long)(size_t)pb_dialog_font(), 1);
    return hw;
}

/* ---- the resource-image control family (batch 178) ---------------------
   CONTROL ADD IMAGE / IMAGEX display a bitmap or icon in a STATIC;
   CONTROL ADD IMGBUTTON / IMGBUTTONX do the same in a BUTTON.  The X form
   resizes the image to fit the control, the plain form draws it at its
   natural size (official help: control_add_image.htm, control_add_imagex.htm,
   control_add_imgbutton.htm, control_add_imgbuttonx.htm - "The bitmap or icon
   used in the image is not resized to fit the control" / "...is resized to
   fit the control").

   The image is named the way the IMAGELIST statements name theirs, because
   the policy already lives in pb_il_load_name() above: a leading '#' is the
   integral resource id, a name containing a period is a disk file, anything
   else is a resource name tried before the disk file.  The documented form is
   the resource form, whose handle is shared and therefore never leaked; a disk
   file is loaded as a private copy (LR_LOADFROMFILE) that the control does not
   own, so it stays alive for the life of the process.

   Format.  The help lets the caller name the format - %SS_ICON(0x3) /
   %SS_BITMAP(0xE) for the STATIC forms, %BS_ICON(0x40) / %BS_BITMAP(0x80) for
   the BUTTON forms - and says that when it is not named, "PowerBASIC will
   examine the file to determine the correct image format".  The style clause
   is not reachable from the parser yet (every CONTROL ADD form in this fork
   stops at the eight documented operands), so the format is always discovered:
   icon first, then bitmap, the same order the IMAGELIST loaders use.  The
   discovered bit is then ADDED to the window style, because a STATIC or a
   BUTTON only paints an image when its style says which kind it is holding.

   Stretch.  A STATIC gets %SS_REALSIZECONTROL(0x40) for the X form, which
   keeps the bitmap fitted even if the control is resized later; a BUTTON has
   no such style bit, so the X form asks LoadImage to scale the resource to the
   control's pixel rectangle instead.

   style == -1 means the clause was omitted: %WS_EX_LEFT(0) for the extended
   style, no primary style for a STATIC, and the documented %WS_TABSTOP
   default for IMGBUTTON.  %WS_CHILD | %WS_VISIBLE are always added, the same
   way every other CONTROL ADD helper in this file does it. */
__declspec(dllimport) long __stdcall GetWindowLongA(void* hWnd, int nIndex);

static void* pb_image_probe(const char* name, unsigned int* type_out, int cx, int cy) {
    int owned = 0;
    void* h = pb_il_load_name(name, 1, cx, cy, &owned);   /* IMAGE_ICON */
    if (h) { *type_out = 1; return h; }
    h = pb_il_load_name(name, 0, cx, cy, &owned);         /* IMAGE_BITMAP */
    *type_out = 0;
    return h;
}

void* pb_control_add_image(void* parent, long id, const char* name,
                           int x, int y, int w, int h,
                           int style, int exstyle, int is_button, int is_x) {
    pb_dlu_to_px(&x, &y, &w, &h);
    unsigned long st = (style < 0) ? 0 : (unsigned long)style;
    unsigned long ex = (exstyle < 0) ? 0 : (unsigned long)exstyle;  /* %WS_EX_LEFT */
    int cx = is_x ? w : 0;
    int cy = is_x ? h : 0;
    unsigned int type = 0;
    int named = 0;
    void* img = 0;

    if (is_button) {
        if (st & 0x80) { type = 0; named = 1; }          /* %BS_BITMAP */
        else if (st & 0x40) { type = 1; named = 1; }     /* %BS_ICON */
    } else {
        unsigned long fmt = st & 0x1F;                   /* %SS_TYPEMASK */
        if (fmt == 0x03) { type = 1; named = 1; }        /* %SS_ICON */
        else if (fmt == 0x0E) { type = 0; named = 1; }   /* %SS_BITMAP */
    }
    if (named) {
        int owned = 0;
        img = pb_il_load_name(name, type, cx, cy, &owned);
    } else {
        img = pb_image_probe(name, &type, cx, cy);
        if (img) {
            if (is_button) st |= (type == 1) ? 0x40 : 0x80;
            else st |= (type == 1) ? 0x03 : 0x0E;
        }
    }
    /* %WS_TABSTOP is the documented default PRIMARY style of an image button,
       and it has to be applied whenever the caller named no primary style.
       Testing `st == 0` instead misses it: the format discovery above has
       already set %BS_ICON / %BS_BITMAP by the time this line runs. */
    if (is_button && style < 0) st |= 0x10000;           /* %WS_TABSTOP default */
    st |= 0x40000000 | 0x10000000;                       /* %WS_CHILD | %WS_VISIBLE */
    if (!is_button && is_x) st |= 0x40;                  /* %SS_REALSIZECONTROL */

    void* hwnd = CreateWindowExA(ex, is_button ? "BUTTON" : "STATIC", "",
                                 st, x, y, w, h, parent,
                                 (void*)(long long)id, GetModuleHandleA(0), 0);
    if (hwnd && img) {
        /* %STM_SETIMAGE(0x172) is the STATIC message, %BM_SETIMAGE(0xF7) the
           BUTTON one; wParam names the kind of image being handed over. */
        SendMessageA(hwnd, is_button ? 0xF7 : 0x172,
                     (pb_wparam_t)type, (pb_lparam_t)(intptr_t)img);
    }
    if (hwnd) SendMessageA(hwnd, 0x0030, (unsigned long long)(size_t)pb_dialog_font(), 1);
    return hwnd;
}

int pb_control_set_image(void* parent, long id, const char* name,
                         int is_button, int is_x) {
    void* hwnd = pb_pb_hwnd(parent, id);
    if (!hwnd || !name || !name[0]) return 0;
    long st = GetWindowLongA(hwnd, -16);                 /* %GWL_STYLE */
    /* The help restricts a replacement to the format already displayed, so the
       format is read back off the control rather than discovered again. */
    unsigned int type;
    if (is_button) type = (st & 0x40) ? 1 : 0;           /* %BS_ICON  else %BS_BITMAP */
    else type = ((st & 0x1F) == 0x03) ? 1 : 0;           /* %SS_ICON  else %SS_BITMAP */
    int cx = 0, cy = 0;
    if (is_x) {
        long rc[4];
        if (GetClientRect(hwnd, rc)) { cx = (int)(rc[2] - rc[0]); cy = (int)(rc[3] - rc[1]); }
    }
    int owned = 0;
    void* img = pb_il_load_name(name, type, cx, cy, &owned);
    if (!img) return 0;
    void* old = (void*)SendMessageA(hwnd, is_button ? 0xF7 : 0x172,
                                    (pb_wparam_t)type, (pb_lparam_t)(intptr_t)img);
    if (old && old != img) {
        /* Both SET statements document this: "When an image is changed,
           CONTROL SET IMAGE automatically releases the old image from
           memory."  A shared resource handle simply fails the delete. */
        if (type == 1) DestroyIcon(old); else DeleteObject(old);
    }
    return 1;
}

/* CONTROL ADD TEXTBOX - a text box, i.e. DDT's edit control.
   Default style %WS_TABSTOP | %ES_LEFT(0) | %ES_AUTOHSCROLL(0x80);
   default extended style %WS_EX_CLIENTEDGE(0x200) with %WS_EX_LEFT(0).
   Measured from the original HelloDDT.EXE (PBWin): Edit style=0x50010080
   (= WS_CHILD|WS_VISIBLE|WS_TABSTOP|ES_AUTOHSCROLL, NO WS_BORDER) with
   exstyle=0x200.  The CHW doc lists %WS_BORDER as part of the default, but
   PBWin's real control has no WS_BORDER bit - trust the measured style.
   txt$ may be empty. */
void* pb_control_add_textbox(void* parent, long id, const char* text,
                             int x, int y, int w, int h) {
    /* TEXTBOX is the one control whose height is not the plain rounded
       DLU value: PBWin's 12-DLU textbox measures 19 px (rounded 20 minus
       the edit border inset).  So x/y/w go through pb_dlu_to_px and the
       height is rounded once, here, minus 1. */
    x = PB_DLU_X_TO_PX(x);
    y = PB_DLU_Y_TO_PX(y);
    w = PB_DLU_X_TO_PX(w);
    int hpx = PB_DLU_Y_TO_PX(h) - 1;
    unsigned long style = 0x80 | 0x10000 | 0x40000000 | 0x10000000;
    unsigned long ex = 0x200;
    void* hw = CreateWindowExA(ex, "EDIT", text, style,
                           x, y, w, hpx, parent,
                           (void*)(long long)id, GetModuleHandleA(0), 0);
    if (hw) SendMessageA(hw, 0x0030, (unsigned long long)(size_t)pb_dialog_font(), 1);
    return hw;
}

/* CONTROL ADD LINE - a line, or an empty/filled rectangle.  Class STATIC;
   the single documented default style is %SS_ETCHEDFRAME(0x12), which is
   what makes the two "line" look.  A LINE never displays its text; the
   string is carried so the program can use it, exactly as documented. */
void* pb_control_add_line(void* parent, long id, const char* text,
                          int x, int y, int w, int h) {
    pb_dlu_to_px(&x, &y, &w, &h);
    unsigned long style = 0x12 | 0x40000000 | 0x10000000;
    void* hw = CreateWindowExA(0, "STATIC", text, style,
                           x, y, w, h, parent,
                           (void*)(long long)id, GetModuleHandleA(0), 0);
    if (hw) SendMessageA(hw, 0x0030, (unsigned long long)(size_t)pb_dialog_font(), 1);
    return hw;
}

/* CONTROL SET OPTION hDlg, id&, minid&, maxid& -- CheckRadioButton() *is*
   this statement: it checks the button whose id is id& and clears the check
   state of every other button in the inclusive range minid&..maxid&. */
__declspec(dllimport) int __stdcall CheckRadioButton(void* hDlg, int nIDFirstButton,
                                                     int nIDLastButton, int nIDCheckButton);
void pb_control_set_option(void* hDlg, long long id, long long minid, long long maxid) {
    CheckRadioButton(hDlg, (int)minid, (int)maxid, (int)id);
}

/* CONTROL ADD SCROLLBAR - CHW control_add_scrollbar.htm: default style is
   %SBS_HORZ(0x0); "if the width is less than the height, the control is
   automatically switched to %SBS_VERT(0x1), regardless of whether %SBS_HORZ
   is specified or not".  The old code hard-coded SBS_VERT + WS_TABSTOP. */
void* pb_control_add_scrollbar(void* parent, long id, int x, int y, int w, int ht) {
    unsigned long sbs = (w < ht) ? 1u : 0u;
    unsigned long style = sbs | 0x40000000u | 0x10000000u;
    void* hbar = CreateWindowExA(0, "SCROLLBAR", "", style,
                            x, y, w, ht, parent,
                            (void*)(long long)id, GetModuleHandleA(0), 0);
    /* Set default range 0-100 so scrollbar works immediately */
    SendMessageA(hbar, 0x00F4 /* SBM_SETRANGE */, 0, 100);
    /* CHW SCROLLBAR_statement.htm: "Upon ScrollBar creation, the default
     * page size is 10."  Win32 defaults nPage to 0, so set it explicitly. */
    if (hbar) {
        struct { unsigned int cbSize; unsigned int fMask; int nMin; int nMax;
                 unsigned int nPage; int nPos; int nTrackPos; } si;
        memset(&si, 0, sizeof(si));
        si.cbSize = (unsigned int)sizeof(si);
        si.fMask = 0x0002 /* SIF_PAGE */;
        si.nPage = 10;
        SendMessageA(hbar, 0x00E9 /* SBM_SETSCROLLINFO */, 0, (long long)&si);
    }
    if (hbar) SendMessageA(hbar, 0x0030, (unsigned long long)(size_t)pb_dialog_font(), 1);
    return hbar;
}

/* CONTROL ADD LABEL */
void* pb_control_add_label(void* parent, long id, const char* text, int x, int y, int w, int ht) {
    /* SS_LEFT=0, WS_CHILD=0x40000000, WS_VISIBLE=0x10000000 */
    unsigned long style = 0x40000000 | 0x10000000;
    pb_dlu_to_px(&x,&y,&w,&ht);
    ht = 16; /* single-line label height, vertically centered with 24px edit */
    void* h = CreateWindowExA(0, "STATIC", text, style,
                           x, y, w, ht, parent,
                           (void*)(long long)id, GetModuleHandleA(0), 0);
    if (h) SendMessageA(h, 0x0030, (unsigned long long)(size_t)pb_dialog_font(), 1);
    return h;
}

/* CONTROL ADD PROGRESSBAR */
void* pb_control_add_progressbar(void* parent, long id, int x, int y, int w, int ht) {
    unsigned long style = 0x00000000 | 0x40000000 | 0x10000000;
    pb_dlu_to_px(&x,&y,&w,&ht);
    void* h = CreateWindowExA(0, "msctls_progress32", "", style,
                           x, y, w, ht, parent,
                           (void*)(long long)id, GetModuleHandleA(0), 0);
    if (h) SendMessageA(h, 0x0030, (unsigned long long)(size_t)pb_dialog_font(), 1);
    return h;
}

/* CONTROL ADD MONTHCAL (batch 190) - class "SysMonthCal32" */
void* pb_control_add_monthcal(void* parent, long id, int x, int y, int w, int ht) {
    unsigned long style = 0x40000000 | 0x10000000 | 0x00010000;
    pb_dlu_to_px(&x,&y,&w,&ht);
    void* h = CreateWindowExA(0, "SysMonthCal32", "", style,
                           x, y, w, ht, parent,
                           (void*)(long long)id, GetModuleHandleA(0), 0);
    if (h) SendMessageA(h, 0x0030, (unsigned long long)(size_t)pb_dialog_font(), 1);
    return h;
}

/* CONTROL ADD ANIMATE (batch 190) - class "SysAnimate32" */
void* pb_control_add_animate(void* parent, long id, int x, int y, int w, int ht) {
    unsigned long style = 0x40000000 | 0x10000000;
    pb_dlu_to_px(&x,&y,&w,&ht);
    void* h = CreateWindowExA(0, "SysAnimate32", "", style,
                           x, y, w, ht, parent,
                           (void*)(long long)id, GetModuleHandleA(0), 0);
    if (h) SendMessageA(h, 0x0030, (unsigned long long)(size_t)pb_dialog_font(), 1);
    return h;
}

/* CONTROL ADD RICHEDIT (batch 190) - class "RICHEDIT50W" */
void* pb_control_add_richedit(void* parent, long id, int x, int y, int w, int ht) {
    unsigned long style = 0x40000000 | 0x10000000 | 0x00010000 | 0x00800000 | 0x4 | 0x40;
    LoadLibraryA("msftedit.dll");  /* registers RICHEDIT50W */
    pb_dlu_to_px(&x,&y,&w,&ht);
    void* h = CreateWindowExA(0, "RICHEDIT50W", "", style,
                           x, y, w, ht, parent,
                           (void*)(long long)id, GetModuleHandleA(0), 0);
    if (h) SendMessageA(h, 0x0030, (unsigned long long)(size_t)pb_dialog_font(), 1);
    return h;
}

/* CONTROL ADD TRACKBAR (batch 189) - class "msctls_trackbar32" */
void* pb_control_add_trackbar(void* parent, long id, int x, int y, int w, int ht) {
    unsigned long style = 0x0001 | 0x40000000 | 0x10000000 | 0x00010000;
    pb_dlu_to_px(&x,&y,&w,&ht);
    void* h = CreateWindowExA(0, "msctls_trackbar32", "", style,
                           x, y, w, ht, parent,
                           (void*)(long long)id, GetModuleHandleA(0), 0);
    if (h) SendMessageA(h, 0x0030, (unsigned long long)(size_t)pb_dialog_font(), 1);
    return h;
}

/* CONTROL ADD UPDOWN (batch 189) - class "msctls_updown32" */
void* pb_control_add_updown(void* parent, long id, int x, int y, int w, int ht) {
    unsigned long style = 0x0002 | 0x0004 | 0x40000000 | 0x10000000;
    pb_dlu_to_px(&x,&y,&w,&ht);
    void* h = CreateWindowExA(0, "msctls_updown32", "", style,
                           x, y, w, ht, parent,
                           (void*)(long long)id, GetModuleHandleA(0), 0);
    if (h) SendMessageA(h, 0x0030, (unsigned long long)(size_t)pb_dialog_font(), 1);
    return h;
}

/* CONTROL ADD HOTKEY (batch 189) - class "msctls_hotkey32" */
void* pb_control_add_hotkey(void* parent, long id, int x, int y, int w, int ht) {
    unsigned long style = 0x40000000 | 0x10000000 | 0x00010000;
    pb_dlu_to_px(&x,&y,&w,&ht);
    void* h = CreateWindowExA(0, "msctls_hotkey32", "", style,
                           x, y, w, ht, parent,
                           (void*)(long long)id, GetModuleHandleA(0), 0);
    if (h) SendMessageA(h, 0x0030, (unsigned long long)(size_t)pb_dialog_font(), 1);
    return h;
}

/* CONTROL ADD IPADDRESS (batch 189) - class "SysIPAddress32" */
void* pb_control_add_ipaddress(void* parent, long id, int x, int y, int w, int ht) {
    unsigned long style = 0x40000000 | 0x10000000 | 0x00010000;
    pb_dlu_to_px(&x,&y,&w,&ht);
    void* h = CreateWindowExA(0, "SysIPAddress32", "", style,
                           x, y, w, ht, parent,
                           (void*)(long long)id, GetModuleHandleA(0), 0);
    if (h) SendMessageA(h, 0x0030, (unsigned long long)(size_t)pb_dialog_font(), 1);
    return h;
}

/* PROGRESSBAR SET RANGE/POS */
void pb_progress_set_range(void* h, int lo, int hi) {
    SendMessageA(h, 0x0401, (unsigned long long)lo, (unsigned long long)hi);
}
void pb_progress_set_pos(void* h, int pos) {
    SendMessageA(h, 0x00F3, (unsigned long long)pos, 0);
}

#define SBM_SETPOS 0x00E0
#define SBM_GETPOS 0x00E1
void pb_control_set_pos(void* hctrl, long pos) {
    SendMessageA(hctrl, SBM_SETPOS, (unsigned long long)pos, 0);
}
long long pb_control_get_pos(void* hctrl) {
    return (long long)SendMessageA(hctrl, SBM_GETPOS, 0, 0);
}

/* CONTROL ADD HSCROLLBAR - horizontal scrollbar */
void* pb_control_add_hscrollbar(void* parent, long id, int x, int y, int w, int ht) {
    /* SBS_HORZ=0, WS_CHILD=0x40000000, WS_VISIBLE=0x10000000, WS_TABSTOP=0x10000 */
    unsigned long style = 0 | 0x40000000 | 0x10000000 | 0x10000;
    void* hbar = CreateWindowExA(0, "SCROLLBAR", "", style,
                            x, y, w, ht, parent,
                            (void*)(long long)id, GetModuleHandleA(0), 0);
    SendMessageA(hbar, 0x00F4 /* SBM_SETRANGE32 */, 0, 100);
    return hbar;
}

/* Additional Win32 declarations */
__declspec(dllimport) long __stdcall PeekMessageA(void*, void*, unsigned long, unsigned long, unsigned long);
__declspec(dllimport) int __stdcall GetClientRect(void*, void*);
__declspec(dllimport) int __stdcall SetWindowPos(void*, void*, int, int, int, int, unsigned int);
typedef struct { long left; long top; long right; long bottom; } RECT;

/* DIALOG DOEVENTS - process pending messages non-blocking */
void pb_dialog_doevents(void) {
    pb_msg_t msg;
    while (PeekMessageA(&msg, 0, 0, 0, 1 /* PM_REMOVE */)) {
        if (msg.message == 0x0012 /* WM_QUIT */) {
            PostQuitMessage(0);
            break;
        }
        if (pb_accel_translate(&msg)) continue;
        TranslateMessage(&msg);
        DispatchMessageA(&msg);
    }
}

/* DIALOG GET TEXT hDlg TO var$ */
void pb_dialog_get_text(void* hDlg, char* buf, long bufsize) {
    GetWindowTextA(hDlg, buf, (int)bufsize);
}

/* CONTROL KILL hCtrl - destroy a control */
void pb_control_kill(void* hctrl) {
    if (hctrl) DestroyWindow(hctrl);
}

/* DIALOG SHOW STATE: nCmdShow=1 normal, 2 min, 3 max */
void pb_dialog_show_state(void* hDlg, int nCmdShow) {
    ShowWindow(hDlg, nCmdShow);
}

/* CONTROL SET CHECK hCtrl, state */
void pb_control_set_check(void* hctrl, int state) {
    SendMessageA(hctrl, 0x00F1 /* BM_SETCHECK */, (unsigned long long)state, 0);
}

/* DIALOG GET SIZE - the official statement returns the "total size of the
 * dialog", i.e. the whole window (GetWindowRect), which pairs with DIALOG SET
 * SIZE (SetWindowPos).  DIALOG GET CLIENT is the client-area variant and was
 * implemented in batch 165 (see the block below).
 *
 * Units: the official help (dialog_get_size.htm / dialog_set_size.htm) says the
 * values are in DIALOG UNITS unless the dialog was created with the PIXELS
 * option.  DIALOG NEW converts its x/y/w/h through GetDialogBaseUnits() (see
 * pb_window_new above) and DIALOG PIXELS is still unimplemented, so every
 * dialog this compiler creates is a dialog-unit dialog: both statements apply
 * the same conversion, in the opposite direction for GET.  Because pb_dlg_units
 * reads exactly the base units pb_window_new uses, "DIALOG NEW ... w,h" followed
 * by "DIALOG GET SIZE" round-trips to w,h for any w divisible by 4 and any h
 * divisible by 8 (320x200 and 500x360 in the example). */
static void pb_dlg_units(int* dlu_x, int* dlu_y) {
    /* batch 239: same base as pb_window_new - the dialog font (6x13 default,
       or the DIALOG DEFAULT FONT's metrics), NOT GetDialogBaseUnits(). */
    pb_dialog_base_units(dlu_x, dlu_y);
    if (*dlu_x <= 0) *dlu_x = 6;   /* never divide by zero */
    if (*dlu_y <= 0) *dlu_y = 13;
}

void pb_dialog_get_size(void* hDlg, long long* pw, long long* ph) {
    pb_rect_t rc;
    int dx, dy;
    rc.left = 0; rc.top = 0; rc.right = 0; rc.bottom = 0;
    GetWindowRect(hDlg, &rc);
    pb_dlg_units(&dx, &dy);
    if (pw) *pw = (long long)(((rc.right - rc.left) * 4 + dx / 2) / dx);
    if (ph) *ph = (long long)(((rc.bottom - rc.top) * 8 + dy / 2) / dy);
}

/* DIALOG SET SIZE - dialog units in, pixels out to SetWindowPos.
 *
 * Batch 175 fix: the flags used to be 0x0040 (SWP_SHOWWINDOW) alone, so
 * this statement did three things instead of one - it resized the dialog,
 * MOVED it to screen (0,0) (no SWP_NOMOVE), and forced a hidden dialog
 * visible.  The official statement only changes the size.  SWP_NOMOVE |
 * SWP_NOZORDER is what DIALOG SET CLIENT / SET LOC already use, and
 * examples/batch175_test.bas asserts the position survives the resize. */
void pb_dialog_set_size(void* hDlg, long long w, long long h) {
    int dx, dy;
    pb_dlg_units(&dx, &dy);
    SetWindowPos(hDlg, 0, 0, 0, (int)((w * dx + 3) / 4), (int)((h * dy + 6) / 8),
                 0x0002u | 0x0004u /* SWP_NOMOVE | SWP_NOZORDER */);
}

/* ===================================================================
   Batch 165 - the DIALOG statement family (21 statements)
   -------------------------------------------------------------------
   Every constant below was read out of the authoritative local
   PowerBASIC include file  C:\PBWin10\WinAPI\WinUser.inc  (not from
   memory):

     %SW_HIDE 0   %SW_MAXIMIZE 3   %SW_SHOW 5   %SW_MINIMIZE 6
     %SW_RESTORE 9
     %GWL_STYLE -16   %GWL_EXSTYLE -20   %WM_SETICON &H0080
     %ICON_SMALL 0    %ICON_BIG 1
     %RDW_INVALIDATE &H0001  %RDW_ALLCHILDREN &H0080  %RDW_UPDATENOW &H0100
     %MF_BYCOMMAND 0  %MF_ENABLED 0  %MF_GRAYED 1  %MF_DISABLED 2
     %SC_CLOSE &HF060   %SWP_NOMOVE &H0002  %SWP_NOZORDER &H0004

   Semantics follow the official help pages in
   C:\PBWin10\bin\PBWin_extracted\html\dialog_*.htm.

   Units: DIALOG NEW does not implement the PIXELS option yet, so every
   dialog this compiler creates is a dialog-unit dialog - the same
   assumption DIALOG GET SIZE / SET SIZE already make.  The geometry
   statements below convert through pb_dlg_units() (GetDialogBaseUnits),
   which is exactly the base pb_window_new uses, so GET and SET
   round-trip against DIALOG NEW.
   =================================================================== */

__declspec(dllimport) int __stdcall RedrawWindow(void*, const void*, void*, unsigned long);
__declspec(dllimport) int __stdcall PostMessageA(void*, unsigned long, pb_wparam_t, pb_lparam_t);
__declspec(dllimport) int __stdcall AdjustWindowRectEx(pb_rect_t*, unsigned long, int, unsigned long);
__declspec(dllimport) void* __stdcall GetMenu(void*);
__declspec(dllimport) void* __stdcall GetParent(void*);
__declspec(dllimport) int __stdcall ClientToScreen(void*, pb_point_t*);
__declspec(dllimport) void* __stdcall LoadIconA(void*, const char*);
__declspec(dllimport) void* __stdcall GetSystemMenu(void*, int);
__declspec(dllimport) int __stdcall DrawMenuBar(void*);
/* GetWindowLongPtrA does not exist in 32-bit user32 - it is a macro to
   GetWindowLongA there, so the arch decides which one is imported. */
#if defined(_WIN64)
__declspec(dllimport) long long __stdcall GetWindowLongPtrA(void*, int);
static long long pb_get_winlong(void* h, int idx) { return GetWindowLongPtrA(h, idx); }
#else
__declspec(dllimport) long __stdcall GetWindowLongA(void*, int);
static long long pb_get_winlong(void* h, int idx) { return (long long)GetWindowLongA(h, idx); }
#endif

/* SetWindowLongPtrA, like GetWindowLongPtrA above, only exists in 64-bit
   user32; the 32-bit build falls back to SetWindowLongA.  Needed by
   LISTVIEW SET MODE, which rewrites the LVS_TYPEMASK style bits. */
#if defined(_WIN64)
__declspec(dllimport) long long __stdcall SetWindowLongPtrA(void*, int, long long);
static long long pb_set_winlong(void* h, int idx, long long v) { return SetWindowLongPtrA(h, idx, v); }
#else
__declspec(dllimport) long __stdcall SetWindowLongA(void*, int, long);
static long long pb_set_winlong(void* h, int idx, long long v) { return (long long)SetWindowLongA(h, idx, (long)v); }
#endif

/* --- WINDOW GET/SET (CHW WINDOW_GET/SET statements) ------------------
   Exported wrappers over the width-portable helpers above. The GWL index
   selects the field: STYLE=-16, EXSTYLE=-20, ID=-12, USERDATA=-21. */
long long pb_window_get_long(void* h, int idx) {
    return pb_get_winlong(h, idx);
}
void* pb_window_get_parent(void* h) {
    return GetParent(h);
}
void* pb_window_get_handle(void* parent, int id) {
    return GetDlgItem(parent, id);
}
/* Returns the prior value (CHW: old value assigned to OldValVar). */
long long pb_window_set_long(void* h, int idx, long long newval) {
    return pb_set_winlong(h, idx, newval);
}

/* --- DIALOG ENABLE hDlg / DIALOG DISABLE hDlg ----------------------- */
void pb_dialog_enable(void* hDlg, int on) { EnableWindow(hDlg, on); }

/* --- DIALOG HIDE / NORMALIZE / MINIMIZE / MAXIMIZE / SHOW MODELESS --- */
void pb_dialog_show(void* hDlg, int cmd) { ShowWindow(hDlg, cmd); }

/* --- DIALOG REDRAW hDlg ---------------------------------------------
   Invalidate now, children included: RDW_INVALIDATE | RDW_UPDATENOW |
   RDW_ALLCHILDREN. */
void pb_dialog_redraw(void* hDlg) {
    RedrawWindow(hDlg, 0, 0, 0x0001u | 0x0100u | 0x0080u);
}

/* --- DIALOG STABILIZE hDlg / DIALOG NONSTABLE hDlg ------------------
   The close item is greyed rather than deleted so the state is
   reversible, and Windows stops honouring ALT-F4 on its own:
   DefWindowProc only posts WM_CLOSE for ALT-F4 while the system menu's
   SC_CLOSE item is enabled. */
void pb_dialog_stabilize(void* hDlg, int stable) {
    void* m = GetSystemMenu(hDlg, 0);
    if (!m) return;
    if (stable)
        EnableMenuItem(m, 0xF060u /* SC_CLOSE */,
                       0x00000000u /* MF_BYCOMMAND */ | 0x00000001u /* MF_GRAYED */
                           | 0x00000002u /* MF_DISABLED */);
    else
        EnableMenuItem(m, 0xF060u /* SC_CLOSE */, 0x00000000u /* MF_BYCOMMAND | MF_ENABLED */);
    DrawMenuBar(hDlg);
}

/* --- DIALOG SEND hDlg, msg&, wParam&, lParam& [TO lResult&] --------- */
long long pb_dialog_send(void* hDlg, long long msg, long long wp, long long lp) {
    return (long long)SendMessageA(hDlg, (unsigned int)msg, (pb_wparam_t)wp, (pb_lparam_t)lp);
}

/* --- DIALOG POST hDlg, msg&, wParam&, lParam& ----------------------- */
void pb_dialog_post(void* hDlg, long long msg, long long wp, long long lp) {
    PostMessageA(hDlg, (unsigned long)msg, (pb_wparam_t)wp, (pb_lparam_t)lp);
}

/* --- DIALOG SET ICON hDlg, newicon$ ---------------------------------
   The icon comes out of the running module's own resources, which is
   where the compiler's #RESOURCE ICON directives put it.  A name that
   begins with '#' is an integral resource id, as documented. */
void pb_dialog_set_icon(void* hDlg, const char* name) {
    const char* p = name ? name : "";
    const char* use = p;
    if (*p == '#') {
        long id = 0;
        const char* q = p + 1;
        while (*q >= '0' && *q <= '9') { id = id * 10 + (*q - '0'); q++; }
        use = (const char*)(long long)id;
    }
    void* hIcon = LoadIconA(GetModuleHandleA(0), use);
    if (!hIcon) return;
    SendMessageA(hDlg, 0x0080 /* WM_SETICON */, 1 /* ICON_BIG */, (pb_lparam_t)hIcon);
    SendMessageA(hDlg, 0x0080 /* WM_SETICON */, 0 /* ICON_SMALL */, (pb_lparam_t)hIcon);
}

/* --- DIALOG SET USER / DIALOG GET USER ------------------------------
   Eight Long values per dialog, index 1..8, completely separate from
   %GWL_USERDATA.  pb_window_new registers the window class with
   cbWndExtra = 0, so the area lives in this side table: a slot is taken
   on first use and kept for the life of the process. */
#define PB_DLG_USER_SLOTS 256
static void* pb_dlg_user_h[PB_DLG_USER_SLOTS];
static long long pb_dlg_user_v[PB_DLG_USER_SLOTS][8];

static int pb_dlg_user_slot(void* h, int make) {
    int i, free_i = -1;
    for (i = 0; i < PB_DLG_USER_SLOTS; i++) {
        if (pb_dlg_user_h[i] == h) return i;
        if (free_i < 0 && pb_dlg_user_h[i] == 0) free_i = i;
    }
    if (!make || free_i < 0) return -1;
    pb_dlg_user_h[free_i] = h;
    for (i = 0; i < 8; i++) pb_dlg_user_v[free_i][i] = 0;
    return free_i;
}

void pb_dialog_set_user(void* hDlg, long long index, long long value) {
    int slot;
    if (index < 1 || index > 8) return;
    slot = pb_dlg_user_slot(hDlg, 1);
    if (slot < 0) return;
    pb_dlg_user_v[slot][index - 1] = value;
}

long long pb_dialog_get_user(void* hDlg, long long index) {
    int slot;
    if (index < 1 || index > 8) return 0;
    slot = pb_dlg_user_slot(hDlg, 0);
    if (slot < 0) return 0;
    return pb_dlg_user_v[slot][index - 1];
}

/* --- CONTROL messages and state (batch 176) ------------------------------
   Every statement in this family names its target as the (hDlg, id) pair, so
   each function resolves the HWND through pb_pb_hwnd() - the same resolver
   the geometry statements use.  A control that does not exist yields 0 or a
   do-nothing call instead of a dereference. */

/* CONTROL HANDLE hDlg, id& TO hCtl& - the window handle Windows assigned
   when CONTROL ADD created the control.  Some API functions need a handle
   where a DDT statement would take the id. */
void* pb_control_handle(void* hDlg, long long id) {
    return pb_pb_hwnd(hDlg, id);
}

/* CONTROL SEND hDlg, id&, Msg&, wParam&, lParam& [TO lResult&] - the
   synchronous form: by the time this returns, the target's callback has
   processed the message, so a result is available. */
/* PB_SCROLLINFO + scrollbar helpers (types moved up from the scrollbar
   section so pb_control_send's LB_GETTOPINDEX fallback can read the
   scrollbar position) */
#define PB_SIF_RANGE    0x0001
#define PB_SIF_PAGE     0x0002
#define PB_SIF_POS      0x0004
#define PB_SIF_TRACKPOS 0x0010
#define PB_SB_CTL       2

typedef struct {
    unsigned int cbSize;
    unsigned int fMask;
    int nMin;
    int nMax;
    unsigned int nPage;
    int nPos;
    int nTrackPos;
} PB_SCROLLINFO;

__declspec(dllimport) int __stdcall GetScrollInfo(void* hWnd, int nBar, PB_SCROLLINFO* lpsi);
__declspec(dllimport) int __stdcall SetScrollInfo(void* hWnd, int nBar, const PB_SCROLLINFO* lpsi, int redraw);

long long pb_control_send(void* hDlg, long long id, long long msg, long long wp, long long lp) {
    void* h = pb_pb_hwnd(hDlg, id);
    if (!h) return 0;
    long long rv = (long long)SendMessageA(h, (unsigned int)msg, (pb_wparam_t)wp, (pb_lparam_t)lp);
    /* batch 245d: on our ListBoxes a WM_MOUSEWHEEL scroll moves the scrollbar
       position but leaves the internal top index stuck at 0 (the wheel goes
       through the scrollbar path, the ListBox's own top-index state is never
       refreshed after we restore WS_VSCROLL via SetWindowLong).  The official
       SYNCLIST sample syncs with LB_GETTOPINDEX, so a wheel-scrolled box
       reports 0 and the two lists never sync.  When GETTOPINDEX returns 0,
       fall back to the scrollbar position, which does track the wheel.
       Safe: a box truly at the top has pos 0, and the drag-thumb path keeps
       returning the real top index (non-zero). */
    if (msg == 0x10E /* LB_GETTOPINDEX */ && rv == 0) {
        PB_SCROLLINFO si;
        memset(&si, 0, sizeof(si));
        si.cbSize = (unsigned int)sizeof(si);
        si.fMask = PB_SIF_POS | PB_SIF_RANGE | PB_SIF_PAGE;
        if (GetScrollInfo(h, 1 /* SB_VERT */, &si) && si.nPos > 0 && si.nMax > 0)
            rv = si.nPos;
    }
    return rv;
}

/* CONTROL POST hDlg, id&, Msg&, wParam&, lParam& - the asynchronous form:
   the message only joins the queue and this returns at once, so - unlike
   CONTROL SEND - no result can exist. */
void pb_control_post(void* hDlg, long long id, long long msg, long long wp, long long lp) {
    void* h = pb_pb_hwnd(hDlg, id);
    if (!h) return;
    PostMessageA(h, (unsigned long)msg, (pb_wparam_t)wp, (pb_lparam_t)lp);
}

/* CONTROL REDRAW hDlg, id& - invalidate the control and schedule the
   repaint (a low-priority event, as the manual warns). */
void pb_control_redraw(void* hDlg, long long id) {
    void* h = pb_pb_hwnd(hDlg, id);
    if (!h) return;
    InvalidateRect(h, 0, 1);
    UpdateWindow(h);
}

/* CONTROL SET FOCUS hDlg, id& - keyboard focus moves to that control, and
   Windows makes its parent dialog the foreground window. */
void pb_control_set_focus(void* hDlg, long long id) {
    void* h = pb_pb_hwnd(hDlg, id);
    if (h) SetFocus(h);
}

/* CONTROL SET FONT hDlg, id&, FontHndl& - a FontHndl& of zero restores the
   default font, which is the one DIALOG DEFAULT FONT recorded in
   pb_default_font; if the program never chose one, the dialog's own current
   font is the closest thing to "the original default". */
void pb_control_set_font(void* hDlg, long long id, long long hFont) {
    void* h = pb_pb_hwnd(hDlg, id);
    if (!h) return;
    if (!hFont) {
        hFont = (long long)(size_t)pb_default_font;
        if (!hFont) hFont = (long long)(size_t)SendMessageA(hDlg, 0x0031 /* WM_GETFONT */, 0, 0);
    }
    SendMessageA(h, 0x0030 /* WM_SETFONT */, (pb_wparam_t)(size_t)hFont, 1 /* redraw */);
}

/* CONTROL SHOW STATE hDlg, id&, showstate& [TO lResult&] - ShowWindow()
   returns the PREVIOUS visibility state, which is exactly what the optional
   TO clause is documented to report (zero = the control was not visible). */
long long pb_control_show_state(void* hDlg, long long id, long long state) {
    void* h = pb_pb_hwnd(hDlg, id);
    if (!h) return 0;
    return (long long)ShowWindow(h, (int)state);
}

/* CONTROL NORMALIZE hDlg, id& - make the control visible. */
void pb_control_normalize(void* hDlg, long long id) {
    void* h = pb_pb_hwnd(hDlg, id);
    if (h) ShowWindow(h, 1 /* SW_SHOWNORMAL */);
}

/* CONTROL SET USER / GET USER hDlg, id&, index& [, usrval&] - eight Long
   slots per control, index 1..8, held separately from %GWL_USERDATA.  The
   storage is the side table DIALOG SET/GET USER already uses; keying it by
   the control's HWND is what keeps one control's slots apart from another's
   and from its dialog's.  The values die with the control. */
void pb_control_set_user(void* hDlg, long long id, long long index, long long value) {
    void* h = pb_pb_hwnd(hDlg, id);
    if (!h) return;
    pb_dialog_set_user(h, index, value);
}

long long pb_control_get_user(void* hDlg, long long id, long long index) {
    void* h = pb_pb_hwnd(hDlg, id);
    if (!h) return 0;
    return pb_dialog_get_user(h, index);
}

/* --- geometry: GET/SET CLIENT, GET/SET LOC, PIXELS, UNITS -----------
   DIALOG GET LOC / SET LOC measure from the parent's client-area origin
   when the dialog has a parent, and from the screen origin (0,0) when it
   does not.  The two official pages disagree here - dialog_get_loc.htm
   says "relative to the upper-left corner of the display" while
   dialog_set_loc.htm says "relative to the upper-left corner of the
   desktop workspace" - so the origin that makes GET and SET round-trip
   against each other was chosen. */
static void pb_dlg_origin(void* hDlg, pb_point_t* org) {
    void* parent = GetParent(hDlg);
    org->x = 0;
    org->y = 0;
    if (parent) ClientToScreen(parent, org);
}

/* DIALOG GET CLIENT hDlg TO nWide&, nHigh& */
void pb_dialog_get_client(void* hDlg, long long* pw, long long* ph) {
    pb_rect_t rc;
    int dx, dy;
    rc.left = 0; rc.top = 0; rc.right = 0; rc.bottom = 0;
    GetClientRect(hDlg, &rc);
    pb_dlg_units(&dx, &dy);
    if (pw) *pw = (long long)(((rc.right - rc.left) * 4 + dx / 2) / dx);
    if (ph) *ph = (long long)(((rc.bottom - rc.top) * 8 + dy / 2) / dy);
}

/* DIALOG SET CLIENT hDlg, x&, y& - the arguments are a CLIENT size, so the
   window rectangle has to be grown by the borders, caption and menu
   first. */
void pb_dialog_set_client(void* hDlg, long long w, long long h) {
    pb_rect_t rc;
    int dx, dy, has_menu;
    unsigned long style, exstyle;
    pb_dlg_units(&dx, &dy);
    rc.left = 0; rc.top = 0;
    rc.right = (long)((w * dx) / 4);
    rc.bottom = (long)((h * dy) / 8);
    style = (unsigned long)pb_get_winlong(hDlg, -16 /* GWL_STYLE */);
    exstyle = (unsigned long)pb_get_winlong(hDlg, -20 /* GWL_EXSTYLE */);
    has_menu = GetMenu(hDlg) != 0;
    AdjustWindowRectEx(&rc, style, has_menu, exstyle);
    SetWindowPos(hDlg, 0, 0, 0, (int)(rc.right - rc.left), (int)(rc.bottom - rc.top),
                 0x0002u | 0x0004u /* SWP_NOMOVE | SWP_NOZORDER */);
}

/* DIALOG GET LOC hDlg TO x&, y& */
void pb_dialog_get_loc(void* hDlg, long long* px, long long* py) {
    pb_rect_t wr;
    pb_point_t org;
    int dx, dy;
    wr.left = 0; wr.top = 0; wr.right = 0; wr.bottom = 0;
    GetWindowRect(hDlg, &wr);
    pb_dlg_origin(hDlg, &org);
    pb_dlg_units(&dx, &dy);
    if (px) *px = (long long)(((wr.left - org.x) * 4 + dx / 2) / dx);
    if (py) *py = (long long)(((wr.top - org.y) * 8 + dy / 2) / dy);
}

/* DIALOG SET LOC hDlg, x&, y& */
void pb_dialog_set_loc(void* hDlg, long long x, long long y) {
    pb_point_t org;
    int dx, dy;
    pb_dlg_units(&dx, &dy);
    pb_dlg_origin(hDlg, &org);
    SetWindowPos(hDlg, 0, org.x + (int)((x * dx + 3) / 4), org.y + (int)((y * dy + 6) / 8),
                 0, 0, 0x0001u | 0x0004u /* SWP_NOSIZE | SWP_NOZORDER */);
}

/* DIALOG PIXELS hDlg, x&, y& TO UNITS xx&, yy&  - device units -> dialog units */
void pb_dialog_pixels(void* hDlg, long long x, long long y, long long* px, long long* py) {
    int dx, dy;
    (void)hDlg;
    pb_dlg_units(&dx, &dy);
    if (px) *px = (long long)((x * 4) / dx);
    if (py) *py = (long long)((y * 8) / dy);
}

/* DIALOG UNITS hDlg, x&, y& TO PIXELS xx&, yy&  - dialog units -> device units */
void pb_dialog_units(void* hDlg, long long x, long long y, long long* px, long long* py) {
    int dx, dy;
    (void)hDlg;
    pb_dlg_units(&dx, &dy);
    if (px) *px = (long long)((x * dx) / 4);
    if (py) *py = (long long)((y * dy) / 8);
}

/* ===================================================================
   Batch 175 - CONTROL geometry: GET/SET CLIENT, GET/SET LOC, GET/SET SIZE
   -------------------------------------------------------------------
   Official syntax (control_get_client.htm, control_get_loc.htm,
   control_get_size.htm, control_set_client.htm, control_set_loc.htm,
   control_set_size.htm) addresses the control by (hDlg, id&): the dialog
   that OWNS the control, plus the control id assigned by CONTROL ADD.
   The HWND comes from GetDlgItem through pb_pb_hwnd() above - the same
   (owner, id) resolution PROGRESSBAR / HEADER use.

   Units: every CONTROL in this compiler is laid out by pb_dlu_to_px(), i.e.
   in the dialog units of PB_CTL_DLU_X x PB_CTL_DLU_Y (6 x 13).  The geometry
   statements convert back with those same two numbers, so CONTROL ADD of
   "12, 12" and CONTROL GET LOC agree, and CONTROL SET LOC puts a control
   exactly where the same numbers in CONTROL ADD would.  (Batch 175 first
   routed these through pb_dlg_units(), which describes the SYSTEM font -
   8 x 16 on this machine - and the two disagreed: a button added at 10,10
   read back 8,8 and a 60x20 button read back 52x17.)
   Divisibility, exactly as in the DIALOG family: a width in units is exact
   when it is a multiple of 4 and a height when it is a multiple of 8 -
   one vertical unit is 1.75 px, so 30 units cannot land on a whole pixel.

   LOC is measured from the upper-left corner of the PARENT DIALOG's client
   area (control_get_loc.htm) - ClientToScreen(hDlg) is that origin.  A
   control is a CHILD window, though, and SetWindowPos interprets x/y of a
   child as offsets from the PARENT'S CLIENT AREA, not screen coordinates:
   SET passes the plain offsets while GET subtracts the origin.  (Batch 175
   first added the origin on both sides, i.e. twice, so SET LOC 20,30 landed
   at 64,75.)
   Negative and off-dialog coordinates are legal and simply clip or hide
   the control (control_set_loc.htm).

   GET SIZE returns the OVERALL size (borders included, GetWindowRect);
   GET CLIENT returns the client area (GetClientRect); SET CLIENT grows the
   window rectangle by the control's own non-client allowance first, which
   is why a bordered control reports a client area a few pixels smaller.
   =================================================================== */

/* CONTROL GET CLIENT hDlg, id& TO nWide&, nHigh& */
void pb_control_get_client(void* hDlg, long long id, long long* pw, long long* ph) {
    void* h = pb_pb_hwnd(hDlg, id);
    pb_rect_t rc;
    if (!h) { if (pw) *pw = 0; if (ph) *ph = 0; return; }
    rc.left = 0; rc.top = 0; rc.right = 0; rc.bottom = 0;
    GetClientRect(h, &rc);
    if (pw) *pw = (long long)PB_PX_X_TO_DLU(rc.right - rc.left);
    if (ph) *ph = (long long)PB_PX_Y_TO_DLU(rc.bottom - rc.top);
}

/* CONTROL GET SIZE hDlg, id& TO nWide&, nHigh& - overall, borders included */
void pb_control_get_size(void* hDlg, long long id, long long* pw, long long* ph) {
    void* h = pb_pb_hwnd(hDlg, id);
    pb_rect_t rc;
    if (!h) { if (pw) *pw = 0; if (ph) *ph = 0; return; }
    rc.left = 0; rc.top = 0; rc.right = 0; rc.bottom = 0;
    GetWindowRect(h, &rc);
    if (pw) *pw = (long long)PB_PX_X_TO_DLU(rc.right - rc.left);
    if (ph) *ph = (long long)PB_PX_Y_TO_DLU(rc.bottom - rc.top);
}

/* CONTROL GET LOC hDlg, id& TO x&, y& - from the parent's client origin */
void pb_control_get_loc(void* hDlg, long long id, long long* px, long long* py) {
    void* h = pb_pb_hwnd(hDlg, id);
    pb_rect_t wr;
    pb_point_t org;
    if (px) *px = 0;
    if (py) *py = 0;
    if (!h) return;
    wr.left = 0; wr.top = 0; wr.right = 0; wr.bottom = 0;
    GetWindowRect(h, &wr);
    org.x = 0; org.y = 0;
    if (hDlg) ClientToScreen(hDlg, &org);
    if (px) *px = (long long)PB_PX_X_TO_DLU(wr.left - org.x);
    if (py) *py = (long long)PB_PX_Y_TO_DLU(wr.top - org.y);
}

/* CONTROL SET LOC hDlg, id&, x&, y& */
void pb_control_set_loc(void* hDlg, long long id, long long x, long long y) {
    void* h = pb_pb_hwnd(hDlg, id);
    if (!h) return;
    /* Child window: x/y are relative to the parent's client area, so the
       ClientToScreen origin must NOT be added here (it belongs in
       DIALOG SET LOC, where the window is top-level and x/y are screen
       coordinates). */
    SetWindowPos(h, 0, (int)PB_DLU_X_TO_PX(x), (int)PB_DLU_Y_TO_PX(y),
                 0, 0, 0x0001u | 0x0004u /* SWP_NOSIZE | SWP_NOZORDER */);
}

/* CONTROL SET SIZE hDlg, id&, nWide&, nHigh& - overall size */
void pb_control_set_size(void* hDlg, long long id, long long w, long long nHigh) {
    void* hc = pb_pb_hwnd(hDlg, id);
    if (!hc) return;
    SetWindowPos(hc, 0, 0, 0,
                 (int)PB_DLU_X_TO_PX(w), (int)PB_DLU_Y_TO_PX(nHigh),
                 0x0002u | 0x0004u /* SWP_NOMOVE | SWP_NOZORDER */);
}

/* CONTROL SET CLIENT hDlg, id&, nWide&, nHigh& - grow the window rectangle by
   the control's own border allowance so the CLIENT area ends up w x h.
   AdjustWindowRectEx handles child windows: it adds whatever WS_BORDER /
   WS_EX_CLIENTEDGE the control's style implies. */
void pb_control_set_client(void* hDlg, long long id, long long w, long long nHigh) {
    void* hc = pb_pb_hwnd(hDlg, id);
    pb_rect_t rc;
    unsigned long style, exstyle;
    if (!hc) return;
    rc.left = 0; rc.top = 0;
    rc.right = (long)PB_DLU_X_TO_PX(w);
    rc.bottom = (long)PB_DLU_Y_TO_PX(nHigh);
    style = (unsigned long)pb_get_winlong(hc, -16 /* GWL_STYLE */);
    exstyle = (unsigned long)pb_get_winlong(hc, -20 /* GWL_EXSTYLE */);
    AdjustWindowRectEx(&rc, style, 0, exstyle);
    SetWindowPos(hc, 0, 0, 0, (int)(rc.right - rc.left), (int)(rc.bottom - rc.top),
                 0x0002u | 0x0004u /* SWP_NOMOVE | SWP_NOZORDER */);
}

/* --- DIALOG SET COLOR hDlg, foreclr&, backclr& ----------------------
   The official help states that foreclr& is not used by the current
   implementation and that -1& means "the system default", so only the
   background is honoured here; -1& clears the override and lets the
   window class brush show through again.  Colours are 0x00BBGGRR. */
static int pb_dlg_bg_lookup(void* hWnd, unsigned long* rgb) {
    int i;
    for (i = 0; i < PB_DLG_BG_SLOTS; i++) {
        if (pb_dlg_bg_set[i] && pb_dlg_bg_h[i] == hWnd) {
            *rgb = (unsigned long)pb_dlg_bg_v[i];
            return 1;
        }
    }
    return 0;
}

/* pb_ctlcolor_brush - the %WM_CTLCOLOR* answer for one control.  A NULL
   return means "nothing registered here", which lets pb_wndproc pass the
   message on to DefWindowProc just as it did before batch 179. */
__declspec(dllimport) unsigned long __stdcall GetSysColor(int nIndex);
__declspec(dllimport) void* __stdcall GetSysColorBrush(int nIndex);
__declspec(dllimport) int __stdcall GetClassNameA(void* hWnd, char* lpClassName, int nMaxCount);

#define PB_COLOR_WINDOW     5   /* COLOR_WINDOW      - EDIT / LISTBOX face */
#define PB_COLOR_BTNFACE    15  /* COLOR_BTNFACE     - dialog face          */
#define PB_COLOR_WINDOWTEXT 8   /* COLOR_WINDOWTEXT  - default text colour  */

static void pb_ctl_color_forget(void* hWnd) {
    int i;
    for (i = 0; i < PB_CTL_COLOR_SLOTS; i++) {
        if (pb_ctl_clr_set[i] && (pb_ctl_clr_ctl[i] == hWnd || pb_ctl_clr_par[i] == hWnd)) {
            if (pb_ctl_clr_br[i]) DeleteObject(pb_ctl_clr_br[i]);
            pb_ctl_clr_ctl[i] = 0;
            pb_ctl_clr_par[i] = 0;
            pb_ctl_clr_br[i] = 0;
            pb_ctl_clr_set[i] = 0;
        }
    }
}

/* The default background is a per-class question: the help page says -1
   selects "the default background text color", and for a STATIC (or a
   button) that default is the dialog face, while a text-bearing control
   paints on COLOR_WINDOW.  Asking the control for its class keeps the two
   apart without guessing from the id. */
static void* pb_ctl_default_brush(void* hCtl) {
    char cls[32];
    int i;
    if (!hCtl) return GetSysColorBrush(PB_COLOR_BTNFACE);
    cls[0] = 0;
    if (GetClassNameA(hCtl, cls, (int)sizeof(cls) - 1) > 0) {
        for (i = 0; cls[i]; i++) {
            if (cls[i] >= 'a' && cls[i] <= 'z') cls[i] = (char)(cls[i] - 32);
        }
        if ((cls[0] == 'E' && cls[1] == 'D' && cls[2] == 'I' && cls[3] == 'T') ||
            (cls[0] == 'L' && cls[1] == 'I' && cls[2] == 'S' && cls[3] == 'T') ||
            (cls[0] == 'S' && cls[1] == 'C' && cls[2] == 'R') ||
            (cls[0] == 'C' && cls[1] == 'O' && cls[2] == 'M' && cls[3] == 'B')) {
            return GetSysColorBrush(PB_COLOR_WINDOW);
        }
    }
    return GetSysColorBrush(PB_COLOR_BTNFACE);
}

static void* pb_ctlcolor_brush(void* hCtl, void* hdc) {
    int i;
    if (!hCtl || !hdc) return 0;
    for (i = 0; i < PB_CTL_COLOR_SLOTS; i++) {
        if (!pb_ctl_clr_set[i] || pb_ctl_clr_ctl[i] != hCtl) continue;
        if (pb_ctl_clr_fg[i] >= 0) SetTextColor(hdc, (unsigned long)pb_ctl_clr_fg[i]);
        else SetTextColor(hdc, GetSysColor(PB_COLOR_WINDOWTEXT));
        if (pb_ctl_clr_bg[i] == -2) {
            SetBkMode(hdc, 1 /* TRANSPARENT */);
            return GetStockObject(5 /* NULL_BRUSH */);
        }
        if (pb_ctl_clr_bg[i] < 0) {
            SetBkMode(hdc, 2 /* OPAQUE */);
            return pb_ctl_default_brush(hCtl);
        }
        if (!pb_ctl_clr_br[i]) {
            pb_ctl_clr_br[i] = CreateSolidBrush((unsigned long)pb_ctl_clr_bg[i]);
            if (!pb_ctl_clr_br[i]) return 0;
        }
        SetBkMode(hdc, 2 /* OPAQUE */);
        return pb_ctl_clr_br[i];
    }
    return 0;
}

/* CONTROL SET COLOR hDlg, id&, foreclr&, backclr&
     The colour is remembered, not painted: the help page is explicit that
     a program changing colours after DIALOG SHOW must follow with CONTROL
     REDRAW (or DIALOG REDRAW for several controls at once), so this
     function deliberately does not redraw behind the program's back.
   The pair (-1, -1) means "both defaults", which is the same as never
   having coloured the control, so the slot is released. */
void pb_control_set_color(void* hDlg, long long id, long long fore, long long back) {
    void* h = pb_pb_hwnd(hDlg, id);
    int i, free_i = -1;
    if (!h) return;
    for (i = 0; i < PB_CTL_COLOR_SLOTS; i++) {
        if (pb_ctl_clr_set[i] && pb_ctl_clr_ctl[i] == h) {
            /* only the exact pair (-1, -1) means "both defaults"; -2 is a */
            /* value of its own, meaning "do not paint the text background". */
            if (fore == -1 && back == -1) {
                if (pb_ctl_clr_br[i]) DeleteObject(pb_ctl_clr_br[i]);
                pb_ctl_clr_ctl[i] = 0;
                pb_ctl_clr_par[i] = 0;
                pb_ctl_clr_br[i] = 0;
                pb_ctl_clr_set[i] = 0;
                return;
            }
            if (pb_ctl_clr_bg[i] != (long)back && pb_ctl_clr_br[i]) {
                DeleteObject(pb_ctl_clr_br[i]);
                pb_ctl_clr_br[i] = 0;
            }
            pb_ctl_clr_fg[i] = (long)fore;
            pb_ctl_clr_bg[i] = (long)back;
            return;
        }
        if (!pb_ctl_clr_set[i] && free_i < 0) free_i = i;
    }
    if (free_i < 0 || (fore == -1 && back == -1)) return;
    pb_ctl_clr_ctl[free_i] = h;
    pb_ctl_clr_par[free_i] = hDlg;
    pb_ctl_clr_fg[free_i] = (long)fore;
    pb_ctl_clr_bg[free_i] = (long)back;
    pb_ctl_clr_set[free_i] = 1;
}

void pb_dialog_set_color(void* hDlg, long long fore, long long back) {
    int i, free_i = -1;
    (void)fore;
    for (i = 0; i < PB_DLG_BG_SLOTS; i++) {
        if (pb_dlg_bg_set[i] && pb_dlg_bg_h[i] == hDlg) {
            if (back < 0) pb_dlg_bg_set[i] = 0;
            else pb_dlg_bg_v[i] = (long)back;
            RedrawWindow(hDlg, 0, 0, 0x0001u | 0x0100u | 0x0080u);
            return;
        }
        if (!pb_dlg_bg_set[i] && free_i < 0) free_i = i;
    }
    if (back < 0 || free_i < 0) return;
    pb_dlg_bg_h[free_i] = hDlg;
    pb_dlg_bg_v[free_i] = (long)back;
    pb_dlg_bg_set[free_i] = 1;
    RedrawWindow(hDlg, 0, 0, 0x0001u | 0x0100u | 0x0080u);
}

/* --- DIALOG DEFAULT FONT fontname$ [, points& [, style& [, charset&]]]
   The official statement sets the font for dialogs and controls created
   afterwards.  Here the font is created once and installed on every
   dialog pb_window_new creates from that point on.  It does NOT change
   the dialog base units used for coordinate conversion - DIALOG NEW and
   the geometry statements keep using GetDialogBaseUnits(), which is the
   same assumption DIALOG GET SIZE / SET SIZE already document.

   style&: 0 = normal, 2 = italic (the only two values the help lists).
   charset&: 0 = ANSI, 1 = default, 2 = symbol, 128 = shiftjis,
   129 = hangeul, 134 = gb2312, 136 = chinese, 177 = hebrew,
   178 = arabic, 186 = baltic, 204 = russian, 222 = thai,
   238 = east europe - all passed straight to CreateFontA. */
void pb_dialog_default_font(const char* name, int points, int style, int charset) {
    int height;
    void* dc;
    if (!name || !*name) return;
    if (points <= 0) points = 8;
    dc = GetDC(0);
    height = dc ? -((points * GetDeviceCaps(dc, 90 /* LOGPIXELSY */)) / 72) : -points;
    if (dc) ReleaseDC(0, dc);
    pb_default_font = CreateFontA(height, 0, 0, 0, 400 /* FW_NORMAL */,
                                  (unsigned char)(style == 2 ? 1 : 0), 0, 0,
                                  (unsigned char)charset, 0, 0, 0, 0, name);
}

/* ===================================================================
   Batch 158 - LISTVIEW + TREEVIEW common controls
   -------------------------------------------------------------------
   Every message constant below was read out of the authoritative local
   PowerBASIC include file:  C:\PBWin10\WinAPI\commctrl.inc
   (not from memory).  Struct layouts are the documented x64 layouts.
   =================================================================== */

/* InitCommonControlsEx - registers the common control window classes.
   The legacy InitCommonControls() only registers a few classes, so the
   LISTVIEW / TREEVIEW classes must be requested explicitly. */
typedef struct { unsigned long dwSize; unsigned long dwICC; } PB_ICC;
__declspec(dllimport) int __stdcall InitCommonControlsEx(const PB_ICC*);

#define PB_ICC_LISTVIEW_CLASSES 0x00000001
#define PB_ICC_TREEVIEW_CLASSES 0x00000002
#define PB_ICC_BAR_CLASSES      0x00000004
#define PB_ICC_WIN95_CLASSES    0x000000FF

static void pb_icc(unsigned long flags) {
    PB_ICC icc;
    icc.dwSize = (unsigned long)sizeof(icc);
    icc.dwICC  = flags;
    InitCommonControlsEx(&icc);
}

/* ---- ListView messages (commctrl.inc: LVM_FIRST = &H1000) ---- */
#define PB_LVM_FIRST                  0x1000
#define PB_LVM_GETITEMCOUNT           (PB_LVM_FIRST + 4)   /* 0x1004 */
#define PB_LVM_INSERTITEMA            (PB_LVM_FIRST + 7)   /* 0x1007 */
#define PB_LVM_DELETEITEM             (PB_LVM_FIRST + 8)   /* 0x1008 */
#define PB_LVM_DELETEALLITEMS         (PB_LVM_FIRST + 9)   /* 0x1009 */
#define PB_LVM_INSERTCOLUMNA          (PB_LVM_FIRST + 27)  /* 0x101B */
#define PB_LVM_GETITEMTEXTA           (PB_LVM_FIRST + 45)  /* 0x102D */
#define PB_LVM_SETITEMTEXTA           (PB_LVM_FIRST + 46)  /* 0x102E */
#define PB_LVM_SETEXTENDEDLISTVIEWSTYLE (PB_LVM_FIRST + 54) /* 0x1036 */

#define PB_LVIF_TEXT   0x0001
#define PB_LVIF_IMAGE  0x0002
#define PB_LVIF_PARAM  0x0004
#define PB_LVIF_STATE  0x0008
#define PB_LVCF_FMT    0x0001
#define PB_LVCF_WIDTH  0x0002
#define PB_LVCF_TEXT   0x0004
#define PB_LVCF_SUBITEM 0x0008
#define PB_LVS_EX_FULLROWSELECT 0x20

/* ---- ListView: the rest of the family (batch 170) ----
   Every value below is copied from C:\PBWin10\WINAPI\commctrl.inc so the
   runtime and the official PowerBASIC headers cannot drift apart. */
#define PB_LVM_SETIMAGELIST           (PB_LVM_FIRST + 3)   /* 0x1003 */
#define PB_LVM_GETITEMA               (PB_LVM_FIRST + 5)   /* 0x1005 */
#define PB_LVM_SETITEMA               (PB_LVM_FIRST + 6)   /* 0x1006 */
#define PB_LVM_GETNEXTITEM            (PB_LVM_FIRST + 12)  /* 0x100C */
#define PB_LVM_FINDITEMA              (PB_LVM_FIRST + 13)  /* 0x100D */
#define PB_LVM_ENSUREVISIBLE          (PB_LVM_FIRST + 19)  /* 0x1013 */
#define PB_LVM_GETCOLUMNA             (PB_LVM_FIRST + 25)  /* 0x1019 */
#define PB_LVM_SETCOLUMNA             (PB_LVM_FIRST + 26)  /* 0x101A */
#define PB_LVM_DELETECOLUMN           (PB_LVM_FIRST + 28)  /* 0x101C */
#define PB_LVM_GETCOLUMNWIDTH         (PB_LVM_FIRST + 29)  /* 0x101D */
#define PB_LVM_SETCOLUMNWIDTH         (PB_LVM_FIRST + 30)  /* 0x101E */
#define PB_LVM_GETHEADER              (PB_LVM_FIRST + 31)  /* 0x101F */
#define PB_LVM_SETITEMSTATE           (PB_LVM_FIRST + 43)  /* 0x102B */
#define PB_LVM_GETITEMSTATE           (PB_LVM_FIRST + 44)  /* 0x102C */
#define PB_LVM_SORTITEMS              (PB_LVM_FIRST + 48)  /* 0x1030 */
#define PB_LVM_GETSELECTEDCOUNT       (PB_LVM_FIRST + 50)  /* 0x1032 */
#define PB_LVM_GETEXTENDEDLISTVIEWSTYLE (PB_LVM_FIRST + 55) /* 0x1037 */

#define PB_LVSCW_AUTOSIZE             (-1)
#define PB_LVSCW_AUTOSIZE_USEHEADER   (-2)
#define PB_LVS_TYPEMASK               0x00000003

#define PB_LVFI_STRING   0x0002
#define PB_LVFI_PARTIAL  0x0008

#define PB_LVNI_SELECTED       0x0002
#define PB_LVIS_SELECTED       0x0002
#define PB_LVIS_OVERLAYMASK    0x0F00
#define PB_LVIS_STATEIMAGEMASK 0xF000

/* LISTVIEW SORT option bits.  This is OUR encoding, shared with the
   parser (pb/src/parser.rs, LISTVIEW SORT arm) - it is not a Win32
   constant, and the two sides have to be changed together. */
#define PB_LVSORT_ASCEND    0x0001
#define PB_LVSORT_DESCEND   0x0002
#define PB_LVSORT_ALPHANUM  0x0004
#define PB_LVSORT_UCASE     0x0008
#define PB_LVSORT_NUMERIC   0x0010
#define PB_LVSORT_MMDDYYYY  0x0020
#define PB_LVSORT_DDMMYYYY  0x0040
#define PB_LVSORT_YYYYMMDD  0x0080
#define PB_LVSORT_YYYYDDMM  0x0100

/* x64 LVFINDINFOA.  Offsets: flags 0, psz 8, lParam 16, pt 24,
   vkDirection 32 (size 40).  The PowerBASIC TYPE declares lParam AS
   LONG, but the Win32 member is LPARAM, which is 64-bit on x64. */
typedef struct {
    unsigned int flags;
    const char*  psz;
    long long    lParam;
    long         pt_x;
    long         pt_y;
    unsigned int vkDirection;
} PB_LVFINDINFOA;

/* ---- TreeView messages (commctrl.inc: TV_FIRST = &H1100) ---- */
#define PB_TV_FIRST          0x1100
#define PB_TVM_INSERTITEMA   (PB_TV_FIRST + 0)   /* 0x1100 */
#define PB_TVM_DELETEITEM    (PB_TV_FIRST + 1)   /* 0x1101 */
#define PB_TVM_GETCOUNT      (PB_TV_FIRST + 5)   /* 0x1105 */
#define PB_TVM_GETITEMA      (PB_TV_FIRST + 12)  /* 0x110C */

#define PB_TVIF_TEXT           0x0001
#define PB_TVIF_IMAGE          0x0002
#define PB_TVIF_SELECTEDIMAGE  0x0020
/* ---- TreeView messages / flags added in batch 172 (commctrl.inc) ---- */
#define PB_TVM_EXPAND         (PB_TV_FIRST + 2)   /* 0x1102 */
#define PB_TVM_SETIMAGELIST   (PB_TV_FIRST + 9)   /* 0x1109 */
#define PB_TVM_GETNEXTITEM    (PB_TV_FIRST + 10)  /* 0x110A */
#define PB_TVM_SELECTITEM     (PB_TV_FIRST + 11)  /* 0x110B */
#define PB_TVM_SETITEMA       (PB_TV_FIRST + 13)  /* 0x110D */

#define PB_TVIF_PARAM          0x0004
#define PB_TVIF_STATE          0x0008

#define PB_TVIS_BOLD           0x0010
#define PB_TVIS_EXPANDED       0x0020
#define PB_TVIS_STATEIMAGEMASK 0xF000

/* TVGN_* selectors for TVM_GETNEXTITEM and TVM_SELECTITEM */
#define PB_TVGN_ROOT      0x0000
#define PB_TVGN_NEXT      0x0001
#define PB_TVGN_PREVIOUS  0x0002
#define PB_TVGN_PARENT    0x0003
#define PB_TVGN_CHILD     0x0004
#define PB_TVGN_CARET     0x0009

/* TVE_* actions for TVM_EXPAND */
#define PB_TVE_COLLAPSE   0x0001
#define PB_TVE_EXPAND     0x0002

/* TVSIL_* image-list selector for TVM_SETIMAGELIST */
#define PB_TVSIL_NORMAL   0x0000

/* Attribute selectors shared by pb_treeview_get_attr / pb_treeview_set_attr.
   codegen.rs duplicates these numbers on purpose (it can only emit an integer
   literal); the two lists must be changed together. */
#define PB_TV_ATTR_BOLD       1
#define PB_TV_ATTR_CHECK      2
#define PB_TV_ATTR_CHILD      3
#define PB_TV_ATTR_EXPANDED   4
#define PB_TV_ATTR_NEXT       5
#define PB_TV_ATTR_PARENT     6
#define PB_TV_ATTR_PREVIOUS   7
#define PB_TV_ATTR_ROOT       8
#define PB_TV_ATTR_SELECT     9
#define PB_TV_ATTR_USER      10

/* TreeView special handles. commctrl.inc declares these as
   %TVI_ROOT = &HFFFF0000, i.e. (HTREEITEM)(LONG_PTR)-0x10000.
   On x64 a handle is 64-bit, so the value must be SIGN-extended to
   0xFFFFFFFFFFFF0000 - a zero-extended 0xFFFF0000 would not match. */
#define PB_TVI_ROOT ((void*)(long long)(-0x10000))  /* -65536  */
#define PB_TVI_LAST ((void*)(long long)(-2))        /* 0xFFFFFFFE TVI_LAST */

/* Convenience: a caller that passes 0 for the parent/after slot means
   "root level" / "append as last child" respectively. A caller that
   passes the documented 0xFFFF0000 / 0xFFFFFFFE literals is mapped too. */
static void* pb_tv_parent(void* v) {
    unsigned long long u = (unsigned long long)v;
    if (u == 0ULL || u == 0xFFFF0000ULL) return PB_TVI_ROOT;
    return v;
}
static void* pb_tv_after(void* v) {
    unsigned long long u = (unsigned long long)v;
    if (u == 0ULL || u == 0xFFFFFFFEULL) return PB_TVI_LAST;
    return v;
}

/* x64 LVCOLUMNA. Offsets: mask 0, fmt 4, cx 8, pszText 16,
   cchTextMax 24, iSubItem 28, iImage 32, iOrder 36,
   cxMin 40, cxDefault 44, cxIdeal 48 (size 56). */
typedef struct {
    unsigned int  mask;
    int           fmt;
    int           cx;
    const char*   pszText;
    int           cchTextMax;
    int           iSubItem;
    int           iImage;
    int           iOrder;
    int           cxMin;
    int           cxDefault;
    int           cxIdeal;
} PB_LVCOLUMN;

/* x64 LVITEMA. Offsets: mask 0, iItem 4, iSubItem 8, state 12,
   stateMask 16, pszText 24, cchTextMax 32, iImage 36, lParam 40,
   iIndent 48, iGroupId 52, cColumns 56, puColumns 64,
   piColFmt 72, iGroup 80 (size 88). */
typedef struct {
    unsigned int  mask;
    int           iItem;
    int           iSubItem;
    unsigned int  state;
    unsigned int  stateMask;
    const char*   pszText;
    int           cchTextMax;
    int           iImage;
    long long     lParam;
    int           iIndent;
    int           iGroupId;
    unsigned int  cColumns;
    unsigned int* puColumns;
    int*          piColFmt;
    int           iGroup;
} PB_LVITEM;

/* x64 TVITEMA. Offsets: mask 0, hItem 8, state 16, stateMask 20,
   pszText 24, cchTextMax 32, iImage 36, iSelectedImage 40,
   cChildren 44, lParam 48 (size 56). */
typedef struct {
    unsigned int  mask;
    void*         hItem;
    unsigned int  state;
    unsigned int  stateMask;
    const char*   pszText;
    int           cchTextMax;
    int           iImage;
    int           iSelectedImage;
    int           cChildren;
    long long     lParam;
} PB_TVITEM;

typedef struct {
    void*      hParent;
    void*      hInsertAfter;
    PB_TVITEM  item;
} PB_TVINSERTSTRUCT;

/* ---------------- CONTROL ADD LISTVIEW ---------------- */
void* pb_control_add_listview(void* parent, long id, int x, int y, int w, int ht) {
    /* LVS_REPORT=1 | LVS_SHOWSELALWAYS=8 | WS_CHILD=0x40000000 |
       WS_VISIBLE=0x10000000 | WS_BORDER=0x00800000 | WS_TABSTOP=0x00010000 */
    unsigned long style = 0x1 | 0x8 | 0x40000000 | 0x10000000 | 0x00800000 | 0x00010000;
    void* h;
    pb_icc(PB_ICC_LISTVIEW_CLASSES);
    pb_dlu_to_px(&x, &y, &w, &ht);
    h = CreateWindowExA(0x00000200 /* WS_EX_CLIENTEDGE */, "SysListView32", "",
                        style, x, y, w, ht, parent,
                        (void*)(long long)id, GetModuleHandleA(0), 0);
    if (h) {
        SendMessageA(h, PB_LVM_SETEXTENDEDLISTVIEWSTYLE,
                     PB_LVS_EX_FULLROWSELECT, PB_LVS_EX_FULLROWSELECT);
    }
    return h;
}

/* ---------------- CONTROL ADD TREEVIEW ---------------- */
/* ---------------- CONTROL ADD TREEVIEW ----------------
   The official statement accepts optional style&/exstyle& operands, and an
   explicit primary style REPLACES the documented default:
     "default TreeView style comprises %WS_TABSTOP, %TVS_HASBUTTONS,
      %TVS_LINESATROOT, %TVS_HASLINES, and %TVS_SHOWSELALWAYS"
     "If you include explicit style values, they replace the default values."
   style 0 therefore means "caller omitted them" and selects the default;
   anything else is used verbatim apart from WS_CHILD|WS_VISIBLE, which are
   what make it a visible child control at all.  That is how a caller reaches
   %TVS_CHECKBOXES (0x0100), which TREEVIEW SET/GET CHECK requires. */
void* pb_control_add_treeview(void* parent, long id, int x, int y, int w, int ht,
                              int style_ovr, int exstyle_ovr) {
    /* TVS_HASBUTTONS=1 | TVS_HASLINES=2 | TVS_LINESATROOT=4 |
       TVS_SHOWSELALWAYS=0x20 | WS_CHILD | WS_VISIBLE | WS_BORDER | WS_TABSTOP */
    unsigned long style = (style_ovr != 0)
        ? ((unsigned long)style_ovr | 0x40000000u | 0x10000000u)
        : (0x1u | 0x2u | 0x4u | 0x20u | 0x40000000u | 0x10000000u
           | 0x00800000u | 0x00010000u);
    unsigned long exstyle = (exstyle_ovr != 0) ? (unsigned long)exstyle_ovr
                                               : 0x00000200u /* WS_EX_CLIENTEDGE */;
    pb_icc(PB_ICC_TREEVIEW_CLASSES);
    pb_dlu_to_px(&x, &y, &w, &ht);
    void* h = CreateWindowExA(exstyle, "SysTreeView32", "",
                           style, x, y, w, ht, parent,
                           (void*)(long long)id, GetModuleHandleA(0), 0);
    if (h) SendMessageA(h, 0x0030, (unsigned long long)(size_t)pb_dialog_font(), 1);
    return h;
}

/* ---------------- CONTROL ADD TOOLBAR / STATUSBAR (batch 164) ----------------
   The x/y/width/height operands are deliberately ignored: both controls dock
   themselves inside the parent according to their style bits (%CCS_TOP /
   %CCS_BOTTOM / %SBARS_SIZEGRIP), which is exactly what the official docs
   say about these two forms.  The style / exstyle operands are passed
   straight through to CreateWindowExA. */
void* pb_control_add_toolbar(void* parent, long id, const char* text,
                             int x, int y, int w, int ht,
                             long long style, long long exstyle) {
    unsigned long s = (unsigned long)(style ? style : PB_CCS_TOP);
    void* h;
    (void)text;
    pb_icc(PB_ICC_BAR_CLASSES);
    pb_dlu_to_px(&x, &y, &w, &ht);
    h = CreateWindowExA((unsigned long)exstyle, PB_TOOLBARCLASSNAME, "",
                        s | 0x40000000 /* WS_CHILD */ | 0x10000000 /* WS_VISIBLE */,
                        x, y, w, ht, parent, (void*)(long long)id,
                        GetModuleHandleA(0), 0);
    if (h) {
        SendMessageA(h, (unsigned int)PB_TB_BUTTONSTRUCTSIZE,
                     (unsigned int)sizeof(PB_TBBUTTON), 0);
    }
    return h;
}

void* pb_control_add_statusbar(void* parent, long id, const char* text,
                               int x, int y, int w, int ht,
                               long long style, long long exstyle) {
    unsigned long s = (unsigned long)(style ? style : PB_CCS_BOTTOM);
    void* h;
    (void)text;
    pb_icc(PB_ICC_BAR_CLASSES);
    pb_dlu_to_px(&x, &y, &w, &ht);
    h = CreateWindowExA((unsigned long)exstyle, PB_STATUSCLASSNAME, "",
                        s | 0x40000000 | 0x10000000,
                        x, y, w, ht, parent, (void*)(long long)id,
                        GetModuleHandleA(0), 0);
    return h;
}

/* CONTROL ADD GRAPHIC, hDlg, ID, Txt$, x, y, nWide, nHigh
                     [,style] [,exstyle] [,CALL CallBack] [TO hCtrl&]
   Official source: control_add_graphic.htm.  A graphic control is a STATIC
   the program draws into with the GRAPHIC statements; the help page's
   documented default style is
       %WS_CHILD | %WS_VISIBLE | %SS_OWNERDRAW(0x0B)
   and, exactly as for every other CONTROL ADD form here, a style the
   program supplies REPLACES that default rather than adding to it.
   Txt$ is carried on the window but never painted: a %SS_OWNERDRAW static
   draws nothing itself, and the help page says outright that a graphic
   control does not display its text. */
void* pb_control_add_graphic(void* parent, long id, const char* text,
                             int x, int y, int w, int ht,
                             int style, int exstyle) {
    /* DDT always creates the control as a visible child, whatever extra
       style the program passes; a supplied style only replaces the TYPE
       bits (omit the style and %SS_OWNERDRAW is the documented default). */
    unsigned long s = (unsigned long)style;
    if (!s) s = 0x0000000Bu /* SS_OWNERDRAW */;
    s |= 0x40000000u /* WS_CHILD */ | 0x10000000u /* WS_VISIBLE */;
    pb_dlu_to_px(&x, &y, &w, &ht);
    void* h = CreateWindowExA((unsigned long)exstyle, "STATIC", text, s,
                           x, y, w, ht, parent, (void*)(long long)id,
                           GetModuleHandleA(0), 0);
    if (h) SendMessageA(h, 0x0030, (unsigned long long)(size_t)pb_dialog_font(), 1);
    return h;
}

/* CONTROL ADD HEADER, hDlg, ID, Txt$, x, y, wide, high
                    [,style] [,exstyle] [,CALL CallBack] [TO hCtrl&]
   Official source: CONTROL_ADD_HEADER_statement.htm.  A free-standing
   header control - the common control whose window class is SysHeader32,
   the same class a LISTVIEW creates for its column headings, except that
   here the header belongs to the dialog itself.  Its documented default
   style is %WS_CHILD | %WS_VISIBLE, and the help page recommends id values
   of 1..65535, 100 and up in practice.
   A header displays no text, but the string is still handed to the window
   so a program can read its own label back with CONTROL GET TEXT.
   ICC_WIN95_CLASSES is the InitCommonControlsEx set that registers the
   header class; the legacy InitCommonControls() call in pb_window_new
   happens to cover it as well, but relying on that would be luck. */
void* pb_control_add_header(void* parent, long id, const char* text,
                            int x, int y, int w, int ht,
                            int style, int exstyle) {
    /* A supplied style is additive here too: always a visible child. */
    unsigned long s = (unsigned long)style
                      | 0x40000000u /* WS_CHILD */ | 0x10000000u /* WS_VISIBLE */;
    pb_icc(PB_ICC_WIN95_CLASSES);
    pb_dlu_to_px(&x, &y, &w, &ht);
    void* h = CreateWindowExA((unsigned long)exstyle, "SysHeader32", text, s,
                           x, y, w, ht, parent, (void*)(long long)id,
                           GetModuleHandleA(0), 0);
    if (h) SendMessageA(h, 0x0030, (unsigned long long)(size_t)pb_dialog_font(), 1);
    return h;
}

/* CONTROL ADD classname$ - the generic custom-control form (batch 166).
   The class name arrives as a string expression, so it goes straight to
   CreateWindowExA.  control_add.htm documents no default style for a custom
   control (see Custom_Control_Style_Note.htm): every primary and extended
   style is the caller's, and the style operand is passed through untouched.
   The one exception is an entirely zero style, which would create an invisible
   control; in that case WS_CHILD|WS_VISIBLE is supplied. */
void* pb_control_add_custom(const char* cls, void* parent, long id, const char* text,
                            int x, int y, int w, int ht,
                            long style, long exstyle) {
    unsigned long s = (unsigned long)style;
    void* h;
    if (s == 0) {
        s = 0x40000000 | 0x10000000; /* WS_CHILD | WS_VISIBLE */
    }
    /* register the common Win95 control families first, so a custom control
       built from one of them (e.g. "MSCTLS_TRACKBAR_CLASS32") exists */
    pb_icc(PB_ICC_LISTVIEW_CLASSES | PB_ICC_TREEVIEW_CLASSES | PB_ICC_BAR_CLASSES);
    pb_dlu_to_px(&x, &y, &w, &ht);
    h = CreateWindowExA((unsigned long)exstyle, cls, text ? text : "", s, x, y, w, ht,
                        parent, (void*)(long long)id, GetModuleHandleA(0), 0);
    return h;
}

/* ==================================================================
   SCROLLBAR family (batch 168)

   Official syntax (`SCROLLBAR_statement.htm`) addresses a standalone
   scroll-bar control by (dialog handle, control id), so the real HWND comes
   from GetDlgItem and the bar argument is SB_CTL.  GET returns -1 when the
   control cannot be resolved, which is the documented "no such control"
   result; SET returns 0.
   ================================================================== */

static void* pb_scrollbar_hwnd(void* hDlg, long id) {
    return GetDlgItem(hDlg, (int)id);
}

static int pb_scrollbar_read(void* hDlg, long id, PB_SCROLLINFO* si, unsigned int mask) {
    void* h = pb_scrollbar_hwnd(hDlg, id);
    if (!h) return 0;
    memset(si, 0, sizeof(*si));
    si->cbSize = (unsigned int)sizeof(*si);
    si->fMask  = mask;
    return GetScrollInfo(h, PB_SB_CTL, si);
}

long long pb_scrollbar_get_pos(void* hDlg, long id) {
    PB_SCROLLINFO si;
    if (!pb_scrollbar_read(hDlg, id, &si, PB_SIF_POS)) return -1;
    return si.nPos;
}

long long pb_scrollbar_get_pagesize(void* hDlg, long id) {
    PB_SCROLLINFO si;
    if (!pb_scrollbar_read(hDlg, id, &si, PB_SIF_PAGE)) return -1;
    return (long long)si.nPage;
}

long long pb_scrollbar_get_trackpos(void* hDlg, long id) {
    PB_SCROLLINFO si;
    if (!pb_scrollbar_read(hDlg, id, &si, PB_SIF_TRACKPOS)) return -1;
    return si.nTrackPos;
}

long long pb_scrollbar_get_lo(void* hDlg, long id) {
    PB_SCROLLINFO si;
    if (!pb_scrollbar_read(hDlg, id, &si, PB_SIF_RANGE)) return -1;
    return si.nMin;
}

long long pb_scrollbar_get_hi(void* hDlg, long id) {
    PB_SCROLLINFO si;
    if (!pb_scrollbar_read(hDlg, id, &si, PB_SIF_RANGE)) return -1;
    return si.nMax;
}

static long long pb_scrollbar_write(void* hDlg, long id, int mask, int lo, int hi,
                                    unsigned int page, int pos) {
    PB_SCROLLINFO si;
    void* h = pb_scrollbar_hwnd(hDlg, id);
    if (!h) return 0;
    memset(&si, 0, sizeof(si));
    si.cbSize = (unsigned int)sizeof(si);
    si.fMask  = (unsigned int)mask;
    si.nMin   = lo;
    si.nMax   = hi;
    si.nPage  = page;
    si.nPos   = pos;
    return SetScrollInfo(h, PB_SB_CTL, &si, 1);
}

long long pb_scrollbar_set_range(void* hDlg, long id, int lo, int hi) {
    return pb_scrollbar_write(hDlg, id, PB_SIF_RANGE, lo, hi, 0, 0);
}

long long pb_scrollbar_set_pagesize(void* hDlg, long id, int page) {
    return pb_scrollbar_write(hDlg, id, PB_SIF_PAGE, 0, 0, (unsigned int)page, 0);
}

long long pb_scrollbar_set_pos(void* hDlg, long id, int pos) {
    return pb_scrollbar_write(hDlg, id, PB_SIF_POS, 0, 0, 0, pos);
}

/* ==================================================================
   COMBOBOX / LISTBOX family (batch 168)

   One implementation drives both families; `kind` is 0 for COMBOBOX and 1 for
   LISTBOX.  The official help indexes every item& from ONE ("1 for the first
   item, 2 for the second item, etc."), while the Win32 CB_* / LB_* messages are
   zero-based, so each index is adjusted by -1 going in and by +1 coming out.
   A returned 0 therefore means "no selection" or "no match", exactly as the
   help text specifies.
   ================================================================== */

#define PB_CBLB_CB 0
#define PB_CBLB_LB 1

#define PB_CB_ADDSTRING        0x0143
#define PB_CB_DELETESTRING     0x0144
#define PB_CB_GETCOUNT         0x0146
#define PB_CB_GETCURSEL        0x0147
#define PB_CB_GETLBTEXT        0x0148
#define PB_CB_INSERTSTRING     0x014A
#define PB_CB_RESETCONTENT     0x014B
#define PB_CB_FINDSTRING       0x014C
#define PB_CB_SETCURSEL        0x014E
#define PB_CB_GETITEMDATA      0x0150
#define PB_CB_SETITEMDATA      0x0151
#define PB_CB_FINDSTRINGEXACT  0x0158

#define PB_LB_ADDSTRING        0x0180
#define PB_LB_INSERTSTRING     0x0181
#define PB_LB_DELETESTRING     0x0182
#define PB_LB_RESETCONTENT     0x0184
#define PB_LB_SETSEL           0x0185
#define PB_LB_SETCURSEL        0x0186
#define PB_LB_GETSEL           0x0187
#define PB_LB_GETCURSEL        0x0188
#define PB_LB_GETTEXT          0x0189
#define PB_LB_GETCOUNT         0x018B
#define PB_LB_FINDSTRING       0x018F
#define PB_LB_GETSELCOUNT      0x0190
#define PB_LB_GETITEMDATA      0x0199
#define PB_LB_SETITEMDATA      0x019A
#define PB_LB_FINDSTRINGEXACT  0x01A2

static void* pb_cblb_hwnd(void* hDlg, long id) {
    return GetDlgItem(hDlg, (int)id);
}

/* The item& operand of COMBOBOX/LISTBOX ADD is the newly added string's
   one-based position; a value below one signals an error. */
long long pb_cblb_add(void* hDlg, long id, const char* text, int kind) {
    void* h = pb_cblb_hwnd(hDlg, id);
    long long r;
    if (!h) return -1;
    r = (long long)(intptr_t)SendMessageA(h,
            (unsigned int)(kind == PB_CBLB_LB ? PB_LB_ADDSTRING : PB_CB_ADDSTRING),
            0, (pb_lparam_t)text);
    return r < 0 ? -1 : r + 1;
}

long long pb_cblb_delete(void* hDlg, long id, int item, int kind) {
    void* h = pb_cblb_hwnd(hDlg, id);
    if (!h) return 0;
    return (long long)(intptr_t)SendMessageA(h,
            (unsigned int)(kind == PB_CBLB_LB ? PB_LB_DELETESTRING : PB_CB_DELETESTRING),
            (unsigned int)(item - 1), 0);
}

/* FIND searches for a prefix, FIND EXACT for a whole string; both start at the
   one-based item& and do not wrap.  No match yields 0. */
long long pb_cblb_find(void* hDlg, long id, int item, const char* text,
                       int exact, int kind) {
    void* h = pb_cblb_hwnd(hDlg, id);
    long long r;
    unsigned int m;
    int start;
    if (!h) return 0;
    if (item < 1) item = 1;
    if (kind == PB_CBLB_LB) {
        m = (unsigned int)(exact ? PB_LB_FINDSTRINGEXACT : PB_LB_FINDSTRING);
    } else {
        m = (unsigned int)(exact ? PB_CB_FINDSTRINGEXACT : PB_CB_FINDSTRING);
    }
    /* DDT FIND starts AT the 1-based "item" and, per the help, never wraps.
       The underlying LB/CB FINDSTRING messages DO wrap: wParam is the
       zero-based item preceding the first one searched (-1 = whole list), so
       it is item-2.  A match at or after the first searched index (item-1)
       is a genuine forward hit; any smaller index was reached only by
       wrapping, so it is reported as no match. */
    start = item - 1;
    r = (long long)(intptr_t)SendMessageA(h, m, (unsigned int)(item - 2),
                                          (pb_lparam_t)text);
    if (r < 0) return 0;
    if (r < start) return 0;
    return r + 1;
}

long long pb_cblb_get_count(void* hDlg, long id, int kind) {
    void* h = pb_cblb_hwnd(hDlg, id);
    if (!h) return -1; /* same failure convention as the other GET statements */
    return (long long)(intptr_t)SendMessageA(h,
            (unsigned int)(kind == PB_CBLB_LB ? PB_LB_GETCOUNT : PB_CB_GETCOUNT), 0, 0);
}

long long pb_cblb_get_selcount(void* hDlg, long id, int kind) {
    void* h = pb_cblb_hwnd(hDlg, id);
    long long r;
    if (!h) return 0;
    if (kind == PB_CBLB_LB) {
        r = (long long)(intptr_t)SendMessageA(h, PB_LB_GETSELCOUNT, 0, 0);
        if (r >= 0) return r;
        /* LB_GETSELCOUNT is documented for multiple-selection list boxes only and
           answers LB_ERR for a single-selection one (what CONTROL ADD LISTBOX
           creates), so fall back to counting the selection with LB_GETSEL. */
        {
            long long n = (long long)(intptr_t)SendMessageA(h, PB_LB_GETCOUNT, 0, 0);
            long long i, c = 0;
            if (n < 0) return 0;
            for (i = 0; i < n; i++) {
                if ((long long)(intptr_t)SendMessageA(h, PB_LB_GETSEL,
                                                      (unsigned int)i, 0) > 0) {
                    c++;
                }
            }
            return c;
        }
    }
    /* The help text notes a COMBOBOX is a single-selection list box, so the
       selected count is always zero or one. */
    r = (long long)(intptr_t)SendMessageA(h, PB_CB_GETCURSEL, 0, 0);
    return r < 0 ? 0 : 1;
}

/* GET SELECT returns the one-based index of the first selected item, starting
   the search at one-based `start`, or 0 when nothing is selected. */
long long pb_cblb_get_select(void* hDlg, long id, int start, int kind) {
    void* h = pb_cblb_hwnd(hDlg, id);
    long long r, n, i;
    if (!h) return 0;
    if (kind != PB_CBLB_LB) {
        r = (long long)(intptr_t)SendMessageA(h, PB_CB_GETCURSEL, 0, 0);
        return r < 0 ? 0 : r + 1;
    }
    n = (long long)(intptr_t)SendMessageA(h, PB_LB_GETCOUNT, 0, 0);
    if (n < 0) return 0;
    if (start < 1) start = 1;
    for (i = start - 1; i < n; i++) {
        if ((long long)(intptr_t)SendMessageA(h, PB_LB_GETSEL, (unsigned int)i, 0) > 0) {
            return i + 1;
        }
    }
    return 0;
}

/* GET STATE reports whether one item is selected: -1 (true) or 0 (false). */
long long pb_cblb_get_state(void* hDlg, long id, int item, int kind) {
    void* h = pb_cblb_hwnd(hDlg, id);
    long long r;
    if (!h) return 0;
    if (kind == PB_CBLB_LB) {
        r = (long long)(intptr_t)SendMessageA(h, PB_LB_GETSEL, (unsigned int)(item - 1), 0);
        return r > 0 ? -1 : 0;
    }
    r = (long long)(intptr_t)SendMessageA(h, PB_CB_GETCURSEL, 0, 0);
    return (r == (long long)(item - 1)) ? -1 : 0;
}

/* GET TEXT without an item& (or with item& = 0) returns the selected text. */
void pb_cblb_get_text(void* hDlg, long id, int item, char* out, int outlen, int kind) {
    void* h = pb_cblb_hwnd(hDlg, id);
    long long idx = item;
    unsigned int m = (unsigned int)(kind == PB_CBLB_LB ? PB_LB_GETTEXT : PB_CB_GETLBTEXT);
    unsigned int cur = (unsigned int)(kind == PB_CBLB_LB ? PB_LB_GETCURSEL : PB_CB_GETCURSEL);
    if (outlen > 0) out[0] = 0;
    if (!h) return;
    if (idx <= 0) {
        idx = (long long)(intptr_t)SendMessageA(h, cur, 0, 0);
        if (idx < 0) return;
        idx += 1;
    }
    SendMessageA(h, m, (unsigned int)(idx - 1), (pb_lparam_t)out);
}

long long pb_cblb_get_user(void* hDlg, long id, int item, int kind) {
    void* h = pb_cblb_hwnd(hDlg, id);
    if (!h) return 0;
    return (long long)(intptr_t)SendMessageA(h,
            (unsigned int)(kind == PB_CBLB_LB ? PB_LB_GETITEMDATA : PB_CB_GETITEMDATA),
            (unsigned int)(item - 1), 0);
}

long long pb_cblb_insert(void* hDlg, long id, int item, const char* text, int kind) {
    void* h = pb_cblb_hwnd(hDlg, id);
    long long r;
    if (!h) return -1;
    r = (long long)(intptr_t)SendMessageA(h,
            (unsigned int)(kind == PB_CBLB_LB ? PB_LB_INSERTSTRING : PB_CB_INSERTSTRING),
            (unsigned int)(item - 1), (pb_lparam_t)text);
    return r < 0 ? -1 : r + 1;
}

long long pb_cblb_reset(void* hDlg, long id, int kind) {
    void* h = pb_cblb_hwnd(hDlg, id);
    if (!h) return 0;
    return (long long)(intptr_t)SendMessageA(h,
            (unsigned int)(kind == PB_CBLB_LB ? PB_LB_RESETCONTENT : PB_CB_RESETCONTENT), 0, 0);
}

long long pb_cblb_select(void* hDlg, long id, int item, int kind) {
    void* h = pb_cblb_hwnd(hDlg, id);
    if (!h) return 0;
    return (long long)(intptr_t)SendMessageA(h,
            (unsigned int)(kind == PB_CBLB_LB ? PB_LB_SETCURSEL : PB_CB_SETCURSEL),
            (unsigned int)(item - 1), 0);
}

/* Neither family has a "set item text" message, so the item is deleted and
   re-inserted at the same one-based position. */
long long pb_cblb_set_text(void* hDlg, long id, int item, const char* text, int kind) {
    void* h = pb_cblb_hwnd(hDlg, id);
    long long r;
    if (!h) return -1;
    SendMessageA(h,
            (unsigned int)(kind == PB_CBLB_LB ? PB_LB_DELETESTRING : PB_CB_DELETESTRING),
            (unsigned int)(item - 1), 0);
    r = (long long)(intptr_t)SendMessageA(h,
            (unsigned int)(kind == PB_CBLB_LB ? PB_LB_INSERTSTRING : PB_CB_INSERTSTRING),
            (unsigned int)(item - 1), (pb_lparam_t)text);
    return r < 0 ? -1 : r + 1;
}

long long pb_cblb_set_user(void* hDlg, long id, int item, long long val, int kind) {
    void* h = pb_cblb_hwnd(hDlg, id);
    if (!h) return 0;
    return (long long)(intptr_t)SendMessageA(h,
            (unsigned int)(kind == PB_CBLB_LB ? PB_LB_SETITEMDATA : PB_CB_SETITEMDATA),
            (unsigned int)(item - 1), (pb_lparam_t)(intptr_t)val);
}

/* COMBOBOX UNSELECT has no item&; LISTBOX UNSELECT takes an optional item&, in
   which case only that item is deselected. */
long long pb_cblb_unselect(void* hDlg, long id, int item, int kind) {
    void* h = pb_cblb_hwnd(hDlg, id);
    long long r;
    if (!h) return 0;
    if (kind == PB_CBLB_LB && item > 0) {
        r = (long long)(intptr_t)SendMessageA(h, PB_LB_SETSEL, 0,
                                              (pb_lparam_t)(intptr_t)(item - 1));
        if (r >= 0) return r; /* multiple-selection list box: item deselected */
        /* LB_SETSEL is documented for multiple-selection list boxes only and
           answers LB_ERR for a single-selection one (what CONTROL ADD LISTBOX
           creates), so clear the cursor when that item is the selected one. */
        if ((long long)(intptr_t)SendMessageA(h, PB_LB_GETCURSEL, 0, 0)
            == (long long)(item - 1)) {
            return (long long)(intptr_t)SendMessageA(h, PB_LB_SETCURSEL,
                                                     (unsigned int)-1, 0);
        }
        return 0;
    }
    if (kind == PB_CBLB_LB) {
        /* No item given: drop every selection.  On a multiple-selection list box
           LB_SETCURSEL only moves the cursor, so deselect each item first. */
        long long total = (long long)(intptr_t)SendMessageA(h, PB_LB_GETCOUNT, 0, 0);
        long long i;
        for (i = 0; i < total; i++) {
            SendMessageA(h, PB_LB_SETSEL, 0, (pb_lparam_t)(intptr_t)i);
        }
        return (long long)(intptr_t)SendMessageA(h, PB_LB_SETCURSEL,
                                                 (unsigned int)-1, 0);
    }
    return (long long)(intptr_t)SendMessageA(h, PB_CB_SETCURSEL, (unsigned int)-1, 0);
}

/* ===================================================================
   batch 169: TAB control family (SysTabControl32)

   Official syntax and semantics: TAB_statement.htm.  Every page is a real
   child dialog -- that is what makes TAB INSERT PAGE able to hand back a
   dialog handle -- so a small registry maps (tab HWND, page number) to the
   page dialog.  Page and image numbers are ONE-based, as the official help
   specifies ("the first item is 1, the second item is 2").

   Message and style values below are taken verbatim from the Windows SDK
   (CommCtrl.h) and the PowerBASIC equates (WINAPI\commctrl.inc), not guessed:
     TCM_FIRST 0x1300 | TCM_SETIMAGELIST +3 | TCM_GETITEMCOUNT +4
     TCM_GETITEMA +5 | TCM_SETITEMA +6 | TCM_INSERTITEMA +7
     TCM_DELETEITEM +8 | TCM_DELETEALLITEMS +9 | TCM_GETCURSEL +11
     TCM_SETCURSEL +12 | TCM_ADJUSTRECT +40
     TCS_TABS 0x0000 | TCS_HOTTRACK 0x0040
     TCIF_TEXT 0x0001 | TCIF_IMAGE 0x0002
     ICC_TAB_CLASSES 0x00000008 | WC_TABCONTROLA "SysTabControl32"
   =================================================================== */
#define PB_TCM_SETIMAGELIST   0x1303u
#define PB_TCM_GETITEMCOUNT   0x1304u
#define PB_TCM_GETITEMA       0x1305u
#define PB_TCM_SETITEMA       0x1306u
#define PB_TCM_INSERTITEMA    0x1307u
#define PB_TCM_DELETEITEM     0x1308u
#define PB_TCM_DELETEALLITEMS 0x1309u
#define PB_TCM_GETCURSEL      0x130Bu
#define PB_TCM_SETCURSEL      0x130Cu
#define PB_TCM_ADJUSTRECT     0x1328u
#define PB_TCS_TABS           0x00000000u
#define PB_TCS_HOTTRACK       0x00000040u
#define PB_TCIF_TEXT          0x0001u
#define PB_TCIF_IMAGE         0x0002u
#define PB_ICC_TAB_CLASSES    0x00000008u

/* NMHDR (WinUser.h) and TCITEMA (CommCtrl.h), x64 layout.  The PB include
   file declares these with DWORD fields because it targets 32-bit PBWin; the
   C runtime is 64-bit, so the pointer/UINT_PTR/LPARAM fields are 8 bytes. */
typedef struct {
    void*              hwndFrom;
    unsigned long long idFrom;
    unsigned int       code;
} pb_nmhdr_t;

typedef struct {
    unsigned int       mask;
    unsigned long      dwState;
    unsigned long      dwStateMask;
    char*              pszText;
    int                cchTextMax;
    int                iImage;
    long long          lParam;
} pb_tcitema_t;

#define PB_TAB_MAX_PAGES 256
static void* pb_tab_page_tab[PB_TAB_MAX_PAGES];
static int   pb_tab_page_num[PB_TAB_MAX_PAGES];
static void* pb_tab_page_dlg[PB_TAB_MAX_PAGES];
static int   pb_tab_page_count = 0;
static int   pb_tabpage_registered = 0;

static int pb_tab_page_find(void* hTab, int page) {
    for (int i = 0; i < pb_tab_page_count; i++) {
        if (pb_tab_page_tab[i] == hTab && pb_tab_page_num[i] == page) return i;
    }
    return -1;
}

static void pb_tab_page_forget(void* hTab, int page) {
    int i = pb_tab_page_find(hTab, page);
    if (i < 0) return;
    if (pb_tab_page_dlg[i]) DestroyWindow(pb_tab_page_dlg[i]);
    pb_tab_page_count--;
    for (int j = i; j < pb_tab_page_count; j++) {
        pb_tab_page_tab[j] = pb_tab_page_tab[j + 1];
        pb_tab_page_num[j] = pb_tab_page_num[j + 1];
        pb_tab_page_dlg[j] = pb_tab_page_dlg[j + 1];
    }
}

static void pb_tab_pages_clear(void* hTab) {
    for (int i = pb_tab_page_count - 1; i >= 0; i--) {
        if (pb_tab_page_tab[i] == hTab) pb_tab_page_forget(hTab, pb_tab_page_num[i]);
    }
}

/* Show the page dialog belonging to the current tab and hide the others.
   Driven by the tab control's TCN_SELCHANGE notification and by TAB SELECT. */
static void pb_tab_apply_selection(void* hTab) {
    long long sel;
    if (!hTab) return;
    sel = (long long)(intptr_t)SendMessageA(hTab, PB_TCM_GETCURSEL, 0, 0);
    for (int i = 0; i < pb_tab_page_count; i++) {
        int want;
        if (pb_tab_page_tab[i] != hTab) continue;
        want = (pb_tab_page_num[i] == (int)sel + 1);
        ShowWindow(pb_tab_page_dlg[i], want ? 5 /* SW_SHOW */ : 0 /* SW_HIDE */);
    }
}

/* Page dialogs get their own window class: pb_wndproc posts WM_QUIT on
   WM_DESTROY, which would tear the message loop down when a page is
   destroyed.  They also route WM_COMMAND to the callback named in
   `TAB INSERT PAGE ... CALL cb`, falling back to the dialog callback. */
static long long __stdcall pb_tabpage_wndproc(void* hWnd, unsigned int Msg,
                                              unsigned long long wParam,
                                              unsigned long long lParam) {
    if (Msg == 0x0111 /* WM_COMMAND */) {
        void* fn;
        cb_msg = Msg;
        cb_hwnd = hWnd;
        cb_ctl = (unsigned int)(wParam & 0xFFFF);
        cb_ctlmsg = (unsigned int)(wParam >> 16);
        cb_wparam = wParam;
        cb_lparam = lParam;
        fn = pb_lookup_callback_hwnd(hWnd);
        if (fn) {
            void (*f)(void) = (void(*)(void))fn;
            f();
        } else if (pb_dialog_cb) {
            void (*f)(void) = (void(*)(void))pb_dialog_cb;
            f();
        }
        return 0;
    }
    __try {
        return DefWindowProcA(hWnd, Msg, wParam, lParam);
    } __except(1) {
        return 0;
    }
}

static void pb_tabpage_register_class(void) {
    pb_wndclassex_t wc;
    if (pb_tabpage_registered) return;
    memset(&wc, 0, sizeof(wc));
    wc.cbSize = sizeof(wc);
    wc.lpfnWndProc = (void*)pb_tabpage_wndproc;
    wc.hInstance = GetModuleHandleA(0);
    wc.hCursor = LoadCursorA(0, (const char*)32512);
    wc.hbrBackground = (void*)5;   /* COLOR_WINDOW + 1 */
    wc.lpszClassName = "PBWIN_TABPAGE_CLASS";
    RegisterClassExA(&wc);
    pb_tabpage_registered = 1;
}

/* Resolve the tab control from (dialog handle, control id). */
static void* pb_tab_hwnd(void* hDlg, long id) {
    if (!hDlg) return 0;
    if (id == 0) return hDlg;
    return GetDlgItem(hDlg, id);
}

/* ---------------- CONTROL ADD TAB ---------------- */
void* pb_control_add_tab(void* parent, long id, int x, int y, int w, int ht) {
    /* TCS_TABS | TCS_HOTTRACK | WS_CHILD | WS_VISIBLE | WS_CLIPSIBLINGS | WS_TABSTOP */
    unsigned long style = PB_TCS_TABS | PB_TCS_HOTTRACK
                        | 0x40000000u | 0x10000000u | 0x04000000u | 0x00010000u;
    pb_icc(PB_ICC_TAB_CLASSES);
    pb_dlu_to_px(&x, &y, &w, &ht);
    void* h = CreateWindowExA(0x00000200 /* WS_EX_CLIENTEDGE */, "SysTabControl32", "",
                           style, x, y, w, ht, parent,
                           (void*)(long long)id, GetModuleHandleA(0), 0);
    if (h) SendMessageA(h, 0x0030, (unsigned long long)(size_t)pb_dialog_font(), 1);
    return h;
}

/* ---------------- TAB DELETE hDlg, ID&, PageNum& ---------------- */
long long pb_tab_delete(void* hDlg, long id, int page) {
    void* h = pb_tab_hwnd(hDlg, id);
    long long r;
    if (!h || page < 1) return -1;
    r = (long long)(intptr_t)SendMessageA(h, PB_TCM_DELETEITEM,
                                          (unsigned int)(page - 1), 0);
    pb_tab_page_forget(h, page);
    /* pages after the deleted one shift down by one */
    for (int i = 0; i < pb_tab_page_count; i++) {
        if (pb_tab_page_tab[i] == h && pb_tab_page_num[i] > page) pb_tab_page_num[i]--;
    }
    pb_tab_apply_selection(h);
    return r ? 0 : -1;
}

/* ---------------- TAB GET COUNT hDlg, ID& TO CountVar& ---------------- */
long long pb_tab_get_count(void* hDlg, long id) {
    void* h = pb_tab_hwnd(hDlg, id);
    if (!h) return -1;   /* same failure convention as the other GET statements */
    return (long long)(intptr_t)SendMessageA(h, PB_TCM_GETITEMCOUNT, 0, 0);
}

/* ---------------- TAB GET DIALOG hDlg, ID&, PageNum& TO PageDlgVar& ------ */
long long pb_tab_get_dialog(void* hDlg, long id, int page) {
    void* h = pb_tab_hwnd(hDlg, id);
    int i;
    if (!h || page < 1) return 0;
    i = pb_tab_page_find(h, page);
    if (i < 0) return 0;   /* page does not exist -> 0 (official) */
    return (long long)(intptr_t)pb_tab_page_dlg[i];
}

/* ---------------- TAB GET IMAGE hDlg, ID&, PageNum& TO ImageVar& --------- */
long long pb_tab_get_image(void* hDlg, long id, int page) {
    void* h = pb_tab_hwnd(hDlg, id);
    pb_tcitema_t it;
    if (!h || page < 1) return 0;
    memset(&it, 0, sizeof(it));
    it.mask = PB_TCIF_IMAGE;
    if (!SendMessageA(h, PB_TCM_GETITEMA, (unsigned int)(page - 1),
                      (pb_lparam_t)(size_t)&it)) {
        return 0;
    }
    if (it.iImage < 0) return 0;   /* no image -> 0 (official) */
    return (long long)it.iImage + 1;
}

/* ---------------- TAB GET PAGE PageDlg TO PageNumVar& -------------------
   The input is the page dialog handle, not an (hDlg, id) pair. */
long long pb_tab_get_page(void* hPage) {
    for (int i = 0; i < pb_tab_page_count; i++) {
        if (pb_tab_page_dlg[i] == hPage) return (long long)pb_tab_page_num[i];
    }
    return 0;
}

/* ---------------- TAB GET SELECT hDlg, ID& TO PageNumVar& --------------- */
long long pb_tab_get_select(void* hDlg, long id) {
    void* h = pb_tab_hwnd(hDlg, id);
    long long sel;
    if (!h) return 0;
    sel = (long long)(intptr_t)SendMessageA(h, PB_TCM_GETCURSEL, 0, 0);
    if (sel < 0) return 0;   /* no current selection -> 0 (official) */
    return sel + 1;          /* pages are 1-based */
}

/* ---------------- TAB GET TEXT hDlg, ID&, PageNum& TO TextVar$ ---------- */
long long pb_tab_get_text(void* hDlg, long id, int page, char* out, int outlen) {
    void* h = pb_tab_hwnd(hDlg, id);
    pb_tcitema_t it;
    if (!out || outlen <= 0) return 0;
    out[0] = 0;
    if (!h || page < 1) return 0;
    memset(&it, 0, sizeof(it));
    it.mask = PB_TCIF_TEXT;
    it.pszText = out;
    it.cchTextMax = outlen;
    SendMessageA(h, PB_TCM_GETITEMA, (unsigned int)(page - 1),
                 (pb_lparam_t)(size_t)&it);
    return 1;
}

/* ---- TAB$(TEXT, hDlg, ID&, PageNum&) — official function form ---------- */
void* pb_tab_get_text_str(void* hDlg, long id, int page) {
    void* h = pb_tab_hwnd(hDlg, id);
    char buf[512];
    pb_tcitema_t it;
    if (!h || page < 1) return pb_bstr_alloc("", 0);
    memset(&it, 0, sizeof(it));
    it.mask = PB_TCIF_TEXT;
    it.pszText = buf;
    it.cchTextMax = 512;
    buf[0] = 0;
    SendMessageA(h, PB_TCM_GETITEMA, (unsigned int)(page - 1),
                 (pb_lparam_t)(size_t)&it);
    return pb_bstr_alloc(buf, (unsigned int)strlen(buf));
}

/* ---------------- TAB INSERT PAGE hDlg, ID&, PageNum&, Image&, Text$
                    [CALL cb] TO PageDlgVar& --------------------------- */
long long pb_tab_insert_page(void* hDlg, long id, int page, int image,
                             const char* text, void* cb) {
    void* h = pb_tab_hwnd(hDlg, id);
    pb_tcitema_t it;
    long rc;
    long rcs[4];
    void* hPage;
    int slot;
    if (!h || page < 1) return 0;
    pb_tabpage_register_class();
    memset(&it, 0, sizeof(it));
    it.mask = PB_TCIF_TEXT | (image > 0 ? PB_TCIF_IMAGE : 0u);
    it.pszText = (char*)(text ? text : "");
    it.iImage = image > 0 ? image - 1 : 0;   /* image numbers are 1-based */
    rc = (long)(intptr_t)SendMessageA(h, PB_TCM_INSERTITEMA,
                                      (unsigned int)(page - 1),
                                      (pb_lparam_t)(size_t)&it);
    if (rc < 0) return 0;

    /* the page dialog fills the tab control's display area */
    rcs[0] = 0; rcs[1] = 0; rcs[2] = 0; rcs[3] = 0;
    GetClientRect(h, rcs);
    SendMessageA(h, PB_TCM_ADJUSTRECT, 0 /* FALSE: give the display area */,
                 (pb_lparam_t)(size_t)rcs);
    hPage = CreateWindowExA(0, "PBWIN_TABPAGE_CLASS", "",
                            0x40000000u /* WS_CHILD */ | 0x10000000u /* WS_VISIBLE */,
                            rcs[0], rcs[1], rcs[2] - rcs[0], rcs[3] - rcs[1],
                            h, 0, GetModuleHandleA(0), 0);
    if (!hPage) {
        SendMessageA(h, PB_TCM_DELETEITEM, (unsigned int)(page - 1), 0);
        return 0;
    }
    if (cb) pb_register_callback_hwnd(hPage, cb);
    if (pb_tab_page_count < PB_TAB_MAX_PAGES) {
        slot = pb_tab_page_count++;
        pb_tab_page_tab[slot] = h;
        pb_tab_page_num[slot] = page;
        pb_tab_page_dlg[slot] = hPage;
    }
    /* pages inserted before an existing one shift up */
    for (int i = 0; i < pb_tab_page_count; i++) {
        if (pb_tab_page_tab[i] == h && pb_tab_page_dlg[i] != hPage
            && pb_tab_page_num[i] >= page) {
            pb_tab_page_num[i]++;
        }
    }
    /* the first page ever inserted becomes the current one */
    if ((long long)(intptr_t)SendMessageA(h, PB_TCM_GETCURSEL, 0, 0) < 0) {
        SendMessageA(h, PB_TCM_SETCURSEL, (unsigned int)(page - 1), 0);
    }
    pb_tab_apply_selection(h);
    return (long long)(intptr_t)hPage;
}

/* ---------------- TAB RESET hDlg, ID& ---------------- */
long long pb_tab_reset(void* hDlg, long id) {
    void* h = pb_tab_hwnd(hDlg, id);
    if (!h) return -1;
    SendMessageA(h, PB_TCM_DELETEALLITEMS, 0, 0);
    pb_tab_pages_clear(h);
    return 0;
}

/* ---------------- TAB SELECT hDlg, ID&, PageNum& ---------------- */
long long pb_tab_select(void* hDlg, long id, int page) {
    void* h = pb_tab_hwnd(hDlg, id);
    if (!h || page < 1) return -1;
    SendMessageA(h, PB_TCM_SETCURSEL, (unsigned int)(page - 1), 0);
    pb_tab_apply_selection(h);
    return 0;
}

/* ---------------- TAB SET IMAGE hDlg, ID&, PageNum&, Image& ------------- */
long long pb_tab_set_image(void* hDlg, long id, int page, int image) {
    void* h = pb_tab_hwnd(hDlg, id);
    pb_tcitema_t it;
    if (!h || page < 1) return -1;
    memset(&it, 0, sizeof(it));
    it.mask = PB_TCIF_IMAGE;
    it.iImage = image > 0 ? image - 1 : -1;   /* image numbers are 1-based */
    return SendMessageA(h, PB_TCM_SETITEMA, (unsigned int)(page - 1),
                        (pb_lparam_t)(size_t)&it) ? 0 : -1;
}

/* ---------------- TAB SET IMAGELIST hDlg, ID&, hLst ---------------------
   The IMAGELIST is owned by the tab control from here on: it is destroyed
   together with the control (official semantics). */
long long pb_tab_set_imagelist(void* hDlg, long id, void* hLst) {
    void* h = pb_tab_hwnd(hDlg, id);
    if (!h) return -1;
    SendMessageA(h, PB_TCM_SETIMAGELIST, 0, (pb_lparam_t)(size_t)hLst);
    return 0;
}

/* ---------------- TAB SET TEXT hDlg, ID&, PageNum&, Text$ --------------- */
long long pb_tab_set_text(void* hDlg, long id, int page, const char* text) {
    void* h = pb_tab_hwnd(hDlg, id);
    pb_tcitema_t it;
    if (!h || page < 1) return -1;
    memset(&it, 0, sizeof(it));
    it.mask = PB_TCIF_TEXT;
    it.pszText = (char*)(text ? text : "");
    it.cchTextMax = 0;
    return SendMessageA(h, PB_TCM_SETITEMA, (unsigned int)(page - 1),
                        (pb_lparam_t)(size_t)&it) ? 0 : -1;
}

/* ---------------- LISTVIEW family ----------------
   PowerBASIC numbers items and columns from 1 ("First=1, second=2...",
   see LISTVIEW_statement.htm), but every Win32 list-view message takes a
   0-based index.  The conversion is done here, once, for the whole
   family, so no caller has to think about it.

   batch 170 corrected the batch 158 functions below: they passed the
   PowerBASIC numbers straight through to SendMessageA and were therefore
   off by one for every caller that followed the documented 1-based
   convention.  examples/batch158_listview_treeview.bas was written
   against the old 0-based behaviour and was updated with this batch. */

static void* pb_lv_hwnd(void* hDlg, long id) {
    if (!hDlg) return 0;
    return GetDlgItem(hDlg, (int)id);
}

/* 1-based PowerBASIC index -> 0-based Win32 index, clamped at 0. */
static int pb_lv_idx(int pb_index) {
    int i = pb_index - 1;
    return i < 0 ? 0 : i;
}

/* ---------------- LISTVIEW INSERT COLUMN ---------------- */
void pb_listview_insert_column(void* hDlg, long id, int col, const char* text,
                               int width, int fmt) {
    PB_LVCOLUMN c;
    void* h = pb_lv_hwnd(hDlg, id);
    if (!h) return;
    memset(&c, 0, sizeof(c));
    c.mask     = PB_LVCF_FMT | PB_LVCF_WIDTH | PB_LVCF_TEXT | PB_LVCF_SUBITEM;
    c.fmt      = fmt;
    c.cx       = width;
    c.pszText  = (const char*)text;
    c.iSubItem = pb_lv_idx(col);
    SendMessageA(h, PB_LVM_INSERTCOLUMNA, (unsigned int)pb_lv_idx(col),
                 (pb_lparam_t)(size_t)&c);
}

/* ---------------- LISTVIEW INSERT ITEM ----------------
   item& is the documented row number, but LVM_INSERTITEM always appends,
   so iItem is advisory only and the row lands at the end of the control.
   This matches the official remark that the remaining columns are empty
   until LISTVIEW SET TEXT fills them. */
void pb_listview_insert_item(void* hDlg, long id, int item, int image,
                             const char* text) {
    PB_LVITEM it;
    void* h = pb_lv_hwnd(hDlg, id);
    if (!h) return;
    memset(&it, 0, sizeof(it));
    it.mask     = PB_LVIF_TEXT | PB_LVIF_IMAGE;
    it.iItem    = pb_lv_idx(item);
    it.iSubItem = 0;
    it.pszText  = (const char*)text;
    it.iImage   = image;
    SendMessageA(h, PB_LVM_INSERTITEMA, 0, (pb_lparam_t)(size_t)&it);
}

/* ---------------- LISTVIEW GET COUNT ---------------- */
long long pb_listview_get_count(void* hDlg, long id) {
    void* h = pb_lv_hwnd(hDlg, id);
    /* -1 on an unresolvable control, matching the rest of the family
       (TAB GET COUNT, LISTVIEW GET MODE / GET SELCOUNT / GET COLUMN).
       A negative count can never be a real answer, so it is unambiguous. */
    if (!h) return -1;
    return (long long)(intptr_t)SendMessageA(h, PB_LVM_GETITEMCOUNT, 0, 0);
}

/* ---------------- LISTVIEW GET TEXT ---------------- */
void pb_listview_get_text(void* hDlg, long id, int item, int col,
                          char* out, int outlen) {
    PB_LVITEM it;
    void* h;
    if (outlen > 0) out[0] = 0;
    h = pb_lv_hwnd(hDlg, id);
    if (!h) return;
    memset(&it, 0, sizeof(it));
    it.mask       = PB_LVIF_TEXT;
    it.iItem      = pb_lv_idx(item);
    it.iSubItem   = pb_lv_idx(col);
    it.pszText    = out;
    it.cchTextMax = outlen;
    SendMessageA(h, PB_LVM_GETITEMTEXTA, (unsigned int)pb_lv_idx(item),
                 (pb_lparam_t)(size_t)&it);
}

/* ---------------- LISTVIEW SET TEXT ---------------- */
void pb_listview_set_text(void* hDlg, long id, int item, int col,
                          const char* text) {
    PB_LVITEM it;
    void* h = pb_lv_hwnd(hDlg, id);
    if (!h) return;
    memset(&it, 0, sizeof(it));
    it.mask     = PB_LVIF_TEXT;
    it.iItem    = pb_lv_idx(item);
    it.iSubItem = pb_lv_idx(col);
    it.pszText  = (const char*)text;
    SendMessageA(h, PB_LVM_SETITEMTEXTA, (unsigned int)pb_lv_idx(item),
                 (pb_lparam_t)(size_t)&it);
}

/* ---------------- LISTVIEW DELETE ITEM ---------------- */
void pb_listview_delete_item(void* hDlg, long id, int item) {
    void* h = pb_lv_hwnd(hDlg, id);
    if (!h) return;
    SendMessageA(h, PB_LVM_DELETEITEM, (unsigned int)pb_lv_idx(item), 0);
}

/* ---------------- LISTVIEW RESET ----------------
   Deletes every data item; columns and their headers survive, as the
   official documentation requires. */
void pb_listview_reset(void* hDlg, long id) {
    void* h = pb_lv_hwnd(hDlg, id);
    if (!h) return;
    SendMessageA(h, PB_LVM_DELETEALLITEMS, 0, 0);
}

/* ============== batch 170: the rest of the LISTVIEW family ============== */

/* ---------------- LISTVIEW DELETE COLUMN ----------------
   Windows refuses to delete column 1 of a list-view control; the
   official documentation states this explicitly and tells the caller to
   insert a zero-width dummy column instead. */
void pb_listview_delete_column(void* hDlg, long id, int col) {
    void* h = pb_lv_hwnd(hDlg, id);
    if (!h) return;
    SendMessageA(h, PB_LVM_DELETECOLUMN, (unsigned int)pb_lv_idx(col), 0);
}

/* ---------------- LISTVIEW FIND / LISTVIEW FIND EXACT ----------------
   Searches column 1 only, case-insensitively, from item& to the last
   item, without wrapping.  Returns the 1-based index of the match, or 0
   when nothing matches - exactly the documented contract.

   `exact` picks LVFI_STRING (whole string) over LVFI_PARTIAL (leading
   substring); that is the only difference between the two statements. */
long long pb_listview_find(void* hDlg, long id, int item, const char* needle,
                           int exact) {
    PB_LVFINDINFOA fi;
    void* h = pb_lv_hwnd(hDlg, id);
    intptr_t r;
    if (!h) return 0;
    memset(&fi, 0, sizeof(fi));
    fi.flags = exact ? PB_LVFI_STRING : PB_LVFI_PARTIAL;
    fi.psz   = (const char*)needle;
    r = (intptr_t)SendMessageA(h, PB_LVM_FINDITEMA,
                               (unsigned int)pb_lv_idx(item),
                               (pb_lparam_t)(size_t)&fi);
    if (r < 0) return 0;
    return (long long)r + 1;   /* Win32 0-based -> PowerBASIC 1-based */
}

/* ---------------- LISTVIEW FIT CONTENT / FIT HEADER ----------------
   LVM_SETCOLUMNWIDTH with LVSCW_AUTOSIZE (-1) fits the column to its
   data, LVSCW_AUTOSIZE_USEHEADER (-2) to data plus header text.  Both
   must be sign-extended into the LPARAM: a zero-extended -1 would not
   reach Windows as the documented sentinel. */
void pb_listview_fit_content(void* hDlg, long id, int col) {
    void* h = pb_lv_hwnd(hDlg, id);
    if (!h) return;
    SendMessageA(h, PB_LVM_SETCOLUMNWIDTH, (unsigned int)pb_lv_idx(col),
                 (pb_lparam_t)(long long)PB_LVSCW_AUTOSIZE);
}

void pb_listview_fit_header(void* hDlg, long id, int col) {
    void* h = pb_lv_hwnd(hDlg, id);
    if (!h) return;
    SendMessageA(h, PB_LVM_SETCOLUMNWIDTH, (unsigned int)pb_lv_idx(col),
                 (pb_lparam_t)(long long)PB_LVSCW_AUTOSIZE_USEHEADER);
}

/* ---------------- LISTVIEW GET COLUMN ----------------
   The current width of the column, in the unit chosen at creation.
   Returns -1 when the control cannot be resolved. */
long long pb_listview_get_column(void* hDlg, long id, int col) {
    void* h = pb_lv_hwnd(hDlg, id);
    if (!h) return -1;
    return (long long)(intptr_t)SendMessageA(h, PB_LVM_GETCOLUMNWIDTH,
                                             (unsigned int)pb_lv_idx(col), 0);
}

/* ---------------- LISTVIEW SET COLUMN ----------------
   Any value other than the two documented sentinels is a width. */
void pb_listview_set_column(void* hDlg, long id, int col, int width) {
    void* h = pb_lv_hwnd(hDlg, id);
    if (!h) return;
    SendMessageA(h, PB_LVM_SETCOLUMNWIDTH, (unsigned int)pb_lv_idx(col),
                 (pb_lparam_t)(long long)width);
}

/* ---------------- LISTVIEW GET HEADER ---------------- */
void pb_listview_get_header(void* hDlg, long id, int col, char* out,
                            int outlen) {
    PB_LVCOLUMN c;
    void* h;
    if (outlen > 0) out[0] = 0;
    h = pb_lv_hwnd(hDlg, id);
    if (!h) return;
    memset(&c, 0, sizeof(c));
    c.mask       = PB_LVCF_TEXT;
    c.pszText    = out;
    c.cchTextMax = outlen;
    SendMessageA(h, PB_LVM_GETCOLUMNA, (unsigned int)pb_lv_idx(col),
                 (pb_lparam_t)(size_t)&c);
}

/* ---------------- LISTVIEW SET HEADER ---------------- */
void pb_listview_set_header(void* hDlg, long id, int col, const char* text) {
    PB_LVCOLUMN c;
    void* h = pb_lv_hwnd(hDlg, id);
    if (!h) return;
    memset(&c, 0, sizeof(c));
    c.mask    = PB_LVCF_TEXT;
    c.pszText = (const char*)text;
    SendMessageA(h, PB_LVM_SETCOLUMNA, (unsigned int)pb_lv_idx(col),
                 (pb_lparam_t)(size_t)&c);
}

/* ---------------- LISTVIEW GET HEADERID ----------------
   Hands back the list-view handle itself plus the window id of the
   HEADER control embedded in it, so the pair can be fed to the HEADER
   statement.  The id is read with GWL_ID (-12), which is what
   GetDlgCtrlID does internally, so no extra import is needed. */
void pb_listview_get_headerid(void* hDlg, long id, void** out_hlv,
                              long* out_hid) {
    void* h = pb_lv_hwnd(hDlg, id);
    void* hdr;
    if (out_hlv) *out_hlv = h;
    if (out_hid) *out_hid = 0;
    if (!h) return;
    hdr = (void*)(intptr_t)SendMessageA(h, PB_LVM_GETHEADER, 0, 0);
    if (out_hid) *out_hid = hdr ? (long)pb_get_winlong(hdr, -12) : 0;
}

/* ---------------- LISTVIEW GET MODE ----------------
   The four classic display modes live in the LVS_TYPEMASK bits of the
   control style: 0=icon, 1=report, 2=small icon, 3=list. */
long long pb_listview_get_mode(void* hDlg, long id) {
    void* h = pb_lv_hwnd(hDlg, id);
    if (!h) return -1;
    return pb_get_winlong(h, -16 /* GWL_STYLE */) & PB_LVS_TYPEMASK;
}

/* ---------------- LISTVIEW SET MODE ----------------
   Replaces only the LVS_TYPEMASK bits so every other style bit
   survives.  There is no list-view message for this; the window style
   has to be rewritten. */
void pb_listview_set_mode(void* hDlg, long id, int mode) {
    void* h = pb_lv_hwnd(hDlg, id);
    long long style;
    if (!h) return;
    style = pb_get_winlong(h, -16 /* GWL_STYLE */);
    style = (style & ~(long long)PB_LVS_TYPEMASK)
          | (long long)((unsigned int)mode & PB_LVS_TYPEMASK);
    pb_set_winlong(h, -16 /* GWL_STYLE */, style);
}

/* ---------------- LISTVIEW GET SELCOUNT ---------------- */
long long pb_listview_get_selcount(void* hDlg, long id) {
    void* h = pb_lv_hwnd(hDlg, id);
    if (!h) return -1;
    return (long long)(intptr_t)SendMessageA(h, PB_LVM_GETSELECTEDCOUNT, 0, 0);
}

/* ---------------- LISTVIEW GET SELECT ----------------
   The next selected primary item at or after item&, as a 1-based index,
   or 0 when there is none.  Callers walk a multi-selection by feeding
   the previous result back in as item&. */
long long pb_listview_get_select(void* hDlg, long id, int start) {
    void* h = pb_lv_hwnd(hDlg, id);
    intptr_t r;
    if (!h) return 0;
    r = (intptr_t)SendMessageA(h, PB_LVM_GETNEXTITEM,
                               (unsigned int)pb_lv_idx(start),
                               (pb_lparam_t)PB_LVNI_SELECTED);
    if (r < 0) return 0;
    return (long long)r + 1;
}

/* ---------------- LISTVIEW GET STATE ----------------
   -1 when the item is selected, 0 otherwise, per the official
   documentation.  col& is accepted because the official syntax has it,
   but Windows tracks selection per item rather than per sub-item, so it
   does not change the answer. */
long long pb_listview_get_state(void* hDlg, long id, int item, int col) {
    void* h = pb_lv_hwnd(hDlg, id);
    unsigned int st;
    (void)col;
    if (!h) return 0;
    st = (unsigned int)(intptr_t)SendMessageA(h, PB_LVM_GETITEMSTATE,
                                              (unsigned int)pb_lv_idx(item),
                                              (pb_lparam_t)PB_LVIS_SELECTED);
    return (st & PB_LVIS_SELECTED) ? -1 : 0;
}

/* ---------------- LISTVIEW SELECT / LISTVIEW UNSELECT ----------------
   Sets or clears LVIS_SELECTED on one item.  UNSELECT takes the same
   optional col& as SELECT and, like GET STATE, the column does not
   affect the outcome on Windows. */
void pb_listview_select(void* hDlg, long id, int item, int col) {
    PB_LVITEM it;
    void* h = pb_lv_hwnd(hDlg, id);
    (void)col;
    if (!h) return;
    memset(&it, 0, sizeof(it));
    it.mask      = PB_LVIF_STATE;
    it.iItem     = pb_lv_idx(item);
    it.state     = PB_LVIS_SELECTED;
    it.stateMask = PB_LVIS_SELECTED;
    SendMessageA(h, PB_LVM_SETITEMSTATE, (unsigned int)pb_lv_idx(item),
                 (pb_lparam_t)(size_t)&it);
}

void pb_listview_unselect(void* hDlg, long id, int item, int col) {
    PB_LVITEM it;
    void* h = pb_lv_hwnd(hDlg, id);
    (void)col;
    if (!h) return;
    memset(&it, 0, sizeof(it));
    it.mask      = PB_LVIF_STATE;
    it.iItem     = pb_lv_idx(item);
    it.state     = 0;
    it.stateMask = PB_LVIS_SELECTED;
    SendMessageA(h, PB_LVM_SETITEMSTATE, (unsigned int)pb_lv_idx(item),
                 (pb_lparam_t)(size_t)&it);
}

/* ---------------- LISTVIEW GET STYLEXX / SET STYLEXX ----------------
   The list-view specific extended style (the %LVS_EX_* set), which is
   deliberately distinct from the primary and extended styles given to
   CONTROL ADD LISTVIEW. */
long long pb_listview_get_stylexx(void* hDlg, long id) {
    void* h = pb_lv_hwnd(hDlg, id);
    if (!h) return -1;
    return (long long)(intptr_t)SendMessageA(h, PB_LVM_GETEXTENDEDLISTVIEWSTYLE,
                                             0, 0);
}

void pb_listview_set_stylexx(void* hDlg, long id, int style) {
    void* h = pb_lv_hwnd(hDlg, id);
    if (!h) return;
    /* The %LVS_EX_* mask is 32 bits wide and two of its members
       (%LVS_EX_COLUMNSNAPPOINTS, %LVS_EX_COLUMNOVERFLOW) have bit 31
       set, so truncate to unsigned before widening into the LPARAM. */
    SendMessageA(h, PB_LVM_SETEXTENDEDLISTVIEWSTYLE, 0,
                 (pb_lparam_t)(long long)(unsigned int)style);
}

/* ---------------- LISTVIEW GET USER / LISTVIEW SET USER ----------------
   The per-row user value is the item lParam.  The official
   documentation warns that LISTVIEW SORT overwrites it, because the
   sort callback is handed the lParam of each row. */
long long pb_listview_get_user(void* hDlg, long id, int item) {
    PB_LVITEM it;
    void* h = pb_lv_hwnd(hDlg, id);
    if (!h) return 0;
    memset(&it, 0, sizeof(it));
    it.mask  = PB_LVIF_PARAM;
    it.iItem = pb_lv_idx(item);
    if (!SendMessageA(h, PB_LVM_GETITEMA, (unsigned int)pb_lv_idx(item),
                      (pb_lparam_t)(size_t)&it))
        return 0;
    return (long long)it.lParam;
}

void pb_listview_set_user(void* hDlg, long id, int item, int value) {
    PB_LVITEM it;
    void* h = pb_lv_hwnd(hDlg, id);
    if (!h) return;
    memset(&it, 0, sizeof(it));
    it.mask   = PB_LVIF_PARAM;
    it.iItem  = pb_lv_idx(item);
    it.lParam = (long long)value;
    SendMessageA(h, PB_LVM_SETITEMA, (unsigned int)pb_lv_idx(item),
                 (pb_lparam_t)(size_t)&it);
}

/* ---------------- LISTVIEW SET IMAGE ----------------
   The primary image index for the row; 0 means no image. */
void pb_listview_set_image(void* hDlg, long id, int item, int image) {
    PB_LVITEM it;
    void* h = pb_lv_hwnd(hDlg, id);
    if (!h) return;
    memset(&it, 0, sizeof(it));
    it.mask     = PB_LVIF_IMAGE;
    it.iItem    = pb_lv_idx(item);
    it.iSubItem = 0;
    it.iImage   = image;
    SendMessageA(h, PB_LVM_SETITEMA, (unsigned int)pb_lv_idx(item),
                 (pb_lparam_t)(size_t)&it);
}

/* ---------------- LISTVIEW SET IMAGE2 ----------------
   Secondary status image.  It lives in the LVIS_STATEIMAGEMASK bits as
   a 1-based index shifted left by 12 (INDEXTOSTATEIMAGEMASK); 0 hides
   it.  The official documentation caps it at 15. */
void pb_listview_set_image2(void* hDlg, long id, int item, int image) {
    PB_LVITEM it;
    void* h = pb_lv_hwnd(hDlg, id);
    if (!h) return;
    if (image < 0) image = 0;
    if (image > 15) image = 15;
    memset(&it, 0, sizeof(it));
    it.mask      = PB_LVIF_STATE;
    it.iItem     = pb_lv_idx(item);
    it.state     = (unsigned int)((unsigned int)image << 12);
    it.stateMask = PB_LVIS_STATEIMAGEMASK;
    SendMessageA(h, PB_LVM_SETITEMSTATE, (unsigned int)pb_lv_idx(item),
                 (pb_lparam_t)(size_t)&it);
}

/* ---------------- LISTVIEW SET OVERLAY ----------------
   The overlay index lives in the LVIS_OVERLAYMASK bits, shifted left by
   8 (INDEXTOOVERLAYMASK); 0 removes the overlay. */
void pb_listview_set_overlay(void* hDlg, long id, int item, int overlay) {
    PB_LVITEM it;
    void* h = pb_lv_hwnd(hDlg, id);
    if (!h) return;
    if (overlay < 0) overlay = 0;
    if (overlay > 15) overlay = 15;
    memset(&it, 0, sizeof(it));
    it.mask      = PB_LVIF_STATE;
    it.iItem     = pb_lv_idx(item);
    it.state     = (unsigned int)((unsigned int)overlay << 8);
    it.stateMask = PB_LVIS_OVERLAYMASK;
    SendMessageA(h, PB_LVM_SETITEMSTATE, (unsigned int)pb_lv_idx(item),
                 (pb_lparam_t)(size_t)&it);
}

/* ---------------- LISTVIEW SET IMAGELIST ----------------
   which& is one of %LVSIL_NORMAL (large icons), %LVSIL_SMALL (small
   icons) or %LVSIL_STATE (status images). */
void pb_listview_set_imagelist(void* hDlg, long id, void* hLst, int which) {
    void* h = pb_lv_hwnd(hDlg, id);
    if (!h) return;
    SendMessageA(h, PB_LVM_SETIMAGELIST, (unsigned int)which,
                 (pb_lparam_t)(size_t)hLst);
}

/* ---------------- LISTVIEW VISIBLE ----------------
   Scrolls the control, if necessary, so the row is on screen. */
void pb_listview_visible(void* hDlg, long id, int item) {
    void* h = pb_lv_hwnd(hDlg, id);
    if (!h) return;
    SendMessageA(h, PB_LVM_ENSUREVISIBLE, (unsigned int)pb_lv_idx(item), 0);
}

/* ---------------- LISTVIEW SORT ----------------
   The option keywords are folded into a single bitmask by the parser
   (see the LISTVIEW SORT arm in pb/src/parser.rs); PB_LVSORT_* above is
   that shared encoding and is not a Win32 constant.

   Windows sorts through a callback that receives only the two item
   lParams and the sort lParam, so the comparison routine reaches its state
   through the file-scope struct below.  (That struct predates batch 176,
   when wParam of SendMessageA was still declared 32-bit and a pointer could
   not travel that way.  wParam is pointer-width since batch 176, but the
   struct stays: it is measured, and nothing here needs the extra channel.  LVM_SORTITEMS is
   synchronous, so a single shared slot is safe as long as nothing sorts
   re-entrantly from inside the comparison callback.

   The callback fetches each row's text, so every row must carry its own
   index as the lParam - which is exactly the overwrite of USER data the
   official documentation warns about. */

typedef struct {
    void* h;
    int   col;    /* 1-based, as written by the programmer */
    int   mode;   /* PB_LVSORT_* bits */
} PB_LVSORTCTX;

static PB_LVSORTCTX pb_lvsort_ctx;

/* Split a string into up to three numeric fields.  The four date
   formats are "exactly ten bytes" with arbitrary delimiters and possibly
   spaces in place of leading zeros, so splitting on field boundaries is
   more robust than counting digits. */
static int pb_lvsort_fields(const char* s, long* f) {
    int n = 0;
    if (!s) return 0;
    while (*s && n < 3) {
        long v = 0;
        while (*s && !(*s >= '0' && *s <= '9')) s++;
        if (!*s) break;
        while (*s >= '0' && *s <= '9') {
            v = v * 10 + (*s - '0');
            s++;
        }
        f[n++] = v;
    }
    return n;
}

/* One comparable yyyymmdd key per documented date layout.
   order: 0 = mm/dd/yyyy, 1 = dd/mm/yyyy, 2 = yyyy/mm/dd, 3 = yyyy/dd/mm. */
static long pb_lvsort_datekey(const char* s, int order) {
    long f[3];
    if (pb_lvsort_fields(s, f) < 3) return 0;
    switch (order) {
    case 0:  return f[2] * 10000 + f[0] * 100 + f[1];
    case 1:  return f[2] * 10000 + f[1] * 100 + f[0];
    case 2:  return f[0] * 10000 + f[1] * 100 + f[2];
    default: return f[0] * 10000 + f[2] * 100 + f[1];
    }
}

static int pb_lvsort_cmp_num(const char* a, const char* b) {
    double da = a ? atof(a) : 0.0;
    double db = b ? atof(b) : 0.0;
    if (da < db) return -1;
    if (da > db) return 1;
    return 0;
}

/* Case-insensitive compare, the UCASE option.  Hand-rolled so the
   runtime does not depend on the CRT spelling of _stricmp. */
static int pb_lvsort_cmp_ci(const char* a, const char* b) {
    while (*a && *b) {
        int ca = toupper((unsigned char)*a);
        int cb = toupper((unsigned char)*b);
        if (ca != cb) return ca < cb ? -1 : 1;
        a++;
        b++;
    }
    if (*a == *b) return 0;
    return *a ? 1 : -1;
}

static void pb_lvsort_text(void* h, int idx, int col, char* out, int outlen) {
    PB_LVITEM it;
    if (outlen > 0) out[0] = 0;
    memset(&it, 0, sizeof(it));
    it.mask       = PB_LVIF_TEXT;
    it.iItem      = idx;
    it.iSubItem   = pb_lv_idx(col);
    it.pszText    = out;
    it.cchTextMax = outlen;
    SendMessageA(h, PB_LVM_GETITEMTEXTA, (unsigned int)idx,
                 (pb_lparam_t)(size_t)&it);
}

static int __stdcall pb_lvsort_cb(pb_lparam_t l1, pb_lparam_t l2,
                                 pb_lparam_t sortparam) {
    char ta[256], tb[256];
    int mode = pb_lvsort_ctx.mode;
    int r;
    (void)sortparam;   /* the state travels in pb_lvsort_ctx, see above */
    pb_lvsort_text(pb_lvsort_ctx.h, (int)l1, pb_lvsort_ctx.col, ta,
                   (int)sizeof(ta));
    pb_lvsort_text(pb_lvsort_ctx.h, (int)l2, pb_lvsort_ctx.col, tb,
                   (int)sizeof(tb));
    if (mode & PB_LVSORT_NUMERIC) {
        r = pb_lvsort_cmp_num(ta, tb);
    } else if (mode & PB_LVSORT_MMDDYYYY) {
        long ka = pb_lvsort_datekey(ta, 0), kb = pb_lvsort_datekey(tb, 0);
        r = (ka < kb) ? -1 : (ka > kb) ? 1 : 0;
    } else if (mode & PB_LVSORT_DDMMYYYY) {
        long ka = pb_lvsort_datekey(ta, 1), kb = pb_lvsort_datekey(tb, 1);
        r = (ka < kb) ? -1 : (ka > kb) ? 1 : 0;
    } else if (mode & PB_LVSORT_YYYYMMDD) {
        long ka = pb_lvsort_datekey(ta, 2), kb = pb_lvsort_datekey(tb, 2);
        r = (ka < kb) ? -1 : (ka > kb) ? 1 : 0;
    } else if (mode & PB_LVSORT_YYYYDDMM) {
        long ka = pb_lvsort_datekey(ta, 3), kb = pb_lvsort_datekey(tb, 3);
        r = (ka < kb) ? -1 : (ka > kb) ? 1 : 0;
    } else if (mode & PB_LVSORT_UCASE) {
        r = pb_lvsort_cmp_ci(ta, tb);
    } else {
        /* ALPHANUM, and the default when no comparison option is given:
           sequenced on the ASCII value of each byte, so case matters. */
        r = strcmp(ta, tb);
    }
    if (mode & PB_LVSORT_DESCEND) r = -r;
    return r;
}

void pb_listview_sort(void* hDlg, long id, int col, int mode) {
    void* h = pb_lv_hwnd(hDlg, id);
    int i, n;
    if (!h) return;
    n = (int)(intptr_t)SendMessageA(h, PB_LVM_GETITEMCOUNT, 0, 0);
    for (i = 0; i < n; i++) {
        PB_LVITEM it;
        memset(&it, 0, sizeof(it));
        it.mask   = PB_LVIF_PARAM;
        it.iItem  = i;
        it.lParam = (long long)i;
        SendMessageA(h, PB_LVM_SETITEMA, (unsigned int)i,
                     (pb_lparam_t)(size_t)&it);
    }
    pb_lvsort_ctx.h    = h;
    pb_lvsort_ctx.col  = col;
    pb_lvsort_ctx.mode = mode;
    SendMessageA(h, PB_LVM_SORTITEMS, 0,
                 (pb_lparam_t)(size_t)pb_lvsort_cb);
}

/* ---------------- TREEVIEW INSERT ITEM ---------------- */
void pb_treeview_insert_item(void* hDlg, long id, void* hParent, void* hAfter,
                             int image, int simage, const char* text,
                             void** out) {
    PB_TVINSERTSTRUCT ins;
    void* h;
    void* r;
    if (out) *out = 0;
    h = GetDlgItem(hDlg, (int)id);
    if (!h) return;
    memset(&ins, 0, sizeof(ins));
    ins.hParent      = pb_tv_parent(hParent);
    ins.hInsertAfter = pb_tv_after(hAfter);
    ins.item.mask          = PB_TVIF_TEXT | PB_TVIF_IMAGE | PB_TVIF_SELECTEDIMAGE;
    ins.item.pszText       = (const char*)text;
    ins.item.cchTextMax    = 0;
    ins.item.iImage        = image;
    ins.item.iSelectedImage = simage;
    r = SendMessageA(h, PB_TVM_INSERTITEMA, 0, (pb_lparam_t)&ins);
    if (out) *out = r;
}

/* ---------------- TREEVIEW GET COUNT ---------------- */
long long pb_treeview_get_count(void* hDlg, long id) {
    void* h = GetDlgItem(hDlg, (int)id);
    if (!h) return 0;
    return (long long)(intptr_t)SendMessageA(h, PB_TVM_GETCOUNT, 0, 0);
}

/* ---------------- TREEVIEW GET TEXT ---------------- */
void pb_treeview_get_text(void* hDlg, long id, void* hItem,
                          char* out, int outlen) {
    PB_TVITEM it;
    void* h;
    if (outlen > 0) out[0] = 0;
    h = GetDlgItem(hDlg, (int)id);
    if (!h) return;
    memset(&it, 0, sizeof(it));
    it.mask       = PB_TVIF_TEXT;
    it.hItem      = hItem;
    it.pszText    = out;
    it.cchTextMax = outlen;
    SendMessageA(h, PB_TVM_GETITEMA, 0, (pb_lparam_t)&it);
}

/* ---------------- TREEVIEW DELETE ---------------- */
void pb_treeview_delete(void* hDlg, long id, void* hItem) {
    void* h = GetDlgItem(hDlg, (int)id);
    if (!h) return;
    SendMessageA(h, PB_TVM_DELETEITEM, 0, (pb_lparam_t)hItem);
}

/* ---------------- TREEVIEW RESET ---------------- */
/* commctrl.inc implements TreeView_DeleteAllItems as
   SendMessage(hWnd, %TVM_DELETEITEM, 0, %TVI_ROOT). */
void pb_treeview_reset(void* hDlg, long id) {
    void* h = GetDlgItem(hDlg, (int)id);
    if (!h) return;
    SendMessageA(h, PB_TVM_DELETEITEM, 0, (pb_lparam_t)PB_TVI_ROOT);
}
/* ---------------- TREEVIEW attribute get / set (batch 172) ----------------
   `which` is one of the PB_TV_ATTR_* selectors defined above, duplicated as
   integer literals in codegen.rs - the two lists must be changed together.

   PB semantics come straight from the official `TREEVIEW statement` help:
     GET BOLD / CHECK / EXPANDED              -> true (-1) / false (0)
     GET CHILD / NEXT / PARENT / PREVIOUS /
         ROOT / SELECT                        -> item handle, 0 when no such item
     GET USER                                 -> the LONG stored by SET USER
   Every "no such item" path therefore returns 0, which is what the help page
   promises, and the Win32 control already returns 0 for a NULL handle. */

long long pb_treeview_get_attr(void* hDlg, long id, void* hItem, int which) {
    PB_TVITEM it;
    void* h = GetDlgItem(hDlg, (int)id);
    unsigned int gn;
    if (!h) return 0;

    /* ROOT / SELECT take no item handle; the other navigators take one. */
    switch (which) {
    case PB_TV_ATTR_ROOT:     gn = PB_TVGN_ROOT;     break;
    case PB_TV_ATTR_SELECT:   gn = PB_TVGN_CARET;    break;
    case PB_TV_ATTR_CHILD:    gn = PB_TVGN_CHILD;    break;
    case PB_TV_ATTR_NEXT:     gn = PB_TVGN_NEXT;     break;
    case PB_TV_ATTR_PARENT:   gn = PB_TVGN_PARENT;   break;
    case PB_TV_ATTR_PREVIOUS: gn = PB_TVGN_PREVIOUS; break;
    default:                  gn = 0xFFFFFFFFu;      break;
    }
    if (gn != 0xFFFFFFFFu) {
        return (long long)(intptr_t)SendMessageA(
            h, PB_TVM_GETNEXTITEM, gn,
            (which == PB_TV_ATTR_ROOT || which == PB_TV_ATTR_SELECT)
                ? 0 : (pb_lparam_t)hItem);
    }

    memset(&it, 0, sizeof(it));
    it.hItem = hItem;
    switch (which) {
    case PB_TV_ATTR_BOLD:
    case PB_TV_ATTR_EXPANDED:
        it.mask      = PB_TVIF_STATE;
        it.stateMask = (which == PB_TV_ATTR_BOLD) ? PB_TVIS_BOLD
                                                  : PB_TVIS_EXPANDED;
        SendMessageA(h, PB_TVM_GETITEMA, 0, (pb_lparam_t)&it);
        return (it.state & it.stateMask) ? -1 : 0;
    case PB_TV_ATTR_CHECK:
        /* With %TVS_CHECKBOXES the state image index is 1 = clear, 2 = checked.
           A control built without that style reports index 0, and that must
           read back as "not checked" - hence >= 2, not "!= 1". */
        it.mask      = PB_TVIF_STATE;
        it.stateMask = PB_TVIS_STATEIMAGEMASK;
        SendMessageA(h, PB_TVM_GETITEMA, 0, (pb_lparam_t)&it);
        return (((it.state & PB_TVIS_STATEIMAGEMASK) >> 12) >= 2) ? -1 : 0;
    case PB_TV_ATTR_USER:
        it.mask = PB_TVIF_PARAM;
        SendMessageA(h, PB_TVM_GETITEMA, 0, (pb_lparam_t)&it);
        return it.lParam;
    default:
        return 0;
    }
}

void pb_treeview_set_attr(void* hDlg, long id, void* hItem, int which,
                          long long flag) {
    PB_TVITEM it;
    void* h = GetDlgItem(hDlg, (int)id);
    if (!h) return;

    if (which == PB_TV_ATTR_EXPANDED) {
        /* commctrl.inc implements TreeView_Expand as TVM_EXPAND + TVE_* */
        SendMessageA(h, PB_TVM_EXPAND,
                     flag ? PB_TVE_EXPAND : PB_TVE_COLLAPSE, (pb_lparam_t)hItem);
        return;
    }
    memset(&it, 0, sizeof(it));
    it.hItem = hItem;
    switch (which) {
    case PB_TV_ATTR_BOLD:
        it.mask      = PB_TVIF_STATE;
        it.stateMask = PB_TVIS_BOLD;
        it.state     = flag ? PB_TVIS_BOLD : 0;
        break;
    case PB_TV_ATTR_CHECK:
        /* index 1 = clear, 2 = checked, expressed in bits 12..15 */
        it.mask      = PB_TVIF_STATE;
        it.stateMask = PB_TVIS_STATEIMAGEMASK;
        it.state     = (flag ? 2u : 1u) << 12;
        break;
    case PB_TV_ATTR_USER:
        it.mask   = PB_TVIF_PARAM;
        it.lParam = flag;
        break;
    default:
        return;
    }
    SendMessageA(h, PB_TVM_SETITEMA, 0, (pb_lparam_t)&it);
}

/* ---------------- TREEVIEW SELECT / UNSELECT ---------------- */
void pb_treeview_select(void* hDlg, long id, void* hItem) {
    void* h = GetDlgItem(hDlg, (int)id);
    if (!h) return;
    SendMessageA(h, PB_TVM_SELECTITEM, PB_TVGN_CARET, (pb_lparam_t)hItem);
}

/* PB: "All items in the TREEVIEW control are set to an unselected state."
   TVM_SELECTITEM with a NULL item is the only way the Win32 control offers
   to drop the current selection. */
void pb_treeview_unselect(void* hDlg, long id) {
    void* h = GetDlgItem(hDlg, (int)id);
    if (!h) return;
    SendMessageA(h, PB_TVM_SELECTITEM, PB_TVGN_CARET, 0);
}

/* ---------------- TREEVIEW SET TEXT ---------------- */
void pb_treeview_set_text(void* hDlg, long id, void* hItem, const char* text) {
    PB_TVITEM it;
    void* h = GetDlgItem(hDlg, (int)id);
    if (!h) return;
    memset(&it, 0, sizeof(it));
    it.mask       = PB_TVIF_TEXT;
    it.hItem      = hItem;
    it.pszText    = (const char*)text;
    it.cchTextMax = 0;
    SendMessageA(h, PB_TVM_SETITEMA, 0, (pb_lparam_t)&it);
}

/* ---------------- TREEVIEW SET IMAGELIST ---------------- */
/* commctrl.inc implements TreeView_SetImageList as
   SendMessage(hWnd, %TVM_SETIMAGELIST, %TVSIL_NORMAL, hLst). */
void pb_treeview_set_imagelist(void* hDlg, long id, void* hLst) {
    void* h = GetDlgItem(hDlg, (int)id);
    if (!h) return;
    SendMessageA(h, PB_TVM_SETIMAGELIST, PB_TVSIL_NORMAL, (pb_lparam_t)hLst);
}

/* ---------------- batch 249: _ex variants (style/exstyle/CALL) ------------
   Mirrors pb_control_add_button_ex: style==0 -> documented PBWin default;
   exstyle==0 -> 0; fn -> pb_register_callback_hwnd. */
void* pb_control_add_checkbox_ex(void* parent, long id, const char* text,
                                 int x, int y, int w, int h,
                                 long style, long exstyle, void* fn) {
    pb_dlu_to_px(&x, &y, &w, &h);
    unsigned long s = style ? (unsigned long)style : 0x3u | 0x100u | 0xC00u | 0x10000u;
    s |= 0x40000000u | 0x10000000u;
    void* hw = CreateWindowExA((unsigned long)exstyle, "BUTTON", text, s,
                           x, y, w, h, parent,
                           (void*)(long long)id, GetModuleHandleA(0), 0);
    if (hw) { SendMessageA(hw, 0x0030, (unsigned long long)(size_t)pb_dialog_font(), 1);
              if (fn) pb_register_callback_hwnd(hw, fn); }
    return hw;
}
void* pb_control_add_radiobutton_ex(void* parent, long id, const char* text,
                                    int x, int y, int w, int h,
                                    long style, long exstyle, void* fn) {
    pb_dlu_to_px(&x, &y, &w, &h);
    unsigned long s = style ? (unsigned long)style : 0x9u | 0x10000u;
    s |= 0x40000000u | 0x10000000u;
    void* hw = CreateWindowExA((unsigned long)exstyle, "BUTTON", text, s,
                           x, y, w, h, parent,
                           (void*)(long long)id, GetModuleHandleA(0), 0);
    if (hw) { SendMessageA(hw, 0x0030, (unsigned long long)(size_t)pb_dialog_font(), 1);
              if (fn) pb_register_callback_hwnd(hw, fn); }
    return hw;
}
void* pb_control_add_groupbox_ex(void* parent, long id, const char* text,
                                int x, int y, int w, int h,
                                long style, long exstyle, void* fn) {
    pb_dlu_to_px(&x, &y, &w, &h);
    unsigned long s = style ? (unsigned long)style : 0x7u;
    s |= 0x40000000u | 0x10000000u;
    void* hw = CreateWindowExA((unsigned long)exstyle, "BUTTON", text, s,
                           x, y, w, h, parent,
                           (void*)(long long)id, GetModuleHandleA(0), 0);
    if (hw) { SendMessageA(hw, 0x0030, (unsigned long long)(size_t)pb_dialog_font(), 1);
              if (fn) pb_register_callback_hwnd(hw, fn); }
    return hw;
}
void* pb_control_add_option_ex(void* parent, long id, const char* text,
                               int x, int y, int w, int h,
                               long style, long exstyle, void* fn) {
    pb_dlu_to_px(&x, &y, &w, &h);
    unsigned long s = style ? (unsigned long)style : 0x9u | 0x100u | 0xC00u | 0x10000u;
    s |= 0x40000000u | 0x10000000u;
    void* hw = CreateWindowExA((unsigned long)exstyle, "BUTTON", text, s,
                           x, y, w, h, parent,
                           (void*)(long long)id, GetModuleHandleA(0), 0);
    if (hw) { SendMessageA(hw, 0x0030, (unsigned long long)(size_t)pb_dialog_font(), 1);
              if (fn) pb_register_callback_hwnd(hw, fn); }
    return hw;
}
void* pb_control_add_check3state_ex(void* parent, long id, const char* text,
                                    int x, int y, int w, int h,
                                    long style, long exstyle, void* fn) {
    pb_dlu_to_px(&x, &y, &w, &h);
    unsigned long s = style ? (unsigned long)style : 0x6u | 0x100u | 0xC00u | 0x10000u;
    s |= 0x40000000u | 0x10000000u;
    void* hw = CreateWindowExA((unsigned long)exstyle, "BUTTON", text, s,
                           x, y, w, h, parent,
                           (void*)(long long)id, GetModuleHandleA(0), 0);
    if (hw) { SendMessageA(hw, 0x0030, (unsigned long long)(size_t)pb_dialog_font(), 1);
              if (fn) pb_register_callback_hwnd(hw, fn); }
    return hw;
}
void* pb_control_add_frame_ex(void* parent, long id, const char* text,
                              int x, int y, int w, int h,
                              long style, long exstyle, void* fn) {
    pb_dlu_to_px(&x, &y, &w, &h);
    unsigned long s = style ? (unsigned long)style : 0x7u | 0x100u | 0x400u;
    s |= 0x40000000u | 0x10000000u;
    void* hw = CreateWindowExA((unsigned long)exstyle, "BUTTON", text, s,
                           x, y, w, h, parent,
                           (void*)(long long)id, GetModuleHandleA(0), 0);
    if (hw) { SendMessageA(hw, 0x0030, (unsigned long long)(size_t)pb_dialog_font(), 1);
              if (fn) pb_register_callback_hwnd(hw, fn); }
    return hw;
}
void* pb_control_add_textbox_ex(void* parent, long id, const char* text,
                                int x, int y, int w, int h,
                                long style, long exstyle, void* fn) {
    x = PB_DLU_X_TO_PX(x);
    y = PB_DLU_Y_TO_PX(y);
    w = PB_DLU_X_TO_PX(w);
    int hpx = PB_DLU_Y_TO_PX(h);
    unsigned long s = style ? (unsigned long)style : 0x80u | 0x10000u | 0x800000u;
    s |= 0x40000000u | 0x10000000u;
    unsigned long ex = exstyle ? (unsigned long)exstyle : 0x200u;
    void* hw = CreateWindowExA(ex, "EDIT", text, s,
                           x, y, w, hpx, parent,
                           (void*)(long long)id, GetModuleHandleA(0), 0);
    if (hw) { SendMessageA(hw, 0x0030, (unsigned long long)(size_t)pb_dialog_font(), 1);
              if (fn) pb_register_callback_hwnd(hw, fn); }
    return hw;
}
void* pb_control_add_line_ex(void* parent, long id, const char* text,
                             int x, int y, int w, int h,
                             long style, long exstyle, void* fn) {
    pb_dlu_to_px(&x, &y, &w, &h);
    unsigned long s = style ? (unsigned long)style : 0x12u;
    s |= 0x40000000u | 0x10000000u;
    void* hw = CreateWindowExA((unsigned long)exstyle, "STATIC", text, s,
                           x, y, w, h, parent,
                           (void*)(long long)id, GetModuleHandleA(0), 0);
    if (hw) { SendMessageA(hw, 0x0030, (unsigned long long)(size_t)pb_dialog_font(), 1);
              if (fn) pb_register_callback_hwnd(hw, fn); }
    return hw;
}
void* pb_control_add_label_ex(void* parent, long id, const char* text,
                              int x, int y, int w, int ht,
                              long style, long exstyle, void* fn) {
    unsigned long s = style ? (unsigned long)style : 0u;
    s |= 0x40000000u | 0x10000000u;
    pb_dlu_to_px(&x, &y, &w, &ht);
    ht = 16;
    void* h = CreateWindowExA((unsigned long)exstyle, "STATIC", text, s,
                           x, y, w, ht, parent,
                           (void*)(long long)id, GetModuleHandleA(0), 0);
    if (h) { SendMessageA(h, 0x0030, (unsigned long long)(size_t)pb_dialog_font(), 1);
             if (fn) pb_register_callback_hwnd(h, fn); }
    return h;
}
void* pb_control_add_scrollbar_ex(void* parent, long id, const char* text,
                                  int x, int y, int w, int ht,
                                  long style, long exstyle, void* fn) {
    unsigned long sbs = (w < ht) ? 1u : 0u;
    unsigned long s = style ? (unsigned long)style : sbs;
    s |= 0x40000000u | 0x10000000u;
    void* hbar = CreateWindowExA((unsigned long)exstyle, "SCROLLBAR", text, s,
                            x, y, w, ht, parent,
                            (void*)(long long)id, GetModuleHandleA(0), 0);
    SendMessageA(hbar, 0x00F4 /* SBM_SETRANGE */, 0, 100);
    /* CHW: default page size is 10 on creation. */
    if (hbar) {
        struct { unsigned int cbSize; unsigned int fMask; int nMin; int nMax;
                 unsigned int nPage; int nPos; int nTrackPos; } si;
        memset(&si, 0, sizeof(si));
        si.cbSize = (unsigned int)sizeof(si);
        si.fMask = 0x0002 /* SIF_PAGE */;
        si.nPage = 10;
        SendMessageA(hbar, 0x00E9 /* SBM_SETSCROLLINFO */, 0, (long long)&si);
    }
    if (hbar) { SendMessageA(hbar, 0x0030, (unsigned long long)(size_t)pb_dialog_font(), 1);
                if (fn) pb_register_callback_hwnd(hbar, fn); }
    return hbar;
}
void* pb_control_add_tab_ex(void* parent, long id, const char* text,
                            int x, int y, int w, int ht,
                            long style, long exstyle, void* fn) {
    unsigned long s = style ? (unsigned long)style
                            : PB_TCS_TABS | PB_TCS_HOTTRACK | 0x04000000u | 0x00010000u;
    s |= 0x40000000u | 0x10000000u;
    unsigned long ex = exstyle ? (unsigned long)exstyle : 0x00000200u;
    pb_icc(PB_ICC_TAB_CLASSES);
    pb_dlu_to_px(&x, &y, &w, &ht);
    void* h = CreateWindowExA(ex, "SysTabControl32", text,
                           s, x, y, w, ht, parent,
                           (void*)(long long)id, GetModuleHandleA(0), 0);
    if (h) { SendMessageA(h, 0x0030, (unsigned long long)(size_t)pb_dialog_font(), 1);
             if (fn) pb_register_callback_hwnd(h, fn); }
    return h;
}

void* pb_control_add_editbox_ex(void* parent, long id, const char* text,
                                int x, int y, int w, int h,
                                long style, long exstyle, void* fn) {
    pb_dlu_to_px(&x, &y, &w, &h);
    unsigned long s = style ? (unsigned long)style : 0x80u | 0x800000u | 0x10000u;
    s |= 0x40000000u | 0x10000000u;
    unsigned long ex = exstyle ? (unsigned long)exstyle : 0x200u;
    void* hw = CreateWindowExA(ex, "EDIT", text, s,
                           x, y, w, h, parent,
                           (void*)(long long)id, GetModuleHandleA(0), 0);
    if (hw) { SendMessageA(hw, 0x0030, (unsigned long long)(size_t)pb_dialog_font(), 1);
              if (fn) pb_register_callback_hwnd(hw, fn); }
    return hw;
}
