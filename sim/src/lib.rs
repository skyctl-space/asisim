mod rpc;
mod rtc;
mod sim;

use sim::ASIAirState;
use std::sync::{Arc, Mutex};
use tokio::sync::watch;

#[derive(Debug, Clone)]
pub struct ASIAirSim {
    // ASIAir simulation state
    state: Arc<Mutex<ASIAirState>>,
    // Channel for shutdown signal
    shutdown_tx: Option<watch::Sender<()>>,
}
