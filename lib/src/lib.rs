mod connection;
mod settings;
pub mod camera;

use byteorder::{BigEndian, ByteOrder};
use serde_json::Value;
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::str::FromStr;
use std::sync::{Arc, Mutex, atomic::AtomicBool};
use tokio::sync::{mpsc, oneshot, watch};
use tokio::time::Duration;

type Responder<T> = oneshot::Sender<Result<T, Box<dyn std::error::Error + Send + Sync>>>;

#[derive(Debug, Clone)]
pub struct BinaryResult {
    pub data: Vec<u8>,
    pub width: u16,
    pub height: u16,
}

#[derive(Debug)]
enum ASIAirCommand {
    BinaryGet {
        method: String,
        params: Option<Value>,
        tx: Responder<BinaryResult>,
    },
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
    #[default]
    English,
}

#[derive(Debug, Clone, Default)]
pub enum ASIAirPage {
    #[default]
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
    #[default]
    Start,
    Downloading,
    Complete,
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

#[allow(dead_code)]
#[derive(Debug)]
struct BinaryHeader {
    magic0: u32,           // 4 bytes
    magic1: u16,           // 2 bytes
    pub payload_size: u32, // 4 bytes
    unknown1: [u8; 5],     // 5 bytes
    pub id: u8,            // 1 byte
    pub width: u16,        // 2 bytes
    pub height: u16,       // 2 bytes
    unknown2: u16,         // 2 bytes
    unknown3: u32,      // 4 bytes
    unknown4: u16,         // 2 bytes
    pub bin: u16,              // 2 bytes
    unknown5: u16,         // 2 bytes
    padding: [u32; 12],    // 48 bytes (12 * 4)
}

impl BinaryHeader {
    fn parse(buf: &[u8; 80]) -> Self {
        let magic0 = BigEndian::read_u32(&buf[0..4]);
        let magic1 = BigEndian::read_u16(&buf[4..6]);
        let payload_size = BigEndian::read_u32(&buf[6..10]);

        let unknown1 = [buf[10], buf[11], buf[12], buf[13], buf[14]];
        let id = buf[15];
        let width = BigEndian::read_u16(&buf[16..18]);
        let height = BigEndian::read_u16(&buf[18..20]);

        let unknown2 = BigEndian::read_u16(&buf[20..22]);
        let unknown3 = BigEndian::read_u32(&buf[22..26]);
        let unknown4 = BigEndian::read_u16(&buf[26..28]);
        let bin = BigEndian::read_u16(&buf[28..30]);
        let unknown5 = BigEndian::read_u16(&buf[30..34]);

        BinaryHeader {
            magic0,
            magic1,
            payload_size,
            unknown1,
            id,
            width,
            height,
            unknown2,
            unknown3,
            unknown4,
            bin,
            unknown5,
            padding: [0u32; 12],
        }
    }
}


#[derive(Debug, Clone)]
pub struct ASIAir {
    // The address of the ASIAir device
    pub addr: Ipv4Addr,
    // Time waiting for command response
    cmd_timeout: Duration,

    // Channel for async commmands to send to the ASIAir
    tx_4500: Option<mpsc::Sender<ASIAirCommand>>,
    tx_4700: Option<mpsc::Sender<ASIAirCommand>>,
    tx_4800: Option<mpsc::Sender<ASIAirCommand>>,

    // Map of pending responses, keyed by request ID
    pending_responses: Arc<Mutex<HashMap<u32, Responder<Value>>>>,
    // Map of pending responses, keyed by request ID
    pending_responses_4500: Arc<Mutex<HashMap<u32, Responder<Value>>>>,
    // Map of pending responses, keyed by request ID
    pending_responses_4800: Arc<Mutex<HashMap<u32, Responder<BinaryResult>>>>,
    // Channel for shutdown signal
    shutdown_tx: Option<watch::Sender<()>>,
    // Channel for reconnection attempts
    reconnect_tx: Option<mpsc::Sender<()>>,
    // Tracks if we should attempt to connect
    pub should_be_connected: Arc<AtomicBool>,

    // Tracks if the ASIAir is connected
    pub connected: Arc<AtomicBool>,

    // Channels to publish events
    // Publicly accessible channel for connection state
    pub connection_state_tx: watch::Sender<bool>,
    pub camera_temperature_tx: watch::Sender<f32>,
    pub camera_state_change_tx: watch::Sender<()>,
    pub cooler_power_tx: watch::Sender<i32>,
    pub camera_control_change_tx: watch::Sender<()>,
    pub exposure_change_tx: watch::Sender<ExposureChangeEvent>,
    pub pi_status_tx: watch::Sender<PiStatusEvent>,
    pub annotate_tx: watch::Sender<AnnotateEvent>,
    pub plate_solve_tx: watch::Sender<PlateSolveEvent>,
}
