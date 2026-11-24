/* Android console compatibility for the PB64Basic runtime.
 *
 * The Windows runtime draws PRINT/CLS/LOCATE/CON.PRINT into a Win32 console
 * screen buffer through GetConsoleScreenBufferInfo + WriteConsoleOutputCharacterA
 * + SetConsoleCursorPosition.  Android has no Win32 console, so this file
 * provides a small 80x25 character/attribute buffer and mirrors completed rows
 * into logcat (tag "PB64Basic").  The buffer is intentionally simple: it is
 * enough for PRINT, PRINT NEWLINE, CLS and LOCATE, and it keeps the runtime's
 * cursor arithmetic intact instead of special-casing every PB statement.
 */
#include <android/log.h>
#include <string.h>
#include <time.h>

typedef struct { short X; short Y; } PB_COORD;
typedef unsigned int PB_DWORD;
typedef struct { short Left; short Top; short Right; short Bottom; } PB_SMALL_RECT;
typedef struct {
    PB_COORD dwSize;
    PB_COORD dwCursorPosition;
    unsigned short wAttributes;
    PB_SMALL_RECT srWindow;
    PB_COORD dwMaximumWindowSize;
} PB_CONSOLE_SCREEN_BUFFER_INFO;

#define PB_ANDROID_ROWS 25
#define PB_ANDROID_COLS 80
#define PB_ANDROID_HANDLE_COUNT 3

static int g_handles[PB_ANDROID_HANDLE_COUNT];
static char g_text[PB_ANDROID_ROWS][PB_ANDROID_COLS];
static unsigned short g_attr[PB_ANDROID_ROWS][PB_ANDROID_COLS];
static PB_COORD g_cursor = {0, 0};
static unsigned short g_current_attr = 0x0007; /* grey on black */
static int g_initialized = 0;
#define PB_ANDROID_CAPTURE 8192
static char g_capture[PB_ANDROID_CAPTURE];
static unsigned short g_capture_attr[PB_ANDROID_CAPTURE];
static size_t g_capture_len = 0;

static void console_init(void) {
    int r, c;
    if (g_initialized) return;
    for (r = 0; r < PB_ANDROID_ROWS; r++) {
        for (c = 0; c < PB_ANDROID_COLS; c++) {
            g_text[r][c] = ' ';
            g_attr[r][c] = g_current_attr;
        }
    }
    g_initialized = 1;
}

static int is_console_handle(void* h) {
    return h == (void*)&g_handles[0] || h == (void*)&g_handles[1] ||
           h == (void*)&g_handles[2];
}

/* Reset the emulated screen and the JNI capture buffer before a new PB run.
 * The runtime keeps console state across main() calls; without this, every
 * re-run appended to the previous run's captured text. */
void pb_android_console_clear(void) {
    int r, c;
    console_init();
    for (r = 0; r < PB_ANDROID_ROWS; r++) {
        for (c = 0; c < PB_ANDROID_COLS; c++) {
            g_text[r][c] = ' ';
            g_attr[r][c] = 0x0007;
        }
    }
    g_cursor.X = 0;
    g_cursor.Y = 0;
    g_current_attr = 0x0007;
    g_capture_len = 0;
    g_capture[0] = '\0';
    g_capture_attr[0] = 0;
}

static void flush_row(int row) {
    char line[PB_ANDROID_COLS + 1];
    int i, last = -1;
    if (row < 0 || row >= PB_ANDROID_ROWS) return;
    console_init();
    for (i = 0; i < PB_ANDROID_COLS; i++) {
        line[i] = g_text[row][i];
        if (line[i] != ' ') last = i;
    }
    line[last + 1] = '\0';
    if (last < 0) line[0] = '\0';
    /* Keep a copy for the JNI bridge.  logcat remains the primary channel. */
    if (g_capture_len + (size_t)last + 2 < PB_ANDROID_CAPTURE) {
        size_t start = g_capture_len;
        if (last >= 0) {
            for (i = 0; i <= last; i++) {
                g_capture_attr[start + (size_t)i] = g_attr[row][i];
            }
            memcpy(g_capture + g_capture_len, line, (size_t)last + 1);
            g_capture_len += (size_t)last + 1;
        }
        g_capture_attr[g_capture_len] = 0;
        g_capture[g_capture_len++] = '\n';
        g_capture[g_capture_len] = '\0';
    }
    __android_log_write(ANDROID_LOG_INFO, "PB64Basic", line);
}

void* GetStdHandle(unsigned int nStdHandle) {
    console_init();
    if (nStdHandle == (unsigned int)-10) return &g_handles[0]; /* stdin */
    if (nStdHandle == (unsigned int)-11) return &g_handles[1]; /* stdout */
    if (nStdHandle == (unsigned int)-12) return &g_handles[2]; /* stderr */
    return (void*)-1;
}

void* CreateFileA(const char* name, unsigned long access, unsigned long share,
                  void* sec, unsigned long disposition, unsigned long flags,
                  void* template_file) {
    (void)access; (void)share; (void)sec; (void)disposition;
    (void)flags; (void)template_file;
    console_init();
    if (name && strcmp(name, "CONOUT$") == 0) return &g_handles[1];
    if (name && strcmp(name, "CONIN$") == 0) return &g_handles[0];
    return (void*)-1;
}

int GetConsoleMode(void* h, PB_DWORD* mode) {
    if (!is_console_handle(h) || !mode) return 0;
    *mode = 0x0003; /* ENABLE_PROCESSED_OUTPUT | ENABLE_WRAP_AT_EOL_OUTPUT */
    return 1;
}

int SetConsoleTextAttribute(void* h, unsigned short attr) {
    if (!is_console_handle(h)) return 0;
    g_current_attr = attr;
    return 1;
}

int GetConsoleScreenBufferInfo(void* h, PB_CONSOLE_SCREEN_BUFFER_INFO* info) {
    if (!is_console_handle(h) || !info) return 0;
    console_init();
    info->dwSize.X = PB_ANDROID_COLS;
    info->dwSize.Y = PB_ANDROID_ROWS;
    info->dwCursorPosition = g_cursor;
    info->wAttributes = g_current_attr;
    info->srWindow.Left = 0;
    info->srWindow.Top = 0;
    info->srWindow.Right = PB_ANDROID_COLS - 1;
    info->srWindow.Bottom = PB_ANDROID_ROWS - 1;
    info->dwMaximumWindowSize.X = PB_ANDROID_COLS;
    info->dwMaximumWindowSize.Y = PB_ANDROID_ROWS;
    return 1;
}

int SetConsoleCursorPosition(void* h, PB_COORD pos) {
    int old_y;
    if (!is_console_handle(h)) return 0;
    console_init();
    if (pos.X < 0) pos.X = 0;
    if (pos.X >= PB_ANDROID_COLS) pos.X = PB_ANDROID_COLS - 1;
    if (pos.Y < 0) pos.Y = 0;
    if (pos.Y >= PB_ANDROID_ROWS) pos.Y = PB_ANDROID_ROWS - 1;
    old_y = g_cursor.Y;
    g_cursor = pos;
    /* A line is complete once the cursor moves below it.  Flush rows crossed
     * by this move, including the old row when moving down one line. */
    if (pos.Y > old_y) {
        int y;
        for (y = old_y; y < pos.Y; y++) flush_row(y);
    }
    return 1;
}

int WriteConsoleOutputCharacterA(void* h, const char* s, unsigned long len,
                                 PB_COORD pos, PB_DWORD* written) {
    unsigned long i;
    if (!is_console_handle(h) || !s) return 0;
    console_init();
    for (i = 0; i < len; i++) {
        int x = pos.X + (int)i;
        int y = pos.Y;
        if (x >= PB_ANDROID_COLS) { x -= PB_ANDROID_COLS; y++; }
        if (y < 0 || y >= PB_ANDROID_ROWS) break;
        if (x < 0) continue;
        g_text[y][x] = s[i];
        g_attr[y][x] = g_current_attr;
    }
    if (written) *written = i;
    return 1;
}

int WriteConsoleOutputAttribute(void* h, const unsigned short* attrs,
                                unsigned long len, PB_COORD pos,
                                PB_DWORD* written) {
    unsigned long i;
    if (!is_console_handle(h) || !attrs) return 0;
    console_init();
    for (i = 0; i < len; i++) {
        int x = pos.X + (int)i;
        int y = pos.Y;
        if (x >= PB_ANDROID_COLS) { x -= PB_ANDROID_COLS; y++; }
        if (y < 0 || y >= PB_ANDROID_ROWS) break;
        if (x < 0) continue;
        g_attr[y][x] = attrs[i];
    }
    if (written) *written = i;
    return 1;
}

int ReadConsoleOutputCharacterA(void* h, char* s, unsigned long len,
                                PB_COORD pos, PB_DWORD* read) {
    unsigned long i;
    if (!is_console_handle(h) || !s) return 0;
    console_init();
    for (i = 0; i < len; i++) {
        int x = pos.X + (int)i;
        int y = pos.Y;
        if (x >= PB_ANDROID_COLS) { x -= PB_ANDROID_COLS; y++; }
        if (y < 0 || y >= PB_ANDROID_ROWS) break;
        s[i] = (x < 0) ? ' ' : g_text[y][x];
    }
    if (read) *read = i;
    return 1;
}

int ReadConsoleOutputAttribute(void* h, unsigned short* attrs,
                               unsigned long len, PB_COORD pos,
                               PB_DWORD* read) {
    unsigned long i;
    if (!is_console_handle(h) || !attrs) return 0;
    console_init();
    for (i = 0; i < len; i++) {
        int x = pos.X + (int)i;
        int y = pos.Y;
        if (x >= PB_ANDROID_COLS) { x -= PB_ANDROID_COLS; y++; }
        if (y < 0 || y >= PB_ANDROID_ROWS) break;
        attrs[i] = (x < 0) ? g_current_attr : g_attr[y][x];
    }
    if (read) *read = i;
    return 1;
}

int FillConsoleOutputCharacterA(void* h, char ch, unsigned long len,
                                PB_COORD pos, PB_DWORD* written) {
    unsigned long i;
    if (!is_console_handle(h)) return 0;
    console_init();
    for (i = 0; i < len; i++) {
        int x = pos.X + (int)i;
        int y = pos.Y;
        if (x >= PB_ANDROID_COLS) { x -= PB_ANDROID_COLS; y++; }
        if (y < 0 || y >= PB_ANDROID_ROWS) break;
        if (x < 0) continue;
        g_text[y][x] = ch;
    }
    if (written) *written = i;
    return 1;
}

int FillConsoleOutputAttribute(void* h, unsigned short attr, unsigned long len,
                               PB_COORD pos, PB_DWORD* written) {
    unsigned long i;
    if (!is_console_handle(h)) return 0;
    console_init();
    for (i = 0; i < len; i++) {
        int x = pos.X + (int)i;
        int y = pos.Y;
        if (x >= PB_ANDROID_COLS) { x -= PB_ANDROID_COLS; y++; }
        if (y < 0 || y >= PB_ANDROID_ROWS) break;
        if (x < 0) continue;
        g_attr[y][x] = attr;
    }
    if (written) *written = i;
    return 1;
}

unsigned long long GetTickCount64(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (unsigned long long)ts.tv_sec * 1000ULL +
           (unsigned long long)(ts.tv_nsec / 1000000L);
}

const char* pb_android_console_text(void) {
    g_capture[g_capture_len] = '\0';
    return g_capture;
}

size_t pb_android_console_attr_len(void) {
    return g_capture_len;
}

const unsigned short* pb_android_console_attrs(void) {
    return g_capture_attr;
}

/* Win32 ABI types are 32-bit even under Android LP64. */

/* ==================== extended console / input surface ====================
 * The runtime declares the Win32 console API even on Android; functions that
 * are reachable from CON.*, INPUT and WAITKEY$ must exist at link time.  These
 * implementations share the same 80x25 buffer as PRINT/CLS/LOCATE.
 */
typedef struct { PB_DWORD dwSize; int bVisible; } PB_CONSOLE_CURSOR_INFO;
typedef struct { unsigned short Char; unsigned short Attributes; } PB_CHAR_INFO;
typedef struct {
    int bKeyDown;
    unsigned short wRepeatCount;
    unsigned short wVirtualKeyCode;
    unsigned short wVirtualScanCode;
    union { char AsciiChar; short UnicodeChar; } uChar;
    PB_DWORD dwControlKeyState;
} PB_KEY_EVENT_RECORD;
typedef struct {
    PB_COORD dwMousePosition;
    PB_DWORD dwButtonState;
    PB_DWORD dwControlKeyState;
    PB_DWORD dwEventFlags;
} PB_MOUSE_EVENT_RECORD;
typedef struct {
    unsigned short EventType;
    unsigned short _pad;
    union {
        PB_KEY_EVENT_RECORD KeyEvent;
        PB_MOUSE_EVENT_RECORD MouseEvent;
        char _raw[16];
    } Event;
} PB_INPUT_RECORD;
typedef struct { PB_DWORD nFont; PB_COORD dwFontSize; } PB_CONSOLE_FONT_INFO;
typedef struct { int left; int top; int right; int bottom; } pb_rect_t;
typedef struct { int x; int y; } PB_SCREEN_POINT;

extern int pb_android_input_count(void);
extern int pb_android_input_take(void);
extern int pb_android_input_peek(int index);
extern int pb_android_read_line(char* out, int max);
extern void pb_android_input_clear(void);

static char g_console_title[256] = "PB64Basic Android Console";
static int g_cursor_visible = 1;
static unsigned long g_cursor_size = 25;

static void console_scroll_up(void) {
    int r, c;
    for (r = 0; r < PB_ANDROID_ROWS - 1; r++) {
        for (c = 0; c < PB_ANDROID_COLS; c++) {
            g_text[r][c] = g_text[r + 1][c];
            g_attr[r][c] = g_attr[r + 1][c];
        }
    }
    for (c = 0; c < PB_ANDROID_COLS; c++) {
        g_text[PB_ANDROID_ROWS - 1][c] = ' ';
        g_attr[PB_ANDROID_ROWS - 1][c] = g_current_attr;
    }
}

static void console_put_byte(char ch) {
    if (ch == '\r') {
        g_cursor.X = 0;
        return;
    }
    if (ch == '\n') {
        flush_row(g_cursor.Y);
        g_cursor.X = 0;
        g_cursor.Y++;
        if (g_cursor.Y >= PB_ANDROID_ROWS) {
            console_scroll_up();
            g_cursor.Y = PB_ANDROID_ROWS - 1;
        }
        return;
    }
    if (g_cursor.X >= PB_ANDROID_COLS) {
        flush_row(g_cursor.Y);
        g_cursor.X = 0;
        g_cursor.Y++;
        if (g_cursor.Y >= PB_ANDROID_ROWS) {
            console_scroll_up();
            g_cursor.Y = PB_ANDROID_ROWS - 1;
        }
    }
    g_text[g_cursor.Y][g_cursor.X] = ch;
    g_attr[g_cursor.Y][g_cursor.X] = g_current_attr;
    g_cursor.X++;
}

int WriteConsoleA(void* h, const void* buf, unsigned long len,
                  PB_DWORD* written, void* reserved) {
    const char* s = (const char*)buf;
    unsigned long i;
    (void)reserved;
    if (!is_console_handle(h) || !s) return 0;
    for (i = 0; i < len; i++) console_put_byte(s[i]);
    if (written) *written = len;
    return 1;
}

int ReadConsoleA(void* h, void* buf, unsigned long n, PB_DWORD* got,
                 void* control) {
    int nread;
    (void)control;
    if (!is_console_handle(h) || !buf || n == 0) return 0;
    nread = pb_android_read_line((char*)buf, (int)n);
    /* Echo the accepted line, matching a Windows console. */
    {
        int i;
        for (i = 0; i < nread; i++) console_put_byte(((char*)buf)[i]);
        console_put_byte('\n');
    }
    if (got) *got = (unsigned long)nread;
    return 1;
}

int GetNumberOfConsoleInputEvents(void* h, PB_DWORD* count) {
    if (!is_console_handle(h) || !count) return 0;
    *count = (unsigned long)pb_android_input_count();
    return 1;
}

static void fill_key_event(PB_INPUT_RECORD* ir, int ch) {
    memset(ir, 0, sizeof(*ir));
    ir->EventType = 0x0001; /* KEY_EVENT */
    ir->Event.KeyEvent.bKeyDown = 1;
    ir->Event.KeyEvent.wRepeatCount = 1;
    ir->Event.KeyEvent.wVirtualKeyCode = 0;
    ir->Event.KeyEvent.wVirtualScanCode = 0;
    ir->Event.KeyEvent.uChar.AsciiChar = (char)ch;
    ir->Event.KeyEvent.dwControlKeyState = 0;
}

int ReadConsoleInputA(void* h, PB_INPUT_RECORD* buf, unsigned long len,
                      PB_DWORD* got) {
    unsigned long i;
    if (!is_console_handle(h) || !buf || !got) return 0;
    for (i = 0; i < len; i++) {
        int ch = pb_android_input_take();
        if (ch < 0) break;
        fill_key_event(&buf[i], ch);
    }
    *got = i;
    return 1;
}

int PeekConsoleInputA(void* h, PB_INPUT_RECORD* buf, unsigned long len,
                      PB_DWORD* got) {
    unsigned long i;
    if (!is_console_handle(h) || !buf || !got) return 0;
    for (i = 0; i < len; i++) {
        int ch = pb_android_input_peek((int)i);
        if (ch < 0) break;
        fill_key_event(&buf[i], ch);
    }
    *got = i;
    return 1;
}

int FlushConsoleInputBuffer(void* h) {
    if (!is_console_handle(h)) return 0;
    pb_android_input_clear();
    return 1;
}

int GetConsoleMode(void* h, PB_DWORD* mode);
int SetConsoleMode(void* h, unsigned long mode) {
    (void)mode;
    return is_console_handle(h) ? 1 : 0;
}

int GetFileType(void* h) {
    return is_console_handle(h) ? 2 /* FILE_TYPE_CHAR */ : 0;
}

int SetConsoleTitleA(const char* title) {
    size_t n;
    if (!title) return 0;
    n = strlen(title);
    if (n >= sizeof(g_console_title)) n = sizeof(g_console_title) - 1;
    memcpy(g_console_title, title, n);
    g_console_title[n] = '\0';
    return 1;
}

unsigned long GetConsoleTitleA(char* buf, unsigned long size) {
    size_t n = strlen(g_console_title);
    if (!buf || size == 0) return 0;
    if (n >= size) n = size - 1;
    memcpy(buf, g_console_title, n);
    buf[n] = '\0';
    return (unsigned long)n;
}

void* GetConsoleWindow(void) { return (void*)1; }
int AllocConsole(void) { return 1; }
int FreeConsole(void) { return 1; }

void* CreateConsoleScreenBuffer(unsigned long access, unsigned long share,
                                const void* sec, unsigned long flags,
                                void* data) {
    (void)access; (void)share; (void)sec; (void)flags; (void)data;
    console_init();
    return &g_handles[1];
}

int SetConsoleActiveScreenBuffer(void* h) {
    return is_console_handle(h) ? 1 : 0;
}

int SetConsoleScreenBufferSize(void* h, PB_COORD size) {
    (void)size;
    return is_console_handle(h) ? 1 : 0;
}

int SetConsoleWindowInfo(void* h, int absolute, const PB_SMALL_RECT* rect) {
    (void)absolute; (void)rect;
    return is_console_handle(h) ? 1 : 0;
}

PB_COORD GetLargestConsoleWindowSize(void* h) {
    PB_COORD size = {PB_ANDROID_COLS, PB_ANDROID_ROWS};
    (void)h;
    return size;
}

int GetConsoleCursorInfo(void* h, PB_CONSOLE_CURSOR_INFO* info) {
    if (!is_console_handle(h) || !info) return 0;
    info->dwSize = g_cursor_size;
    info->bVisible = g_cursor_visible;
    return 1;
}

int SetConsoleCursorInfo(void* h, const PB_CONSOLE_CURSOR_INFO* info) {
    if (!is_console_handle(h) || !info) return 0;
    g_cursor_size = info->dwSize;
    g_cursor_visible = info->bVisible;
    return 1;
}

int GetCurrentConsoleFont(void* h, int maximum, PB_CONSOLE_FONT_INFO* info) {
    (void)maximum;
    if (!is_console_handle(h) || !info) return 0;
    info->nFont = 0;
    info->dwFontSize.X = 8;
    info->dwFontSize.Y = 16;
    return 1;
}

int ScrollConsoleScreenBufferA(void* h, const PB_SMALL_RECT* scroll,
                               const PB_SMALL_RECT* clip, PB_COORD dest,
                               const PB_CHAR_INFO* fill) {
    int top, left, bottom, right, r, c;
    (void)clip;
    if (!is_console_handle(h) || !scroll) return 0;
    left = scroll->Left; top = scroll->Top;
    right = scroll->Right; bottom = scroll->Bottom;
    if (left < 0) left = 0;
    if (top < 0) top = 0;
    if (right >= PB_ANDROID_COLS) right = PB_ANDROID_COLS - 1;
    if (bottom >= PB_ANDROID_ROWS) bottom = PB_ANDROID_ROWS - 1;
    for (r = top; r <= bottom; r++) {
        int sr = r + (dest.Y - top);
        if (sr < 0 || sr >= PB_ANDROID_ROWS) continue;
        for (c = left; c <= right; c++) {
            int sc = c + (dest.X - left);
            if (sc < 0 || sc >= PB_ANDROID_COLS) continue;
            g_text[sr][sc] = g_text[r][c];
            g_attr[sr][sc] = g_attr[r][c];
        }
    }
    if (fill) {
        for (r = top; r <= bottom; r++) {
            for (c = left; c <= right; c++) {
                g_text[r][c] = fill->Char;
                g_attr[r][c] = fill->Attributes;
            }
        }
    }
    return 1;
}

int GetSystemMetrics(int index) {
    if (index == 0) return 1080;  /* SM_CXSCREEN */
    if (index == 1) return 1920;  /* SM_CYSCREEN */
    return 0;
}

int GetWindowRect(void* h, pb_rect_t* rect) {
    (void)h;
    if (!rect) return 0;
    rect->left = 0; rect->top = 0; rect->right = 1080; rect->bottom = 1920;
    return 1;
}

int GetClientRect(void* h, pb_rect_t* rect) {
    (void)h;
    if (!rect) return 0;
    rect->left = 0; rect->top = 0; rect->right = 1080; rect->bottom = 1920;
    return 1;
}

int ClientToScreen(void* h, PB_SCREEN_POINT* pt) {
    (void)h; (void)pt;
    return 1;
}

int SetWindowPos(void* h, void* after, int x, int y, int cx, int cy,
                 unsigned int flags) {
    (void)h; (void)after; (void)x; (void)y; (void)cx; (void)cy; (void)flags;
    return 1;
}

int SetForegroundWindow(void* h) { (void)h; return 1; }
int ShowWindow(void* h, int cmd) { (void)h; (void)cmd; return 1; }

