//! C-ABI / Foreign Function Interface (FFI) for FerzDB
//! 
//! This module provides a stable C-compatible interface for integrating
//! FerzDB with other languages like Pawn, C, C++, or Go.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use crate::database::Database;
use std::sync::Mutex;
use std::sync::Arc;

pub struct FerzContext {
    db: Arc<Mutex<Database>>,
}

#[no_mangle]
pub unsafe extern "C" fn ferz_open(path: *const c_char) -> *mut FerzContext {
    let path_str = CStr::from_ptr(path).to_string_lossy().into_owned();
    match Database::open(&path_str) {
        Ok(db) => {
            let context = Box::new(FerzContext {
                db: Arc::new(Mutex::new(db)),
            });
            Box::into_raw(context)
        }
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn ferz_execute(ctx: *mut FerzContext, sql: *const c_char) -> i32 {
    if ctx.is_null() { return -1; }
    let context = &*ctx;
    let sql_str = CStr::from_ptr(sql).to_string_lossy().into_owned();
    
    let mut db = context.db.lock().unwrap();
    match db.execute(&sql_str) {
        Ok(_) => 0,
        Err(_) => -2,
    }
}

#[no_mangle]
pub unsafe extern "C" fn ferz_close(ctx: *mut FerzContext) {
    if !ctx.is_null() {
        let _ = Box::from_raw(ctx);
    }
}

/// Returns the last error message or a success indicator
/// In a more advanced FFI, we'd store the last error in the context.
#[no_mangle]
pub unsafe extern "C" fn ferz_get_version() -> *mut c_char {
    CString::new("FerzDB v0.4.0 (FFI Enabled)").unwrap().into_raw()
}

#[no_mangle]
pub unsafe extern "C" fn ferz_free_string(s: *mut c_char) {
    if !s.is_null() {
        let _ = CString::from_raw(s);
    }
}
