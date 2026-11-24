/* Minimal Android platform primitives for the first full-runtime link test. */
#include <time.h>

extern void pb_android_thread_suspend_point(void);

void Sleep(unsigned long ms) {
    struct timespec ts;
    pb_android_thread_suspend_point();
    if (ms == 0) return;
    ts.tv_sec = (time_t)(ms / 1000UL);
    ts.tv_nsec = (long)((ms % 1000UL) * 1000000UL);
    nanosleep(&ts, 0);
    pb_android_thread_suspend_point();
}
