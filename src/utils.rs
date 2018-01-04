
macro_rules! into_type {
    ($tn:ident, $spec:ident) => (into_type!(into_$tn, is_$tn, $spec););

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

macro_rules! is_thing_p {
    ($fname:ident => $cfunc:ident) => {
        is_thing_p!($fname() => $cfunc);
    };
    ($fname:ident ($($an:ident: $tn:ident <$at:path>),*) => $cfunc:ident) => {
        // /// Retrun guile true value when is condition
        #[inline]
        pub fn $fname<$($tn: $at),*>(&self, $($an: &Scm<$tn>),*) -> Scm<BoolSpec> {
            Scm::_from_raw(unsafe { $cfunc(self.data, $($an.data),*) })
        }
    };
}

macro_rules! is_thing {
    ($fname:ident => $cfunc:ident) => {
        is_thing!($fname() => $cfunc);
    };
    ($fname:ident ($($an:ident: $tn:ident <$at:path>),*) => $cfunc:ident) => {
        // /// Retrun true when is condition
        #[inline]
        pub fn $fname<$($tn: $at),*>(&self, $($an: &Scm<$tn>),*) -> bool {
            unsafe { $cfunc(self.data, $($an.data),*) == 1 }
        }
    };
}

macro_rules! is_thing_manual {
    ($fname:ident => $cfunc:ident) => {
        is_thing_manual!($fname() => $cfunc);
    };
    ($fname:ident ($($an:ident: $tn:ident <$at:path>),*) => $cfunc:ident) => {
        // /// Retrun true when is condition
        #[inline]
        pub fn $fname<$($tn: $at),*>(&self, $($an: &Scm<$tn>),*) -> bool {
            unsafe { gu_scm_is_true($cfunc(self.data, $($an.data),*)) }
        }
    };
}

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
