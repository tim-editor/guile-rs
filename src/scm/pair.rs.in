use scm::TypeSpec;
use scm::Scm;
use scm::*;

#[derive(Debug)]
pub struct Pair;
impl TypeSpec for Pair {}

// impl Scm<Pair> {
//     scm_func!(car() -> Scm<Untyped>, gu_scm_car);
//     scm_func!(cdr() -> Scm<Untyped>, gu_scm_cdr);

//     scm_func!(P set_car(value: T<TypeSpec>), scm_set_car_x);
//     scm_func!(P set_cdr(value: T<TypeSpec>), scm_set_cdr_x);
// }

guile_impl!(Scm<Pair> {
    pub fn car() => gu_scm_car(@s) -> @r Scm<Untyped>
    pub fn cdr() => gu_scm_cdr(@s) -> @r Scm<Untyped>
    pub fn cons(@_, &Scm<A>|A:TypeSpec, &Scm<B>|B:TypeSpec) => gu_scm_cons(@*#) -> @r Scm<Pair>

    pub fn set_car(Scm<T>|T:TypeSpec) => scm_set_car_x(@s, @*#) -> @r Scm<Untyped>
    pub fn set_cdr(Scm<T>|T:TypeSpec) => scm_set_cdr_x(@s, @*#) -> @r Scm<Untyped>

    into_type!(into_list,        is_list,       List);
});

