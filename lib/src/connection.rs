use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{mpsc, oneshot, watch};
use std::sync::atomic::{AtomicU32, AtomicBool, Ordering};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::time::Duration;

use super::ASIAir;

pub type Responder<T> = oneshot::Sender<Result<T, Box<dyn std::error::Error + Send + Sync>>>;

#[derive(Debug)]
pub enum ASIAirCommand {
    Get {
        method: String,
        params: Vec<Value>,
        tx: Responder<Value>,
    },
    Set {
        method: String,
        params: Vec<Value>,
    },
}


impl ASIAir {
    pub fn new(addr: SocketAddr) -> Self {
        let (connection_state_tx, _) = watch::channel(false);

        ASIAir {
            addr,
            tx: None,
            pending_responses: Arc::new(Mutex::new(HashMap::new())),
            shutdown_tx: None,
            reconnect_tx: None,
            should_be_connected: Arc::new(AtomicBool::new(false)),
            connected: Arc::new(AtomicBool::new(false)),
            connection_state_tx,
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
            while reconnect_rx.recv().await.is_some() && this.should_be_connected.load(Ordering::SeqCst) {
                this.reconnect().await.unwrap_or_else(|e| {
                    log::error!("Reconnection failed: {}", e);
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
        let stream = TcpStream::connect(self.addr.clone()).await?;
        let (mut reader, mut writer) = tokio::io::split(stream);

        let (tx, mut rx) = mpsc::channel::<ASIAirCommand>(32);
        self.tx = Some(tx.clone());

        let (shutdown_tx, shutdown_rx) = watch::channel(());
        self.shutdown_tx = Some(shutdown_tx);
           
        let pending_responses_writer = Arc::clone(&self.pending_responses);
        let pending_responses_reader = Arc::clone(&self.pending_responses);
    
        let mut shutdown_reader_rx = shutdown_rx.clone();
        let mut shutdown_writer_rx = shutdown_rx.clone();
        let mut shutdown_watchdog_rx = shutdown_rx.clone();
        let reconnect_tx = self.reconnect_tx.clone().unwrap();

        self.connected.store(true, Ordering::SeqCst);
        let _ = self.connection_state_tx.send(true); // Notify that we are connected

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
                            params: vec![],
                            tx: response_tx,
                        };
                        log::debug!("Feeding watchdog...");
                        if tx.send(command).await.is_ok() {
                            // Wait for the response with a timeout
                            match tokio::time::timeout(Duration::from_secs(2), response_rx).await {
                                Ok(Ok(response)) => {
                                    log::debug!("Watchdog response received: {:?}", response);
                                }
                                Ok(Err(_)) | Err(_) => {
                                    log::warn!("Connection to ASIAir lost or timed out");
                                    let _ = reconnect_tx.send(()).await;
                                }
                            }
                        }
                    }
                }
            }
        });


        // Read loop
        tokio::spawn(async move {
            
            let mut buf = [0u8; 2048];
            loop {
                tokio::select! {
                    read_result = reader.read(&mut buf) => {
                        match read_result {
                            Ok(0) => break, // EOF
                            Ok(len) => {
                                if let Ok(response) = serde_json::from_slice::<Value>(&buf[..len]) {
                                    if let Some(id) = response.get("id").and_then(|id| id.as_u64()) {
                                        if let Some(tx) = pending_responses_reader
                                            .lock()
                                            .unwrap()
                                            .remove(&(id as u32))
                                        {
                                            let _ = tx.send(Ok(response["result"].clone()));
                                        }
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
                                let request = json!({
                                    "id": id,
                                    "method": method,
                                    "params": params,
                                });
                                pending_responses_writer.lock().unwrap().insert(id, tx);
                                let mut message = request.to_string();
                                message.push_str("\r\n");
                                if let Err(e) = writer.write_all(message.as_bytes()).await {
                                    eprintln!("Write error: {:?}", e);
                                }
                            }
                            ASIAirCommand::Set { method, params } => {
                                let id = id_counter.fetch_add(1, Ordering::SeqCst);
                                let request = json!({
                                    "id": id,
                                    "method": method,
                                    "params": params,
                                });
                                let mut message = request.to_string();
                                message.push_str("\r\n");
                                if let Err(e) = writer.write_all(message.as_bytes()).await {
                                    eprintln!("Write error: {:?}", e);
                                }
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

        Err("Aborting reconnections because should not longer be connected".into())
    }

    // Cleanup function to terminate the previous loops and clear pending responses
    async fn cleanup(&mut self) {
        log::debug!("Cleaning up previous connections");

        // Kill existing read and write loops
        if let Some(shutdown_tx) = &self.shutdown_tx {
            let _ = shutdown_tx.send(());  // Notify the tasks to shut down
        }

        // Clear pending responses
        self.pending_responses.lock().unwrap().clear();

        // Disconnect and reset the state
        self.tx = None;
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

    /// Test the connection to the ASIAir device
    pub async fn test_connection(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(tx) = &self.tx {
            let (response_tx, response_rx) = oneshot::channel();
            let command = ASIAirCommand::Get {
                method: "test_connection".to_string(),
                params: vec![],
                tx: response_tx,
            };
            tx.send(command).await.unwrap();

            // Wait for the response with a timeout
            match tokio::time::timeout(std::time::Duration::from_secs(5), response_rx).await {
                Ok(Ok(_)) => {
                    Ok(())
                }
                Ok(Err(_)) | Err(_) => {
                    Err("Failed to test connection".into())
                }
            }
        } else {
            Err("Not connected".into())
        }
    }
}