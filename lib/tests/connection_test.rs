
mod common;

#[cfg(test)]
mod tests {
    use super::common::init_logger;

    use std::net::Ipv4Addr;
    use asiair::ASIAir;
    use asisim::ASIAirSim;
    use std::time::Duration;
    use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
    use tokio::sync::{Mutex, Barrier};


    #[tokio::test]
    async fn test_asiair_connection() {
        init_logger();
        let addr : Ipv4Addr = Ipv4Addr::from([127, 0, 0, 1]));

        // Create a new ASIAir instance
        let mut asiair = ASIAir::new(addr);

        // Attempt to connect to the ASIAir simulator when is not running
        // This should fail with a connection error
        let result = asiair.connect().await;
        assert!(result.is_err(), "Expected error when connecting to ASIAir simulator that is not running");

        // Test connection when not connected
        let result = asiair.test_connection().await;
        assert!(result.is_err(), "Expected error when testing connection to ASIAir simulator that is not running");

        // Create a new ASIAir simulator instance
        let mut asiair_sim = ASIAirSim::new();
        // Start the ASIAir simulator
        asiair_sim.start().await.unwrap();
    
        // Give the simulator some time to start
        tokio::time::sleep(Duration::from_secs(1)).await;

        // Let's define an atomic boolean to track the connection state for verification
        let should_be_connected: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
        let assertion_failed = Arc::new(AtomicBool::new(false));
        let barrier_holder: Arc<Mutex<Option<Arc<Barrier>>>> = Arc::new(Mutex::new(Some(Arc::new(Barrier::new(2)))));

        let mut conn_rx = asiair.subscribe_connection_state();
        let should_be_connected_clone = should_be_connected.clone();
        let assertion_failed_clone = assertion_failed.clone();
        let barrier_holder_clone = barrier_holder.clone();

        tokio::spawn(async move {
            while conn_rx.changed().await.is_ok() {
                let connected = *conn_rx.borrow();
                let expected = should_be_connected_clone.load(Ordering::SeqCst);
                log::debug!("[watcher] connection state changed: {} (expected {})", connected, expected);
                if connected != expected {
                    log::debug!("Connection state mismatch: got {}, expected {}", connected, expected);
                    assertion_failed_clone.store(true, Ordering::SeqCst);
                }
                if let Some(barrier) = {
                    let guard = barrier_holder_clone.lock().await;
                    guard.clone()
                } {
                    log::debug!("[watcher] waiting for barrier...");
                    barrier.wait().await;
                }
                log::debug!("[watcher] waiting for next update...");
            }
        });

        // Helper to set up a new barrier and wait
        async fn wait_for_update(barrier_holder: &Arc<Mutex<Option<Arc<Barrier>>>>) {
            // Take the current barrier and wait on it
            let current_barrier = {
                let guard = barrier_holder.lock().await;
                guard.clone()
            };
        
            if let Some(barrier) = current_barrier {
                log::debug!("[main] waiting for barrier...");
                barrier.wait().await;
            }
        
            // Install a new barrier for the next phase
            let new_barrier = Arc::new(Barrier::new(2));
            let mut guard = barrier_holder.lock().await;
            *guard = Some(new_barrier);
        }
        
        // Connect to the ASIAir simulator
        should_be_connected.store(true, Ordering::SeqCst);
        asiair.connect().await.unwrap();


        // Wait for the subscriber watcher to receive the signal
        wait_for_update(&barrier_holder).await;
        assert!(
            !assertion_failed.load(Ordering::SeqCst),
            "Connection state did not match expected value"
        );

        // Test connection when connected
        let result = asiair.test_connection().await;
        log::debug!("Test connection result: {:?}", result);
        assert!(result.is_ok(), "Expected successful connection test to ASIAir simulator");

        // Disconnect from the ASIAir simulator
        should_be_connected.store(false, Ordering::SeqCst);
        asiair.disconnect().await;

        // Wait for the subscriber watcher to receive the signal
        wait_for_update(&barrier_holder).await;
        assert!(
            !assertion_failed.load(Ordering::SeqCst),
            "Connection state did not match expected value"
        );

        // Test connection after disconnecting
        let result = asiair.test_connection().await;
        assert!(result.is_err(), "Expected error when testing connection to ASIAir simulator after disconnecting");

        // Test reconnecting to the ASIAir simulator
        should_be_connected.store(true, Ordering::SeqCst);
        asiair.connect().await.unwrap();

        // Wait for the subscriber watcher to receive the signal
        wait_for_update(&barrier_holder).await;
        assert!(
            !assertion_failed.load(Ordering::SeqCst),
            "Connection state did not match expected value"
        );

        // Test connection after reconnecting
        let result = asiair.test_connection().await;
        assert!(result.is_ok(), "Expected successful connection test to ASIAir simulator after reconnecting");

        // Kill the ASIAir simulator and wait for the watchdog to detect the disconnection
        should_be_connected.store(false, Ordering::SeqCst);

        asiair_sim.shutdown();

        // Wait for watcher to finish
        wait_for_update(&barrier_holder).await;
        assert!(
            !assertion_failed.load(Ordering::SeqCst),
            "Background connection state watcher detected a mismatch"
        );

        // Restart the ASIAir simulator and wait for the connection to be restored
        should_be_connected.store(true, Ordering::SeqCst);
        asiair_sim.start().await.unwrap();

        // Wait for watcher to finish
        wait_for_update(&barrier_holder).await;
        assert!(
            !assertion_failed.load(Ordering::SeqCst),
            "Background connection state watcher detected a mismatch"
        );

        asiair.disconnect().await;

    }
    
}