//! TypeSpecs and Scm struct for handling guile values
 //!
 //! # Example
 //! ```rust,ignore
 //! let s: Scm<Untyped> = Guile::eval("\"test string...\"");
 //! let s: Scm<String>      = s.into_string().unwrap();
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
 # [ allow (unused_macros) ] macro_rules! guile_defs {
{
$ ($ e : tt) *
} => {
$ ($ e) *
}
} macro_rules! guile_impl {
{
$ ($ e : tt) *
} => {
$ ($ e) *
}
} extern crate guile_rs_sys;
mod untyped;
mod bool;
mod string;
mod numeric;
mod symbol;
mod pair;
mod list;
mod hashtable;
mod foreign;
pub use self :: untyped :: Untyped;
pub use self :: bool :: Bool;
pub use self :: string :: String;
pub use self :: numeric :: *;
pub use self :: symbol :: Symbol;
pub use self :: pair :: Pair;
pub use self :: list :: List;
pub use self :: hashtable :: {
HashTable , HashQTable , HashVTable , HashXTable
};
pub use self :: foreign :: {
Foreign , ForeignObject , ForeignSpec
};
use self :: guile_rs_sys :: *;
use std :: marker :: PhantomData;
use std :: ptr;
use std :: mem :: transmute;
use std :: collections :: VecDeque;
use libc;
pub trait TypeSpec {
} pub trait NumericSpec : TypeSpec {
} # [ derive (Clone , Debug) ] pub struct Scm < TS : TypeSpec > {
pub (crate) data : SCM , spec : PhantomData < TS >
} unsafe impl < TS : TypeSpec > Send for Scm < TS > {
} unsafe impl < TS : TypeSpec > Sync for Scm < TS > {
} impl < TS : TypeSpec > Scm < TS > {
# [ inline ] pub (crate) fn _from_raw (data : SCM) -> Scm < TS > {
Scm {
data , spec : PhantomData
}
} # [ inline ] pub fn from_raw (data : SCM) -> Scm < Untyped > {
Scm :: _from_raw (data)
} # [ inline ] pub unsafe fn into_raw (self) -> SCM {
self . data
} fn into_type < S : TypeSpec > (self) -> Scm < S > {
Scm :: _from_raw (self . data)
} # [ inline ] pub fn into_unspecified (self) -> Scm < Untyped > {
Scm :: into_type (self)
} # [ inline ] pub fn as_bits (& self) -> scm_t_bits {
unsafe {
gu_SCM_UNPACK (self . data)
}
} # [ inline ] pub fn is_true (& self) -> bool {
unsafe {
gu_scm_is_true (self . data)
}
} # [ inline ] pub fn is_false (& self) -> bool {
unsafe {
gu_scm_is_false (self . data)
}
} # [ inline ] pub fn is_bool (& self) -> bool {
unsafe {
scm_is_bool (self . data) == 1
}
} # [ inline ] pub fn is_string (& self) -> bool {
unsafe {
gu_scm_is_string (self . data) == 1
}
} is_thing! (is_number => scm_is_number);
is_thing! (is_integer => scm_is_integer);
is_thing! (is_exact_integer => scm_is_exact_integer);
is_thing_p! (number_p => scm_number_p);
is_thing_p! (integer_p => scm_integer_p);
is_thing_p! (exact_integer_p => scm_exact_integer_p);
is_thing! (is_symbol => gu_scm_is_symbol);
is_thing! (is_pair => gu_scm_is_pair);
is_thing_manual! (is_list => scm_list_p);
is_thing_manual! (is_hash_table => scm_hash_table_p);
is_thing_p! (symbol_p => scm_symbol_p);
is_thing_p! (pair_p => scm_pair_p);
is_thing_p! (list_p => scm_list_p);
is_thing_p! (hash_table_p => scm_hash_table_p);
/// check for identity (`scm_eq_p`)
 /// scheme operation: `eq?`
 # [ inline ] pub fn eq_p < OS : TypeSpec > (& self , other : & Scm < OS >) -> Scm < Bool > {
Scm :: _from_raw (unsafe {
scm_eq_p (self . data , other . data)
})
} /// check for identity (`scm_eq_p`)
 /// scheme operation: `eq?`
 # [ inline ] pub fn is_eq < OS : TypeSpec > (& self , other : & Scm < OS >) -> bool {
unsafe {
gu_scm_is_eq (self . data , other . data)
}
} is_thing_p! (equal_p (other : OS < TypeSpec >) => scm_equal_p) ;
} /// A binary list of types known at compile time
 /// `Box<TypeList>` should always be built from the `type_list!()` macro!
 pub trait TypeList : Send + Sync {
/// Drop the node's contents
 ///
 /// IMPORTANT: length of `v` should be equal to length of the ndoe
 unsafe fn consume_node (& self , v : VecDeque < * mut libc :: c_void >);
/// Get the length of the node
 /// 0 if node is a Nil
 fn len (& self) -> usize;
fn cloned (& self) -> Box < TypeList > ;
} impl Clone for Box < TypeList > {
fn clone (& self) -> Self {
self . cloned ()
}
} /// A Type element from a `TypeList`
 pub trait TypeElem : Send + Sync {
unsafe fn consume (& self , v : * mut libc :: c_void);
fn cloned (& self) -> Box < TypeElem > ;
} impl Clone for Box < TypeElem > {
fn clone (& self) -> Self {
self . cloned ()
}
} # [ derive (Clone) ] /// An Item in the list representing the type at that position
 pub struct TypeItem < T : 'static + Send + Sync > (pub PhantomData < T >);
impl < T : 'static + Send + Sync > TypeElem for TypeItem < T > {
/// Properly drop the variable
 ///
 /// IMPORTANT: the value of the `v` parameter should be a raw pointer from a `Box<T>`
 /// where `T` is the TypeItem's `T`. (check code for clearer view of functionality)
 unsafe fn consume (& self , v : * mut libc :: c_void) {
let v : Box < T > = Box :: from_raw (transmute (v));
drop (v) ;
} fn cloned (& self) -> Box < TypeElem > {
Box :: new (TypeItem :: < T > (PhantomData))
}
} # [ derive (Clone) ] /// Marks end of a TyepeList
 pub struct Nil {
} impl TypeList for Nil {
unsafe fn consume_node (& self , v : VecDeque < * mut libc :: c_void >) {
assert_eq! (v . len () , 0) ;
} fn len (& self) -> usize {
0
} fn cloned (& self) -> Box < TypeList > {
Box :: new (self . clone ())
}
} # [ derive (Clone) ] /// A node of the binary spine that makes the TypeList
 pub struct TypePair (pub Box < TypeElem > , pub Box < TypeList >);
impl TypeList for TypePair {
unsafe fn consume_node (& self , mut v : VecDeque < * mut libc :: c_void >) {
assert_eq! (v . len () , self . len ());
self . 1 . consume_node (v . split_off (1));
assert_eq! (v . len () , 1);
self . 0 . consume (v [ 0 ]) ;
} fn len (& self) -> usize {
1 + self . 1 . len ()
} fn cloned (& self) -> Box < TypeList > {
Box :: new (self . clone ())
}
} /// Initialize a `Box<TypeList>`
 # [ macro_export ] macro_rules! type_list {
[ $ head : ty , $ ($ tail : ty) ,* ] => {
{
Box :: new (TypePair (Box :: new (TypeItem ::<$ head > (PhantomData)) , type_list! [ $ ($ tail) ,* ]))
}
};
[ $ head : ty ] => {
{
type_list! [ $ head , ]
}
};
[ ] => {
{
Box :: new (Nil {
})
}
} ;
} impl < N : NumericSpec > From < Scm < N > > for Scm < self :: String > {
fn from (numeric : Scm < N >) -> Scm < self :: String > {
Self {
data : unsafe {
scm_number_to_string (numeric . data , ptr :: null_mut ())
} , spec : PhantomData ,
}
}
} pub trait TryAs < T , E > {
/// attemp to get `&self` as type `T`
 fn try_as (& self) -> Result < T , E > ;
}