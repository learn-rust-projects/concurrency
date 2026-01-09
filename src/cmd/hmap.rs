use crate::{
    backend::Backend,
    cmd::{
        CommandError, CommandExecutor, CommandHGet, CommandHGetAll, CommandHSet, RESP_OK,
        valid_command,
    },
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
// | HSET    | key field val| "*4\r\n$4\r\nhset\r\n$3\r\nmap\r\n$5\r\nhello\r\n$5\r\nworld\r\n"         |
impl CommandExecutor for CommandHGet {
    fn execute(self, backend: &Backend) -> RespFrame {
        backend
            .hget(&self.key, &self.field)
            .unwrap_or(RespFrame::RespNull(RespNull))
    }
}
impl CommandExecutor for CommandHGetAll {
    fn execute(self, backend: &Backend) -> RespFrame {
        let map = backend.hgetall(&self.key);
        match map {
            Some(map) => {
                let mut data = map
                    .iter()
                    .map(|v| (v.key().clone(), v.value().clone()))
                    .collect::<Vec<_>>();
                data.sort_unstable_by(|a, b| a.0.cmp(&b.0));
                let mut ret = Vec::with_capacity(data.len() * 2);

                for (k, v) in data {
                    ret.push(BulkString::from_slice(k).into());
                    ret.push(v);
                }
                RespArray::new(Some(ret)).into()
            }
            None => RespFrame::RespNull(RespNull),
        }
    }
}

impl CommandExecutor for CommandHSet {
    fn execute(self, backend: &Backend) -> RespFrame {
        backend.hset(self.key, self.field, self.value.clone());
        RESP_OK.clone()
    }
}
impl TryFrom<RespArray> for CommandHGet {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        let args = valid_command(&value, &["HGET"], 2)?;
        match (args[0], args[1]) {
            (
                RespFrame::BulkString(BulkString { content: Some(key) }),
                RespFrame::BulkString(BulkString {
                    content: Some(field),
                }),
            ) => Ok(CommandHGet {
                key: String::from_utf8(key.to_vec())?,
                field: String::from_utf8(field.to_vec())?,
            }),
            _ => Err(CommandError::InvalidArguments(
                "HGET command expect bulk string".to_string(),
            )),
        }
    }
}

impl TryFrom<RespArray> for CommandHGetAll {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        let args = valid_command(&value, &["HGETALL"], 1)?;
        match args[0] {
            RespFrame::BulkString(BulkString { content: Some(key) }) => Ok(CommandHGetAll {
                key: String::from_utf8(key.to_vec())?,
            }),
            _ => Err(CommandError::InvalidArguments(
                "HGETALL command expect bulk string".to_string(),
            )),
        }
    }
}

impl TryFrom<RespArray> for CommandHSet {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        let args = valid_command(&value, &["HSET"], 3)?;
        match (args[0], args[1], args[2]) {
            (
                RespFrame::BulkString(BulkString { content: Some(key) }),
                RespFrame::BulkString(BulkString {
                    content: Some(field),
                }),
                resp_frame,
            ) => Ok(CommandHSet {
                key: String::from_utf8(key.to_vec())?,
                field: String::from_utf8(field.to_vec())?,
                value: resp_frame.clone(),
            }),
            _ => Err(CommandError::InvalidArguments(
                "HSET command expect bulk string".to_string(),
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
    fn test_hget_command() {
        let cmd: RespArray = vec!["HGET".into(), "key".into(), "field".into()].into();
        let cmd = CommandHGet::try_from(cmd).unwrap();
        assert_eq!(cmd.key, "key");
        assert_eq!(cmd.field, "field");
    }
    #[test]
    fn test_hgetall_command() {
        let cmd: RespArray = vec!["HGETALL".into(), "key".into()].into();
        let cmd = CommandHGetAll::try_from(cmd).unwrap();
        assert_eq!(cmd.key, "key".to_string());
    }
    #[test]
    fn test_command_hget_try_from() -> Result<(), CommandError> {
        let mut byte_get = BytesMut::from("*3\r\n$4\r\nhget\r\n$3\r\nmap\r\n$5\r\nhello\r\n");
        let array = RespArray::decode(&mut byte_get)?;
        let cmd = CommandHGet::try_from(array)?;
        assert_eq!(cmd.key, "map".to_string());
        assert_eq!(cmd.field, "hello".to_string());
        Ok(())
    }
    #[test]
    fn test_command_hgetall_try_from() -> Result<(), CommandError> {
        let mut byte_get = BytesMut::from("*2\r\n$7\r\nhgetall\r\n$3\r\nmap\r\n");
        let array = RespArray::decode(&mut byte_get)?;
        let cmd = CommandHGetAll::try_from(array)?;
        assert_eq!(cmd.key, "map".to_string());
        Ok(())
    }
    #[test]
    fn test_command_hset_try_from() -> Result<(), CommandError> {
        let mut byte_get =
            BytesMut::from("*4\r\n$4\r\nhset\r\n$3\r\nmap\r\n$5\r\nhello\r\n$5\r\nworld\r\n");
        let array = RespArray::decode(&mut byte_get)?;
        let cmd = CommandHSet::try_from(array)?;
        assert_eq!(cmd.key, "map".to_string());
        assert_eq!(cmd.field, "hello".to_string());
        assert_eq!(
            cmd.value,
            RespFrame::BulkString(BulkString {
                content: Some("world".into())
            })
        );
        Ok(())
    }
    #[test]
    /// hset value是array的
    fn test_command_hset_try_from_array() -> Result<(), CommandError> {
        let mut byte_get =
            BytesMut::from("*4\r\n$4\r\nhset\r\n$3\r\nmap\r\n$5\r\nhello\r\n*1\r\n$5\r\nworld\r\n");
        let array = RespArray::decode(&mut byte_get)?;
        let cmd = CommandHSet::try_from(array)?;
        assert_eq!(cmd.key, "map".to_string());
        assert_eq!(cmd.field, "hello".to_string());
        assert_eq!(
            cmd.value,
            RespFrame::Array(RespArray::from(vec!["world".into()]))
        );
        Ok(())
    }
    #[test]
    fn test_hset_hget_hgetall_commands() -> Result<(), CommandError> {
        let backend = Backend::new();
        let cmd = CommandHSet {
            key: "map".to_string(),
            field: "hello".to_string(),
            value: "world".into(),
        };
        let result = cmd.execute(&backend);
        assert_eq!(result, RESP_OK.clone());

        let cmd = CommandHSet {
            key: "map".to_string(),
            field: "hello1".to_string(),
            value: "world1".into(),
        };
        cmd.execute(&backend);

        let cmd = CommandHGet {
            key: "map".to_string(),
            field: "hello".to_string(),
        };
        let result = cmd.execute(&backend);
        assert_eq!(result, "world".into());

        let cmd = CommandHGetAll {
            key: "map".to_string(),
        };
        let result: RespFrame = cmd.execute(&backend);

        let expected: RespArray = vec![
            "hello".into(),
            "world".into(),
            "hello1".into(),
            "world1".into(),
        ]
        .into();
        assert_eq!(result, expected.into());
        Ok(())
    }
}
