use serde_json::{Value, json};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use axum::response::Json;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub client_ip: String,
    pub target_host: String,
    pub timestamp: DateTime<Utc>,
    pub user_agent: Option<String>,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub status: String, // "active", "completed", "blocked", "failed", "payment_required"
    pub duration_ms: Option<u64>,
}

pub type MonitoringState = Arc<RwLock<HashMap<String, Vec<ConnectionInfo>>>>;

pub async fn get_connections(
    axum::extract::State(state): axum::extract::State<MonitoringState>
) -> Json<Value> {
    let connections = state.read().await;
    Json(json!({
        "total_clients": connections.len(),
        "connections": *connections
    }))
}

pub 
async fn get_active_connections(
    axum::extract::State(state): axum::extract::State<MonitoringState>
) -> Json<Value> {
    let connections = state.read().await;
    let mut active_count = 0;
    let mut active_connections = Vec::new();
    
    for (client_ip, conn_list) in connections.iter() {
        for conn in conn_list {
            if conn.status == "active" {
                active_count += 1;
                active_connections.push(json!({
                    "client_ip": client_ip,
                    "target_host": conn.target_host,
                    "timestamp": conn.timestamp,
                    "user_agent": conn.user_agent
                }));
            }
        }
    }
    
    Json(json!({
        "active_connections": active_count,
        "connections": active_connections
    }))
}

pub 
async fn log_connection(state: MonitoringState, conn_info: ConnectionInfo) {
    let mut connections = state.write().await;
    connections
        .entry(conn_info.client_ip.clone())
        .or_insert_with(Vec::new)
        .push(conn_info);
}

pub async fn update_connection_status(
    state: MonitoringState,
    client_ip: &str,
    target_host: &str,
    bytes_sent: u64,
    bytes_received: u64,
    duration_ms: u64,
    status: &str
) {
    let mut connections = state.write().await;
    if let Some(conn_list) = connections.get_mut(client_ip) {
        for conn in conn_list.iter_mut().rev() {
            if conn.target_host == target_host && conn.status == "active" {
                conn.bytes_sent = bytes_sent;
                conn.bytes_received = bytes_received;
                conn.duration_ms = Some(duration_ms);
                conn.status = status.to_string();
                break;
            }
        }
    }
}