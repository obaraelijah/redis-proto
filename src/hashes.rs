use crate::op_variants;
use crate::types::{Count, Key, ReturnValue, StateRef, Value};

op_variants! {
    HashOps,
    HGet(Key, Key),
    HSet(Key, Key, Value)
}

make_reader!(hashes, read_hashes);
make_writer!(hashes, write_hashes);

pub async fn hash_interact(hash_ops: HashOps, state: StateRef) -> ReturnValue {
    match hash_ops {
        HashOps::HGet(key, field) => match read_hashes!(state, &key) {
            None => ReturnValue::Nil,
            Some(hash) => hash
                .get(&field)
                .map_or(ReturnValue::Nil, |f| ReturnValue::StringRes(f.clone())),
        },
        HashOps::HSet(key, field, value) => {
            state.hashes.entry(key).or_default().insert(field, value);
            ReturnValue::Ok
        }
    }
}