use crate::ops::RVec;
use crate::types::{Count, Index, Key, ReturnValue, Score, StateRef};
use crate::{make_reader, make_writer, op_variants};

op_variants! {
    ZSetOps,
    ZAdd(Key, RVec<(Score, Key)>),
    ZRem(Key, RVec<Key>),
    ZRange(Key, Score, Score)
}

make_reader!(zsets, read_zsets);
make_writer!(zsets, write_zsets);

fn deal_with_negative_indices(coll_size: Count, bounds: (Index, Index)) -> (Index, Index) {
    let (start, end) = bounds;
    let start = if start < 0 { start + coll_size } else { start };
    let end = if end < 0 { end + coll_size } else { end };
    (start, end)
}

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
            .into(),
        ZSetOps::ZRange(zset_key, start, stop) => read_zsets!(state, &zset_key)
            .map(|zset| {
                let (start, stop) = deal_with_negative_indices(zset.card(), (start, stop));
                zset.range((start, stop))
                    .into_iter()
                    .map(|item| item.member)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
            .into(),
    }
}
