use super::ASIAirState;
use crate::sim::CameraState;
use crate::sim::CAMERAS_INFO;
use crate::sim::CAMERA_CONTROL_TYPES;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};

pub fn get_app_state(_: &Option<Value>, state: Arc<Mutex<ASIAirState>>) -> Result<(Value, u8), (String, u8)> {
    let state = state.lock().unwrap();

    Ok((serde_json::to_value(&state.app_state).unwrap(), 0))
}

pub fn get_app_setting(_: &Option<Value>, state: Arc<Mutex<ASIAirState>>) -> Result<(Value, u8), (String, u8)> {
    let state = state.lock().unwrap();

    Ok((serde_json::to_value(&state.app_setting).unwrap(), 0))
}

pub fn get_connected_cameras(_: &Option<Value>, state: Arc<Mutex<ASIAirState>>) -> Result<(Value, u8), (String, u8)> {
    let state = state.lock().unwrap();

    Ok((serde_json::to_value(&state.connected_cameras).unwrap(), 0))
}

pub fn get_camera_state(_: &Option<Value>, state: Arc<Mutex<ASIAirState>>) -> Result<(Value, u8), (String, u8)> {
    let state = state.lock().unwrap();

    Ok((serde_json::to_value(&state.camera_state).unwrap(), 0))
}

pub fn open_camera(_: &Option<Value>, state: Arc<Mutex<ASIAirState>>) -> Result<(Value, u8), (String, u8)> {
    let mut state = state.lock().unwrap();

    for camera in state.connected_cameras.iter() {
        if camera.name == state.app_setting.main_camera_name {
            state.camera_state = CameraState::Idle {
                name: state.app_setting.main_camera_name.clone(),
                path: camera.path.clone(),
            };

            return Ok((json!(0), 0));
        }
    }

    Err(("Camera not found".to_string(), 1))
}

pub fn close_camera(_: &Option<Value>, state: Arc<Mutex<ASIAirState>>) -> Result<(Value, u8), (String, u8)> {
    let mut state = state.lock().unwrap();

    state.camera_state = CameraState::Close;

    Ok((json!(0), 0))
}

pub fn get_camera_info(_: &Option<Value>, state: Arc<Mutex<ASIAirState>>) -> Result<(Value, u8), (String, u8)> {
    let state = state.lock().unwrap();

    if let Some(camera_info) = CAMERAS_INFO.get(state.app_setting.main_camera_name.as_str()) {
        return Ok((serde_json::to_value(camera_info).unwrap(), 0));
    }

    Err(("Unknown Camera".to_string(), 1))
}

pub fn get_control_value(
    params: &Option<Value>,
    state: Arc<Mutex<ASIAirState>>,
) -> Result<(Value, u8), (String, u8)> {
    let state = state.lock().unwrap();

    match params {
        Some(value) => {
            if !value.is_array() {
                return Err(("params is not an array".to_string(), 1));
            }
            if let Some(control_name) = value[0].as_str() {
                if !CAMERA_CONTROL_TYPES.contains_key(control_name) {
                    return Err(("control name is not valid".to_string(), 1));
                }
                let control_type = CAMERA_CONTROL_TYPES.get(control_name).unwrap();

                let value: f64 = match control_name {
                    "Exposure" => state.camera_controls.exposure as f64,
                    "Gain" => state.camera_controls.gain.into(),
                    "CoolerOn" => state.camera_controls.cooler_on.into(),
                    "Temperature" => state.camera_controls.temperature.into(),
                    "CoolerPowerPercent" => state.camera_controls.cooler_power_perc.into(),
                    "TargetTemp" => state.camera_controls.target_temp.into(),
                    "AntiDewHeater" => state.camera_controls.anti_dew_heater.into(),
                    "Red" => state.camera_controls.red.into(),
                    "Blue" => state.camera_controls.blue.into(),
                    _ => {
                        return Err(("control name is not valid".to_string(), 1));
                    }
                };

                return Ok((json!({
                    "name": control_name,
                    "type": control_type,
                    "value": value,
                }), 0));
            }

            return Err(("unknown control name".to_string(), 1));
        }
        None => return Err(("params is not provided".to_string(), 1)),
    }
}
