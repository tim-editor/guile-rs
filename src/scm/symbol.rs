use std::ffi::CString;
use scm::Scm;
use scm::TypeSpec;
use scm::String as ScmString;

use guile_rs_sys::*;


#[derive(Debug)]
pub struct Symbol;
impl TypeSpec for Symbol {}


guile_impl!(Scm<Symbol> {
    pub fn from_str(@_, &str)
        => scm_from_utf8_symbol(CString::new(@0).unwrap().as_ptr())
        -> @r Scm<Symbol>
    // pub fn from_str(s: &str) -> Scm<Symbol> {
    //     Scm::_from_raw(unsafe { scm_from_utf8_symbol(CString::new(s).unwrap().as_ptr()) })
    // }

    pub fn into_string(self)
        => scm_symbol_to_string(@s)
        -> @r Scm<ScmString>

    // pub fn into_string(self) -> Scm<ScmString> {
    //     Scm::_from_raw(unsafe { scm_symbol_to_string(self.data) })
    // }
});

impl<'a> From<&'a str> for Scm<Symbol> {
    #[inline]
    fn from(s: &'a str) -> Scm<Symbol> {
        Scm::<Symbol>::from_str(s)
    }
}

