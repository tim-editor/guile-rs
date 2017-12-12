extern crate guile_sys;
extern crate libc;

pub mod scm;
pub mod interp;

pub use scm::{Scm, UnspecifiedSpec, NumericSpec, BoolSpec, StringSpec, IntSpec, TryAs};
pub use interp::Guile;


mod tests {

    pub use scm::{Scm, UnspecifiedSpec, NumericSpec, BoolSpec, StringSpec, IntSpec, TryAs};
    pub use interp::Guile;

    #[test]
    pub fn guile_test() {
        let _ = Guile::call_with_guile(|_| {
            // Guile::eval("(display \"testing...\")");

            let s: Scm<UnspecifiedSpec> = Guile::eval("\"test string...\"");
            let s: Scm<StringSpec>      = s.into_string().unwrap();
            let s: String               = s.to_string();
            assert_eq!(s, "test string...");

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




        }, ());
    }
}



