extern crate guile_rs_sys;
extern crate libc;

#[cfg(test)]
#[macro_use]
extern crate lazy_static;

#[macro_use]
mod utils;

pub mod scm;
pub mod interp;

pub use scm::{Scm, UnspecifiedSpec, NumericSpec, BoolSpec, StringSpec, IntSpec, TryAs};
pub use interp::Guile;


#[cfg(test)]
mod tests {

    pub use scm::{
        Scm,
        TypeSpec,
        UnspecifiedSpec,
        ForeignTypeSpec,
        ForeignObjectSpec,
        ForeignType,
        NumericSpec,
        BoolSpec,
        StringSpec,
        IntSpec,
        TryAs
    };
    pub use interp::Guile;

    use std::thread;

    #[test]
    pub fn guile_test() {
        let _ = Guile::call_with_guile(|_| {
            // Guile::eval("(display \"testing...\")");

            // DSL using stringify!() macro with eval
            let s1: Scm<UnspecifiedSpec> = Guile::eval(stringify!(

                    (display "test display...\n")
                    "test string..."

                ));
            let s1: Scm<StringSpec>      = s1.into_string().unwrap();
            let s: String                = s1.to_string();
            assert_eq!(s, "test string...");

            let s2: Scm<StringSpec>      = Scm::from("test string...");
            let s: String                = s2.to_string();
            assert_eq!(s, "test string...");

            assert!(s1.equal_p(&s2).is_true());

            // let s = "string123".to_owned();
            // assert!(Guile::eval(&format!("\"{}\"", s)) == Scm::<StringSpec>::from_str(&s));

            assert!(Scm::true_c().is_bool());   // is boolean scheme type
            assert!(Scm::true_c().is_true());   // is true ...
            assert!(Scm::false_c().is_false()); // ...
            assert!(Scm::true_c().to_bool());   // as rust boolean type

            let v: Scm<IntSpec> = Scm::from(12345);
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

            // Operations on numerics produce unspecified numeric type (NumericSpec)
            let r: Scm<NumericSpec> = Scm::from(9) + Scm::from(8) * Scm::from(90) / (Scm::from(123) - Scm::from(113));
            let rr = 9 + 8 * 90 / (123 - 113);
            assert!(Scm::from(rr) == r);

            let r = Scm::from(9);
            assert!(r == Scm::from(9));
            assert!(r.oneplus() == Scm::from(10));
            assert!(r == Scm::from(9));

            struct TestStruct {
                val1: u8
            }

            lazy_static! {
                static ref FTYPE: Scm<ForeignTypeSpec> = {
                    Guile::call_with_guile(|_| {
                        Scm::new_type(/* args here... */)
                    }, ())
                };
            }

            struct TestType { }
            impl ForeignType for TestType {
                type Struct = TestStruct;
                fn get_type<'a>() -> &'a Scm<ForeignTypeSpec> { &FTYPE }
                fn as_struct_mut<'a>() -> &'a mut Self::Struct {
                    &mut TestStruct { val1: 7 }
                }
                fn as_struct<'a>() -> &'a Self::Struct {
                    // This should actually pull it from the SCM data...

                    // Dummy:
                    &TestStruct { val1: 7 }
                }
            }

            type TestTypeSpec = ForeignObjectSpec<TestType>;

            let st: Scm<TestTypeSpec>
                = Scm::from_struct(TestStruct { val1: 21 });


        }, ());

        let _ = Guile::call_with_guile(|_| {
            Guile::eval("(define h (make-hash-table 32))");
            Guile::eval(r#"(hashq-set! h 'foo "bar")"#);
        }, ());

    }

    #[test]
    pub fn scope_test() {
    }
}



