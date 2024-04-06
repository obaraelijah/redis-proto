use crate::ops::RVec;
use crate::types::{Count, Index, Key, ReturnValue, StateRef, UTimeout, Value};
use crate::{make_reader, make_writer, op_variants};

op_variants! {
    ListOps,
}

pub async fn list_interact(list_op: ListOps, state: StateRef) -> ReturnValue {
    unimplemented!()
}