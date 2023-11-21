use std::io::{Read, Write};
use serde::{Deserialize, Serialize};
use atlas_smr_application::serialize::ApplicationData;

struct AppData;

impl ApplicationData for AppData {
    type Request = Request;
    type Reply = Reply;

    fn serialize_request<W>(w: W, request: &Self::Request) -> atlas_common::error::Result<()> where W: Write {
        todo!()
    }

    fn deserialize_request<R>(r: R) -> atlas_common::error::Result<Self::Request> where R: Read {
        todo!()
    }

    fn serialize_reply<W>(w: W, reply: &Self::Reply) -> atlas_common::error::Result<()> where W: Write {
        todo!()
    }

    fn deserialize_reply<R>(r: R) -> atlas_common::error::Result<Self::Reply> where R: Read {
        todo!()
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