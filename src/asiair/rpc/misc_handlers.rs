use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use super::ASIAirState;

// This method is called over UDP by the ASIAIR to discover ASIAIR on the network
pub fn scan_air(_params: &Option<Value>, state: Arc<Mutex<ASIAirState>>) -> (Value, u8) {
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

// This method is called over TCP by the ASIAIR to test the connection
pub fn test_connection(_params: &Option<Value>, state: Arc<Mutex<ASIAirState>>) -> (Value, u8) {
    let state = state.lock().unwrap();
    (json!("server connected!"), 0)
}