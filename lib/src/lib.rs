mod connection;

use tokio::sync::{mpsc, watch};
use serde_json::Value;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use connection::ASIAirCommand;
use connection::Responder;

#[derive(Debug)]
pub struct ASIAir {
    // The address of the ASIAir device
    addr: SocketAddr,

    // Channel for async commmands to send to the ASIAir
    tx: Option<mpsc::Sender<ASIAirCommand>>,
    // Map of pending responses, keyed by request ID
    pending_responses: Arc<Mutex<HashMap<u32, Responder<Value>>>>,
    // Channel for shutdown signal
    shutdown_tx: Option<watch::Sender<()>>,
}