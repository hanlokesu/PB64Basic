# PB64Basic Android APK builder

This folder is the minimal Android builder shipped with the Windows release.
It lets JK-IDE compile the active `.bas` file into an Android APK without a
separate PB64Basic-Android checkout.

## Contents

- `tools\build_apk.ps1` - one-source APK builder.
- `PBApp` - Gradle template used by `build_apk.ps1`.
- `pb_android_*.c` - Android support sources linked into `libpbapp.so`.
- `PBApp\app\src\main\jni\pb_jni.c` - JNI bridge.

## Requirements

Run `setup-android.ps1` once, or run `setup.bat` and answer `Y` to
"Install Android build toolchain?". The builder needs JDK 17, Gradle 8.14.3,
Android SDK platform/build-tools 34/35, and NDK r27c.

## Build

From this folder:

```powershell
.\tools\build_apk.ps1 -Source '.\Welcome Hello.bas' -Abi both
```

`setup.bat` fills `AndroidBuildScript` with the absolute path to
`tools\build_apk.ps1`, and `AndroidPb64Exe` with the release
`PB64Basic.exe`. JK-IDE then passes the selected source, ABI, toolchain paths,
and compiler path to the builder.
