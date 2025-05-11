mod connection;
mod settings;

use tokio::time::Duration;
use tokio::sync::{mpsc, oneshot, watch};
use serde_json::Value;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, atomic::AtomicBool};
use std::str::FromStr;

type Responder<T> = oneshot::Sender<Result<T, Box<dyn std::error::Error + Send + Sync>>>;

#[derive(Debug)]
enum ASIAirCommand {
    Get {
        method: String,
        params: Option<Value>,
        tx: Responder<Value>,
    },
    Set {
        method: String,
        params: Option<Value>,
    },
}

#[derive(Debug, Clone, Default)]
pub enum ASIAirLanguage {
    #[default] English,
}

#[derive(Debug, Clone, Default)]
pub enum ASIAirPage {
    #[default] Preview,
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

impl FromStr for ASIAirPage {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "preview" => Ok(ASIAirPage::Preview),
            "focus" => Ok(ASIAirPage::Focus),
            "pa" => Ok(ASIAirPage::PA),
            "stack" => Ok(ASIAirPage::Stack),
            "autosave" => Ok(ASIAirPage::Autosave),
            "plan" => Ok(ASIAirPage::Plan),
            "rmtp" => Ok(ASIAirPage::RMTP),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum EventState {
    #[default] Start,
    Downloading,
    Complete
}

impl EventState {
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        match self {
            EventState::Start => "start",
            EventState::Downloading => "downloading",
            EventState::Complete => "complete",
        }
    }
}

impl FromStr for EventState {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "start" => Ok(EventState::Start),
            "downloading" => Ok(EventState::Downloading),
            "complete" => Ok(EventState::Complete),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ExposureChangeEvent {
    pub page: ASIAirPage,
    pub state: EventState,
    pub exp_us: u64,
    pub gain: u32,
}

#[derive(Debug, Clone, Default)]
pub struct PiStatusEvent {
    pub is_overtemp: bool,
    pub temp: f32,
    pub is_undervolt: bool,
    pub is_over_current: bool,
}

#[derive(Debug, Clone, Default)]
pub struct AnnotateEvent {
    pub page: ASIAirPage,
    pub tag: String,
    pub state: EventState,
}

#[derive(Debug, Clone, Default)]
pub struct PlateSolveEvent {
    pub page: ASIAirPage,
    pub tag: String,
    pub state: EventState,
}

#[derive(Debug, Clone)]
pub struct ASIAir {
    // The address of the ASIAir device
    addr: SocketAddr,
    // Time waiting for command response
    cmd_timeout: Duration,

    // Channel for async commmands to send to the ASIAir
    tx: Option<mpsc::Sender<ASIAirCommand>>,
    // Map of pending responses, keyed by request ID
    pending_responses: Arc<Mutex<HashMap<u32, Responder<Value>>>>,
    // Channel for shutdown signal
    shutdown_tx: Option<watch::Sender<()>>,
    // Channel for reconnection attempts
    reconnect_tx: Option<mpsc::Sender<()>>,
    // Tracks if we should attempt to connect
    should_be_connected: Arc<AtomicBool>,

    // Tracks if the ASIAir is connected
    pub connected: Arc<AtomicBool>,

    // Channels to publish events
    // Publicly accessible channel for connection state
    pub connection_state_tx: watch::Sender<bool>,
    pub camera_temperature_tx: watch::Sender<f32>,
    pub cooler_power_tx: watch::Sender<i32>,
    pub camera_control_change_tx: watch::Sender<()>,
    pub exposure_change_tx: watch::Sender<ExposureChangeEvent>,
    pub pi_status_tx: watch::Sender<PiStatusEvent>,
    pub annotate_tx: watch::Sender<AnnotateEvent>,
    pub plate_solve_tx: watch::Sender<PlateSolveEvent>,

}

