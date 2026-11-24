/* Android thread compatibility layer for PB64Basic.
 *
 * Maps CreateThread / WaitForSingleObject / GetExitCodeThread and the
 * suspend/resume counters onto pthreads.  Android pthreads have no forced
 * suspend, so SuspendThread delivers SIGUSR1 and the target parks in a
 * signal handler until ResumeThread delivers SIGUSR2.  This makes SUSPEND
 * effective for arbitrary user code, not only at runtime call boundaries.
 */
#include <errno.h>
#include <pthread.h>
#include <signal.h>
#include <stdlib.h>
#include <sys/resource.h>
#include <sys/syscall.h>
#include <time.h>
#include <unistd.h>

#define PB_THREAD_MAGIC 0x50425448UL /* "PBTH" */
#define PB_WAIT_OBJECT_0 0UL
#define PB_WAIT_TIMEOUT  258UL
#define PB_STILL_ACTIVE  259UL
#define PB_CREATE_SUSPENDED 0x4UL
#define PB_INFINITE 0xFFFFFFFFUL

typedef unsigned long (*PB_THREAD_PROC)(void*);

typedef struct {
    unsigned long magic;
    pthread_t thread;
    pthread_mutex_t lock;
    pthread_cond_t finished_cond;
    pthread_cond_t gate_cond;
    int finished;
    int closed;
    volatile int suspended;
    volatile int sig_ack;
    volatile int sig_resume;
    unsigned long exit_code;
    PB_THREAD_PROC proc;
    void* param;
    unsigned long thread_id;
} PB_THREAD_HANDLE;

static __thread PB_THREAD_HANDLE* g_signal_thread;

static void pb_suspend_signal(int sig) {
    PB_THREAD_HANDLE* h = g_signal_thread;
    (void)sig;
    if (!h) return;
    h->sig_ack = 1;
    while (h->suspended > 0 && !h->sig_resume) pause();
    h->sig_resume = 0;
    h->sig_ack = 0;
}

static void pb_resume_signal(int sig) {
    PB_THREAD_HANDLE* h = g_signal_thread;
    (void)sig;
    if (h) h->sig_resume = 1;
}

static void install_suspend_signals_once(void) {
    struct sigaction sa;
    sa.sa_handler = pb_suspend_signal;
    sigemptyset(&sa.sa_mask);
    sa.sa_flags = SA_RESTART;
    sigaction(SIGUSR1, &sa, NULL);
    sa.sa_handler = pb_resume_signal;
    sa.sa_flags = SA_RESTART;
    sigaction(SIGUSR2, &sa, NULL);
}

static void install_suspend_signals(void) {
    static pthread_once_t once = PTHREAD_ONCE_INIT;
    pthread_once(&once, install_suspend_signals_once);
}

static long now_ms(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (long)ts.tv_sec * 1000L + ts.tv_nsec / 1000000L;
}

static void thread_finish(PB_THREAD_HANDLE* h, unsigned long code) {
    pthread_mutex_lock(&h->lock);
    h->exit_code = code;
    h->finished = 1;
    pthread_cond_broadcast(&h->finished_cond);
    pthread_mutex_unlock(&h->lock);
}

static void* thread_boot(void* arg) {
    PB_THREAD_HANDLE* h = (PB_THREAD_HANDLE*)arg;
    g_signal_thread = h;
    install_suspend_signals();
    pthread_mutex_lock(&h->lock);
    while (h->suspended > 0) pthread_cond_wait(&h->gate_cond, &h->lock);
    pthread_mutex_unlock(&h->lock);
    {
        unsigned long code = h->proc ? h->proc(h->param) : 0;
        thread_finish(h, code);
    }
    g_signal_thread = NULL;
    return NULL;
}

void* CreateThread(void* lpThreadAttributes, unsigned long dwStackSize,
                   void* lpStartAddress, void* lpParameter,
                   unsigned long dwCreationFlags, unsigned int* lpThreadId) {
    PB_THREAD_HANDLE* h;
    pthread_attr_t attr;
    int rc;
    (void)lpThreadAttributes;
    if (!lpStartAddress) return NULL;
    h = (PB_THREAD_HANDLE*)calloc(1, sizeof(*h));
    if (!h) return NULL;
    h->magic = PB_THREAD_MAGIC;
    h->proc = (PB_THREAD_PROC)lpStartAddress;
    h->param = lpParameter;
    pthread_mutex_init(&h->lock, NULL);
    pthread_cond_init(&h->finished_cond, NULL);
    pthread_cond_init(&h->gate_cond, NULL);
    if (dwCreationFlags & PB_CREATE_SUSPENDED) h->suspended = 1;

    pthread_attr_init(&attr);
    if (dwStackSize) {
        unsigned long stack = dwStackSize;
        if (stack < 16384) stack = 16384;
        pthread_attr_setstacksize(&attr, (size_t)stack);
    }
    rc = pthread_create(&h->thread, &attr, thread_boot, h);
    pthread_attr_destroy(&attr);
    if (rc != 0) {
        pthread_mutex_destroy(&h->lock);
        pthread_cond_destroy(&h->finished_cond);
        pthread_cond_destroy(&h->gate_cond);
        free(h);
        return NULL;
    }
    h->thread_id = (unsigned long)pthread_gettid_np(h->thread);
    if (lpThreadId) *lpThreadId = (unsigned int)h->thread_id;
    return h;
}

static void thread_release(PB_THREAD_HANDLE* h) {
    pthread_mutex_destroy(&h->lock);
    pthread_cond_destroy(&h->finished_cond);
    pthread_cond_destroy(&h->gate_cond);
    h->magic = 0;
    free(h);
}

unsigned long WaitForSingleObject(void* hObject, unsigned long dwMilliseconds) {
    PB_THREAD_HANDLE* h = (PB_THREAD_HANDLE*)hObject;
    struct timespec deadline;
    long start;
    if (!h || h->magic != PB_THREAD_MAGIC) return 1UL; /* WAIT_FAILED */
    pthread_mutex_lock(&h->lock);
    if (h->finished) {
        pthread_mutex_unlock(&h->lock);
        return PB_WAIT_OBJECT_0;
    }
    if (dwMilliseconds == PB_INFINITE) {
        while (!h->finished) pthread_cond_wait(&h->finished_cond, &h->lock);
        pthread_mutex_unlock(&h->lock);
        return PB_WAIT_OBJECT_0;
    }
    start = now_ms();
    while (!h->finished) {
        long elapsed = now_ms() - start;
        long remain = (long)dwMilliseconds - elapsed;
        if (remain <= 0) break;
        clock_gettime(CLOCK_REALTIME, &deadline);
        deadline.tv_sec += remain / 1000;
        deadline.tv_nsec += (remain % 1000) * 1000000L;
        if (deadline.tv_nsec >= 1000000000L) {
            deadline.tv_sec += 1;
            deadline.tv_nsec -= 1000000000L;
        }
        pthread_cond_timedwait(&h->finished_cond, &h->lock, &deadline);
    }
    {
        unsigned long rc = h->finished ? PB_WAIT_OBJECT_0 : PB_WAIT_TIMEOUT;
        pthread_mutex_unlock(&h->lock);
        return rc;
    }
}

int GetExitCodeThread(void* hThread, unsigned int* lpExitCode) {
    PB_THREAD_HANDLE* h = (PB_THREAD_HANDLE*)hThread;
    if (!h || h->magic != PB_THREAD_MAGIC || !lpExitCode) return 0;
    pthread_mutex_lock(&h->lock);
    *lpExitCode = h->finished ? h->exit_code : PB_STILL_ACTIVE;
    pthread_mutex_unlock(&h->lock);
    return 1;
}

unsigned long SuspendThread(void* hThread) {
    PB_THREAD_HANDLE* h = (PB_THREAD_HANDLE*)hThread;
    unsigned long prev;
    if (!h || h->magic != PB_THREAD_MAGIC) return 0xFFFFFFFFUL;
    pthread_mutex_lock(&h->lock);
    prev = (unsigned long)h->suspended;
    h->suspended++;
    if (prev == 0 && !h->finished) {
        h->sig_resume = 0;
        h->sig_ack = 0;
        pthread_kill(h->thread, SIGUSR1);
    }
    pthread_mutex_unlock(&h->lock);
    if (prev == 0) {
        /* Give the target a chance to park before the caller checks status. */
        long start = now_ms();
        while (!h->sig_ack && !h->finished && now_ms() - start < 50) {
            struct timespec ts = {0, 1000000L};
            nanosleep(&ts, NULL);
        }
    }
    return prev;
}

unsigned long ResumeThread(void* hThread) {
    PB_THREAD_HANDLE* h = (PB_THREAD_HANDLE*)hThread;
    unsigned long prev;
    int remaining;
    if (!h || h->magic != PB_THREAD_MAGIC) return 0xFFFFFFFFUL;
    pthread_mutex_lock(&h->lock);
    prev = (unsigned long)h->suspended;
    if (h->suspended > 0) h->suspended--;
    remaining = h->suspended;
    if (remaining == 0) {
        pthread_cond_broadcast(&h->gate_cond);
        if (!h->finished) {
            h->sig_resume = 1;
            pthread_kill(h->thread, SIGUSR2);
        }
    }
    pthread_mutex_unlock(&h->lock);
    if (remaining == 0) {
        long start = now_ms();
        while (h->sig_ack && now_ms() - start < 50) {
            struct timespec ts = {0, 1000000L};
            nanosleep(&ts, NULL);
        }
    }
    return prev;
}

int CloseHandle(void* hObject) {
    PB_THREAD_HANDLE* h = (PB_THREAD_HANDLE*)hObject;
    if (!h || h->magic != PB_THREAD_MAGIC) return 1; /* non-thread handle */
    pthread_mutex_lock(&h->lock);
    if (!h->finished) {
        h->closed = 1;
        pthread_mutex_unlock(&h->lock);
        pthread_detach(h->thread);
        return 1;
    }
    pthread_mutex_unlock(&h->lock);
    pthread_join(h->thread, NULL);
    thread_release(h);
    return 1;
}

int SetThreadPriority(void* hThread, int nPriority) {
    PB_THREAD_HANDLE* h = (PB_THREAD_HANDLE*)hThread;
    pid_t tid;
    int nice_value;
    if (!h || h->magic != PB_THREAD_MAGIC) return 0;
    tid = (pid_t)(h->thread_id ? (long)h->thread_id : 0);
    /* Win32 priority levels: -15 idle .. 0 normal .. 15 time critical. */
    if (nPriority <= -15) nice_value = 19;
    else if (nPriority <= -1) nice_value = 10;
    else if (nPriority == 0) nice_value = 0;
    else if (nPriority <= 1) nice_value = -5;
    else if (nPriority <= 2) nice_value = -10;
    else nice_value = -19;
    if (tid > 0) setpriority(PRIO_PROCESS, tid, nice_value);
    return 1;
}

unsigned long GetCurrentThreadId(void) {
    return (unsigned long)syscall(SYS_gettid);
}

/* Cooperative gate used by runtime blocking calls (Sleep and friends).
 * The signal path parks arbitrary code; this gate is the fallback when a
 * signal is blocked or delivered late. */
void pb_android_thread_suspend_point(void) {
    PB_THREAD_HANDLE* h = g_signal_thread;
    if (!h) return;
    pthread_mutex_lock(&h->lock);
    while (h->suspended > 0) pthread_cond_wait(&h->gate_cond, &h->lock);
    pthread_mutex_unlock(&h->lock);
}
