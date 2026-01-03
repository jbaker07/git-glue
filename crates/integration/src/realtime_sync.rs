//! Production-ready real-time sync: WebSocket server for pushing updates to UI.

use crate::adapter::NormalizedEvent;
use crate::fact::Incident;
use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, Mutex, RwLock};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use tracing::{debug, error, info, warn};

// ============================================================================
// Message Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SyncMessage {
    /// New incident detected
    IncidentCreated {
        incident: Incident,
    },
    /// Incident updated with new evidence
    IncidentUpdated {
        incident_id: String,
        update: IncidentUpdate,
    },
    /// New event ingested
    EventIngested {
        event: NormalizedEvent,
    },
    /// Coverage status changed
    CoverageUpdate {
        primitive: String,
        sources: Vec<String>,
        confidence: f32,
    },
    /// Correlation result
    CorrelationResult {
        incident_id: String,
        contradictions: Vec<String>,
        gaps: Vec<String>,
    },
    /// Export completed
    ExportCompleted {
        incident_id: String,
        format: String,
        path: String,
    },
    /// Error notification
    Error {
        code: String,
        message: String,
    },
    /// Heartbeat
    Ping,
    Pong,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentUpdate {
    pub field: String,
    pub old_value: Option<String>,
    pub new_value: String,
    pub reason: String,
}

// ============================================================================
// Client Subscription
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub incident_ids: Option<Vec<String>>,
    pub event_types: Option<Vec<String>>,
    pub primitives: Option<Vec<String>>,
    pub all: bool,
}

impl Default for Subscription {
    fn default() -> Self {
        Self {
            incident_ids: None,
            event_types: None,
            primitives: None,
            all: true,
        }
    }
}

// ============================================================================
// Client Session
// ============================================================================

struct ClientSession {
    id: String,
    subscription: Subscription,
    tx: broadcast::Sender<SyncMessage>,
}

// ============================================================================
// Realtime Sync Server
// ============================================================================

pub struct RealtimeSyncServer {
    clients: Arc<RwLock<HashMap<String, ClientSession>>>,
    broadcast_tx: broadcast::Sender<SyncMessage>,
    incident_queue: Arc<Mutex<VecDeque<Incident>>>,
    event_queue: Arc<Mutex<VecDeque<NormalizedEvent>>>,
}

impl RealtimeSyncServer {
    pub fn new() -> Self {
        let (broadcast_tx, _) = broadcast::channel(1024);
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            broadcast_tx,
            incident_queue: Arc::new(Mutex::new(VecDeque::new())),
            event_queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    /// Start the WebSocket server
    pub async fn start(&self, addr: SocketAddr) -> Result<()> {
        let listener = TcpListener::bind(addr).await?;
        info!("WebSocket server listening on {}", addr);

        let clients = self.clients.clone();
        let broadcast_tx = self.broadcast_tx.clone();

        tokio::spawn(async move {
            while let Ok((stream, peer_addr)) = listener.accept().await {
                let clients = clients.clone();
                let broadcast_tx = broadcast_tx.clone();

                tokio::spawn(async move {
                    if let Err(e) =
                        handle_connection(stream, peer_addr, clients, broadcast_tx).await
                    {
                        error!("Connection error from {}: {}", peer_addr, e);
                    }
                });
            }
        });

        Ok(())
    }

    /// Broadcast a message to all subscribed clients
    pub async fn broadcast(&self, msg: SyncMessage) {
        if let Err(e) = self.broadcast_tx.send(msg.clone()) {
            debug!("No active receivers: {}", e);
        }
    }

    /// Push a new incident
    pub async fn push_incident(&self, incident: Incident) {
        self.incident_queue.lock().await.push_back(incident.clone());
        self.broadcast(SyncMessage::IncidentCreated { incident })
            .await;
    }

    /// Push a new event
    pub async fn push_event(&self, event: NormalizedEvent) {
        self.event_queue.lock().await.push_back(event.clone());
        self.broadcast(SyncMessage::EventIngested { event }).await;
    }

    /// Push coverage update
    pub async fn push_coverage_update(
        &self,
        primitive: String,
        sources: Vec<String>,
        confidence: f32,
    ) {
        self.broadcast(SyncMessage::CoverageUpdate {
            primitive,
            sources,
            confidence,
        })
        .await;
    }

    /// Push correlation result
    pub async fn push_correlation_result(
        &self,
        incident_id: String,
        contradictions: Vec<String>,
        gaps: Vec<String>,
    ) {
        self.broadcast(SyncMessage::CorrelationResult {
            incident_id,
            contradictions,
            gaps,
        })
        .await;
    }

    /// Push export completion
    pub async fn push_export_completed(&self, incident_id: String, format: String, path: String) {
        self.broadcast(SyncMessage::ExportCompleted {
            incident_id,
            format,
            path,
        })
        .await;
    }

    /// Get queued incidents (drains the queue)
    pub async fn drain_incidents(&self) -> Vec<Incident> {
        self.incident_queue.lock().await.drain(..).collect()
    }

    /// Get queued events (drains the queue)
    pub async fn drain_events(&self) -> Vec<NormalizedEvent> {
        self.event_queue.lock().await.drain(..).collect()
    }

    /// Get connected client count
    pub async fn client_count(&self) -> usize {
        self.clients.read().await.len()
    }
}

async fn handle_connection(
    stream: TcpStream,
    peer_addr: SocketAddr,
    clients: Arc<RwLock<HashMap<String, ClientSession>>>,
    broadcast_tx: broadcast::Sender<SyncMessage>,
) -> Result<()> {
    let ws_stream = accept_async(stream).await?;
    let (mut write, mut read) = ws_stream.split();

    let client_id = uuid::Uuid::new_v4().to_string();
    info!("Client connected: {} ({})", client_id, peer_addr);

    // Create client session
    let (client_tx, mut client_rx) = broadcast::channel(256);
    let session = ClientSession {
        id: client_id.clone(),
        subscription: Subscription::default(),
        tx: client_tx,
    };
    clients.write().await.insert(client_id.clone(), session);

    // Subscribe to broadcast messages
    let mut broadcast_rx = broadcast_tx.subscribe();

    // Handle incoming and outgoing messages
    loop {
        tokio::select! {
            // Handle incoming messages from client
            msg = read.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(sub) = serde_json::from_str::<Subscription>(&text) {
                            if let Some(session) = clients.write().await.get_mut(&client_id) {
                                session.subscription = sub;
                                debug!("Client {} updated subscription", client_id);
                            }
                        }
                    }
                    Some(Ok(Message::Ping(data))) => {
                        if write.send(Message::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        info!("Client {} disconnected", client_id);
                        break;
                    }
                    Some(Err(e)) => {
                        warn!("Client {} error: {}", client_id, e);
                        break;
                    }
                    _ => {}
                }
            }
            // Handle broadcast messages to send to client
            msg = broadcast_rx.recv() => {
                if let Ok(sync_msg) = msg {
                    let should_send = {
                        let clients_read = clients.read().await;
                        if let Some(session) = clients_read.get(&client_id) {
                            should_send_to_client(&session.subscription, &sync_msg)
                        } else {
                            false
                        }
                    };

                    if should_send {
                        let json = serde_json::to_string(&sync_msg)?;
                        if write.send(Message::Text(json)).await.is_err() {
                            break;
                        }
                    }
                }
            }
            // Handle client-specific messages
            msg = client_rx.recv() => {
                if let Ok(sync_msg) = msg {
                    let json = serde_json::to_string(&sync_msg)?;
                    if write.send(Message::Text(json)).await.is_err() {
                        break;
                    }
                }
            }
        }
    }

    // Clean up
    clients.write().await.remove(&client_id);
    info!("Client {} session ended", client_id);

    Ok(())
}

fn should_send_to_client(subscription: &Subscription, msg: &SyncMessage) -> bool {
    if subscription.all {
        return true;
    }

    match msg {
        SyncMessage::IncidentCreated { incident } => subscription
            .incident_ids
            .as_ref()
            .map(|ids| ids.contains(&incident.id))
            .unwrap_or(true),
        SyncMessage::IncidentUpdated { incident_id, .. } => subscription
            .incident_ids
            .as_ref()
            .map(|ids| ids.contains(incident_id))
            .unwrap_or(true),
        SyncMessage::EventIngested { event } => subscription
            .event_types
            .as_ref()
            .map(|types| types.contains(&event.event_type))
            .unwrap_or(true),
        SyncMessage::CoverageUpdate { primitive, .. } => subscription
            .primitives
            .as_ref()
            .map(|prims| prims.contains(primitive))
            .unwrap_or(true),
        _ => true,
    }
}

// ============================================================================
// Backward-compatible RealtimeSync (non-server mode)
// ============================================================================

/// Simple queue-based sync for non-WebSocket usage
pub struct RealtimeSync {
    pub incident_queue: Arc<Mutex<VecDeque<Incident>>>,
    pub event_queue: Arc<Mutex<VecDeque<NormalizedEvent>>>,
}

impl RealtimeSync {
    pub fn new() -> Self {
        Self {
            incident_queue: Arc::new(Mutex::new(VecDeque::new())),
            event_queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub async fn push_incident(&self, incident: Incident) {
        self.incident_queue.lock().await.push_back(incident);
    }

    pub async fn push_event(&self, event: NormalizedEvent) {
        self.event_queue.lock().await.push_back(event);
    }

    pub async fn poll_incidents(&self) -> Vec<Incident> {
        self.incident_queue.lock().await.drain(..).collect()
    }

    pub async fn poll_events(&self) -> Vec<NormalizedEvent> {
        self.event_queue.lock().await.drain(..).collect()
    }
}

impl Default for RealtimeSync {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for RealtimeSyncServer {
    fn default() -> Self {
        Self::new()
    }
}
