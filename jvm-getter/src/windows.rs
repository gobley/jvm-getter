// Copyright (c) 2025 Gobley Contributors.

//! Finding `JNI_GetCreatedJavaVMs()` using `LoadLibrary` and `GetProcAddress`.

use core::mem;

use windows_sys::core::PCSTR;
use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};

use crate::JNI_GetCreatedJavaVMs;

pub(crate) unsafe fn find_jni_get_created_java_vms_from_current_process(
) -> Option<JNI_GetCreatedJavaVMs> {
    let jvm_module = GetModuleHandleA(b"jvm.dll\0".as_ptr() as PCSTR);
    if jvm_module.is_null() {
        GetLastError();
        return None;
    }

    let Some(symbol) = GetProcAddress(jvm_module, b"JNI_GetCreatedJavaVMs\0".as_ptr() as PCSTR)
    else {
        GetLastError();
        return None;
    };

    Some(mem::transmute::<
        unsafe extern "system" fn() -> isize,
        JNI_GetCreatedJavaVMs,
    >(symbol))
}
