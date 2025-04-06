// Copyright (c) 2025 Gobley Contributors.

//! Finding `JNI_GetCreatedJavaVMs()` using `LoadLibrary` and `GetProcAddress`.

use libloading::os::windows::Library;

use crate::JNI_GetCreatedJavaVMs;

pub(crate) unsafe fn find_jni_get_created_java_vms_from_current_process(
) -> Option<JNI_GetCreatedJavaVMs> {
    if let Ok(current_module) = Library::open_already_loaded("jvm.dll") {
        if let Ok(symbol) = current_module.get(b"JNI_GetCreatedJavaVMs") {
            return Some(*symbol);
        }
    }
    None
}
