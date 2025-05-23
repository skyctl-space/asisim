use super::ASIAirState;
use serde_json::Value;
use std::sync::{Arc, Mutex};

pub fn get_app_state(_: &Option<Value>, state: Arc<Mutex<ASIAirState>>) -> Result<(Value, u8), (String, u8)> {
    let state = state.lock().unwrap();

    Ok((serde_json::to_value(&state.app_state).unwrap(), 0))
}

pub fn get_app_setting(_: &Option<Value>, state: Arc<Mutex<ASIAirState>>) -> Result<(Value, u8), (String, u8)> {
    let state = state.lock().unwrap();

    Ok((serde_json::to_value(&state.app_setting).unwrap(), 0))
}

pub fn set_app_setting(
    params: &Option<Value>,
    state: Arc<Mutex<ASIAirState>>,
) -> Result<(Value, u8), (String, u8)> {
    let mut state = state.lock().unwrap();

    if let Some(params) = params {
       for (key, value) in params[0].as_object().unwrap() {
            match key.as_str() {
                "main_camera_name" => {
                    state.app_setting.main_camera_name = serde_json::from_value(value.clone()).unwrap();
                }
                "guide_camera_name" => {
                    state.app_setting.guide_camera_name = serde_json::from_value(value.clone()).unwrap();
                }
                "autogoto_exp_us" => {
                    state.app_setting.autogoto_exp_us = serde_json::from_value(value.clone()).unwrap();
                }
                "comets_version" => {
                    state.app_setting.comets_version = serde_json::from_value(value.clone()).unwrap();
                }
                "continuous_preview" => {
                    state.app_setting.continuous_preview = serde_json::from_value(value.clone()).unwrap();
                }
                "goto_auto" => {
                    state.app_setting.goto_auto = serde_json::from_value(value.clone()).unwrap();
                }
                "flat_auto_exp" => {
                    state.app_setting.flat_auto_exp = serde_json::from_value(value.clone()).unwrap();
                }
                "light_custom_exp" => {
                    state.app_setting.light_custom_exp = serde_json::from_value(value.clone()).unwrap();
                }
                "flat_custom_exp" => {
                    state.app_setting.flat_custom_exp = serde_json::from_value(value.clone()).unwrap();
                }
                "dark_custom_exp" => {
                    state.app_setting.dark_custom_exp = serde_json::from_value(value.clone()).unwrap();
                }
                "bias_custom_exp" => {
                    state.app_setting.bias_custom_exp = serde_json::from_value(value.clone()).unwrap();
                }
                "bias_exposure" => {
                    state.app_setting.bias_exposure = serde_json::from_value(value.clone()).unwrap();
                }
                "flat_exposure" => {
                    state.app_setting.flat_exposure = serde_json::from_value(value.clone()).unwrap();
                }
                "light_exposure" => {
                    state.app_setting.light_exposure = serde_json::from_value(value.clone()).unwrap();
                }
                "dark_exposure" => {
                    state.app_setting.dark_exposure = serde_json::from_value(value.clone()).unwrap();
                }
                "flat_bin" => {
                    state.app_setting.flat_bin = serde_json::from_value(value.clone()).unwrap();
                }
                "bias_bin" => {
                    state.app_setting.bias_bin = serde_json::from_value(value.clone()).unwrap();
                }
                "dark_bin" => {
                    state.app_setting.dark_bin = serde_json::from_value(value.clone()).unwrap();
                }
                "light_bin" => {
                    state.app_setting.light_bin = serde_json::from_value(value.clone()).unwrap();
                }
                "guide_rate" => {
                    state.app_setting.guide_rate = serde_json::from_value(value.clone()).unwrap();
                }
                "goto_target_dec" => {
                    state.app_setting.goto_target_dec = serde_json::from_value(value.clone()).unwrap();
                }
                "goto_target_ra" => {
                    state.app_setting.goto_target_ra = serde_json::from_value(value.clone()).unwrap();
                }
                "goto_target_name" => {
                    state.app_setting.goto_target_name = serde_json::from_value(value.clone()).unwrap();
                }
                _ => return Err(("Invalid parameter".to_string(), 1)),
            }
        }

        return Ok((serde_json::to_value(&state.app_setting).unwrap(), 0));
    }

    Err(("Invalid parameters".to_string(), 1))
}
