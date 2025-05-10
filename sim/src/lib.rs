mod rtc;
mod rpc;
mod sim;

use std::sync::{Arc, Mutex};
use sim::ASIAirState;

#[derive(Debug, Clone)]
pub struct ASIAirSim {
    // ASIAir simulation state
    state: Arc<Mutex<ASIAirState>>,
}
