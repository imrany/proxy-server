use axum::{
    body::Body,
    extract::Request,
    http::{
        Method, StatusCode
    },
    response::{
        IntoResponse, Response
    },
    routing::{get},
    Router,
};

mod serve;
use serve::{
    serve_pac,
    index_page, 
    notfound_page
};

mod handlers;
use handlers::{
    users::{
        UserStatsState,
        get_user_stats,
    },
    connections::{
        ConnectionInfo,
        MonitoringState,
        get_connections,
        get_active_connections,
    },   
};

use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::upgrade::Upgraded;
use std::net::{SocketAddr, IpAddr};
use std::sync::Arc;
use std::time::Duration;
use dashmap::DashMap;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{RwLock, Semaphore};
use tower::Service;
use tower::ServiceExt;
use tower_http::{
    trace::{self, TraceLayer}
};
use hyper_util::rt::TokioIo;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};
use tracing::Level;
use chrono::Utc;

mod read_txt;
use read_txt::check_address_block;

// Configuration constants
const MAX_CONCURRENT_CONNECTIONS: usize = 1000;
const CONNECTION_TIMEOUT_SECS: u64 = 30;
const TUNNEL_TIMEOUT_SECS: u64 = 300;

// Optimized state types using DashMap for better concurrent performance
type OptimizedMonitoringState = Arc<DashMap<String, ConnectionInfo>>;
type OptimizedUserStatsState = Arc<DashMap<String, UserStats>>;

#[derive(Clone, Debug)]
struct UserStats {
    total_connections: u64,
    total_bytes: u64,
    blocked_connections: u64,
    last_seen: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone)]
struct AppState {
    monitoring_state: OptimizedMonitoringState,
    user_stats_state: OptimizedUserStatsState,
    connection_semaphore: Arc<Semaphore>,
    router: Router,
}

#[tokio::main]
async fn main() {
    // Initialize tracing with less verbose output to reduce CPU overhead
    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_target(false)
                .with_thread_ids(false) // Disable thread IDs for performance
                .with_level(true)
                .with_file(false) // Disable file info for performance
                .with_line_number(false) // Disable line numbers for performance
        )
        .init();

    // Initialize optimized state with DashMap
    let monitoring_state: OptimizedMonitoringState = Arc::new(DashMap::new());
    let user_stats_state: OptimizedUserStatsState = Arc::new(DashMap::new());
    let connection_semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_CONNECTIONS));

    // Convert to legacy state types for handlers (if needed)
    let legacy_monitoring_state: MonitoringState = Arc::new(RwLock::new(std::collections::HashMap::new()));
    let legacy_user_stats_state: UserStatsState = Arc::new(RwLock::new(std::collections::HashMap::new()));

    let monitoring_api = Router::new()
        .route("/connections", get(get_connections))
        .route("/active", get(get_active_connections))
        .with_state(legacy_monitoring_state.clone());

    let stats_api = Router::new()
        .route("/stats", get(get_user_stats))
        .with_state(legacy_user_stats_state.clone());

    let api_routes = Router::new()
        .merge(monitoring_api)
        .merge(stats_api);

    let pac_routes = Router::new()
        .route("/proxy.pac", get(serve_pac));

    let page_routes = Router::new()
        .route("/", get(index_page));
        
    let router = Router::new()
        .merge(page_routes)
        .nest("/pac", pac_routes)
        .nest("/api", api_routes)
        .fallback(notfound_page)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(
                    trace::DefaultMakeSpan::new()
                        .level(Level::WARN) // Reduce logging level for performance
                )
                .on_response(
                    trace::DefaultOnResponse::new().level(Level::WARN)
                ),
        );

    let app_state = AppState {
        monitoring_state,
        user_stats_state,
        connection_semaphore,
        router,
    };

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("🚀 Proxy server listening on {}", addr);
    tracing::info!("📊 Monitor endpoints:");
    tracing::info!("  - GET /api/connections - All connections");
    tracing::info!("  - GET /api/stats - Statistics");
    tracing::info!("  - GET /api/active - Active connections");
    tracing::info!("⚙️  Max concurrent connections: {}", MAX_CONCURRENT_CONNECTIONS);
   
    let listener = TcpListener::bind(addr).await.unwrap();

    // Use a worker pool approach instead of spawning unlimited tasks
    loop {
        let (stream, client_addr) = match listener.accept().await {
            Ok(pair) => pair,
            Err(e) => {
                tracing::error!("Failed to accept connection: {:?}", e);
                continue;
            }
        };

        // Acquire connection permit (rate limiting)
        let permit = match app_state.connection_semaphore.clone().try_acquire_owned() {
            Ok(permit) => permit,
            Err(_) => {
                tracing::warn!("🚫 Connection limit reached, rejecting connection from {}", client_addr.ip());
                // Immediately drop the connection if we're at capacity
                drop(stream);
                continue;
            }
        };

        let app_state = app_state.clone();
        let client_ip = client_addr.ip();

        tokio::spawn(async move {
            let _permit = permit; // Keep permit alive for the duration of the connection
            
            if let Err(e) = handle_connection(stream, client_ip, app_state).await {
                tracing::warn!("❌ Connection error from {}: {:?}", client_ip, e);
            }
        });
    }
}

async fn handle_connection(
    stream: TcpStream,
    client_ip: IpAddr,
    app_state: AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client_ip_str = client_ip.to_string();
    
    let tower_service = tower::service_fn(move |req: Request<_>| {
        let app_state = app_state.clone();
        let client_ip_str = client_ip_str.clone();
        let req = req.map(Body::new);

        async move {
            if req.method() == Method::CONNECT {
                proxy(req, app_state, client_ip_str).await
            } else {
                app_state.router.clone().oneshot(req)
                    .await
                    .map_err(|err| {
                        tracing::error!("Router service error: {:?}", err);
                        match err {}
                    })
            }
        }
    });

    let hyper_service = hyper::service::service_fn(move |request: Request<Incoming>| {
        tower_service.clone().call(request)
    });

    let io = TokioIo::new(stream);

    // Add connection timeout
    let serve_future = http1::Builder::new()
        .preserve_header_case(true)
        .title_case_headers(true)
        .serve_connection(io, hyper_service)
        .with_upgrades();

    tokio::time::timeout(
        Duration::from_secs(CONNECTION_TIMEOUT_SECS),
        serve_future
    ).await??;

    Ok(())
}

async fn proxy(
    req: Request, 
    app_state: AppState,
    client_ip: String
) -> Result<Response, hyper::Error> {
    let headers = req.headers().clone();
    let user_agent = headers.get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    if let Some(host_addr) = req.uri().authority().map(|auth| auth.to_string()) {
        let timestamp = Utc::now();
        
        // Check if address should be blocked (existing block list check)
        if check_address_block(&host_addr) {
            tracing::warn!("🚫 BLOCKED: {} attempting to connect to {}", client_ip, host_addr);
            
            // Update stats efficiently using DashMap
            update_user_stats_optimized(&app_state.user_stats_state, &client_ip, true).await;
            
            let conn_info = ConnectionInfo {
                client_ip: client_ip.clone(),
                target_host: host_addr.clone(),
                timestamp,
                user_agent: user_agent.clone(),
                bytes_sent: 0,
                bytes_received: 0,
                status: "blocked".to_string(),
                duration_ms: Some(0),
            };
            
            // Log connection efficiently
            let key = format!("{}_{}", client_ip, timestamp.timestamp_millis());
            app_state.monitoring_state.insert(key, conn_info);
            
            return Ok(Response::builder()
                .status(StatusCode::FORBIDDEN)
                .header("Content-Type", "text/html")
                .body(Body::from(format!(
                    r#"<!DOCTYPE html>
                    <html><head><title>Access Denied</title></head>
                    <body><h1>🚫 Access Denied</h1>
                    <p>Connection to <strong>{}</strong> blocked by policy.</p>
                    <p>Your IP: {}</p><p>Timestamp: {}</p></body></html>"#,
                    host_addr, client_ip, timestamp.format("%Y-%m-%d %H:%M:%S UTC")
                )))
                .unwrap());
        }

        tracing::info!("✅ ALLOWED: {} → {}", client_ip, host_addr);

        // Log connection attempt efficiently
        let conn_key = format!("{}_{}", client_ip, timestamp.timestamp_millis());
        let conn_info = ConnectionInfo {
            client_ip: client_ip.clone(),
            target_host: host_addr.clone(),
            timestamp,
            user_agent,
            bytes_sent: 0,
            bytes_received: 0,
            status: "active".to_string(),
            duration_ms: None,
        };
        
        app_state.monitoring_state.insert(conn_key.clone(), conn_info);
        update_user_stats_optimized(&app_state.user_stats_state, &client_ip, false).await;

        // Spawn tunnel task with timeout
        let monitoring_state = app_state.monitoring_state.clone();
        let user_stats_state = app_state.user_stats_state.clone();
        
        tokio::task::spawn(async move {
            match hyper::upgrade::on(req).await {
                Ok(upgraded) => {
                    let start_time = Utc::now();
                    
                    // Add timeout to tunnel operation
                    let tunnel_result = tokio::time::timeout(
                        Duration::from_secs(TUNNEL_TIMEOUT_SECS),
                        tunnel(upgraded, host_addr.clone())
                    ).await;
                    
                    match tunnel_result {
                        Ok(Ok((bytes_sent, bytes_received))) => {
                            let duration = Utc::now().signed_duration_since(start_time);
                            let duration_ms = duration.num_milliseconds().max(0) as u64;
                            
                            tracing::info!("✅ Tunnel completed: {} → {} | ⬆️ {} bytes ⬇️ {} bytes | ⏱️ {}ms", 
                                client_ip, host_addr, bytes_sent, bytes_received, duration_ms);
                            
                            // Update connection status efficiently
                            if let Some(mut conn) = monitoring_state.get_mut(&conn_key) {
                                conn.bytes_sent = bytes_sent;
                                conn.bytes_received = bytes_received;
                                conn.duration_ms = Some(duration_ms);
                                conn.status = "completed".to_string();
                            }
                            
                            // Update user stats efficiently
                            update_user_stats_bytes(&user_stats_state, &client_ip, bytes_sent + bytes_received).await;
                        }
                        Ok(Err(e)) => {
                            tracing::error!("❌ Tunnel error: {} → {} | Error: {}", client_ip, host_addr, e);
                            if let Some(mut conn) = monitoring_state.get_mut(&conn_key) {
                                conn.status = "failed".to_string();
                                conn.duration_ms = Some(0);
                            }
                        }
                        Err(_) => {
                            tracing::warn!("⏱️ Tunnel timeout: {} → {}", client_ip, host_addr);
                            if let Some(mut conn) = monitoring_state.get_mut(&conn_key) {
                                conn.status = "timeout".to_string();
                                conn.duration_ms = Some(TUNNEL_TIMEOUT_SECS * 1000);
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("❌ Upgrade error: {} → {} | Error: {}", client_ip, host_addr, e);
                    if let Some(mut conn) = monitoring_state.get_mut(&conn_key) {
                        conn.status = "failed".to_string();
                        conn.duration_ms = Some(0);
                    }
                }
            }
        });

        Ok(Response::new(Body::empty()))
    } else {
        tracing::warn!("⚠️ Invalid CONNECT request from {}: {:?}", client_ip, req.uri());
        Ok((
            StatusCode::BAD_REQUEST,
            "CONNECT must be to a socket address",
        ).into_response())
    }
}

async fn tunnel(upgraded: Upgraded, addr: String) -> std::io::Result<(u64, u64)> {
    // Add connection timeout for the target server
    let mut server = tokio::time::timeout(
        Duration::from_secs(10),
        TcpStream::connect(&addr)
    ).await
    .map_err(|_| std::io::Error::new(std::io::ErrorKind::TimedOut, "Connection timeout"))??;
    
    let mut upgraded = TokioIo::new(upgraded);

    let (from_client, from_server) =
        tokio::io::copy_bidirectional(&mut upgraded, &mut server).await?;

    Ok((from_client, from_server))
}

// Optimized user stats update using DashMap
async fn update_user_stats_optimized(
    user_stats_state: &OptimizedUserStatsState,
    client_ip: &str,
    is_blocked: bool,
) {
    let now = Utc::now();
    
    user_stats_state.entry(client_ip.to_string())
        .and_modify(|stats| {
            stats.total_connections += 1;
            if is_blocked {
                stats.blocked_connections += 1;
            }
            stats.last_seen = now;
        })
        .or_insert(UserStats {
            total_connections: 1,
            total_bytes: 0,
            blocked_connections: if is_blocked { 1 } else { 0 },
            last_seen: now,
        });
}

// Optimized bytes update using DashMap
async fn update_user_stats_bytes(
    user_stats_state: &OptimizedUserStatsState,
    client_ip: &str,
    bytes: u64,
) {
    if let Some(mut stats) = user_stats_state.get_mut(client_ip) {
        stats.total_bytes += bytes;
    }
}