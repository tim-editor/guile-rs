use std::ops::Not;

use scm::Scm;
use scm::TypeSpec;

use guile_rs_sys::*;


#[derive(Clone, Debug)]
pub struct Bool;
impl TypeSpec for Bool {}

impl Scm<Bool> {
    /// Return a true litteral Scm object
    #[inline]
    pub fn true_c() -> Scm<Bool> {
        Scm::_from_raw(unsafe { gu_SCM_BOOL_T() })
        // Scm { data: unsafe { gu_SCM_BOOL_T() } , spec: PhantomData }
    }

    /// Return a false litteral Scm object
    #[inline]
    pub fn false_c() -> Scm<Bool> {
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

impl Not for Scm<Bool> {
    type Output = Scm<Bool>;
    fn not(self) -> Scm<Bool> {
        Scm::_from_raw(unsafe { scm_not(self.data) })
    }
}
