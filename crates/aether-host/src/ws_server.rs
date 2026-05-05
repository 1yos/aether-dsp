//! WebSocket server — thin client protocol.
//!
//! The host owns the graph. The UI sends intents, the host mutates and responds.
//!
//! All graph mutations go through GraphManager which holds the shared Scheduler.
//! The audio thread accesses the same Scheduler via try_lock().

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async, tungstenite::Message};

use crate::graph_manager::{GraphManager, Intent, Response};
use aether_core::{arena::NodeId, scheduler::Scheduler};
use aether_midi::MidiEngine;
use ringbuf::HeapCons;
use ringbuf::traits::{Consumer, Observer};

pub struct WsState {
    pub graph_manager: tokio::sync::Mutex<GraphManager>,
    /// Shared with the audio thread. Control thread locks this to mutate the graph.
    pub scheduler: Arc<Mutex<Scheduler>>,
    /// MIDI engine — shared so the WebSocket handler can expose port selection to the UI.
    pub midi_engine: Arc<Mutex<MidiEngine>>,
    /// Scope ring-buffer consumers, keyed by NodeId. Populated when a ScopeNode is added.
    pub scope_consumers: tokio::sync::Mutex<HashMap<NodeId, HeapCons<f32>>>,
}

pub async fn run(state: Arc<WsState>) {
    let listener = TcpListener::bind("127.0.0.1:9001")
        .await
        .expect("WS: failed to bind 127.0.0.1:9001");

    println!("WebSocket server listening on ws://127.0.0.1:9001");

    while let Ok((stream, addr)) = listener.accept().await {
        println!("WS: connection from {addr}");
        let state = Arc::clone(&state);
        tokio::spawn(handle_connection(stream, state));
    }
}

async fn handle_connection(stream: tokio::net::TcpStream, state: Arc<WsState>) {
    let ws = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("WS handshake error: {e}");
            return;
        }
    };

    let (mut tx, mut rx) = ws.split();

    // Send current snapshot immediately on connect
    {
        let snap = {
            let gm = state.graph_manager.lock().await;
            let sched = state.scheduler.lock().unwrap();
            gm.snapshot(&sched)
            // sched guard dropped here, before any await
        };
        if let Ok(json) = serde_json::to_string(&snap) {
            let _ = tx.send(Message::Text(json)).await;
        }
    }

    // 33ms scope polling interval (~30 fps)
    let mut scope_interval = tokio::time::interval(Duration::from_millis(33));

    loop {
        tokio::select! {
            _ = scope_interval.tick() => {
                // Apply modulation connections every frame (33ms) — reads LFO/envelope
                // output buffers and injects values into target parameters.
                {
                    let gm = state.graph_manager.lock().await;
                    if !gm.mod_connections.is_empty() {
                        if let Ok(mut sched) = state.scheduler.try_lock() {
                            gm.apply_modulation(&mut sched);
                        }
                    }
                }

                // Poll all scope consumers and send binary frames (64 samples = 256 bytes each)
                let mut consumers = state.scope_consumers.lock().await;
                for consumer in consumers.values_mut() {
                    if consumer.occupied_len() >= 64 {
                        let mut samples = [0.0f32; 64];
                        let n = consumer.pop_slice(&mut samples);
                        if n == 64 {
                            let mut bytes = Vec::with_capacity(256);
                            for s in &samples {
                                bytes.extend_from_slice(&s.to_le_bytes());
                            }
                            if let Err(e) = tx.send(Message::Binary(bytes)).await {
                                eprintln!("WS scope send error: {e}");
                                return;
                            }
                        }
                    }
                }
            }

            msg = rx.next() => {
                let msg = match msg {
                    Some(Ok(m)) => m,
                    Some(Err(e)) => {
                        eprintln!("WS recv error: {e}");
                        return;
                    }
                    None => return, // connection closed
                };

                if !msg.is_text() {
                    continue;
                }

                let text = msg.to_text().unwrap_or("");
                let response: Response = match serde_json::from_str::<Intent>(text) {
                    Ok(intent) => {
                        // Handle MIDI intents here — they need access to WsState::midi_engine
                        // which GraphManager::handle() does not have.
                        match intent {
                            Intent::MidiListPorts => {
                                let ports = MidiEngine::list_ports();
                                Response::MidiPorts { ports }
                            }
                            Intent::MidiConnect { port_index } => {
                                let mut engine = state.midi_engine.lock().unwrap();
                                match engine.connect_port(port_index) {
                                    Ok(()) => Response::Ack { command: "midi_connect".into() },
                                    Err(e) => Response::Error { message: e },
                                }
                            }
                            other => {
                                let mut gm = state.graph_manager.lock().await;
                                let response = {
                                    // Lock the scheduler — audio thread will output silence during this window
                                    let mut sched = state.scheduler.lock().unwrap();
                                    gm.handle(other, &mut sched)
                                    // sched guard dropped here, before any await
                                };
                                // Drain any new scope consumers produced by this intent
                                if !gm.scope_consumers.is_empty() {
                                    let mut sc = state.scope_consumers.lock().await;
                                    for (id, consumer) in gm.scope_consumers.drain() {
                                        sc.insert(id, consumer);
                                    }
                                }
                                response
                            }
                        }
                    }
                    Err(e) => Response::Error {
                        message: format!("Parse error: {e}"),
                    },
                };

                match serde_json::to_string(&response) {
                    Ok(json) => {
                        if let Err(e) = tx.send(Message::Text(json)).await {
                            eprintln!("WS send error: {e}");
                            return;
                        }
                    }
                    Err(e) => eprintln!("WS serialize error: {e}"),
                }
            }
        }
    }
}
