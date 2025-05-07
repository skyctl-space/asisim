use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use super::ASIAirState;

pub fn handle(_params: &Option<Value>, state: Arc<Mutex<ASIAirState>>) -> (Value, u8) {
    let state = state.lock().unwrap();
    (json!({
        "name": state.name,
        "ip": state.ip,
        "ssid": state.ssid,
        "guid": state.guid,
        "is_pi4": state.is_pi4,
        "model": state.model,
        "connect_lock": state.connect_lock,
    }), 0)
}
