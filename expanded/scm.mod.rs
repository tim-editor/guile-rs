
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


// placeholders for build.rs-expanded macros
#[allow(unused_macros)]
macro_rules! guile_defs {
    {$($e:tt)*} => {$($e)*}
}

macro_rules! guile_impl {
    {$($e:tt)*} => {$($e)*}
}


extern crate guile_rs_sys;

use self::guile_rs_sys::*;
use std::ffi::CString;
use std::marker::PhantomData;
use std::ptr;
use std::ops::Not;
use std::mem::{transmute, forget};
use std::collections::VecDeque;
// use std::ops::{BitAnd, BitOr, BitXor, Not};
// use std::os::raw::*;
use libc;

mod numeric;


pub trait TypeSpec {}
pub trait Numeric: TypeSpec {}

// Spec system inspired by RustAudio/rust-jack (github)
#[derive(Debug)]
pub struct UnspecifiedSpec;
impl TypeSpec for UnspecifiedSpec {}

#[derive(Debug)]
pub struct BoolSpec;
impl TypeSpec for BoolSpec {}

// === NUMERICS ===
// For numerics of unknown numeric type
/// See [spec implementation](struct.Scm.html#impl-4)
#[derive(Debug)]
pub struct NumericSpec;
impl TypeSpec for NumericSpec {}
impl Numeric for NumericSpec {}

/// See [spec implementation](struct.Scm.html#impl-5)
#[derive(Debug)]
pub struct IntSpec;
impl TypeSpec for IntSpec {}
impl Numeric for IntSpec {}

#[derive(Debug)]
pub struct RationalSpec;
impl TypeSpec for RationalSpec {}
impl Numeric for RationalSpec {}

#[derive(Debug)]
pub struct RealSpec;
impl TypeSpec for RealSpec {}
impl Numeric for RealSpec {}

#[derive(Debug)]
pub struct ComplexSpec;
impl TypeSpec for ComplexSpec {}
impl Numeric for ComplexSpec {}
// === \\\ ===

#[derive(Debug)]
pub struct StringSpec;
impl TypeSpec for StringSpec {}

#[derive(Debug)]
pub struct SymbolSpec;
impl TypeSpec for SymbolSpec {}

// TODO: impl keys

#[derive(Debug)]
pub struct PairSpec;
impl TypeSpec for PairSpec {}

// NOTE: should we have this? (list types are really just pair chains)
#[derive(Debug)]
pub struct ListSpec;
impl TypeSpec for ListSpec {}

// pub struct AlistSpec;
// impl TypeSpec for AlistSpec {}

#[derive(Debug)]
pub struct HashTableSpec;
impl TypeSpec for HashTableSpec {}
#[derive(Debug)]
pub struct HashQTableSpec;
impl TypeSpec for HashQTableSpec {}
#[derive(Debug)]
pub struct HashVTableSpec;
impl TypeSpec for HashVTableSpec {}
#[derive(Debug)]
pub struct HashXTableSpec;
impl TypeSpec for HashXTableSpec {}

#[derive(Debug)]
pub struct ForeignTypeSpec;
impl TypeSpec for ForeignTypeSpec {}

pub trait ForeignType {
    type Struct;
    fn get_type<'a>()       -> &'a Scm<ForeignTypeSpec>;
    fn get_slot_types()     -> Box<TypeList>;
    fn as_struct<'a>()      -> &'a Self::Struct;
    fn as_struct_mut<'a>()  -> &'a mut Self::Struct;
}

#[derive(Debug)]
pub struct ForeignObjectSpec<FT: ForeignType> { type_: PhantomData<FT> }
impl<FT: ForeignType> TypeSpec for ForeignObjectSpec<FT> {}

#[derive(Clone, Debug)]
pub struct Scm<TS: TypeSpec> {
    pub(crate) data: SCM,
    spec: PhantomData<TS>
}

unsafe impl<TS: TypeSpec> Send for Scm<TS> {}
unsafe impl<TS: TypeSpec> Sync for Scm<TS> {}


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


impl<TS: TypeSpec> Scm<TS> {
    #[inline]
    pub(crate) fn _from_raw(data: SCM) -> Scm<TS> {
        Scm { data, spec: PhantomData }
    }

    #[inline]
    pub fn from_raw(data: SCM) -> Scm<UnspecifiedSpec> {
        Scm::_from_raw(data)
    }

    #[inline]
    pub unsafe fn into_raw(self) -> SCM { self.data }

    // Do not use this without checking for type first
    fn into_type<S: TypeSpec>(self) -> Scm<S> {
        Scm::_from_raw(self.data)
    }

    #[inline]
    pub fn into_unspecified(self) -> Scm<UnspecifiedSpec> {
        Scm::into_type(self)
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

    is_thing!(is_symbol => gu_scm_is_symbol);
    is_thing!(is_pair => gu_scm_is_pair);
    is_thing_manual!(is_list => scm_list_p);
    is_thing_manual!(is_hash_table => scm_hash_table_p);

    is_thing_p!(symbol_p => scm_symbol_p);
    is_thing_p!(pair_p => scm_pair_p);
    is_thing_p!(list_p => scm_list_p);
    is_thing_p!(hash_table_p => scm_hash_table_p);

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

    is_thing_p!(equal_p(other: OS<TypeSpec>) => scm_equal_p);


}

/// A binary list of types known at compile time
/// `Box<TypeList>` should always be built from the `type_list!()` macro!
pub trait TypeList: Send + Sync {
    /// Drop the node's contents
    ///
    /// IMPORTANT: length of `v` should be equal to length of the ndoe
    unsafe fn consume_node(&self, v: VecDeque<*mut libc::c_void>);

    /// Get the length of the node
    /// 0 if node is a Nil
    fn len(&self) -> usize;

    fn cloned(&self) -> Box<TypeList>;
}

// unsafe impl Send for TypeList {}

impl Clone for Box<TypeList> {
    fn clone(&self) -> Self {
        self.cloned()
    }
}

/// A Type element from a `TypeList`
pub trait TypeElem: Send + Sync {
    unsafe fn consume(&self, v: *mut libc::c_void);
    fn cloned(&self) -> Box<TypeElem>;
}

impl Clone for Box<TypeElem> {
    fn clone(&self) -> Self {
        self.cloned()
    }
}

#[derive(Clone)]
/// An Item in the list representing the type at that position
pub struct TypeItem<T: 'static + Send + Sync>(pub PhantomData<T>);
impl<T: 'static + Send + Sync> TypeElem for TypeItem<T> {
    /// Properly drop the variable
    ///
    /// IMPORTANT: the value of the `v` parameter should be a raw pointer from a `Box<T>`
    /// where `T` is the TypeItem's `T`. (check code for clearer view of functionality)
    unsafe fn consume(&self, v: *mut libc::c_void) {
        let v: Box<T> = Box::from_raw(transmute(v));
        drop(v);
    }
    fn cloned(&self) -> Box<TypeElem> {
        Box::new(TypeItem::<T>(PhantomData))
    }
}

#[derive(Clone)]
/// Marks end of a TyepeList
pub struct Nil {}
impl TypeList for Nil {
    unsafe fn consume_node(&self, v: VecDeque<*mut libc::c_void>) {
        assert_eq!(v.len(), 0);
    }
    fn len(&self) -> usize { 0 }
    fn cloned(&self) -> Box<TypeList> { Box::new(self.clone()) }
}

#[derive(Clone)]
/// A node of the binary spine that makes the TypeList
pub struct TypePair(pub Box<TypeElem>, pub Box<TypeList>);
impl TypeList for TypePair {
    unsafe fn consume_node(&self, mut v: VecDeque<*mut libc::c_void>) {
        assert_eq!(v.len(), self.len());
        self.1.consume_node(v.split_off(1));
        assert_eq!(v.len(), 1);
        self.0.consume(v[0]);
    }
    fn len(&self) -> usize { 1 + self.1.len() }
    fn cloned(&self) -> Box<TypeList> { Box::new(self.clone()) }
}

/// Initialize a `Box<TypeList>`
#[macro_export]
macro_rules! type_list {
    [$head:ty, $($tail:ty),*] => { {
            Box::new(TypePair(Box::new(TypeItem::<$head>(PhantomData)),
                              type_list![$($tail),*]))
        } };

    [$head:ty] => { { type_list![$head,] } };

    [] => { { Box::new(Nil {}) } };
}

impl Scm<ForeignTypeSpec> {
    unsafe extern "C" fn finalizer(obj: SCM) {
        // a pointer to TypeList does not fit in a void pointer...
        // ---- reconstruct TypeList ----
        let slot_types_r: *mut Box<TypeList> = transmute(scm_foreign_object_ref(obj, 0));
        let slot_types: Box<TypeList> = ptr::read(slot_types_r);
        // ---- ----

        // ---- build VecDeque of pointers to boxed values to free ----
        let mut vals = VecDeque::new();
        for i in 1..slot_types.len()+1-1 {
            let slot_c: *mut libc::c_void = scm_foreign_object_ref(obj, i);
            vals.push_back(slot_c);
        }
        // ---- ----

        // ---- free the boxes ----
        // call this line only once!!! otherwise double-free
        slot_types.consume_node(vals);
        // ---- ----

        // drop(slot_types);
        forget(slot_types);
        let slot_types_r: Box<Box<TypeList>> = Box::from_raw(slot_types_r);
        drop(slot_types_r);

    }

    // NOTE: types in slots should probably be Boxes!!!!!
    pub fn new_type(name: &Scm<StringSpec>, slot_names: &Scm<ListSpec>, slot_types: Box<TypeList>) -> Self {

        // NOTE: This is in the wrong function, it should be in the new object initializer...
        // NOTE: we also need a way of forcing these types
        let slot_types: Box<Box<TypeList>> = Box::new(slot_types);
        // keep in mind this takes ownership of slot_types
        let slot_types_r: *mut Box<TypeList> = Box::into_raw(slot_types);


        let slot_names: Scm<ListSpec> = Scm::cons(&Scm::<StringSpec>::from("types"), &slot_names).into_list().unwrap();

        Scm::_from_raw(unsafe {
            scm_make_foreign_object_type(name.data, slot_names.data, Some(Scm::finalizer))
        })
    }
}

// NOTE: Most functions in here have the wrong ideas...
// from_struct and as_struct make no sense since what we need is rather as_struct for every slot
impl<FT: ForeignType> Scm<ForeignObjectSpec<FT>> {
    pub fn from_struct(strct: FT::Struct) -> Self {
        unimplemented!()
    }
    pub fn get_type<'a>() -> &'a Scm<ForeignTypeSpec> { FT::get_type() }
    pub fn get_slot_types() -> Box<TypeList> { FT::get_slot_types() }

    pub fn as_struct_mut<'a>() -> &'a mut FT::Struct { FT::as_struct_mut() }
    pub fn as_struct<'a>()     -> &'a     FT::Struct { FT::as_struct()     }
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


pub trait TryAs<T, E> {
    /// attemp to get `&self` as type `T`
    fn try_as(&self) -> Result<T, E>;
}

impl Scm<UnspecifiedSpec> {
    into_type!(into_bool,        is_bool,       BoolSpec);
    into_type!(into_string,      is_string,     StringSpec);
    into_type!(into_integer,     is_integer,    IntSpec);
    into_type!(into_symbol,      is_symbol,     SymbolSpec);
    into_type!(into_pair,        is_pair,       PairSpec);
    into_type!(into_list,        is_list,       ListSpec);
    into_type!(into_hash_table,  is_hash_table, HashTableSpec);
    into_type!(into_hashq_table, is_hash_table, HashQTableSpec);
    into_type!(into_hashv_table, is_hash_table, HashVTableSpec);
    into_type!(into_hashx_table, is_hash_table, HashXTableSpec);
    // pub fn into_foreign_object<FT: ForeignType>(self) -> Scm<ForeignObjectSpec<FT>> {

    // }
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

impl Not for Scm<BoolSpec> {
    type Output = Scm<BoolSpec>;
    fn not(self) -> Scm<BoolSpec> {
        Scm::_from_raw(unsafe { scm_not(self.data) })
    }
}

guile_impl!(Scm<StringSpec> {
    pub fn from_str(@_, &str)
        => scm_from_utf8_string(CString::new(@0).unwrap().as_ptr())
        -> @r Scm<StringSpec>

    /// to utf8 string
    pub fn to_string(&self) -> String {
        unsafe {
            CString::from_raw(scm_to_utf8_string(self.data)).into_string().unwrap()
        }
    }

    pub fn into_symbol(self) -> Scm<SymbolSpec> {
        Scm::_from_raw(unsafe { scm_string_to_symbol(self.data) })
    }
});

impl<'a> From<&'a str> for Scm<StringSpec> {
    #[inline]
    fn from(s: &'a str) -> Scm<StringSpec> {
        Scm::<StringSpec>::from_str(s)
    }
}

impl From<String> for Scm<StringSpec> {
    #[inline]
    fn from(s: String) -> Scm<StringSpec> {
        Scm::<StringSpec>::from_str(&s)
    }
}

guile_impl!(Scm<SymbolSpec> {
    pub fn from_str(@_, &str)
        => scm_from_utf8_symbol(CString::new(@0).unwrap().as_ptr())
        -> @r Scm<SymbolSpec>
    // pub fn from_str(s: &str) -> Scm<SymbolSpec> {
    //     Scm::_from_raw(unsafe { scm_from_utf8_symbol(CString::new(s).unwrap().as_ptr()) })
    // }

    pub fn into_string(self)
        => scm_symbol_to_string(@s)
        -> @r Scm<StringSpec>

    // pub fn into_string(self) -> Scm<StringSpec> {
    //     Scm::_from_raw(unsafe { scm_symbol_to_string(self.data) })
    // }
});

impl<'a> From<&'a str> for Scm<SymbolSpec> {
    #[inline]
    fn from(s: &'a str) -> Scm<SymbolSpec> {
        Scm::<SymbolSpec>::from_str(s)
    }
}

// impl Scm<PairSpec> {
//     scm_func!(car() -> Scm<UnspecifiedSpec>, gu_scm_car);
//     scm_func!(cdr() -> Scm<UnspecifiedSpec>, gu_scm_cdr);

//     scm_func!(P set_car(value: T<TypeSpec>), scm_set_car_x);
//     scm_func!(P set_cdr(value: T<TypeSpec>), scm_set_cdr_x);
// }

guile_impl!(Scm<PairSpec> {
    pub fn car() => gu_scm_car(@s) -> @r Scm<UnspecifiedSpec>
    pub fn cdr() => gu_scm_cdr(@s) -> @r Scm<UnspecifiedSpec>
    pub fn cons(@_, &Scm<A>|A:TypeSpec, &Scm<B>|B:TypeSpec) => gu_scm_cons(@*#) -> @r Scm<PairSpec>

    pub fn set_car(Scm<T>|T:TypeSpec) => scm_set_car_x(@s, @*#) -> @r Scm<UnspecifiedSpec>
    pub fn set_cdr(Scm<T>|T:TypeSpec) => scm_set_cdr_x(@s, @*#) -> @r Scm<UnspecifiedSpec>

    into_type!(into_list,        is_list,       ListSpec);
});

impl<TS: TypeSpec> From<Vec<Scm<TS>>> for Scm<ListSpec> {
    fn from(l: Vec<Scm<TS>>) -> Scm<ListSpec> {
        let mut l: Vec<SCM> = l.into_iter().map(|e| e.data).collect();
        l.push(unsafe { gu_SCM_UNDEFINED() });
        Scm::_from_raw(unsafe { gu_scm_list_n(l.as_mut_ptr()) })
    }
}

// impl Scm<ListSpec> {
//     // scm_func!(length() -> Scm<IntSpec>, scm_length);
//     // scm_func!(last_pair() -> Scm<PairSpec>, scm_last_pair);
//     // // m_ref to avoid name conflict with rust `ref` keyword
//     // scm_func!(m_ref(k: Scm<IntSpec>) -> Scm<UnspecifiedSpec>, scm_list_ref);
//     // scm_func!(tail(k: Scm<IntSpec>) -> Scm<ListSpec>, scm_list_tail);
//     // scm_func!(head(k: Scm<IntSpec>) -> Scm<ListSpec>, scm_list_head);

//     guile_defs! {
//         pub fn length()            => scm_length(@d)    -> @r Scm<IntSpec>;
//         pub fn last_pair()         => scm_last_pair(@d) -> @r Scm<PairSpec>;

//         pub fn m_ref(Scm<IntSpec>) => scm_list_ref(@d, @0#)  -> @r Scm<UnspecifiedSpec>;
//         pub fn tail(Scm<IntSpec>)  => scm_list_tail(@d, @0#) -> @r Scm<ListSpec>;
//         pub fn head(Scm<IntSpec>)  => scm_list_head(@d, @*#) -> @r Scm<ListSpec>;
//     }

//     // scm_func!(append(lst: Scm<ListSpec>) -> Scm<ListSpec>, scm_append);
// }

guile_impl! (Scm<ListSpec> {
    pub fn length()            => scm_length(@s)    -> @r Scm<IntSpec>
    pub fn last_pair()         => scm_last_pair(@s) -> @r Scm<PairSpec>

    pub fn m_ref(Scm<IntSpec>) => scm_list_ref(@s, @*#)  -> @r Scm<UnspecifiedSpec>
    pub fn tail(Scm<IntSpec>)  => scm_list_tail(@s, @*#) -> @r Scm<ListSpec>
    pub fn head(Scm<IntSpec>)  => scm_list_head(@s, @*#) -> @r Scm<ListSpec>

    into_type!(into_pair,        is_pair,       PairSpec);
});

guile_impl! (Scm<HashTableSpec> {
    // TODO:/// test doc...
    pub fn new(@_)            => scm_make_hash_table(ptr::null_mut())    -> @r Scm<HashTableSpec>
    pub fn with_size(@_, i32) => scm_make_hash_table(Scm::from(@0).data) -> @r Scm<HashTableSpec>

    pub fn clear_x()          => scm_hash_clear_x(@s)

    pub fn m_ref(Scm<KS>|KS:TypeSpec, Option<Scm<DS>>|DS:TypeSpec)
        => scm_hash_ref(@s, @0#, @1.map_or(ptr::null_mut(), |d| d.data))
        -> @r Scm<UnspecifiedSpec>

    pub fn set_x(&mut self, Scm<KS>|KS:TypeSpec, Scm<VS>|VS:TypeSpec)
        => scm_hash_set_x(@s, @*#)

    pub fn remove_x(&mut self, Scm<KS>|KS:TypeSpec)
        => scm_hash_remove_x(@s, @*#)
});
