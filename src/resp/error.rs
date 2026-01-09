use super::preludes::*;

#[derive(Debug, Clone, PartialEq)]
pub struct SimpleError {
    pub msg: String,
}

impl SimpleError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self { msg: msg.into() }
    }
}
// | Error         | `-`    | "-Error message\r\n"                                           |
impl RespDecode for SimpleError {
    const PREFIX: &'static str = "-";
    const TYPE: &'static str = "SimpleError";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        Ok(SimpleError::new(extract_end_string(
            buf,
            Self::PREFIX,
            Self::TYPE,
        )?))
    }
    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        compute_end_with_crlf(buf, Self::PREFIX, Self::TYPE)
    }
}

impl RespEncode for SimpleError {
    fn encode(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(1 + self.msg.len() + 2); // - + msg + \r\n
        result.extend_from_slice(b"-");
        result.extend_from_slice(self.msg.as_bytes());
        result.extend_from_slice(b"\r\n");
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_error_encode() {
        let error = SimpleError::new("Error message");
        let frame: RespFrame = error.into();
        let encoded = frame.encode();
        assert_eq!(encoded, b"-Error message\r\n");
    }
    #[test]
    fn test_simple_error_decode() {
        let mut buf = BytesMut::from("-334\r\n");
        let simple_error = SimpleError::decode(&mut buf).unwrap();
        assert_eq!(simple_error, SimpleError::new("334".to_string()));

        buf.extend_from_slice(b"-334\r");
        let simple_error: Result<SimpleError, RespError> = SimpleError::decode(&mut buf);
        assert_eq!(simple_error, Err(RespError::NotComplete));

        buf.extend_from_slice(b"\n");
        let simple_error: Result<SimpleError, RespError> = SimpleError::decode(&mut buf);
        assert_eq!(simple_error, Ok(SimpleError::new("334".to_string())));

        buf.extend_from_slice(b"+OK");
        let simple_error = SimpleError::decode(&mut buf);
        assert_eq!(
            simple_error,
            Err(RespError::InvalidFrameType(format!(
                "SimpleError expect: - , but got {:?}",
                buf.as_ref()
            )))
        );
    }
}
