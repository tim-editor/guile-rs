
//! TypeSpecs and Scm struct for handling guile values
//!
//! # Example
//! ```rust,ignore
//! let s: Scm<UnspecifiedSpec> = Guile::eval("\"test string...\"");
//! let s: Scm<StringSpec>      = s.into_string().unwrap();
//! let s: String               = s.to_string();
//! assert_eq!(s, "test string...");
//! ```
//!
//! # Numeric Operations
//!
//! Operations on numerics produce unspecified numeric type ([NumericSpec](struct.NumericSpec.html))
//!
//! ```rust,ignore
//! let r: Scm<NumericSpec> = Scm::from(9) + Scm::from(8)
//! let r: Scm<NumericSpec> = r * Scm::from(90)
//! let r: Scm<NumericSpec> = r / (Scm::from(123) - Scm::from(113));
//!
//! let rr = 9 + 8
//! let rr = rr * 90
//! let rr = rr / (123 - 113);
//!
//! assert!(Scm::from(rr) == r);
//! ```


extern crate guile_rs_sys;

use self::guile_rs_sys::*;
use std::ffi::CString;
use std::marker::PhantomData;
use std::ptr;
use std::cmp::Ordering;
use std::ops::{Add, Sub, Mul, Div};
// use std::os::raw::*;
// use self::libc;



pub trait TypeSpec {}
pub trait Numeric: TypeSpec {}

// Spec system inspired by RustAudio/rust-jack (github)
pub struct UnspecifiedSpec;
impl TypeSpec for UnspecifiedSpec {}

pub struct BoolSpec;
impl TypeSpec for BoolSpec {}

// === NUMERICS ===
// For numerics of unknown numeric type
/// See [spec implementation](struct.Scm.html#impl-4)
pub struct NumericSpec;
impl TypeSpec for NumericSpec {}
impl Numeric for NumericSpec {}

/// See [spec implementation](struct.Scm.html#impl-5)
pub struct IntSpec;
impl TypeSpec for IntSpec {}
impl Numeric for IntSpec {}

pub struct RationalSpec;
impl TypeSpec for RationalSpec {}
impl Numeric for RationalSpec {}

pub struct RealSpec;
impl TypeSpec for RealSpec {}
impl Numeric for RealSpec {}

pub struct ComplexSpec;
impl TypeSpec for ComplexSpec {}
impl Numeric for ComplexSpec {}
// === \\\ ===

pub struct StringSpec;
impl TypeSpec for StringSpec {}

pub struct Scm<TS: TypeSpec> {
    data: SCM,
    spec: PhantomData<TS>
}

unsafe impl<TS: TypeSpec> Send for Scm<TS> {}
unsafe impl<TS: TypeSpec> Sync for Scm<TS> {}

macro_rules! into_type {
    ($tn:ident, $spec:ident) => (into_type!(into_$tn, is_$tn, $spec););

    ($inn:ident, $isn:ident, $spec:ident) => {
        pub fn $inn(self) -> Result<Scm<$spec>, ()> {
            if self.$isn() {
                Ok(self.into_type())
            } else {
                Err(())
            }
        }
    };
}

//
// TODO: uncomment these macros and use them once
// `concat_idents` is in stable rust and is actually useful
//
// // for types like bool to work
// // ex: _is_thing_p!(bool_p);
// macro_rules! _is_thing_p {
//     ($thing:ident) => {
//         #[inline]
//         pub fn is_$thing(&self) -> Scm<BoolSpec> {
//             Scm::_from_raw(unsafe { concat_idents!(scm_, $thing)(self.data) })
//         }
//     };
// }

// macro_rules! is_thing_p {
//     ($thing:ident) => {
//         _is_thing_p!($thing_p);
//         // #[inline]
//         // pub fn is_$(thing)_p(&self) -> Scm<BoolSpec> {
//         //     Scm::_from_raw(unsafe { scm_$(thing)_p(self.data) })
//         // }
//     };
// }

// // for types like bool to work
// // ex: _is_thing!(is_bool);
// macro_rules! _is_thing {
//     ($thing:ident) => {
//         #[inline]
//         pub fn $thing(&self) -> bool {
//             unsafe { concat_idents!(scm_, $thing)(self.data) == 1 }
//         }
//     };
// }

// macro_rules! is_thing {
//     ($thing:ident) => {
//         _is_thing!(is_$thing);
//         // #[inline]
//         // pub fn is_$thing(&self) -> bool {
//         //     unsafe { scm_is_$thing(self.data) == 1 }
//         // }
//     };
// }

macro_rules! is_thing_p {
    ($fname:ident => $cfunc:ident) => {
        is_thing_p!($fname() => $cfunc);
    };
    ($fname:ident ($($an:ident: $tn:ident <$at:path>),*) => $cfunc:ident) => {
        // /// Retrun guile true value when is condition
        #[inline]
        pub fn $fname<$($tn: $at),*>(&self, $($an: &Scm<$tn>),*) -> Scm<BoolSpec> {
            Scm::_from_raw(unsafe { $cfunc(self.data, $($an.data),*) })
        }
    };
}

macro_rules! is_thing {
    ($fname:ident => $cfunc:ident) => {
        is_thing!($fname() => $cfunc);
    };
    ($fname:ident ($($an:ident: $tn:ident <$at:path>),*) => $cfunc:ident) => {
        // /// Retrun true when is condition
        #[inline]
        pub fn $fname<$($tn: $at),*>(&self, $($an: &Scm<$tn>),*) -> bool {
            unsafe { $cfunc(self.data, $($an.data),*) == 1 }
        }
    };
}

macro_rules! is_thing_manual {
    ($fname:ident => $cfunc:ident) => {
        is_thing_manual!($fname() => $cfunc);
    };
    ($fname:ident ($($an:ident: $tn:ident <$at:path>),*) => $cfunc:ident) => {
        // /// Retrun true when is condition
        #[inline]
        pub fn $fname<$($tn: $at),*>(&self, $($an: &Scm<$tn>),*) -> bool {
            unsafe { gu_scm_is_true($cfunc(self.data, $($an.data),*)) }
        }
    };
}

macro_rules! scm_func {
    ($fname:ident ($($an:ident: $at:ty),*) -> $r:ty, $cfunc:ident) => {
        #[inline]
        pub fn $fname(&self, $($an: $at),*) -> $r {
            Scm::_from_raw(unsafe { $cfunc(self.data, $($an.data),*) })
        }
    }
}

impl<TS: TypeSpec> Scm<TS> {
    fn _from_raw(data: SCM) -> Scm<TS> {
        Scm { data, spec: PhantomData }
    }

    pub fn from_raw(data: SCM) -> Scm<UnspecifiedSpec> {
        Scm::_from_raw(data)
    }

    #[inline]
    pub fn as_bits(&self) -> scm_t_bits {
        unsafe { gu_SCM_UNPACK(self.data) }
        // unsafe { transmute::<SCM, scm_t_bits>(self.data) }
    }

    #[inline]
    pub fn is_true(&self) -> bool { unsafe { gu_scm_is_true(self.data) } }

    #[inline]
    pub fn is_false(&self) -> bool { unsafe { gu_scm_is_false(self.data) } }

    #[inline]
    pub fn is_bool(&self) -> bool { unsafe { scm_is_bool(self.data) == 1 } }

    #[inline]
    pub fn is_string(&self) -> bool { unsafe { gu_scm_is_string(self.data) == 1 } }

    // === NUMERICS ===
    is_thing!(is_number => scm_is_number);
    is_thing!(is_integer => scm_is_integer);
    is_thing!(is_exact_integer => scm_is_exact_integer);

    is_thing_p!(number_p => scm_number_p);
    is_thing_p!(integer_p => scm_integer_p);
    is_thing_p!(exact_integer_p => scm_exact_integer_p);
    // === \\\ ===

    /// check for identity (`scm_eq_p`)
    /// scheme operation: `eq?`
    #[inline]
    pub fn eq_p<OS: TypeSpec>(&self, other: &Scm<OS>) -> Scm<BoolSpec> {
        Scm::_from_raw(unsafe { scm_eq_p(self.data, other.data) })
    }

    /// check for identity (`scm_eq_p`)
    /// scheme operation: `eq?`
    #[inline]
    pub fn is_eq<OS: TypeSpec>(&self, other: &Scm<OS>) -> bool {
        unsafe { gu_scm_is_eq(self.data, other.data) }
    }


}

// impl<TS: TypeSpec, OS: TypeSpec> PartialEq<Scm<OS>> for Scm<TS> {
//     /// runs the `scm_num_eq_p` guile function
//     fn eq(&self, other: &Scm<OS>) -> bool { unsafe { gu_scm_is_true(scm_num_eq_p(self.data, other.data)) } }
// }

// impl<TS: TypeSpec> Eq for Scm<TS> {}

impl<N: Numeric> From<Scm<N>> for Scm<StringSpec> {
    fn from(numeric: Scm<N>) -> Scm<StringSpec> {
        Self {
            data: unsafe { scm_number_to_string(numeric.data, ptr::null_mut()) },
            spec: PhantomData,
        }
    }
}

macro_rules! simple_from {
    ($from:ty, $cfunc: ident, $to:ty) => {
        impl From<$from> for $to {
            fn from(f: $from) -> $to {
                Self {
                    data: unsafe { $cfunc(f) },
                    spec: PhantomData,
                }
            }
        }
    };
}

simple_from!(i8, scm_from_int8, Scm<IntSpec>);
simple_from!(u8, scm_from_uint8, Scm<IntSpec>);
simple_from!(i16, scm_from_int16, Scm<IntSpec>);
simple_from!(u16, scm_from_uint16, Scm<IntSpec>);
simple_from!(i32, scm_from_int32, Scm<IntSpec>);
simple_from!(u32, scm_from_uint32, Scm<IntSpec>);
simple_from!(i64, scm_from_int64, Scm<IntSpec>);
simple_from!(u64, scm_from_uint64, Scm<IntSpec>);
// simple_from!(scm_t_intmax, gu_scm_from_intmax, Scm<IntSpec>);
// simple_from!(scm_t_uintmax, gu_scm_from_uintmax, Scm<IntSpec>);

pub trait TryAs<T, E> {
    /// attemp to get `&self` as type `T`
    fn try_as(&self) -> Result<T, E>;
}

macro_rules! simple_try_as {
    ($from:ty, $cfunc:ident, $to:ty) => {
        impl TryAs<$to, ()> for $from {
            fn try_as(&self) -> Result<$to, ()> {
                if self.is_exact_integer() {
                    // TODO: handle runtime guile errors
                    // TODO: handle guile int not fitting target type
                    Ok(unsafe { $cfunc(self.data) })
                } else {
                    Err(())
                }
            }
        }
    }
}

simple_try_as!(Scm<IntSpec>, scm_to_int8, i8);
simple_try_as!(Scm<IntSpec>, scm_to_uint8, u8);
simple_try_as!(Scm<IntSpec>, scm_to_int16, i16);
simple_try_as!(Scm<IntSpec>, scm_to_uint16, u16);
simple_try_as!(Scm<IntSpec>, scm_to_int32, i32);
simple_try_as!(Scm<IntSpec>, scm_to_uint32, u32);
simple_try_as!(Scm<IntSpec>, scm_to_int64, i64);
simple_try_as!(Scm<IntSpec>, scm_to_uint64, u64);

impl Scm<UnspecifiedSpec> {
    // Do not use this without checking for type first
    fn into_type<S: TypeSpec>(self) -> Scm<S> {
        Scm::_from_raw(self.data)
    }

    into_type!(into_bool, is_bool, BoolSpec);
    into_type!(into_string, is_string, StringSpec);
    into_type!(into_integer, is_integer, IntSpec);
}

impl Scm<BoolSpec> {
    /// Return a true litteral Scm object
    #[inline]
    pub fn true_c() -> Scm<BoolSpec> {
        Scm::_from_raw(unsafe { gu_SCM_BOOL_T() })
        // Scm { data: unsafe { gu_SCM_BOOL_T() } , spec: PhantomData }
    }

    /// Return a false litteral Scm object
    #[inline]
    pub fn false_c() -> Scm<BoolSpec> {
        Scm::_from_raw(unsafe { gu_SCM_BOOL_F() })
        // Scm { data: unsafe { gu_SCM_BOOL_F() }, spec: PhantomData }
    }

    /// to rust boolean
    /// use is_true() for testing trueness
    pub fn to_bool(&self) -> bool {
        unsafe {
            scm_to_bool(self.data) == 1
        }
    }

}

impl Scm<StringSpec> {
    /// Scm<StringSpec> from an &str (utf8)
    pub fn from_str(s: &str) -> Scm<StringSpec> {
        Scm {
            data: unsafe { scm_from_utf8_string(CString::new(s).unwrap().as_ptr()) },
            spec: PhantomData,
        }
    }

    /// to utf8 string
    pub fn to_string(&self) -> String {
        unsafe {
            CString::from_raw(scm_to_utf8_string(self.data)).into_string().unwrap()
        }
    }
}

impl<TS: Numeric> Scm<TS> {
    is_thing_p!(exact_p => scm_exact_p);
    is_thing!(is_exact => scm_is_exact);

    is_thing_p!(inexact_p => scm_inexact_p);
    is_thing!(is_inexact => scm_is_inexact);

    // Comparison Predicates (numeric)
    is_thing_manual!(is_num_eq(other: T<Numeric>) => scm_num_eq_p);
    is_thing_manual!(is_less(other: T<Numeric>) => scm_less_p);
    is_thing_manual!(is_gr(other: T<Numeric>) => scm_gr_p);
    is_thing_manual!(is_leq(other: T<Numeric>) => scm_leq_p);
    is_thing_manual!(is_geq(other: T<Numeric>) => scm_geq_p);
    is_thing_manual!(is_zero => scm_zero_p);
    is_thing_manual!(is_positive => scm_positive_p);
    is_thing_manual!(is_negative => scm_negative_p);

    is_thing_p!(num_eq_p(other: T<Numeric>) => scm_num_eq_p);
    is_thing_p!(less_p(other: T<Numeric>) => scm_less_p);
    is_thing_p!(gr_p(other: T<Numeric>) => scm_gr_p);
    is_thing_p!(leq_p(other: T<Numeric>) => scm_leq_p);
    is_thing_p!(geq_p(other: T<Numeric>) => scm_geq_p);
    is_thing_p!(zero_p => scm_zero_p);
    is_thing_p!(positive_p => scm_positive_p);
    is_thing_p!(negative_p => scm_negative_p);

    // Conversion
    scm_func!(into_string(radix: Scm<IntSpec>) -> Scm<StringSpec>, scm_number_to_string);
}

impl<TS: Numeric, OS: Numeric> PartialEq<Scm<OS>> for Scm<TS> {
    fn eq(&self, other: &Scm<OS>) -> bool { self.is_num_eq(other) }
}
impl<TS: Numeric> Eq for Scm<TS> {}

impl<TS: Numeric, OS: Numeric> PartialOrd<Scm<OS>> for Scm<TS> {
    fn partial_cmp(&self, other: &Scm<OS>) -> Option<Ordering> {
        if self.is_less(other) {
            Some(Ordering::Less)
        } else if self.is_num_eq(other) {
            Some(Ordering::Equal)
        } else if self.is_gr(other) {
            Some(Ordering::Greater)
        } else {
            None
        }
    }
    fn le(&self, other: &Scm<OS>) -> bool { self.is_leq(other) }
    fn ge(&self, other: &Scm<OS>) -> bool { self.is_geq(other) }
}

macro_rules! impl_op {
    (P $lhs:path |$op:ident:$func:ident:$cfunc:ident| $rhs:path => $out:path) => {
        impl<LT: $lhs, RT: $rhs> $op<Scm<RT>> for Scm<LT> {
            type Output = Scm<$out>;

            fn $func(self, other: Scm<RT>) -> Scm<$out> {
                Scm::_from_raw(unsafe { $cfunc(self.data, other.data) })
            }
        }
    };
    (T $lhs:ty |$op:ident:$func:ident:$cfunc:ident| $rhs:ty => $out:ty) => {
        impl $op<Scm<$rhs>> for Scm<$lhs> {
            type Output = Scm<$out>;

            fn $func(self, other: Scm<$rhs>) -> Scm<$out> {
                Scm::_from_raw(unsafe { $cfunc(self.data, other.data) })
            }
        }
    };
}

// impl_op!(T IntSpec |Add:add:scm_sum| IntSpec => IntSpec);
// impl_op!(T IntSpec |Add:add:scm_sum| => IntSpec);

// Operations on numerics produce unspecified numeric type (NumericSpec)
impl_op!(P Numeric |Add:add:scm_sum       | Numeric => NumericSpec );
impl_op!(P Numeric |Sub:sub:scm_difference| Numeric => NumericSpec );
impl_op!(P Numeric |Mul:mul:scm_product   | Numeric => NumericSpec );
impl_op!(P Numeric |Div:div:scm_divide    | Numeric => NumericSpec );


impl Scm<IntSpec> {
    is_thing_p!(odd_p => scm_odd_p);
    is_thing_p!(even_p => scm_even_p);

    is_thing_manual!(is_even => scm_even_p);
    is_thing_manual!(is_odd => scm_odd_p);

    scm_func!(quotient(d: &Scm<IntSpec>) -> Scm<IntSpec>, scm_quotient);
    scm_func!(remainder(d: &Scm<IntSpec>) -> Scm<IntSpec>, scm_remainder);
    scm_func!(modulo(d: &Scm<IntSpec>) -> Scm<IntSpec>, scm_modulo);
    scm_func!(gcd(y: &Scm<IntSpec>) -> Scm<IntSpec>, scm_gcd);
    scm_func!(lcm(y: &Scm<IntSpec>) -> Scm<IntSpec>, scm_lcm);

    scm_func!(modulo_expt(k: &Scm<IntSpec>, m: &Scm<IntSpec>) -> Scm<IntSpec>, scm_modulo_expt);

    pub fn exact_integer_sqrt(&self) -> Result<(Scm<IntSpec>, Scm<IntSpec>), ()> {
        if self.is_exact_integer() && self.is_positive() {
            Ok(unsafe {
                let mut s: SCM = ptr::null_mut();
                let mut r: SCM = ptr::null_mut();
                scm_exact_integer_sqrt(self.data, &mut s, &mut r);
                let s: Scm<IntSpec> = Scm::_from_raw(s);
                let r: Scm<IntSpec> = Scm::_from_raw(r);
                (s, r)
            })
        } else {
            Err(())
        }
    }
}

