use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize, Debug)]
pub struct ASIAirRequest {
    pub id: Value, // Changed to Value to handle both numbers and strings
    pub method: String,
    pub _name: Option<String>,
    pub params: Option<Value>,
}

#[derive(Serialize)]
pub struct ASIAirResponse {
    pub id: Value, // Changed to Value to match ASIAirRequest
    pub code: u8,
    pub jsonrpc: String,
    #[serde(rename = "Timestamp")]
    pub timestamp: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
}
