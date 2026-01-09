use super::preludes::*;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct BulkString {
    pub content: Option<Vec<u8>>,
}

impl BulkString {
    pub fn new(content: Vec<u8>) -> Self {
        Self {
            content: Some(content),
        }
    }

    pub fn null() -> Self {
        Self { content: None }
    }

    pub fn from_string(content: &[u8]) -> Self {
        Self {
            content: Some(content.to_vec()),
        }
    }
}

// | Bulk String   | `$`    | "$<length>\r\n<data>\r\n"                                      |
impl RespDecode for BulkString {
    const PREFIX: &'static str = "$";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let (len_end, len) = extract_len(buf, Self::PREFIX, Self::TYPE)?;
        let remain = &buf[len_end + CRLF.len()..];
        if remain.len() < len_end + CRLF.len() {
            return Err(RespError::NotComplete);
        }
        let end = compute_end(&buf[len_end + CRLF.len()..], "", Self::TYPE)?;
        if end != len {
            return Err(RespError::InvalidFrameLength(format!(
                "{} expect: {:?} , but got {:?}",
                Self::TYPE,
                len,
                end
            )));
        }
        buf.advance(len_end + CRLF.len());

        Ok(BulkString::new(split_vec(buf, len)?))
    }
    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let (end, len) = extract_len(buf, Self::PREFIX, Self::TYPE)?;
        Ok(end + CRLF.len() + len + CRLF.len())
    }
}

impl RespEncode for BulkString {
    fn encode(&self) -> Vec<u8> {
        match &self.content {
            Some(data) => {
                let len_str = data.len().to_string();
                let mut result = Vec::with_capacity(1 + len_str.len() + 2 + data.len() + 2); // $ + len + \r\n + data + \r\n
                result.extend_from_slice(b"$");
                result.extend_from_slice(len_str.as_bytes());
                result.extend_from_slice(b"\r\n");
                result.extend_from_slice(data);
                result.extend_from_slice(b"\r\n");
                result
            }
            None => b"$-1\r\n".to_vec(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct NullBulkString;

// - null bulk string: "$-1\r\n"
impl RespEncode for NullBulkString {
    fn encode(&self) -> Vec<u8> {
        b"$-1\r\n".to_vec()
    }
}

impl RespDecode for NullBulkString {
    const PREFIX: &'static str = "$-1\r\n";
    const TYPE: &'static str = "RespNullBulkString";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        extract_fix(buf, Self::PREFIX, Self::TYPE)?;
        Ok(NullBulkString)
    }
    fn expect_length(_buf: &[u8]) -> Result<usize, RespError> {
        Ok(Self::PREFIX.len())
    }
}
impl From<&str> for RespFrame {
    fn from(s: &str) -> Self {
        BulkString::new(s.as_bytes().to_vec()).into()
    }
}

impl From<&[u8]> for RespFrame {
    fn from(s: &[u8]) -> Self {
        BulkString::new(s.to_vec()).into()
    }
}

impl<const N: usize> From<&[u8; N]> for RespFrame {
    fn from(s: &[u8; N]) -> Self {
        BulkString::new(s.to_vec()).into()
    }
}

#[cfg(test)]
mod tests {
    use super::{super::*, *};
    #[test]
    fn test_bulk_string_encode() {
        let bulk_string = BulkString::from_string(b"hello");
        let frame: RespFrame = bulk_string.into();
        let encoded = frame.encode();
        assert_eq!(encoded, b"$5\r\nhello\r\n");

        let null_bulk_string = BulkString::null();
        let frame: RespFrame = null_bulk_string.into();
        let encoded = frame.encode();
        assert_eq!(encoded, b"$-1\r\n");
    }
    #[test]
    fn test_bulk_string_decode() {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"$5\r\nhello\r\n".as_ref());
        let bulk_string = BulkString::decode(&mut buf).unwrap();
        assert_eq!(bulk_string, BulkString::from_string(b"hello"));
    }
    #[test]
    fn test_null_bulk_string_decode() {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"$-1\r\n".as_ref());
        let null_bulk_string = NullBulkString::decode(&mut buf).unwrap();
        assert_eq!(null_bulk_string, NullBulkString);
    }
    #[test]
    fn test_null_bulk_string_encode() {
        let null_bulk_string = NullBulkString;
        let frame: RespFrame = null_bulk_string.into();
        let encoded = frame.encode();
        assert_eq!(encoded, b"$-1\r\n");
    }
}
