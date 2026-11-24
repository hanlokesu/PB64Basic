plugins {
    id("com.android.application")
}

val pbAppName = (project.findProperty("pbAppName") as String?)
    ?.takeIf { it.isNotBlank() }
    ?: "PB64Basic App"

val pbApplicationId = (project.findProperty("pbApplicationId") as String?)
    ?.takeIf { it.isNotBlank() }
    ?: "com.example.pb64app"

val pbVersionName = (project.findProperty("pbVersionName") as String?)
    ?.takeIf { it.isNotBlank() }
    ?: "1.0"

android {
    namespace = "com.example.pb64app"
    compileSdk = 35

    defaultConfig {
        applicationId = pbApplicationId
        minSdk = 21
        targetSdk = 35
        versionCode = 1
        versionName = pbVersionName
        resValue("string", "app_name", pbAppName)
    }

    buildTypes {
        release {
            isMinifyEnabled = false
        }
    }

    packaging {
        jniLibs {
            // libpbapp.so is a prebuilt cross-compiled runtime; AGP cannot strip
            // it without a matching NDK strip tool, so keep its symbols explicitly.
            keepDebugSymbols += "**/libpbapp.so"
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
}
