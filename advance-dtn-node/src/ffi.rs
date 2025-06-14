use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_uchar, c_void};

use crate::peers::PeerDirectory;
use crate::cache::{CachedMsg, Cache};

#[no_mangle]
pub extern "C" fn ffi_add_peer(peers_ptr: *mut c_void, eid_ptr: *const c_char, addr_ptr: *const c_char) {
    let eid = unsafe { CStr::from_ptr(eid_ptr).to_string_lossy().to_string() };
    let addr = unsafe { CStr::from_ptr(addr_ptr).to_string_lossy().to_string() };
    let peers = unsafe { &*(peers_ptr as *const PeerDirectory) };
    peers.add(eid, addr);
}

#[no_mangle]
pub extern "C" fn ffi_cache_bundle(cache_ptr: *mut c_void, dest_ptr: *const c_char, data_ptr: *const c_uchar, data_len: usize) {
    let dest = unsafe { CStr::from_ptr(dest_ptr).to_string_lossy().to_string() };
    let data = unsafe { std::slice::from_raw_parts(data_ptr, data_len) }.to_vec();
    let cache = unsafe { &*(cache_ptr as *const Cache) };
    let msg = CachedMsg { dest, data };
    let _ = cache.put(&msg);
}

// More FFI functions can be added as needed, e.g., send_bundle, receive_bundle, etc.