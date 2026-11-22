#include <jni.h>
#include <stdint.h>

extern void android_init(const char *save_dir);
extern void android_frame(int32_t *out_argb, int32_t len);
extern void android_audio(int16_t *out_pcm, int32_t len);
extern void android_key(int32_t code, int32_t down);
extern void android_char(int32_t code);
extern void android_mouse_move(int32_t dx, int32_t dy);
extern void android_mouse_to(int32_t x, int32_t y);
extern void android_mouse_button(int32_t button, int32_t down);
extern void android_wheel(int32_t delta);
extern void android_blur(void);
extern int32_t android_ui_state(void);

JNIEXPORT void JNICALL
Java_com_rustcraft_minecraft_MainActivity_nativeInit(JNIEnv *env, jclass cls, jstring saveDir) {
    (void)cls;
    const char *path = saveDir ? (*env)->GetStringUTFChars(env, saveDir, 0) : "";
    android_init(path);
    if (saveDir && path) {
        (*env)->ReleaseStringUTFChars(env, saveDir, path);
    }
}

JNIEXPORT void JNICALL
Java_com_rustcraft_minecraft_MainActivity_nativeFrame(JNIEnv *env, jclass cls, jintArray outPixels) {
    (void)cls;
    jsize len = (*env)->GetArrayLength(env, outPixels);
    jint *ptr = (*env)->GetIntArrayElements(env, outPixels, 0);
    if (ptr) {
        android_frame((int32_t *)ptr, (int32_t)len);
        (*env)->ReleaseIntArrayElements(env, outPixels, ptr, 0);
    }
}

JNIEXPORT void JNICALL
Java_com_rustcraft_minecraft_MainActivity_nativeAudio(JNIEnv *env, jclass cls, jshortArray outPcm) {
    (void)cls;
    jsize len = (*env)->GetArrayLength(env, outPcm);
    jshort *ptr = (*env)->GetShortArrayElements(env, outPcm, 0);
    if (ptr) {
        android_audio((int16_t *)ptr, (int32_t)len);
        (*env)->ReleaseShortArrayElements(env, outPcm, ptr, 0);
    }
}

JNIEXPORT void JNICALL
Java_com_rustcraft_minecraft_MainActivity_nativeKey(JNIEnv *env, jclass cls, jint code, jint down) {
    (void)env; (void)cls;
    android_key(code, down);
}

JNIEXPORT void JNICALL
Java_com_rustcraft_minecraft_MainActivity_nativeChar(JNIEnv *env, jclass cls, jint code) {
    (void)env; (void)cls;
    android_char(code);
}

JNIEXPORT void JNICALL
Java_com_rustcraft_minecraft_MainActivity_nativeMouseMove(JNIEnv *env, jclass cls, jint dx, jint dy) {
    (void)env; (void)cls;
    android_mouse_move(dx, dy);
}

JNIEXPORT void JNICALL
Java_com_rustcraft_minecraft_MainActivity_nativeMouseTo(JNIEnv *env, jclass cls, jint x, jint y) {
    (void)env; (void)cls;
    android_mouse_to(x, y);
}

JNIEXPORT void JNICALL
Java_com_rustcraft_minecraft_MainActivity_nativeMouseButton(JNIEnv *env, jclass cls, jint button, jint down) {
    (void)env; (void)cls;
    android_mouse_button(button, down);
}

JNIEXPORT void JNICALL
Java_com_rustcraft_minecraft_MainActivity_nativeWheel(JNIEnv *env, jclass cls, jint delta) {
    (void)env; (void)cls;
    android_wheel(delta);
}

JNIEXPORT void JNICALL
Java_com_rustcraft_minecraft_MainActivity_nativeBlur(JNIEnv *env, jclass cls) {
    (void)env; (void)cls;
    android_blur();
}

JNIEXPORT jint JNICALL
Java_com_rustcraft_minecraft_MainActivity_nativeUiState(JNIEnv *env, jclass cls) {
    (void)env; (void)cls;
    return (jint)android_ui_state();
}
