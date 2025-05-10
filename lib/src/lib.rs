mod connection;

use tokio::sync::{mpsc, watch};
use serde_json::Value;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, atomic::AtomicBool};

use connection::ASIAirCommand;
use connection::Responder;

#[derive(Debug, Clone)]
pub struct ASIAir {
    // The address of the ASIAir device
    addr: SocketAddr,

    // Channel for async commmands to send to the ASIAir
    tx: Option<mpsc::Sender<ASIAirCommand>>,
    // Map of pending responses, keyed by request ID
    pending_responses: Arc<Mutex<HashMap<u32, Responder<Value>>>>,
    // Channel for shutdown signal
    shutdown_tx: Option<watch::Sender<()>>,
    // Channel for reconnection attempts
    reconnect_tx: Option<mpsc::Sender<()>>,

    should_be_connected: Arc<AtomicBool>,

    // Tracks if the ASIAir is connected
    pub connected: Arc<AtomicBool>,
    // Publicly accessible channel for connection state
    pub connection_state_tx: watch::Sender<bool>,
}