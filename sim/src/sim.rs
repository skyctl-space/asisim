use crate::rpc::protocol::{ASIAirRequest, ASIAirResponse};
use crate::rpc::{asiair_tcp_handler, asiair_udp_handler};
use crate::rtc;
use local_ip_address::local_ip;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, UdpSocket};
use tokio::sync::watch;

use super::ASIAirSim;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ASIAirPage {
    Preview,
    Focus,
    PA,
    Stack,
    Autosave,
    Plan,
    RMTP,
}

impl ASIAirPage {
    pub fn as_str(&self) -> &str {
        match self {
            ASIAirPage::Preview => "preview",
            ASIAirPage::Focus => "focus",
            ASIAirPage::PA => "pa",
            ASIAirPage::Stack => "stack",
            ASIAirPage::Autosave => "autosave",
            ASIAirPage::Plan => "plan",
            ASIAirPage::RMTP => "rmtp",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AnnotateState {
    pub is_working: bool,
    pub lapse_ms: u32,
}

#[derive(Debug, Clone)]
pub struct SolveState {
    pub is_working: bool,
    pub lapse_ms: u32,
    pub filename: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ExposureModes {
    Single,
    Continuous,
}

impl ExposureModes {
    pub fn as_str(&self) -> &str {
        match self {
            ExposureModes::Single => "single",
            ExposureModes::Continuous => "continuous",
        }
    }
}

#[derive(Debug, Clone)]
pub enum CaptureStatus {
    Idle,
    // Working,
}

impl CaptureStatus {
    pub fn as_str(&self) -> &str {
        match self {
            CaptureStatus::Idle => "idle",
            // CaptureStatus::Working => "working",
        }
    }
}

#[derive(Debug, Clone)]
pub struct CaptureState {
    pub exposure_mode: ExposureModes,
    pub is_working: bool,
    pub state: CaptureStatus,
}

#[derive(Debug, Clone)]
pub struct PaState {
    pub is_working: bool,
}

#[derive(Debug, Clone)]
pub struct AutoGotoState {
    pub is_working: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum FrameType {
    Light,
    Dark,
    Flat,
    Bias,
}

impl FrameType {
    pub fn as_str(&self) -> &str {
        match self {
            FrameType::Light => "light",
            FrameType::Dark => "dark",
            FrameType::Flat => "flat",
            FrameType::Bias => "bias",
        }
    }
}

#[derive(Debug, Clone)]
pub struct StackState {
    pub is_working: bool,
    pub frame_type: FrameType,
    pub stacked_frame: u32,
    pub dropped_frame: u32,
    pub total_frame: u32,
}
#[derive(Debug, Clone)]
pub struct ExportImageState {
    pub is_working: bool,
    pub success_frame: u32,
    pub total_frame: u32,
    pub keep: bool,
    pub dst_storage: String,
}

#[derive(Debug, Clone)]
pub struct MeridFlipState {
    pub is_working: bool,
}

#[derive(Debug, Clone)]
pub struct AutoFocusResult {
    // Fields for AutoFocusResult
}

#[derive(Debug, Clone)]
pub struct AutoFocuserReason {
    pub comment: String,
    pub code: u32,
}

#[derive(Debug, Clone)]
pub struct AutoFocusState {
    #[allow(dead_code)]
    pub result: AutoFocusResult,
    pub is_working: bool,
    pub focuser_opened: bool,
    pub reason: AutoFocuserReason,
}

#[derive(Debug, Clone)]
pub struct FindStarState {
    pub is_working: bool,
    pub lapse_ms: u32,
}

#[derive(Debug, Clone)]
pub struct AviRecordState {
    pub is_working: bool,
    pub lapse_sec: u32,
    pub fps: f32,
    pub write_file_fps: f32,
}

#[derive(Debug, Clone)]
pub struct RtmpState {
    pub is_working: bool,
}

#[derive(Debug, Clone)]
pub struct AutoExpState {
    pub is_working: bool,
}

#[derive(Debug, Clone)]
pub struct RestartGuideState {
    pub is_working: bool,
}

#[derive(Debug, Clone)]
pub struct BatchStackState {
    pub is_working: bool,
}

#[derive(Debug, Clone)]
pub struct DemonstrateState {
    pub is_working: bool,
}

#[derive(Debug, Clone)]
pub struct FormatDriveState {
    pub is_working: bool,
}

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

    pub page: ASIAirPage,
    pub annotate: AnnotateState,
    pub solve: SolveState,
    pub capture: CaptureState,
    pub pa: PaState,
    pub auto_goto: AutoGotoState,
    pub stack: StackState,
    pub export_image: ExportImageState,
    pub merid_flip: MeridFlipState,
    pub auto_focus: AutoFocusState,
    pub find_star: FindStarState,
    pub avi_record: AviRecordState,
    pub rtmp: RtmpState,
    pub auto_exp: AutoExpState,
    pub restart_guide: RestartGuideState,
    pub batch_stack: BatchStackState,
    pub demonstrate: DemonstrateState,
    pub format_drive: FormatDriveState,
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

                page: ASIAirPage::Preview,
                annotate: AnnotateState {
                    is_working: false,
                    lapse_ms: 0,
                },
                solve: SolveState {
                    is_working: false,
                    lapse_ms: 0,
                    filename: "".to_string(),
                },
                capture: CaptureState {
                    exposure_mode: ExposureModes::Single,
                    is_working: false,
                    state: CaptureStatus::Idle,
                },
                pa: PaState { is_working: false },
                auto_goto: AutoGotoState { is_working: false },
                stack: StackState {
                    is_working: false,
                    frame_type: FrameType::Light,
                    stacked_frame: 0,
                    dropped_frame: 0,
                    total_frame: 0,
                },
                export_image: ExportImageState {
                    is_working: false,
                    success_frame: 0,
                    total_frame: 0,
                    keep: false,
                    dst_storage: "".to_string(),
                },
                merid_flip: MeridFlipState { is_working: false },
                auto_focus: AutoFocusState {
                    result: AutoFocusResult {},
                    is_working: false,
                    focuser_opened: false,
                    reason: AutoFocuserReason {
                        comment: "manual".to_string(),
                        code: 0,
                    },
                },
                find_star: FindStarState {
                    is_working: false,
                    lapse_ms: 0,
                },
                avi_record: AviRecordState {
                    is_working: false,
                    lapse_sec: 0,
                    fps: 10.0,
                    write_file_fps: 0.0,
                },
                rtmp: RtmpState { is_working: false },
                auto_exp: AutoExpState { is_working: false },
                restart_guide: RestartGuideState { is_working: false },
                batch_stack: BatchStackState { is_working: false },
                demonstrate: DemonstrateState { is_working: false },
                format_drive: FormatDriveState { is_working: false },
            })),
            shutdown_tx: None,
        }
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let udp_socket = UdpSocket::bind("0.0.0.0:4720").await?;
        let tcp_listener = TcpListener::bind("0.0.0.0:4700").await?;
        println!("ASIAIR Simulator listening on 0.0.0.0");

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

                                                                    let mut json = serde_json::to_string(&response).unwrap();
                                                                    json.push_str("\r\n");
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
