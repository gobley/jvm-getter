// Copyright (c) 2025 Gobley Contributors.

//! Finding `JNI_GetCreatedJavaVMs()` using `dlopen` and `dlsym`.

use core::{mem, ptr};

use libc::{c_char, dlerror, dlopen, dlsym, RTLD_LAZY, RTLD_LOCAL};

use crate::JNI_GetCreatedJavaVMs;

pub(crate) unsafe fn find_jni_get_created_java_vms_from_current_process(
) -> Option<JNI_GetCreatedJavaVMs> {
    let handle = dlopen(ptr::null(), RTLD_LAZY | RTLD_LOCAL);
    if handle.is_null() {
        dlerror();
        return None;
    }

    let symbol = dlsym(handle, b"JNI_GetCreatedJavaVMs\0".as_ptr() as *const c_char);
    if symbol.is_null() {
        dlerror();
        return None;
    }

    Some(mem::transmute::<*mut libc::c_void, JNI_GetCreatedJavaVMs>(
        symbol,
    ))
}
