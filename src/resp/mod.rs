// * RESP 协议格式汇总:
// | 类型          | 起始符  | 格式样例                                                        |
// |---------------|--------|----------------------------------------------------------------|
// | Simple String | `+`    | "+OK\r\n"                                                      |
// | Error         | `-`    | "-Error message\r\n"                                           |
// | Integer       | `:`    | ":[<+|->]<value>\r\n"                                          |
// | Bulk String   | `$`    | "$<length>\r\n<data>\r\n"                                      |
// | Null Bulk Str | `!`    | "!<length>\r\n<error>\r\n"                                     |
// | Null          | `_`    | "_\r\n"                                                        |
// | Array         | `*`    | "*<count>\r\n<element-1>...<element-n>"                        |
// | Boolean       | `#`    | "#t\r\n" (true) 或 "#f\r\n" (false)                            |
// | Map           | `%`    | "%<count>\r\n<key1><val1>...<keyN><valN>"                      |
// | Set           | `~`    | "~<count>\r\n<element-1>...<element-n>"                        |
// | Double        | `,`    | ",[<+|->]<integral>[.<fractional>][<E|e>[sign]<exponent>]\r\n" |
// * 示例 (GET hello): "*2\r\n$3\r\nget\r\n$5\r\nhello\r\n"
use bytes::BytesMut;
use enum_dispatch::enum_dispatch;
mod decode;
mod preludes;
// 添加新模块
mod array;
mod boolean;
mod bulk_string;
mod double;
mod error;
mod integer;
mod map;
mod null;
mod resp_frame;
mod set;
mod simple_string;
// 导出新模块中的结构体
pub use array::RespArray;
pub use boolean::RespBoolean;
pub use bulk_string::{BulkString, NullBulkString};
pub use double::RespDouble;
pub use error::SimpleError;
pub use integer::RespInteger;
pub use map::RespMap;
pub use null::RespNull;
pub use resp_frame::{RespFrame, RespNullArray};
pub use set::RespSet;
pub use simple_string::SimpleString;

#[enum_dispatch]
pub trait RespEncode {
    fn encode(&self) -> Vec<u8>;
}

pub trait RespDecode: Sized {
    const PREFIX: &'static str = "";
    const TYPE: &'static str = "";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError>;
    fn expect_length(buf: &[u8]) -> Result<usize, RespError>;
}
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum RespError {
    #[error("invalid resp frame: {0}")]
    InvalidFrame(String),
    #[error("invalid resp frame type: {0}")]
    InvalidFrameType(String),
    #[error("invalid resp frame length: {0}")]
    InvalidFrameLength(String),
    #[error("not complete frame")]
    NotComplete,
    #[error("from utf8 error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error("parse int error: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("parse float error: {0}")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("from utf8 error: {0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
}
