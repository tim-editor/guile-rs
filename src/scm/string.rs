use std;

use std::ffi::CString;
use scm::Scm;
use scm::TypeSpec;
use scm::Symbol;

use guile_rs_sys::*;

#[derive(Clone, Debug)]
pub struct String;

use scm::string::String as ScmString;
impl TypeSpec for ScmString {}


impl<'a> From<&'a str> for Scm<ScmString> {
    #[inline]
    fn from(s: &'a str) -> Scm<ScmString> {
        Scm::<ScmString>::from_str(s)
    }
}

impl From<std::string::String> for Scm<ScmString> {
    #[inline]
    fn from(s: std::string::String) -> Scm<ScmString> {
        Scm::<ScmString>::from_str(&s)
    }
}

guile_impl!(Scm<ScmString> {
    pub fn from_str(@_, &str)
        => scm_from_utf8_string(CString::new(@0).unwrap().as_ptr())
        -> @r Scm<ScmString>

    /// to utf8 string
    pub fn to_string(&self) -> std::string::String {
        unsafe {
            CString::from_raw(scm_to_utf8_string(self.data)).into_string().unwrap()
        }
    }

    pub fn into_symbol(self) -> Scm<Symbol> {
        Scm::_from_raw(unsafe { scm_string_to_symbol(self.data) })
    }
});
