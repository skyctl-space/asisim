use std::net::SocketAddr;
use asiair::ASIAir;
use asisim::ASIAirSim;
use std::time::Duration;

#[tokio::test]
async fn test_asiair_basics() {
    let addr : SocketAddr = SocketAddr::from(([127, 0, 0, 1], 4720));

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
    let asiair_sim = ASIAirSim::new();
    // Start the ASIAir simulator in the background
    let simulator_clone = asiair_sim.clone();
    tokio::spawn(async move {
        simulator_clone.start().await.unwrap();
    });
   
    // Give the simulator some time to start
    tokio::time::sleep(Duration::from_secs(1)).await;
    
    // Connect to the ASIAir simulator
    asiair.connect().await.unwrap();

    // Test connection when connected
    let result = asiair.test_connection().await;
    assert!(result.is_ok(), "Expected successful connection test to ASIAir simulator");

    // Disconnect from the ASIAir simulator
    asiair.disconnect();

    // Test connection after disconnecting
    let result = asiair.test_connection().await;
    assert!(result.is_err(), "Expected error when testing connection to ASIAir simulator after disconnecting");

    // Test reconnecting to the ASIAir simulator
    asiair.connect().await.unwrap();

    // Test connection after reconnecting
    let result = asiair.test_connection().await;
    assert!(result.is_ok(), "Expected successful connection test to ASIAir simulator after reconnecting");
    
    // Disconnect from the ASIAir simulator
    asiair.disconnect();
}
    