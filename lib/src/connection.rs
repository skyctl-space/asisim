use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{mpsc, oneshot, watch};
use std::sync::atomic::{AtomicU32, Ordering};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

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
        ASIAir {
            addr,
            tx: None,
            pending_responses: Arc::new(Mutex::new(HashMap::new())),
            shutdown_tx: None,
        }
    }

    pub async fn connect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let stream = TcpStream::connect(self.addr.clone()).await?;
        let (mut reader, mut writer) = tokio::io::split(stream);

        let (tx, mut rx) = mpsc::channel::<ASIAirCommand>(32);
        self.tx = Some(tx);

        let (shutdown_tx, shutdown_rx) = watch::channel(());
        self.shutdown_tx = Some(shutdown_tx);
           
        let pending_responses_writer = Arc::clone(&self.pending_responses);
        let pending_responses_reader = Arc::clone(&self.pending_responses);
    
        let mut shutdown_reader_rx = shutdown_rx.clone();
        let mut shutdown_writer_rx = shutdown_rx.clone();

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
                                            let _ = tx.send(Ok(response));
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

    pub fn disconnect(&mut self) {
        if let Some(shutdown_tx) = &self.shutdown_tx {
            let _ = shutdown_tx.send(()); 
        }
   
        self.tx = None;
        self.shutdown_tx = None;
    }

    pub async fn test_connection(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(tx) = &self.tx {
            let (response_tx, response_rx) = oneshot::channel();
            let command = ASIAirCommand::Get {
                method: "test_connection".to_string(),
                params: vec![],
                tx: response_tx,
            };
            tx.send(command).await.unwrap();
            let response = response_rx.await?;
            if let Ok(_) = response {
                Ok(())
            } else {
                Err("Failed to test connection".into())
            }
        } else {
            Err("Not connected".into())
        }
    }
}