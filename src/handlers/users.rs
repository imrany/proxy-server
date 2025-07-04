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