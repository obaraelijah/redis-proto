use crate::ops::RVec;
use crate::types::{Count, Key, ReturnValue, StateRef, Value, Index};
use crate::{make_reader, make_writer, op_variants};

op_variants! {
    ListOps,
    LPush(Key, RVec<Value>),
    LPushX(Key, Value),
    RPushX(Key, Value),
    LLen(Key),
    LPop(Key),
    RPop(Key),
    RPush(Key, RVec<Value>),
    LIndex(Key, Index),
    LSet(Key, Index, Value)
}

make_reader!(lists, read_lists);
make_writer!(lists, write_lists);

pub async fn list_interact(list_op: ListOps, state: StateRef) -> ReturnValue {
    match list_op {
        ListOps::LPush(key, vals) => {
            let mut list = state.lists.entry(key.clone()).or_default();
            for val in vals {
                list.push_front(val);
            }
            state.wake_list(&key);
            ReturnValue::IntRes(list.len() as Count)
        }
        ListOps::LPushX(key, val) => match state.lists.get_mut(&key) {
            Some(mut list) => {
                list.push_front(val);
                state.wake_list(&key);
                ReturnValue::IntRes(list.len() as Count)
            }
            None => ReturnValue::IntRes(0),
        },
        ListOps::RPushX(key, val) => match state.lists.get_mut(&key) {
            Some(mut list) => {
                list.push_back(val);
                state.wake_list(&key);
                ReturnValue::IntRes(list.len() as Count)
            }
            None => ReturnValue::IntRes(0),
        },
        ListOps::LLen(key) => match read_lists!(state, &key) {
            Some(l) => ReturnValue::IntRes(l.len() as Count),
            None => ReturnValue::IntRes(0),
        },
        ListOps::LPop(key) => match write_lists!(state, &key).and_then(|mut v| v.pop_front()) {
            Some(v) => ReturnValue::StringRes(v),
            None => ReturnValue::Nil,
        },
        ListOps::RPop(key) => match write_lists!(state, &key).and_then(|mut v| v.pop_back()) {
            Some(v) => ReturnValue::StringRes(v),
            None => ReturnValue::Nil,
        },
        ListOps::RPush(key, vals) => {
            let mut list = state.lists.entry(key).or_default();
            for val in vals {
                list.push_back(val)
            }
            ReturnValue::IntRes(list.len() as Count)
        }
        ListOps::LIndex(key, index) => match write_lists!(state, &key) {
            Some(list) => {
                let llen = list.len() as i64;
                let real_index = if index < 0 { llen + index } else { index };
                if !(0 <= real_index && real_index < llen) {
                    return ReturnValue::Error(b"Bad Range!");
                }
                let real_index = real_index as usize;
                ReturnValue::StringRes(list[real_index].clone())
            }
            None => ReturnValue::Nil,
        },
        ListOps::LSet(key, index, value) => match write_lists!(state, &key) {
            Some(mut list) => {
                let llen = list.len() as i64;
                let real_index = if index < 0 { llen + index } else { index };
                if !(0 <= real_index && real_index < llen) {
                    return ReturnValue::Error(b"Bad Range!");
                }
                let real_index = real_index as usize;
                list[real_index] = value;
                ReturnValue::Ok
            }
            None => ReturnValue::Error(b"No list at key!"),
        },
    }
}