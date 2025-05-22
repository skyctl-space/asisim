use super::ASIAirState;
use crate::sim::CameraState;
use crate::sim::CAMERAS_INFO;
use crate::sim::CAMERA_CONTROL_TYPES;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};

pub fn get_connected_cameras(_: &Option<Value>, state: Arc<Mutex<ASIAirState>>) -> Result<(Value, u8), (String, u8)> {
    let state = state.lock().unwrap();

    Ok((serde_json::to_value(&state.connected_cameras).unwrap(), 0))
}

pub fn get_camera_state(_: &Option<Value>, state: Arc<Mutex<ASIAirState>>) -> Result<(Value, u8), (String, u8)> {
    let state = state.lock().unwrap();

    Ok((serde_json::to_value(&state.camera_state).unwrap(), 0))
}

pub async fn open_camera(_: &Option<Value>, state: Arc<Mutex<ASIAirState>>, event_tx: tokio::sync::mpsc::Sender<Value>) -> Result<(Value, u8), (String, u8)> {
    // Need this pattern to avoid sending the MutexGuard across the async call 
    let mut success: bool = false;

    {
        let mut state = state.lock().unwrap();

        for camera in state.connected_cameras.iter() {
            if camera.name == state.app_setting.main_camera_name {
                state.camera_state = CameraState::Idle {
                    name: state.app_setting.main_camera_name.clone(),
                    path: camera.path.clone(),
                };
                success = true;
                break;
            }
        }
    }

    if !success {
        return Err(("Camera not found".to_string(), 1));
    }

    let _ = event_tx.send(json!({
        "Event": "CameraStateChange",
        "Timestamp": "2025-05-06T00:00:00Z".to_string(),
    })).await;

    return Ok((json!(0), 0));
}

pub async fn close_camera(_: &Option<Value>, state: Arc<Mutex<ASIAirState>>, event_tx: tokio::sync::mpsc::Sender<Value>) -> Result<(Value, u8), (String, u8)> {
    // Need this pattern to avoid sending the MutexGuard across the async call 
    {
        let mut state = state.lock().unwrap();
        state.camera_state = CameraState::Close;
    }

    let _ = event_tx.send(json!({
        "Event": "CameraStateChange",
        "Timestamp": "2025-05-06T00:00:00Z".to_string(),
    })).await;

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
                    return Err(("unexpected param".to_string(), 1));
                }
                let control_type = CAMERA_CONTROL_TYPES.get(control_name).unwrap();

                match *control_type {
                    "number" => {
                        let value: i64 = match control_name {
                            "Exposure" => state.camera_controls.exposure,
                            "Gain" => state.camera_controls.gain,
                            "CoolerOn" => state.camera_controls.cooler_on,
                            "CoolPowerPerc" => state.camera_controls.cool_power_perc,
                            "Temperature" => state.camera_controls.temperature,
                            "AntiDewHeater" => state.camera_controls.anti_dew_heater,
                            "Red" => state.camera_controls.red,
                            "Blue" => state.camera_controls.blue,
                            "MonoBin" => state.camera_controls.mono_bin,
                            _ => {
                                return Err(("unexpected param".to_string(), 1));
                            }
                        };

                        return Ok((json!({
                            "name": control_name,
                            "type": control_type,
                            "value": value,
                        }), 0));
                    }
                    "text" => {
                        let value: f64 = match control_name {
                            "TargetTemp" => state.camera_controls.target_temp,
                            _ => {
                                return Err(("unexpected param".to_string(), 1));
                            }
                        };

                        return Ok((json!({
                            "name": control_name,
                            "type": control_type,
                            "value": value,
                        }), 0));
                    }
                    _ => {
                        return Err(("unexpected param".to_string(), 1));
                    }
                }
            }

            return Err(("unknown control name".to_string(), 1));
        }
        None => return Err(("params is not provided".to_string(), 1)),
    }
}

pub fn set_control_value(
    params: &Option<Value>,
    state: Arc<Mutex<ASIAirState>>,
) -> Result<(Value, u8), (String, u8)> {
    let mut state = state.lock().unwrap();

    match params {
        Some(value) => {
            if !value.is_array() {
                return Err(("params is not an array".to_string(), 1));
            }
            match value[0].as_str() {
                Some(control_name) => {
                    match control_name {
                        "Exposure" => {
                            if let Some(exposure) = value[1].as_i64() {
                                state.camera_controls.exposure = exposure;
                            } else {
                                return Err(("invalid exposure value".to_string(), 1));
                            }
                        }
                        "Gain" => {
                            if let Some(gain) = value[1].as_i64() {
                                state.camera_controls.gain = gain;
                            } else {
                                return Err(("invalid gain value".to_string(), 1));
                            }
                        }
                        "CoolerOn" => {
                            if let Some(cooler_on) = value[1].as_i64() {
                                state.camera_controls.cooler_on = cooler_on;
                            } else {
                                return Err(("invalid cooler_on value".to_string(), 1));
                            }
                        }
                        "CoolPowerPerc" => {
                            if let Some(cool_power_perc) = value[1].as_i64() {
                                state.camera_controls.cool_power_perc = cool_power_perc;
                            } else {
                                return Err(("invalid cool_power_perc value".to_string(), 1));
                            }
                        }
                        "TargetTemp" => {
                            if let Some(target_temp) = value[1].as_f64() {
                                state.camera_controls.target_temp = target_temp;
                            } else {
                                return Err(("invalid target_temp value".to_string(), 1));
                            }
                        }
                        "AntiDewHeater" => {
                            if let Some(anti_dew_heater) = value[1].as_i64() {
                                state.camera_controls.anti_dew_heater = anti_dew_heater;
                            } else {
                                return Err(("invalid anti_dew_heater value".to_string(), 1));
                            }
                        }
                        "Red" => {
                            if let Some(red) = value[1].as_i64() {
                                state.camera_controls.red = red;
                            } else {
                                return Err(("invalid red value".to_string(), 1));
                            }
                        }
                        "Blue" => {
                            if let Some(blue) = value[1].as_i64() {
                                state.camera_controls.blue = blue;
                            } else {
                                return Err(("invalid blue value".to_string(), 1));
                            }
                        }
                        "MonoBin" => {
                            if let Some(mono_bin) = value[1].as_i64() {
                                state.camera_controls.mono_bin = mono_bin;
                            } else {
                                return Err(("invalid mono_bin value".to_string(), 1));
                            }
                        }
                        _ => {
                            return Err(("unexpected param".to_string(), 1));
                        }
                    }
                }
                None => return Err(("unexpect control name".to_string(), 1)),
            }
        }
        None => return Err(("params is not provided".to_string(), 1)),
    }

    Ok((json!(0), 0))
}

pub fn get_camera_bin(
    _: &Option<Value>,
    state: Arc<Mutex<ASIAirState>>,
) -> Result<(Value, u8), (String, u8)> {
    let state = state.lock().unwrap();

    Ok((serde_json::to_value(&state.camera_bin).unwrap(), 0))
}

pub fn set_camera_bin(
    params: &Option<Value>,
    state: Arc<Mutex<ASIAirState>>,
) -> Result<(Value, u8), (String, u8)> {
    let mut state = state.lock().unwrap();

    match params {
        Some(value) => {
            if !value.is_array() {
                return Err(("params is not an array".to_string(), 1));
            }
            if let Some(bin) = value[0].as_u64() {
                state.camera_bin = bin as u32;
            } else {
                return Err(("invalid bin value".to_string(), 1));
            }
        }
        None => return Err(("params is not provided".to_string(), 1)),
    }

    Ok((json!(0), 0))
}

pub async fn start_exposure(
    params: &Option<Value>,
    state: Arc<Mutex<ASIAirState>>,
    event_tx: tokio::sync::mpsc::Sender<Value>
) -> Result<(Value, u8), (String, u8)> {
    let exposure_us: i64;
    let gain: i64;
    let page: String;

    {
        let state = state.lock().unwrap();
        exposure_us = state.camera_controls.exposure;
        gain = state.camera_controls.gain;
        page = state.app_state.page.as_str().to_string();
    }

    match params {
        Some(value) => {
            if !value.is_array() {
                return Err(("params is not an array".to_string(), 1));
            }
            match value[0].as_str() {
                Some(exposure_type) => {
                    match exposure_type {
                        "light" => {
                            let _ = event_tx.send(json!({
                                "Event": "Exposure",
                                "Timestamp": "2025-05-06T00:00:00Z".to_string(),
                                "page": page,
                                "state": "start",
                                "exp_us": exposure_us,
                                "gain": gain,
                            })).await;

                            tokio::spawn(async move {
                                tokio::time::sleep(std::time::Duration::from_millis((exposure_us / 1000).try_into().unwrap())).await;

                                let _ = event_tx.send(json!({
                                    "Event": "Exposure",
                                    "Timestamp": "2025-05-06T00:00:00Z".to_string(),
                                    "state": "downloading"
                                })).await;

                                let _ = event_tx.send(json!({
                                    "Event": "Exposure",
                                    "Timestamp": "2025-05-06T00:00:00Z".to_string(),
                                    "state": "complete"
                                })).await;
                            });
                        }
                        _ => return Err(("unexpected param".to_string(), 1)),
                    }
                }
                None => return Err(("unexpected param".to_string(), 1)),
            }
        }
        None => return Err(("params is not provided".to_string(), 1)),
    }

    Ok((json!(0), 0))
}