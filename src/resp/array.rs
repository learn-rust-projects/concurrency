use super::preludes::*;

#[derive(Debug, Clone, PartialEq)]
pub struct RespArray {
    pub elements: Option<Vec<RespFrame>>,
}
impl Deref for RespArray {
    type Target = Option<Vec<RespFrame>>;
    fn deref(&self) -> &Self::Target {
        &self.elements
    }
}
impl RespArray {
    pub fn new(elements: Option<Vec<RespFrame>>) -> Self {
        Self { elements }
    }

    pub fn from_vec(elements: Vec<RespFrame>) -> Self {
        Self {
            elements: Some(elements),
        }
    }

    pub fn empty() -> Self {
        Self {
            elements: Some(Vec::new()),
        }
    }

    pub fn null() -> Self {
        Self { elements: None }
    }
}
impl From<Vec<RespFrame>> for RespArray {
    fn from(elements: Vec<RespFrame>) -> Self {
        Self::from_vec(elements)
    }
}
// | Array         | `*`    | "*<count>\r\n<element-1>...<element-n>"                        |
impl RespDecode for RespArray {
    const PREFIX: &'static str = "*";
    const TYPE: &'static str = "RespArray";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let len = check_len(buf, Self::PREFIX, Self::TYPE)?;
        match len {
            0 => Ok(RespArray::null()),
            _ => {
                let mut elements = Vec::with_capacity(len);
                for _ in 0..len {
                    let element = RespFrame::decode(buf)?;
                    elements.push(element);
                }
                Ok(RespArray::from_vec(elements))
            }
        }
    }
    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let (end, len) = extract_len(buf, Self::PREFIX, Self::TYPE)?;
        calc_total_length(buf, end, len, Self::PREFIX)
    }
}

impl RespEncode for RespArray {
    fn encode(&self) -> Vec<u8> {
        match &self.elements {
            Some(elements) => {
                if elements.is_empty() {
                    return b"*0\r\n".to_vec();
                }

                // 首先对所有元素进行编码
                let encoded_elements: Vec<Vec<u8>> =
                    elements.iter().map(|element| element.encode()).collect();

                // 计算总容量
                let count_str = format!("{}", elements.len());
                let total_capacity = 1 + count_str.len() + 2; // * + count + \r\n

                let total_capacity = encoded_elements
                    .iter()
                    .fold(total_capacity, |acc, element| acc + element.len());

                let mut result = Vec::with_capacity(total_capacity);
                result.extend_from_slice(b"*");
                result.extend_from_slice(count_str.as_bytes());
                result.extend_from_slice(b"\r\n");

                for encoded_element in &encoded_elements {
                    result.extend_from_slice(encoded_element);
                }
                result
            }
            None => b"*-1\r\n".to_vec(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct RespNullArray;

// - null array: "*-1\r\n"
impl RespEncode for RespNullArray {
    fn encode(&self) -> Vec<u8> {
        b"*-1\r\n".to_vec()
    }
}

impl RespDecode for RespNullArray {
    const PREFIX: &'static str = "*-1\r\n";
    const TYPE: &'static str = "RespNullArray";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        extract_fix(buf, "*-1\r\n", Self::TYPE)?;
        Ok(RespNullArray)
    }
    fn expect_length(_buf: &[u8]) -> Result<usize, RespError> {
        Ok(Self::PREFIX.len())
    }
}
#[cfg(test)]
mod tests {
    use super::{super::*, *};
    #[test]
    fn test_array_encode() {
        let array = RespArray::from_vec(vec![
            RespFrame::SimpleString(SimpleString::new("a".to_string())),
            RespFrame::SimpleString(SimpleString::new("b".to_string())),
        ]);
        let encoded = array.encode();
        assert_eq!(encoded, b"*2\r\n+a\r\n+b\r\n");
    }
    #[test]
    fn test_null_array_encode() {
        let array = RespArray::null();
        let encoded = array.encode();
        assert_eq!(encoded, b"*-1\r\n");
    }
    #[test]
    fn test_array_decode() {
        let mut buf = BytesMut::from("*2\r\n+a\r\n+b\r\n");
        let array = RespArray::decode(&mut buf).unwrap();
        assert_eq!(
            array,
            RespArray::from_vec(vec![
                RespFrame::SimpleString(SimpleString::new("a".to_string())),
                RespFrame::SimpleString(SimpleString::new("b".to_string())),
            ])
        );
    }
    #[test]
    fn test_null_array_decode() {
        let mut buf = BytesMut::from("*-1\r\n");
        let array = RespNullArray::decode(&mut buf).unwrap();
        assert_eq!(array, RespNullArray);
    }
}
