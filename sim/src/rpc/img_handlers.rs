use super::sample_raw::RAW_IMAGE_ZIP;
use super::ASIAirState;
use crate::sim::BinaryResult;
use serde_json::Value;
use std::sync::{Arc, Mutex};

pub fn get_current_img(_params: &Option<Value>, _state: Arc<Mutex<ASIAirState>>) -> BinaryResult {
    BinaryResult {
        data: RAW_IMAGE_ZIP.zip_data.to_vec(),
        width: RAW_IMAGE_ZIP.width,
        height: RAW_IMAGE_ZIP.height,
    }
}
