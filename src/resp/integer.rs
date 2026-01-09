use super::preludes::*;

#[derive(Debug, Clone, PartialEq)]
pub struct RespInteger {
    pub value: i64,
}

impl RespInteger {
    pub fn new(value: i64) -> Self {
        Self { value }
    }
}
// | Integer       | `:`    | ":[<+|->]<value>\r\n"                                          |
impl RespDecode for RespInteger {
    const PREFIX: &'static str = ":";
    const TYPE: &'static str = "RespInteger";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        Ok(RespInteger::new(
            extract_end_string(buf, Self::PREFIX, Self::TYPE)?
                .parse()
                .unwrap(),
        ))
    }
    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        compute_end_with_crlf(buf, Self::PREFIX, Self::TYPE)
    }
}

impl RespEncode for RespInteger {
    fn encode(&self) -> Vec<u8> {
        let value_str = format!("{}", self.value);
        let mut result = Vec::with_capacity(1 + value_str.len() + 2); // : + value + \r\n
        result.extend_from_slice(b":");
        result.extend_from_slice(value_str.as_bytes());
        result.extend_from_slice(b"\r\n");
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_integer_encode() {
        let integer = RespInteger::new(123);
        let frame: RespFrame = integer.into();
        let encoded = frame.encode();
        assert_eq!(encoded, b":123\r\n");

        let negative_integer = RespInteger::new(-456);
        let frame: RespFrame = negative_integer.into();
        let encoded = frame.encode();
        assert_eq!(encoded, b":-456\r\n");
    }

    #[test]
    fn test_integer_decode() {
        let mut buf = BytesMut::from(":123\r\n");
        let integer = RespInteger::decode(&mut buf).unwrap();
        assert_eq!(integer, RespInteger::new(123));

        buf.extend_from_slice(b":+123\r");
        let integer: Result<RespInteger, RespError> = RespInteger::decode(&mut buf);
        assert_eq!(integer, Err(RespError::NotComplete));

        buf.extend_from_slice(b"\n");
        let integer: Result<RespInteger, RespError> = RespInteger::decode(&mut buf);
        assert_eq!(integer, Ok(RespInteger::new(123)));

        buf.extend_from_slice(b":-123\r\n");
        let integer = RespInteger::decode(&mut buf);
        assert_eq!(integer, Ok(RespInteger::new(-123)));

        buf.extend_from_slice(b"+OK");
        let integer = RespInteger::decode(&mut buf);
        assert_eq!(
            integer,
            Err(RespError::InvalidFrameType(format!(
                "RespInteger expect: : , but got {:?}",
                buf.as_ref()
            )))
        );
    }
}
