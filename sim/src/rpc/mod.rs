use serde_json::{json, Value};
use std::sync::{Arc, Mutex};

mod app_handlers;
mod misc_handlers;
pub mod protocol;

use super::ASIAirState;

pub fn asiair_udp_handler(
    method: &str,
    params: &Option<Value>, // Currently unused, consider removing if not needed
    state: Arc<Mutex<ASIAirState>>, // Currently unused, consider removing if not needed
) -> (Value, u8) {
    match method {
        "scan_air" => misc_handlers::scan_air(params, state),
        _ => (json!({ "error": format!("Unknown method: {}", method) }), 1),
    }
}

pub fn asiair_tcp_handler(
    method: &str,
    params: &Option<Value>, // Currently unused, consider removing if not needed
    state: Arc<Mutex<ASIAirState>>, // Currently unused, consider removing if not needed
) -> (Value, u8) {
    match method {
        "test_connection" => misc_handlers::test_connection(params, state),
        "pi_set_time" => misc_handlers::pi_set_time(params, state),
        "set_setting" => misc_handlers::set_setting(params, state),
        "get_setting" => misc_handlers::get_setting(params, state),
        "get_app_state" => app_handlers::get_app_state(params, state),
        _ => (json!({ "error": format!("Unknown method: {}", method) }), 1),
    }
}

pub fn asiair_tcp_4500_handler(
    method: &str,
    params: &Option<Value>, // Currently unused, consider removing if not needed
    state: Arc<Mutex<ASIAirState>>, // Currently unused, consider removing if not needed
) -> (Value, u8) {
    match method {
        "test_connection" => misc_handlers::test_connection(params, state),
        _ => (json!({ "error": format!("Unknown method: {}", method) }), 1),
    }
}

pub fn asiair_tcp_4800_handler(
    method: &str,
    params: &Option<Value>, // Currently unused, consider removing if not needed
    state: Arc<Mutex<ASIAirState>>, // Currently unused, consider removing if not needed
) -> (Vec<u8>, u8) {
    match method {
        "test_connection" => {
            let response = misc_handlers::test_connection(params, state);
            (serde_json::to_string(&response).unwrap().into_bytes(), 0)
        }, 
        // "get_current_img" => misc_handlers::get_setting(params, state),
        _ => (vec![], 1),
    }
}