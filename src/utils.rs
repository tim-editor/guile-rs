/** into_type!()
 *
 *       Generate a function `pub fn $inn(self)` that
 *       checks for `self.$isn()` before converting to
 *       the target typespec: `$spec`
 *
 *      ```rust
 *
 *      into_type!(into_bool, is_bool, Bool);
 *
 *      ```
 *
 *      would produce:
 *
 *      ```rust
 *
 *      pub fn into_bool(self) -> Result<Scm<Bool>, ()> {
 *          if self.is_bool() {
 *              Ok(self.into_type())
 *          } else {
 *              Err(())
 *          }
 *      }
 *
 *      ```
 *
 */
macro_rules! into_type {
    // concatenating identifiers is not allowed
    //  (waiting for macros 2.0)
    // ($tn:ident, $spec:ident) => (
    //     into_type!(into_$tn, is_$tn, $spec);
    // );

    ($inn:ident, $isn:ident, $spec:ident) => {
        pub fn $inn(self) -> Result<Scm<$spec>, ()> {
            if self.$isn() {
                Ok(self.into_type())
            } else {
                Err(())
            }
        }
    };
}

/** is_thing_p!()
 *
 *      Named so to be similar to the scm `_p` suffix
 *      meaning `is_thing?` as in `is_<thing>?` because
 *      the generated func returns an Scm<Bool> object
 *
 *      Generate a function `$fname(&self, args...)` that
 *      checks if `self` is `<thing>` by calling a guile
 *      function that performs the check on `self.data`
 *
 *      ```rust
 *
 *      is_thing_p!(exact_p => scm_exact_p);
 *      is_thing_p!(gr_p(other: T<NumericSpec>) => scm_gr_p);
 *
 *      ```
 *
 *      would produce:
 *
 *      ```rust
 *
 *      #[inline]
 *      pub fn exact_p(&self) -> Scm<Bool> {
 *          Scm::_from_raw(unsafe { scm_exact_p(self.data) })
 *      }
 *
 *      #[inline]
 *      pub fn gr_p<T: NumericSpec>(&self, other: T) -> Scm<Bool> {
 *          Scm::_from_raw(unsafe {
 *                  scm_gr_p(self.data, other.data)
 *              })
 *      }
 *
 *      ```
 *
 *      NOTE: currently only supports generic arguments
 *              (no concrete types except for `self`)
 *
 */
macro_rules! is_thing_p {
    // with no args... (except for `self` of course)
    ($fname:ident => $cfunc:ident) => {
        is_thing_p!($fname() => $cfunc);
    };

    // with generic args...
    ($fname:ident ($($an:ident: $tn:ident <$at:path>),*)
     => $cfunc:ident) => {
        // /// Retrun guile true value when is condition
        #[inline]
        pub fn $fname <$($tn: $at),*>
            (&self, $($an: &Scm<$tn>),*) -> Scm<Bool> {

            Scm::_from_raw(unsafe {
                $cfunc(self.data, $($an.data),*)
            })
        }
    };
}

/** is_thing!()
 *
 *      Generate a function `$fname(&self, args...)` similar
 *      to `is_thing_p!()` but that returns a boolean value
 *
 *      This macro assumes that the c function given itself
 *      returns a bool (0 or 1).
 *
 *      ```rust
 *
 *      is_thing!(is_exact => scm_is_exact);
 *
 *      ```
 *
 *      would produce:
 *
 *      ```rust
 *
 *      #[inline]
 *      pub fn is_exact(&self) -> bool {
 *          unsafe { scm_exact_p(self.data) == 1 }
 *      }
 *
 *      ```
 *
 *      NOTE: currently only supports generic arguments
 *              (no concrete types except for `self`)
 *
 */
macro_rules! is_thing {
    ($fname:ident => $cfunc:ident) => {
        is_thing!($fname() => $cfunc);
    };

    ($fname:ident ($($an:ident: $tn:ident <$at:path>),*)
     => $cfunc:ident) => {
        // /// Retrun true when is condition
        #[inline]
        pub fn $fname<$($tn: $at),*>
            (&self, $($an: &Scm<$tn>),*) -> bool {

            unsafe {
                $cfunc(self.data, $($an.data),*) == 1
            }
        }
    };
}

/** is_thing_manual!()
 *
 *      Generate a function `$fname(&self, args...)` similar
 *      to `is_thing_p!()` but that returns a boolean value
 *
 *      This macro assumes that the c function given itself
 *      returns an SCM boolean and __manually__ checks for
 *      tueness in order to return the rust bool type
 *
 *      ```rust
 *
 *      is_thing_manual!(is_exact => scm_exact_p);
 *
 *      ```
 *
 *      would produce:
 *
 *      ```rust
 *
 *      #[inline]
 *      pub fn is_exact(&self) -> bool {
 *          unsafe { scm_exact_p(self.data) == 1 }
 *      }
 *
 *      ```
 *
 *      NOTE: currently only supports generic arguments
 *              (no concrete types except for `self`)
 *
 */
macro_rules! is_thing_manual {
    ($fname:ident => $cfunc:ident) => {
        is_thing_manual!($fname() => $cfunc);
    };

    ($fname:ident ($($an:ident: $tn:ident <$at:path>),*)
     => $cfunc:ident) => {
        // /// Retrun true when is condition
        #[inline]
        pub fn $fname<$($tn: $at),*>
            (&self, $($an: &Scm<$tn>),*) -> bool {

            unsafe {
                gu_scm_is_true($cfunc(self.data, $($an.data),*))
            }
        }
    };
}

#[allow(unused_macros)]
macro_rules! scm_func {
    ($fname:ident ($($an:ident: $at:ty),*) -> $r:ty, $cfunc:ident) => {
        #[inline]
        pub fn $fname(&self, $($an: $at),*) -> $r {
            Scm::_from_raw(unsafe { $cfunc(self.data, $($an.data),*) })
        }
    };
    ($fname:ident ($($an:ident: $at:ty),*), $cfunc:ident) => {
        #[inline]
        pub fn $fname(&self, $($an: $at),*) {
            unsafe { $cfunc(self.data, $($an.data),*); }
        }
    };
    (P $fname:ident ($($an:ident: $tn:ident <$at:path>),*) -> $r:ty, $cfunc:ident) => {
        #[inline]
        pub fn $fname<$($tn: $at),*>(&self, $($an: &Scm<$tn>),*) -> $r {
            Scm::_from_raw(unsafe { $cfunc(self.data, $($an.data),*) })
        }
    };
    (P $fname:ident ($($an:ident: $tn:ident <$at:path>),*), $cfunc:ident) => {
        #[inline]
        pub fn $fname<$($tn: $at),*>(&self, $($an: &Scm<$tn>),*) {
            unsafe { $cfunc(self.data, $($an.data),*); }
        }
    };
}

/** simple_from!()
 *
 *      Generate an implementation of `From<$from> for $to`
 *      by running `$func()` conversion func that should
 *      take in a value of type `$from` and return type `SCM`
 *
 */
#[allow(unused_macros)]
macro_rules! simple_from {
    ($from:ty, $cfunc: ident, $to:ty) => {
        impl From<$from> for $to {
            fn from(f: $from) -> $to {
                Self {
                    data: unsafe { $cfunc(f) },
                    spec: PhantomData,
                }
            }
        }
    };
}

/** simple_try_as!()
 *
 *      Generate an implementation of `TryAs<$to, ()> for $from`
 *      by running `$func()` conversion func that should take
 *      in a type `SCM` and return a value of type `$to`
 *
 */
#[allow(unused_macros)]
macro_rules! simple_try_as {
    ($from:ty, $cfunc:ident, $to:ty) => {
        impl TryAs<$to, ()> for $from {
            fn try_as(&self) -> Result<$to, ()> {
                if self.is_exact_integer() {
                    // TODO: handle runtime guile errors
                    // TODO: handle guile int not fitting target type
                    Ok(unsafe { $cfunc(self.data) })
                } else {
                    Err(())
                }
            }
        }
    }
}
