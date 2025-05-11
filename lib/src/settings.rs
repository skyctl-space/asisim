use chrono::DateTime;
use chrono::Timelike;
use chrono::Datelike;
use chrono_tz::Tz;
use serde_json::json;
use serde::Serialize;

use super::ASIAir;
use super::ASIAirLanguage;

#[derive(Serialize)]
struct TimeParams {
    time_zone: String,
    hour: u32,
    min: u32,
    sec: u32,
    day: u32,
    year: i32,
    mon: u32,
}

impl ASIAirLanguage {
    pub fn as_str(&self) -> &'static str {
        match self {
            ASIAirLanguage::English => "en",
        }
    }
}

impl ASIAir {
    pub async fn set_time(&mut self, date_time: DateTime<Tz>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let method = "pi_set_time";
        let response = self.rpc_request(method,
         Some(
            json!(vec![
                TimeParams {
                    time_zone: date_time.timezone().name().to_string(),
                    hour: date_time.hour(),
                    min: date_time.minute(),
                    sec: date_time.second(),
                    day: date_time.day(),
                    year: date_time.year(),
                    mon: date_time.month(),
                }])
        )).await;
        if let Ok(value) = response {
            if value.as_i64() == Some(0) {
                Ok(())
            } else {
                return Err("unexpected response".into());
            }
        } else {
            response
                .map(|_| ())
                .map_err(|e| {
                    log::debug!("{} failed: {}", method, e);
                    e
                })
        }
    }


    pub async fn set_language(&mut self, lang: ASIAirLanguage) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let method = "set_setting";
        let response = self.rpc_request(method, Some(json!({ "lang": lang.as_str() }))).await;
        if let Ok(value) = response {
            if value.as_i64() == Some(0) {
                Ok(())
            } else {
                return Err("unexpected response".into());
            }
        } else {
            response
                .map(|_| ())
                .map_err(|e| {
                    log::debug!("{} failed: {}", method, e);
                    e
                })
        }
    }

    
}