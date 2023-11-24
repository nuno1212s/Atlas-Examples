use std::io::{Read, Write};
use serde::{Deserialize, Serialize};
use atlas_smr_application::serialize::ApplicationData;
use anyhow::Context;

pub struct AppData;

impl ApplicationData for AppData {
    type Request = Request;
    type Reply = Reply;

    fn serialize_request<W>(mut w: W, request: &Self::Request) -> atlas_common::error::Result<()> where W: Write {
        bincode::serde::encode_into_std_write(request, &mut w, bincode::config::standard())
            .context("Failed to serialize request")?;

        Ok(())
    }

    fn deserialize_request<R>(mut r: R) -> atlas_common::error::Result<Self::Request> where R: Read {
        bincode::serde::decode_from_std_read(&mut r, bincode::config::standard())
            .context("Failed to deserialize request")
    }

    fn serialize_reply<W>(mut w: W, reply: &Self::Reply) -> atlas_common::error::Result<()> where W: Write {
        bincode::serde::encode_into_std_write(reply, &mut w, bincode::config::standard())
            .context("Failed to serialize request")?;

        Ok(())
    }

    fn deserialize_reply<R>(mut r: R) -> atlas_common::error::Result<Self::Reply> where R: Read {
        bincode::serde::decode_from_std_read(&mut r, bincode::config::standard())
            .context("Failed to deserialize request")
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Operation {
    Add,
    Sub,
    Mult,
    Divide,
    Remainder,
    Exponent
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Request {
    operation: Operation,
    value: i32
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Reply {
    value: i32
}

impl Request {
    pub fn new(operation: Operation, value: i32) -> Self {
        Request {
            operation,
            value
        }
    }

    pub fn into(self) -> (Operation, i32) {
        (self.operation, self.value)
    }

}

impl Reply {
    pub fn new(value: i32) -> Self {
        Reply {
            value
        }
    }
}