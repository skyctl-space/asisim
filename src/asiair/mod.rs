mod rpc;

use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, UdpSocket};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use rpc::{asiair_udp_handler, asiair_tcp_handler};
use rpc::protocol::{ASIAirRequest, ASIAirResponse};
use local_ip_address::local_ip;
use serde_json::{Value, Number}; // Import Value and Number for handling JSON types
use log::debug;

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
        let udp_socket = UdpSocket::bind("0.0.0.0:4720").await?;
        let tcp_listener = TcpListener::bind("0.0.0.0:4720").await?;
        println!("ASIAIR Simulator listening on 0.0.0.0:4720 for both UDP and TCP");

        let udp_state = self.state.clone();
        let tcp_state = self.state.clone();

        tokio::spawn(async move {
            let mut buf = [0u8; 2048];
            loop {
                let (len, addr) = udp_socket.recv_from(&mut buf).await.unwrap();
                let data = &buf[..len];

                if let Ok(text) = std::str::from_utf8(data) {
                    log::debug!("Received UDP from {}: {}", addr, text);

                    match serde_json::from_str::<ASIAirRequest>(text) {
                        Ok(req) => {
                            let (result, code) = asiair_udp_handler(&req.method, &req.params, udp_state.clone());

                            let response = ASIAirResponse {
                                id: req.id,
                                code: code as u8,
                                jsonrpc: "2.0".to_string(),
                                timestamp: "2025-05-06T00:00:00Z".to_string(),
                                method: req.method.clone(),
                                result: result,
                            };

                            let json = serde_json::to_string(&response).unwrap();
                            udp_socket.send_to(json.as_bytes(), addr).await.unwrap();
                            log::debug!("Sent UDP response to {}: {}", addr, json);
                        }
                        Err(err) => {
                            eprintln!("Failed to parse UDP JSON-RPC: {}", err);
                        }
                    }
                }
            }
        });

        tokio::spawn(async move {
            loop {
                let (mut stream, addr) = tcp_listener.accept().await.unwrap();
                log::debug!("Received TCP connection from {}", addr);

                let tcp_state = tcp_state.clone();
                tokio::spawn(async move {
                    let mut buf = [0u8; 2048];
                    let len = stream.read(&mut buf).await.unwrap();
                    let data = &buf[..len];

                    if let Ok(text) = std::str::from_utf8(data) {
                        log::debug!("Received TCP from {}: {}", addr, text);

                        match serde_json::from_str::<ASIAirRequest>(text) {
                            Ok(req) => {
                                let (result, code) = asiair_tcp_handler(&req.method, &req.params, tcp_state);

                                let response = ASIAirResponse {
                                    id: req.id,
                                    code: code as u8,
                                    jsonrpc: "2.0".to_string(),
                                    timestamp: "2025-05-06T00:00:00Z".to_string(),
                                    method: req.method.clone(),
                                    result: result,
                                };

                                let json = serde_json::to_string(&response).unwrap();
                                stream.write_all(json.as_bytes()).await.unwrap();
                                log::debug!("Sent TCP response to {}: {}", addr, json);
                            }
                            Err(err) => {
                                eprintln!("Failed to parse TCP JSON-RPC: {}", err);
                            }
                        }
                    }
                });
            }
        });

        Ok(())
    }

    pub async fn shutdown(&self) {
        // Stop the ASIAir simulation
    }
}