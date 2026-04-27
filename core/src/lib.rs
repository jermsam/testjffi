use std::sync::{Arc, Mutex};
use uniffi;
use anyhow::{anyhow, Result};
use iroh::{Endpoint, EndpointAddr, endpoint::presets};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::runtime::Runtime;

const ALPN: &[u8] = b"sad-chat/0";

struct State {
    endpoint: Option<Endpoint>,
    messages: Vec<String>,
}

#[derive(uniffi::Object)]
pub struct Core {
    rt: Runtime,
    state: Arc<Mutex<State>>,
}

#[uniffi::export]
impl Core {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {
            rt: Runtime::new().expect("failed to create tokio runtime"),
            state: Arc::new(Mutex::new(State {
                endpoint: None,
                messages: Vec::new(),
            })),
        }
    }

    pub fn greeting(&self) -> String {
        "Iroh core ready".to_string()
    }
    
    pub fn start(&self) -> Result<String, String> {
        self.rt.block_on(async {
            // Your async code here
            let endpoint = Endpoint::builder(presets::N0)
                .alpns(vec![ALPN.to_vec()])
                .bind()
                .await
                .map_err(|e| e.to_string())?;
            let addr = endpoint.addr().to_string();


            let recv_endpoint = endpoint.clone();
            let messages = self.state.clone();

            tokio::spawn(async move {
                receive_loop(recv_endpoint, messages).await;
            });

            self.state.lock().unwrap().endpoint = Some(endpoint);

            Ok(addr)
        })
    }

    pub fn send_message(&self, peer_addr: String, message: String) -> Result<(), String> {
        self.rt.block_on(async {
            let endpoint = {
                self.state
                    .lock()
                    .unwrap()
                    .endpoint
                    .clone()
                    .ok_or("Core.start() must be called first")?
            };

            let peer: EndpointAddr = peer_addr.parse::<EndpointAddr>()
                .map_err(|e| e.to_string())?;

            let conn = endpoint
                .connect(peer, ALPN)
                .await
                .map_err(|e| e.to_string())?;

            let mut stream = conn
                .open_uni()
                .await
                .map_err(|e| e.to_string())?;

            stream
                .write_all(message.as_bytes())
                .await
                .map_err(|e| e.to_string())?;

            stream.finish().map_err(|e| e.to_string())?;

            Ok(())
        })
    }

    pub fn drain_messages(&self) -> Vec<String> {
        let mut state = self.state.lock().unwrap();
        std::mem::take(&mut state.messages)
    }
}

async fn receive_loop(endpoint: Endpoint, state: Arc<Mutex<State>>) {
    while let Some(incoming) = endpoint.accept().await {
        let state = state.clone();

        tokio::spawn(async move {
            let Ok(conn) = incoming.await else {
                return;
            };

            let Ok(mut stream) = conn.accept_uni().await else {
                return;
            };

            let Ok(bytes) = stream.read_to_end(64 * 1024).await else {
                return;
            };

            let text = String::from_utf8_lossy(&bytes).to_string();

            state.lock().unwrap().messages.push(text);
        });
    }
}

uniffi::setup_scaffolding!();
