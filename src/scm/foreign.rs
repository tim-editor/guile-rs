use libc;
use std::collections::VecDeque;
use std::marker::PhantomData;
use std::mem::forget;
use std::mem::transmute;
use std::ptr;

use scm::Scm;
use scm::TypeSpec;
use scm::TypeList;
use scm::List;
use scm::String as ScmString;

use guile_rs_sys::*;


#[derive(Debug)]
pub struct Foreign;
impl TypeSpec for Foreign {}

pub trait ForeignSpec {
    type Struct;
    fn get_type<'a>()       -> &'a Scm<Foreign>;
    fn get_slot_types()     -> Box<TypeList>;
    fn as_struct<'a>()      -> &'a Self::Struct;
    fn as_struct_mut<'a>()  -> &'a mut Self::Struct;
}

#[derive(Debug)]
pub struct ForeignObject<FT: ForeignSpec> { type_: PhantomData<FT> }
impl<FT: ForeignSpec> TypeSpec for ForeignObject<FT> {}

impl Scm<Foreign> {
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
    pub fn new_type(name: &Scm<ScmString>, slot_names: &Scm<List>, slot_types: Box<TypeList>) -> Self {

        // NOTE: This is in the wrong function, it should be in the new object initializer...
        // NOTE: we also need a way of forcing these types
        let slot_types: Box<Box<TypeList>> = Box::new(slot_types);
        // keep in mind this takes ownership of slot_types
        let slot_types_r: *mut Box<TypeList> = Box::into_raw(slot_types);


        let slot_names: Scm<List> = Scm::cons(&Scm::<ScmString>::from("types"), &slot_names).into_list().unwrap();

        Scm::_from_raw(unsafe {
            scm_make_foreign_object_type(name.data, slot_names.data, Some(Scm::finalizer))
        })
    }
}

// NOTE: Most functions in here have the wrong ideas...
// from_struct and as_struct make no sense since what we need is rather as_struct for every slot
impl<FT: ForeignSpec> Scm<ForeignObject<FT>> {
    pub fn from_struct(strct: FT::Struct) -> Self {
        unimplemented!()
    }
    pub fn get_type<'a>() -> &'a Scm<Foreign> { FT::get_type() }
    pub fn get_slot_types() -> Box<TypeList> { FT::get_slot_types() }

    pub fn as_struct_mut<'a>() -> &'a mut FT::Struct { FT::as_struct_mut() }
    pub fn as_struct<'a>()     -> &'a     FT::Struct { FT::as_struct()     }
}
