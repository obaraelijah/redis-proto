use crate::ops::RVec;
use crate::types::{Count, Index, Key, ReturnValue, Score, StateRef};
use crate::{make_reader, make_writer, op_variants};

op_variants! {
    ZSetOps,
    ZAdd(Key, RVec<(Score, Key)>),
    ZRem(Key, RVec<Key>)      
}

make_reader!(zsets, read_zsets);
make_writer!(zsets, write_zsets);

pub async fn zset_interact(zset_op: ZSetOps, state: StateRef) -> ReturnValue {
    match zset_op {
        ZSetOps::ZAdd(zset_key, member_scores) => {
            let mut zset = state.zsets.entry(zset_key).or_default();
            let num_added = zset.add(member_scores);
            ReturnValue::IntRes(num_added)
        }
        ZSetOps::ZRem(zset_key, keys) => write_zsets!(state, &zset_key)
        .map(|mut zset| zset.remove(&keys))
        .unwrap_or(0)
        .into()
    }
}

