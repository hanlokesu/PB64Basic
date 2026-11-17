#include <windows.h>
#include <stdio.h>

static FILE* g_log;

static void show(const char* tag, HANDLE h) {
    CONSOLE_SCREEN_BUFFER_INFO i;
    if (!GetConsoleScreenBufferInfo(h, &i)) {
        fprintf(g_log, "%s: GetConsoleScreenBufferInfo failed %lu\n", tag, GetLastError());
        return;
    }
    fprintf(g_log, "%s: size=%d,%d win=%d,%d..%d,%d max=%d,%d\n", tag,
           (int)i.dwSize.X, (int)i.dwSize.Y,
           (int)i.srWindow.Left, (int)i.srWindow.Top,
           (int)i.srWindow.Right, (int)i.srWindow.Bottom,
           (int)i.dwMaximumWindowSize.X, (int)i.dwMaximumWindowSize.Y);
}

int main(void) {
    FILE* log = fopen("_diag_conview.txt", "w");
    HANDLE h = CreateFileA("CONOUT$", GENERIC_READ | GENERIC_WRITE,
                           FILE_SHARE_READ | FILE_SHARE_WRITE, NULL,
                           OPEN_EXISTING, 0, NULL);
    if (!log) return 1;
    g_log = log;
    if (h == INVALID_HANDLE_VALUE) {
        fprintf(g_log, "CONOUT$ open failed %lu\n", GetLastError());
        fclose(log);
        return 1;
    }
    show("start", h);

    SMALL_RECT one = {0, 0, 0, 0};
    fprintf(g_log, "shrink=%d err=%lu\n", SetConsoleWindowInfo(h, TRUE, &one), GetLastError());
    COORD sz = {120, 40};
    fprintf(g_log, "buffer=%d err=%lu\n", SetConsoleScreenBufferSize(h, sz), GetLastError());
    show("after buffer", h);
    SMALL_RECT ten = {0, 0, 79, 9};
    fprintf(g_log, "window10=%d err=%lu\n", SetConsoleWindowInfo(h, TRUE, &ten), GetLastError());
    show("after setup", h);

    SMALL_RECT abs5 = {6, 4, 85, 13};
    fprintf(g_log, "abs=%d err=%lu\n", SetConsoleWindowInfo(h, TRUE, &abs5), GetLastError());
    show("after abs", h);

    SMALL_RECT rel2 = {0, 0, 79, 9};
    fprintf(g_log, "rel=%d err=%lu\n", SetConsoleWindowInfo(h, FALSE, &rel2), GetLastError());
    show("after rel", h);

    SMALL_RECT abs10 = {10, 10, 89, 19};
    fprintf(g_log, "abs10=%d err=%lu\n", SetConsoleWindowInfo(h, TRUE, &abs10), GetLastError());
    show("after abs10", h);
    fclose(log);
    CloseHandle(h);
    return 0;
}
