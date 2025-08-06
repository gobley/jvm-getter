# `jvm-getter`

[![License](https://img.shields.io/github/license/gobley/jvm-getter)](https://github.com/gobley/jvm-getter/blob/main/LICENSE)
[![Crates.io](https://img.shields.io/crates/v/jvm-getter)](https://crates.io/crates/jvm-getter)
[![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/gobley/jvm-getter/pr-test-desktop.yml?branch=main&label=tests)](https://github.com/gobley/jvm-getter/actions/workflows/pr-test-desktop.yml?query=branch%3Amain)

> ⚠️ This library depends on implementation details of Android, not its public APIs. Use at your own
> risk.

A tiny `no_std` library for finding [`JNI_GetCreatedJavaVMs()`] on Android 24 to 30.

[`JNI_GetCreatedJavaVMs()`] is a JNI function that returns the list of Java VM instances that have
been created during runtime. Unfortunately, on Android API level 30 or lower,
[`JNI_GetCreatedJavaVMs()`] is **not** one of the public APIs. Therefore, the recommended way by the
official Android documentation is to use `JNI_OnLoad()`, which has a `JavaVM` parameter.

This is painful for cross-platform library developers, especially when the OS feature they want to
use is coupled with Java on Android, as they have to provide a way to pass `JNIEnv` to the library.
By using [`JNI_GetCreatedJavaVMs()`], you can retrieve the `JavaVM` instance, and you can even
create `JNIEnv` instances for threads created on the Rust side.

With `jvm-getter`, libraries can provide cross-platform interfaces without demaning the consumers
to manually handle Java-specific logic for Android. To learn about the strategy to find
[`JNI_GetCreatedJavaVMs()`] used by `jvm-getter`, please refer to the documentation of
[`find_jni_get_created_java_vms()`]. For compatibility, [`find_jni_get_created_java_vms()`] is
also available on Desktop platforms, including Windows, macOS, and Linux.

[`JNI_GetCreatedJavaVMs()`]: https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/invocation.html#JNI_GetCreatedJavaVMs
[`find_jni_get_created_java_vms()`]: ./jvm-getter/src/lib.rs#L42-L60

## How to use

`jvm-getter` is designed to have only one public function, `find_jni_get_created_java_vms`. This
function directly retrieves the pointer to `JNI_GetCreatedJavaVMs` without implementing any internal
caching. For efficient and safe access to the JavaVM pointer, it is recommended to implement your
own caching mechanism. A common approach is to use `std::sync::OnceLock` or
`once_cell::sync::OnceCell` to store and initialize the JavaVM instance exactly once. The following
example demonstrates how to `retrieve` and cache the `JavaVM` pointer using `jvm-getter` and
std::sync::OnceLock:

```rust
use std::mem::MaybeUninit;
use std::sync::OnceLock;

use jni::sys::{JavaVM as RawJavaVM, JNI_OK};
use jni::JavaVM;

pub fn vm() -> &'static JavaVM {
    static VM: OnceLock<JavaVM> = OnceLock::new();
    VM.get_or_init(|| {
        #[allow(non_snake_case)]
        let JNI_GetCreatedJavaVMs = unsafe {
            jvm_getter::find_jni_get_created_java_vms()
                .expect("could not find JNI_GetCreatedJavaVMs")
        };

        let mut vm: MaybeUninit<*mut RawJavaVM> = MaybeUninit::uninit();
        let status = unsafe { JNI_GetCreatedJavaVMs(vm.as_mut_ptr(), 1, &mut 0) };
        if status != JNI_OK {
            panic!("no JavaVM was found by JNI_GetCreatedJavaVMs");
        }

        let vm = unsafe { JavaVM::from_raw(vm.assume_init()) };
        vm.expect("JNI_GetCreatedJavaVMs returned nullptr")
    })
}
```

`jvm-getter` provides several Cargo features to control the behavior of
`find_jni_get_created_java_vms`. Here's the list of the Cargo features:

- `alloc`: Enables the use of `alloc::vec::Vec` (`std::vec::Vec`) for allocating a memory buffer for
  `libart.so`.
- `sym-search`: Enables finding `JNI_GetCreatedJavaVMs` using platform-specific symbol-searching APIs.
  - `sym-search-unix`: Enables finding using `dlsym` on Unix systems.
  - `sym-search-windows`: Enables finding using `GetProcAddress` on Windows.
- `art-parsing`: Enables finding `JNI_GetCreatedJavaVMs` by directly parsing `libart.so`.

All features are enabled by default. If you want to completely remove the dependency on the
standard library, set `default-features` to `false` and enable `sym-search` and `art-parsing` only.

If your app targets API level 31 (Android 12) or a higher version, you can also disable
`art-parsing` as `JNI_GetCreatedJavaVMs` is a public API.

```toml
jvm-getter = { version = "0.1", default-features = false, features = [
    "sym-search",
    "art-parsing",
] }
```

## Contribution

If you find a device on which `jvm-getter` doesn’t run correctly, please report it. If you also
succeed in getting `jvm-getter` to work on that device without breaking compatibility on other
supported devices, you’re welcome to open a pull request.
