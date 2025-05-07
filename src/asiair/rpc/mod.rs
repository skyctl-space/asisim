use serde_json::{json, Value};
use std::sync::{Arc, Mutex};

pub mod protocol;
mod discover;

use super::ASIAirState;

pub fn handle_asiair_method(
    method: &str,
    params: &Option<Value>,
    state: Arc<Mutex<ASIAirState>>,
) -> (Value, u8) {
    match method {
        "scan_air" => discover::handle(params, state),
        _ => (json!({ "error": format!("Unknown method: {}", method) }), 1),
    }
}
