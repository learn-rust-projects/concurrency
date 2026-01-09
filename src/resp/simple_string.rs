use super::preludes::*;

#[derive(Debug, Clone, PartialEq)]
pub struct SimpleString {
    pub content: String,
}

impl SimpleString {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
        }
    }
}
impl Deref for SimpleString {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.content
    }
}

impl From<&str> for SimpleString {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl RespDecode for SimpleString {
    const PREFIX: &'static str = "+";
    const TYPE: &'static str = "SimpleString";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        Ok(SimpleString::new(extract_end_string(
            buf,
            Self::PREFIX,
            Self::TYPE,
        )?))
    }
    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        compute_end_with_crlf(buf, Self::PREFIX, Self::TYPE)
    }
}

impl RespEncode for SimpleString {
    fn encode(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(2 + self.content.len() + 2); // + + content + \r\n
        result.extend_from_slice(b"+");
        result.extend_from_slice(self.content.as_bytes());
        result.extend_from_slice(b"\r\n");
        result
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_simple_string_decode() {
        let mut buf = BytesMut::from("+OK\r\n");
        let simple_string = SimpleString::decode(&mut buf).unwrap();
        assert_eq!(simple_string, SimpleString::new("OK".to_string()));

        buf.extend_from_slice(b"+OK\r");
        let simple_string: Result<SimpleString, RespError> = SimpleString::decode(&mut buf);
        assert_eq!(simple_string, Err(RespError::NotComplete));

        buf.extend_from_slice(b"\n");
        let simple_string: Result<SimpleString, RespError> = SimpleString::decode(&mut buf);
        assert_eq!(simple_string, Ok(SimpleString::new("OK".to_string())));

        buf.extend_from_slice(b"-334");
        let simple_string = SimpleString::decode(&mut buf);
        assert_eq!(
            simple_string,
            Err(RespError::InvalidFrameType(format!(
                "SimpleString expect: + , but got {:?}",
                buf.as_ref()
            )))
        );
    }
}
