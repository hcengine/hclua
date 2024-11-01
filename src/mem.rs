use std::{ffi::c_void, ptr};

use libc::{free, realloc};
use log::{info, warn};

pub struct MemLimit {
    pub mem: isize,
    pub mem_limit: isize,
    pub mem_report: isize,
    pub name: Option<String>,
}

impl MemLimit {
    pub fn new(mem_limit: usize, name: Option<String>) -> Self {
        Self {
            mem: 0,
            mem_limit: mem_limit as isize,
            mem_report: 1024,
            name,
        }
    }
}

pub extern "C" fn lalloc(
    ud: *mut c_void,
    ptr: *mut c_void,
    osize: libc::size_t,
    nsize: libc::size_t,
) -> *mut c_void {
    unsafe {
        let mem = ud as *mut MemLimit;
        if nsize == 0 {
            if !ptr.is_null() {
                free(ptr);
                (*mem).mem -= osize as isize;
            }
            return ptr::null_mut() as *mut c_void;
        }

        let mem_diff = nsize as isize - osize as isize;
        (*mem).mem += mem_diff;

        if (*mem).mem > (*mem).mem_report {
            if (*mem).mem > (*mem).mem_limit && (ptr.is_null() || nsize > osize) {
                warn!(
                    "{} Memory error current {} M, limit {} M",
                    (*mem).name.as_ref().unwrap_or(&"unknow".to_string()),
                    (*mem).mem as f32 / (1024f32 * 1024f32),
                    (*mem).mem_limit as f32 / (1024f32 * 1024f32)
                );
                return ptr::null_mut() as *mut c_void;
            }
            (*mem).mem_report *= 2;
            info!("{} 内存发生拓展 {} M", (*mem).name.as_ref().unwrap_or(&"unknow".to_string()), 
            ((*mem).mem_report as f32 / (1024f32 * 1024f32)));
        }

        return realloc(ptr, nsize);
    }
}
