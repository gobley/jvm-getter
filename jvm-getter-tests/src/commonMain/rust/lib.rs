// Copyright (c) 2025 Gobley Contributors.

use std::ffi::c_void;
use std::mem::MaybeUninit;

use jni::objects::{JClass, JValue, JValueGen};
use jni::sys::{JavaVM, JNI_OK};
use jni::JNIEnv;

#[uniffi::export]
pub fn find_jni_get_created_java_vms() -> u64 {
    unsafe { jvm_getter::find_jni_get_created_java_vms() }
        .map(|f| f as *mut c_void as usize as u64)
        .unwrap_or_default()
}

#[uniffi::export]
pub fn get_java_vm() -> u64 {
    unsafe { get_java_vm_impl() }
        .map(|f| f as *mut c_void as usize as u64)
        .unwrap_or_default()
}

#[uniffi::export]
pub fn get_simple_object_field_value_without_jni_on_load() -> Option<String> {
    let java_vm = unsafe { get_java_vm_impl()? };
    let java_vm = unsafe { jni::JavaVM::from_raw(java_vm).ok()? };
    let mut env = java_vm.attach_current_thread_as_daemon().ok()?;

    #[allow(non_snake_case)]
    let SimpleObject = find_class_via_env(&mut env, "dev/gobley/jvmgetter/tests/SimpleObject")?;
    #[allow(non_snake_case)]
    let simple_value = env
        .get_static_field(SimpleObject, "simpleValue", "Ljava/lang/String;")
        .ok()?;
    let JValueGen::Object(simple_value) = simple_value else {
        return None;
    };
    let simple_value = simple_value.into();
    let simple_value = env.get_string(&simple_value).ok()?;
    Some(simple_value.to_string_lossy().to_string())
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn get_simple_object_field_value_without_jni_on_load_from_rust_thread() -> Option<String>
{
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .thread_name("jvm-getter thread")
        .build()
        .ok()?;

    let caller_id = std::thread::current().id();
    let (callee_id, value) = runtime
        .spawn(async move {
            (
                std::thread::current().id(),
                get_simple_object_field_value_without_jni_on_load(),
            )
        })
        .await
        .ok()?;

    if caller_id == callee_id {
        panic!("invoked from a Java thread");
    }

    value
}

fn find_class_via_env<'a>(env: &mut JNIEnv<'a>, class_name: &str) -> Option<JClass<'a>> {
    if let Ok(class) = env.find_class(class_name) {
        return Some(class);
    }
    let _ = env.exception_clear();
    find_class_via_application_class_loader(env, class_name)
}

/// [`find_class_via_application_class_loader`] loads the given class using the current application
/// instance's class loader. When the current thread is created on the Rust side and there are no
/// Java function frames in the current call stack, [`JNIEnv::find_class`] will use the default
/// system class loader, which can't find classes other than the ones in the Java standard library
/// or in the Android framework.
///
/// This function retrieves the current application instance using the private `ActivityThread`
/// and retrieves the application's class loader using [`Context.getClassLoader`].
///
/// [`Context.getClassLoader`]: https://developer.android.com/reference/android/content/Context#getClassLoader()
fn find_class_via_application_class_loader<'a>(
    env: &mut JNIEnv<'a>,
    class_name: &str,
) -> Option<JClass<'a>> {
    #[allow(non_snake_case)]
    let ActivityThread = env.find_class("android/app/ActivityThread").ok()?;

    let current_activity_thread = env
        .call_static_method(
            ActivityThread,
            "currentActivityThread",
            "()Landroid/app/ActivityThread;",
            &[],
        )
        .ok()?
        .l()
        .ok()?;

    let application = env
        .call_method(
            current_activity_thread,
            "getApplication",
            "()Landroid/app/Application;",
            &[],
        )
        .ok()?
        .l()
        .ok()?;

    let application_class_loader = env
        .call_method(
            application,
            "getClassLoader",
            "()Ljava/lang/ClassLoader;",
            &[],
        )
        .ok()?
        .l()
        .ok()?;

    Some(
        env.call_method(
            application_class_loader,
            "loadClass",
            "(Ljava/lang/String;)Ljava/lang/Class;",
            &[JValue::Object(&env.new_string(class_name).ok()?.into())],
        )
        .ok()?
        .l()
        .ok()?
        .into(),
    )
}

unsafe fn get_java_vm_impl() -> Option<*mut JavaVM> {
    let get_java_vm = jvm_getter::find_jni_get_created_java_vms().unwrap();
    let mut java_vm = MaybeUninit::uninit();
    let status = get_java_vm(java_vm.as_mut_ptr(), 1, &mut 0);
    if status != JNI_OK {
        return None;
    }
    Some(java_vm.assume_init())
}

uniffi::setup_scaffolding!();
