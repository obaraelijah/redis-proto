use crate::keys::KeyOps;

#[derive(Debug, Clone)]
pub enum Ops {
    Keys(KeyOps),
}

use smallvec::SmallVec;
const DEFAULT_SMALL_VEC_SIZE: usize = 2;
pub type RVec<T> = SmallVec<[T; DEFAULT_SMALL_VEC_SIZE]>;