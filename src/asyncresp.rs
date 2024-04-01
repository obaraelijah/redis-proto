use memchr::memchr;
use std::convert::From;
use std::str;

use crate::types::{RedisValueRef, NULL_ARRAY, NULL_BULK_STRING};

use bytes::{Bytes, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

#[derive(Debug)]
pub enum RESPError {
    UnexpectedEnd,
    UnknownStartingByte,
    IOError(std::io::Error),
    IntParseFailure,
    BadBulkStringSize(i64),
    BadArraySize(i64),
}

impl From<std::io::Error> for RESPError {
    fn from(error: std::io::Error) -> Self {
        RESPError::IOError(error)
    }
}

enum RedisBufSplit {
    String(BufSplit),
    Error(BufSplit),
    Int(i64),
    Array(Vec<RedisBufSplit>),
    NullArray,
    NullBulkString,
}

#[derive(Default)]
pub struct RespParser;

impl RedisBufSplit {
    fn redis_value(self, buf: &Bytes) -> RedisValueRef {
        match self {
            // bfs is BufSplit(start, end), which has the as_bytes
            RedisBufSplit::String(bfs) => RedisValueRef::BulkString(bfs.as_bytes(buf)),
            RedisBufSplit::Error(bfs) => RedisValueRef::Error(bfs.as_bytes(buf)),
            RedisBufSplit::Array(arr) => {
                RedisValueRef::Array(arr.into_iter().map(|bfs| bfs.redis_value(buf)).collect())
            }
            RedisBufSplit::NullArray => RedisValueRef::NullArray,
            RedisBufSplit::NullBulkString => RedisValueRef::NullBulkString,
            RedisBufSplit::Int(i) => RedisValueRef::Int(i),
        }
    }
}

type RedisResult = Result<Option<(usize, RedisBufSplit)>, RESPError>;

/// Fundamental struct for viewing byte slices
///
/// Used for zero-copy redis values.
struct BufSplit(usize, usize); //useful for creating a view into a larger byte slice without copying the data

impl BufSplit {
    /// Get a lifetime appropriate slice of the underlying buffer.
    ///
    /// Constant time.
    #[inline]
    fn as_slice<'a>(&self, buf: &'a BytesMut) -> &'a [u8] {
        &buf[self.0..self.1]
    }

    /// Get a Bytes object representing the appropriate slice
    /// of bytes.
    ///
    /// Constant time.
    fn as_bytes(&self, buf: &Bytes) -> Bytes {
        buf.slice(self.0..self.1)
    }
}
/// Get a word from `buf` starting at `pos`
#[inline]
fn word(buf: &BytesMut, pos: usize) -> Option<(usize, BufSplit)> {
    // We're at the edge of `buf`, so we can't find a word.
    if buf.len() <= pos {
        return None;
    }
    // Find the position of the b'\r'
    memchr(b'\r', &buf[pos..]).and_then(|end| {
        if end + 1 < buf.len() {
            // pos + end == first index of b'\r' after `pos`
            // pos + end + 2 == ..word\r\n<HERE> -- skip to after CLRF
            Some((pos + end + 2, BufSplit(pos, pos + end)))
        } else {
            // Edge case: We received just enough bytes from the client
            // to get the \r but not the \n
            None
        }
    })
}

fn int(buf: &BytesMut, pos: usize) -> Result<Option<(usize, i64)>, RESPError> {
    match word(buf, pos) {
        Some((pos, word)) => {
            // word.as_slice(buf) is the method call BufSplit::as_slice(&self, &BytesMut) to access the byte slice.
            let s = str::from_utf8(word.as_slice(buf)).map_err(|_| RESPError::IntParseFailure)?;
            // Convert the string to an i64. Note the `?` for early returns.
            let i = s.parse().map_err(|_| RESPError::IntParseFailure)?;
            Ok(Some((pos, i)))
        }
        None => Ok(None),
    }
}

fn bulk_string(buf: &BytesMut, pos: usize) -> RedisResult {
    // `pos` returned by `int` is the first index of the string content.
    match int(buf, pos)? {
        // special case: redis defines a NullBulkString type, with length of -1.
        Some((pos, -1)) => Ok(Some((pos, RedisBufSplit::NullBulkString))),
        Some((pos, size)) if size >= 0 => {
            // We trust the client here, and directly calculate the end index of string (absolute w.r.t pos)
            let total_size = pos + size as usize;
            // The client hasn't sent us enough bytes
            if buf.len() < total_size + 2 {
                Ok(None)
            } else {
                let rs = RedisBufSplit::String(BufSplit(pos, total_size));
                // total_size + 2 == ...bulkstring\r\n<HERE> -- after CLRF
                Ok(Some((total_size + 2, rs)))
            }
        }
        // We recieved a garbage size (size < -1), so error out
        Some((_pos, bad_size)) => Err(RESPError::BadBulkStringSize(bad_size)),
        // Not enough bytes to parse an int (i.e. no CLRF to delimit the int)
        None => Ok(None),
    }
}

#[allow(clippy::unnecessary_wraps)]
fn simple_string(buf: &BytesMut, pos: usize) -> RedisResult {
    Ok(word(buf, pos).map(|(pos, word)| (pos, RedisBufSplit::String(word))))
}

#[allow(clippy::unnecessary_wraps)]
fn error(buf: &BytesMut, pos: usize) -> RedisResult {
    Ok(word(buf, pos).map(|(pos, word)| (pos, RedisBufSplit::Error(word))))
}

fn resp_int(buf: &BytesMut, pos: usize) -> RedisResult {
    Ok(int(buf, pos)?.map(|(pos, int)| (pos, RedisBufSplit::Int(int))))
}

fn array(buf: &BytesMut, pos: usize) -> RedisResult {
    match int(buf, pos)? {
        // Not enough bytes to determine the array size
        None => Ok(None),
        // special value: NullArray. Has size -1.
        Some((pos, -1)) => Ok(Some((pos, RedisBufSplit::NullArray))),
        Some((pos, num_elements)) if num_elements >= 0 => {
            // As we're recieving a dynamic number of elements, we need to heap allocate our BufSplits.
            let mut values = Vec::with_capacity(num_elements as usize);
            // We're going to forward iterate on `curr_pos`
            let mut curr_pos = pos;
            for _ in 0..num_elements {
                // Mutual Recursion! We need to parse the value at `curr_pos`
                match parse(buf, curr_pos)? {
                    // We got a value, so add it to the `values` vector and
                    // update `curr_pos`.
                    Some((new_pos, value)) => {
                        curr_pos = new_pos;
                        values.push(value);
                    }
                    // Not enough bytes. Abandon parsing and free vec.
                    None => return Ok(None),
                }
            }
            // We had enough bytes to fully parse the array! Return it.
            Ok(Some((curr_pos, RedisBufSplit::Array(values))))
        }
        // Client sent us a garbage size (num_elements < -1)
        Some((_pos, bad_num_elements)) => Err(RESPError::BadArraySize(bad_num_elements)),
    }
}

fn parse(buf: &BytesMut, pos: usize) -> RedisResult {
    if buf.is_empty() {
        return Ok(None);
    }

    match buf[pos] {
        b'+' => simple_string(buf, pos + 1),
        b'-' => error(buf, pos + 1),
        b'$' => bulk_string(buf, pos + 1),
        b':' => resp_int(buf, pos + 1),
        b'*' => array(buf, pos + 1),
        _ => Err(RESPError::UnknownStartingByte),
    }
}

impl Decoder for RespParser {
    type Item = RedisValueRef;
    type Error = RESPError;
    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if buf.is_empty() {
            return Ok(None);
        }

        match parse(buf, 0)? {
            Some((pos, value)) => {
                // We parsed a value! Shave off the bytes so tokio can continue filling the buffer.
                let our_data = buf.split_to(pos);
                Ok(Some(value.redis_value(&our_data.freeze())))
            }
            None => Ok(None),
        }
    }
}

impl Encoder<RedisValueRef> for RespParser {
    type Error = std::io::Error;

    fn encode(&mut self, item: RedisValueRef, dst: &mut BytesMut) -> std::io::Result<()> {
        write_redis_value(item, dst);
        Ok(())
    }
}

fn write_redis_value(item: RedisValueRef, dst: &mut BytesMut) {
    match item {
        RedisValueRef::Error(e) => {
            dst.extend_from_slice(b"-");
            dst.extend_from_slice(&e);
            dst.extend_from_slice(b"\r\n");
        }
        RedisValueRef::ErrorMsg(e) => {
            dst.extend_from_slice(b"-");
            dst.extend_from_slice(&e);
            dst.extend_from_slice(b"\r\n");
        }
        RedisValueRef::SimpleString(s) => {
            dst.extend_from_slice(b"+");
            dst.extend_from_slice(&s);
            dst.extend_from_slice(b"\r\n");
        }
        RedisValueRef::BulkString(s) => {
            dst.extend_from_slice(b"$");
            dst.extend_from_slice(s.len().to_string().as_bytes());
            dst.extend_from_slice(b"\r\n");
            dst.extend_from_slice(&s);
            dst.extend_from_slice(b"\r\n");
        }
        RedisValueRef::Array(array) => {
            dst.extend_from_slice(b"*");
            dst.extend_from_slice(array.len().to_string().as_bytes());
            dst.extend_from_slice(b"\r\n");
            for redis_value in array {
                write_redis_value(redis_value, dst);
            }
        }
        RedisValueRef::Int(i) => {
            dst.extend_from_slice(b":");
            dst.extend_from_slice(i.to_string().as_bytes());
            dst.extend_from_slice(b"\r\n");
        }
        RedisValueRef::NullBulkString => dst.extend_from_slice(NULL_BULK_STRING.as_bytes()),
        RedisValueRef::NullArray => dst.extend_from_slice(NULL_ARRAY.as_bytes()),
    }
}
