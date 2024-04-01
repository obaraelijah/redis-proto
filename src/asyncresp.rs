use bytes::Bytes;

struct BufSplit(usize, usize); //useful for creating a view into a larger byte slice without copying the data

enum RedisBufSplit {
    String(BufSplit),
    Error(BufSplit),
    Int(i64),
    Array(Vec<RedisBufSplit>),
    NullArray,
    NullBulkString,
}