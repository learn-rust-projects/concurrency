mod hmap;
mod map;
use std::{convert::TryFrom, sync::LazyLock};

use thiserror::Error;

use crate::{
    backend::Backend,
    resp::{BulkString, RespArray, RespError, RespFrame, SimpleError, SimpleString},
};

pub static RESP_OK: LazyLock<RespFrame> =
    LazyLock::new(|| RespFrame::SimpleString(SimpleString::from("OK")));

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("invalid command: {0}")]
    InvalidCommand(String),
    #[error("invalid arguments: {0}")]
    InvalidArguments(String),
    #[error("resp error: {0}")]
    RespErr(#[from] RespError),
    #[error("invalid number of arguments: {0}")]
    InvalidNumberOfArguments(String),
    #[error("invalid utf8: {0}")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),
}

pub trait CommandExecutor {
    fn execute(self, backend: &Backend) -> RespFrame;
}
#[derive(Debug)]
pub enum Command {
    Ping,
    Set(CommandSet),
    Get(CommandGet),
    HGet(CommandHGet),
    HSet(CommandHSet),
    HGetAll(CommandHGetAll),
    Unrecognized(Unrecognized),
}
#[derive(Debug)]
pub struct Unrecognized {
    command: String,
}
impl CommandExecutor for Unrecognized {
    fn execute(self, _backend: &Backend) -> RespFrame {
        SimpleError::new(format!("unrecognized command: {}", self.command)).into()
    }
}
impl CommandExecutor for Command {
    fn execute(self, backend: &Backend) -> RespFrame {
        match self {
            Command::Ping => RespFrame::SimpleString(SimpleString::from("PONG")),
            Command::Set(cmd) => cmd.execute(backend),
            Command::Get(cmd) => cmd.execute(backend),
            Command::HGet(cmd) => cmd.execute(backend),
            Command::HSet(cmd) => cmd.execute(backend),
            Command::HGetAll(cmd) => cmd.execute(backend),
            Command::Unrecognized(unrecognized) => unrecognized.execute(backend),
        }
    }
}
#[derive(Debug)]
#[allow(dead_code)]
pub struct CommandGet {
    key: String,
}
#[derive(Debug)]
#[allow(dead_code)]
pub struct CommandSet {
    key: String,
    value: RespFrame,
}
#[allow(dead_code)]
#[derive(Debug)]
pub struct CommandHGet {
    key: String,
    field: String,
}
#[derive(Debug)]
pub struct CommandHGetAll {
    #[allow(dead_code)]
    key: String,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct CommandHSet {
    key: String,
    field: String,
    value: RespFrame,
}

pub fn valid_command<'a>(
    value: &'a RespArray,
    name_slice: &[&'static str],
    n_args: usize,
) -> Result<Vec<&'a RespFrame>, CommandError> {
    if let Some(ref elements) = value.elements {
        if elements.len() != n_args + 1 {
            return Err(CommandError::InvalidNumberOfArguments(format!(
                "{} command expects {} arguments, got {}",
                name_slice.join(" "),
                n_args,
                elements.len()
            )));
        }
        for (i, name) in name_slice.iter().enumerate() {
            match elements[i] {
                RespFrame::BulkString(BulkString {
                    content: Some(ref bytes),
                }) => {
                    if bytes.to_ascii_lowercase() != name.to_ascii_lowercase().as_bytes() {
                        return Err(CommandError::InvalidArguments(format!(
                            "{} command must have the {} argument: expect {:?}, but got {:?}",
                            name_slice.join(" "),
                            i,
                            name,
                            bytes
                        )));
                    }
                }
                _ => {
                    return Err(CommandError::InvalidArguments(format!(
                        "{} command must have a bulk string as the {} argument: expect {:?} find, but got {:?}",
                        name_slice.join(" "),
                        i,
                        name,
                        elements[i]
                    )));
                }
            }
        }
        Ok(elements.iter().skip(name_slice.len()).collect())
    } else {
        Err(CommandError::InvalidArguments(format!(
            "{} command expects {} arguments, got 0",
            name_slice.join(" "),
            n_args
        )))
    }
}

impl TryFrom<RespFrame> for Command {
    type Error = CommandError;
    fn try_from(v: RespFrame) -> Result<Self, Self::Error> {
        match v {
            RespFrame::Array(resp_array) => resp_array.try_into(),
            _ => Err(CommandError::InvalidCommand(
                "invalid command array".to_string(),
            )),
        }
    }
}

impl TryFrom<RespArray> for Command {
    type Error = CommandError;
    fn try_from(v: RespArray) -> Result<Self, Self::Error> {
        match v {
            RespArray {
                elements: Some(ref elements),
            } => match elements.first() {
                Some(RespFrame::BulkString(BulkString {
                    content: Some(bytes),
                })) => match bytes.as_slice() {
                    b"ping" => Ok(Command::Ping),
                    b"set" => CommandSet::try_from(v).map(Command::Set),
                    b"get" => CommandGet::try_from(v).map(Command::Get),
                    b"hget" => CommandHGet::try_from(v).map(Command::HGet),
                    b"hset" => CommandHSet::try_from(v).map(Command::HSet),
                    b"hgetall" => CommandHGetAll::try_from(v).map(Command::HGetAll),
                    _ => Ok(Command::Unrecognized(Unrecognized {
                        command: String::from_utf8_lossy(bytes).to_string(),
                    })),
                },
                _ => Err(CommandError::InvalidCommand(format!(
                    "unknown command: {:?}",
                    elements.first()
                ))),
            },
            _ => Err(CommandError::InvalidCommand(
                "empty command array".to_string(),
            )),
        }
    }
}
