use crate::{
    backend::Backend,
    cmd::{CommandError, CommandExecutor, CommandGet, CommandSet, RESP_OK, valid_command},
    resp::{BulkString, RespArray, RespFrame, RespNull},
};

// Redis命令与RESP协议格式对应表
// | 命令    | 参数         | 对应格式                                                                 |
// |---------|--------------|--------------------------------------------------------------------------|
// | SET     | key val      | "*3\r\n$3\r\nset\r\n$5\r\nhello\r\n$5\r\nworld\r\n"                      |
// | GET     | key          | "*2\r\n$3\r\nget\r\n$5\r\nhello\r\n"                                     |
// | HSET    | key field val| "*4\r\n$4\r\nhset\r\n$3\r\nmap\r\n$5\r\nhello\r\n$5\r\nworld\r\n"         |
// | HGET    | key field    | "*3\r\n$4\r\nhget\r\n$3\r\nmap\r\n$5\r\nhello\r\n"                       |
// | HGETALL | key          | "*2\r\n$7\r\nhgetall\r\n$3\r\nmap\r\n"                                   |

impl CommandExecutor for CommandGet {
    fn execute(self, backend: &Backend) -> RespFrame {
        backend
            .get(&self.key)
            .unwrap_or(RespFrame::RespNull(RespNull))
    }
}

impl CommandExecutor for CommandSet {
    fn execute(self, backend: &Backend) -> RespFrame {
        backend.set(self.key, self.value);
        RESP_OK.clone()
    }
}

impl TryFrom<RespArray> for CommandGet {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        let args = valid_command(&value, &["GET"], 1)?;
        match args[0] {
            RespFrame::BulkString(BulkString { content: Some(key) }) => Ok(CommandGet {
                key: String::from_utf8(key.to_vec())?,
            }),
            _ => Err(CommandError::InvalidArguments(
                "GET command expect bulk string".to_string(),
            )),
        }
    }
}
impl TryFrom<RespArray> for CommandSet {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        let args = valid_command(&value, &["SET"], 2)?;
        match (args[0], args[1]) {
            (
                RespFrame::BulkString(BulkString { content: Some(key) }),
                RespFrame::BulkString(BulkString {
                    content: Some(value),
                }),
            ) => Ok(CommandSet {
                key: String::from_utf8(key.to_vec())?,
                value: RespFrame::BulkString(BulkString {
                    content: Some(value.to_vec()),
                }),
            }),
            _ => Err(CommandError::InvalidArguments(
                "SET command expect bulk string".to_string(),
            )),
        }
    }
}
#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use super::*;
    use crate::resp::RespDecode;
    #[test]
    fn test_command_get_try_from() -> Result<(), CommandError> {
        let mut byte_get = BytesMut::from("*2\r\n$3\r\nget\r\n$5\r\nhello\r\n");
        let array = RespArray::decode(&mut byte_get)?;
        let cmd = CommandGet::try_from(array)?;
        assert_eq!(cmd.key, "hello".to_string());
        Ok(())
    }
    #[test]
    fn test_command_set_try_from() -> Result<(), CommandError> {
        let mut byte_set = BytesMut::from("*3\r\n$3\r\nset\r\n$5\r\nhello\r\n$5\r\nworld\r\n");
        let array = RespArray::decode(&mut byte_set)?;
        let cmd = CommandSet::try_from(array)?;
        assert_eq!(cmd.key, "hello".to_string());
        assert_eq!(cmd.value, "world".into());
        Ok(())
    }
    #[test]
    fn test_set_get_command() -> Result<(), CommandError> {
        let backend = Backend::new();
        let cmd_set = CommandSet::try_from(RespArray::decode(&mut BytesMut::from(
            "*3\r\n$3\r\nset\r\n$5\r\nhello\r\n$5\r\nworld\r\n",
        ))?)?;
        let resp_set = cmd_set.execute(&backend);
        assert_eq!(resp_set, RESP_OK.clone());
        let cmd_get = CommandGet::try_from(RespArray::decode(&mut BytesMut::from(
            "*2\r\n$3\r\nget\r\n$5\r\nhello\r\n",
        ))?)?;
        let resp_get = cmd_get.execute(&backend);
        assert_eq!(resp_get, "world".into());
        Ok(())
    }
}
