extern crate guile_rs_sys;
extern crate libc;

#[macro_use]
extern crate proc_macro_hack;
#[allow(unused_imports)]
#[macro_use]
extern crate internal_proc_macros;

#[macro_use]
extern crate lazy_static;

#[macro_use]
mod utils;

#[macro_use]
pub mod scm;
#[macro_use]
pub mod interp;

pub use scm::{Scm, TryAs};
pub use scm::ForeignSpec;
pub use scm::{Untyped, Numeric, Bool, Int, Foreign};
pub use scm::String as ScmString;
pub use interp::Guile;


#[cfg(test)]
#[allow(unused_imports)]
mod tests {

    // pub use scm::{
    //     Scm,
    //     TypeSpec,
    //     Untyped,
    //     Foreign,
    //     ForeignObject,
    //     ForeignSpec,
    //     Numeric,
    //     Bool,
    //     ScmString,
    //     Symbol,
    //     Int,
    //     TryAs
    // };
    pub use scm::*;
    pub use std::string::String;
    pub use interp::Guile;
    pub use guile_rs_sys::{SCM, scm_t_subr, scm_c_define_gsubr};

    use std;
    use std::thread;
    use std::marker::PhantomData;
    use std::ffi::CString;

    #[test]
    pub fn guile_test() {
        let _ = Guile::call_with_guile(|_| {
            // Guile::eval("(display \"testing...\")");

            // DSL using stringify!() macro with eval
            let s1: Scm<Untyped> = Guile::eval(stringify!(

                    (display "test display...\n")
                    "test string..."

                ));
            let s1: Scm<ScmString> = s1.into_string().unwrap();
            let s: String = s1.to_string();
            assert_eq!(s, "test string...");

            let s2: Scm<ScmString>      = Scm::from("test string...");
            let s: String                = s2.to_string();
            assert_eq!(s, "test string...");

            assert!(s1.equal_p(&s2).is_true());

            // let s = "string123".to_owned();
            // assert!(Guile::eval(&format!("\"{}\"", s)) == Scm::<ScmString>::from_str(&s));

            assert!(Scm::true_c().is_bool());   // is boolean scheme type
            assert!(Scm::true_c().is_true());   // is true ...
            assert!(Scm::false_c().is_false()); // ...
            assert!(Scm::true_c().to_bool());   // as rust boolean type

            let v: Scm<Int> = Scm::from(12345);
            assert!(v.is_number());
            assert!(v.is_exact_integer());
            assert!(v.is_exact());
            assert!(v.exact_p().is_true());
            assert!(!v.is_inexact());

            let n: i32 = v.try_as().unwrap();
            assert_eq!(n, 12345);

            // // should fail:
            // let n: i8 = v.try_as().unwrap();

            // s^2 + r = k
            let (s, r) = Scm::from(10).exact_integer_sqrt().unwrap();
            let (s, r): (i32, i32) = (s.try_as().unwrap(), r.try_as().unwrap());
            assert_eq!(s, 3);
            assert_eq!(r, 1);

            assert!(Scm::from(90) == Scm::from(90));
            assert!(!Scm::from(123).is_zero());
            assert!(Scm::from(0).is_zero());

            assert!(Scm::from(123) >  Scm::from(90));
            assert!(Scm::from(123) >= Scm::from(90));
            assert!(Scm::from(90)  >= Scm::from(90));

            // Operations on numerics produce unspecified numeric type (Numeric)
            let r: Scm<Numeric> = Scm::from(9) + Scm::from(8) * Scm::from(90) / (Scm::from(123) - Scm::from(113));
            let rr = 9 + 8 * 90 / (123 - 113);
            assert!(Scm::from(rr) == r);

            let r = Scm::from(9);
            assert!(r == Scm::from(9));
            assert!(r.oneplus() == Scm::from(10));
            assert!(r == Scm::from(9));

            assert!(Guile::call_with_catch("test".into(), |_| {
                scm_eval!{ (throw 'test) }
            }, ()).is_err());

            assert!(Guile::call_with_catch_all(|_| {
                scm_eval!{ (throw 'any) }
            }, ()).is_err());

            assert!(Guile::call_with_catch_all(|_| {
                scm_eval!{ "test" }
            }, ()).unwrap().equal_p(&Scm::<ScmString>::from("test")).is_true());


        }, ());

        let _ = Guile::call_with_guile(|_| {
            Guile::eval("(define h (make-hash-table 32))");
            Guile::eval(r#"(hashq-set! h 'foo "bar")"#);
        }, ());
    }

    #[test]
    pub fn foreign_test() {
        #[allow(dead_code)]
        #[derive(Debug)]
        struct TestStruct {
            data0: u8
        }


        lazy_static! {
            // static ref FTYPE: Scm<Foreign> = {
            //     Guile::call_with_guile(|_| {
            //         Scm::new_type(&"Test".into(), &vec![Scm::<ScmString>::from("val1")].into(), type_list![TestStruct])
            //     }, ())
            // };
            // static ref FSLOTS: Box<TypeList> = type_list![TestStruct];
            //
            static ref FTYPE: Scm<Foreign<TestStruct>> = {
                Guile::call_with_guile(|_| {
                    Scm::new_type(&"Test".into())
                }, ())
            };
        }

        // struct TestType {}
        // impl ForeignSpec for TestType {
        //     // type SlotTypes = type_list![TestStruct];

        unsafe extern "C" fn test_data(fo: SCM) -> SCM {
            let st = Scm::<Untyped>::from_raw(fo).into_foreign(&*FTYPE).unwrap();
            assert!(st.is_foreign(&*FTYPE));

            if st.get_data().unwrap().data0 == 32 {
                Scm::true_c()
            } else {
                Scm::false_c()
            }.into_raw()
        }

        unsafe extern "C" fn get_foreign_o(n: SCM) -> SCM {
            let n: u8 = Scm::<Untyped>::from_raw(n).into_integer().unwrap().try_as().unwrap();

            let st = Scm::<ForeignObject<TestStruct>>::new(&*FTYPE, TestStruct { data0: n });

            st.into_raw()
        }

        let st = Scm::<ForeignObject<TestStruct>>::new(&*FTYPE, TestStruct { data0: 21 });


        let _ = Guile::call_with_guile(|_| {
            unsafe {
                let _ = scm_c_define_gsubr(
                    CString::new("test-data").unwrap().as_ptr(),
                    1, 0, 0,
                    test_data as scm_t_subr
                    );

                let _ = scm_c_define_gsubr(
                    CString::new("get-foreign-o").unwrap().as_ptr(),
                    1, 0, 0,
                    get_foreign_o as scm_t_subr
                    );
            }

            assert!(st.is_foreign(&*FTYPE));
            assert_eq!(st.get_data().unwrap().data0, 21u8);

            let res: Scm<Untyped> = Guile::eval("

                    (test-data (get-foreign-o 32))

                ");

            assert!(res.is_true())

        }, ());

        //     // fn get_type<'a>() -> &'a Scm<Foreign> { &FTYPE }
        //     // fn get_slot_types() -> Box<TypeList> {
        //     //     // &FTYPE.get_slot_types()
        //     //     // Box clone clones the boxes contents
        //     //     FSLOTS.clone()
        //     // }
        //     // fn as_struct_mut<'a>() -> &'a mut Self::Struct {
        //     //     unimplemented!()
        //     //     // &mut TestStruct { val1: 7 }
        //     // }
        //     // fn as_struct<'a>() -> &'a Self::Struct {
        //     //     // This should actually pull it from the SCM data...

        //     //     // Dummy:
        //     //     &TestStruct { val1: 7 }
        //     // }
        // }

        //type TestTypeSpec = ForeignObject<TestType>;

        // NOTE: this commented test makes no sense anymore
        // let st: Scm<TestTypeSpec>
        //     = Scm::from_struct(TestStruct { val1: 21 });

        // let _ = Guile::call_with_guile(|_| {}, ());

    }

    #[test]
    pub fn scope_test() {

    }
}
