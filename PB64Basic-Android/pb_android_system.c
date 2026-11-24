/* Android kernel32 system-information compatibility layer.
 *
 * Covers the non-GUI Win32 calls used by FRE(), COMMAND$, OBJRESULT$,
 * THREAD PRIORITY and END.  Values are real where Android exposes them and
 * explicitly synthetic where the concept does not exist.
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

typedef unsigned int PB_DWORD;

typedef struct {
    PB_DWORD dwLength;
    PB_DWORD dwMemoryLoad;
    unsigned long long ullTotalPhys;
    unsigned long long ullAvailPhys;
    unsigned long long ullTotalPageFile;
    unsigned long long ullAvailPageFile;
    unsigned long long ullTotalVirtual;
    unsigned long long ullAvailVirtual;
    unsigned long long ullAvailExtendedVirtual;
} PB_MEMORYSTATUSEX;

typedef struct {
    unsigned int dwLowDateTime;
    unsigned int dwHighDateTime;
} PB_FILETIME;

static unsigned long long read_meminfo_kb(const char* key) {
    FILE* f = fopen("/proc/meminfo", "r");
    char line[256];
    unsigned long long value = 0;
    if (!f) return 0;
    while (fgets(line, sizeof(line), f)) {
        size_t n = strlen(key);
        if (strncmp(line, key, n) == 0 && line[n] == ':') {
            sscanf(line + n + 1, " %llu", &value);
            break;
        }
    }
    fclose(f);
    return value;
}

int GlobalMemoryStatusEx(PB_MEMORYSTATUSEX* lpBuffer) {
    unsigned long long total_kb;
    unsigned long long avail_kb;
    if (!lpBuffer) return 0;
    total_kb = read_meminfo_kb("MemTotal");
    avail_kb = read_meminfo_kb("MemAvailable");
    if (avail_kb == 0) avail_kb = read_meminfo_kb("MemFree");
    if (total_kb == 0) return 0;
    memset(lpBuffer, 0, sizeof(*lpBuffer));
    lpBuffer->dwLength = (PB_DWORD)sizeof(*lpBuffer);
    lpBuffer->dwMemoryLoad = (PB_DWORD)((total_kb - avail_kb) * 100 / total_kb);
    lpBuffer->ullTotalPhys = total_kb * 1024ULL;
    lpBuffer->ullAvailPhys = avail_kb * 1024ULL;
    lpBuffer->ullTotalPageFile = total_kb * 1024ULL;
    lpBuffer->ullAvailPageFile = avail_kb * 1024ULL;
    lpBuffer->ullTotalVirtual = total_kb * 1024ULL;
    lpBuffer->ullAvailVirtual = avail_kb * 1024ULL;
    return 1;
}

void* GetCurrentProcess(void) { return (void*)(long)-1; }

unsigned long GetPriorityClass(void* hProcess) {
    (void)hProcess;
    return 0x20UL; /* NORMAL_PRIORITY_CLASS */
}

int SetPriorityClass(void* hProcess, unsigned long dwPriorityClass) {
    (void)hProcess;
    (void)dwPriorityClass;
    return 1;
}

int GetThreadPriority(void* hThread) {
    (void)hThread;
    return 0; /* THREAD_PRIORITY_NORMAL */
}

int GetThreadTimes(void* hThread, PB_FILETIME* creation, PB_FILETIME* exit_time,
                   PB_FILETIME* kernel_time, PB_FILETIME* user_time) {
    (void)hThread;
    if (creation) memset(creation, 0, sizeof(*creation));
    if (exit_time) memset(exit_time, 0, sizeof(*exit_time));
    if (kernel_time) memset(kernel_time, 0, sizeof(*kernel_time));
    if (user_time) memset(user_time, 0, sizeof(*user_time));
    return 1;
}

void GetSystemTimeAsFileTime(PB_FILETIME* ft) {
    struct timespec ts;
    unsigned long long ticks;
    if (!ft) return;
    if (clock_gettime(CLOCK_REALTIME, &ts) != 0) {
        ft->dwLowDateTime = 0;
        ft->dwHighDateTime = 0;
        return;
    }
    ticks = ((unsigned long long)ts.tv_sec + 11644473600ULL) * 10000000ULL +
            (unsigned long long)ts.tv_nsec / 100ULL;
    ft->dwLowDateTime = (unsigned int)(ticks & 0xFFFFFFFFULL);
    ft->dwHighDateTime = (unsigned int)(ticks >> 32);
}

char* GetCommandLineA(void) {
    static char cmdline[64] = "pb64app";
    return cmdline;
}

void* GetModuleHandleA(const char* lpModuleName) {
    (void)lpModuleName;
    return NULL;
}

unsigned long FormatMessageA(unsigned long dwFlags, const void* lpSource,
                             unsigned long dwMessageId, unsigned long dwLanguageId,
                             char* lpBuffer, unsigned long nSize,
                             const void* Arguments) {
    const char* text;
    (void)dwFlags;
    (void)lpSource;
    (void)dwLanguageId;
    (void)Arguments;
    if (!lpBuffer || nSize == 0) return 0;
    if (dwMessageId == 0)
        text = "The operation completed successfully.";
    else {
        static __thread char fallback[96];
        snprintf(fallback, sizeof(fallback),
                 "PB64Basic Android error %lu", dwMessageId);
        text = fallback;
    }
    snprintf(lpBuffer, nSize, "%s", text);
    return (unsigned long)strlen(text);
}

void* LocalFree(void* hMem) {
    free(hMem);
    return NULL;
}

void ExitProcess(unsigned int uExitCode) {
    exit((int)uExitCode);
}
