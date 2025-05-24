use serde_json::{Value, json};
use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, oneshot, watch};
use tokio::time::Duration;

use super::ASIAir;
use super::ASIAirCommand;
use super::ASIAirPage;
use super::AnnotateEvent;
use super::BinaryHeader;
use super::BinaryResult;
use super::ExposureEvent;
use super::PiStatusEvent;
use super::PlateSolveEvent;

impl ASIAir {
    pub fn new(addr: Ipv4Addr) -> Self {
        let (connection_state_tx, _) = watch::channel(false);
        let (camera_temperature_tx, _) = watch::channel(0.0);
        let (cooler_power_tx, _) = watch::channel(0);
        let (camera_control_change_tx, _) = watch::channel(());
        let (camera_state_change_tx, _) = watch::channel(());
        let (exposure_tx, _) = watch::channel(ExposureEvent::default());
        let (pi_status_tx, _) = watch::channel(PiStatusEvent::default());
        let (annotate_tx, _) = watch::channel(AnnotateEvent::default());
        let (plate_solve_tx, _) = watch::channel(PlateSolveEvent::default());

        ASIAir {
            addr,
            cmd_timeout: Duration::from_secs(5),
            binary_cmd_timeout: Duration::from_secs(120),
            tx_4500: None,
            tx_4700: None,
            tx_4800: None,
            pending_responses: Arc::new(Mutex::new(HashMap::new())),
            pending_responses_4500: Arc::new(Mutex::new(HashMap::new())),
            pending_responses_4800: Arc::new(Mutex::new(HashMap::new())),
            shutdown_tx: None,
            reconnect_tx: None,
            should_be_connected: Arc::new(AtomicBool::new(false)),
            connected: Arc::new(AtomicBool::new(false)),
            connection_state_tx,
            camera_temperature_tx,
            camera_state_change_tx,
            cooler_power_tx,
            camera_control_change_tx,
            exposure_tx,
            pi_status_tx,
            annotate_tx,
            plate_solve_tx,
        }
    }

    /// Connect to the ASIAir device
    pub async fn connect(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.should_be_connected.load(Ordering::SeqCst) {
            // Even if we are not actually connected, there would be a reconnection attempt
            // in the background, so we can just return
            return Ok(());
        }

        log::info!("Connecting to ASIAir at {}", self.addr);
        let (reconnect_tx, mut reconnect_rx) = mpsc::channel::<()>(1);
        self.reconnect_tx = Some(reconnect_tx);

        self.try_connect().await?;

        // Start a background task to handle reconnections
        let mut this = self.clone();
        tokio::spawn(async move {
            while reconnect_rx.recv().await.is_some()
                && this.should_be_connected.load(Ordering::SeqCst)
            {
                this.reconnect().await.unwrap_or_else(|e| {
                    log::info!("Reconnection failed: {}", e);
                });
            }
        });

        self.should_be_connected.store(true, Ordering::SeqCst);
        Ok(())
    }

    /// Disconnect from the ASIAir device
    pub async fn disconnect(&mut self) {
        if self.should_be_connected.load(Ordering::SeqCst) {
            self.should_be_connected.store(false, Ordering::SeqCst);
            self.cleanup().await;
            log::info!("Disconnected from ASIAir");
        }
    }

    async fn try_connect(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let socket = SocketAddrV4::new(self.addr.clone(), 4700);
        let stream = TcpStream::connect(socket).await?;
        let (mut reader, mut writer) = tokio::io::split(stream);

        let (tx, mut rx) = mpsc::channel::<ASIAirCommand>(32);
        self.tx_4700 = Some(tx.clone());

        let (shutdown_tx, shutdown_rx) = watch::channel(());
        self.shutdown_tx = Some(shutdown_tx);

        let pending_responses_writer = Arc::clone(&self.pending_responses);
        let pending_responses_reader = Arc::clone(&self.pending_responses);

        let mut shutdown_reader_rx = shutdown_rx.clone();
        let mut shutdown_writer_rx = shutdown_rx.clone();
        let mut shutdown_watchdog_rx = shutdown_rx.clone();
        let reconnect_tx = self.reconnect_tx.clone().unwrap();
        let reconnect_tx_reader = self.reconnect_tx.clone().unwrap();
        let should_be_connected = self.should_be_connected.clone();

        self.connected.store(true, Ordering::SeqCst);
        let _ = self.connection_state_tx.send(true); // Notify that we are connected

        let camera_temperature_tx = self.camera_temperature_tx.clone();
        let cooler_power_tx = self.cooler_power_tx.clone();
        let camera_control_change_tx = self.camera_control_change_tx.clone();
        let camera_state_change_tx = self.camera_state_change_tx.clone();
        let exposure_tx = self.exposure_tx.clone();
        let pi_status_tx = self.pi_status_tx.clone();
        let annotate_tx = self.annotate_tx.clone();
        let plate_solve_tx = self.plate_solve_tx.clone();

        let socket_4800 = SocketAddrV4::new(self.addr.clone(), 4800);
        let stream_4800 = TcpStream::connect(socket_4800).await?;
        let (mut reader_4800, mut writer_4800) = tokio::io::split(stream_4800);
        let mut shutdown_reader_rx_4800 = shutdown_rx.clone();
        let mut shutdown_writer_rx_4800 = shutdown_rx.clone();
        let reconnect_tx_reader_4800 = self.reconnect_tx.clone().unwrap();
        let reconnect_tx_reader_4800_watchdog = self.reconnect_tx.clone().unwrap();
        let should_be_connected_4800 = self.should_be_connected.clone();

        // Create a new pending responses map for port 4800
        let pending_responses_writer_4800 = Arc::clone(&self.pending_responses_4800);
        let pending_responses_reader_4800 = Arc::clone(&self.pending_responses_4800);

        let (tx_4800, mut rx_4800) = mpsc::channel::<ASIAirCommand>(32);
        self.tx_4800 = Some(tx_4800.clone());

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown_watchdog_rx.changed() => {
                        log::debug!("Watchdog task received shutdown");
                        break;
                    }
                    _ = tokio::time::sleep(Duration::from_secs(2)) => {
                        let (response_tx, response_rx) = oneshot::channel();
                        let command = ASIAirCommand::Get {
                            method: "test_connection".to_string(),
                            params: None,
                            tx: response_tx,
                        };
                        // log::debug!("Testing ASIAIR connection watchdog...");
                        if tx.send(command).await.is_ok() {
                            // Wait for the response with a timeout
                            match tokio::time::timeout(Duration::from_secs(2), response_rx).await {
                                Ok(Ok(_response)) => {
                                    // log::debug!("Watchdog response received: {:?}", response);
                                }
                                Ok(Err(_)) | Err(_) => {
                                    log::warn!("Connection to ASIAir lost or timed out");
                                    let _ = reconnect_tx.send(()).await;
                                }
                            }
                        }

                        let (response_tx, response_rx) = oneshot::channel();
                        let command = ASIAirCommand::BinaryGet {
                            method: "test_connection".to_string(),
                            params: None,
                            tx: response_tx,
                        };
                        if tx_4800.send(command).await.is_ok() {
                            // Wait for the response with a timeout
                            match tokio::time::timeout(Duration::from_secs(60), response_rx).await {
                                Ok(Ok(_response)) => {
                                    // log::debug!("Watchdog response received: {:?}", response);
                                }
                                Ok(Err(_)) | Err(_) => {
                                    log::warn!("Connection to ASIAir lost or timed out");
                                    let _ = reconnect_tx_reader_4800_watchdog.send(()).await;
                                }
                            }
                        }
                    }
                }
            }
        });

        // Read loop
        tokio::spawn(async move {
            let mut buffer = Vec::new();

            let mut buf = [0u8; 2048];
            loop {
                tokio::select! {
                    read_result = reader.read(&mut buf) => {
                        match read_result {
                            Ok(0) => { // EOF
                                if should_be_connected.load(Ordering::SeqCst) {
                                    // If we are still supposed to be connected, and the socket
                                    // closed, trigger the reconnection loops
                                    let _ = reconnect_tx_reader.send(()).await;
                                }
                                break
                            },
                            Ok(len) => {
                                buffer.extend_from_slice(&buf[..len]);

                                // Use a sliding window to find complete frames
                                // ASIAir frames are terminated by \r\n
                                while let Some(pos) = buffer.windows(2).position(|window| window == b"\r\n") {
                                    let frame = buffer.drain(..pos + 2).collect::<Vec<_>>();
                                    if let Ok(response) = serde_json::from_slice::<Value>(&frame) {
                                        // Process the response as before
                                        if let Some(event) = response.get("Event") {
                                            match event.as_str() {
                                                Some("Temperature") => {
                                                    if let Some(temp) = response.get("value").and_then(|r| r.as_f64()) {
                                                        let _ = camera_temperature_tx.send(temp as f32);
                                                    }
                                                },
                                                Some("CoolerPower") => {
                                                    if let Some(power) = response.get("value").and_then(|r| r.as_i64()) {
                                                        let _ = cooler_power_tx.send(power as i32);
                                                    }
                                                },
                                                Some("CameraControlChange") => {
                                                    let _ = camera_control_change_tx.send(());
                                                },
                                                Some("CameraStateChange") => {
                                                    let _ = camera_state_change_tx.send(());
                                                },
                                                Some("Exposure") => {
                                                    match response.get("state").and_then(|r| r.as_str()) {
                                                        Some("start") => {
                                                            if let Some(exp_us) = response.get("exp_us").and_then(|r| r.as_u64()) {
                                                                if let Some(gain) = response.get("gain").and_then(|r| r.as_u64()) {
                                                                    if let Some(page) = response.get("page").and_then(|r| r.as_str()) {
                                                                        if let Ok(page) = ASIAirPage::from_str(page) {
                                                                            let _ = exposure_tx.send(ExposureEvent::Start {
                                                                                page: page,
                                                                                exp_us: exp_us,
                                                                                gain: gain as u64,
                                                                            });
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        },
                                                        Some("complete") => {
                                                            let _ = exposure_tx.send(ExposureEvent::Complete);
                                                        },
                                                        Some("downloading") => {
                                                            let _ = exposure_tx.send(ExposureEvent::Downloading);
                                                        }
                                                        _ => {}
                                                    }
                                                },
                                                Some("PiStatus") => {
                                                    if let Some(is_overtemp) = response.get("is_overtemp").and_then(|r| r.as_bool()) {
                                                        if let Some(temp) = response.get("temp").and_then(|r| r.as_f64()) {
                                                            if let Some(is_undervolt) = response.get("is_undervolt").and_then(|r| r.as_bool()) {
                                                                if let Some(is_over_current) = response.get("is_over_current").and_then(|r| r.as_bool()) {
                                                                    let _ = pi_status_tx.send(PiStatusEvent {
                                                                        is_overtemp: is_overtemp,
                                                                        temp: temp as f32,
                                                                        is_undervolt: is_undervolt,
                                                                        is_over_current: is_over_current,
                                                                    });
                                                                }
                                                            }
                                                        }
                                                    }
                                                },
                                                Some("Annotate") => {
                                                    if let Some(page) = response.get("page").and_then(|r| r.as_str()) {
                                                        if let Some(tag) = response.get("tag").and_then(|r| r.as_str()) {
                                                            if let Some(state) = response.get("state").and_then(|r| r.as_str()) {
                                                                if let Ok(page) = ASIAirPage::from_str(page) {
                                                                    let _ = annotate_tx.send(AnnotateEvent {
                                                                        page: page,
                                                                        tag: tag.to_string(),
                                                                        state: state.to_string(),
                                                                    });
                                                                }
                                                            }
                                                        }
                                                    }
                                                },
                                                Some("PlateSolve") => {
                                                    if let Some(page) = response.get("page").and_then(|r| r.as_str()) {
                                                        if let Some(tag) = response.get("tag").and_then(|r| r.as_str()) {
                                                            if let Some(state) = response.get("state").and_then(|r| r.as_str()) {
                                                                if let Ok(page) = ASIAirPage::from_str(page) {
                                                                    let _ = plate_solve_tx.send(PlateSolveEvent {
                                                                        page: page,
                                                                        tag: tag.to_string(),
                                                                        state: state.to_string(),
                                                                    });
                                                                }
                                                            }
                                                        }
                                                    }
                                                },
                                                _ => {}
                                            }
                                        } else if response.get("jsonrpc").is_some() {
                                            if let Some(id) = response.get("id").and_then(|id| id.as_u64()) {
                                                if let Some(tx) = pending_responses_reader
                                                    .lock()
                                                    .unwrap()
                                                    .remove(&(id as u32))
                                                {
                                                    let _ = tx.send(Ok(response["result"].clone()));
                                                } else {
                                                    log::warn!("No pending response for ID {}: {:?}", id, response);
                                                }
                                            }
                                        } else {
                                            log::warn!("Unexpected response: {:?}", response);
                                        }
                                    } else {
                                        log::warn!("Failed to parse JSON from frame: {:?}", frame);
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Read error: {:?}", e);
                                break;
                            }
                        }
                    }
                    _ = shutdown_reader_rx.changed() => {
                        log::debug!("Reader task received shutdown");
                        break;
                    }
                }
            }
        });

        // Write loop
        tokio::spawn(async move {
            let id_counter = AtomicU32::new(1);

            loop {
                tokio::select! {
                    Some(command) = rx.recv() => {
                        match command {
                            ASIAirCommand::Get { method, params, tx } => {
                                let id = id_counter.fetch_add(1, Ordering::SeqCst);
                                let request = if let Some(params) = params {
                                    json!({
                                        "id": id,
                                        "method": method,
                                        "params": params,
                                    })
                                } else {
                                    json!({
                                        "id": id,
                                        "method": method,
                                    })
                                };
                                pending_responses_writer.lock().unwrap().insert(id, tx);
                                let mut message = request.to_string();
                                message.push_str("\r\n");
                                if let Err(e) = writer.write_all(message.as_bytes()).await {
                                    eprintln!("Write error: {:?}", e);
                                }
                            }
                            ASIAirCommand::Set { method, params } => {
                                let id = id_counter.fetch_add(1, Ordering::SeqCst);
                                let request = if let Some(params) = params {
                                    json!({
                                        "id": id,
                                        "method": method,
                                        "params": params,
                                    })
                                } else {
                                    json!({
                                        "id": id,
                                        "method": method,
                                    })
                                };
                                let mut message = request.to_string();
                                message.push_str("\r\n");
                                if let Err(e) = writer.write_all(message.as_bytes()).await {
                                    eprintln!("Write error: {:?}", e);
                                }
                            }
                            _ => {
                                log::warn!("Unexpected command: {:?}", command);
                            }
                        }
                    }
                    _ = shutdown_writer_rx.changed() => {
                        log::debug!("Writer task received shutdown");
                        break;
                    }
                }
            }
        });

        // Read loop for port 4800
        tokio::spawn(async move {
            let mut hdr_buf = [0u8; 80];
            loop {
                tokio::select! {
                    read_result = reader_4800.read_exact(&mut hdr_buf) => {
                        match read_result {
                            Ok(0) => {
                                if should_be_connected_4800.load(Ordering::SeqCst) {
                                    let _ = reconnect_tx_reader_4800.send(()).await;
                                }
                                break
                            },
                            Ok(_len) => {
                                let header = BinaryHeader::parse(&hdr_buf);

                                let mut payload = vec![0u8; header.payload_size as usize];
                                if let Err(e) = reader_4800.read_exact(&mut payload).await {
                                    eprintln!("Read error (4800): {:?}", e);
                                    break;
                                }

                                if let Some(tx) = pending_responses_reader_4800
                                    .lock()
                                    .unwrap()
                                    .remove(&(header.id as u32))
                                {
                                    let result = BinaryResult {
                                        data: payload.clone(),
                                        width: header.width,
                                        height: header.height,
                                    };
                                    let _ = tx.send(Ok(result));
                                } else {
                                    log::warn!("No pending response for ID {}: {:?}", header.id, hdr_buf);
                                }

                            },
                            Err(e) => {
                                eprintln!("Read error (4800): {:?}", e);
                                break;
                            }
                        }
                    }
                    _ = shutdown_reader_rx_4800.changed() => {
                        log::debug!("Reader 4800 task received shutdown");
                        break;
                    }
                }
            }
        });

        // Write loop for port 4800
        tokio::spawn(async move {
            let id_counter = AtomicU32::new(1);
            loop {
                tokio::select! {
                    Some(command) = rx_4800.recv() => {
                        match command {
                            ASIAirCommand::BinaryGet { method, params, tx } => {
                                let id = id_counter.fetch_add(1, Ordering::SeqCst);
                                let request = if let Some(params) = params {
                                    json!({
                                        "id": id,
                                        "method": method,
                                        "params": params,
                                    })
                                } else {
                                    json!({
                                        "id": id,
                                        "method": method,
                                    })
                                };
                                pending_responses_writer_4800.lock().unwrap().insert(id, tx);
                                let mut message = request.to_string();
                                message.push_str("\r\n");
                                if let Err(e) = writer_4800.write_all(message.as_bytes()).await {
                                    eprintln!("Write error (4800): {:?}", e);
                                }
                            }
                            _ => {
                                log::warn!("Unexpected command for port 4800: {:?}", command);
                            }
                        }
                    }
                    _ = shutdown_writer_rx_4800.changed() => {
                        log::debug!("Writer 4800 task received shutdown");
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    async fn reconnect(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut retries = 0;

        // First, shutdown existing connections and cleanup
        self.cleanup().await;

        while self.should_be_connected.load(Ordering::SeqCst) {
            log::info!("Attempting to connect to ASIAir...");

            match self.try_connect().await {
                Ok(_) => {
                    log::info!("Reconnected successfully to ASIAir.");
                    return Ok(());
                }
                Err(_) => {
                    log::warn!("Reconnect attempt {} failed", retries + 1);
                    retries += 1;
                    let backoff_duration = Duration::from_secs(2u64.pow(retries as u32)); // Exponential backoff
                    tokio::time::sleep(backoff_duration).await;
                }
            }
        }

        Err("Aborting reconnections because connection canceled".into())
    }

    // Cleanup function to terminate the previous loops and clear pending responses
    async fn cleanup(&mut self) {
        log::debug!("Cleaning up previous connections");

        // Kill existing read and write loops
        if let Some(shutdown_tx) = &self.shutdown_tx {
            let _ = shutdown_tx.send(()); // Notify the tasks to shut down
        }

        // Clear pending responses
        self.pending_responses.lock().unwrap().clear();

        // Disconnect and reset the state
        self.tx_4500 = None;
        self.tx_4700 = None;
        self.tx_4800 = None;
        self.shutdown_tx = None;
        self.connected.store(false, Ordering::SeqCst);
        let _ = self.connection_state_tx.send(false); // Notify disconnection
    }

    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::SeqCst)
    }

    pub fn subscribe_connection_state(&self) -> watch::Receiver<bool> {
        self.connection_state_tx.subscribe()
    }

    pub fn subscribe_camera_temperature(&self) -> watch::Receiver<f32> {
        self.camera_temperature_tx.subscribe()
    }

    pub fn subscribe_camera_state_change(&self) -> watch::Receiver<()> {
        self.camera_state_change_tx.subscribe()
    }

    pub fn subscribe_cooler_power(&self) -> watch::Receiver<i32> {
        self.cooler_power_tx.subscribe()
    }

    pub fn subscribe_camera_control_change(&self) -> watch::Receiver<()> {
        self.camera_control_change_tx.subscribe()
    }

    pub fn subscribe_exposure(&self) -> watch::Receiver<ExposureEvent> {
        self.exposure_tx.subscribe()
    }

    pub fn subscribe_pi_status(&self) -> watch::Receiver<PiStatusEvent> {
        self.pi_status_tx.subscribe()
    }

    pub fn subscribe_annotate(&self) -> watch::Receiver<AnnotateEvent> {
        self.annotate_tx.subscribe()
    }

    pub fn subscribe_plate_solve(&self) -> watch::Receiver<PlateSolveEvent> {
        self.plate_solve_tx.subscribe()
    }

    pub async fn rpc_request_4700(
        &self,
        method: &str,
        params: Option<Value>,
    ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        if !self.should_be_connected.load(Ordering::SeqCst) {
            return Err("Not connected".into());
        }
        if let Some(tx) = &self.tx_4700 {
            let (response_tx, response_rx) = oneshot::channel();
            let command = ASIAirCommand::Get {
                method: method.to_string(),
                params,
                tx: response_tx,
            };
            tx.send(command).await.unwrap();

            // Wait for the response with a timeout
            match tokio::time::timeout(self.cmd_timeout, response_rx).await {
                Ok(Ok(response)) => response,
                Ok(Err(_)) | Err(_) => Err("Failed to get response".into()),
            }
        } else {
            Err("Not connected".into())
        }
    }

    pub async fn rpc_request_4800(
        &self,
        method: &str,
        params: Option<Value>,
    ) -> Result<BinaryResult, Box<dyn std::error::Error + Send + Sync>> {
        if !self.should_be_connected.load(Ordering::SeqCst) {
            return Err("Not connected".into());
        }
        if let Some(tx) = &self.tx_4800 {
            let (response_tx, response_rx) = oneshot::channel();
            let command = ASIAirCommand::BinaryGet {
                method: method.to_string(),
                params,
                tx: response_tx,
            };
            tx.send(command).await.unwrap();

            // Wait for the response with a timeout for this port we have an longer timeout
            match tokio::time::timeout(self.binary_cmd_timeout, response_rx).await {
                Ok(Ok(result)) => result,
                Ok(Err(_)) | Err(_) => Err("Failed to get response".into()),
            }
        } else {
            Err("Not connected".into())
        }
    }

    /// Test the connection to the ASIAir device
    pub async fn test_connection(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let response = self.rpc_request_4700("test_connection", None).await;
        if let Ok(value) = response {
            if value.as_str() == Some("server connected!") {
                Ok(())
            } else {
                return Err("Connection test failed: unexpected response".into());
            }
        } else {
            response.map(|_| ()).map_err(|e| {
                log::debug!("Connection test failed: {}", e);
                e
            })
        }
    }

    pub async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.should_be_connected.load(Ordering::SeqCst) {
            return Err("Not connected".into());
        }

        // Send a sequence of commands to get to a known state
        let result = self.set_page(ASIAirPage::Preview).await?;

        Ok(result)
    }

    pub async fn set_page(
        &self,
        page: ASIAirPage,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let response = self
            .rpc_request_4700("set_page", Some(json!(vec![page.as_str()])))
            .await;
        if let Ok(value) = response {
            if value.as_i64() == Some(0) {
                Ok(())
            } else {
                return Err("unexpected response".into());
            }
        } else {
            response.map(|_| ()).map_err(|e| {
                log::debug!("set_page failed: {}", e);
                e
            })
        }
    }
}
