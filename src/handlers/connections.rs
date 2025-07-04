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

// pub async fn get_location_ipinfo(ip: String) -> String{
//     let url = format!("https://ipinfo.io/{}", ip);

//     #[derive(Deserialize)]
//     struct IpInfo{
//         ip: String,
//         hostname: String,
//         city: String,
//         region: String,
//         country: String,
//         loc: String,
//         org: String,
//         postal: String,
//         timezone: String,
//         // readme: Option<String>,
//         anycast:bool
//     }

//     let response = reqwest::get(&url)
//         .await;
            
//     match response {
//         Ok(resp) => {
//             if resp.status().is_success() {
//                 match resp.json::<IpInfo>().await {
//                     Ok(info) => {
//                         println!("IP: {}", info.ip);
//                         println!("Hostname: {}", info.hostname);
//                         println!("City: {}", info.city);
//                         println!("Region: {}", info.region);
//                         println!("Country: {}", info.country);
//                         println!("Location: {}", info.loc);
//                         println!("Organization: {}", info.org); 
//                         println!("Postal Code: {}", info.postal);
//                         println!("Timezone: {}", info.timezone);
//                         println!("Anycast: {}", info.anycast);
//                         info.loc
//                     },
//                     Err(e) => {
//                         eprintln!("Failed to parse JSON: {}", e);
//                         "".to_string()
//                     }
//                 }
//             } else {
//                 eprintln!("Failed to fetch IP info: {}", resp.status());
//                 "".to_string()
//             }
//         },
//         Err(e) => {
//             eprintln!("Error making request: {}", e);
//             "".to_string()
//         }
//     }
// }

