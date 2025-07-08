// Copyright (c) 2025 Gobley Contributors.

//! Android implementation of finding `JNI_GetCreatedJavaVMs()`.

extern crate alloc;

use alloc::vec::Vec;
use core::ffi::{c_char, c_int, c_void, CStr};
use core::mem::MaybeUninit;
use libc::O_RDONLY;

use goblin::elf::Elf;

use crate::JNI_GetCreatedJavaVMs;

pub(crate) const PROP_VALUE_MAX: usize = 93;

pub(crate) unsafe fn get_art_library_filename(
    art_library_filename: &mut MaybeUninit<[u8; PROP_VALUE_MAX]>,
) -> &str {
    extern "C" {
        fn __system_property_get(name: *const c_char, value: *mut c_char) -> c_int;
    }
    // System property persist.sys.dalvik.vm.lib.2 contains the name of the ART shared library file.
    let length = __system_property_get(
        b"persist.sys.dalvik.vm.lib.2\0".as_ptr() as *const _,
        art_library_filename.as_mut_ptr() as *mut _,
    );
    if length == 0 {
        "libart.so"
    } else {
        core::str::from_utf8_unchecked(&art_library_filename.assume_init_ref()[0..length as usize])
    }
}

pub(crate) unsafe fn find_jni_get_created_java_vms_from_library_filename(
    art_library_filename: &str,
) -> Option<JNI_GetCreatedJavaVMs> {
    let mut result: MaybeUninit<JNI_GetCreatedJavaVMs> = MaybeUninit::uninit();
    if dl_iterate_phdr(
        parse_and_find_jni_get_created_java_vms,
        &mut FindContext {
            art_library_filename,
            result: &mut result,
        } as *mut FindContext as *mut c_void,
    ) != 0
    {
        return Some(result.assume_init());
    }

    None
}

#[repr(C)]
#[allow(non_camel_case_types)]
struct dl_phdr_info {
    dlpi_addr: usize,
    dlpi_name: *const c_char,
    // other fields are omitted as unused
}

extern "C" {
    /// Iterates over all loaded shared libraries of the current process.
    fn dl_iterate_phdr(
        callback: unsafe extern "C" fn(
            info: &dl_phdr_info,
            info_size: usize,
            context: *mut c_void,
        ) -> c_int,
        context: *mut c_void,
    ) -> c_int;
}

struct FindContext<'a> {
    art_library_filename: &'a str,
    result: &'a mut MaybeUninit<JNI_GetCreatedJavaVMs>,
}

unsafe extern "C" fn parse_and_find_jni_get_created_java_vms(
    info: &dl_phdr_info,
    _info_size: usize,
    context: *mut c_void,
) -> c_int {
    let context = &mut *(context as *mut FindContext);

    // Check whether the current library is the library we're finding.
    let library_path = CStr::from_ptr(info.dlpi_name);
    let library_path = library_path.to_bytes();
    if !library_path.ends_with(context.art_library_filename.as_bytes()) {
        return 0;
    }

    let Some(library) = read_file(library_path) else {
        return 0;
    };

    #[cfg(feature = "alloc")]
    let library = library.as_slice();
    #[cfg(not(feature = "alloc"))]
    let library = unsafe { library.as_slice() };

    let Ok(library) = Elf::parse(library) else {
        return 0;
    };

    // Find JNI_GetCreatedJavaVMs in the parsed library file.
    let Some(symbol) = library
        .syms
        .iter()
        .find(|sym| library.strtab.get_at(sym.st_name) == Some("JNI_GetCreatedJavaVMs"))
    else {
        return 0;
    };

    // The sum of the library's base address and the symbol offset is the exact address of
    // JNI_GetCreatedJavaVMs.
    context
        .result
        .write(core::mem::transmute::<usize, JNI_GetCreatedJavaVMs>(
            info.dlpi_addr + symbol.st_value as usize,
        ));

    // We found the symbol
    1
}

#[cfg(feature = "alloc")]
unsafe fn read_file(path: &[u8]) -> Option<Vec<u8>> {
    let file = libc::open(path.as_ptr() as _, O_RDONLY);
    if file <= 0 {
        return None;
    }

    let mut stat = MaybeUninit::uninit();

    if libc::fstat(file, stat.as_mut_ptr()) < 0 {
        libc::close(file);
        return None;
    }
    let stat = stat.assume_init();
    let file_size = stat.st_size as usize;

    let mut result = Vec::with_capacity(file_size);
    let buffer = result.spare_capacity_mut();

    let mut num_total_bytes_read = 0usize;
    loop {
        let num_bytes_read = libc::read(
            file,
            buffer[num_total_bytes_read..].as_mut_ptr() as _,
            buffer.len() - num_total_bytes_read,
        );
        if num_bytes_read < 0 {
            libc::close(file);
            return None;
        }
        if num_bytes_read == 0 {
            break;
        }
        num_total_bytes_read += num_bytes_read as usize;
    }
    result.set_len(file_size);

    libc::close(file);
    Some(result)
}

#[cfg(not(feature = "alloc"))]
struct RawVec {
    buffer: core::ptr::NonNull<u8>,
    len: usize,
}

#[cfg(not(feature = "alloc"))]
impl RawVec {
    pub fn new(len: usize) -> Option<RawVec> {
        let buffer = core::ptr::NonNull::new(unsafe { libc::malloc(len) } as *mut _)?;
        Some(RawVec { buffer, len })
    }

    pub fn as_uninit_slice(&mut self) -> &mut [MaybeUninit<u8>] {
        unsafe { core::slice::from_raw_parts_mut(self.buffer.cast().as_ptr(), self.len) }
    }

    pub unsafe fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.buffer.as_ptr(), self.len) }
    }

    pub unsafe fn as_slice(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.buffer.as_ptr(), self.len) }
    }
}

#[cfg(not(feature = "alloc"))]
impl Drop for RawVec {
    fn drop(&mut self) {
        unsafe { libc::free(self.buffer.as_ptr() as *mut _) }
    }
}

#[cfg(not(feature = "alloc"))]
unsafe fn read_file(path: &[u8]) -> Option<RawVec> {
    let file = libc::open(path.as_ptr() as _, O_RDONLY);
    if file <= 0 {
        return None;
    }

    let mut stat = MaybeUninit::uninit();

    if libc::fstat(file, stat.as_mut_ptr()) < 0 {
        libc::close(file);
        return None;
    }
    let stat = stat.assume_init();
    let file_size = stat.st_size as usize;

    let mut result = RawVec::new(file_size)?;
    let buffer = result.as_uninit_slice();

    let mut num_total_bytes_read = 0usize;
    loop {
        let num_bytes_read = libc::read(
            file,
            buffer[num_total_bytes_read..].as_mut_ptr() as _,
            buffer.len() - num_total_bytes_read,
        );
        if num_bytes_read < 0 {
            libc::close(file);
            return None;
        }
        if num_bytes_read == 0 {
            break;
        }
        num_total_bytes_read += num_bytes_read as usize;
    }

    libc::close(file);
    Some(result)
}
