use std::fmt::Debug;
use std::convert::TryFrom;
use bytes::Bytes;

use crate::bloom::{bloom_interact, BloomOps};
use crate::hashes::{hash_interact, HashOps};
use crate::hyperloglog::{hyperloglog_interact, HyperLogLogOps};
use crate::keys::{key_interact, KeyOps};
use crate::lists::{list_interact, ListOps};
use crate::sets::{set_interact, SetOps};
use crate::sorted_sets::{zset_interact, ZSetOps};
use crate::stack::{stack_interact, StackOps};
use crate::misc::MiscOps;
use crate::types::{ReturnValue, StateRef, StateStoreRef};

use crate::types::{Count, Index, Key, RedisValueRef, Score, UTimeout, Value};

#[derive(Debug, Clone)]
pub enum Ops {
    Keys(KeyOps),
    Sets(SetOps),
    Lists(ListOps),
    Misc(MiscOps),
    Hashes(HashOps),
    ZSets(ZSetOps),
    Stacks(StackOps),
    Blooms(BloomOps),
    HyperLogLogs(HyperLogLogOps),
}

/// Top level interaction function. Used by the server to run
/// operations against state.
pub async fn op_interact(op: Ops, state: StateRef) -> ReturnValue {
    match op {
        Ops::Keys(op) => key_interact(op, state).await,
        Ops::Sets(op) => set_interact(op, state).await,
        Ops::Lists(op) => list_interact(op, state).await,
        Ops::Hashes(op) => hash_interact(op, state).await,
        Ops::ZSets(op) => zset_interact(op, state).await,
        Ops::Stacks(op) => stack_interact(op, state).await,
        Ops::Blooms(op) => bloom_interact(op, state).await,
        Ops::HyperLogLogs(op) => hyperloglog_interact(op, state).await,
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
        match r {
            RedisValueRef::Int(e) => Ok(*e as Count),
            // TODO: Not copy here
            RedisValueRef::BulkString(s) => match String::from_utf8(s.to_owned().to_vec()) {
                Ok(s) => s.parse().map_err(|_| OpsError::InvalidType),
                Err(_) => Err(OpsError::InvalidType),
            },
            _ => Err(OpsError::InvalidType),
        }
    }
}

/// Ensure the passed collection has an even number of arguments.
#[inline]
fn ensure_even<T>(v: &[T]) -> Result<(), OpsError> {
    if v.len() % 2 != 0 {
        return Err(OpsError::InvalidArgPattern(
            "even number of arguments required!",
        ));
    }
    Ok(())
}

use smallvec::SmallVec;
const DEFAULT_SMALL_VEC_SIZE: usize = 2;
pub type RVec<T> = SmallVec<[T; DEFAULT_SMALL_VEC_SIZE]>;

fn collect_from_tail<'a, ValueType>(tail: &[&'a RedisValueRef]) -> Result<RVec<ValueType>, OpsError>
where
    ValueType: TryFrom<&'a RedisValueRef, Error = OpsError>,
{
    let mut items: SmallVec<[ValueType; DEFAULT_SMALL_VEC_SIZE]> =SmallVec::new(); 
    for item in tail.iter() {
        let value = ValueType::try_from(item)?;
        items.push(value);
    }
    Ok(items)
}

fn values_from_tail<'a, ValueType>(tail: &[&'a RedisValueRef]) -> Result<Vec<ValueType>, OpsError>
where
    ValueType: TryFrom<&'a RedisValueRef, Error = OpsError>,
{
    let mut items: Vec<ValueType> = Vec::new();
    for item in tail.iter() {
        let value = ValueType::try_from(item)?;
        items.push(value);
    }
    Ok(items)
}

/// Verify the exact size of a sequence.
/// Useful for some commands that require an exact number of arguments (like get and set)
fn verify_size<T>(v: &[T], size: usize) -> Result<(), OpsError> {
    if v.len() != size {
        return Err(OpsError::WrongNumberOfArgs(size, v.len()));
    }
    Ok(())
}

/// Get a tuple of (KeyType, ValueType)
/// Mainly used for the thousand 2-adic ops
fn get_key_and_value<'a, KeyType, ValueType>(
    array: &'a [RedisValueRef],
) -> Result<(KeyType, ValueType), OpsError>
where
    KeyType: TryFrom<&'a RedisValueRef, Error = OpsError>,
    ValueType: TryFrom<&'a RedisValueRef, Error = OpsError>,
{
    if array.len() < 3 {
        return Err(OpsError::WrongNumberOfArgs(2, array.len() - 1));
    }
    let key = KeyType::try_from(&array[1])?;
    let val = ValueType::try_from(&array[2])?;
    Ok((key, val))
}

/// Transform &[RedisValueRef] into (KeyType, Vec<TailType>)
/// Used for commands like DEL arg1 arg2...
fn get_key_and_tail<'a, KeyType, TailType>(
    array: &'a [RedisValueRef],
) -> Result<(KeyType, RVec<TailType>), OpsError>
where
    KeyType: TryFrom<&'a RedisValueRef, Error = OpsError>,
    TailType: TryFrom<&'a RedisValueRef, Error = OpsError>,
{
    if array.len() < 3 {
        return Err(OpsError::WrongNumberOfArgs(3, array.len()));
    }
    let set_key = KeyType::try_from(&array[1])?;
    let mut tail = RVec::new();
    for tail_item in array.iter().skip(2) {
        let tmp = TailType::try_from(tail_item)?;
        tail.push(tmp)
    }
    Ok((set_key, tail))
}

/// Transform a sequence of [Key1, Val1, Key2, Val2, ...] -> Vec<(Key, Value)>
fn get_key_value_pairs<'a, KeyType, ValueType>(
    tail: &[&'a RedisValueRef],
) -> Result<RVec<(KeyType, ValueType)>, OpsError>
where
    KeyType: TryFrom<&'a RedisValueRef, Error = OpsError> + Debug,
    ValueType: TryFrom<&'a RedisValueRef, Error = OpsError> + Debug,
{
    ensure_even(tail)?;
    let keys = tail.iter().step_by(2);
    let vals = tail.iter().skip(1).step_by(2);
    let mut ret = RVec::new();
    for (&key, &val) in keys.zip(vals) {
        let key = KeyType::try_from(key)?;
        let val = ValueType::try_from(val)?;
        ret.push((key, val))
    }
    Ok(ret)
}

/// Convenience macro to automatically construct the right variant
/// of Ops.
macro_rules! ok {
    (KeyOps::$OpName:ident($($OpArg:expr),*)) => {
        Ok(Ops::Keys(KeyOps::$OpName($( $OpArg ),*)))
    };
    (MiscOps::$OpName:ident($($OpArg:expr),*)) => {
        Ok(Ops::Misc(MiscOps::$OpName($( $OpArg ),*)))
    };
    (MiscOps::$OpName:ident) => {
        Ok(Ops::Misc(MiscOps::$OpName))
    };
    (SetOps::$OpName:ident($($OpArg:expr),*)) => {
        Ok(Ops::Sets(SetOps::$OpName($( $OpArg ),*)))
    };
    (HashOps::$OpName:ident($($OpArg:expr),*)) => {
        Ok(Ops::Hashes(HashOps::$OpName($( $OpArg ),*)))
    };
    (ListOps::$OpName:ident($($OpArg:expr),*)) => {
        Ok(Ops::Lists(ListOps::$OpName($( $OpArg ),*)))
    };
    (ZSetOps::$OpName:ident($($OpArg:expr),*)) => {
        Ok(Ops::ZSets(ZSetOps::$OpName($( $OpArg ),*)))
    };
    (StackOps::$OpName:ident($($OpArg:expr),*)) => {
        Ok(Ops::Stacks(StackOps::$OpName($( $OpArg ),*)))
    };
    (BloomOps::$OpName:ident($($OpArg:expr),*)) => {
        Ok(Ops::Blooms(BloomOps::$OpName($( $OpArg ),*)))
    };
    (HyperLogLogOps::$OpName:ident($($OpArg:expr),*)) => {
        Ok(Ops::HyperLogLogs(HyperLogLogOps::$OpName($( $OpArg ),*)))
    };
}

fn translate_array(array: &[RedisValueRef], state_store: StateStoreRef) -> Result<Ops, OpsError> {
    let head = Value::try_from(&array[0])?;
    let head_s = String::from_utf8_lossy(&head);
    let tail: Vec<&RedisValueRef> = array.iter().skip(1).collect();
    match head_s.to_lowercase().as_ref() {
        "ping" => ok!(MiscOps::Pong()),
        "keys" => ok!(MiscOps::Keys()),
        "flushall" => ok!(MiscOps::FlushAll()),
        "flushdb" => ok!(MiscOps::FlushDB()),
    }
    // Key-Value
    "set" => {
        let (key, val) = get_key_and_value(array)?;
        ok!(KeyOps::Set(key, val))
    }
}