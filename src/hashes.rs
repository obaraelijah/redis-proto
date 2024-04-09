use crate::op_variants;
use crate::types::{Count, Key, ReturnValue, StateRef, Value};
use crate::ops::RVec;

op_variants! {
    HashOps,
    HGet(Key, Key),
    HSet(Key, Key, Value),
    HExists(Key, Key),
    HGetAll(Key),
    HMGet(Key, RVec<Key>),
    HKeys(Key),
    HMSet(Key, RVec<(Key, Value)>),
    HLen(Key),
    HDel(Key, RVec<Key>)
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
        HashOps::HExists(key, field) => read_hashes!(state)
            .get(&key)
            .map(|hashes| hashes.contains_key(&field))
            .map_or(ReturnValue::IntRes(0), |v: bool| {
                ReturnValue::IntRes(if v { 1 } else { 0 })
            }),
        // HashOps::HGetAll(key) => {
        //     read_hashes!(state, &key, hash);
        //     if hash.is_none() {
        //         return ;
        //     }
        //     let mut ret = Vec::new();
        //     for (key, val) in hash.unwrap().iter() {
        //         ret.push(key.clone());
        //         ret.push(val.clone());
        //     }
        //     ReturnValue::MultiStringRes(ret)
        // }
        HashOps::HGetAll(key) => match read_hashes!(state, &key) {
            Some(hash) => {
                let mut ret = Vec::with_capacity(hash.len() * 2);
                for (key, value) in hash.iter() {
                    ret.push(key.clone());
                    ret.push(value.clone());
                }
                ReturnValue::MultiStringRes(ret)
            }
            None => ReturnValue::MultiStringRes(Vec::with_capacity(0)),
        },
        HashOps::HMGet(key, fields) => ReturnValue::Array(match read_hashes!(state, &key) {
            None => std::iter::repeat_with(|| ReturnValue::Nil)
                .take(fields.len())
                .collect(),
            Some(hash) => fields
                .iter()
                .map(|field| {
                    hash.get(field)
                        .map_or(ReturnValue::Nil, |v| ReturnValue::StringRes(v.clone()))
                })
                .collect(),
        }),
        HashOps::HKeys(key) => match read_hashes!(state, &key) {
            Some(hash) => {
                ReturnValue::Array(hash.keys().cloned().map(ReturnValue::StringRes).collect())
            }
            None => ReturnValue::Array(vec![]),
        },
        HashOps::HMSet(key, key_values) => {
            state.hashes.entry(key).or_default().extend(key_values);
            ReturnValue::Ok
        }
        HashOps::HLen(key) => read_hashes!(state, &key) 
            .map_or(0, |hash| hash.len() as Count)
            .into(),
        // HashOps::HLen(key) => read_hashes!(state, &key)
        //     .map(|hash| hash.len() as Count)
        //     .unwrap_or(0)
        //     .into(),
        // HashOps::HLen(key) => match read_hashes!(state, &key) {
        //     Some(hash) => ReturnValue::IntRes(hash.len() as Count),
        //     None => ReturnValue::IntRes(0),
        // }
        HashOps::HDel(key, fields) => match write_hashes!(state, &key) {
            Some(mut hash) => {
                let res = fields.iter().filter_map(|field| hash.remove(field)).count();
                ReturnValue::IntRes(res as Count)
            }
            None => ReturnValue::IntRes(0),
        },
    }
}