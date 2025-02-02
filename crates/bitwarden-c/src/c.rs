use std::{ffi::CStr, os::raw::c_char, str};

use crate::{box_ptr, ffi_ref};
use bitwarden_json::client::Client;

#[no_mangle]
#[tokio::main]
pub async extern "C" fn run_command(
    c_str_ptr: *const c_char,
    client_ptr: *mut Client,
) -> *mut c_char {
    let client = unsafe { ffi_ref!(client_ptr) };
    let input_str = str::from_utf8(unsafe { CStr::from_ptr(c_str_ptr).to_bytes() }).unwrap();
    println!("{}", input_str);

    let result = client.run_command(input_str).await;
    return match std::ffi::CString::new(result) {
        Ok(cstr) => cstr.into_raw(),
        Err(_) => panic!("failed to return command result: null encountered"),
    };
}

// Init client, potential leak! You need to call free_mem after this!
#[no_mangle]
pub extern "C" fn init(c_str_ptr: *const c_char) -> *mut Client {
    env_logger::init();
    if c_str_ptr.is_null() {
        return box_ptr!(Client::new(None));
    } else {
        let input_string = str::from_utf8(unsafe { CStr::from_ptr(c_str_ptr).to_bytes() })
            .unwrap()
            .to_owned();
        return box_ptr!(Client::new(Some(input_string)));
    }
}

// Free mem
#[no_mangle]
pub extern "C" fn free_mem(client_ptr: *mut Client) {
    std::mem::drop(unsafe { Box::from_raw(client_ptr) });
}
