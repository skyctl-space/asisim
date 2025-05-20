use super::ASIAirState;
use crate::sim::CameraState;
use crate::sim::CAMERAS_INFO;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};

pub fn get_app_state(_: &Option<Value>, state: Arc<Mutex<ASIAirState>>) -> (Value, u8) {
    let state = state.lock().unwrap();

    (serde_json::to_value(&state.app_state).unwrap(), 0)
}

pub fn get_app_setting(_: &Option<Value>, state: Arc<Mutex<ASIAirState>>) -> (Value, u8) {
    let state = state.lock().unwrap();

    (serde_json::to_value(&state.app_setting).unwrap(), 0)
}

pub fn get_connected_cameras(_: &Option<Value>, state: Arc<Mutex<ASIAirState>>) -> (Value, u8) {
    let state = state.lock().unwrap();

    (serde_json::to_value(&state.connected_cameras).unwrap(), 0)
}

pub fn get_camera_state(_: &Option<Value>, state: Arc<Mutex<ASIAirState>>) -> (Value, u8) {
    let state = state.lock().unwrap();

    (serde_json::to_value(&state.camera_state).unwrap(), 0)
}

pub fn open_camera(_: &Option<Value>, state: Arc<Mutex<ASIAirState>>) -> (Value, u8) {
    let mut state = state.lock().unwrap();

    for camera in state.connected_cameras.iter() {
        if camera.name == state.app_setting.main_camera_name {
            state.camera_state = CameraState::Idle{
                name: state.app_setting.main_camera_name.clone(),
                path: camera.path.clone(),
            };

            return (json!(0), 0);
        }
    }

    (json!({ "error": "Camera not found" }), 1)
}

pub fn close_camera(_: &Option<Value>, state: Arc<Mutex<ASIAirState>>) -> (Value, u8) {
    let mut state = state.lock().unwrap();

    state.camera_state = CameraState::Close;

    return (json!(0), 0);
}

pub fn get_camera_info(_: &Option<Value>, state: Arc<Mutex<ASIAirState>>) -> (Value, u8) {
    let state = state.lock().unwrap();

    if let Some(camera_info) = CAMERAS_INFO.get(state.app_setting.main_camera_name.as_str()) {
        return (serde_json::to_value(camera_info).unwrap(), 0);
    }

    return (json!({ "error": "Unknown Camera" }), 1);
}