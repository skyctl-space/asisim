use serde_json::{json, Value};
use std::sync::{Arc, Mutex};

mod app_handlers;
mod img_handlers;
mod misc_handlers;
pub mod protocol;
mod sample_raw;

use super::ASIAirState;
use crate::sim::BinaryResult;

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
) -> Result<(Value, u8), (String, u8)> {
    match method {
        "test_connection" => misc_handlers::test_connection(params, state),
        "pi_set_time" => misc_handlers::pi_set_time(params, state),
        "set_setting" => misc_handlers::set_setting(params, state),
        "get_setting" => misc_handlers::get_setting(params, state),
        "get_app_state" => app_handlers::get_app_state(params, state),
        "get_app_setting" => app_handlers::get_app_setting(params, state),
        "get_connected_cameras" => app_handlers::get_connected_cameras(params, state),
        "get_camera_state" => app_handlers::get_camera_state(params, state),
        "open_camera" => app_handlers::open_camera(params, state),
        "close_camera" => app_handlers::close_camera(params, state),
        "get_camera_info" => app_handlers::get_camera_info(params, state),
        "get_control_value" => app_handlers::get_control_value(params, state),
        _ => Err(("Unknown method".to_string(), 1)),
    }
}

pub fn asiair_tcp_4500_handler(
    method: &str,
    params: &Option<Value>, // Currently unused, consider removing if not needed
    state: Arc<Mutex<ASIAirState>>, // Currently unused, consider removing if not needed
) -> Result<(Value, u8), (String, u8)> {
    match method {
        "test_connection" => misc_handlers::test_connection(params, state),
        _ => Err(("Unknown method".to_string(), 1)),
    }
}

pub fn asiair_tcp_4800_handler(
    method: &str,
    params: &Option<Value>, // Currently unused, consider removing if not needed
    state: Arc<Mutex<ASIAirState>>, // Currently unused, consider removing if not needed
) -> Result<BinaryResult, Box<dyn std::error::Error + Send + Sync>> {
    println!("Asiair TCP 4800 handler called with method: {}", method);
    match method {
        "test_connection" => {
            let response = misc_handlers::test_connection(params, state);
            Ok(BinaryResult {
                data: serde_json::to_string(&response).unwrap().into_bytes(),
                width: 0,
                height: 0,
            })
        }
        "get_current_img" => Ok(img_handlers::get_current_img(params, state)),
        _ => {
            return Err(format!("Unknown method: {}", method).into());
        }
    }
}
