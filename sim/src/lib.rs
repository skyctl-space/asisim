mod rtc;
mod rpc;
mod sim;

use std::sync::{Arc, Mutex};
use tokio::sync::watch;
use sim::ASIAirState;

#[derive(Debug, Clone)]
pub struct ASIAirSim {
    // ASIAir simulation state
    state: Arc<Mutex<ASIAirState>>,
     // Channel for shutdown signal
     shutdown_tx: Option<watch::Sender<()>>,
}
