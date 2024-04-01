use bytes::Bytes;

// These types are used by state and ops to actually perform useful work.
pub type Value = Bytes;
// Key is the standard type to index our structures
pub type Key = Bytes;

/// RedisValueRef is the canonical type for values flowing
/// through the system. Inputs are converted into RedisValues,
/// and outputs are converted into RedisValues.
#[derive(PartialEq, Clone)]
pub enum RedisValueRef {
    String(Bytes),
    Error(Bytes),
    Int(i64),
    Array(Vec<RedisValueRef>),
    NullArray,
    NullBulkString,
    ErrorMsg(Vec<u8>), // This is not a RESP type. This is an redis-proto internal error type.
}


