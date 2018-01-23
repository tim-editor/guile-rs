//! TypeSpecs and Scm struct for handling guile values
 //!
 //! # Example
 //! ```rust,ignore
 //! let s: Scm<Untyped> = Guile::eval("\"test string...\"");
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
pub use self :: untyped :: Untyped;
pub use self :: bool :: Bool;
pub use self :: string :: String;
pub use self :: numeric :: *;
use self :: guile_rs_sys :: *;
use std :: ffi :: CString;
use std :: marker :: PhantomData;
use std :: ptr;
use std :: mem :: {
transmute , forget
};
use std :: collections :: VecDeque;
use libc;
pub trait TypeSpec {
} pub trait NumericSpec : TypeSpec {
} # [ derive (Debug) ] pub struct SymbolSpec;
impl TypeSpec for SymbolSpec {
} # [ derive (Debug) ] pub struct PairSpec;
impl TypeSpec for PairSpec {
} # [ derive (Debug) ] pub struct ListSpec;
impl TypeSpec for ListSpec {
} # [ derive (Debug) ] pub struct HashTableSpec;
impl TypeSpec for HashTableSpec {
} # [ derive (Debug) ] pub struct HashQTableSpec;
impl TypeSpec for HashQTableSpec {
} # [ derive (Debug) ] pub struct HashVTableSpec;
impl TypeSpec for HashVTableSpec {
} # [ derive (Debug) ] pub struct HashXTableSpec;
impl TypeSpec for HashXTableSpec {
} # [ derive (Debug) ] pub struct ForeignTypeSpec;
impl TypeSpec for ForeignTypeSpec {
} pub trait ForeignType {
type Struct;
fn get_type < 'a > () -> & 'a Scm < ForeignTypeSpec >;
fn get_slot_types () -> Box < TypeList >;
fn as_struct < 'a > () -> & 'a Self :: Struct;
fn as_struct_mut < 'a > () -> & 'a mut Self :: Struct ;
} # [ derive (Debug) ] pub struct ForeignObjectSpec < FT : ForeignType > {
type_ : PhantomData < FT >
} impl < FT : ForeignType > TypeSpec for ForeignObjectSpec < FT > {
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
} impl Scm < ForeignTypeSpec > {
unsafe extern "C" fn finalizer (obj : SCM) {
let slot_types_r : * mut Box < TypeList > = transmute (scm_foreign_object_ref (obj , 0));
let slot_types : Box < TypeList > = ptr :: read (slot_types_r);
let mut vals = VecDeque :: new ();
for i in 1 .. slot_types . len () + 1 - 1 {
let slot_c : * mut libc :: c_void = scm_foreign_object_ref (obj , i);
vals . push_back (slot_c) ;
} slot_types . consume_node (vals);
forget (slot_types);
let slot_types_r : Box < Box < TypeList > > = Box :: from_raw (slot_types_r);
drop (slot_types_r) ;
} pub fn new_type (name : & Scm < self :: String > , slot_names : & Scm < ListSpec > , slot_types : Box < TypeList >) -> Self {
let slot_types : Box < Box < TypeList > > = Box :: new (slot_types);
let slot_types_r : * mut Box < TypeList > = Box :: into_raw (slot_types);
let slot_names : Scm < ListSpec > = Scm :: cons (& Scm :: < self :: String > :: from ("types") , & slot_names) . into_list () . unwrap ();
Scm :: _from_raw (unsafe {
scm_make_foreign_object_type (name . data , slot_names . data , Some (Scm :: finalizer))
})
}
} impl < FT : ForeignType > Scm < ForeignObjectSpec < FT > > {
pub fn from_struct (strct : FT :: Struct) -> Self {
unimplemented! ()
} pub fn get_type < 'a > () -> & 'a Scm < ForeignTypeSpec > {
FT :: get_type ()
} pub fn get_slot_types () -> Box < TypeList > {
FT :: get_slot_types ()
} pub fn as_struct_mut < 'a > () -> & 'a mut FT :: Struct {
FT :: as_struct_mut ()
} pub fn as_struct < 'a > () -> & 'a FT :: Struct {
FT :: as_struct ()
}
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
} guile_impl! (impl Scm < SymbolSpec > {
pub fn from_str < > (a0 : & str) -> Scm < SymbolSpec > {
Scm :: _from_raw (unsafe {
scm_from_utf8_symbol (CString :: new (a0) . unwrap () . as_ptr ())
})
} pub fn into_string < > (self) -> Scm < self :: String > {
Scm :: _from_raw (unsafe {
scm_symbol_to_string (self . data)
})
}
});
impl < 'a > From < & 'a str > for Scm < SymbolSpec > {
# [ inline ] fn from (s : & 'a str) -> Scm < SymbolSpec > {
Scm :: < SymbolSpec > :: from_str (s)
}
} guile_impl! (impl Scm < PairSpec > {
pub fn car < > (& self ,) -> Scm < Untyped > {
Scm :: _from_raw (unsafe {
gu_scm_car (self . data)
})
} pub fn cdr < > (& self ,) -> Scm < Untyped > {
Scm :: _from_raw (unsafe {
gu_scm_cdr (self . data)
})
} pub fn cons < A : TypeSpec , B : TypeSpec > (a0 : & Scm < A > , a1 : & Scm < B >) -> Scm < PairSpec > {
Scm :: _from_raw (unsafe {
gu_scm_cons (a0 . data , a1 . data)
})
} pub fn set_car < T : TypeSpec > (& self , a0 : Scm < T >) -> Scm < Untyped > {
Scm :: _from_raw (unsafe {
scm_set_car_x (self . data , a0 . data)
})
} pub fn set_cdr < T : TypeSpec > (& self , a0 : Scm < T >) -> Scm < Untyped > {
Scm :: _from_raw (unsafe {
scm_set_cdr_x (self . data , a0 . data)
})
} into_type! (into_list , is_list , ListSpec) ;
});
impl < TS : TypeSpec > From < Vec < Scm < TS > > > for Scm < ListSpec > {
fn from (l : Vec < Scm < TS > >) -> Scm < ListSpec > {
let mut l : Vec < SCM > = l . into_iter () . map (| e | e . data) . collect ();
l . push (unsafe {
gu_SCM_UNDEFINED ()
});
Scm :: _from_raw (unsafe {
gu_scm_list_n (l . as_mut_ptr ())
})
}
} guile_impl! (impl Scm < ListSpec > {
pub fn length < > (& self ,) -> Scm < Int > {
Scm :: _from_raw (unsafe {
scm_length (self . data)
})
} pub fn last_pair < > (& self ,) -> Scm < PairSpec > {
Scm :: _from_raw (unsafe {
scm_last_pair (self . data)
})
} pub fn m_ref < > (& self , a0 : Scm < Int >) -> Scm < Untyped > {
Scm :: _from_raw (unsafe {
scm_list_ref (self . data , a0 . data)
})
} pub fn tail < > (& self , a0 : Scm < Int >) -> Scm < ListSpec > {
Scm :: _from_raw (unsafe {
scm_list_tail (self . data , a0 . data)
})
} pub fn head < > (& self , a0 : Scm < Int >) -> Scm < ListSpec > {
Scm :: _from_raw (unsafe {
scm_list_head (self . data , a0 . data)
})
} into_type! (into_pair , is_pair , PairSpec) ;
});
guile_impl! (impl Scm < HashTableSpec > {
pub fn new < > () -> Scm < HashTableSpec > {
Scm :: _from_raw (unsafe {
scm_make_hash_table (ptr :: null_mut ())
})
} pub fn with_size < > (a0 : i32) -> Scm < HashTableSpec > {
Scm :: _from_raw (unsafe {
scm_make_hash_table (Scm :: from (a0) . data)
})
} pub fn clear_x < > (& self ,) -> () {
unsafe {
scm_hash_clear_x (self . data)
} ;
} pub fn m_ref < KS : TypeSpec , DS : TypeSpec > (& self , a0 : Scm < KS > , a1 : Option < Scm < DS > >) -> Scm < Untyped > {
Scm :: _from_raw (unsafe {
scm_hash_ref (self . data , a0 . data , a1 . map_or (ptr :: null_mut () , | d | d . data))
})
} pub fn set_x < KS : TypeSpec , VS : TypeSpec > (& mut self , a1 : Scm < KS > , a2 : Scm < VS >) -> () {
unsafe {
scm_hash_set_x (self . data , a1 . data , a2 . data)
} ;
} pub fn remove_x < KS : TypeSpec > (& mut self , a1 : Scm < KS >) -> () {
unsafe {
scm_hash_remove_x (self . data , a1 . data)
} ;
}
}) ;