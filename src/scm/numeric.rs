extern crate guile_rs_sys;

use scm::*;

// use self::guile_rs_sys::*;
use std::ptr;
use std::cmp::Ordering;
use std::ops::{Add, Sub, Mul, Div};
use std::ops::{BitAnd, BitOr, BitXor};

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
    // NOTE: why am I using the scm_func! macro here?
    scm_func!(into_string(radix: &Scm<IntSpec>) -> Scm<StringSpec>, scm_number_to_string);

    // Arithmetic
    scm_func!(P sum(other: T<Numeric>) -> Scm<NumericSpec>, scm_sum);
    scm_func!(P difference(other: T<Numeric>) -> Scm<NumericSpec>, scm_difference);
    scm_func!(P product(other: T<Numeric>) -> Scm<NumericSpec>, scm_product);
    scm_func!(P divide(other: T<Numeric>) -> Scm<NumericSpec>, scm_divide);

    scm_func!(P oneplus() -> Scm<NumericSpec>, scm_oneplus);
    scm_func!(P oneminus() -> Scm<NumericSpec>, scm_oneminus);

    scm_func!(P abs() -> Scm<NumericSpec>, scm_abs);

    scm_func!(P max(other: T<Numeric>) -> Scm<NumericSpec>, scm_max);
    scm_func!(P min(other: T<Numeric>) -> Scm<NumericSpec>, scm_min);

    scm_func!(P truncate() -> Scm<NumericSpec>, scm_truncate_number);
    scm_func!(P round() -> Scm<NumericSpec>, scm_round_number);
    scm_func!(P floor() -> Scm<NumericSpec>, scm_floor);
    scm_func!(P ceiling() -> Scm<NumericSpec>, scm_ceiling);
    // TODO: impl other arithmetic functions (more complicated ones)
    // TODO: impl scientific funcions


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

impl_op!(T IntSpec |BitAnd:bitand:scm_logand| IntSpec => IntSpec );
impl_op!(T IntSpec |BitOr :bitor :scm_logior| IntSpec => IntSpec );
impl_op!(T IntSpec |BitXor:bitxor:scm_logxor| IntSpec => IntSpec );


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

simple_try_as!(Scm<IntSpec>, scm_to_int8, i8);
simple_try_as!(Scm<IntSpec>, scm_to_uint8, u8);
simple_try_as!(Scm<IntSpec>, scm_to_int16, i16);
simple_try_as!(Scm<IntSpec>, scm_to_uint16, u16);
simple_try_as!(Scm<IntSpec>, scm_to_int32, i32);
simple_try_as!(Scm<IntSpec>, scm_to_uint32, u32);
simple_try_as!(Scm<IntSpec>, scm_to_int64, i64);
simple_try_as!(Scm<IntSpec>, scm_to_uint64, u64);
