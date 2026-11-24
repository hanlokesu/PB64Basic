/* Android Win32 global-memory and clipboard compatibility layer.
 *
 * PB's GMEM family and CLIPBOARD statements use HGLOBAL handles plus the
 * clipboard ownership API.  Android has neither, so back them with malloc
 * blocks that carry a size header and a process-local clipboard table.
 */
#include <stdlib.h>
#include <string.h>

#define PB_GMEM_MAGIC 0x5042474DUL /* "PBGM" */
#define PB_CLIP_FORMATS 64

typedef struct {
    unsigned long magic;
    size_t size;
} PB_GMEM_HEADER;

static void* g_clipboard[PB_CLIP_FORMATS];

void* GlobalAlloc(unsigned int uFlags, unsigned long dwBytes) {
    PB_GMEM_HEADER* h;
    size_t total = sizeof(PB_GMEM_HEADER) + (size_t)dwBytes;
    h = (PB_GMEM_HEADER*)malloc(total);
    if (!h) return NULL;
    h->magic = PB_GMEM_MAGIC;
    h->size = (size_t)dwBytes;
    if (uFlags & 0x0040u) /* GMEM_ZEROINIT */
        memset(h + 1, 0, (size_t)dwBytes);
    return (void*)(h + 1);
}

void* GlobalFree(void* hMem) {
    PB_GMEM_HEADER* h;
    if (!hMem) return NULL;
    h = ((PB_GMEM_HEADER*)hMem) - 1;
    if (h->magic == PB_GMEM_MAGIC) free(h);
    return NULL;
}

void* GlobalLock(void* hMem) {
    return hMem;
}

int GlobalUnlock(void* hMem) {
    (void)hMem;
    return 0; /* success; non-zero would mean the block is still locked */
}

unsigned long long GlobalSize(void* hMem) {
    PB_GMEM_HEADER* h;
    if (!hMem) return 0;
    h = ((PB_GMEM_HEADER*)hMem) - 1;
    return h->magic == PB_GMEM_MAGIC ? (unsigned long long)h->size : 0;
}

int OpenClipboard(void* hWndNewOwner) {
    (void)hWndNewOwner;
    return 1;
}

int CloseClipboard(void) {
    return 1;
}

int EmptyClipboard(void) {
    int i;
    for (i = 0; i < PB_CLIP_FORMATS; ++i) {
        if (g_clipboard[i]) {
            GlobalFree(g_clipboard[i]);
            g_clipboard[i] = NULL;
        }
    }
    return 1;
}

void* SetClipboardData(unsigned int uFormat, void* hMem) {
    if (uFormat >= PB_CLIP_FORMATS) return NULL;
    if (g_clipboard[uFormat]) GlobalFree(g_clipboard[uFormat]);
    g_clipboard[uFormat] = hMem; /* ownership transfers to the clipboard */
    return hMem;
}

void* GetClipboardData(unsigned int uFormat) {
    if (uFormat >= PB_CLIP_FORMATS) return NULL;
    return g_clipboard[uFormat];
}
