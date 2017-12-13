#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

extern crate libc;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// extern "C" {
//     #[no_mangle]
//     static gu_SCM_BOOL_T: SCM;

//     #[no_mangle]
//     static gu_SCM_BOOL_F: SCM;
// }

#[test]
fn test1() {
    unsafe {
        test_func();
    }
}
