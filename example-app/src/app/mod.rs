mod messages;

use atlas_smr_application::app::{Application, Reply, Request};
use crate::state::CalculatorState;

pub struct App;

impl Application<CalculatorState> for App {
    type AppData = ();

    fn initial_state() -> atlas_common::error::Result<CalculatorState> {
        todo!()
    }

    fn unordered_execution(&self, state: &CalculatorState, request: Request<Self, CalculatorState>) -> Reply<Self, CalculatorState> {
        todo!()
    }

    fn update(&self, state: &mut CalculatorState, request: Request<Self, CalculatorState>) -> Reply<Self, CalculatorState> {
        todo!()
    }
}