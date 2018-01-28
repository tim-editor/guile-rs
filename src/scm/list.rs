use scm::Scm;
use scm::TypeSpec;
use scm::*;


// NOTE: should we have this? (list types are really just pair chains)
#[derive(Debug)]
pub struct List;
impl TypeSpec for List {}


impl<TS: TypeSpec> From<Vec<Scm<TS>>> for Scm<List> {
    fn from(l: Vec<Scm<TS>>) -> Scm<List> {
        let mut l: Vec<SCM> = l.into_iter().map(|e| e.data).collect();
        l.push(unsafe { gu_SCM_UNDEFINED() });
        Scm::_from_raw(unsafe { gu_scm_list_n(l.as_mut_ptr()) })
    }
}

// impl Scm<List> {
//     // scm_func!(length() -> Scm<Int>, scm_length);
//     // scm_func!(last_pair() -> Scm<Pair>, scm_last_pair);
//     // // m_ref to avoid name conflict with rust `ref` keyword
//     // scm_func!(m_ref(k: Scm<Int>) -> Scm<Untyped>, scm_list_ref);
//     // scm_func!(tail(k: Scm<Int>) -> Scm<List>, scm_list_tail);
//     // scm_func!(head(k: Scm<Int>) -> Scm<List>, scm_list_head);

//     guile_defs! {
//         pub fn length()            => scm_length(@d)    -> @r Scm<Int>;
//         pub fn last_pair()         => scm_last_pair(@d) -> @r Scm<Pair>;

//         pub fn m_ref(Scm<Int>) => scm_list_ref(@d, @0#)  -> @r Scm<Untyped>;
//         pub fn tail(Scm<Int>)  => scm_list_tail(@d, @0#) -> @r Scm<List>;
//         pub fn head(Scm<Int>)  => scm_list_head(@d, @*#) -> @r Scm<List>;
//     }

//     // scm_func!(append(lst: Scm<List>) -> Scm<List>, scm_append);
// }

guile_impl! (Scm<List> {
    pub fn length()            => scm_length(@s)    -> @r Scm<Int>
    pub fn last_pair()         => scm_last_pair(@s) -> @r Scm<Pair>

    pub fn m_ref(Scm<Int>) => scm_list_ref(@s, @*#)  -> @r Scm<Untyped>
    pub fn tail(Scm<Int>)  => scm_list_tail(@s, @*#) -> @r Scm<List>
    pub fn head(Scm<Int>)  => scm_list_head(@s, @*#) -> @r Scm<List>

    into_type!(into_pair,        is_pair,       Pair);
});
