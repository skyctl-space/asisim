use serde_json::{Number, Value};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct ASIAirRequest {
    pub id: Number,
    pub method: String,
    pub name: Option<String>,
    pub params: Option<Value>,
}

#[derive(Serialize)]
pub struct ASIAirResponse {
    pub id: Number,
    pub code: u8,
    pub jsonrpc: String,
    #[serde(rename = "Timestamp")]
    pub timestamp: String,
    pub method: String,
    pub result: Value,
}