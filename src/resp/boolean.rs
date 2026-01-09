use super::preludes::*;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct RespBoolean {
    pub value: bool,
}

impl RespBoolean {
    pub fn new(value: bool) -> Self {
        Self { value }
    }
}
// | Boolean       | `#`    | "#t\r\n" (true) 或 "#f\r\n" (false)                            |
impl RespDecode for RespBoolean {
    const PREFIX: &'static str = "#";
    const TYPE: &'static str = "RespBoolean";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let next_byte = consume_byte_mut(buf, Self::PREFIX, Self::TYPE)?;
        let value = match next_byte.as_ref() {
            b"t" => true,
            b"f" => false,
            _ => {
                return Err(RespError::InvalidFrameType(format!(
                    "{} expect: #t or #f, but got {:?}",
                    Self::TYPE,
                    next_byte
                )));
            }
        };
        Ok(RespBoolean::new(value))
    }
    fn expect_length(_buf: &[u8]) -> Result<usize, RespError> {
        Ok(4)
    }
}

impl RespEncode for RespBoolean {
    fn encode(&self) -> Vec<u8> {
        if self.value {
            b"#t\r\n".to_vec()
        } else {
            b"#f\r\n".to_vec()
        }
    }
}
#[cfg(test)]
mod tests {
    use super::{super::*, *};
    #[test]
    fn test_boolean_encode() {
        let true_boolean = RespBoolean::new(true);
        let frame: RespFrame = true_boolean.into();
        let encoded = frame.encode();
        assert_eq!(encoded, b"#t\r\n");

        let false_boolean = RespBoolean::new(false);
        let frame: RespFrame = false_boolean.into();
        let encoded = frame.encode();
        assert_eq!(encoded, b"#f\r\n");
    }
    #[test]
    fn test_boolean_decode() -> anyhow::Result<()> {
        let true_boolean = RespBoolean::new(true);
        let frame: RespFrame = true_boolean.into();
        let encoded = frame.encode();
        let mut encoded = BytesMut::from(encoded.as_slice());
        let decoded = RespBoolean::decode(&mut encoded)?;
        assert_eq!(decoded, RespBoolean::new(true));

        let false_boolean = RespBoolean::new(false);
        let frame: RespFrame = false_boolean.into();
        let encoded = frame.encode();
        let mut encoded = BytesMut::from(encoded.as_slice());
        let decoded = RespBoolean::decode(&mut encoded)?;
        assert_eq!(decoded, RespBoolean::new(false));

        Ok(())
    }
}
