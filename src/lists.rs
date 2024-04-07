use crate::ops::RVec;
use crate::types::{Key, ReturnValue, StateRef, Value};
use crate::{make_reader, make_writer, op_variants};

op_variants! {
    ListOps,
    LPush(Key, RVec<Value>)
}

make_reader!(lists, read_lists);
make_writer!(lists, write_lists);

pub async fn list_interact(list_op: ListOps, state: StateRef) -> ReturnValue {
   unimplemented!()
}

