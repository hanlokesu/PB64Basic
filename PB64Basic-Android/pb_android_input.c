/* Android keyboard queue for PB64Basic.
 *
 * Android has no Win32 console input handle, and bionic stdin is not a tty
 * under an Activity.  The Java layer pushes bytes here; the runtime's
 * _getch/_kbhit and ReadConsoleA consume them.  The queue is deliberately
 * byte-oriented so WAITKEY$ and INPUT share one input stream.
 */
#include <pthread.h>
#include <stdio.h>
#include <string.h>

#define PB_ANDROID_INPUT_CAP 4096

static unsigned char g_input[PB_ANDROID_INPUT_CAP];
static int g_input_head = 0;
static int g_input_tail = 0;
static int g_input_count = 0;
static pthread_mutex_t g_input_lock = PTHREAD_MUTEX_INITIALIZER;
static pthread_cond_t g_input_ready = PTHREAD_COND_INITIALIZER;

static void push_byte_locked(unsigned char ch) {
    if (g_input_count >= PB_ANDROID_INPUT_CAP) {
        /* Drop the oldest byte instead of blocking the UI thread. */
        g_input_head = (g_input_head + 1) % PB_ANDROID_INPUT_CAP;
        g_input_count--;
    }
    g_input[g_input_tail] = ch;
    g_input_tail = (g_input_tail + 1) % PB_ANDROID_INPUT_CAP;
    g_input_count++;
}

void pb_android_input_clear(void) {
    pthread_mutex_lock(&g_input_lock);
    g_input_head = 0;
    g_input_tail = 0;
    g_input_count = 0;
    pthread_mutex_unlock(&g_input_lock);
}

void pb_android_input_push_byte(int ch) {
    pthread_mutex_lock(&g_input_lock);
    push_byte_locked((unsigned char)ch);
    pthread_cond_signal(&g_input_ready);
    pthread_mutex_unlock(&g_input_lock);
}

void pb_android_input_push_text(const char* text, int add_newline) {
    const unsigned char* p = (const unsigned char*)text;
    pthread_mutex_lock(&g_input_lock);
    if (p) {
        while (*p) push_byte_locked(*p++);
    }
    if (add_newline) push_byte_locked('\n');
    pthread_cond_signal(&g_input_ready);
    pthread_mutex_unlock(&g_input_lock);
}

int pb_android_input_count(void) {
    int n;
    pthread_mutex_lock(&g_input_lock);
    n = g_input_count;
    pthread_mutex_unlock(&g_input_lock);
    return n;
}

/* Non-blocking pop used by ReadConsoleInputA when the caller just checked
 * GetNumberOfConsoleInputEvents.  Returns -1 when the queue is empty. */
int pb_android_input_take(void) {
    int ch = -1;
    pthread_mutex_lock(&g_input_lock);
    if (g_input_count > 0) {
        ch = (int)g_input[g_input_head];
        g_input_head = (g_input_head + 1) % PB_ANDROID_INPUT_CAP;
        g_input_count--;
    }
    pthread_mutex_unlock(&g_input_lock);
    return ch;
}

int pb_android_input_peek(int index) {
    int ch = -1;
    if (index < 0) return -1;
    pthread_mutex_lock(&g_input_lock);
    if (index < g_input_count) {
        ch = (int)g_input[(g_input_head + index) % PB_ANDROID_INPUT_CAP];
    }
    pthread_mutex_unlock(&g_input_lock);
    return ch;
}

int pb_android_kbhit(void) {
    return pb_android_input_count() > 0;
}

int pb_android_getch(void) {
    int ch;
    pthread_mutex_lock(&g_input_lock);
    while (g_input_count == 0) {
        pthread_cond_wait(&g_input_ready, &g_input_lock);
    }
    ch = (int)g_input[g_input_head];
    g_input_head = (g_input_head + 1) % PB_ANDROID_INPUT_CAP;
    g_input_count--;
    pthread_mutex_unlock(&g_input_lock);
    return ch;
}

/* Read one line for ReadConsoleA.  Blocks until CR/LF; strips the terminator.
 * Backspace removes the previous byte.  Returns the number of bytes stored. */
int pb_android_read_line(char* out, int max) {
    int n = 0;
    int ch;
    if (!out || max <= 0) return 0;
    for (;;) {
        ch = pb_android_getch();
        if (ch == '\r') {
            /* Swallow the LF of a CRLF pair if it is already queued. */
            pthread_mutex_lock(&g_input_lock);
            if (g_input_count > 0 && g_input[g_input_head] == '\n') {
                g_input_head = (g_input_head + 1) % PB_ANDROID_INPUT_CAP;
                g_input_count--;
            }
            pthread_mutex_unlock(&g_input_lock);
            break;
        }
        if (ch == '\n') break;
        if (ch == 0x08 || ch == 0x7F) {
            if (n > 0) n--;
            continue;
        }
        if (ch == 0) continue;
        if (n < max - 1) out[n++] = (char)ch;
    }
    out[n] = '\0';
    return n;
}
