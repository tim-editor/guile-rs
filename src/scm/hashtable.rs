use std::ptr;
use scm::Scm;
use scm::TypeSpec;
use scm::Untyped;

use guile_rs_sys::*;


#[derive(Clone, Debug)]
pub struct HashTable;
impl TypeSpec for HashTable {}

#[derive(Clone, Debug)]
pub struct HashQTable;
impl TypeSpec for HashQTable {}

#[derive(Clone, Debug)]
pub struct HashVTable;
impl TypeSpec for HashVTable {}

#[derive(Clone, Debug)]
pub struct HashXTable;
impl TypeSpec for HashXTable {}


guile_impl! (Scm<HashTable> {
    // TODO: test doc...
    pub fn new(@_)
        => scm_make_hash_table(ptr::null_mut())
        -> @r Scm<HashTable>

    pub fn with_size(@_, i32)
        => scm_make_hash_table(Scm::from(@0).data)
        -> @r Scm<HashTable>

    pub fn clear_x()
        => scm_hash_clear_x(@s)

    pub fn m_ref(Scm<KS>|KS:TypeSpec, Option<Scm<DS>>|DS:TypeSpec)
        => scm_hash_ref(@s, @0#, @1.map_or(ptr::null_mut(), |d| d.data))
        -> @r Scm<Untyped>

    pub fn set_x(&mut self, Scm<KS>|KS:TypeSpec, Scm<VS>|VS:TypeSpec)
        => scm_hash_set_x(@s, @*#)

    pub fn remove_x(&mut self, Scm<KS>|KS:TypeSpec)
        => scm_hash_remove_x(@s, @*#)
});
