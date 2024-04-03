use crate::types::{RedisValueRef, ReturnValue, StateRef, Value, Count};
use crate::keys::{KeyOps, key_interact};

use std::convert::TryFrom;

#[derive(Debug, Clone)]
pub enum Ops {
    Keys(KeyOps),
}

pub async fn op_interact(op: Ops, state: StateRef) -> ReturnValue {
    match op {
        Ops::Keys(op) => key_interact(op, state).await,
        _ => unreachable!(),
    }
}

#[derive(Debug)]
pub enum OpsError {
    InvalidStart,
    Noop,
    UnknownOp,
    NotEnoughArgs(usize, usize), // req, given
    WrongNumberOfArgs(usize, usize),
    InvalidArgPattern(&'static str),
    InvalidType,
    SyntaxError,
    InvalidArgs(String),
}

// impl Error for OpsError {}

impl From<OpsError> for RedisValueRef {
    fn from(op: OpsError) -> RedisValueRef {
        match op {
            OpsError::InvalidStart => RedisValueRef::ErrorMsg(b"Invalid start!".to_vec()),
            OpsError::UnknownOp => RedisValueRef::ErrorMsg(b"Unknown Operation!".to_vec()),
            OpsError::InvalidArgPattern(explain) => {
                let f = format!("Invalid Arg Pattern, {}", explain);
                RedisValueRef::ErrorMsg(f.as_bytes().to_vec())
            }
            OpsError::NotEnoughArgs(req, given) => {
                let f = format!("Not enough arguments, {} required, {} given!", req, given);
                RedisValueRef::ErrorMsg(f.as_bytes().to_vec())
            }
            OpsError::WrongNumberOfArgs(required, given) => {
                let f = format!(
                    "Wrong number of arguments! ({} required, {} given)",
                    required, given
                );
                RedisValueRef::ErrorMsg(f.as_bytes().to_vec())
            }
            OpsError::InvalidType => RedisValueRef::ErrorMsg(b"Invalid Type!".to_vec()),
            OpsError::SyntaxError => RedisValueRef::ErrorMsg(b"Syntax Error!".to_vec()),
            OpsError::Noop => RedisValueRef::ErrorMsg(b"".to_vec()),
            OpsError::InvalidArgs(s) => RedisValueRef::ErrorMsg(s.as_bytes().to_vec()),
        }
    }
}

impl TryFrom<RedisValueRef> for Bytes {
    type Error = OpsError;

    fn try_from(r: RedisValueRef) -> Result<Value, Self::Error> {
        match r {
            RedisValueRef::BulkString(s) => Ok(s),
            _ => Err(OpsError::InvalidType),
        }
    }
}

impl TryFrom<&RedisValueRef> for Bytes {
    type Error = OpsError;

    fn try_from(r: &RedisValueRef) -> Result<Value, Self::Error> {
        match r {
            RedisValueRef::BulkString(r) => Ok(r.clone()),
            _ => Err(OpsError::InvalidType),
        }
    }
}

impl TryFrom<RedisValueRef> for String {
    type Error = OpsError;

    fn try_from(r: RedisValueRef) -> Result<String, Self::Error> {
        match r {
            RedisValueRef::BulkString(s) => Ok(String::from_utf8_lossy(&s).to_string()),
            _ => Err(OpsError::InvalidType),
        }
    }
}

impl TryFrom<&RedisValueRef> for String {
    type Error = OpsError;

    fn try_from(r: &RedisValueRef) -> Result<String, Self::Error> {
        String::try_from(r.clone())
    }
}

impl TryFrom<&RedisValueRef> for Count {
    type Error = OpsError;

    fn try_from(r: &RedisValueRef) -> Result<Count, Self::Error> {
       unimplemented!()
    }
}

use bytes::Bytes;
use smallvec::SmallVec;
const DEFAULT_SMALL_VEC_SIZE: usize = 2;
pub type RVec<T> = SmallVec<[T; DEFAULT_SMALL_VEC_SIZE]>;
