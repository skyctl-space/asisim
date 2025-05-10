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
pub fn test_connection(_params: &Option<Value>, _state: Arc<Mutex<ASIAirState>>) -> (Value, u8) {
    (json!("server connected!"), 0)
}

pub fn pi_set_time(params: &Option<Value>, state: Arc<Mutex<ASIAirState>>) -> (Value, u8) {
    let mut state = state.lock().unwrap();

    // Check if params is None or not
    if params.is_none() {
        return (json!({ "error": "params is None" }), 1);
    }

    let params = params.as_ref().unwrap();
    let params = params.as_str().unwrap_or("[]");
    let params = serde_json::from_str::<Vec<Value>>(params).unwrap_or_else(|_| vec![]);

    let year = params[0]["year"].as_i64().unwrap_or(0) as i32;
    let month = params[0]["mon"].as_i64().unwrap_or(0) as u32;
    let day = params[0]["day"].as_i64().unwrap_or(0) as u32;
    let hour = params[0]["hour"].as_i64().unwrap_or(0) as u32;
    let minute = params[0]["min"].as_i64().unwrap_or(0) as u32;
    let second = params[0]["sec"].as_i64().unwrap_or(0) as u32;
    let time_zone = params[0]["time_zone"].as_str().unwrap_or("UTC");

    state.rtc.set_time(
        year,
        month,
        day,
        hour,
        minute,
        second,
        time_zone,
    ).unwrap();


    (json!(0), 0)
}

pub fn set_setting(params: &Option<Value>, state: Arc<Mutex<ASIAirState>>) -> (Value, u8) {
    let mut state = state.lock().unwrap();

    // Check if params is None or not
    if params.is_none() {
        return (json!({ "error": "params is None" }), 1);
    }

    let params = serde_json::from_str::<Value>(params.as_ref().unwrap().as_str().unwrap_or("{}")).unwrap_or(json!({}));
    let lang = params["lang"].as_str().unwrap_or("en");

    state.language = lang.to_string();

    (json!(0), 0)
}