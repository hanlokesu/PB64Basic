/* Android Win32 file/directory compatibility layer for PB64Basic.
 *
 * The runtime is written against the Win32 ANSI file API.  Android has no
 * Win32, so this file maps the subset reachable from DIR$, KILL, FILECOPY,
 * NAME, SETATTR, CHDIR, CURDIR$, TMP$ and sequential file I/O onto bionic
 * POSIX calls.  WIN32_FIND_DATAA is filled byte-for-byte because the runtime
 * reads cFileName at offset 44 and dwFileAttributes at offset 0.
 */
#include <ctype.h>
#include <dirent.h>
#include <errno.h>
#include <fcntl.h>
#include <fnmatch.h>
#include <limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/statvfs.h>
#include <sys/types.h>
#include <time.h>
#include <unistd.h>

#define PB_FIND_DATA_SIZE 592

/* Win32 DWORD/LONG stay 32-bit under Android LP64. */
typedef unsigned int PB_DWORD;
#define PB_FILE_ATTRIBUTE_READONLY  0x01UL
#define PB_FILE_ATTRIBUTE_HIDDEN    0x02UL
#define PB_FILE_ATTRIBUTE_SYSTEM    0x04UL
#define PB_FILE_ATTRIBUTE_DIRECTORY 0x10UL
#define PB_FILE_ATTRIBUTE_ARCHIVE   0x20UL
#define PB_FILE_ATTRIBUTE_NORMAL    0x80UL
#define PB_INVALID_FILE_ATTRIBUTES  0xFFFFFFFFUL
#define PB_INVALID_HANDLE_VALUE     ((void*)(long)-1)

#define PB_ERROR_SUCCESS            0UL
#define PB_ERROR_FILE_NOT_FOUND     2UL
#define PB_ERROR_PATH_NOT_FOUND     3UL
#define PB_ERROR_ACCESS_DENIED      5UL
#define PB_ERROR_INVALID_HANDLE     6UL
#define PB_ERROR_NOT_ENOUGH_MEMORY  8UL
#define PB_ERROR_FILE_EXISTS        80UL
#define PB_ERROR_ALREADY_EXISTS     183UL
#define PB_ERROR_DISK_FULL          112UL
#define PB_ERROR_WRITE_PROTECT      19UL
#define PB_ERROR_SHARING_VIOLATION  32UL
#define PB_ERROR_DIR_NOT_EMPTY      145UL
#define PB_ERROR_INVALID_NAME       123UL

static __thread unsigned long g_last_error = PB_ERROR_SUCCESS;

/* PB sources routinely use Win32-style "dir\file" paths even on Android.
 * Translate separators before handing the string to bionic.  Returns the
 * original pointer when no translation is needed. */
static const char* norm_path(const char* in, char* buf, size_t cap) {
    size_t i;
    const char* src = in;
    int changed = 0;
    if (!in) return in;
    /* Map a Win32 drive prefix onto the Android root: C:\dir -> /dir. */
    if (in[0] && in[1] == ':') {
        src = in + 2;
        changed = 1;
    }
    if (!changed && strchr(src, '\\') == NULL) return src;
    for (i = 0; src[i] && i + 1 < cap; ++i) {
        buf[i] = (src[i] == '\\') ? '/' : src[i];
        if (src[i] == '\\') changed = 1;
    }
    buf[i] = '\0';
    return changed ? buf : src;
}

unsigned long GetLastError(void) { return g_last_error; }
void SetLastError(unsigned long code) { g_last_error = code; }

static unsigned long map_errno(int e) {
    switch (e) {
    case 0: return PB_ERROR_SUCCESS;
    case ENOENT: return PB_ERROR_FILE_NOT_FOUND;
    case ENOTDIR: return PB_ERROR_PATH_NOT_FOUND;
    case EACCES:
    case EPERM:
    case EROFS: return PB_ERROR_ACCESS_DENIED;
    case EEXIST: return PB_ERROR_FILE_EXISTS;
    case ENOSPC: return PB_ERROR_DISK_FULL;
    case ENAMETOOLONG: return PB_ERROR_INVALID_NAME;
    case ENOTEMPTY: return PB_ERROR_DIR_NOT_EMPTY;
    case EBUSY: return PB_ERROR_SHARING_VIOLATION;
    default: return (unsigned long)e;
    }
}

static void set_errno_error(void) { g_last_error = map_errno(errno); }

static void filetime_from_time(long long seconds, char* out) {
    /* 100-ns ticks since 1601-01-01. */
    unsigned long long ticks = (unsigned long long)(seconds + 11644473600LL) *
                               10000000ULL;
    memcpy(out, &ticks, 8);
}

static void fill_find_data(char* data, const char* name, const struct stat* st) {
    unsigned long attrs;
    if (!data) return;
    memset(data, 0, PB_FIND_DATA_SIZE);
    attrs = (st && S_ISDIR(st->st_mode)) ? PB_FILE_ATTRIBUTE_DIRECTORY
                                         : PB_FILE_ATTRIBUTE_ARCHIVE;
    if (name && name[0] == '.' && name[1] && strcmp(name, ".") &&
        strcmp(name, "..")) {
        attrs |= PB_FILE_ATTRIBUTE_HIDDEN;
    }
    memcpy(data + 0, &attrs, 4);
    if (st) {
        filetime_from_time((long long)st->st_ctime, data + 4);
        filetime_from_time((long long)st->st_atime, data + 12);
        filetime_from_time((long long)st->st_mtime, data + 20);
        {
            unsigned long hi = (unsigned long)(((unsigned long long)st->st_size) >> 32);
            unsigned long lo = (unsigned long)(((unsigned long long)st->st_size) & 0xFFFFFFFFULL);
            memcpy(data + 28, &hi, 4);
            memcpy(data + 32, &lo, 4);
        }
    }
    if (name) {
        size_t n = strlen(name);
        if (n > 259) n = 259;
        memcpy(data + 44, name, n);
        data[44 + n] = '\0';
    }
}

typedef struct {
    DIR* dir;
    char pattern[PATH_MAX];
    char dirpart[PATH_MAX];
    char single_name[260];
    int single_only;
    int single_done;
    int valid;
} PB_FIND_HANDLE;

static void split_pattern(const char* path, char* dirpart, size_t dcap,
                          char* leaf, size_t lcap) {
    const char* slash = NULL;
    const char* p;
    for (p = path; p && *p; ++p) {
        if (*p == '/' || *p == '\\') slash = p;
    }
    if (slash) {
        size_t n = (size_t)(slash - path) + 1;
        if (n >= dcap) n = dcap - 1;
        if (n == 0) {
            strcpy(dirpart, ".");
        } else {
            memcpy(dirpart, path, n);
            dirpart[n] = '\0';
        }
        snprintf(leaf, lcap, "%s", slash + 1);
    } else {
        strcpy(dirpart, ".");
        snprintf(leaf, lcap, "%s", path);
    }
}

static int has_wildcard(const char* s) {
    return s && (strchr(s, '*') || strchr(s, '?'));
}

/* Win32 "*.*" means "any name", including names without a dot; fnmatch()
 * would require a literal dot, so normalize that one pattern. */
static const char* pb_fnmatch_pattern(const char* pattern) {
    return (pattern && strcmp(pattern, "*.*") == 0) ? "*" : pattern;
}

void* FindFirstFileA(const char* lpFileName, char* lpFindFileData) {
    PB_FIND_HANDLE* h;
    char leaf[PATH_MAX];
    char pbuf[PATH_MAX];
    const char* path;
    struct stat st;

    if (!lpFileName || !*lpFileName || !lpFindFileData) {
        g_last_error = PB_ERROR_PATH_NOT_FOUND;
        return PB_INVALID_HANDLE_VALUE;
    }
    path = norm_path(lpFileName, pbuf, sizeof(pbuf));
    h = (PB_FIND_HANDLE*)calloc(1, sizeof(*h));
    if (!h) {
        g_last_error = PB_ERROR_NOT_ENOUGH_MEMORY;
        return PB_INVALID_HANDLE_VALUE;
    }
    split_pattern(path, h->dirpart, sizeof(h->dirpart), leaf, sizeof(leaf));

    if (!has_wildcard(leaf)) {
        if (stat(path, &st) != 0) {
            set_errno_error();
            free(h);
            return PB_INVALID_HANDLE_VALUE;
        }
        snprintf(h->single_name, sizeof(h->single_name), "%s", leaf);
        h->single_only = 1;
        h->valid = 1;
        fill_find_data(lpFindFileData, leaf, &st);
        return h;
    }

    h->dir = opendir(h->dirpart);
    if (!h->dir) {
        set_errno_error();
        free(h);
        return PB_INVALID_HANDLE_VALUE;
    }
    snprintf(h->pattern, sizeof(h->pattern), "%s", leaf);
    h->valid = 1;

    for (;;) {
        struct dirent* de = readdir(h->dir);
        char full[PATH_MAX];
        if (!de) {
            closedir(h->dir);
            free(h);
            g_last_error = PB_ERROR_FILE_NOT_FOUND;
            return PB_INVALID_HANDLE_VALUE;
        }
        if (strcmp(de->d_name, ".") == 0 || strcmp(de->d_name, "..") == 0)
            continue;
        if (fnmatch(pb_fnmatch_pattern(h->pattern), de->d_name, 0) != 0) continue;
        snprintf(full, sizeof(full), "%s/%s", h->dirpart, de->d_name);
        if (stat(full, &st) != 0) continue;
        fill_find_data(lpFindFileData, de->d_name, &st);
        return h;
    }
}

int FindNextFileA(void* hFindFile, char* lpFindFileData) {
    PB_FIND_HANDLE* h = (PB_FIND_HANDLE*)hFindFile;
    if (!h || !h->valid || !lpFindFileData) {
        g_last_error = PB_ERROR_INVALID_HANDLE;
        return 0;
    }
    if (h->single_only) {
        if (h->single_done) {
            g_last_error = PB_ERROR_FILE_NOT_FOUND;
            return 0;
        }
        h->single_done = 1;
        {
            struct stat st;
            char full[PATH_MAX];
            snprintf(full, sizeof(full), "%s/%s", h->dirpart, h->single_name);
            if (stat(full, &st) != 0) {
                set_errno_error();
                return 0;
            }
            fill_find_data(lpFindFileData, h->single_name, &st);
        }
        return 1;
    }
    for (;;) {
        struct dirent* de = readdir(h->dir);
        char full[PATH_MAX];
        struct stat st;
        if (!de) {
            g_last_error = PB_ERROR_FILE_NOT_FOUND;
            return 0;
        }
        if (strcmp(de->d_name, ".") == 0 || strcmp(de->d_name, "..") == 0)
            continue;
        if (fnmatch(pb_fnmatch_pattern(h->pattern), de->d_name, 0) != 0) continue;
        snprintf(full, sizeof(full), "%s/%s", h->dirpart, de->d_name);
        if (stat(full, &st) != 0) continue;
        fill_find_data(lpFindFileData, de->d_name, &st);
        return 1;
    }
}

int FindClose(void* hFindFile) {
    PB_FIND_HANDLE* h = (PB_FIND_HANDLE*)hFindFile;
    if (!h) return 0;
    if (h->dir) closedir(h->dir);
    free(h);
    return 1;
}

unsigned long GetFileAttributesA(const char* lpFileName) {
    struct stat st;
    unsigned long attrs;
    char pbuf[PATH_MAX];
    const char* path = norm_path(lpFileName, pbuf, sizeof(pbuf));
    const char* base;
    if (!path || stat(path, &st) != 0) {
        set_errno_error();
        return PB_INVALID_FILE_ATTRIBUTES;
    }
    attrs = S_ISDIR(st.st_mode) ? PB_FILE_ATTRIBUTE_DIRECTORY
                                : PB_FILE_ATTRIBUTE_ARCHIVE;
    base = strrchr(path, '/');
    base = base ? base + 1 : path;
    if (base[0] == '.' && base[1] && strcmp(base, ".") && strcmp(base, ".."))
        attrs |= PB_FILE_ATTRIBUTE_HIDDEN;
    if (!(st.st_mode & S_IWUSR)) attrs |= PB_FILE_ATTRIBUTE_READONLY;
    return attrs;
}

int SetFileAttributesA(const char* lpFileName, unsigned long dwFileAttributes) {
    struct stat st;
    char pbuf[PATH_MAX];
    const char* path = norm_path(lpFileName, pbuf, sizeof(pbuf));
    mode_t mode;
    if (!path || stat(path, &st) != 0) {
        set_errno_error();
        return 0;
    }
    mode = st.st_mode;
    if (dwFileAttributes & PB_FILE_ATTRIBUTE_READONLY)
        mode &= (mode_t)~0222;
    else
        mode |= S_IWUSR;
    if (chmod(path, mode) != 0) {
        set_errno_error();
        return 0;
    }
    return 1;
}

int DeleteFileA(const char* lpFileName) {
    struct stat st;
    char pbuf[PATH_MAX];
    const char* path = norm_path(lpFileName, pbuf, sizeof(pbuf));
    if (!path) {
        g_last_error = PB_ERROR_INVALID_NAME;
        return 0;
    }
    if (stat(path, &st) == 0 && S_ISDIR(st.st_mode)) {
        g_last_error = PB_ERROR_ACCESS_DENIED;
        return 0;
    }
    if (unlink(path) != 0) {
        set_errno_error();
        return 0;
    }
    return 1;
}

static int copy_file_data(const char* src, const char* dst, int fail_if_exists) {
    FILE* in;
    FILE* out;
    char buf[8192];
    size_t n;
    struct stat sst;
    struct stat dst_st;
    if (stat(src, &sst) == 0 && stat(dst, &dst_st) == 0 &&
        sst.st_dev == dst_st.st_dev && sst.st_ino == dst_st.st_ino) {
        g_last_error = PB_ERROR_SHARING_VIOLATION;
        return 0;
    }
    in = fopen(src, "rb");
    if (!in) {
        set_errno_error();
        return 0;
    }
    if (fail_if_exists && access(dst, F_OK) == 0) {
        fclose(in);
        g_last_error = PB_ERROR_FILE_EXISTS;
        return 0;
    }
    out = fopen(dst, "wb");
    if (!out) {
        set_errno_error();
        fclose(in);
        return 0;
    }
    while ((n = fread(buf, 1, sizeof(buf), in)) > 0) {
        if (fwrite(buf, 1, n, out) != n) {
            set_errno_error();
            fclose(in);
            fclose(out);
            return 0;
        }
    }
    fclose(in);
    fclose(out);
    return 1;
}

int CopyFileA(const char* lpExistingFileName, const char* lpNewFileName,
              int bFailIfExists) {
    char srcbuf[PATH_MAX];
    char dstbuf[PATH_MAX];
    const char* src = norm_path(lpExistingFileName, srcbuf, sizeof(srcbuf));
    const char* dst = norm_path(lpNewFileName, dstbuf, sizeof(dstbuf));
    if (!src || !dst) {
        g_last_error = PB_ERROR_INVALID_NAME;
        return 0;
    }
    return copy_file_data(src, dst, bFailIfExists);
}

int MoveFileExA(const char* lpExisting, const char* lpNew, unsigned long flags) {
    #define PB_MOVEFILE_REPLACE_EXISTING 0x1UL
    #define PB_MOVEFILE_COPY_ALLOWED     0x2UL
    char srcbuf[PATH_MAX];
    char dstbuf[PATH_MAX];
    const char* src = norm_path(lpExisting, srcbuf, sizeof(srcbuf));
    const char* dst = norm_path(lpNew, dstbuf, sizeof(dstbuf));
    if (!src || !dst) {
        g_last_error = PB_ERROR_INVALID_NAME;
        return 0;
    }
    if (!(flags & PB_MOVEFILE_REPLACE_EXISTING) && access(dst, F_OK) == 0) {
        g_last_error = PB_ERROR_ALREADY_EXISTS;
        return 0;
    }
    if (rename(src, dst) == 0) return 1;
    if (errno == EXDEV && (flags & PB_MOVEFILE_COPY_ALLOWED)) {
        if (!copy_file_data(src, dst, 0)) return 0;
        if (unlink(src) != 0) {
            set_errno_error();
            return 0;
        }
        return 1;
    }
    set_errno_error();
    return 0;
}

int CreateDirectoryA(const char* lpPathName, void* lpSecurityAttributes) {
    char pbuf[PATH_MAX];
    const char* path = norm_path(lpPathName, pbuf, sizeof(pbuf));
    (void)lpSecurityAttributes;
    if (!path || mkdir(path, 0777) != 0) {
        set_errno_error();
        return 0;
    }
    return 1;
}

int RemoveDirectoryA(const char* lpPathName) {
    char pbuf[PATH_MAX];
    const char* path = norm_path(lpPathName, pbuf, sizeof(pbuf));
    if (!path || rmdir(path) != 0) {
        set_errno_error();
        return 0;
    }
    return 1;
}

int GetDiskFreeSpaceExA(char* lpDirectoryName, unsigned long long* lpFreeBytesAvailable,
                        unsigned long long* lpTotalNumberOfBytes,
                        unsigned long long* lpTotalNumberOfFreeBytes) {
    struct statvfs vfs;
    char pbuf[PATH_MAX];
    const char* path = norm_path((lpDirectoryName && *lpDirectoryName)
                                     ? lpDirectoryName : ".", pbuf, sizeof(pbuf));
    if (statvfs(path, &vfs) != 0) {
        set_errno_error();
        return 0;
    }
    if (lpFreeBytesAvailable)
        *lpFreeBytesAvailable = (unsigned long long)vfs.f_bavail * vfs.f_frsize;
    if (lpTotalNumberOfBytes)
        *lpTotalNumberOfBytes = (unsigned long long)vfs.f_blocks * vfs.f_frsize;
    if (lpTotalNumberOfFreeBytes)
        *lpTotalNumberOfFreeBytes = (unsigned long long)vfs.f_bfree * vfs.f_frsize;
    return 1;
}

unsigned long GetFileSize(const char* lpFileName, PB_DWORD* lpFileSizeHigh) {
    struct stat st;
    char pbuf[PATH_MAX];
    const char* path = norm_path(lpFileName, pbuf, sizeof(pbuf));
    if (lpFileSizeHigh) *lpFileSizeHigh = 0;
    if (!path || stat(path, &st) != 0) {
        set_errno_error();
        return PB_INVALID_FILE_ATTRIBUTES;
    }
    if (lpFileSizeHigh) *lpFileSizeHigh = (PB_DWORD)(((unsigned long long)st.st_size) >> 32);
    return (unsigned long)(((unsigned long long)st.st_size) & 0xFFFFFFFFULL);
}

int GetFileSizeEx(const char* lpFileName, long long* lpFileSize) {
    struct stat st;
    char pbuf[PATH_MAX];
    const char* path = norm_path(lpFileName, pbuf, sizeof(pbuf));
    if (!path || !lpFileSize || stat(path, &st) != 0) {
        set_errno_error();
        return 0;
    }
    *lpFileSize = (long long)st.st_size;
    return 1;
}

int GetFileTime(const char* lpFileName, void* lpCreationTime, void* lpLastAccessTime,
                void* lpLastWriteTime) {
    struct stat st;
    char pbuf[PATH_MAX];
    const char* path = norm_path(lpFileName, pbuf, sizeof(pbuf));
    if (!path || stat(path, &st) != 0) {
        set_errno_error();
        return 0;
    }
    if (lpCreationTime) filetime_from_time((long long)st.st_ctime, (char*)lpCreationTime);
    if (lpLastAccessTime) filetime_from_time((long long)st.st_atime, (char*)lpLastAccessTime);
    if (lpLastWriteTime) filetime_from_time((long long)st.st_mtime, (char*)lpLastWriteTime);
    return 1;
}

static void filetime_to_timespec(const void* value, struct timespec* out) {
    unsigned long long ticks;
    long long seconds;
    if (!value) {
        out->tv_sec = 0;
        out->tv_nsec = UTIME_OMIT;
        return;
    }
    memcpy(&ticks, value, 8);
    seconds = (long long)(ticks / 10000000ULL) - 11644473600LL;
    out->tv_sec = (time_t)seconds;
    out->tv_nsec = (long)((ticks % 10000000ULL) * 100ULL);
}

int SetFileTime(const char* lpFileName, const void* lpCreationTime,
                const void* lpLastAccessTime, const void* lpLastWriteTime) {
    struct timespec times[2];
    char pbuf[PATH_MAX];
    const char* path = norm_path(lpFileName, pbuf, sizeof(pbuf));
    (void)lpCreationTime; /* POSIX cannot set a creation time. */
    if (!path) {
        g_last_error = PB_ERROR_INVALID_NAME;
        return 0;
    }
    filetime_to_timespec(lpLastAccessTime, &times[0]);
    filetime_to_timespec(lpLastWriteTime, &times[1]);
    if (utimensat(AT_FDCWD, path, times, 0) != 0) {
        set_errno_error();
        return 0;
    }
    return 1;
}

unsigned long GetCurrentDirectoryA(unsigned long nBufferLength, char* lpBuffer) {
    char cwd[PATH_MAX];
    if (!getcwd(cwd, sizeof(cwd))) {
        set_errno_error();
        return 0;
    }
    if (lpBuffer && nBufferLength > 0) {
        size_t n = strlen(cwd);
        if (n + 1 > nBufferLength) {
            g_last_error = PB_ERROR_INVALID_NAME;
            return (unsigned long)(n + 1); /* Win32 required-size contract. */
        }
        memcpy(lpBuffer, cwd, n + 1);
        return (unsigned long)n;
    }
    return (unsigned long)(strlen(cwd) + 1);
}

int SetCurrentDirectoryA(const char* lpPathName) {
    char pbuf[PATH_MAX];
    const char* path = norm_path(lpPathName, pbuf, sizeof(pbuf));
    if (!path || chdir(path) != 0) {
        set_errno_error();
        return 0;
    }
    return 1;
}

unsigned long GetFullPathNameA(const char* lpFileName, unsigned long nBufferLength,
                               char* lpBuffer, char** lpFilePart) {
    char resolved[PATH_MAX];
    char pbuf[PATH_MAX];
    const char* path;
    size_t n;
    if (lpFilePart) *lpFilePart = NULL;
    path = norm_path(lpFileName, pbuf, sizeof(pbuf));
    if (!path) {
        g_last_error = PB_ERROR_INVALID_NAME;
        return 0;
    }
    if (realpath(path, resolved) == NULL) {
        /* realpath fails for a not-yet-existing output file; return the
           absolute form based on cwd instead of failing the whole call. */
        char cwd[PATH_MAX];
        if (path[0] == '/') {
            snprintf(resolved, sizeof(resolved), "%s", path);
        } else if (getcwd(cwd, sizeof(cwd))) {
            snprintf(resolved, sizeof(resolved), "%s/%s", cwd, path);
        } else {
            snprintf(resolved, sizeof(resolved), "%s", path);
        }
    }
    n = strlen(resolved);
    if (lpFilePart && lpBuffer) {
        const char* slash = strrchr(resolved, '/');
        if (slash) *lpFilePart = lpBuffer + (slash - resolved) + 1;
    }
    if (lpBuffer && nBufferLength > 0) {
        if (n + 1 > nBufferLength) {
            g_last_error = PB_ERROR_INVALID_NAME;
            return (unsigned long)(n + 1); /* Win32 required-size contract. */
        }
        memcpy(lpBuffer, resolved, n + 1);
    }
    return (unsigned long)n;
}

unsigned long GetTempPathA(unsigned long nBufferLength, char* lpBuffer) {
    const char* tmp = getenv("TMPDIR");
    size_t n;
    size_t needed;
    if (!tmp || !*tmp) tmp = ".";
    n = strlen(tmp);
    needed = n + (tmp[n - 1] == '/' ? 1 : 2); /* trailing slash + NUL */
    if (lpBuffer && nBufferLength > 0) {
        if (needed > nBufferLength) {
            g_last_error = PB_ERROR_INVALID_NAME;
            return (unsigned long)needed;
        }
        memcpy(lpBuffer, tmp, n);
        if (tmp[n - 1] != '/') lpBuffer[n++] = '/';
        lpBuffer[n] = '\0';
        return (unsigned long)n;
    }
    return (unsigned long)needed;
}

unsigned long GetModuleFileNameA(void* hModule, char* lpFilename,
                                 unsigned long nSize) {
    (void)hModule;
    if (!lpFilename || nSize == 0) return 0;
    snprintf(lpFilename, nSize, "%s", "/proc/self/exe");
    return (unsigned long)strlen(lpFilename);
}

unsigned long GetLogicalDrives(void) { return 1UL; }

unsigned long GetDriveTypeA(const char* lpRootPathName) {
    (void)lpRootPathName;
    return 3UL; /* DRIVE_FIXED */
}

int GetVolumeInformationA(const char* lpRootPathName, char* lpVolumeNameBuffer,
                          unsigned long nVolumeNameSize, PB_DWORD* lpVolumeSerialNumber,
                          PB_DWORD* lpMaximumComponentLength,
                          PB_DWORD* lpFileSystemFlags, char* lpFileSystemNameBuffer,
                          unsigned long nFileSystemNameSize) {
    (void)lpRootPathName;
    if (lpVolumeNameBuffer && nVolumeNameSize) lpVolumeNameBuffer[0] = '\0';
    if (lpVolumeSerialNumber) *lpVolumeSerialNumber = 0;
    if (lpMaximumComponentLength) *lpMaximumComponentLength = 255;
    if (lpFileSystemFlags) *lpFileSystemFlags = 0;
    if (lpFileSystemNameBuffer && nFileSystemNameSize)
        snprintf(lpFileSystemNameBuffer, nFileSystemNameSize, "ANDROID");
    return 1;
}

/* UTF-8 <-> UTF-16 helpers.  CP_ACP is treated as UTF-8 on Android. */
static int utf8_to_utf16(const char* src, int src_len, unsigned short* dst,
                         int dst_cap) {
    int i = 0;
    int out = 0;
    while (i < src_len) {
        unsigned char c = (unsigned char)src[i];
        unsigned int cp;
        if (c < 0x80) { cp = c; i += 1; }
        else if ((c & 0xE0) == 0xC0 && i + 1 < src_len) {
            cp = ((unsigned int)(c & 0x1F) << 6) |
                 ((unsigned int)(src[i + 1] & 0x3F));
            i += 2;
        } else if ((c & 0xF0) == 0xE0 && i + 2 < src_len) {
            cp = ((unsigned int)(c & 0x0F) << 12) |
                 ((unsigned int)(src[i + 1] & 0x3F) << 6) |
                 ((unsigned int)(src[i + 2] & 0x3F));
            i += 3;
        } else if ((c & 0xF8) == 0xF0 && i + 3 < src_len) {
            cp = ((unsigned int)(c & 0x07) << 18) |
                 ((unsigned int)(src[i + 1] & 0x3F) << 12) |
                 ((unsigned int)(src[i + 2] & 0x3F) << 6) |
                 ((unsigned int)(src[i + 3] & 0x3F));
            i += 4;
        } else {
            cp = 0xFFFD;
            i += 1;
        }
        if (cp >= 0x10000) {
            cp -= 0x10000;
            if (dst && out + 1 < dst_cap) {
                dst[out] = (unsigned short)(0xD800 | (cp >> 10));
                dst[out + 1] = (unsigned short)(0xDC00 | (cp & 0x3FF));
            }
            out += 2;
        } else {
            if (dst && out < dst_cap) dst[out] = (unsigned short)cp;
            out += 1;
        }
    }
    if (dst && out < dst_cap) dst[out] = 0;
    return out;
}

static int utf16_to_utf8(const unsigned short* src, int src_len, char* dst,
                         int dst_cap) {
    int i = 0;
    int out = 0;
    while (i < src_len) {
        unsigned int cp = src[i++];
        unsigned int need;
        if (cp >= 0xD800 && cp <= 0xDBFF && i < src_len) {
            unsigned int low = src[i];
            if (low >= 0xDC00 && low <= 0xDFFF) {
                cp = 0x10000 + ((cp - 0xD800) << 10) + (low - 0xDC00);
                i++;
            }
        } else if (cp >= 0xD800 && cp <= 0xDFFF) {
            cp = 0xFFFD;
        }
        if (cp < 0x80) need = 1;
        else if (cp < 0x800) need = 2;
        else if (cp < 0x10000) need = 3;
        else need = 4;
        if (dst && out + (int)need <= dst_cap) {
            if (need == 1) dst[out] = (char)cp;
            else if (need == 2) {
                dst[out] = (char)(0xC0 | (cp >> 6));
                dst[out + 1] = (char)(0x80 | (cp & 0x3F));
            } else if (need == 3) {
                dst[out] = (char)(0xE0 | (cp >> 12));
                dst[out + 1] = (char)(0x80 | ((cp >> 6) & 0x3F));
                dst[out + 2] = (char)(0x80 | (cp & 0x3F));
            } else {
                dst[out] = (char)(0xF0 | (cp >> 18));
                dst[out + 1] = (char)(0x80 | ((cp >> 12) & 0x3F));
                dst[out + 2] = (char)(0x80 | ((cp >> 6) & 0x3F));
                dst[out + 3] = (char)(0x80 | (cp & 0x3F));
            }
        }
        out += (int)need;
    }
    if (dst && out < dst_cap) dst[out] = 0;
    return out;
}

int MultiByteToWideChar(unsigned int CodePage, unsigned long dwFlags,
                        const char* lpMultiByteStr, int cbMultiByte,
                        unsigned short* lpWideCharStr, int cchWideChar) {
    (void)CodePage;
    (void)dwFlags;
    if (!lpMultiByteStr) return 0;
    if (cbMultiByte < 0) cbMultiByte = (int)strlen(lpMultiByteStr);
    if (!lpWideCharStr || cchWideChar == 0)
        return utf8_to_utf16(lpMultiByteStr, cbMultiByte, NULL, 0);
    return utf8_to_utf16(lpMultiByteStr, cbMultiByte, lpWideCharStr, cchWideChar);
}

int WideCharToMultiByte(unsigned int CodePage, unsigned long dwFlags,
                        const unsigned short* lpWideCharStr, int cchWideChar,
                        char* lpMultiByteStr, int cbMultiByte,
                        const char* lpDefaultChar, int* lpUsedDefaultChar) {
    (void)CodePage;
    (void)dwFlags;
    (void)lpDefaultChar;
    if (lpUsedDefaultChar) *lpUsedDefaultChar = 0;
    if (!lpWideCharStr) return 0;
    if (cchWideChar < 0) {
        cchWideChar = 0;
        while (lpWideCharStr[cchWideChar]) cchWideChar++;
    }
    if (!lpMultiByteStr || cbMultiByte == 0)
        return utf16_to_utf8(lpWideCharStr, cchWideChar, NULL, 0);
    return utf16_to_utf8(lpWideCharStr, cchWideChar, lpMultiByteStr, cbMultiByte);
}
