use bytes::{Bytes, BytesMut};
use memchr::memchr;

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

fn simple_string(buf: &BytesMut, pos: usize) -> RedisResult {
    match word(buf, pos) {
        Some((pos, word)) => Ok(Some((pos, RedisBufSplit::String(word)))),
        None => Ok(None),
    }
}

fn error(buf: &BytesMut, pos: usize) -> RedisResult {
    match word(buf, pos) {
        Some((pos, word)) => Ok(Some((pos, RedisBufSplit::Error(word)))),
        None => Ok(None),
    }
}