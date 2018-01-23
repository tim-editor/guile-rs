use scm::*;
use scm::String as ScmString;

#[derive(Debug)]
pub struct Untyped;
impl TypeSpec for Untyped {}

impl Scm<Untyped> {
    into_type!(into_bool,        is_bool,       Bool);
    into_type!(into_string,      is_string,     ScmString);
    into_type!(into_integer,     is_integer,    Int);
    into_type!(into_symbol,      is_symbol,     SymbolSpec);
    into_type!(into_pair,        is_pair,       PairSpec);
    into_type!(into_list,        is_list,       ListSpec);
    into_type!(into_hash_table,  is_hash_table, HashTableSpec);
    into_type!(into_hashq_table, is_hash_table, HashQTableSpec);
    into_type!(into_hashv_table, is_hash_table, HashVTableSpec);
    into_type!(into_hashx_table, is_hash_table, HashXTableSpec);
}
