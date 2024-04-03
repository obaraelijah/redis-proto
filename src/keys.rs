use crate::op_variants;
use crate::types::{Key, Value, ReturnValue, StateRef};

op_variants! {
    KeyOps,
    Set(Key, Value),
    Get(Key)
}

pub async fn key_interact(key_op: KeyOps, state: StateRef) -> ReturnValue {
    match key_op {
        KeyOps::Get(key) => state.kv.get(&key).map_or(ReturnValue::Nil, |v| {
            ReturnValue::StringRes(v.value().clone())
        }),
        KeyOps::Set(key, value) => {
            state.kv.insert(key, value);
            ReturnValue::Ok
        },
    }   
}