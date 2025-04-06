// Copyright (c) 2025 Gobley Contributors.

//! Finding `JNI_GetCreatedJavaVMs()` using `dlopen` and `dlsym`.

use libloading::os::unix::Library;

use crate::JNI_GetCreatedJavaVMs;

pub(crate) unsafe fn find_jni_get_created_java_vms_from_current_process(
) -> Option<JNI_GetCreatedJavaVMs> {
    if let Ok(symbol) = Library::this().get(b"JNI_GetCreatedJavaVMs") {
        return Some(*symbol);
    }
    None
}
