use super::ASIAirState;
use serde_json::Value;
use std::sync::{Arc, Mutex};

pub fn get_app_state(_: &Option<Value>, state: Arc<Mutex<ASIAirState>>) -> Result<(Value, u8), (String, u8)> {
    let state = state.lock().unwrap();

    Ok((serde_json::to_value(&state.app_state).unwrap(), 0))
}

pub fn get_app_setting(_: &Option<Value>, state: Arc<Mutex<ASIAirState>>) -> Result<(Value, u8), (String, u8)> {
    let state = state.lock().unwrap();

    Ok((serde_json::to_value(&state.app_setting).unwrap(), 0))
}

