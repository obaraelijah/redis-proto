use crate::op_variants;
use crate::types::{Key, Value, ReturnValue, StateRef, Count};
use crate::ops::RVec;

op_variants! {
    KeyOps,
    Set(Key, Value),
    MSet(RVec<(Key, Value)>),
    Get(Key),
    MGet(RVec<Key>),
    Del(RVec<Key>)
}

pub async fn key_interact(key_op: KeyOps, state: StateRef) -> ReturnValue {
    match key_op {
        KeyOps::Get(key) => state.kv.get(&key).map_or(ReturnValue::Nil, |v| {
            ReturnValue::StringRes(v.value().clone())
        }),
        KeyOps::MGet(keys) => {
            let vals = keys
                .iter()
                .map(|key| match state.kv.get(key) {
                    Some(v) => ReturnValue::StringRes(v.value().clone()),
                    None => ReturnValue::Nil,
                })
                .collect();
            ReturnValue::Array(vals)
        }
        KeyOps::Set(key, value) => {
            state.kv.insert(key, value);
            ReturnValue::Ok
        }
        KeyOps::MSet(key_vals) => {
            let kv = &state.kv;
            for (key, val) in key_vals.into_iter() {
                kv.insert(key, val);
            }
            ReturnValue::Ok
        }
        KeyOps::Del(keys) => {
            let deleted = keys
                .iter()
                .map(|x| state.kv.remove(x))
                .filter(Option::is_some)
                .count();
            ReturnValue::IntRes(deleted as Count)
        }
    }   
}