use libc;
use std::collections::VecDeque;
use std::marker::PhantomData;
use std::mem::forget;
use std::mem::transmute;
use std::any::Any;
use std::ptr;

use scm::Scm;
use scm::TypeSpec;
use scm::TypeList;
use scm::List;
use scm::String as ScmString;
use scm::Symbol;

use guile_rs_sys::*;


#[derive(Debug)]
pub struct Foreign<T> {
    _data: PhantomData<T>,
    // pub(crate) slot_types: Box<TypeList>
}

impl<T> Clone for Foreign<T> {
    fn clone(&self) -> Foreign<T> { Foreign { _data: PhantomData } }
}

impl<T> TypeSpec for Foreign<T> {}

pub trait ForeignSpec {
    // type SlotTypes;
    // fn get_type<'a>()       -> &'a Scm<Foreign>;
    // fn get_slot_types()     -> Box<TypeList>;
}

#[derive(Debug)]
// pub struct ForeignObject<FT: ForeignSpec> { type_: PhantomData<FT> }
// impl<FT: ForeignSpec> TypeSpec for ForeignObject<FT> {}
pub struct ForeignObject<T> { pub(crate) typ: Scm<Foreign<T>> }
impl<T> Clone for ForeignObject<T> {
    fn clone(&self) -> ForeignObject<T> { ForeignObject { typ: self.typ.clone() } }
}
impl<T> TypeSpec for ForeignObject<T> {}

impl<T> Scm<Foreign<T>> {
    // unsafe extern "C" fn finalizer(obj: SCM) {
    //     // a pointer to TypeList does not fit in a void pointer...
    //     // ---- reconstruct TypeList ----
    //     let slot_types_r: *mut Box<TypeList> = transmute(scm_foreign_object_ref(obj, 0));
    //     if slot_types_r == ptr::null_mut() { return; }

    //     let slot_types: Box<TypeList> = ptr::read(slot_types_r);
    //     // ---- ----

    //     // ---- build VecDeque of pointers to boxed values to free ----
    //     let mut vals = VecDeque::new();
    //     for i in 1..slot_types.len()+1-1 {
    //         let slot_c: *mut libc::c_void = scm_foreign_object_ref(obj, i);
    //         vals.push_back(slot_c);
    //     }
    //     // ---- ----

    //     // ---- free the boxes ----
    //     // call this line only once!!! otherwise double-free
    //     slot_types.consume_node(vals);
    //     // ---- ----

    //     // drop(slot_types);
    //     forget(slot_types);
    //     let slot_types_r: Box<Box<TypeList>> = Box::from_raw(slot_types_r);
    //     drop(slot_types_r);

    // }
    unsafe extern "C" fn finalizer(obj: SCM) {
        let slot: *mut T = transmute(scm_foreign_object_ref(obj, 0));
        if slot == ptr::null_mut() { return; }

        let slot: Box<T> = Box::from_raw(slot);
        drop(slot);
    }

    pub fn new_type(name: &Scm<Symbol>) -> Self {
        let slots: Scm<List> = vec![
            Scm::<Symbol>::from("data")
        ].into();

        Scm::_from_raw_with_spec(
            unsafe {
                scm_make_foreign_object_type(name.data,
                                             slots.data,
                                             Some(Self::finalizer))
            },
            Foreign { _data: PhantomData })
    }
    // // NOTE: types in slots should probably be Boxes!!!!!
    // pub fn new_type(name: &Scm<ScmString>, slot_names: &Scm<List>, slot_types: Box<TypeList>) -> Self {

    //     // NOTE: This is in the wrong function, it should be in the new object initializer...
    //     // NOTE: we also need a way of forcing these types
    //     // let slot_types: Box<Box<TypeList>> = Box::new(slot_types);
    //     // keep in mind this takes ownership of slot_types
    //     // let slot_types_r: *mut Box<TypeList> = Box::into_raw(slot_types);


    //     let slot_names: Scm<List> = Scm::cons(&Scm::<ScmString>::from("types"), &slot_names).into_list().unwrap();

    //     Scm::_from_raw_with_spec(
    //         unsafe {
    //             scm_make_foreign_object_type(name.data, slot_names.data, Some(Scm::finalizer))
    //         },
    //         Foreign { slot_types })
    // }

    // pub fn get_slot_types<'a>(&'a self) -> &'a TypeList {
    //     &*self.spec.as_ref().unwrap().slot_types
    // }
}

// NOTE: Most functions in here have the wrong ideas...
// from_struct and as_struct make no sense since what we need is rather as_struct for every slot
// impl<FT: ForeignSpec> Scm<ForeignObject<FT>> {
impl<T> Scm<ForeignObject<T>> {
    // pub fn new() -> Scm<ForeignObject<FT>> {
    // }
    // pub fn from_struct(strct: FT::Struct) -> Self {
    //     unimplemented!()
    // }
    // pub fn get_type<'a>() -> &'a Scm<Foreign> { FT::get_type() }
    // pub fn get_slot_types<'a>() -> &'a TypeList { FT::get_type().get_slot_types() }

    // pub fn set_slot(&self, n: usize, val: &Any) -> Option<&Any> {
    // }

    pub fn new(typ: &Scm<Foreign<T>>, data: T) ->
        Scm<ForeignObject<T>> {

        Scm::_from_raw_with_spec(
            unsafe {
                scm_make_foreign_object_1(
                    typ.data,
                    Box::into_raw(Box::new(data))
                        as *mut libc::c_void)
            },
            ForeignObject { typ: (*typ).clone() })
    }

    pub unsafe fn get_raw(&self) -> *mut T {
        let slot_c: *mut libc::c_void
            = scm_foreign_object_ref(self.data, 0);

        slot_c as *mut T
    }

    pub fn get_data(&self) -> Option<&T> {
        unsafe {
            self.get_raw().as_ref()
        }
    }

    pub fn get_data_mut(&mut self) -> Option<&mut T> {
        unsafe {
            self.get_raw().as_mut()
        }
    }

    // pub fn get_slot(&self, n: usize) -> Option<&Any> {
    //     unsafe {
    //         let slot_c: *mut libc::c_void = scm_foreign_object_ref(self.data, n+1);
    //         if slot_c != ptr::null_mut() {
    //             Self::get_slot_types().deref(slot_c, n)
    //         } else {
    //             None
    //         }
    //     }
    // }

    // pub fn get_slots(&self) -> &TypeList { FT::get_slot_types().borrow() }





    // pub fn as_struct_mut<'a>() -> &'a mut FT::Struct { FT::as_struct_mut() }
    // pub fn as_struct<'a>()     -> &'a     FT::Struct { FT::as_struct()     }
}
