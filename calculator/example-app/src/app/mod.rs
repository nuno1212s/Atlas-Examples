pub mod messages;

use atlas_smr_application::app::{Application, Reply, Request};
use crate::state::CalculatorState;

pub struct App;

impl App {
    pub fn init() -> Self {
        Self {}
    }
}

impl Application<CalculatorState> for App {
    type AppData = messages::AppData;

    fn initial_state() -> atlas_common::error::Result<CalculatorState> {
        Ok(Default::default())
    }

    fn unordered_execution(&self, state: &CalculatorState, _: Request<Self, CalculatorState>) -> Reply<Self, CalculatorState> {
        messages::Reply::new(state.value())
    }
    fn update(&self, state: &mut CalculatorState, request: messages::Request) -> messages::Reply {
        let (op, value) = request.into();

        match op {
            messages::Operation::Add => {
                state.set_value(state.value() + value);
            },
            messages::Operation::Sub => {
                state.set_value(state.value() - value);
            },
            messages::Operation::Mult => {
                state.set_value(state.value() * value);
            },
            messages::Operation::Divide => {
                state.set_value(state.value() / value);
            },
            messages::Operation::Remainder => {
                state.set_value(state.value() % value);
            },
            messages::Operation::Exponent => {
                state.set_value(state.value().pow(value as u32));
            }
        }

        messages::Reply::new(state.value())
    }
}