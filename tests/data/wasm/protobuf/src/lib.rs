use prost::Message;
use serde_json::Value;
use std::collections::BTreeMap;
use vector_wasm::{hostcall, Registration};

// Import from prost.
pub mod items {
    include!(concat!(env!("OUT_DIR"), "/messages.rs"));
}

#[no_mangle]
pub extern "C" fn init() -> *mut Registration {
    &mut Registration::transform().set_wasi(true) as *mut Registration
}

#[no_mangle]
pub extern "C" fn process(data: u64, length: u64) -> usize {
    let data = unsafe {
        std::ptr::slice_from_raw_parts_mut(data as *mut u8, length as usize)
            .as_mut()
            .unwrap()
    };
    let mut event: BTreeMap<String, Value> = serde_json::from_slice(data).unwrap();
    let proto = match event.get("message") {
        Some(value) => {
            let value_str = value.as_str().expect("Protobuf field not a str");
            let decoded = crate::items::AddressBook::decode(value_str.as_bytes()).unwrap();
            serde_json::to_string(&decoded).unwrap()
        }
        None => return 0,
    };
    event.insert("processed".into(), proto.into());
    hostcall::emit(serde_json::to_vec(&event).unwrap());
    1
}

#[no_mangle]
pub extern "C" fn shutdown() {
    ();
}

use std::alloc::{alloc_zeroed, dealloc, Layout};

#[no_mangle]
pub extern "C" fn allocate_buffer(bytes: u64) -> *mut u8 {
    unsafe {
        let data: Vec<u8> = Vec::with_capacity(bytes as usize);
        let mut boxed = data.into_boxed_slice();
        boxed.as_mut_ptr()
    }
}

#[no_mangle]
pub extern "C" fn drop_buffer(start: *mut u8, length: usize) {
    let _ = std::ptr::slice_from_raw_parts_mut(start, length);
}