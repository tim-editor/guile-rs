extern crate guile_rs_sys;
extern crate libc;

use self::guile_rs_sys::*;
use std::ptr;
use std::ffi::CString;
use std::mem::{transmute};

use scm::Scm;
use scm::{UnspecifiedSpec};

pub struct Guile {
}

impl Guile {
    unsafe extern "C" fn proxy_guile_function<F: FnOnce(A)->R, A, R>(data: *mut libc::c_void) -> *mut libc::c_void {
        // read is highly unsafe!
        let data: (F, A) = ptr::read(transmute(data));
        let (fun, args) = data;

        let ret: *mut R = &mut (fun)(args);
        transmute::<*mut R, *mut libc::c_void>(ret)
    }

    pub fn call_with_guile<F: FnOnce(A)->R, A, R>(fun: F, args: A) -> R {
        unsafe {
            let args_pt: *mut (F, A) = &mut (fun, args);
            let args_pt = args_pt as *mut libc::c_void;

            let ret = scm_with_guile(Some(Self::proxy_guile_function::<F, A, R>), args_pt);

            // read = very unsafe!
            ptr::read(transmute::<*mut libc::c_void, *mut R>(ret))
        }
    }

    pub fn eval(s: &str) -> Scm<UnspecifiedSpec> {
        let raw = unsafe {
            scm_c_eval_string(CString::new(s).unwrap().as_ptr())
        };
        Scm::<UnspecifiedSpec>::from_raw(raw)
    }
}

