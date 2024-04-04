use std::collections::HashSet;
use std::os::linux::raw::stat;

use crate::op_variants;
use crate::ops::RVec;
use crate::types::{Count, Key, ReturnValue, StateRef, Value};

op_variants! {
    SetOps,
    SAdd(Key, RVec<Value>)
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