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
    
    if let Some(value) = params {
        log::info!("pi_set_time: {:?}", value);

        let year = value[0]["year"].as_i64().unwrap_or(0);
        let month = value[0]["mon"].as_u64().unwrap_or(0);
        let day = value[0]["day"].as_u64().unwrap_or(0);
        let hour = value[0]["hour"].as_u64().unwrap_or(0);
        let minute = value[0]["min"].as_u64().unwrap_or(0);
        let second = value[0]["sec"].as_u64().unwrap_or(0);
        let time_zone = value[0]["time_zone"].as_str().unwrap_or("UTC");

        log::info!("Setting time: {}-{}-{} {}:{}:{}", year, month, day, hour, minute, second);
        log::info!("Setting timezone: {}", time_zone);

        state.rtc.set_time(
            year.try_into().unwrap(),
            month.try_into().unwrap(),
            day.try_into().unwrap(),
            hour.try_into().unwrap(),
            minute.try_into().unwrap(),
            second.try_into().unwrap(),
            time_zone.try_into().unwrap(),
        ).unwrap();


        (json!(0), 0)
    } else {
        (json!({ "error": "params is None" }), 1)
    }
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