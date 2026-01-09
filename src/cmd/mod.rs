mod hmap;
mod map;
use std::sync::LazyLock;

use thiserror::Error;

use crate::{
    backend::Backend,
    resp::{BulkString, RespArray, RespError, RespFrame, SimpleString},
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
pub enum Command {
    Ping,
    Set(CommandSet),
    Get(CommandGet),
    HGet(CommandHGet),
    HGetAll,
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
