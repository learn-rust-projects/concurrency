use std::collections::BTreeMap;

use super::{SimpleString, preludes::*};
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RespMap {
    pub pairs: BTreeMap<String, RespFrame>,
}
impl Deref for RespMap {
    type Target = BTreeMap<String, RespFrame>;

    fn deref(&self) -> &Self::Target {
        &self.pairs
    }
}
impl DerefMut for RespMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.pairs
    }
}

// | Map           | `%`    | "%<count>\r\n<key1><val1>...<keyN><valN>"                      |
impl RespDecode for RespMap {
    const PREFIX: &'static str = "%";
    const TYPE: &'static str = "RespMaps";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let len = check_len(buf, Self::PREFIX, Self::TYPE)?;

        let mut pairs = RespMap::default();

        for _ in 0..len {
            let key = SimpleString::decode(buf)?; // Keys are encoded as strings in the map
            let value = RespFrame::decode(buf)?;

            pairs.insert(key.content, value);
        }
        Ok(pairs)
    }
    // todo
    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let (end, size) = extract_len(buf, Self::PREFIX, Self::TYPE)?;
        calc_total_length(buf, size, end, Self::PREFIX)
    }
}
// | Map           | `%`    | "%<count>\r\n<key1><val1>...<keyN><valN>"                      |
impl RespEncode for RespMap {
    fn encode(&self) -> Vec<u8> {
        let count_str = format!("{}", self.pairs.len());
        let mut result = Vec::with_capacity(1 + count_str.len() + 2); // % + count + \r\n
        result.extend_from_slice(b"%");
        result.extend_from_slice(count_str.as_bytes());
        result.extend_from_slice(b"\r\n");

        for (key, value) in &self.pairs {
            // Encode key as SimpleString
            let key_frame = RespFrame::SimpleString(SimpleString::new(key.clone()));
            result.extend_from_slice(&key_frame.encode());
            result.extend_from_slice(&value.encode());
        }
        result
    }
}
#[cfg(test)]
mod tests {
    use super::{super::*, *};
    #[test]
    fn test_map_encode() {
        let mut map = BTreeMap::new();
        map.insert("first".to_string(), RespInteger::new(123).into());
        map.insert(
            "second".to_string(),
            RespFrame::SimpleString(SimpleString::new("hello")),
        );
        map.insert(
            "third".to_string(),
            RespFrame::Double(RespDouble::new(4.14)),
        );
        map.insert(
            "fourth".to_string(),
            RespFrame::Boolean(RespBoolean::new(true)),
        );
        let map_struct = RespMap { pairs: map };
        let frame: RespFrame = map_struct.into();
        let encoded = frame.encode();
        // Map encoding is order-independent, so we just check the format
        let encoded_str = String::from_utf8(encoded).unwrap();
        assert_eq!(
            encoded_str,
            "%4\r\n+first\r\n:123\r\n+fourth\r\n#t\r\n+second\r\n+hello\r\n+third\r\n,+4.14\r\n"
        );
    }
    #[test]
    fn test_map_decode() {
        let encoded =
            b"%4\r\n+first\r\n:123\r\n+fourth\r\n#t\r\n+second\r\n+hello\r\n+third\r\n,+4.14\r\n";
        let mut buf = BytesMut::from(encoded.as_ref());
        let map = RespMap::decode(&mut buf).unwrap();
        assert_eq!(map.len(), 4);
        assert_eq!(map["first"], RespFrame::Integer(RespInteger::new(123)));
        assert_eq!(
            map["second"],
            RespFrame::SimpleString(SimpleString::new("hello"))
        );
        assert_eq!(map["third"], RespFrame::Double(RespDouble::new(4.14)));
        assert_eq!(map["fourth"], RespFrame::Boolean(RespBoolean::new(true)));
    }
}
