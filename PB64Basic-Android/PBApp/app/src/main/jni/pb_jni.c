/* JNI bridge: run the PB64Basic-generated main() and return captured console text. */
#include <jni.h>
#include <stddef.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

extern int main(void);
extern const char* pb_android_console_text(void);
extern void pb_android_console_clear(void);
extern size_t pb_android_console_attr_len(void);
extern const unsigned short* pb_android_console_attrs(void);
extern void pb_android_input_clear(void);
extern void pb_android_input_push_byte(int ch);
extern void pb_android_input_push_text(const char* text, int add_newline);

static JavaVM* g_vm;
static jclass g_bridge;
static jmethodID g_msgbox_mid;
static jmethodID g_tone_mid;
static jmethodID g_beep_mid;

JNIEXPORT jint JNICALL JNI_OnLoad(JavaVM* vm, void* reserved) {
    JNIEnv* env = NULL;
    jclass local;
    (void)reserved;
    g_vm = vm;
    if ((*vm)->GetEnv(vm, (void**)&env, JNI_VERSION_1_6) != JNI_OK) {
        return JNI_ERR;
    }
    local = (*env)->FindClass(env, "com/example/pb64app/PbAndroidBridge");
    if (!local) return JNI_ERR;
    g_bridge = (*env)->NewGlobalRef(env, local);
    (*env)->DeleteLocalRef(env, local);
    if (!g_bridge) return JNI_ERR;
    g_msgbox_mid = (*env)->GetStaticMethodID(
        env, g_bridge, "showMessageBox",
        "(Ljava/lang/String;Ljava/lang/String;I)I");
    g_tone_mid = (*env)->GetStaticMethodID(env, g_bridge, "playTone", "(II)I");
    g_beep_mid = (*env)->GetStaticMethodID(env, g_bridge, "messageBeep", "()I");
    if (!g_msgbox_mid || !g_tone_mid || !g_beep_mid) return JNI_ERR;
    return JNI_VERSION_1_6;
}

static JNIEnv* bridge_env(int* attached) {
    JNIEnv* env = NULL;
    *attached = 0;
    if (!g_vm) return NULL;
    if ((*g_vm)->GetEnv(g_vm, (void**)&env, JNI_VERSION_1_6) == JNI_OK)
        return env;
    if ((*g_vm)->AttachCurrentThread(g_vm, (JNIEnv**)&env, NULL) != JNI_OK)
        return NULL;
    *attached = 1;
    return env;
}

/* Called by the Android branch of MessageBoxA in pb_runtime.c.  A negative
   return means "no Activity / bridge unavailable"; the runtime then falls
   back to the console stub instead of hanging or dropping the message. */
int pb_android_ui_messagebox(const char* title, const char* text, unsigned int style) {
    JNIEnv* env;
    int attached;
    int rc = -1;
    jstring jtitle;
    jstring jtext;
    env = bridge_env(&attached);
    if (!env || !g_bridge || !g_msgbox_mid) return -1;
    jtitle = (*env)->NewStringUTF(env, title ? title : "");
    jtext = (*env)->NewStringUTF(env, text ? text : "");
    if (jtitle && jtext) {
        rc = (int)(*env)->CallStaticIntMethod(env, g_bridge, g_msgbox_mid,
                                              jtitle, jtext, (jint)style);
        if ((*env)->ExceptionCheck(env)) {
            (*env)->ExceptionClear(env);
            rc = -1;
        }
    }
    if (jtitle) (*env)->DeleteLocalRef(env, jtitle);
    if (jtext) (*env)->DeleteLocalRef(env, jtext);
    if (attached) (*g_vm)->DetachCurrentThread(g_vm);
    return rc;
}

int pb_android_ui_play_tone(long freq, long dur) {
    JNIEnv* env;
    int attached;
    int rc = -1;
    env = bridge_env(&attached);
    if (!env || !g_bridge || !g_tone_mid) return -1;
    rc = (int)(*env)->CallStaticIntMethod(env, g_bridge, g_tone_mid,
                                          (jint)freq, (jint)dur);
    if ((*env)->ExceptionCheck(env)) {
        (*env)->ExceptionClear(env);
        rc = -1;
    }
    if (attached) (*g_vm)->DetachCurrentThread(g_vm);
    return rc;
}

int pb_android_ui_message_beep(void) {
    JNIEnv* env;
    int attached;
    int rc = -1;
    env = bridge_env(&attached);
    if (!env || !g_bridge || !g_beep_mid) return -1;
    rc = (int)(*env)->CallStaticIntMethod(env, g_bridge, g_beep_mid);
    if ((*env)->ExceptionCheck(env)) {
        (*env)->ExceptionClear(env);
        rc = -1;
    }
    if (attached) (*g_vm)->DetachCurrentThread(g_vm);
    return rc;
}

static jstring run_main(JNIEnv* env, jstring initial) {
    const char* text;
    const char* seed;
    int rc;
    pb_android_console_clear();
    pb_android_input_clear();
    if (initial) {
        seed = (*env)->GetStringUTFChars(env, initial, NULL);
        if (seed) {
            pb_android_input_push_text(seed, 0);
            (*env)->ReleaseStringUTFChars(env, initial, seed);
        }
    }
    rc = main();
    text = pb_android_console_text();
    if (!text || text[0] == '\0') {
        return (*env)->NewStringUTF(env,
            rc == 0 ? "PB64Basic finished with no console output"
                    : "PB64Basic returned a non-zero status");
    }
    return (*env)->NewStringUTF(env, text);
}

JNIEXPORT jstring JNICALL
Java_com_example_pb64app_MainActivity_runPb(JNIEnv* env, jobject thiz) {
    (void)thiz;
    return run_main(env, NULL);
}

JNIEXPORT jstring JNICALL
Java_com_example_pb64app_MainActivity_runPbWithInput(JNIEnv* env, jobject thiz,
                                                     jstring initial) {
    (void)thiz;
    return run_main(env, initial);
}

JNIEXPORT jshortArray JNICALL
Java_com_example_pb64app_MainActivity_runPbAttrs(JNIEnv* env, jobject thiz) {
    size_t len;
    const unsigned short* attrs;
    jshortArray array;
    (void)thiz;
    len = pb_android_console_attr_len();
    attrs = pb_android_console_attrs();
    array = (*env)->NewShortArray(env, (jsize)len);
    if (array && len > 0) {
        (*env)->SetShortArrayRegion(env, array, 0, (jsize)len,
                                    (const jshort*)attrs);
    }
    return array;
}

JNIEXPORT void JNICALL
Java_com_example_pb64app_MainActivity_pushText(JNIEnv* env, jobject thiz,
                                               jstring text, jboolean newline) {
    const char* chars;
    (void)thiz;
    if (!text) return;
    chars = (*env)->GetStringUTFChars(env, text, NULL);
    if (chars) {
        pb_android_input_push_text(chars, newline ? 1 : 0);
        (*env)->ReleaseStringUTFChars(env, text, chars);
    }
}

JNIEXPORT void JNICALL
Java_com_example_pb64app_MainActivity_pushKey(JNIEnv* env, jobject thiz,
                                              jint key) {
    (void)env;
    (void)thiz;
    pb_android_input_push_byte((int)key);
}

JNIEXPORT jint JNICALL
Java_com_example_pb64app_MainActivity_setWorkDir(JNIEnv* env, jobject thiz,
                                                 jstring path) {
    const char* chars;
    int rc = -1;
    (void)thiz;
    if (!path) return -1;
    chars = (*env)->GetStringUTFChars(env, path, NULL);
    if (chars) {
        rc = chdir(chars);
        /* TMP$ and GetTempPathA must point at an app-writable directory. */
        if (rc == 0) setenv("TMPDIR", chars, 1);
        (*env)->ReleaseStringUTFChars(env, path, chars);
    }
    return rc;
}
