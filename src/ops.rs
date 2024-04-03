use crate::keys::KeyOps;

#[derive(Debug, Clone)]
pub enum Ops {
    Keys(KeyOps),
}
