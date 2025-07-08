import gobley.gradle.GobleyHost
import gobley.gradle.cargo.dsl.android
import gobley.gradle.cargo.dsl.jvm
import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    alias(libs.plugins.kotlin.multiplatform)
    alias(libs.plugins.kotlin.plugin.atomicfu)
    alias(libs.plugins.android.library)
    alias(libs.plugins.gobley.cargo)
    alias(libs.plugins.gobley.uniffi)
}

cargo {
    builds.jvm {
        embedRustLibrary = (GobleyHost.current.rustTarget == rustTarget)
        features.add("sym-search")
    }
    builds.android {
        if (findProperty("jvm-getter.tests.android-sym-search")?.toString()?.toBoolean() == true) {
            features.add("sym-search")
        }
        features.add("art-parsing")
    }
}

uniffi {
    generateFromLibrary {
        packageName = "dev.gobley.jvmgetter.tests"
        build = GobleyHost.current.rustTarget
    }
}

kotlin {
    androidTarget {
        compilerOptions {
            jvmTarget = JvmTarget.JVM_17
        }
    }
    jvm()
    sourceSets {
        jvmTest {
            dependencies {
                implementation(kotlin("test"))
                implementation(libs.kotest.assertions.core)
                implementation(libs.kotlinx.coroutines.test)
            }
        }
        androidUnitTest {
            dependencies {
                implementation(libs.junit)
                implementation(libs.androidx.test.core)
                implementation(libs.kotest.assertions.core)
                implementation(libs.kotlinx.coroutines.test)
            }
        }
        androidInstrumentedTest {
            dependencies {
                implementation(libs.junit)
                implementation(libs.androidx.test.core)
                implementation(libs.androidx.test.runner)
                implementation(libs.kotest.assertions.core)
                implementation(libs.kotlinx.coroutines.test)
            }
        }
    }
}

android {
    namespace = "dev.gobley.jvmgetter.tests"
    compileSdk = libs.versions.android.compileSdk.get().toInt()
    defaultConfig {
        minSdk = libs.versions.android.minSdk.get().toInt()
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    }
    packaging {
        resources {
            excludes += "/META-INF/{AL2.0,LGPL2.1}"
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
}
