extern crate guile_rs_sys;
extern crate libc;

use self::guile_rs_sys::*;
use std::ptr;
use std::ffi::CString;
use std::mem::{transmute};
use std::sync::Mutex;

use scm::Scm;
use scm::{Untyped, TypeSpec, Symbol, List};

#[macro_export]
macro_rules! scm_eval {
    {$($tts:tt)*} => {
        {
            $crate::interp::Guile::eval(stringify!(
                $($tts)*
            ))
        }
    }
}

pub struct Guile {}

// Used in call_with_guile to work around an
// initialization bug in guile.
static mut GUILE_INIT: bool = false;

impl Guile {
    unsafe extern "C" fn proxy_guile_function<F: FnOnce(A)->R, A, R>(data: *mut libc::c_void) -> *mut libc::c_void {
        // read is highly unsafe!
        let data: (F, A) = ptr::read(transmute(data));
        let (fun, args) = data;

        let ret: *mut R = &mut (fun)(args);
        transmute::<*mut R, *mut libc::c_void>(ret)
    }


    pub fn call_with_guile<F: FnOnce(A)->R, A, R>(fun: F, args: A) -> R {

        // Due to a bug with guile initialization, guile must be
        // fully initialized in one thread before another thread
        // do the same. Otherwise a segfault occurs.
        //
        // To work around this, we are using a mutex to lock only
        // on the first call to this function, after which the
        // GUILE_INIT variable prevents that.
        lazy_static! {
            static ref INIT_LOCK: Mutex<usize> = Mutex::new(0);
        }

        let _lock;
        unsafe {

            if !GUILE_INIT {
                _lock = INIT_LOCK.lock().unwrap();
            }


            let args_pt: *mut (F, A) = &mut (fun, args);
            let args_pt = args_pt as *mut libc::c_void;

            let ret = scm_with_guile(Some(Self::proxy_guile_function::<F, A, R>), args_pt);


            GUILE_INIT = true;

            // read = very unsafe!
            ptr::read(transmute::<*mut libc::c_void, *mut R>(ret))
        }
    }

    unsafe extern "C" fn catch_handler(data: *mut libc::c_void, key: SCM, args: SCM) -> SCM {
        let data: *mut (bool, SCM, SCM) = transmute(data);
        (*data).0 = true;
        (*data).1 = key;
        (*data).2 = args;
        // let flag: *mut bool = transmute::<*mut libc::c_void, *mut bool>(data);
        // *flag = true;

        gu_SCM_UNDEFINED()
    }

    unsafe extern "C" fn caught_body<F: FnOnce(A)->Scm<TS>, A, TS: TypeSpec>(data: *mut libc::c_void) -> SCM {
        let data: (F, A) = ptr::read(transmute(data));
        let (fun, args) = data;

        (fun)(args).into_raw()
    }

    fn _call_with_catch<TS: TypeSpec, RT: TypeSpec, F: FnOnce(A)->Scm<RT>, A>
        (key: Scm<TS>, body: F, body_args: A) -> Result<Scm<RT>, (Scm<Symbol>, Scm<List>)> {
            assert!(key.is_true() || key.is_symbol());

            unsafe {
                let body_args_pt: *mut (F, A) = &mut (body, body_args);
                let body_args_pt = body_args_pt as *mut libc::c_void;

                let mut err_data: (bool, SCM, SCM) = (false, gu_SCM_UNDEFINED(), gu_SCM_UNDEFINED());
                let err_data_ptr: *mut (bool, SCM, SCM) = &mut err_data;
                let err_data_ptr: *mut libc::c_void = transmute(err_data_ptr);

                let ret: SCM = scm_internal_catch(key.data,
                                                  Some(Self::caught_body::<F, A, RT>), // lol
                                                  body_args_pt,
                                                  Some(Self::catch_handler),
                                                  err_data_ptr);
                if err_data.0 {
                    // NOTE: not sure if always these types...
                    // instead of assuming, we check before rewraping for now
                    // NOTE: not sure if taking the key and args out of the scope of the handler is
                    // a good idea...
                    Err((Scm::<Untyped>::from_raw(err_data.1).into_symbol().unwrap(),
                         Scm::<Untyped>::from_raw(err_data.2).into_list().unwrap()))
                } else {
                    Ok(Scm::<RT>::_from_raw(ret))
                }
            }
    }

    pub fn call_with_catch<RT: TypeSpec, F: FnOnce(A)->Scm<RT>, A>(key: Scm<Symbol>, body: F, body_args: A) -> Result<Scm<RT>, (Scm<Symbol>, Scm<List>)> {
        Self::_call_with_catch(key, body, body_args)
    }

    pub fn call_with_catch_all<RT: TypeSpec, F: FnOnce(A)->Scm<RT>, A>(body: F, body_args: A) -> Result<Scm<RT>, (Scm<Symbol>, Scm<List>)> {
        Self::_call_with_catch(Scm::true_c(), body, body_args)
    }

    pub fn eval(s: &str) -> Scm<Untyped> {
        let raw = unsafe {
            scm_c_eval_string(CString::new(s).unwrap().as_ptr())
        };

        Scm::<Untyped>::from_raw(raw)
    }
}
