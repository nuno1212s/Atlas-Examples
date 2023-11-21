use std::io::{Read, Write};
use serde::{Deserialize, Serialize};
use atlas_smr_application::state::monolithic_state::MonolithicState;

#[derive(Serialize, Deserialize, Clone)]
pub struct CalculatorState {
    value: i32
}

impl MonolithicState for CalculatorState {
    fn serialize_state<W>(w: W, request: &Self) -> atlas_common::error::Result<()> where W: Write {
        todo!()
    }

    fn deserialize_state<R>(r: R) -> atlas_common::error::Result<Self> where R: Read, Self: Sized {
        todo!()
    }
}
