use super::preludes::*;
#[derive(Debug, Clone, PartialEq)]
pub struct RespNull;
impl RespNull {
    const PREFIX: &'static str = "_\r\n";
}
impl RespDecode for RespNull {
    const PREFIX: &'static str = "_\r\n";
    const TYPE: &'static str = "RespNull";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        extract_fix(buf, Self::PREFIX, Self::TYPE)?;
        Ok(Self)
    }
    fn expect_length(_buf: &[u8]) -> Result<usize, RespError> {
        Ok(Self::PREFIX.len())
    }
}
impl RespEncode for RespNull {
    fn encode(&self) -> Vec<u8> {
        b"_\r\n".to_vec()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_null_encode() {
        let null = RespNull;
        let frame: RespFrame = null.into();
        let encoded = frame.encode();
        assert_eq!(encoded, b"_\r\n");
    }
}
