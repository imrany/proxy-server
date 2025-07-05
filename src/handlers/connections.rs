use serde_json::{Value, json};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap};
use axum::response::Json;

use crate::OptimizedMonitoringState;

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

pub async fn get_connections(
    axum::extract::State(state): axum::extract::State<OptimizedMonitoringState>
) -> Json<Value> {
    let mut client_map = HashMap::new();

    for entry in state.iter() {
        let (client_ip, conn_list) = entry.pair();
        let hosts: Vec<String> = vec![format!("{}:{}", conn_list.target_host, conn_list.user_agent.as_deref().unwrap_or("unknown"))];
        client_map.insert(client_ip.clone(), hosts);
    }

    Json(json!({
        "total_clients": client_map.len(),
        "connections": client_map
    }))
}

pub async fn get_active_connections(
    axum::extract::State(state): axum::extract::State<OptimizedMonitoringState>
) -> Json<Value> {
    let connections = &state;
    let mut active_count = 0;
    let mut active_connections = Vec::new();
    
    for entry in connections.iter() {
        let (client_ip, conn) = entry.pair();
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
    
    Json(json!({
        "active_connections": active_count,
        "connections": active_connections
    }))
}