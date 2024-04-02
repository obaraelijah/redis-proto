use bytes::Bytes;
use dashmap::DashMap;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::File;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

// These types are used by state and ops to actually perform useful work.
pub type Value = Bytes;
// Key is the standard type to index our structures
pub type Key = Bytes;
/// Count is used for commands that count
pub type Count = i64;
/// Index is used to reprcent indices in structures
pub type Index = i64;

/// DumpTimeoutUnitpe alias.
pub type Dumpfile = Arc<Mutex<File>>;

/// RedisValueRef is the canonical type for values flowing
/// through the system. Inputs are converted into RedisValues,
/// and outputs are converted into RedisValues.
#[derive(PartialEq, Clone)]
pub enum RedisValueRef {
    BulkString(Bytes),
    SimpleString(Bytes),
    Error(Bytes),
    Int(i64),
    Array(Vec<RedisValueRef>),
    ErrorMsg(Vec<u8>), // This is not a RESP type. This is an redis-proto internal error type.
    NullArray,
    NullBulkString,
}

impl std::fmt::Debug for RedisValueRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RedisValueRef::BulkString(s) => write!(
                f,
                "RedisValueRef::BulkString({:?})",
                String::from_utf8_lossy(s)
            ),
            RedisValueRef::SimpleString(s) => write!(
                f,
                "RedisValueRef::SimpleString({:?})",
                String::from_utf8_lossy(s)
            ),
            RedisValueRef::Error(s) => {
                write!(f, "RedisValueRef::Error({:?})", String::from_utf8_lossy(s))
            }
            RedisValueRef::ErrorMsg(s) => write!(f, "RedisValueRef::ErrorMsg({:?})", s),

            RedisValueRef::Int(i) => write!(f, "RedisValueRef::Int({:?})", i),
            RedisValueRef::NullBulkString => write!(f, "RedisValueRef::NullBulkString"),
            RedisValueRef::NullArray => write!(f, "RedisValueRef::NullArray"),
            RedisValueRef::Array(arr) => {
                write!(f, "RedisValueRef::Array(")?;
                for item in arr {
                    write!(f, "{:?}", item)?;
                    write!(f, ",")?;
                }
                write!(f, ")")?;
                Ok(())
            }
        }
    }
}

/// Special constants in the RESP protocol.
pub const NULL_BULK_STRING: &str = "$-1\r\n";
pub const NULL_ARRAY: &str = "*-1\r\n";
pub const EMPTY_ARRAY: &str = "*0\r\n";

/// Canonical type for key-value storage
type KeyString = DashMap<Key, Value>;
/// Canonical type for key-set storage
type KeySet = DashMap<Key, HashSet<Value>>;

///Top level database struct
/// Holds all Stateref dbs, and will hand them out on request
#[derive(Default, Serialize, Deserialize)]
pub struct StateStore {
    pub states: DashMap<Index, StateRef>,
    #[serde(skip)]
    pub commands_threshold: u64,
    #[serde(skip)]
    pub commands_ran_since_save: AtomicU64,
    #[serde(skip)]
    pub memory_only: bool,
}

/// Reference type for `StateStore`
pub type StateStoreRef = Arc<StateStore>;

/// Reference type for `State`
pub type StateRef = Arc<State>;

/// Reference type for State
/// The state stored by redis-proto. These fields are the ones
/// used by the various datastructures files (keys.rs, etc)
#[derive(Default, Serialize, Deserialize)]
pub struct State {
    #[serde(default)]
    pub kv: KeyString,
    #[serde(default)]
    pub sets: KeySet,
}
