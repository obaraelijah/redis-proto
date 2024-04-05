use std::collections::HashSet;

use crate::op_variants;
use crate::ops::RVec;
use crate::types::{Count, Key, ReturnValue, StateRef, Value};

op_variants! {
    SetOps,
    SAdd(Key, RVec<Value>),
    SMembers(Key),
    SCard(Key),
    SRem(Key, RVec<Value>),
    SDiff(RVec<Value>),
    SDiffStore(Key, RVec<Value>),
    SUnion(RVec<Value>),
    SUnionStore(Key, RVec<Value>),
    SInter(RVec<Value>),
    SInterStore(Key, RVec<Value>)
}

pub enum SetAction {
    Diff,
    Union,
    Inter,
}

make_reader!(sets, read_sets);
make_writer!(sets, write_sets);

fn many_set_op(state: &StateRef, keys: RVec<Key>, op: SetAction) -> Option<HashSet<Value>> {
    let sets_that_exist: Vec<_> = keys
        .iter()
        .filter(|&k| state.sets.contains_key(k))
        .collect();
    if sets_that_exist.is_empty() {
        return None;
    }
    #[allow(clippy::mutable_key_type)]
    let mut head: HashSet<Key> = state
        .sets
        .get_mut(sets_that_exist[0])
        .unwrap()
        .value_mut()
        .clone();
    // TODO: Make this _way_ cleaner.
    for set_key in sets_that_exist.into_iter().skip(1) {
        head = match op {
            SetAction::Diff => head
                .difference(state.sets.get(set_key).unwrap().value())
                .cloned()
                .collect(),
            SetAction::Union => head
                .union(state.sets.get(set_key).unwrap().value())
                .cloned()
                .collect(),
            SetAction::Inter => head
                .intersection(state.sets.get(set_key).unwrap().value())
                .cloned()
                .collect(),
        }
    }
    Some(head)
}

pub async fn set_interact(set_op: SetOps, state: StateRef) -> ReturnValue {
    match set_op {
        SetOps::SAdd(set_key, vals) => {
            let mut set = state.sets.entry(set_key).or_default();
            vals.into_iter()
                .fold(0, |acc, val| acc + set.insert(val) as Count)
                .into()
        }
        SetOps::SMembers(set_key) => read_sets!(state, &set_key)
            .map(|set| set.iter().cloned().collect())
            .unwrap_or_else(RVec::new)
            .into(),
        SetOps::SCard(set_key) => read_sets!(state, &set_key)
            .map(|set| set.len() as Count)
            .unwrap_or(0)
            .into(),
        SetOps::SRem(set_key, vals) => write_sets!(state, &set_key)
            .map(|mut set| {
                vals.into_iter()
                    .fold(0, |acc, val| acc + set.insert(val) as Count)
            })
            .unwrap_or(0)
            .into(),
        SetOps::SDiff(keys) => many_set_op(&state, keys, SetAction::Diff)
            .map(|set| set.into_iter().collect())
            .unwrap_or_else(RVec::new)
            .into(),
        SetOps::SUnion(keys) => many_set_op(&state, keys, SetAction::Union)
            .map(|set| set.into_iter().collect())
            .unwrap_or_else(RVec::new)
            .into(),
        SetOps::SInter(keys) => many_set_op(&state, keys, SetAction::Inter)
            .map(|set| set.into_iter().collect())
            .unwrap_or_else(RVec::new)
            .into(),
        SetOps::SDiffStore(to_store, keys) => match many_set_op(&state, keys, SetAction::Diff) {
            Some(hash_set) => {
                let hash_set_size = hash_set.len();
                write_sets!(state).insert(to_store, hash_set);
                ReturnValue::IntRes(hash_set_size as Count)
            }
            None => ReturnValue::IntRes(0),
        },
        SetOps::SUnionStore(to_store, keys) => match many_set_op(&state, keys, SetAction::Union) {
            Some(hash_set) => {
                let hash_set_size = hash_set.len();
                write_sets!(state).insert(to_store, hash_set);
                ReturnValue::IntRes(hash_set_size as Count)
            }
            None => ReturnValue::IntRes(0),
        },
        SetOps::SInterStore(to_store, keys) => match many_set_op(&state, keys, SetAction::Inter) {
            Some(hash_set) => {
                let hash_set_size = hash_set.len();
                write_sets!(state).insert(to_store, hash_set);
                ReturnValue::IntRes(hash_set_size as Count)
            }
            None => ReturnValue::IntRes(0),
        },
    }
}
