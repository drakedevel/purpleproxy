#![allow(unused_must_use)]
#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

use capnp_rpc::capability::WaitForContent;
use purpleproxy::with_proxy;
use std::ffi::CString;
use std::io::Write;
use std::ptr;
use std::mem;

unsafe fn rtld_next(sym: &str) -> *mut u8 {
    const RTLD_NEXT: *mut u8 = -1is as *mut u8;
    extern "C" { fn dlsym(handle: *mut u8, sym: *const i8) -> *mut u8; }
    let raw_sym = CString::from_slice(sym.as_bytes());
    dlsym(RTLD_NEXT, raw_sym.as_ptr())
}

type __builtin_va_list = [*const (); 1];

// bindgen!("purple.h", link="purple", types_only=true);
include! { concat!(env!("OUT_DIR"), "/purple.rs") }
include! { concat!(env!("OUT_DIR"), "/stubs.rs") }
