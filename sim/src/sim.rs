use std::sync::{Arc, Mutex};
use tokio::sync::watch;
use tokio::net::{TcpListener, UdpSocket};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::rpc::{asiair_udp_handler, asiair_tcp_handler};
use crate::rpc::protocol::{ASIAirRequest, ASIAirResponse};
use local_ip_address::local_ip;
use crate::rtc;

use super::ASIAirSim;

#[derive(Debug, Clone)]
pub struct ASIAirState {
    pub name: String,
    pub guid: String,
    pub ip: String,
    pub is_pi4: bool,
    pub model: String,
    pub ssid: String,
    pub connect_lock: bool,

    pub rtc: rtc::RTC,
    pub language: String,
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
                rtc: rtc::RTC::new(),
                language: "en".to_string(),
            })),
            shutdown_tx: None,
        }
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let udp_socket = UdpSocket::bind("0.0.0.0:4720").await?;
        let tcp_listener = TcpListener::bind("0.0.0.0:4720").await?;
        println!("ASIAIR Simulator listening on 0.0.0.0:4720 for both UDP and TCP");

        let udp_state = self.state.clone();
        let tcp_state = self.state.clone();

        let (shutdown_tx, shutdown_rx) = watch::channel(());
        self.shutdown_tx = Some(shutdown_tx);

        let mut udp_shutdown_rx = shutdown_rx.clone();
        let mut tcp_shutdown_rx = shutdown_rx.clone();

        tokio::spawn(async move {
            let mut buf = [0u8; 2048];
            loop {
                tokio::select! {
                    _ = udp_shutdown_rx.changed() => {
                        break;
                    }
                    read_result = udp_socket.recv_from(&mut buf) => { 
                        match read_result {
                            Ok((len, addr)) => {
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
                            Err(err) => {
                                eprintln!("Error receiving UDP data: {}", err);
                            }
                        }
                    }
                }
            }
        });

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = tcp_shutdown_rx.changed() => {
                        break;
                    }
                    read_result = tcp_listener.accept() => {
                        match read_result {
                            Ok((mut stream, addr)) => {
                                log::debug!("Received TCP connection from {}", addr);

                                let tcp_state = tcp_state.clone();
                                let mut per_connection_shutdown_rx = shutdown_rx.clone();
                                tokio::spawn(async move {
                                    let mut buf = [0u8; 2048];
                                    loop {
                                        tokio::select! {
                                            _ = per_connection_shutdown_rx.changed() => {
                                                break;
                                            }
                                            read_result = stream.read(&mut buf) => {
                                                match read_result {
                                                    Ok(len) if len > 0 => {
                                                        let data = &buf[..len];

                                                        if let Ok(text) = std::str::from_utf8(data) {
                                                            log::debug!("Received TCP from {}: {}", addr, text);

                                                            match serde_json::from_str::<ASIAirRequest>(text) {
                                                                Ok(req) => {
                                                                    let (result, code) = asiair_tcp_handler(&req.method, &req.params, tcp_state.clone());

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
                                                    }
                                                    Ok(_) => {
                                                        log::debug!("TCP connection from {} closed", addr);
                                                        break;
                                                    }
                                                    Err(err) => {
                                                        eprintln!("Error reading from TCP stream: {}", err);
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                });
                            },
                            Err(err) => {
                                eprintln!("Error accepting TCP connection: {}", err);
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }

    pub fn shutdown(&self) {
        if let Some(tx) = &self.shutdown_tx {
            println!("Shutting down ASIAIR simulator...");
            let _ = tx.send(());
        }
    }
}