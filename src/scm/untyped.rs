use scm::*;
use scm::String as ScmString;
use scm::Foreign;
use scm::ForeignObject;


#[derive(Clone, Debug)]
pub struct Untyped;
impl TypeSpec for Untyped {}

impl Scm<Untyped> {
    into_type!(into_bool,        is_bool,       Bool);
    into_type!(into_string,      is_string,     ScmString);
    into_type!(into_integer,     is_integer,    Int);
    into_type!(into_symbol,      is_symbol,     Symbol);
    into_type!(into_pair,        is_pair,       Pair);
    into_type!(into_list,        is_list,       List);
    into_type!(into_hash_table,  is_hash_table, HashTable);
    into_type!(into_hashq_table, is_hash_table, HashQTable);
    into_type!(into_hashv_table, is_hash_table, HashVTable);
    into_type!(into_hashx_table, is_hash_table, HashXTable);

    pub fn into_foreign<T>(self, typ: &Scm<Foreign<T>>) ->
        Result<Scm<ForeignObject<T>>, ()> {

        if self.is_foreign(typ) {
            Ok(Scm::_from_raw_with_spec(
                    self.data,
                    ForeignObject { typ: typ.clone() }))
        } else {
            Err(())
        }
    }
}
