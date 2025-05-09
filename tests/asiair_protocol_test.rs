use tokio::net::UdpSocket;
use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use tokio::io::AsyncReadExt;
use serde_json::{json, Value};
use std::time::Duration;
use tokio::time::timeout;
use rand::Rng; // Import random number generator
use asisim::asiair::ASIAirSim;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_udp_scan_air() {
    let asiair_sim = ASIAirSim::new();

    // Start the ASIAir simulator in the background
    tokio::spawn(async move {
        asiair_sim.start().await.unwrap();
    });

    // Give the simulator some time to start
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Create a UDP socket to send a request
    let socket = UdpSocket::bind("0.0.0.0:0").await.unwrap();
    socket.connect("127.0.0.1:4720").await.unwrap();

    // Generate a random ID for the request
    let random_id: u64 = rand::rng().random_range(1..1000);

    // Send a scan_air request
    let request = json!({
        "id": random_id,
        "method": "scan_air",
        "params": null,
        "jsonrpc": "2.0"
    });
    socket.send(request.to_string().as_bytes()).await.unwrap();

    // Receive the response
    let mut buf = [0u8; 2048];
    let len = socket.recv(&mut buf).await.unwrap();
    let response: Value = serde_json::from_slice(&buf[..len]).unwrap();

    // Verify the response
    assert_eq!(response["id"], random_id);
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response["result"]["name"].is_string());
    assert!(response["result"]["ip"].is_string());
    assert!(response["result"]["ssid"].is_string());
    assert!(response["result"]["guid"].is_string());
    assert!(response["result"]["is_pi4"].is_boolean());
    assert!(response["result"]["model"].is_string());
    assert!(response["result"]["connect_lock"].is_boolean());
    assert_eq!(response["code"], 0);
}

#[tokio::test]
#[serial]
async fn test_tcp_test_connection() {
    let asiair_sim = ASIAirSim::new();

    // Start the ASIAir simulator in the background
    tokio::spawn(async move {
        asiair_sim.start().await.unwrap();
    });

    // Give the simulator some time to start
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Connect to the TCP server
    let mut stream = TcpStream::connect("127.0.0.1:4720").await.unwrap();

    // Generate a random ID for the request
    let random_id: u64 = rand::rng().random_range(1..1000);

    // Send a test_connection request
    let request = json!({
        "id": random_id,
        "method": "test_connection",
        "params": null,
        "jsonrpc": "2.0"
    });
    stream.write_all(request.to_string().as_bytes()).await.unwrap();

    // Receive the response
    let mut buf = [0u8; 2048];
    let len = timeout(Duration::from_secs(2), stream.read(&mut buf)).await.unwrap().unwrap();
    let response: Value = serde_json::from_slice(&buf[..len]).unwrap();

    // Verify the response
    assert_eq!(response["id"], random_id);
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["result"], "server connected!");
    assert_eq!(response["code"], 0);
}