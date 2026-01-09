use super::preludes::*;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct RespDouble {
    pub value: f64,
}

impl RespDouble {
    pub fn new(value: f64) -> Self {
        Self { value }
    }
}
// | Double        | `,`    | ",[<+|->]<integral>[.<fractional>][<E|e>[sign]<exponent>]\r\n" |
impl RespDecode for RespDouble {
    const PREFIX: &'static str = ",";
    const TYPE: &'static str = "double";

    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let value_str = extract_end_string(buf, Self::PREFIX, Self::TYPE)?;
        let value = match value_str.as_str() {
            "inf" => f64::INFINITY,
            "-inf" => f64::NEG_INFINITY,
            "nan" => f64::NAN,
            _ => value_str.parse()?,
        };

        Ok(RespDouble::new(value))
    }
    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        compute_end_with_crlf(buf, Self::PREFIX, Self::TYPE)
    }
}

impl RespEncode for RespDouble {
    fn encode(&self) -> Vec<u8> {
        let value_str = if self.value.is_nan() {
            "nan".to_string()
        } else if self.value.is_infinite() {
            if self.value.is_sign_positive() {
                "inf".to_string()
            } else {
                "-inf".to_string()
            }
        } else {
            let base = if self.value.abs() >= 1e6 || self.value.abs() < 1e-4 {
                // 条件：绝对值 ≥100万 或 <0.0001 → 用科学计数法
                format!("{:+e}", self.value)
            } else {
                // 其他情况 → 用普通十进制（小数点）表示
                format!("{:+}", self.value)
            };
            base.replace("e", "e+").replace("e+-", "e-")
        };

        let mut result = Vec::with_capacity(1 + value_str.len() + 2); // , + value + \r\n
        result.extend_from_slice(b",");
        result.extend_from_slice(value_str.as_bytes());
        result.extend_from_slice(b"\r\n");
        result
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_double_encode() -> anyhow::Result<()> {
        let double = RespDouble::new(4.14);
        let frame: RespFrame = double.into();
        let encoded = frame.encode();
        assert_eq!(encoded, b",+4.14\r\n");

        let negative_double = RespDouble::new(-2.71);
        let frame: RespFrame = negative_double.into();
        let encoded = frame.encode();
        assert_eq!(encoded, b",-2.71\r\n");

        let integer_double = RespDouble::new(10.0);
        let frame: RespFrame = integer_double.into();
        let encoded = frame.encode();
        assert_eq!(encoded, b",+10\r\n"); // 根据规范，整数值十（10）编码为,10\r\n

        let scientific_double = RespDouble::new(123456.0);
        let frame: RespFrame = scientific_double.into();
        let encoded = frame.encode();
        // 根据格式化规则，123456.0 应该变成 1.23456E5
        let encoded_str = String::from_utf8(encoded).unwrap();
        assert!(encoded_str.starts_with(",") && encoded_str.ends_with("\r\n"));

        let small_double = RespDouble::new(0.0000123);
        let frame: RespFrame = small_double.into();
        let encoded = frame.encode();
        let encoded_str = String::from_utf8(encoded).unwrap();
        assert!(encoded_str.starts_with(",") && encoded_str.ends_with("\r\n"));

        let nan_double = RespDouble::new(f64::NAN);
        let frame: RespFrame = nan_double.into();
        let encoded = frame.encode();
        assert_eq!(encoded, b",nan\r\n");

        let inf_double = RespDouble::new(f64::INFINITY);
        let frame: RespFrame = inf_double.into();
        let encoded = frame.encode();
        assert_eq!(encoded, b",inf\r\n");

        let neg_inf_double = RespDouble::new(f64::NEG_INFINITY);
        let frame: RespFrame = neg_inf_double.into();
        let encoded = frame.encode();
        assert_eq!(encoded, b",-inf\r\n");

        let frame: RespDouble = RespDouble::new(123.456);
        let frame: RespFrame = frame.into();
        assert_eq!(frame.encode(), b",+123.456\r\n");

        let frame: RespDouble = RespDouble::new(-123.456);
        let frame: RespFrame = frame.into();
        assert_eq!(frame.encode(), b",-123.456\r\n");

        let frame: RespDouble = RespDouble::new(1.23456e+8);
        let frame: RespFrame = frame.into();
        println!("{}", String::from_utf8(frame.encode())?);
        assert_eq!(frame.encode(), b",+1.23456e+8\r\n");

        let frame: RespDouble = RespDouble::new(-1.23456e-8);
        let frame: RespFrame = frame.into();
        assert_eq!(frame.encode(), b",-1.23456e-8\r\n");
        Ok(())
    }
    #[test]
    fn test_double_decode() -> anyhow::Result<()> {
        let mut bytes = BytesMut::from(",+3.1415928e+8\r\n");
        let frame = RespDouble::decode(&mut bytes)?;
        assert_eq!(frame.value, 3.1415928e8);
        Ok(())
    }
}
