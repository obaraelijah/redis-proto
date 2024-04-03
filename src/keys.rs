use crate::op_variants;
use crate::types::{Key, Value};

// used to generate an enum with variants for different key operations.
op_variants! {
    KeyOps,
    Set(Key, Value),
    Get(Key)
}
