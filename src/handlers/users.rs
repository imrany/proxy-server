use std::{collections::HashMap, sync::Arc};
use serde_json::{Value, json};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use axum:: response::Json;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStats {
    pub total_connections: u64,
    pub blocked_connections: u64,
    pub data_transferred: u64,
    pub unique_domains: Vec<String>,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}


// Global state for monitoring and payments
pub type UserStatsState = Arc<RwLock<HashMap<String, UserStats>>>;

pub async fn get_user_stats(
    axum::extract::State(stats): axum::extract::State<UserStatsState>
) -> Json<Value> {
    let user_stats = stats.read().await;
    Json(json!({
        "user_statistics": *user_stats
    }))
}

pub async fn update_user_stats(
    state: UserStatsState,
    client_ip: &str,
    target_host: &str,
    bytes_transferred: u64,
    is_blocked: bool
) {
    let mut stats = state.write().await;
    let user_stats = stats.entry(client_ip.to_string()).or_insert_with(|| UserStats {
        total_connections: 0,
        blocked_connections: 0,
        data_transferred: 0,
        unique_domains: Vec::new(),
        first_seen: Utc::now(),
        last_seen: Utc::now(),
    });
    
    user_stats.total_connections += 1;
    if is_blocked {
        user_stats.blocked_connections += 1;
    }
    user_stats.data_transferred += bytes_transferred;
    user_stats.last_seen = Utc::now();
    
    // Extract domain from target_host
    let domain = if let Some(colon_pos) = target_host.find(':') {
        target_host[..colon_pos].to_string()
    } else {
        target_host.to_string()
    };
    
    if !user_stats.unique_domains.contains(&domain) {
        user_stats.unique_domains.push(domain);
    }
}