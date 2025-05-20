use asisim::ASIAirSim;
use env_logger;
use rand::Rng;
use serde_json::{json, Value};
use serial_test::serial;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::net::UdpSocket;
use tokio::time::timeout;

async fn setup_simulator() -> ASIAirSim {
    let mut asiair_sim = ASIAirSim::new();

    // Start the ASIAir simulator in the background
    asiair_sim.start().await.unwrap();

    // Give the simulator some time to start
    tokio::time::sleep(Duration::from_secs(1)).await;

    asiair_sim
}

async fn test_scan_air_request() {
    // Create a UDP socket to send a request
    let socket = UdpSocket::bind("0.0.0.0:0").await.unwrap();
    socket.connect("127.0.0.1:4720").await.unwrap();

    // Generate a random ID for the request
    let random_id: u64 = rand::rng().random_range(1..1000);

    // Send a scan_air request
    let request = json!({
        "id": random_id,
        "method": "scan_air",
        "name": "iphone",
    });
    socket.send(request.to_string().as_bytes()).await.unwrap();

    // Receive the response
    let mut buf = [0u8; 2048];
    let len = socket.recv(&mut buf).await.unwrap();
    let response: Value = serde_json::from_slice(&buf[..len]).unwrap();

    // Verify the response
    assert_eq!(response["id"], random_id);
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["method"], "scan_air");
    assert!(response["result"]["name"].is_string());
    assert!(response["result"]["ip"].is_string());
    assert!(response["result"]["ssid"].is_string());
    assert!(response["result"]["guid"].is_string());
    assert!(response["result"]["is_pi4"].is_boolean());
    assert!(response["result"]["model"].is_string());
    assert!(response["result"]["connect_lock"].is_boolean());
    assert_eq!(response["code"], 0);
}

async fn test_tcp_test_connection_request(stream: &mut TcpStream) {
    // Generate a random ID for the request
    let random_id: u64 = rand::rng().random_range(1..1000);

    // Send a test_connection request
    let request = json!({
        "id": random_id,
        "method": "test_connection",
        "params": null,
    });
    stream
        .write_all(request.to_string().as_bytes())
        .await
        .unwrap();

    // Receive the response
    let mut buf = [0u8; 2048];
    let len = timeout(Duration::from_secs(2), stream.read(&mut buf))
        .await
        .unwrap()
        .unwrap();
    let response: Value = serde_json::from_slice(&buf[..len]).unwrap();

    // Verify the response
    assert_eq!(response["id"], random_id);
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["method"], "test_connection");
    assert_eq!(response["result"], "server connected!");
    assert_eq!(response["code"], 0);
}

async fn test_pi_set_time_request(stream: &mut TcpStream) {
    // Generate a random ID for the request
    let random_id: u64 = rand::rng().random_range(1..1000);

    // Send a test_connection request
    let request = json!({
        "id": random_id,
        "method": "pi_set_time",
        "params": [{ "time_zone" : "America/Costa_Rica", "hour" : 18, "min" : 44, "sec" : 31, "day" : 6, "year" : 2025, "mon" : 5 } ]
    });
    stream
        .write_all(request.to_string().as_bytes())
        .await
        .unwrap();

    // Receive the response
    let mut buf = [0u8; 2048];
    let len = timeout(Duration::from_secs(2), stream.read(&mut buf))
        .await
        .unwrap()
        .unwrap();

    let response: Value = serde_json::from_slice(&buf[..len]).unwrap();

    // Verify the response
    assert_eq!(response["id"], random_id);
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["method"], "pi_set_time");
    assert_eq!(response["code"], 0);
}

async fn test_set_setting_request(stream: &mut TcpStream) {
    // Generate a random ID for the request
    let random_id: u64 = rand::rng().random_range(1..1000);

    // Send a test_connection request
    let request = json!({
        "id": random_id,
        "method": "set_setting",
        "params": "{ \"lang\" : \"en\" }",
    });
    stream
        .write_all(request.to_string().as_bytes())
        .await
        .unwrap();

    // Receive the response
    let mut buf = [0u8; 2048];
    let len = timeout(Duration::from_secs(2), stream.read(&mut buf))
        .await
        .unwrap()
        .unwrap();

    let response: Value = serde_json::from_slice(&buf[..len]).unwrap();

    // Verify the response
    assert_eq!(response["id"], random_id);
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["method"], "set_setting");
    assert_eq!(response["code"], 0);
}

async fn test_get_setting_request(stream: &mut TcpStream) {
    // Generate a random ID for the request
    let random_id: u64 = rand::rng().random_range(1..1000);

    // Send a get_setting request
    let request = json!({
        "id": random_id,
        "method": "get_setting",
        "params": {
            "lang": "en",
        },
    });
    stream
        .write_all(request.to_string().as_bytes())
        .await
        .unwrap();

    // Receive the response
    let mut buf = [0u8; 2048];
    let len = timeout(Duration::from_secs(2), stream.read(&mut buf))
        .await
        .unwrap()
        .unwrap();

    let response: Value = serde_json::from_slice(&buf[..len]).unwrap();

    // Verify the response
    assert_eq!(response["id"], random_id);
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["method"], "get_setting");
    assert!(response["result"].is_object());
    assert_eq!(response["code"], 0);
}

async fn test_get_app_setting_request(stream: &mut TcpStream) {
    // Generate a random ID for the request
    let random_id: u64 = rand::rng().random_range(1..1000);

    // Send a get_app_setting request
    let request = json!({
        "id": random_id,
        "method": "get_app_setting",
        "params": null,
    });
    stream
        .write_all(request.to_string().as_bytes())
        .await
        .unwrap();

    // Receive the response
    let mut buf = [0u8; 2048];
    let len = timeout(Duration::from_secs(2), stream.read(&mut buf))
        .await
        .unwrap()
        .unwrap();

    let response: Value = serde_json::from_slice(&buf[..len]).unwrap();

    // Verify the response
    assert_eq!(response["id"], random_id);
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["method"], "get_app_setting");
    assert!(response["result"].is_object());
    assert_eq!(response["code"], 0);
}

async fn test_get_app_state_request(stream: &mut TcpStream) {
    // Generate a random ID for the request
    let random_id: u64 = rand::rng().random_range(1..1000);

    // Send a get_app_state request
    let request = json!({
        "id": random_id,
        "method": "get_app_state",
        "params": null,
    });
    stream
        .write_all(request.to_string().as_bytes())
        .await
        .unwrap();

    // Receive the response
    let mut buf = [0u8; 2048];
    let len = timeout(Duration::from_secs(2), stream.read(&mut buf))
        .await
        .unwrap()
        .unwrap();

    let response: Value = serde_json::from_slice(&buf[..len]).unwrap();

    // Verify the response
    assert_eq!(response["id"], random_id);
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["method"], "get_app_state");
    assert!(response["result"].is_object());
    assert_eq!(response["code"], 0);
}

async fn test_get_connected_cameras_request(stream: &mut TcpStream) {
    // Generate a random ID for the request
    let random_id: u64 = rand::rng().random_range(1..1000);

    // Send a get_connected_cameras request
    let request = json!({
        "id": random_id,
        "method": "get_connected_cameras",
        "params": null,
    });
    stream
        .write_all(request.to_string().as_bytes())
        .await
        .unwrap();

    // Receive the response
    let mut buf = [0u8; 2048];
    let len = timeout(Duration::from_secs(2), stream.read(&mut buf))
        .await
        .unwrap()
        .unwrap();

    let response: Value = serde_json::from_slice(&buf[..len]).unwrap();

    // Verify the response
    assert_eq!(response["id"], random_id);
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["method"], "get_connected_cameras");
    assert!(response["result"].is_array());
    assert_eq!(response["code"], 0);
}

async fn test_get_camera_state_request(stream: &mut TcpStream, should_be_opened : bool) {
    // Generate a random ID for the request
    let random_id: u64 = rand::rng().random_range(1..1000);

    // Send a get_camera_state request
    let request = json!({
        "id": random_id,
        "method": "get_camera_state",
        "params": null,
    });
    stream
        .write_all(request.to_string().as_bytes())
        .await
        .unwrap();

    // Receive the response
    let mut buf = [0u8; 2048];
    let len = timeout(Duration::from_secs(2), stream.read(&mut buf))
        .await
        .unwrap()
        .unwrap();

    let response: Value = serde_json::from_slice(&buf[..len]).unwrap();

    // Verify the response
    assert_eq!(response["id"], random_id);
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["method"], "get_camera_state");
    assert!(response["result"].is_object());
    if should_be_opened {
        assert_eq!(response["result"]["state"], "idle");
    } else {
        assert_eq!(response["result"]["state"], "close");
    }
    assert_eq!(response["code"], 0);
}

async fn test_open_camera_request(stream: &mut TcpStream) {
    // Generate a random ID for the request
    let random_id: u64 = rand::rng().random_range(1..1000);

    // Send a open_camera request
    let request = json!({
        "id": random_id,
        "method": "open_camera",
        "params": null,
    });
    stream
        .write_all(request.to_string().as_bytes())
        .await
        .unwrap();

    // Receive the response
    let mut buf = [0u8; 2048];
    let len = timeout(Duration::from_secs(2), stream.read(&mut buf))
        .await
        .unwrap()
        .unwrap();

    let response: Value = serde_json::from_slice(&buf[..len]).unwrap();

    // Verify the response
    assert_eq!(response["id"], random_id);
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["method"], "open_camera");
    assert_eq!(response["code"], 0);
}

async fn test_close_camera_request(stream: &mut TcpStream) {
    // Generate a random ID for the request
    let random_id: u64 = rand::rng().random_range(1..1000);

    // Send a close_camera request
    let request = json!({
        "id": random_id,
        "method": "close_camera",
        "params": null,
    });
    stream
        .write_all(request.to_string().as_bytes())
        .await
        .unwrap();

    // Receive the response
    let mut buf = [0u8; 2048];
    let len = timeout(Duration::from_secs(2), stream.read(&mut buf))
        .await
        .unwrap()
        .unwrap();

    let response: Value = serde_json::from_slice(&buf[..len]).unwrap();

    // Verify the response
    assert_eq!(response["id"], random_id);
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["method"], "close_camera");
    assert_eq!(response["code"], 0);
}

async fn test_get_camera_info_request(stream: &mut TcpStream) {
    // Generate a random ID for the request
    let random_id: u64 = rand::rng().random_range(1..1000);

    // Send a get_camera_info request
    let request = json!({
        "id": random_id,
        "method": "get_camera_info",
        "params": null,
    });
    stream
        .write_all(request.to_string().as_bytes())
        .await
        .unwrap();

    // Receive the response
    let mut buf = [0u8; 2048];
    let len = timeout(Duration::from_secs(2), stream.read(&mut buf))
        .await
        .unwrap()
        .unwrap();

    let response: Value = serde_json::from_slice(&buf[..len]).unwrap();

    // Verify the response
    assert_eq!(response["id"], random_id);
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["method"], "get_camera_info");
    assert!(response["result"].is_object());
    assert_eq!(response["code"], 0);
}


#[tokio::test]
#[serial]
async fn test_asiair_protocol() {
    env_logger::init();
    let _simulator = setup_simulator().await;

    // Connect to the TCP server
    let mut stream = TcpStream::connect("127.0.0.1:4700").await.unwrap();

    test_scan_air_request().await;
    test_tcp_test_connection_request(&mut stream).await;
    test_pi_set_time_request(&mut stream).await;
    test_set_setting_request(&mut stream).await;
    test_get_setting_request(&mut stream).await;
    test_get_app_setting_request(&mut stream).await;
    test_get_app_state_request(&mut stream).await;
    test_get_connected_cameras_request(&mut stream).await;
    test_open_camera_request(&mut stream).await;
    test_get_camera_state_request(&mut stream, true).await;
    test_close_camera_request(&mut stream).await;
    test_get_camera_state_request(&mut stream, false).await;
    test_get_camera_info_request(&mut stream).await;
}
