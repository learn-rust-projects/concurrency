use enum_dispatch::enum_dispatch;

use super::preludes::*;
// 导出新模块中的结构体
pub use super::{
    array::{RespArray, RespNullArray},
    boolean::RespBoolean,
    bulk_string::{BulkString, NullBulkString},
    double::RespDouble,
    error::SimpleError,
    integer::RespInteger,
    map::RespMap,
    null::RespNull,
    set::RespSet,
    simple_string::SimpleString,
};
#[enum_dispatch(RespEncode)]
#[derive(Debug, Clone, PartialEq)]
pub enum RespFrame {
    SimpleString(SimpleString),
    SimpleError(SimpleError),

    Integer(RespInteger),
    BulkString(BulkString),
    NullBulkString(NullBulkString),
    Array(RespArray),
    Boolean(RespBoolean),
    Double(RespDouble),
    Map(RespMap),
    Set(RespSet),
    RespNull(RespNull),
    RespNullArray(RespNullArray),
}

impl RespDecode for RespFrame {
    const PREFIX: &'static str = "";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let mut iter = buf.iter().peekable();
        match iter.peek() {
            Some(b'+') => {
                let frame = SimpleString::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'-') => {
                let frame = SimpleError::decode(buf)?;
                Ok(frame.into())
            }
            Some(b':') => {
                let frame = RespInteger::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'$') => match NullBulkString::decode(buf) {
                Ok(frame) => Ok(frame.into()),
                Err(RespError::NotComplete) => Err(RespError::NotComplete),
                Err(_) => {
                    let frame = BulkString::decode(buf)?;
                    Ok(frame.into())
                }
            },
            Some(b'*') => match RespNullArray::decode(buf) {
                Ok(frame) => Ok(frame.into()),
                Err(RespError::NotComplete) => Err(RespError::NotComplete),
                Err(_) => {
                    let frame = RespArray::decode(buf)?;
                    Ok(frame.into())
                }
            },
            Some(b'#') => {
                let frame = RespBoolean::decode(buf)?;
                Ok(frame.into())
            }
            Some(b',') => {
                let frame = RespDouble::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'%') => {
                let frame = RespMap::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'~') => {
                let frame = RespSet::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'_') => {
                let frame = RespNull::decode(buf)?;
                Ok(frame.into())
            }
            None => Err(RespError::NotComplete),
            _ => Err(RespError::InvalidFrameType(format!(
                "RespFrame expect: +, -, :, $, *, #, ,, %, ~, _, but got {:?}",
                buf
            ))),
        }
    }
    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let mut iter = buf.iter().peekable();
        match iter.peek() {
            Some(b'+') => SimpleString::expect_length(buf),
            Some(b'-') => SimpleError::expect_length(buf),
            Some(b':') => RespInteger::expect_length(buf),
            Some(b'$') => BulkString::expect_length(buf),
            Some(b'*') => RespArray::expect_length(buf),
            Some(b'#') => RespBoolean::expect_length(buf),
            Some(b',') => RespDouble::expect_length(buf),
            Some(b'%') => RespMap::expect_length(buf),
            Some(b'~') => RespSet::expect_length(buf),
            Some(b'_') => RespNull::expect_length(buf),
            _ => Err(RespError::NotComplete),
        }
    }
}
