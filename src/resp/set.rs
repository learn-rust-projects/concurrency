use super::preludes::*;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RespSet {
    pub elements: Vec<RespFrame>,
}

impl Deref for RespSet {
    type Target = Vec<RespFrame>;

    fn deref(&self) -> &Self::Target {
        &self.elements
    }
}
impl DerefMut for RespSet {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.elements
    }
}

impl RespSet {
    pub fn new(elements: Vec<RespFrame>) -> Self {
        Self { elements }
    }
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            elements: Vec::with_capacity(capacity),
        }
    }
}

// | Set           | `~`    | "~<count>\r\n<element-1>...<element-n>"                        |
impl RespDecode for RespSet {
    const PREFIX: &'static str = "~";
    const TYPE: &'static str = "RespSet";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let len = check_len(buf, Self::PREFIX, Self::TYPE)?;
        let mut set = RespSet::with_capacity(len);
        for _ in 0..len {
            let element = RespFrame::decode(buf)?;
            set.push(element);
        }

        Ok(set)
    }
    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let (end, len) = extract_len(buf, Self::PREFIX, Self::TYPE)?;
        calc_total_length(buf, end, len, Self::PREFIX)
    }
}

impl RespEncode for RespSet {
    fn encode(&self) -> Vec<u8> {
        let count_str = format!("{}", self.elements.len());
        let mut result = Vec::with_capacity(1 + count_str.len() + 2); // ~ + count + \r\n
        result.extend_from_slice(b"~");
        result.extend_from_slice(count_str.as_bytes());
        result.extend_from_slice(b"\r\n");

        for element in &self.elements {
            result.extend_from_slice(&element.encode());
        }
        result
    }
}
#[cfg(test)]
mod tests {
    use super::{super::*, *};

    #[test]
    fn test_set_encode() {
        let set = vec![
            RespFrame::SimpleString(SimpleString::new("value1")),
            RespFrame::Integer(RespInteger::new(123)),
            RespFrame::Double(RespDouble::new(4.14)),
            RespFrame::Boolean(RespBoolean::new(false)),
            RespFrame::Array(RespArray::new(Some(vec![
                RespFrame::SimpleString(SimpleString::new("nested1")),
                RespFrame::Integer(RespInteger::new(456)),
            ]))),
        ];
        let set_struct = RespSet::new(set);
        let frame: RespFrame = set_struct.into();
        let encoded = frame.encode();
        // Set encoding is order-independent, so we just check the format
        let encoded_str = String::from_utf8(encoded).unwrap();
        assert_eq!(
            encoded_str,
            "~5\r\n+value1\r\n:123\r\n,+4.14\r\n#f\r\n*2\r\n+nested1\r\n:456\r\n"
        );
    }
    #[test]
    fn test_set_decode() {
        let mut buf = BytesMut::new();
        buf.extend(b"~5\r\n+value1\r\n:123\r\n,+3.14\r\n#f\r\n*2\r\n+nested1\r\n:456\r\n".as_ref());
        let set = RespSet::decode(&mut buf).unwrap();
        assert_eq!(set.len(), 5);
    }
}
