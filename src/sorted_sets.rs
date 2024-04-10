use crate::ops::RVec;
use crate::types::{Count, Index, Key, ReturnValue, Score, StateRef};
use crate::{make_reader, make_writer, op_variants};

op_variants! {
    ZSetOps,      
}

pub async fn zset_interact(zset_op: ZSetOps, state: StateRef) -> ReturnValue {
    unimplemented!()
}