use super::ASIAir;
use super::MainCamera;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use super::ASIAirCommand;
use super::BinaryResult;
use tokio::sync::oneshot;
use std::sync::atomic::Ordering;
use std::io::Cursor;
use std::io::Read;
use zip::ZipArchive;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectedCamera {
    pub name: String,
    pub id: u32,
    pub path: String,
    pub dslr: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "state")]
pub enum CameraState {
    #[serde(rename = "close")]
    Close,
    #[serde(rename = "idle")]
    Idle { name: String, path: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraInfo {
    pub chip_size: [u32; 2],
    pub bins: Vec<u32>,
    pub pixel_size_um: f32,
    pub unity_gain: u32,
    pub has_cooler: bool,
    pub is_color: bool,
    pub is_usb3_host: bool,
    pub debayer_pattern: String,
}

enum CameraControl {
    Exposure,
    Gain,
    CoolerOn,
    Temperature,
    CoolPowerPerc,
    TargetTemp,
    AntiDewHeater,
    LedOn,
    FanHalfSpeed,
    FrameSize,
    Red,
    Blue,
    MonoBin
}

impl CameraControl {
    pub fn to_str(&self) -> &'static str {
        match self {
            CameraControl::Exposure => "Exposure",
            CameraControl::Gain => "Gain",
            CameraControl::CoolerOn => "CoolerOn",
            CameraControl::Temperature => "Temperature",
            CameraControl::CoolPowerPerc => "CoolPowerPerc",
            CameraControl::TargetTemp => "TargetTemp",
            CameraControl::AntiDewHeater => "AntiDewHeater",
            CameraControl::LedOn => "LedOn",
            CameraControl::FanHalfSpeed => "FanHalfSpeed",
            CameraControl::FrameSize => "FrameSize",
            CameraControl::Red => "Red",
            CameraControl::Blue => "Blue",
            CameraControl::MonoBin => "MonoBin",
        }
    }
}

impl MainCamera {
    pub async fn start_exposure(
        &mut self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let method = "start_exposure";
        let params = Some(serde_json::json!([ "light" ]));
        self.rpc_request_4700(method, params).await?;
        Ok(())
    }

    pub async fn get_info(&self) -> Result<CameraInfo, Box<dyn std::error::Error + Send + Sync>> {
        let method = "get_camera_info";
        let result = self.rpc_request_4700(method, None).await?;

        let info: CameraInfo = serde_json::from_value(result)?;
        return Ok(info);
    }

    pub async fn get_exposure(
        &mut self
    ) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        let method = "get_control_value";
        let params = Some(serde_json::json!([ CameraControl::Exposure.to_str(), true ]));
        let result = self.rpc_request_4700(method, params).await?;

        let value: u64 = serde_json::from_value(result["value"].clone())?;
        Ok(value)
    }

    pub async fn set_exposure(
        &mut self,
        exposure: u64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let method = "set_control_value";
        let params = Some(serde_json::json!([ CameraControl::Exposure.to_str(), exposure ]));
        self.rpc_request_4700(method, params).await?;
        Ok(())
    }

    pub async fn get_temperature(
        &mut self
    ) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
        let method = "get_control_value";
        let params = Some(serde_json::json!([ CameraControl::Temperature.to_str(), true ]));
        let result = self.rpc_request_4700(method, params).await?;

        let value: i64 = serde_json::from_value(result["value"].clone())?;
        Ok(value)
    }

    pub async fn get_cooler(
        &mut self
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let method = "get_control_value";
        let params = Some(serde_json::json!([ CameraControl::CoolerOn.to_str(), true ]));
        let result = self.rpc_request_4700(method, params).await?;

        let value: u64 = serde_json::from_value(result["value"].clone())?;
        let value = if value == 1 { true } else { false };
        Ok(value)
    }

    pub async fn set_cooler(
        &mut self,
        cooler_on: bool,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let method = "set_control_value";
        let value : u64 = if cooler_on { 1 } else { 0 };
        let params = Some(serde_json::json!([ CameraControl::CoolerOn.to_str(), value ]));
        self.rpc_request_4700(method, params).await?;
        Ok(())
    }

    pub async fn get_gain(
        &mut self
    ) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
        let method = "get_control_value";
        let params = Some(serde_json::json!([ CameraControl::Gain.to_str(), true ]));
        let result = self.rpc_request_4700(method, params).await?;

        let value: i64 = serde_json::from_value(result["value"].clone())?;
        Ok(value)
    }

    pub async fn set_gain(
        &mut self,
        gain: i64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let method = "set_control_value";
        let params = Some(serde_json::json!([ CameraControl::Gain.to_str(), gain ]));
        self.rpc_request_4700(method, params).await?;
        Ok(())
    }

    pub async fn get_cooler_percentage(
        &mut self
    ) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        let method = "get_control_value";
        let params = Some(serde_json::json!([ CameraControl::CoolPowerPerc.to_str(), true ]));
        let result = self.rpc_request_4700(method, params).await?;

        let value: u64 = serde_json::from_value(result["value"].clone())?;
        Ok(value)
    }

    pub async fn set_cooler_percentage(
        &mut self,
        cooler_percentage: u64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let method = "set_control_value";
        let params = Some(serde_json::json!([ CameraControl::CoolPowerPerc.to_str(), cooler_percentage ]));
        self.rpc_request_4700(method, params).await?;
        Ok(())
    }

    pub async fn get_target_temperature(
        &mut self
    ) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
        let method = "get_control_value";
        let params = Some(serde_json::json!([ CameraControl::TargetTemp.to_str(), true ]));
        let result = self.rpc_request_4700(method, params).await?;

        let value: f64 = serde_json::from_value(result["value"].clone())?;
        Ok(value)
    }

    pub async fn set_target_temperature(
        &mut self,
        target_temperature: f64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let method = "set_control_value";
        let params = Some(serde_json::json!([ CameraControl::TargetTemp.to_str(), target_temperature ]));
        self.rpc_request_4700(method, params).await?;
        Ok(())
    }

    pub async fn get_anti_dew_heater(
        &mut self
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let method = "get_control_value";
        let params = Some(serde_json::json!([ CameraControl::AntiDewHeater.to_str(), true ]));
        let result = self.rpc_request_4700(method, params).await?;

        let value: u64 = serde_json::from_value(result["value"].clone())?;
        let value = if value == 1 { true } else { false };
        Ok(value)
    }

    pub async fn set_anti_dew_heater(
        &mut self,
        anti_dew_heater: bool,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let method = "set_control_value";
        let value : u64 = if anti_dew_heater { 1 } else { 0 };
        let params = Some(serde_json::json!([ CameraControl::AntiDewHeater.to_str(), value ]));
        self.rpc_request_4700(method, params).await?;
        Ok(())
    }

    pub async fn get_red_gain(
        &mut self
    ) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        let method = "get_control_value";
        let params = Some(serde_json::json!([ CameraControl::Red.to_str(), true ]));
        let result = self.rpc_request_4700(method, params).await?;

        let value: u64 = serde_json::from_value(result["value"].clone())?;
        Ok(value)
    }

    pub async fn set_red_gain(
        &mut self,
        red_gain: u64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let method = "set_control_value";
        let params = Some(serde_json::json!([ CameraControl::Red.to_str(), red_gain ]));
        self.rpc_request_4700(method, params).await?;
        Ok(())
    }

    pub async fn get_blue_gain(
        &mut self
    ) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        let method = "get_control_value";
        let params = Some(serde_json::json!([ CameraControl::Blue.to_str(), true ]));
        let result = self.rpc_request_4700(method, params).await?;

        let value: u64 = serde_json::from_value(result["value"].clone())?;
        Ok(value)
    }

    pub async fn set_blue_gain(
        &mut self,
        blue_gain: u64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let method = "set_control_value";
        let params = Some(serde_json::json!([ CameraControl::Blue.to_str(), blue_gain ]));
        self.rpc_request_4700(method, params).await?;
        Ok(())
    }

    pub async fn get_mono_bin(
        &mut self
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let method = "get_control_value";
        let params = Some(serde_json::json!([ CameraControl::MonoBin.to_str(), true ]));
        let result = self.rpc_request_4700(method, params).await?;

        let value: u64 = serde_json::from_value(result["value"].clone())?;
        let value = if value == 1 { true } else { false };
        Ok(value)
    }

    pub async fn set_mono_bin(
        &mut self,
        mono_bin: bool,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let method = "set_control_value";
        let value : u64 = if mono_bin { 1 } else { 0 };
        let params = Some(serde_json::json!([ CameraControl::MonoBin.to_str(), value ]));
        self.rpc_request_4700(method, params).await?;
        Ok(())
    }

    pub async fn get_bin(
        &mut self
    ) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
        let method = "get_camera_bin";
        let result = self.rpc_request_4700(method, None).await?;

        let value: u32 = serde_json::from_value(result.clone())?;
        Ok(value)
    }

    pub async fn set_bin(
        &mut self,
        bin: u32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let method = "set_camera_bin";
        let params = Some(serde_json::json!([ bin ]));
        self.rpc_request_4700(method, params).await?;
        Ok(())
    }

    pub async fn get_current_img(
        &mut self,
    ) -> Result<(Vec<u8>, u16, u16), Box<dyn std::error::Error + Send + Sync>> {
        let method = "get_current_img";
        let result = self.rpc_request_4800(method, None).await?;

        let cursor = Cursor::new(&result.data);
        let mut archive = ZipArchive::new(cursor)?;

        // Assuming you want the first file in the archive
        if archive.len() == 0 {
            return Err("Zip archive is empty".into());
        }
        let mut file = archive.by_index(0)?;
        let mut extracted_data = Vec::new();
        file.read_to_end(&mut extracted_data)?;

        return Ok((extracted_data, result.width, result.height));
    }

    pub async fn rpc_request_4700(
        &self,
        method: &str,
        params: Option<Value>,
    ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        if !self.should_be_connected.load(Ordering::SeqCst) {
            return Err("Not connected".into());
        }
        if let Some(tx) = &self.tx_4700 {
            let (response_tx, response_rx) = oneshot::channel();
            let command = ASIAirCommand::Get {
                method: method.to_string(),
                params,
                tx: response_tx,
            };
            tx.send(command).await.unwrap();

            // Wait for the response with a timeout
            match tokio::time::timeout(self.cmd_timeout, response_rx).await {
                Ok(Ok(response)) => response,
                Ok(Err(_)) | Err(_) => Err("Failed to get response".into()),
            }
        } else {
            Err("Not connected".into())
        }
    }

    pub async fn rpc_request_4800(
        &self,
        method: &str,
        params: Option<Value>,
    ) -> Result<BinaryResult, Box<dyn std::error::Error + Send + Sync>> {
        if !self.should_be_connected.load(Ordering::SeqCst) {
            return Err("Not connected".into());
        }
        if let Some(tx) = &self.tx_4800 {
            let (response_tx, response_rx) = oneshot::channel();
            let command = ASIAirCommand::BinaryGet {
                method: method.to_string(),
                params,
                tx: response_tx,
            };
            tx.send(command).await.unwrap();

            // Wait for the response with a timeout
            match tokio::time::timeout(self.cmd_timeout, response_rx).await {
                Ok(Ok(result)) => result,
                Ok(Err(_)) | Err(_) => Err("Failed to get response".into()),
            }
        } else {
            Err("Not connected".into())
        }
    }
}


impl ASIAir {
    pub fn get_main_camera(&self) -> Option<MainCamera> {
        self.main_camera.clone()
    }

    pub async fn get_connected_cameras(
        &mut self,
    ) -> Result<Vec<ConnectedCamera>, Box<dyn std::error::Error + Send + Sync>> {
        let method = "get_connected_cameras";
        let result = self.rpc_request_4700(method, None).await?;

        let cameras: Vec<ConnectedCamera> = serde_json::from_value(result)?;
        Ok(cameras)
    }

    async fn get_camera_state(
        &mut self
    ) -> Result<CameraState, Box<dyn std::error::Error + Send + Sync>> {
        let method = "get_camera_state";
        let result = self.rpc_request_4700(method, None).await?;

        let state: CameraState = serde_json::from_value(result)?;
        Ok(state)
    }

    pub async fn open_main_camera(
        &mut self
        //camera: ConnectedCamera
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let method = "open_camera";
        self.rpc_request_4700(method, None).await?;

        self.main_camera = Some(MainCamera {
            tx_4700: self.tx_4700.clone(),
            tx_4800: self.tx_4800.clone(),
            cmd_timeout: self.cmd_timeout,
            should_be_connected: self.should_be_connected.clone(),
        });

        Ok(())
    }

    pub async fn close_main_camera(
        &mut self
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let method = "close_camera";
        self.rpc_request_4700(method, None).await?;

        self.main_camera = None;

        Ok(())
    }
}

