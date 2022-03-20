use std::ffi::CString;
use std::os::raw::c_char;
use std::panic::*;
#[no_mangle]
pub extern "C" fn demo_free_rust(ptr: *mut i8, length: u32) {
    let catch_result = catch_unwind(AssertUnwindSafe(|| {
        let len: usize = length as usize;
        unsafe {
            Vec::from_raw_parts(ptr, len, len);
        }
    }));
    match catch_result {
        Ok(_) => {}
        Err(e) => {
            println!("catch_unwind of `rsbind free_rust` error: {:?}", e);
        }
    };
}
#[no_mangle]
pub extern "C" fn demo_free_str(ptr: *mut c_char) {
    let catch_result = catch_unwind(AssertUnwindSafe(|| unsafe {
        CString::from_raw(ptr);
    }));
    match catch_result {
        Ok(_) => {}
        Err(e) => {
            println!("catch_unwind of `rsbind free_str` error: {:?}", e);
        }
    };
}
#[repr(C)]
pub struct CInt8Array {
    pub ptr: *const i8,
    pub len: i32,
}
#[repr(C)]
pub struct CInt16Array {
    pub ptr: *const i16,
    pub len: i32,
}
#[repr(C)]
pub struct CInt32Array {
    pub ptr: *const i32,
    pub len: i32,
}
#[repr(C)]
pub struct CInt64Array {
    pub ptr: *const i64,
    pub len: i32,
}
