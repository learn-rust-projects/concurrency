use super::{super::SimpleString, preludes::*};
pub const CRLF: &[u8] = b"\r\n";
pub const CRLF_LEN: usize = CRLF.len();
use bytes::Bytes;

pub fn extract_end_string(
    buf: &mut BytesMut,
    prefix: &str,
    type_name: &str,
) -> Result<String, RespError> {
    Ok(String::from_utf8(extract_end_vec(buf, prefix, type_name)?)?)
}
/// 提取字节数组，消耗buf中的数据，不包含前缀和CRLF
pub fn extract_end_vec(
    buf: &mut BytesMut,
    prefix: &str,
    type_name: &str,
) -> Result<Vec<u8>, RespError> {
    let s = consume_byte_mut(buf, prefix, type_name)?;
    Ok(str::from_utf8(s.as_ref())?.as_bytes().to_vec())
}
/// bulk_string: 提取字节数组，消耗buf中的数据，不包含前缀和CRLF
pub fn split_vec(buf: &mut BytesMut, len: usize) -> Result<Vec<u8>, RespError> {
    let data: BytesMut = buf.split_to(len + CRLF_LEN);
    Ok(data[..len].to_vec())
}
/// 提取length，不消耗buf中的数据
/// 返回
/// 结束位置\r'
/// 长度
pub fn extract_len(buf: &[u8], prefix: &str, type_name: &str) -> Result<(usize, usize), RespError> {
    let (end, s) = extract_byte_slice(buf, prefix, type_name)?;
    Ok((
        end,
        str::from_utf8(&s[prefix.len()..s.len() - CRLF_LEN])?.parse::<usize>()?,
    ))
}

/// 提取长度并判断是否满足期望大小
/// 返回
/// 结束位置\r'
/// 长度
pub fn check_len(buf: &mut BytesMut, prefix: &str, type_name: &str) -> Result<usize, RespError> {
    let (end, len) = extract_len(buf, prefix, type_name)?;
    let total_length = calc_total_length(buf, end, len, prefix)?;
    if buf.len() < total_length {
        return Err(RespError::NotComplete);
    }
    buf.advance(end + CRLF_LEN);
    Ok(len)
}

/// 提取固定长度的字节数组，消耗buf中的数据，不包含前缀和CRLF
pub fn extract_fix(buf: &mut BytesMut, prefix: &str, type_name: &str) -> Result<(), RespError> {
    starts_with(buf, prefix, type_name)?;
    buf.advance(prefix.len());
    Ok(())
}
fn starts_with(buf: &[u8], prefix: &str, type_name: &str) -> Result<(), RespError> {
    if !buf.starts_with(prefix.as_bytes()) {
        return Err(RespError::InvalidFrameType(format!(
            "{} expect: {} , but got {:?}",
            type_name, prefix, buf
        )));
    }
    Ok(())
}
/// 计算结束位置
pub fn compute_end(buf: &[u8], prefix: &str, type_name: &str) -> Result<usize, RespError> {
    starts_with(buf, prefix, type_name)?;
    let mut end = 0;
    for i in 0..buf.len() - 1 {
        if buf[i] == b'\r' && buf[i + 1] == b'\n' {
            end = i;
            break;
        }
    }
    if end == 0 {
        return Err(RespError::NotComplete);
    }
    Ok(end)
}

/// 计算结束位置
pub fn compute_end_with_crlf(
    buf: &[u8],
    prefix: &str,
    type_name: &str,
) -> Result<usize, RespError> {
    let end = compute_end(buf, prefix, type_name)?;
    Ok(end + CRLF_LEN)
}
/// 提取字节数组，消耗buf中的数据，不包含前缀和CRLF
pub fn consume_byte_mut(
    buf: &mut BytesMut,
    prefix: &str,
    type_name: &str,
) -> Result<Bytes, RespError> {
    let end = compute_end(buf, prefix, type_name)?;
    let byte = buf.split_to(end + CRLF_LEN);
    Ok(byte.freeze().slice(prefix.len()..end))
}
/// 提取字节数组，不消耗buf中的数据
pub fn extract_byte_slice<'a>(
    buf: &'a [u8],
    prefix: &str,
    type_name: &str,
) -> Result<(usize, &'a [u8]), RespError> {
    let end = compute_end(buf, prefix, type_name)?;
    Ok((end, &buf[0..end + CRLF_LEN]))
}

pub fn calc_total_length(
    buf: &[u8],
    end: usize,
    len: usize,
    prefix: &str,
) -> Result<usize, RespError> {
    let mut total = end + CRLF_LEN;
    let mut data = &buf[total..];
    match prefix {
        "*" | "~" => {
            for _ in 0..len {
                let len = RespFrame::expect_length(data)?;
                data = &data[len..];
                total += len;
            }
            Ok(total)
        }
        "%" => {
            for _ in 0..len {
                let len = SimpleString::expect_length(data)?;

                data = &data[len..];
                total += len;

                let len = RespFrame::expect_length(data)?;
                data = &data[len..];
                total += len;
            }
            Ok(total)
        }
        _ => Ok(len + CRLF_LEN),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_extract_end_string() {
        let mut buf = BytesMut::from("+OK\r\n");
        let end = extract_end_string(&mut buf, "+", "SimpleString");
        assert_eq!(end, Ok("OK".to_string()));
        let mut buf = BytesMut::from("+334\r\n");
        let end = extract_end_string(&mut buf, "-", "RespInteger");
        assert_eq!(
            end,
            Err(RespError::InvalidFrameType(format!(
                "RespInteger expect: - , but got {:?}",
                buf.as_ref()
            )))
        );
        // 测试notComplete
        let mut buf = BytesMut::from("+OK\r");
        let end = extract_end_string(&mut buf, "+", "SimpleString");
        assert_eq!(end, Err(RespError::NotComplete));
    }
}
