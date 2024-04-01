use bytes::{buf, Bytes, BytesMut};
use memchr::memchr;
use std::str;

#[derive(Debug)]
pub enum RESPError {
    UnexpectedEnd,
    UnknownStartingByte,
    IOError(std::io::Error),
    IntParseFailure,
    BadBulkStringSize(i64),
    BadArraySize(i64),
}
enum RedisBufSplit {
    String(BufSplit),
    Error(BufSplit),
    Int(i64),
    Array(Vec<RedisBufSplit>),
    NullArray,
    NullBulkString,
}

type RedisResult = Result<Option<(usize, RedisBufSplit)>, RESPError>;

struct BufSplit(usize, usize); //useful for creating a view into a larger byte slice without copying the data

impl BufSplit {
    /// Get a lifetime appropriate slice of the underlying buffer.
    ///
    /// Constant time.
    fn as_slice<'a>(&self, buf: &'a BytesMut) -> &'a [u8] {
        &buf[self.0..self.1]
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

fn simple_string(buf: &BytesMut, pos: usize) -> RedisResult {
    Ok(word(buf, pos).map(|(pos, word)| (pos, RedisBufSplit::String(word))))
}

fn error(buf: &BytesMut, pos: usize) -> RedisResult {
    Ok(word(buf, pos).map(|(pos, word)| (pos, RedisBufSplit::Error(word))))
}

fn resp_int(buf: &BytesMut, pos: usize) -> RedisResult {
    Ok(int(buf, pos)?.map(|(pos, int)| (pos, RedisBufSplit::Int(int))))
}
