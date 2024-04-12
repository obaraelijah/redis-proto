use crate::{make_reader, op_variants};
use crate::types::{Key, ReturnValue, StateRef, Value};

op_variants! {
    StackOps,
}

make_reader!(stacks, read_stacks);

pub async fn stack_interact(stack_op: StackOps, state: StateRef) -> ReturnValue {
    todo!()
}