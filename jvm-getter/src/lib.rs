// Copyright (c) 2025 Gobley Contributors.

//! > ⚠️ This library depends on implementation details of Android, not its public APIs. Use at your
//! > own risk.
//!
//! A tiny `no_std` library for finding [`JNI_GetCreatedJavaVMs()`] on Android 24 to 30.
//!
//! [`JNI_GetCreatedJavaVMs()`] is a JNI function that returns the list of Java VM instances that
//! have been created during runtime. Unfortunately, on Android API level 30 or lower,
//! [`JNI_GetCreatedJavaVMs()`] is **not** one of the public APIs. Therefore, the recommended way by
//! the official Android documentation is to use `JNI_OnLoad()`, which has a `JavaVM` parameter.
//!
//! This is painful for cross-platform library developers, especially when the OS feature they want
//! to use is coupled with Java on Android, as they have to provide a way to pass `JNIEnv` to the
//! library. By using [`JNI_GetCreatedJavaVMs()`], you can retrieve the `JavaVM` instance, and you
//! can even create `JNIEnv` instances for threads created on the Rust side.
//!
//! With `jvm-getter`, libraries can provide cross-platform interfaces without demaning the
//! consumers to manually handle Java-specific logic for Android. To learn about the strategy to
//! find [`JNI_GetCreatedJavaVMs()`] used by `jvm-getter`, please refer to the documentation of
//! [`find_jni_get_created_java_vms()`]. For compatibility, [`find_jni_get_created_java_vms()`] is
//! also available on Desktop platforms, including Windows, macOS, and Linux.
//!
//! [`JNI_GetCreatedJavaVMs()`]: https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/invocation.html#JNI_GetCreatedJavaVMs

#![no_std]

#[cfg(target_os = "android")]
mod android;
#[cfg(target_family = "unix")]
mod unix;
#[cfg(target_os = "windows")]
mod windows;

use jni_sys::{jint, jsize, JavaVM};

/// The function pointer type of `JNI_GetCreatedJavaVMs()`.
#[allow(non_camel_case_types)]
pub type JNI_GetCreatedJavaVMs =
    unsafe extern "system" fn(vmBuf: *mut *mut JavaVM, bufLen: jsize, nVMs: *mut jsize) -> jint;

/// Finds the current process's [JNI_GetCreatedJavaVMs]. For compatibility, this function is
/// also available on Desktop platforms other than Android, including Windows, macOS, and Linux.
///
/// # Strategy
///
/// This function finds the address where the Android Runtime is loaded, parses `libart.so` using
/// [goblin], finds the location of [JNI_GetCreatedJavaVMs] in `libart.so`, and computes its
/// location in the memory by adding an offset to the address of `libart.so`.
///
/// On API level 24 or higher, `dlopen`ing private API results in an runtime error. Thus, we locate
/// `libart.so` by iterating over the loaded shared object list of the current process using
/// `dl_iterate_phdr`.
///
/// Since `dlsym` is also prohibited for private APIs, using [goblin] is necessary.
///
/// # Safety
///
/// This function depends on implementation details of Android, not its public APIs. Use at your
/// own risk.
pub unsafe fn find_jni_get_created_java_vms() -> Option<JNI_GetCreatedJavaVMs> {
    // For API level 31 or higher, or level 23 or lower, where JNI_GetCreatedJavaVMs is a public
    // API, we can just use `dlsym` to find the symbol.
    #[cfg(target_family = "unix")]
    let symbol = unix::find_jni_get_created_java_vms_from_current_process();

    #[cfg(target_os = "windows")]
    let symbol = windows::find_jni_get_created_java_vms_from_current_process();

    if let Some(symbol) = symbol {
        return Some(symbol);
    }

    #[cfg(target_os = "android")]
    {
        use core::mem::MaybeUninit;

        let mut art_library_filename = MaybeUninit::uninit();
        let art_library_filename = android::get_art_library_filename(&mut art_library_filename);
        android::find_jni_get_created_java_vms_from_library_filename(art_library_filename)
    }
    #[cfg(not(target_os = "android"))]
    None
}
