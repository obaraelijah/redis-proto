use crate::types::{Key, RedisBool, ReturnValue, StateRef, Value};
use crate::make_reader;
use growable_bloom_filter::GrowableBloom;

op_variants! {
    BloomOps,
    BInsert(Key, Value),
    BContains(Key, Value)
}

const DESIRED_FAILURE_RATE: f64 = 0.05;
const EST_INSERTS: usize = 10;

make_reader!(blooms, read_blooms);

pub async fn bloom_interact(bloom_op: BloomOps, state: StateRef) -> ReturnValue {
    match bloom_op {
        BloomOps::BInsert(bloom_key, value) => {
            state
                .blooms
                .entry(bloom_key)
                .or_insert_with(|| GrowableBloom::new(DESIRED_FAILURE_RATE, EST_INSERTS))
                .insert(value);
            ReturnValue::Ok
        }
        BloomOps::BContains(bloom_key, value) => read_blooms!(state, &bloom_key)
            .map(|bloom| bloom.contains(value) as RedisBool)
            .unwrap_or(0)
            .into(),
    }
}