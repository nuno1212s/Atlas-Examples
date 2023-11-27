use std::io::{Read, Write};
use anyhow::Context;
use serde::{Deserialize, Serialize};
use atlas_smr_application::state::monolithic_state::MonolithicState;

#[derive(Serialize, Deserialize, Clone)]
pub struct CalculatorState {
    value: i32
}

impl Default for CalculatorState {
    fn default() -> Self {
        CalculatorState {
            value: 0,
        }
    }
}

impl CalculatorState {
    pub fn value(&self) -> i32 {
        self.value
    }

    pub fn set_value(&mut self, value: i32) {
        self.value = value;
    }
}

impl MonolithicState for CalculatorState {
    fn serialize_state<W>(mut w: W, request: &Self) -> atlas_common::error::Result<()> where W: Write {
        bincode::serde::encode_into_std_write(request, &mut w, bincode::config::standard()).context("Failed to serialize state")?;
        
        Ok(())
    }

    fn deserialize_state<R>(mut r: R) -> atlas_common::error::Result<Self> where R: Read, Self: Sized {
        bincode::serde::decode_from_std_read(&mut r, bincode::config::standard()).context("Failed to deserialize state")
    }
}
