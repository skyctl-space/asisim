mod rpc;

use std::sync::{Arc, Mutex};
use tokio::net::UdpSocket;
use rpc::handle_asiair_method;
use rpc::protocol::{ASIAirRequest, ASIAirResponse};
use local_ip_address::local_ip;

#[derive(Debug, Clone)]
struct ASIAirState {
    name: String,
    guid: String,
    ip: String,
    is_pi4: bool,
    model: String,
    ssid: String,
    connect_lock: bool,
}

#[derive(Debug, Clone)]
pub struct ASIAirSim {
    // ASIAir simulation state
    state: Arc<Mutex<ASIAirState>>,
}

impl ASIAirSim {
    pub fn new() -> Self {
        let local_ip = local_ip().unwrap_or_else(|_| "0.0.0.0".parse().unwrap());

        ASIAirSim {
            state: Arc::new(Mutex::new(ASIAirState {
                name: "ASIAIR_SIM".to_string(),
                guid: "1234567890".to_string(),
                ip: local_ip.to_string(), // Set the local IP address
                is_pi4: false,
                model: "ZWO AirPlus-RK3568 (Linux)".to_string(),
                ssid: "ASIAir SIM".to_string(),
                connect_lock: false,
            })),
        }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let socket = UdpSocket::bind("0.0.0.0:4720").await?;
        println!("ASIAIR Simulator listening on 0.0.0.0:4720");

        let mut buf = [0u8; 2048];

        loop {
            let (len, addr) = socket.recv_from(&mut buf).await?;
            let data = &buf[..len];

            if let Ok(text) = std::str::from_utf8(data) {
                println!("Received from {}: {}", addr, text);

                match serde_json::from_str::<ASIAirRequest>(text) {
                    Ok(req) => {
                        let (result, code) = handle_asiair_method(&req.method, &req.params, self.state.clone());

                        let response = ASIAirResponse {
                            jsonrpc: "2.0".to_string(),
                            timestamp: "2023-10-01T12:00:00Z".to_string(),
                            code: code,
                            method: req.method.clone(),
                            result: result,
                            id: req.id.clone(),
                        };

                        let json = serde_json::to_string(&response).unwrap();
                        socket.send_to(json.as_bytes(), addr).await?;
                        println!("Sent response to {}: {}", addr, json);
                    }
                    Err(err) => {
                        eprintln!("Failed to parse JSON-RPC: {}", err);
                    }
                }
            }
        }
    }

    pub async fn shutdown(&self) {
        // Stop the ASIAir simulation
    }
}